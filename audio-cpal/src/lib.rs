//! Optional audio support built directly on cpal.
//!
//! This crate is standalone: applications can use it without importing the
//! framework runtime or a graphics backend.

pub use cpal;
use cpal::traits::HostTrait;

pub struct AudioHost {
    host: cpal::Host,
}

impl AudioHost {
    pub fn default() -> Result<Self, cpal::HostUnavailable> {
        Ok(Self {
            host: cpal::default_host(),
        })
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
}
