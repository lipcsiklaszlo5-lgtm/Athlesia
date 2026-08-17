
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
    let (min_x, _min_y, max_x, _max_y) = bounding_box(obj);

    let mirrored: HashSet<(i8, i8)> = original
        .iter()
        .map(|&(x, y)| (max_x + min_x - x, y))
        .collect();

    jaccard(&original, &mirrored)
}

/// Függőleges tengelyre vett tükörszimmetria (fent-lent).
pub fn vertical_symmetry(obj: &GameObject) -> f32 {
    let original: HashSet<(i8, i8)> = obj.cells.iter().map(|c| (c.x, c.y)).collect();
    let (_min_x, min_y, _max_x, max_y) = bounding_box(obj);

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
