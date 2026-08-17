#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
TYPES_LIB = os.path.join(PROJECT, "crates", "athlesia-types", "src", "lib.rs")
WM_DIR = os.path.join(PROJECT, "crates", "athlesia-world-model")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Action típus hozzáadása a types-hez, ha még nincs
types_content = pathlib.Path(TYPES_LIB).read_text()
if "pub struct Action" not in types_content:
    types_content += r'''

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Action {
    pub prim: PrimName,
    pub params: Params,
}
'''
    write_file(TYPES_LIB, types_content)
    print("[INFO] Action típus hozzáadva az athlesia-types-hez.")

# 2. World Model lib.rs bővítése predict-tel
wm_lib_path = os.path.join(WM_DIR, "src", "lib.rs")
wm_content = pathlib.Path(wm_lib_path).read_text()

if "pub fn predict" not in wm_content:
    # Predict függvény beszúrása a WorldModel impl blokkba
    predict_code = '''
    /// Előrejelzés egy akcióra.
    /// A konfidencia a hozzá tartozó hipotézis evidencia-arányából jön.
    /// Ha nincs ilyen hipotézis, semleges 0.5.
    pub fn predict(&self, state: &Grid, action: &Action) -> (Grid, f64) {
        let program = vec![(action.prim, action.params)];
        let mut budget = Budget { max_steps: 1 };
        let predicted_grid = run_program(&program, state, &mut budget).unwrap_or(*state);

        // Keresünk olyan hipotézist, amelynek programja pontosan ez az akció
        let mut confidence = 0.5;
        for hyp in &self.hypotheses {
            if hyp.program == program {
                if hyp.evidence_for + hyp.evidence_against > 0 {
                    confidence = (hyp.evidence_for as f64 + 1.0)
                        / (hyp.evidence_for as f64 + hyp.evidence_against as f64 + 2.0);
                }
                break;
            }
        }
        (predicted_grid, confidence)
    }
'''
    # Beszúrás az impl WorldModel blokkba, az update metódus után
    marker = "    pub fn update"
    insertion_point = wm_content.find(marker)
    if insertion_point == -1:
        print("[ERROR] Nem találom a WorldModel impl update markerét.")
        sys.exit(1)

    wm_content = wm_content[:insertion_point] + predict_code + wm_content[insertion_point:]
    write_file(wm_lib_path, wm_content)
    print("[INFO] predict interfész hozzáadva.")

# 3. Teszt hozzáadása
write_file(os.path.join(WM_DIR, "tests", "predict_test.rs"), r'''
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, PrimName, Params, Action};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn predict_translate_returns_expected_grid() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

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

    let (predicted, conf) = wm.predict(&input, &action);

    let expected = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    assert_eq!(predicted, expected);
    // Nincs még hipotézis, ezért semleges konfidencia
    assert_eq!(conf, 0.5);
}

#[test]
fn predict_uses_hypothesis_confidence() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    let action = Action {
        prim: PrimName::Translate,
        params: Params::Translate(1, 0),
    };

    // Adjunk hozzá egy hipotézist, ami pontosan ezt az akciót ismeri
    let program = vec![(action.prim, action.params)];
    wm.add_hypothesis(program.clone());

    // Megerősítjük a hipotézist néhány jó predikcióval
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

    let (_, conf) = wm.predict(&input, &action);
    // 3 sikeres predikció után a konfidencia magasabb 0.5-nél
    assert!(conf > 0.5);
}
''')

print("[INFO] World Model predict tesztek hozzáadva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-world-model"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] World Model predict tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] World Model predict tesztek zöldek.")

# 5. Git commit és push (szülőből)
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add WorldModel predict interface with Action"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
