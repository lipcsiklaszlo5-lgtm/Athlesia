#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. WorldModel lib.rs bővítése evaluate_with_residual metódussal
p = pathlib.Path("crates/athlesia-world-model/src/lib.rs")
s = p.read_text()

# Új metódus beszúrása a record_prediction_error elé
anchor = '''    /// A predikciós hiba tárolása későbbi absztrakcióhoz.
    pub fn record_prediction_error(&mut self, error: PredictionError) {
        self.recent_errors.push(error);
    }
'''

new_method = '''    /// Egyetlen hívás, amely visszaadja a tudásállapotot és a reziduálist is.
    /// Ez a Phase 13 későbbi ciklusának alapművelete.
    pub fn evaluate_with_residual(
        &self,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
    ) -> (KnowledgeState, PredictionResidual) {
        let state = self.evaluate_prediction(action, prediction, observation);
        let residual = self.compute_prediction_residual(action, prediction, observation);
        (state, residual)
    }

    /// A predikciós hiba tárolása későbbi absztrakcióhoz.
    pub fn record_prediction_error(&mut self, error: PredictionError) {
        self.recent_errors.push(error);
    }
'''

if anchor not in s:
    print("[ERROR] record_prediction_error blokk nem található.")
    sys.exit(1)

s = s.replace(anchor, new_method)
p.write_text(s)
print("[1] WorldModel lib.rs frissítve: evaluate_with_residual hozzáadva.")

# 2. Új tesztfájl
test_code = r'''
use athlesia_world_model::{WorldModel, KnowledgeState, Observation, Prediction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};

fn grid_5x5_with_one_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

fn make_grid(x: usize, y: usize) -> Grid {
    grid_5x5_with_one_pixel(x, y, 1)
}

#[test]
fn evaluate_with_residual_explained() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(action.prim, action.params.clone())]);

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: prediction.state.clone() };

    let (state, residual) = wm.evaluate_with_residual(&action, &prediction, &observation);
    assert_eq!(state, KnowledgeState::Explained);
    assert_eq!(residual.mismatch_score, 0.0);
    assert!(residual.unexplained_features.is_empty());
}

#[test]
fn evaluate_with_residual_contradicted() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(action.prim, action.params.clone())]);

    let prediction = wm.predict(&initial, &action);
    let wrong_observation = Observation { state: initial.clone() }; // nem a predikció

    let (state, residual) = wm.evaluate_with_residual(&action, &prediction, &wrong_observation);
    assert_eq!(state, KnowledgeState::Contradicted);
    assert!(residual.mismatch_score > 0.0);
    assert_eq!(residual.unexplained_features, vec!["pixel_mismatch"]);
}

#[test]
fn evaluate_with_residual_uncertain() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let wm = WorldModel::new(initial.clone()); // nincs hipotézis

    let prediction = wm.predict(&initial, &action);
    let wrong_observation = Observation { state: initial.clone() };

    let (state, residual) = wm.evaluate_with_residual(&action, &prediction, &wrong_observation);
    assert_eq!(state, KnowledgeState::Uncertain);
    assert!(residual.mismatch_score > 0.0);
}

#[test]
fn evaluate_with_residual_out_of_model() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    // Van egy másik hipotézis (ReflectH), de nem az akcióra vonatkozik
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let prediction = wm.predict(&initial, &action);
    let wrong_observation = Observation { state: initial.clone() };

    let (state, residual) = wm.evaluate_with_residual(&action, &prediction, &wrong_observation);
    assert_eq!(state, KnowledgeState::OutOfModel);
    assert!(residual.mismatch_score > 0.0);
}
'''

write_file("crates/athlesia-world-model/tests/evaluate_with_residual_test.rs", test_code)
print("[2] evaluate_with_residual_test.rs létrehozva.")

# 3. WorldModel tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-world-model"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] WorldModel tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] WorldModel tesztek zöldek.")

# 4. Teljes workspace teszt futtatása
result = subprocess.run(
    ["cargo", "test", "--workspace", "--no-fail-fast"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Teljes workspace tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Teljes workspace tesztek zöldek.")

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 3: evaluate_with_residual combines KnowledgeState and PredictionResidual"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
