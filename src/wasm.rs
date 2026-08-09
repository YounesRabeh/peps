//! WebAssembly bindings for running Peps entirely in the browser.

use wasm_bindgen::prelude::*;

use crate::browser::run_source_for_browser;

/// Compile and execute Peps source, returning the IDE response as JSON.
#[wasm_bindgen]
pub fn run_peps(source: &str) -> String {
    serde_json::to_string(&run_source_for_browser(source))
        .expect("serializing a Peps browser response cannot fail")
}
