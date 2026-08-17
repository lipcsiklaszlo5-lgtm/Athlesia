#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
PERCEPTION_DIR = os.path.join(PROJECT, "crates", "athlesia-perception")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. shape.rs létrehozása
shape_rs = r'''
use crate::{GameObject, bounding_box};

/// Az objektum mérete cellában.
pub fn cell_count(obj: &GameObject) -> usize {
    obj.cells.len()
}

/// Az objektum befoglaló téglalapjának szélessége és magassága.
pub fn bbox_dimensions(obj: &GameObject) -> (usize, usize) {
    let (min_x, min_y, max_x, max_y) = bounding_box(obj);
    ((max_x - min_x + 1) as usize, (max_y - min_y + 1) as usize)
}

/// Mennyire tölti ki az alakzat a befoglaló téglalapját? 1.0 = tömör.
pub fn fill_ratio(obj: &GameObject) -> f32 {
    let (w, h) = bbox_dimensions(obj);
    if w == 0 || h == 0 {
        return 0.0;
    }
    obj.cells.len() as f32 / (w * h) as f32
}

/// Mennyire vonalszerű az alakzat?
/// 0.0 = kompakt (pl. négyzet), 1.0 = tökéletes egyenes.
/// A cellakoordináták kovarianciamátrixának sajátérték-arányából számolva.
pub fn linearity(obj: &GameObject) -> f32 {
    let n = obj.cells.len() as f64;
    if n == 0.0 {
        return 0.0;
    }

    // Centroid
    let sum_x: f64 = obj.cells.iter().map(|c| c.x as f64).sum();
    let sum_y: f64 = obj.cells.iter().map(|c| c.y as f64).sum();
    let cx = sum_x / n;
    let cy = sum_y / n;

    // Kovariancia-mátrix elemei
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    let mut cov_xy = 0.0;
    for c in &obj.cells {
        let dx = c.x as f64 - cx;
        let dy = c.y as f64 - cy;
        var_x += dx * dx;
        var_y += dy * dy;
        cov_xy += dx * dy;
    }
    var_x /= n;
    var_y /= n;
    cov_xy /= n;

    // Sajátértékek zárt képlettel
    let trace = var_x + var_y;
    let det = var_x * var_y - cov_xy * cov_xy;
    let disc = (trace * trace / 4.0) - det;
    if disc < 0.0 {
        return 0.0; // numerikus hiba esetén semleges
    }
    let sqrt_disc = disc.sqrt();
    let lambda1 = trace / 2.0 + sqrt_disc;
    let lambda2 = trace / 2.0 - sqrt_disc;

    if lambda1 <= 0.0 {
        return 0.0;
    }

    // linearitás = 1 - (kisebb/nagyobb)
    // Ha lambda2 kicsi lambda1-hez képest, a pontok egy egyenes mentén helyezkednek el.
    let ratio = lambda2 / lambda1; // 0.0 és 1.0 között
    1.0 - ratio
}
'''
write_file(os.path.join(PERCEPTION_DIR, "src", "shape.rs"), shape_rs)
print("[INFO] shape.rs létrehozva.")

# 2. lib.rs modul regisztrálása
lib_path = os.path.join(PERCEPTION_DIR, "src", "lib.rs")
lib_content = pathlib.Path(lib_path).read_text()
if "pub mod shape;" not in lib_content:
    # A `use athlesia_types::{...}` sor után szúrjuk be
    lib_content = lib_content.replace(
        "use athlesia_types::{Color, Coord, Grid};",
        "use athlesia_types::{Color, Coord, Grid};\n\npub mod shape;"
    )
    pathlib.Path(lib_path).write_text(lib_content)
    print("[INFO] shape modul hozzáadva a lib.rs-hez.")
else:
    print("[INFO] shape modul már létezik.")

# 3. Tesztek létrehozása
test_content = r'''
use athlesia_perception::shape::{cell_count, bbox_dimensions, fill_ratio, linearity};
use athlesia_perception::segment;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn shape_metrics_for_solid_square() {
    let grid = build_grid([
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    let obj = &objects[0];

    assert_eq!(cell_count(obj), 9);
    assert_eq!(bbox_dimensions(obj), (3, 3));
    assert_eq!(fill_ratio(obj), 1.0);
    // A négyzet kompakt, a linearitás 0-hoz közeli
    assert!(linearity(obj) < 0.1, "linearitás négyzetre: {}", linearity(obj));
}

#[test]
fn shape_metrics_for_horizontal_line() {
    let grid = build_grid([
        [1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    let obj = &objects[0];

    assert_eq!(cell_count(obj), 5);
    assert_eq!(bbox_dimensions(obj), (5, 1));
    assert_eq!(fill_ratio(obj), 1.0);
    // Egy vízszintes vonal erősen vonalszerű, linearitás közel 1
    assert!(linearity(obj) > 0.9, "linearitás vonalra: {}", linearity(obj));
}

#[test]
fn shape_metrics_for_l_shape() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    let obj = &objects[0];

    assert_eq!(cell_count(obj), 4);
    assert_eq!(bbox_dimensions(obj), (2, 3));
    // L-alak nem tölti ki a bboxot
    assert!(fill_ratio(obj) < 1.0);
    // L-alak se nem teljesen vonal, se nem négyzet
    let lin = linearity(obj);
    assert!(lin > 0.1 && lin < 0.9, "linearitás L-alakra: {}", lin);
}
'''
write_file(os.path.join(PERCEPTION_DIR, "tests", "shape_test.rs"), test_content)
print("[INFO] shape_test.rs létrehozva.")

# 4. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-perception", "--test", "shape_test"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Perception shape tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Perception shape tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add shape metrics to perception module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
