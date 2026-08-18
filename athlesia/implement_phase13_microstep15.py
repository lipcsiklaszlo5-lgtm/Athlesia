#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. world-model Cargo.toml bővítése perception függőséggel
p = pathlib.Path("crates/athlesia-world-model/Cargo.toml")
s = p.read_text()
if "athlesia-perception" not in s:
    if "[dependencies]" in s:
        s = s.replace(
            "[dependencies]",
            "[dependencies]\nathlesia-perception = { path = \"../athlesia-perception\" }",
            1,
        )
    else:
        s += "\n[dependencies]\nathlesia-perception = { path = \"../athlesia-perception\" }\n"
    p.write_text(s)
    print("[1] world-model Cargo.toml frissítve perception függőséggel.")
else:
    print("[1] world-model Cargo.toml már tartalmazza a perception függőséget.")

# 2. world-model lib.rs frissítése: compute_prediction_residual bővítése objektum-szintű jellemzőkkel
p = pathlib.Path("crates/athlesia-world-model/src/lib.rs")
s = p.read_text()

# Import hozzáadása
if "use athlesia_perception::segment;" not in s:
    s = s.replace(
        "use athlesia_types::{Grid, PrimName, Params, Program, Budget, Action};",
        "use athlesia_types::{Grid, PrimName, Params, Program, Budget, Action};\nuse athlesia_perception::segment;",
        1,
    )

# A compute_prediction_residual metódus cseréje
old_method = '''    pub fn compute_prediction_residual(
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
'''

new_method = '''    pub fn compute_prediction_residual(
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

        // Objektum-szintű eltérések detektálása a perception szegmentációval.
        // Csak azonos dimenziójú grideken értelmes.
        if prediction.state.width == observation.state.width
            && prediction.state.height == observation.state.height
        {
            let pred_objects = segment(&prediction.state);
            let obs_objects = segment(&observation.state);
            if pred_objects.len() != obs_objects.len() {
                unexplained_features.push("object_count_changed".to_string());
            }
        }

        PredictionResidual {
            expected_observation: Observation { state: prediction.state.clone() },
            observed_observation: observation.clone(),
            mismatch_score,
            unexplained_features,
        }
    }
'''

if old_method not in s:
    print("[ERROR] A compute_prediction_residual blokk nem található.")
    sys.exit(1)
s = s.replace(old_method, new_method)
p.write_text(s)
print("[2] world-model lib.rs frissítve: compute_prediction_residual objektum-szintű jellemzőkkel.")

# 3. Meglévő teszt módosítása: a partial mismatch teszt ne pontos listát várjon
p = pathlib.Path("crates/athlesia-world-model/tests/prediction_residual_test.rs")
s = p.read_text()
old_assert = "    assert_eq!(residual.unexplained_features, vec![\"pixel_mismatch\"]);"
new_assert = "    assert!(residual.unexplained_features.contains(&\"pixel_mismatch\".to_string()));"
if old_assert not in s:
    print("[ERROR] A régi assert nem található.")
    sys.exit(1)
s = s.replace(old_assert, new_assert)
p.write_text(s)
print("[3] prediction_residual_test.rs frissítve: rugalmas feature-ellenőrzés.")

# 4. Új teszt: object_count_changed megjelenése
test_code = r'''
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

#[test]
fn residual_includes_object_count_changed_when_segmentation_differs() {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let mut observation = grid_5x5_with_pixel(0, 0, 1);
    // Adjunk egy extra objektumot a megfigyeléshez, hogy a szegmensek száma eltérjen.
    observation.set(2, 2, Color(1));

    let wm = WorldModel::new(initial.clone());
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: initial.clone(), confidence: 0.5 };
    let observation = Observation { state: observation };

    let residual = wm.compute_prediction_residual(&action, &prediction, &observation);
    assert!(residual.unexplained_features.contains(&"object_count_changed".to_string()));
}
'''
write_file("crates/athlesia-world-model/tests/object_level_residual_test.rs", test_code)
print("[4] object_level_residual_test.rs létrehozva.")

# 5. World-model tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-world-model"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] World-model tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] World-model tesztek zöldek.")

# 6. Teljes workspace teszt
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

# 7. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 15: object-level features in PredictionResidual"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
