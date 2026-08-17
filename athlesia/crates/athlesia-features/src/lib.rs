
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
    let (symmetric_h, symmetric_v) = bounding_box_symmetry(grid);

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

/// Szimmetria: az objektumok befoglaló téglalapján belül ellenőrizzük.
/// Ez eltolás-invariáns: ugyanaz a minta más pozícióban ugyanazt adja.
fn bounding_box_symmetry(grid: &Grid) -> (bool, bool) {
    let rows = grid.cells.len();
    let cols = grid.cells[0].len();

    // Keressük meg a legkisebb befoglaló téglalapot, ami minden nem-nulla cellát tartalmaz
    let mut min_i = rows;
    let mut max_i = 0;
    let mut min_j = cols;
    let mut max_j = 0;
    let mut has_object = false;

    for i in 0..rows {
        for j in 0..cols {
            if grid.cells[i][j] != 0 {
                has_object = true;
                if i < min_i { min_i = i; }
                if i > max_i { max_i = i; }
                if j < min_j { min_j = j; }
                if j > max_j { max_j = j; }
            }
        }
    }

    if !has_object {
        return (true, true); // üres grid mindig szimmetrikus
    }

    let _bbox_height = max_i - min_i + 1;
    let _bbox_width = max_j - min_j + 1;

    // Vízszintes szimmetria a bounding boxon belül
    let mut sym_h = true;
    for i in min_i..=max_i {
        for j in min_j..=max_j {
            let mirrored_j = max_j - (j - min_j);
            if grid.cells[i][j] != grid.cells[i][mirrored_j] {
                sym_h = false;
                break;
            }
        }
        if !sym_h { break; }
    }

    // Függőleges szimmetria a bounding boxon belül
    let mut sym_v = true;
    for i in min_i..=max_i {
        for j in min_j..=max_j {
            let mirrored_i = max_i - (i - min_i);
            if grid.cells[i][j] != grid.cells[mirrored_i][j] {
                sym_v = false;
                break;
            }
        }
        if !sym_v { break; }
    }

    (sym_h, sym_v)
}
