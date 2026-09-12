Rust program framework workspace

Purpose:
This repository provides a small foundation for general Rust programs. The core owns the winit application lifecycle and reusable runtime services. Audio and graphics are optional standalone crates, so an application can import only the pieces it needs or use any other crate directly.

Structure:
- core-api: winit application lifecycle, runtime, scheduler, services, resources, and events
- audio-cpal: optional standalone cpal audio host wrapper
- graphics-wgpu: optional standalone wgpu graphics context with web-sys logging on WASM
- demo-native: native winit application
- demo-web: wasm entry that drives the core runtime from JavaScript
- website/: static site + pkg directory (artifact output from wasm-pack)

Quick start (Windows):
- Install Rust: https://rustup.rs/
- Add wasm target: rustup target add wasm32-unknown-unknown
- Install wasm-pack for web builds: cargo install wasm-pack --locked
- Run native demo: cargo run -p demo-native
- Check all crates: cargo check --workspace
- Build web demo (produces website/pkg): wasm-pack build examples/demo-web --target web --out-dir website/pkg

Using the crates:

Add only the parts your application needs. When using this repository as a
local workspace, use these path dependencies from an application outside the
workspace:

```toml
[dependencies]
core-api = { path = "path/to/rust-framework/core-api" }
audio-cpal = { path = "path/to/rust-framework/audio-cpal" }
graphics-wgpu = { path = "path/to/rust-framework/graphics-wgpu" }
```

The package names use hyphens, while Rust imports use underscores:

```rust
use core_api::{run, Application, Runtime};
use audio_cpal::AudioHost;
use graphics_wgpu::GraphicsContext;
```

The crates are independent, so an application can import just one crate. For
example, a core-only application can use the winit-based lifecycle and runtime:

```rust
use core_api::{run, Application, Runtime};

struct App {
	runtime: Runtime,
}

impl Application for App {}

fn main() -> Result<(), winit::error::EventLoopError> {
	run(App { runtime: Runtime::new() })
}
```

```toml
[dependencies]
core-api = { path = "path/to/rust-framework/core-api" }
winit = "0.30"
```

For audio, create the default cpal host and inspect its devices:

```rust
let audio = audio_cpal::AudioHost::default();
let stream = audio.output_stream(|samples| {
	samples.fill(0.0);
})?;
stream.play()?;
// Keep `stream` alive while audio should continue playing.
```

The callback runs on cpal's audio thread and must remain real-time friendly.

For graphics, pass an `Arc<Window>` to initialize the adapter, device, queue,
and surface:

```rust
let graphics = pollster::block_on(graphics_wgpu::GraphicsContext::new(window))?;
graphics.render_clear(graphics_wgpu::wgpu::Color::BLACK)?;
```

CI: GitHub Actions builds the native workspace and the web demo. Pushes to
`main` publish `website/` to GitHub Pages using the repository's Pages
deployment environment. In the repository settings, set **Pages > Build and
deployment > Source** to **GitHub Actions**.
