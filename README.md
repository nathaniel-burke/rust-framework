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
- Build web demo (produces website/pkg): wasm-pack build demo-web --target web --out-dir website/pkg

CI: GitHub Actions workflow builds native + wasm and deploys website/ to GitHub Pages.
