use athlesia_integrated_cognitive_agent::custom_mechanism::{run_custom, CustomChallengeSpec};
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

/// Run a user-supplied bounded mechanism without exposing the environment
/// implementation to the browser-side caller.  A valid but ambiguous or
/// topologically incompatible challenge returns a machine-readable ABSTAIN
/// or REFUSED report rather than being converted into a success.
#[wasm_bindgen]
pub fn run_custom_challenge_json(spec_json: String) -> Result<String, JsValue> {
    let spec =
        CustomChallengeSpec::from_json(&spec_json).map_err(|error| JsValue::from_str(&error))?;
    let report = run_custom(spec).map_err(|error| JsValue::from_str(&error))?;
    Ok(report.jsonl().to_owned())
}

#[cfg(test)]
mod tests {
    use super::{run_custom_challenge_json, WasmFreshWorldSession};

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

    #[test]
    fn custom_json_bridge_returns_machine_readable_result() {
        let json = r#"{
          "name":"custom clamp",
          "seed":42,
          "source":{"action":"press","effect":"open","nodes":[
            {"id":"press","kind":"operation","type_tag":10},
            {"id":"pressure","kind":"state_field","type_tag":11},
            {"id":"latch","kind":"state_field","type_tag":12},
            {"id":"open","kind":"state_field","type_tag":13}],
            "edges":[
              {"from":"press","to":"pressure","kind":"requires"},
              {"from":"press","to":"latch","kind":"requires"},
              {"from":"press","to":"open","kind":"writes"}]},
          "source_examples":[
            {"before":{"pressure":true,"latch":true,"open":false},"after":{"pressure":true,"latch":true,"open":true}},
            {"before":{"pressure":true,"latch":true,"open":false},"after":{"pressure":true,"latch":true,"open":true}},
            {"before":{"pressure":true,"latch":true,"open":false},"after":{"pressure":true,"latch":true,"open":true}}],
          "target":{"action":"fire_grip","effect":"jaws_closed","nodes":[
            {"id":"fire_grip","kind":"operation","type_tag":10},
            {"id":"hydraulic_ok","kind":"state_field","type_tag":11},
            {"id":"safety_pin","kind":"state_field","type_tag":12},
            {"id":"jaws_closed","kind":"state_field","type_tag":13}],
            "edges":[
              {"from":"fire_grip","to":"hydraulic_ok","kind":"requires"},
              {"from":"fire_grip","to":"safety_pin","kind":"requires"},
              {"from":"fire_grip","to":"jaws_closed","kind":"writes"}]},
          "holdout":{"action":"engage","effect":"locked","nodes":[
            {"id":"engage","kind":"operation","type_tag":10},
            {"id":"line_live","kind":"state_field","type_tag":11},
            {"id":"interlock_clear","kind":"state_field","type_tag":12},
            {"id":"locked","kind":"state_field","type_tag":13}],
            "edges":[
              {"from":"engage","to":"line_live","kind":"requires"},
              {"from":"engage","to":"interlock_clear","kind":"requires"},
              {"from":"engage","to":"locked","kind":"writes"}]},
          "hidden_guard":{"target":["hydraulic_ok"],"holdout":["line_live"]}
        }"#;
        let output = run_custom_challenge_json(json.to_owned()).expect("custom bridge");
        assert!(output.contains("HOLDOUT_VERIFIED"), "{output}");
    }
}
