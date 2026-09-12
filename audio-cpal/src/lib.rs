//! Low-level audio support built directly on cpal.

pub use cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::fmt;
use std::sync::{Arc, Mutex};

pub struct AudioHost {
    host: cpal::Host,
}

impl AudioHost {
    pub fn default() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    pub fn host(&self) -> &cpal::Host {
        &self.host
    }

    pub fn default_output_device(&self) -> Option<cpal::Device> {
        self.host.default_output_device()
    }

    pub fn default_input_device(&self) -> Option<cpal::Device> {
        self.host.default_input_device()
    }

    /// Build an output stream that asks the callback to fill interleaved f32
    /// samples. The callback runs on cpal's real-time audio thread.
    pub fn output_stream<F>(&self, callback: F) -> Result<AudioStream, AudioError>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        let device = self
            .default_output_device()
            .ok_or(AudioError::NoOutputDevice)?;
        AudioStream::new(device, callback)
    }
}

pub struct AudioStream {
    stream: cpal::Stream,
}

impl AudioStream {
    pub fn new<F>(device: cpal::Device, callback: F) -> Result<Self, AudioError>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        let supported = device.default_output_config()?;
        let stream_config: cpal::StreamConfig = supported.clone().into();
        let callback = Arc::new(Mutex::new(callback));
        let error_callback = |error| eprintln!("audio stream error: {error}");

        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => {
                build_stream::<f32, F>(&device, &stream_config, callback, error_callback)?
            }
            cpal::SampleFormat::I16 => {
                build_stream::<i16, F>(&device, &stream_config, callback, error_callback)?
            }
            cpal::SampleFormat::U16 => {
                build_stream::<u16, F>(&device, &stream_config, callback, error_callback)?
            }
            format => return Err(AudioError::UnsupportedSampleFormat(format)),
        };

        Ok(Self { stream })
    }

    pub fn play(&self) -> Result<(), AudioError> {
        self.stream.play().map_err(AudioError::PlayStream)
    }

    pub fn pause(&self) -> Result<(), AudioError> {
        self.stream.pause().map_err(AudioError::PauseStream)
    }
}

fn build_stream<T, F>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    callback: Arc<Mutex<F>>,
    error_callback: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, AudioError>
where
    T: cpal::SizedSample + cpal::FromSample<f32>,
    F: FnMut(&mut [f32]) + Send + 'static,
{
    let mut samples = Vec::new();
    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _| {
            samples.resize(output.len(), 0.0);
            if let Ok(mut callback) = callback.lock() {
                callback(&mut samples);
            }
            for (output_sample, sample) in output.iter_mut().zip(samples.iter().copied()) {
                *output_sample = T::from_sample(sample);
            }
        },
        error_callback,
        None,
    )?;
    Ok(stream)
}

#[derive(Debug)]
pub enum AudioError {
    NoOutputDevice,
    DefaultOutputConfig(cpal::DefaultStreamConfigError),
    BuildStream(cpal::BuildStreamError),
    PlayStream(cpal::PlayStreamError),
    PauseStream(cpal::PauseStreamError),
    UnsupportedSampleFormat(cpal::SampleFormat),
}

impl fmt::Display for AudioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoOutputDevice => write!(formatter, "no default audio output device"),
            Self::DefaultOutputConfig(error) => write!(formatter, "default output config: {error}"),
            Self::BuildStream(error) => write!(formatter, "build audio stream: {error}"),
            Self::PlayStream(error) => write!(formatter, "play audio stream: {error}"),
            Self::PauseStream(error) => write!(formatter, "pause audio stream: {error}"),
            Self::UnsupportedSampleFormat(format) => {
                write!(formatter, "unsupported sample format: {format:?}")
            }
        }
    }
}

impl std::error::Error for AudioError {}

impl From<cpal::DefaultStreamConfigError> for AudioError {
    fn from(error: cpal::DefaultStreamConfigError) -> Self {
        Self::DefaultOutputConfig(error)
    }
}

impl From<cpal::BuildStreamError> for AudioError {
    fn from(error: cpal::BuildStreamError) -> Self {
        Self::BuildStream(error)
    }
}
