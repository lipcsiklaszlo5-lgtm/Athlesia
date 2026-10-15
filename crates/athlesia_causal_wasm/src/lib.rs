use athlesia_integrated_cognitive_agent::fresh_world_session::FreshWorldSession;
use wasm_bindgen::prelude::*;

/// The only stateful object exposed to the browser.
///
/// The browser receives rendered evidence snapshots and can advance the real
/// Rust session. It never receives the hidden guard itself.
#[wasm_bindgen]
pub struct WasmFreshWorldSession {
    inner: FreshWorldSession,
}

#[wasm_bindgen]
impl WasmFreshWorldSession {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        Self {
            inner: FreshWorldSession::new(u64::from(seed)),
        }
    }

    pub fn seed(&self) -> u32 {
        self.inner.seed() as u32
    }

    pub fn phase(&self) -> String {
        self.inner.phase().label().to_owned()
    }

    pub fn world_jsonl(&self) -> String {
        self.inner.world_jsonl()
    }

    pub fn sabotage(&mut self) -> Result<String, JsValue> {
        self.inner.sabotage().map_err(JsValue::from_str)
    }

    pub fn transfer_to_unknown(&mut self) -> Result<String, JsValue> {
        self.inner.transfer_to_unknown().map_err(JsValue::from_str)
    }

    pub fn reset(&mut self, seed: u32) {
        self.inner = FreshWorldSession::new(u64::from(seed));
    }
}

#[cfg(test)]
mod tests {
    use super::WasmFreshWorldSession;

    #[test]
    fn bridge_preserves_the_three_real_phase_boundaries() {
        let mut session = WasmFreshWorldSession::new(101);
        assert_eq!(session.phase(), "GENERATED");
        assert!(session.world_jsonl().contains("solver has not acted"));

        let sabotage = session.sabotage().expect("sabotage should be accepted");
        assert_eq!(session.phase(), "SABOTAGED");
        assert!(sabotage.contains("CONTRADICTION_FOUND"));
        assert!(!sabotage.contains("HOLDOUT_VERIFIED"));

        let transfer = session
            .transfer_to_unknown()
            .expect("transfer should be accepted");
        assert_eq!(session.phase(), "HOLDOUT_VERIFIED");
        assert!(transfer.contains("holdout_positive"));
        assert!(transfer.contains("holdout_negative"));
    }
}
