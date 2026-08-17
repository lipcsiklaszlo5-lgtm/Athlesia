#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
FEATURES_DIR = os.path.join(PROJECT, "crates", "athlesia-features")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. FeatureVector bővítése
lib_path = os.path.join(FEATURES_DIR, "src", "lib.rs")
content = pathlib.Path(lib_path).read_text()

# Új mezők hozzáadása, ha még nincsenek
if "has_hole" not in content:
    content = content.replace(
        "pub struct FeatureVector {\n    pub object_count: u8,\n    pub color_counts: [u8; 4],\n    pub touching_pairs: u8,\n}",
        "pub struct FeatureVector {\n    pub object_count: u8,\n    pub color_counts: [u8; 4],\n    pub touching_pairs: u8,\n    pub has_hole: bool,\n    pub symmetric_h: bool,\n    pub symmetric_v: bool,\n}"
    )
    # extract_features függvény bővítése
    content = content.replace(
        "    FeatureVector {\n        object_count,\n        color_counts,\n        touching_pairs,\n    }",
        "    let has_hole = detect_hole(grid);\n    let (symmetric_h, symmetric_v) = detect_symmetry(grid);\n\n    FeatureVector {\n        object_count,\n        color_counts,\n        touching_pairs,\n        has_hole,\n        symmetric_h,\n        symmetric_v,\n    }"
    )
    # Segédfüggvények hozzáadása a fájl végére
    content += r'''

/// Lyukasság: van-e olyan 0-s cella, amit teljesen körbezárnak nem-0 cellák?
/// Egyszerű definíció: egy 0-s cella, amelynek mind a 4 közvetlen szomszédja nem-0.
fn detect_hole(grid: &Grid) -> bool {
    for i in 0..grid.cells.len() {
        for j in 0..grid.cells[0].len() {
            if grid.cells[i][j] == 0 {
                let up = i > 0 && grid.cells[i - 1][j] != 0;
                let down = i + 1 < grid.cells.len() && grid.cells[i + 1][j] != 0;
                let left = j > 0 && grid.cells[i][j - 1] != 0;
                let right = j + 1 < grid.cells[0].len() && grid.cells[i][j + 1] != 0;
                if up && down && left && right {
                    return true;
                }
            }
        }
    }
    false
}

/// Szimmetria: vízszintes és függőleges tengelyes tükrözés ellenőrzése.
fn detect_symmetry(grid: &Grid) -> (bool, bool) {
    let rows = grid.cells.len();
    let cols = grid.cells[0].len();
    let mut sym_h = true;
    let mut sym_v = true;

    for i in 0..rows {
        for j in 0..cols {
            if grid.cells[i][j] != grid.cells[i][cols - 1 - j] {
                sym_h = false;
            }
            if grid.cells[i][j] != grid.cells[rows - 1 - i][j] {
                sym_v = false;
            }
        }
    }
    (sym_h, sym_v)
}
'''
    write_file(lib_path, content)
    print("[INFO] FeatureVector bővítve lyukassággal és szimmetriával.")

# 2. Tesztek hozzáadása
write_file(os.path.join(FEATURES_DIR, "tests", "geometry_features_test.rs"), r'''
use athlesia_features::extract_features;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn detects_hole_in_object() {
    let grid = build_grid([
        [1, 1, 1, 0, 0],
        [1, 0, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let fv = extract_features(&grid);
    assert!(fv.has_hole);
}

#[test]
fn detects_symmetry_horizontal() {
    let grid = build_grid([
        [1, 2, 3, 2, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let fv = extract_features(&grid);
    assert!(fv.symmetric_h);
    assert!(!fv.symmetric_v);
}

#[test]
fn detects_symmetry_vertical() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [2, 0, 0, 0, 0],
        [3, 0, 0, 0, 0],
        [2, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
    ]);
    let fv = extract_features(&grid);
    assert!(fv.symmetric_v);
    assert!(!fv.symmetric_h);
}
''')

print("[INFO] Geometriai jellemzők tesztei hozzáadva.")

# 3. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-features"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Features geometriai tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Features geometriai tesztek zöldek.")

# 4. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Extend features with hole and symmetry detection"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
