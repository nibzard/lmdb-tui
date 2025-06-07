use wasm_bindgen::prelude::*;

/// Entry point for the WebAssembly build.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("lmdb-tui WebAssembly build initialised");
}
