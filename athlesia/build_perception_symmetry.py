#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
PERCEPTION_DIR = os.path.join(PROJECT, "crates", "athlesia-perception")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. symmetry.rs létrehozása
symmetry_rs = r'''
use std::collections::HashSet;
use crate::{GameObject, bounding_box};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagonalAxis {
    Main,
    Anti,
}

/// Jaccard-egyezés két cellahalmaz között.
fn jaccard(a: &HashSet<(i8, i8)>, b: &HashSet<(i8, i8)>) -> f32 {
    let intersection = a.intersection(b).count();
    let union = a.len() + b.len() - intersection;
    if union == 0 {
        1.0
    } else {
        intersection as f32 / union as f32
    }
}

/// Vízszintes tengelyre vett tükörszimmetria (bal-jobb).
pub fn horizontal_symmetry(obj: &GameObject) -> f32 {
    let original: HashSet<(i8, i8)> = obj.cells.iter().map(|c| (c.x, c.y)).collect();
    let (min_x, min_y, max_x, max_y) = bounding_box(obj);

    let mirrored: HashSet<(i8, i8)> = original
        .iter()
        .map(|&(x, y)| (max_x + min_x - x, y))
        .collect();

    jaccard(&original, &mirrored)
}

/// Függőleges tengelyre vett tükörszimmetria (fent-lent).
pub fn vertical_symmetry(obj: &GameObject) -> f32 {
    let original: HashSet<(i8, i8)> = obj.cells.iter().map(|c| (c.x, c.y)).collect();
    let (min_x, min_y, max_x, max_y) = bounding_box(obj);

    let mirrored: HashSet<(i8, i8)> = original
        .iter()
        .map(|&(x, y)| (x, max_y + min_y - y))
        .collect();

    jaccard(&original, &mirrored)
}

/// Átlós szimmetria. Main = főátló, Anti = mellékátló.
pub fn diagonal_symmetry(obj: &GameObject, axis: DiagonalAxis) -> f32 {
    let original: HashSet<(i8, i8)> = obj.cells.iter().map(|c| (c.x, c.y)).collect();
    let (min_x, min_y, max_x, max_y) = bounding_box(obj);

    // Relatív koordináták a bbox-on belül
    let rel: HashSet<(usize, usize)> = original
        .iter()
        .map(|&(x, y)| ((x - min_x) as usize, (y - min_y) as usize))
        .collect();

    let w = (max_x - min_x + 1) as usize;
    let h = (max_y - min_y + 1) as usize;

    let mirrored: HashSet<(usize, usize)> = rel
        .iter()
        .map(|&(x, y)| {
            match axis {
                DiagonalAxis::Main => (y, x),          // főátló: x <-> y
                DiagonalAxis::Anti => (h - 1 - y, w - 1 - x), // anti-átló
            }
        })
        .collect();

    jaccard_usize(&rel, &mirrored)
}

/// Jaccard segéd usize koordinátákkal.
fn jaccard_usize(a: &HashSet<(usize, usize)>, b: &HashSet<(usize, usize)>) -> f32 {
    let intersection = a.intersection(b).count();
    let union = a.len() + b.len() - intersection;
    if union == 0 {
        1.0
    } else {
        intersection as f32 / union as f32
    }
}

/// 180 fokos elforgatási szimmetria.
pub fn rotational_symmetry_180(obj: &GameObject) -> f32 {
    let original: HashSet<(i8, i8)> = obj.cells.iter().map(|c| (c.x, c.y)).collect();
    let (min_x, min_y, max_x, max_y) = bounding_box(obj);

    let rotated: HashSet<(i8, i8)> = original
        .iter()
        .map(|&(x, y)| (max_x + min_x - x, max_y + min_y - y))
        .collect();

    jaccard(&original, &rotated)
}
'''
write_file(os.path.join(PERCEPTION_DIR, "src", "symmetry.rs"), symmetry_rs)
print("[INFO] symmetry.rs létrehozva.")

# 2. lib.rs modul regisztrálása
lib_path = os.path.join(PERCEPTION_DIR, "src", "lib.rs")
lib_content = pathlib.Path(lib_path).read_text()
if "pub mod symmetry;" not in lib_content:
    lib_content = lib_content.replace(
        "pub mod holes;",
        "pub mod holes;\npub mod symmetry;"
    )
    pathlib.Path(lib_path).write_text(lib_content)
    print("[INFO] symmetry modul hozzáadva a lib.rs-hez.")
else:
    print("[INFO] symmetry modul már létezik.")

# 3. Tesztek létrehozása
test_content = r'''
use athlesia_perception::symmetry::{
    horizontal_symmetry, vertical_symmetry, diagonal_symmetry, rotational_symmetry_180, DiagonalAxis
};
use athlesia_perception::segment;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn full_symmetry_for_solid_square() {
    let grid = build_grid([
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    let obj = &objects[0];

    assert_eq!(horizontal_symmetry(obj), 1.0);
    assert_eq!(vertical_symmetry(obj), 1.0);
    assert_eq!(diagonal_symmetry(obj, DiagonalAxis::Main), 1.0);
    assert_eq!(diagonal_symmetry(obj, DiagonalAxis::Anti), 1.0);
    assert_eq!(rotational_symmetry_180(obj), 1.0);
}

#[test]
fn horizontal_line_has_vertical_symmetry_but_not_horizontal() {
    let grid = build_grid([
        [1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    let obj = &objects[0];

    // Vízszintes vonal: függőleges szimmetriája van (fent-lent), de vízszintes nincs (bal-jobb)
    assert_eq!(vertical_symmetry(obj), 1.0);
    assert!(horizontal_symmetry(obj) < 1.0);
}

#[test]
fn l_shape_is_not_fully_symmetric() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    let obj = &objects[0];

    // L-alak se nem tükrös, se nem 180 fokos, se nem átlós
    assert!(horizontal_symmetry(obj) < 1.0);
    assert!(vertical_symmetry(obj) < 1.0);
    assert!(rotational_symmetry_180(obj) < 1.0);
    assert!(diagonal_symmetry(obj, DiagonalAxis::Main) < 1.0);
}

#[test]
fn cross_is_fully_symmetric() {
    let grid = build_grid([
        [0, 1, 0, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    let obj = &objects[0];

    assert_eq!(horizontal_symmetry(obj), 1.0);
    assert_eq!(vertical_symmetry(obj), 1.0);
    assert_eq!(diagonal_symmetry(obj, DiagonalAxis::Main), 1.0);
    assert_eq!(diagonal_symmetry(obj, DiagonalAxis::Anti), 1.0);
    assert_eq!(rotational_symmetry_180(obj), 1.0);
}
'''
write_file(os.path.join(PERCEPTION_DIR, "tests", "symmetry_test.rs"), test_content)
print("[INFO] symmetry_test.rs létrehozva.")

# 4. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-perception", "--test", "symmetry_test"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Perception symmetry tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Perception symmetry tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add symmetry features to perception module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
