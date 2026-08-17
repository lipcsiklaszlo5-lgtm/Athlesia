
use athlesia_types::Grid;
use athlesia_perception::{segment, touches};

/// FeatureVector: ezeket a jellemzőket fogja később a MetaLearner használni.
/// A Hash miatt könnyen lehet HashMap kulcs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureVector {
    pub object_count: u8,
    pub color_counts: [u8; 4],
    pub touching_pairs: u8,
    pub has_hole: bool,
    pub symmetric_h: bool,
    pub symmetric_v: bool,
}

pub fn extract_features(grid: &Grid) -> FeatureVector {
    let objects = segment(grid);
    let object_count = objects.len() as u8;

    // Szín gyakoriságok
    let mut color_counts = [0u8; 4];
    for row in &grid.cells {
        for &cell in row {
            if cell < 4 {
                color_counts[cell as usize] += 1;
            }
        }
    }

    // Érintkező párok száma
    let mut touching_pairs = 0;
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if touches(&objects[i], &objects[j]) {
                touching_pairs += 1;
            }
        }
    }

    let has_hole = detect_hole(grid);
    let (symmetric_h, symmetric_v) = detect_symmetry(grid);

    FeatureVector {
        object_count,
        color_counts,
        touching_pairs,
        has_hole,
        symmetric_h,
        symmetric_v,
    }
}


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
