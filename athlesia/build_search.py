#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
SEARCH_DIR = os.path.join(PROJECT, "crates", "athlesia-search")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-search" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Search crate létrehozása
os.makedirs(os.path.join(SEARCH_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(SEARCH_DIR, "tests"), exist_ok=True)

write_file(os.path.join(SEARCH_DIR, "Cargo.toml"), '''[package]
name = "athlesia-search"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-executor = { path = "../athlesia-executor" }
''')

write_file(os.path.join(SEARCH_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Grid, PrimName, Params, Program, Budget};
use athlesia_executor::run_program;

/// Determinisztikus, korlátos mélységű programkeresés.
/// A cél: olyan programot találni, amely az inputból a cél gridet állítja elő.
/// A keresés a lehetséges primitívek kombinációit próbálja ki,
/// de nem az összeset, hanem egy rögzített, korlátozott paraméterhalmazt.
///
/// A keresés mélysége `max_depth` lépés. Minden lépésben minden primitív kipróbálható.
/// Determinisztikus, mert a primitívek listája és a bejárás sorrendje rögzített.

fn candidate_primitives() -> Vec<(PrimName, Params)> {
    let mut v = Vec::new();

    // Eltolások: 4 irány + identitás
    for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0), (0, 0)] {
        v.push((PrimName::Translate, Params::Translate(dx, dy)));
    }

    // Tükrözések
    v.push((PrimName::ReflectH, Params::None));
    v.push((PrimName::ReflectV, Params::None));

    // Forgás
    v.push((PrimName::Rotate90, Params::None));

    // Néhány színpermutáció
    for perm in [
        [1, 0, 2, 3],
        [2, 1, 0, 3],
        [3, 2, 1, 0],
        [1, 2, 3, 0],
        [0, 1, 2, 3],
    ] {
        v.push((PrimName::Recolor, Params::Recolor(perm)));
    }

    v
}

/// Rekurzív keresés: a `depth` hátralévő lépés számát jelzi.
/// A `current` a jelenlegi program, a `input` az eredeti rács, a `target` a cél.
fn dfs(
    input: &Grid,
    target: &Grid,
    max_depth: usize,
    depth: usize,
    current: &mut Program,
    budget: &mut Budget,
) -> Option<Program> {
    if depth == max_depth {
        // Kiértékeljük a teljes programot
        let mut b = Budget { max_steps: max_depth as u64 };
        if let Ok(output) = run_program(current, input, &mut b) {
            if output == *target {
                return Some(current.clone());
            }
        }
        return None;
    }

    for (prim, params) in candidate_primitives() {
        current.push((prim, params));
        if let Some(found) = dfs(input, target, max_depth, depth + 1, current, budget) {
            return Some(found);
        }
        current.pop();
    }
    None
}

/// Nyilvános kereső: iterál a mélységeken, és visszaadja az első találatot.
pub fn search(input: &Grid, target: &Grid, max_depth: usize) -> Option<Program> {
    for d in 1..=max_depth {
        let mut program = Vec::new();
        let mut budget = Budget { max_steps: d as u64 };
        if let Some(p) = dfs(input, target, d, 0, &mut program, &mut budget) {
            return Some(p);
        }
    }
    None
}
''')

# 3. Tesztek
write_file(os.path.join(SEARCH_DIR, "tests", "search_test.rs"), r'''
use athlesia_search::search;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn finds_single_step_translate() {
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

    let program = search(&input, &target, 1);
    assert!(program.is_some());
}

#[test]
fn finds_two_step_program() {
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    // Két eltolás jobbra: (1,0) + (1,0) => (2,0)
    let target = build_grid([
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let program = search(&input, &target, 2);
    assert!(program.is_some());
}

#[test]
fn returns_none_when_no_solution() {
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);
    let program = search(&input, &target, 2);
    assert!(program.is_none());
}
''')

print("[INFO] Search crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-search"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Search tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Search tesztek zöldek.")

# 5. Git commit és push a szülőkönyvtárból
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-search module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
