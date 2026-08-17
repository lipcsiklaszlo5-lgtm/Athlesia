
use std::collections::HashMap;
use athlesia_types::{Grid, Color};
use athlesia_perception::{segment, touches, contains, distance_between, relative_direction};

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

    let mut color_counts = [0u8; 4];
    for &cell in &grid.cells {
        if cell.0 < 4 {
            color_counts[cell.0 as usize] += 1;
        }
    }

    let mut touching_pairs = 0;
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if touches(&objects[i], &objects[j]) {
                touching_pairs += 1;
            }
        }
    }

    let mut contains_pairs = 0;
    for i in 0..objects.len() {
        for j in (i + 1)..objects.len() {
            if contains(&objects[i], &objects[j]) || contains(&objects[j], &objects[i]) {
                contains_pairs += 1;
            }
        }
    }

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

    let mut dir_counts: HashMap<(i8, i8), usize> = HashMap::new();
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

fn detect_hole(grid: &Grid) -> bool {
    let width = grid.width as usize;
    let height = grid.height as usize;
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if grid.cells[idx] == Color(0) {
                let up = y > 0 && grid.cells[(y - 1) * width + x] != Color(0);
                let down = y + 1 < height && grid.cells[(y + 1) * width + x] != Color(0);
                let left = x > 0 && grid.cells[y * width + x - 1] != Color(0);
                let right = x + 1 < width && grid.cells[y * width + x + 1] != Color(0);
                if up && down && left && right {
                    return true;
                }
            }
        }
    }
    false
}

fn bounding_box_symmetry(grid: &Grid) -> (bool, bool) {
    let width = grid.width as usize;
    let height = grid.height as usize;

    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut has_object = false;

    for y in 0..height {
        for x in 0..width {
            if grid.cells[y * width + x] != Color(0) {
                has_object = true;
                if x < min_x { min_x = x; }
                if x > max_x { max_x = x; }
                if y < min_y { min_y = y; }
                if y > max_y { max_y = y; }
            }
        }
    }

    if !has_object {
        return (true, true);
    }

    let _bbox_width = max_x - min_x + 1;
    let _bbox_height = max_y - min_y + 1;

    let mut sym_h = true;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let mir_x = max_x - (x - min_x);
            if grid.cells[y * width + x] != grid.cells[y * width + mir_x] {
                sym_h = false;
                break;
            }
        }
        if !sym_h { break; }
    }

    let mut sym_v = true;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let mir_y = max_y - (y - min_y);
            if grid.cells[y * width + x] != grid.cells[mir_y * width + x] {
                sym_v = false;
                break;
            }
        }
        if !sym_v { break; }
    }

    (sym_h, sym_v)
}
