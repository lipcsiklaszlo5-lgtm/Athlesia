#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
SYNTH_DIR = os.path.join(PROJECT, "crates", "athlesia-synthesis")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-synthesis" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Synthesis crate létrehozása
os.makedirs(os.path.join(SYNTH_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(SYNTH_DIR, "tests"), exist_ok=True)

write_file(os.path.join(SYNTH_DIR, "Cargo.toml"), '''[package]
name = "athlesia-synthesis"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-executor = { path = "../athlesia-executor" }
''')

write_file(os.path.join(SYNTH_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Grid, PrimName, Params, Program, Budget};
use athlesia_executor::run_program;

/// Keresési primitívek listája. A Synthesis Engine innen építkezik.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveTemplate {
    Translate,
    ReflectH,
    ReflectV,
    Rotate90,
    Recolor,
}

/// Elemi transzformációk generálása a template-hez.
/// Itt csak a lehetséges paraméterek egy rögzített, korlátozott halmazát adjuk vissza.
/// A cél a determinisztikus, korlátos keresés, nem az összes lehetséges paraméter.
fn generate_primitives(template: PrimitiveTemplate) -> Vec<(PrimName, Params)> {
    match template {
        PrimitiveTemplate::Translate => {
            // 4 környező irány + identitás
            let mut v = Vec::new();
            for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0), (0, 0)] {
                v.push((PrimName::Translate, Params::Translate(dx, dy)));
            }
            v
        }
        PrimitiveTemplate::ReflectH => {
            vec![(PrimName::ReflectH, Params::None)]
        }
        PrimitiveTemplate::ReflectV => {
            vec![(PrimName::ReflectV, Params::None)]
        }
        PrimitiveTemplate::Rotate90 => {
            vec![(PrimName::Rotate90, Params::None)]
        }
        PrimitiveTemplate::Recolor => {
            // Néhány gyakori permutáció
            let mut v = Vec::new();
            for perm in [
                [1, 0, 2, 3],
                [2, 1, 0, 3],
                [3, 2, 1, 0],
                [1, 2, 3, 0],
            ] {
                v.push((PrimName::Recolor, Params::Recolor(perm)));
            }
            v
        }
    }
}

/// Egyszerű, 1 lépéses program szintézis.
/// Megpróbál minden sablont és az azokhoz tartozó primitíveket,
/// és visszaadja az első olyan programot, ami a kívánt kimenetet adja.
pub fn synthesize(input: &Grid, target: &Grid, templates: &[PrimitiveTemplate]) -> Option<Program> {
    for template in templates {
        for (prim, params) in generate_primitives(*template) {
            let program = vec![(prim, params)];
            let mut budget = Budget { max_steps: 1 };
            if let Ok(output) = run_program(&program, input, &mut budget) {
                if output == *target {
                    return Some(program);
                }
            }
        }
    }
    None
}
''')

# 3. Tesztek
write_file(os.path.join(SYNTH_DIR, "tests", "synthesis_test.rs"), r'''
use athlesia_synthesis::{synthesize, PrimitiveTemplate};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn synthesizes_translate_right() {
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

    let program = synthesize(&input, &target, &[
        PrimitiveTemplate::Translate,
        PrimitiveTemplate::ReflectH,
        PrimitiveTemplate::ReflectV,
        PrimitiveTemplate::Rotate90,
        PrimitiveTemplate::Recolor,
    ]);

    assert!(program.is_some());
}

#[test]
fn synthesizes_recolor() {
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let program = synthesize(&input, &target, &[
        PrimitiveTemplate::Recolor,
        PrimitiveTemplate::Translate,
    ]);

    assert!(program.is_some());
}

#[test]
fn fails_when_no_program_solves() {
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);

    let program = synthesize(&input, &target, &[
        PrimitiveTemplate::Translate,
        PrimitiveTemplate::ReflectH,
    ]);

    assert!(program.is_none());
}
''')

print("[INFO] Synthesis crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-synthesis"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Synthesis tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Synthesis tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-synthesis module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
