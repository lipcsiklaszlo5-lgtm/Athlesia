pub mod theater;

use athlesia_integrated_cognitive_agent::custom_mechanism::{run_custom, CustomChallengeSpec};
use athlesia_integrated_cognitive_agent::fresh_world_session::FreshWorldSession;
use athlesia_integrated_cognitive_agent::opaque_action_port::{
    run_opaque_custom, OpaqueInteractionSession,
};
use wasm_bindgen::prelude::*;

mod byod;
pub use byod::{
    run_byod_break_test_json, run_byod_transfer_json, verify_byod_break_test_json,
};

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

/// Execute the same JSON fixture through the strict opaque action-port
/// boundary.  The environment owns the world specification; the solver sees
/// only interaction logs and reconstructed observations.
#[wasm_bindgen]
pub fn run_opaque_challenge_json(spec_json: String) -> Result<String, JsValue> {
    let spec =
        CustomChallengeSpec::from_json(&spec_json).map_err(|error| JsValue::from_str(&error))?;
    let report = run_opaque_custom(spec).map_err(|error| JsValue::from_str(&error))?;
    Ok(report.jsonl().to_owned())
}

/// Stateful black-box session for the interactive browser demonstration.
/// The browser submits only the visible opaque action token; all environment
/// state and the hidden mechanism remain inside the Rust host.
#[wasm_bindgen]
pub struct WasmOpaqueInteractionSession {
    inner: OpaqueInteractionSession,
}

#[wasm_bindgen]
impl WasmOpaqueInteractionSession {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Result<WasmOpaqueInteractionSession, JsValue> {
        OpaqueInteractionSession::new(u64::from(seed))
            .map(|inner| Self { inner })
            .map_err(|error| JsValue::from_str(&error))
    }

    pub fn seed(&self) -> u32 {
        self.inner.seed() as u32
    }

    pub fn phase(&self) -> String {
        self.inner.phase().label().to_owned()
    }

    pub fn action_token(&self) -> String {
        self.inner.action_token().to_owned()
    }

    pub fn snapshot(&self) -> String {
        self.inner.observation_jsonl()
    }

    pub fn send_action(&mut self, token: String) -> Result<String, JsValue> {
        self.inner
            .send_action(&token)
            .map_err(|error| JsValue::from_str(&error))
    }

    pub fn sabotage(&mut self) -> Result<String, JsValue> {
        self.inner
            .sabotage()
            .map_err(|error| JsValue::from_str(&error))
    }

    pub fn probe(&mut self) -> Result<String, JsValue> {
        self.inner
            .probe()
            .map_err(|error| JsValue::from_str(&error))
    }

    pub fn verify_holdout(&mut self) -> Result<String, JsValue> {
        self.inner
            .verify_holdout()
            .map_err(|error| JsValue::from_str(&error))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        run_custom_challenge_json, run_opaque_challenge_json, WasmFreshWorldSession,
        WasmOpaqueInteractionSession,
    };

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

    #[test]
    fn opaque_json_bridge_reports_observation_only_solver_input() {
        let json = r#"{
          "name":"opaque clamp",
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
              {"from":"engage","to":"locked","kind":"writes"}]}
        }"#;
        let output = run_opaque_challenge_json(json.to_owned()).expect("opaque bridge");
        assert!(output.contains("HOLDOUT_VERIFIED"), "{output}");
        assert!(output.contains("observation_logs_only"), "{output}");
    }

    #[test]
    fn interactive_opaque_session_runs_the_real_ordered_cycle() {
        let mut session = WasmOpaqueInteractionSession::new(77).expect("session");
        let action = session.action_token();
        assert_eq!(session.phase(), "READY_FOR_ACTION");
        let first = session.send_action(action).expect("target action");
        assert!(first.contains("TARGET_TRIAL"), "{first}");
        let sabotaged = session.sabotage().expect("sabotage");
        assert!(sabotaged.contains("CONTRADICTION_FOUND"), "{sabotaged}");
        for _ in 0..8 {
            if session.phase() == "REVISED" {
                break;
            }
            session.probe().expect("probe");
        }
        assert_eq!(session.phase(), "REVISED");
        let result = session.verify_holdout().expect("holdout");
        assert_eq!(session.phase(), "HOLDOUT_VERIFIED");
        assert!(result.contains("\"holdout_positive\":true"), "{result}");
        assert!(result.contains("\"holdout_negative\":true"), "{result}");
        assert!(!result.contains("hidden_guard"), "{result}");
    }
}
