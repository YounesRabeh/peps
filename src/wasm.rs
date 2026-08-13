//! WebAssembly bindings for running Peps entirely in the browser.

use wasm_bindgen::prelude::*;

use crate::browser::run_source_for_browser;

/// Compile and execute Peps source, returning the IDE response as JSON.
#[wasm_bindgen]
pub fn run_peps(source: &str, inputs_json: &str) -> String {
    let inputs = serde_json::from_str::<Vec<String>>(inputs_json).unwrap_or_default();
    serde_json::to_string(&run_source_for_browser(source, &inputs))
        .expect("serializing a Peps browser response cannot fail")
}
