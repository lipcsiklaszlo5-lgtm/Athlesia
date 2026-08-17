
use athlesia_features::extract_features;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn horizontal_symmetry_is_translation_invariant() {
    // 3x3 minta: vízszintesen szimmetrikus
    let original = build_grid([
        [1, 2, 1, 0, 0],
        [3, 4, 3, 0, 0],
        [5, 6, 5, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    // Ugyanaz a minta eltolva jobbra és lefelé
    let shifted = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 2, 1],
        [0, 0, 3, 4, 3],
    ]);

    let fv_original = extract_features(&original);
    let fv_shifted = extract_features(&shifted);

    assert!(fv_original.symmetric_h);
    assert!(!fv_original.symmetric_v);
    assert_eq!(fv_original.symmetric_h, fv_shifted.symmetric_h);
    assert_eq!(fv_original.symmetric_v, fv_shifted.symmetric_v);
}

#[test]
fn vertical_symmetry_is_translation_invariant() {
    let original = build_grid([
        [1, 2, 3, 0, 0],
        [4, 5, 6, 0, 0],
        [1, 2, 3, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let shifted = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 1, 2, 3],
        [0, 0, 4, 5, 6],
        [0, 0, 1, 2, 3],
        [0, 0, 0, 0, 0],
    ]);

    let fv_original = extract_features(&original);
    let fv_shifted = extract_features(&shifted);

    assert!(fv_original.symmetric_v);
    assert!(!fv_original.symmetric_h);
    assert_eq!(fv_original.symmetric_v, fv_shifted.symmetric_v);
    assert_eq!(fv_original.symmetric_h, fv_shifted.symmetric_h);
}

#[test]
fn hole_detection_still_works() {
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
