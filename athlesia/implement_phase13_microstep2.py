#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. WorldModel lib.rs bővítése a compute_prediction_residual metódussal
p = pathlib.Path("crates/athlesia-world-model/src/lib.rs")
s = p.read_text()

# Módszert a learn_from_error után szúrjuk be, de a record_prediction_error elé.
method_anchor = '''    /// A predikciós hiba tárolása későbbi absztrakcióhoz.
    pub fn record_prediction_error(&mut self, error: PredictionError) {
        self.recent_errors.push(error);
    }
'''

new_method = '''    /// Strukturált predikciós reziduális előállítása.
    /// - mismatch_score: 0.0 = nincs eltérés, 1.0 = teljes eltérés vagy dimenzióeltérés.
    /// - unexplained_features: jelenleg alapvető "pixel_mismatch" jelzés, ha eltérés van.
    pub fn compute_prediction_residual(
        &self,
        _action: &Action,
        prediction: &Prediction,
        observation: &Observation,
    ) -> PredictionResidual {
        let mismatch_score = if prediction.state.width != observation.state.width
            || prediction.state.height != observation.state.height
        {
            1.0
        } else {
            let total = (prediction.state.width as usize) * (prediction.state.height as usize);
            if total == 0 {
                0.0
            } else {
                let mismatches = prediction
                    .state
                    .cells
                    .iter()
                    .zip(observation.state.cells.iter())
                    .filter(|(a, b)| a != b)
                    .count();
                mismatches as f64 / total as f64
            }
        };

        let mut unexplained_features = Vec::new();
        if mismatch_score > 0.0 {
            unexplained_features.push("pixel_mismatch".to_string());
        }

        PredictionResidual {
            expected_observation: Observation { state: prediction.state.clone() },
            observed_observation: observation.clone(),
            mismatch_score,
            unexplained_features,
        }
    }

    /// A predikciós hiba tárolása későbbi absztrakcióhoz.
    pub fn record_prediction_error(&mut self, error: PredictionError) {
        self.recent_errors.push(error);
    }
'''

if method_anchor not in s:
    print("[ERROR] record_prediction_error blokk nem található.")
    sys.exit(1)

s = s.replace(method_anchor, new_method)
p.write_text(s)
print("[1] WorldModel lib.rs frissítve: compute_prediction_residual hozzáadva.")

# 2. Új tesztfájl
test_code = r'''
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};

fn grid_5x5_with_one_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

fn grid_3x3_zeros() -> Grid {
    Grid { width: 3, height: 3, cells: vec![Color(0); 9] }
}

#[test]
fn residual_zero_for_exact_match() {
    let initial = grid_5x5_with_one_pixel(0, 0, 1);
    let wm = WorldModel::new(initial.clone());
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: initial.clone(), confidence: 0.5 };
    let observation = Observation { state: initial.clone() };

    let residual = wm.compute_prediction_residual(&action, &prediction, &observation);
    assert_eq!(residual.mismatch_score, 0.0);
    assert!(residual.unexplained_features.is_empty());
}

#[test]
fn residual_fraction_for_partial_mismatch() {
    let initial = grid_5x5_with_one_pixel(0, 0, 1);
    let expected = grid_5x5_with_one_pixel(1, 0, 1); // egy eltérő pixel
    let wm = WorldModel::new(initial.clone());
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: expected.clone(), confidence: 0.5 };
    let observation = Observation { state: initial.clone() }; // a tényleges megfigyelés más

    let residual = wm.compute_prediction_residual(&action, &prediction, &observation);
    // 25 pixelből 1 tér el, ha expected[1,0]=1 és initial[0,0]=1, a többi 0.
    assert!((residual.mismatch_score - 0.04).abs() < 0.0001);
    assert_eq!(residual.unexplained_features, vec!["pixel_mismatch"]);
}

#[test]
fn residual_one_for_dimension_mismatch() {
    let initial_5x5 = grid_5x5_with_one_pixel(0, 0, 1);
    let initial_3x3 = grid_3x3_zeros();
    let wm = WorldModel::new(initial_5x5.clone());
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: initial_5x5.clone(), confidence: 0.5 };
    let observation = Observation { state: initial_3x3 };

    let residual = wm.compute_prediction_residual(&action, &prediction, &observation);
    assert_eq!(residual.mismatch_score, 1.0);
    assert_eq!(residual.unexplained_features, vec!["pixel_mismatch"]);
}
'''

write_file("crates/athlesia-world-model/tests/prediction_residual_test.rs", test_code)
print("[2] prediction_residual_test.rs létrehozva.")

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
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 2: compute_prediction_residual with mismatch_score"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
