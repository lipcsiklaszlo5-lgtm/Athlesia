#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
WM_DIR = os.path.join(PROJECT, "crates", "athlesia-world-model")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Predict teszt figyelmeztetés javítása: az első tesztben a `mut` felesleges
test_path = os.path.join(WM_DIR, "tests", "predict_test.rs")
test_content = pathlib.Path(test_path).read_text()
test_content = test_content.replace(
    "fn predict_translate_returns_expected_grid() {\n    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));",
    "fn predict_translate_returns_expected_grid() {\n    let wm = WorldModel::new(build_grid([[0; 5]; 5]));"
)
write_file(test_path, test_content)
print("[INFO] Felesleges mut eltávolítva a predict tesztből.")

# 2. Uncertainty metódus hozzáadása a WorldModel impl blokkba
wm_lib_path = os.path.join(WM_DIR, "src", "lib.rs")
wm_content = pathlib.Path(wm_lib_path).read_text()

if "pub fn uncertainty" not in wm_content:
    uncertainty_code = '''
    /// Bizonytalanság: 1 - konfidencia. Determinisztikus, mert a konfidencia
    /// az evidencia-számlálókból jön, nem valószínűségi mintavételből.
    pub fn uncertainty(&self, state: &Grid, action: &Action) -> f64 {
        let (_, confidence) = self.predict(state, action);
        1.0 - confidence
    }
'''
    # Beszúrjuk az update metódus után (a predict metódus most már Action alapú)
    marker = "    pub fn update"
    insertion_point = wm_content.find(marker)
    if insertion_point == -1:
        print("[ERROR] Nem találom az update markert.")
        sys.exit(1)
    wm_content = wm_content[:insertion_point] + uncertainty_code + wm_content[insertion_point:]
    write_file(wm_lib_path, wm_content)
    print("[INFO] uncertainty metódus hozzáadva.")

# 3. Teszt hozzáadása
write_file(os.path.join(WM_DIR, "tests", "uncertainty_test.rs"), r'''
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, PrimName, Params, Action};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn uncertainty_initial_is_half() {
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));

    let action = Action {
        prim: PrimName::Translate,
        params: Params::Translate(1, 0),
    };

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    // Nincs hipotézis, a bizonytalanság 1 - 0.5 = 0.5
    assert_eq!(wm.uncertainty(&input, &action), 0.5);
}

#[test]
fn uncertainty_decreases_after_successful_updates() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    let action = Action {
        prim: PrimName::Translate,
        params: Params::Translate(1, 0),
    };

    let program = vec![(action.prim, action.params)];
    wm.add_hypothesis(program.clone());

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    wm.update(&input, &obs);
    wm.update(&input, &obs);
    wm.update(&input, &obs);

    let uncertainty = wm.uncertainty(&input, &action);
    assert!(uncertainty < 0.5);
}
''')
print("[INFO] uncertainty teszt hozzáadva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-world-model"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] World Model uncertainty tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] World Model uncertainty tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add uncertainty to WorldModel and clean warnings"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
