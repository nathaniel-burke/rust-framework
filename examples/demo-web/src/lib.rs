use core_api::Runtime;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmApp {
    runtime: Runtime,
}

#[wasm_bindgen]
pub fn init() -> WasmApp {
    console_error_panic_hook::set_once();
    WasmApp {
        runtime: Runtime::new(),
    }
}

#[wasm_bindgen]
impl WasmApp {
    pub fn tick(&mut self) {
        self.runtime.update(1.0 / 60.0);
    }
}
