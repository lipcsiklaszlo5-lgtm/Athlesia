#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
CORE_DIR = os.path.join(PROJECT, "crates", "athlesia-core")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-core" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Core crate létrehozása
os.makedirs(os.path.join(CORE_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(CORE_DIR, "tests"), exist_ok=True)

write_file(os.path.join(CORE_DIR, "Cargo.toml"), '''[package]
name = "athlesia-core"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-executor = { path = "../athlesia-executor" }
athlesia-features = { path = "../athlesia-features" }
athlesia-metalearner = { path = "../athlesia-metalearner" }
athlesia-verifier = { path = "../athlesia-verifier" }
athlesia-synthesis = { path = "../athlesia-synthesis" }
''')

write_file(os.path.join(CORE_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Grid, Program, PrimName, Params};
use athlesia_features::extract_features;
use athlesia_metalearner::MetaLearner;
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_synthesis::{synthesize, PrimitiveTemplate};

/// A Manhattan Kernel első tanuló magja.
/// Összeköti a jellemzőkinyerést, a MetaLearnert, a Verifiert és a Synthesis Engine-t.
#[derive(Debug, Default)]
pub struct CoreEngine {
    pub known_programs: Vec<Program>,
    pub meta: MetaLearner,
    pub verifier: Verifier,
}

impl CoreEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Megold egyetlen (input, target) párt. Ha a meglévő programok egyike sem működik,
    /// a Synthesis Engine megpróbál új programot generálni, verifikálja, és megtanulja.
    pub fn solve(&mut self, input: &Grid, target: &Grid) -> Option<Program> {
        let fv = extract_features(input);
        let ids: Vec<u64> = (0..self.known_programs.len() as u64).collect();

        // 1. Próbáljuk a már ismert programokat a MetaLearner rangsora szerint
        let ranked = self.meta.rank_in_context(fv, &ids);
        for id in ranked {
            let program = self.known_programs[id as usize].clone();
            let result = self.verifier.verify(&program, &[(input.clone(), target.clone())]);
            if result == VerificationResult::Accept {
                self.meta.record_success_in_context(fv, id);
                return Some(program);
            } else {
                self.meta.record_failure_in_context(fv, id);
            }
        }

        // 2. Ha nincs megfelelő, szintetizáljunk
        let templates = vec![
            PrimitiveTemplate::Translate,
            PrimitiveTemplate::ReflectH,
            PrimitiveTemplate::ReflectV,
            PrimitiveTemplate::Rotate90,
            PrimitiveTemplate::Recolor,
        ];

        if let Some(program) = synthesize(input, target, &templates) {
            // Verifikáljuk a szintetizált programot
            if self.verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
                let id = self.known_programs.len() as u64;
                self.known_programs.push(program.clone());
                self.meta.record_success_in_context(fv, id);
                return Some(program);
            }
        }

        None
    }
}
''')

# 3. Tesztek
write_file(os.path.join(CORE_DIR, "tests", "core_test.rs"), r'''
use athlesia_core::CoreEngine;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn learns_new_program_and_reuses_it() {
    let mut core = CoreEngine::new();

    // Két kezdő hipotézis, amelyek rosszak erre a feladatra
    core.known_programs.push(vec![(PrimName::ReflectH, Params::None)]);
    core.known_programs.push(vec![(PrimName::ReflectV, Params::None)]);

    // A valódi szabály: jobbra tolás (Translate(1,0))
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    // Első megoldás: szintetizálnia kell a Translate programot
    let program = core.solve(&input, &target).expect("Meg kell oldania a feladatot");
    assert_eq!(core.known_programs.len(), 3); // 2 kezdeti + 1 új

    // Második megoldás ugyanarra a feladatra: már ne szintetizáljon újat,
    // hanem a megtanultat használja
    let program2 = core.solve(&input, &target).expect("Másodjára is meg kell oldania");
    assert_eq!(core.known_programs.len(), 3); // nem nőtt a programok száma
    assert_eq!(program, program2); // ugyanazt a programot adta vissza
}

#[test]
fn fails_gracefully_when_no_solution_exists() {
    let mut core = CoreEngine::new();

    // Olyan target, amit a jelenlegi primitívek nem tudnak előállítani
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);

    // 9-es szín nem létezik (csak 0-3), tehát a szintézis nem találhatja meg
    let program = core.solve(&input, &target);
    assert!(program.is_none());
}
''')

print("[INFO] Core crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-core"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Core tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-core module with learning loop"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
