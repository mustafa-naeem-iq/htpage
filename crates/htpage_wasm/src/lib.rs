use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(source: &str) -> Result<String, JsError> {
    htpage_core::compile(source).map_err(|e| JsError::new(&e))
}
