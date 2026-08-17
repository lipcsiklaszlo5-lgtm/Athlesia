
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
