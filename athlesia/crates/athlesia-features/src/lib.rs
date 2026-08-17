
use athlesia_types::Grid;
use athlesia_perception::{segment, touches, contains, distance_between, relative_direction};

/// FeatureVector: ezeket a jellemzőket fogja később a MetaLearner használni.
/// A Hash miatt könnyen lehet HashMap kulcs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FeatureVector {
    pub object_count: u8,
    pub color_counts: [u8; 4],
    pub touching_pairs: u8,
    pub has_hole: bool,
    pub symmetric_h: bool,
    pub symmetric_v: bool,
    pub contains_pairs: u8,
    pub min_distance_category: u8,
    pub dominant_direction: (i8, i8),
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

    // Tartalmazási párok száma
    let mut contains_pairs = 0;
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if contains(&objects[i], &objects[j]) || contains(&objects[j], &objects[i]) {
                contains_pairs += 1;
            }
        }
    }

    // Minimális távolság kategória
    let mut min_distance_category = 0u8;
    if objects.len() >= 2 {
        let mut any_touching = false;
        let mut min_dist = f64::MAX;
        for i in 0..objects.len() {
            for j in (i + 1)..objects.len() {
                if touches(&objects[i], &objects[j]) {
                    any_touching = true;
                }
                let d = distance_between(&objects[i], &objects[j]);
                if d < min_dist {
                    min_dist = d;
                }
            }
        }
        if any_touching {
            min_distance_category = 1;
        } else if min_dist <= 2.0 {
            min_distance_category = 2;
        } else {
            min_distance_category = 3;
        }
    }

    // Domináns relatív irány az objektumpárok között
    let mut dir_counts: std::collections::HashMap<(i8, i8), usize> = std::collections::HashMap::new();
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            let dir = relative_direction(&objects[i], &objects[j]);
            *dir_counts.entry(dir).or_insert(0) += 1;
        }
    }
    let dominant_direction = dir_counts
        .into_iter()
        .max_by_key(|(dir, count)| (*count, std::cmp::Reverse(*dir)))
        .map(|(dir, _)| dir)
        .unwrap_or((0, 0));

    let has_hole = detect_hole(grid);
    let (symmetric_h, symmetric_v) = bounding_box_symmetry(grid);

    FeatureVector {
        object_count,
        color_counts,
        touching_pairs,
        has_hole,
        symmetric_h,
        symmetric_v,
        contains_pairs,
        min_distance_category,
        dominant_direction,
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
