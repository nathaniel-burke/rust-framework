//! Standalone graphics support for native and web applications.

pub use wgpu;

pub struct GraphicsContext {
    instance: wgpu::Instance,
}

impl GraphicsContext {
    pub fn new() -> Self {
        Self {
            instance: wgpu::Instance::default(),
        }
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }
}

#[cfg(target_arch = "wasm32")]
pub fn log(message: &str) {
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(message));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn log(message: &str) {
    println!("{message}");
}
