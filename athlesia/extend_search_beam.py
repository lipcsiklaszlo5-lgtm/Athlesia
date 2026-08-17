#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
SEARCH_DIR = os.path.join(PROJECT, "crates", "athlesia-search")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# --- lib.rs frissítése beam search hozzáadásával ---
lib_path = os.path.join(SEARCH_DIR, "src", "lib.rs")
content = pathlib.Path(lib_path).read_text()

if "pub fn beam_search" not in content:
    # Segédfüggvény a beam search-höz: állapot-kiértékelés
    beam_code = r'''

/// A beam search a lehetséges programteret szélességben járja be,
/// de egyszerre csak a legjobb `beam_width` jelöltet tartja meg.
/// A "jóság" mértéke most egyszerű: hány cella egyezik a cél-griddel.
/// Ez a jövőben a MetaLearner tanult súlyaira cserélhető.
pub fn beam_search(
    input: &Grid,
    target: &Grid,
    max_depth: usize,
    beam_width: usize,
) -> Option<Program> {
    // A jelöltek: (program, eddigi kimenet, pontszám)
    let mut beam: Vec<(Program, Grid)> = Vec::new();

    // Kezdeti üres program
    let mut initial_program = Vec::new();
    let mut budget = Budget { max_steps: 0 };
    let initial_grid = match run_program(&initial_program, input, &mut budget) {
        Ok(g) => g,
        Err(_) => input.clone(),
    };
    beam.push((initial_program, initial_grid));

    for _depth in 0..max_depth {
        let mut next_beam: Vec<(Program, Grid)> = Vec::new();

        for (program, current_grid) in &beam {
            for (prim, params) in candidate_primitives() {
                let mut new_program = program.clone();
                new_program.push((prim, params));
                let mut b = Budget { max_steps: new_program.len() as u64 };
                if let Ok(new_grid) = run_program(&new_program, input, &mut b) {
                    next_beam.push((new_program, new_grid));
                }
            }
        }

        // Rendezés pontszám szerint: hány cella egyezik a target-tel
        next_beam.sort_by(|a, b| {
            let score_a = score_grid(&a.1, target);
            let score_b = score_grid(&b.1, target);
            score_b.cmp(&score_a)
        });

        // Csak a legjobb beam_width darab marad
        beam = next_beam.into_iter().take(beam_width).collect();

        // Ha valamelyik pontosan célba ért, visszaadjuk
        for (program, grid) in &beam {
            if *grid == *target {
                return Some(program.clone());
            }
        }
    }

    None
}

/// Pontszám: a targettel egyező cellák száma.
fn score_grid(grid: &Grid, target: &Grid) -> usize {
    let mut score = 0;
    for i in 0..GRID_SIZE {
        for j in 0..GRID_SIZE {
            if grid.cells[i][j] == target.cells[i][j] {
                score += 1;
            }
        }
    }
    score
}
'''
    # GRID_SIZE import szükséges
    content = content.replace(
        "use athlesia_types::{Grid, PrimName, Params, Program, Budget};",
        "use athlesia_types::{Grid, PrimName, Params, Program, Budget, GRID_SIZE};"
    )
    # A beam_search-t a fájl végére szúrjuk be a search függvény után
    content += beam_code
    write_file(lib_path, content)
    print("[INFO] beam_search hozzáadva a search crate-hez.")

# --- Teszt hozzáadása ---
test_content = r'''
use athlesia_search::{search, beam_search};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn beam_search_finds_same_solution_as_dfs() {
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let dfs_solution = search(&input, &target, 2);
    let beam_solution = beam_search(&input, &target, 2, 5);

    assert!(dfs_solution.is_some());
    assert!(beam_solution.is_some());
    assert_eq!(dfs_solution, beam_solution);
}
'''
write_file(os.path.join(SEARCH_DIR, "tests", "beam_search_test.rs"), test_content)
print("[INFO] beam_search teszt hozzáadva.")

# --- Teszt futtatása ---
result = subprocess.run(["cargo", "test", "-p", "athlesia-search"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Search beam_search tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Search beam_search tesztek zöldek.")

# --- Git commit és push ---
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add beam search to search engine"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
