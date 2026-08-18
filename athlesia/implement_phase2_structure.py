#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Kernel Cargo.toml: athlesia-structure függőség hozzáadása
p = pathlib.Path("crates/athlesia-kernel/Cargo.toml")
s = p.read_text()
if "athlesia-structure" not in s:
    s = s.replace(
        "[dependencies]\n",
        "[dependencies]\nathlesia-structure = { path = \"../athlesia-structure\" }\n"
    )
    write_file(p, s)
    print("[1] athlesia-structure hozzáadva a kernel függőségeihez.")
else:
    print("[1] athlesia-structure már szerepel.")

# 2. Cognitive modul bővítése strukturális elemzéssel
p = pathlib.Path("crates/athlesia-kernel/src/cognitive.rs")
s = p.read_text()

# Importok bővítése
if "use athlesia_structure::TargetDecomposer;" not in s:
    s = s.replace(
        "use athlesia_metalearner::MetaLearner;",
        "use athlesia_metalearner::MetaLearner;\nuse athlesia_structure::TargetDecomposer;"
    )
    s = s.replace(
        "use athlesia_types::Program;",
        "use athlesia_types::{Program, Grid};"
    )

# estimate függvény kiegészítése strukturális egyezéssel
old_estimate = """    /// Kompetenciabecslés kiszámítása.
    pub fn estimate(
        features: &FeatureVector,
        meta: &MetaLearner,
    ) -> CompetenceEstimate {
        // Egyszerű közelítés: a konfidencia a 0. hipotézisre.
        let conf = meta.priority_in_context(*features, 0) as f32;
        CompetenceEstimate {
            familiarity: conf,
            structural_match: 0.0,
            hypothesis_confidence: conf,
            predicted_search_cost: 100.0,
            expected_information_gain: 0.5,
        }
    }"""

new_estimate = """    /// Kompetenciabecslés kiszámítása.
    pub fn estimate(
        features: &FeatureVector,
        meta: &MetaLearner,
        input: &Grid,
        target: &Grid,
    ) -> CompetenceEstimate {
        // Konfidencia a 0. hipotézisre
        let conf = meta.priority_in_context(*features, 0) as f32;

        // Strukturális egyezés a Target Decomposer alapján
        let structural_match = if let Some(meta_grid) = TargetDecomposer.decompose_dimensions(input, target) {
            // Ha a dimenziók oszthatók, és van blokk-dekompozíció,
            // akkor erős strukturális egyezést feltételezünk.
            let block_count = (meta_grid.block_rows * meta_grid.block_cols) as f32;
            if block_count > 0.0 {
                1.0 / (1.0 + block_count) // minél több blokk, annál kisebb, de még mindig jel
            } else {
                0.0
            }
        } else {
            0.0
        };

        CompetenceEstimate {
            familiarity: conf,
            structural_match,
            hypothesis_confidence: conf,
            predicted_search_cost: 100.0,
            expected_information_gain: 0.5,
        }
    }"""

if old_estimate in s:
    s = s.replace(old_estimate, new_estimate)
    write_file(p, s)
    print("[2] Cognitive estimate függvény bővítve strukturális elemzéssel.")
else:
    print("[ERROR] Nem találtam az estimate függvényt a cognitive.rs-ben.")
    sys.exit(1)

# 3. Teszt frissítése: becslés új szignatúrához és strukturális felismerés
p = pathlib.Path("crates/athlesia-kernel/tests/cognitive_test.rs")
s = p.read_text()

# A régi estimate hívás cseréje
old_call = "let estimate = CognitiveController::estimate(&fv, &meta);"
new_call = (
    "let input = Grid::new(3, 3);\n"
    "    let target = Grid::new(9, 9);\n"
    "    let estimate = CognitiveController::estimate(&fv, &meta, &input, &target);"
)
if old_call in s:
    s = s.replace(old_call, new_call)

# Ellenőrzés: a structural_match legyen > 0.0, ha a dimenziók oszthatók
old_assert = "    assert!(estimate.hypothesis_confidence >= 0.0 && estimate.hypothesis_confidence <= 1.0);"
new_assert = (
    old_assert +
    "\n    assert!(estimate.structural_match > 0.0, "
    "\"Strukturális egyezésnek pozitívnak kell lennie 3x3->9x9 esetén.\");"
)
if old_assert in s:
    s = s.replace(old_assert, new_assert)

# Grid import hozzáadása, ha hiányzik
if "use athlesia_types::Grid;" not in s:
    s = s.replace(
        "use athlesia_types::{PrimName, Params, Program};",
        "use athlesia_types::{Grid, PrimName, Params, Program};"
    )

write_file(p, s)
print("[3] cognitive_test.rs frissítve.")

# 4. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel", "--test", "cognitive_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Phase 2 tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Phase 2 tesztek zöldek.")

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Add structural understanding to cognitive controller"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
