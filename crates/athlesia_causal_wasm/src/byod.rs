use athlesia_integrated_cognitive_agent::byod_causal_transfer::{
    create_byod_proof, run_byod_transfer, verify_byod_proof, ByodProofBundle, ByodTransferRequest,
};
use wasm_bindgen::prelude::*;

fn js_error(error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// Backwards-compatible report-only boundary.
#[wasm_bindgen]
pub fn run_byod_transfer_json(source_csv: String, target_csv: String) -> Result<String, JsValue> {
    let request = ByodTransferRequest::new(source_csv, target_csv);
    serde_json::to_string(&run_byod_transfer(&request)).map_err(js_error)
}

/// Run the exact bounded Rust adapter and return the inputs, result and replay digest.
#[wasm_bindgen]
pub fn run_byod_break_test_json(source_csv: String, target_csv: String) -> Result<String, JsValue> {
    serde_json::to_string(&create_byod_proof(ByodTransferRequest::new(
        source_csv, target_csv,
    )))
    .map_err(js_error)
}

/// Recompute a downloaded bundle through the same Rust code. No browser-side
/// status is trusted and no mock fallback exists at this boundary.
#[wasm_bindgen]
pub fn verify_byod_break_test_json(bundle_json: String) -> Result<String, JsValue> {
    let bundle: ByodProofBundle = serde_json::from_str(&bundle_json).map_err(js_error)?;
    serde_json::to_string(&verify_byod_proof(&bundle)).map_err(js_error)
}
