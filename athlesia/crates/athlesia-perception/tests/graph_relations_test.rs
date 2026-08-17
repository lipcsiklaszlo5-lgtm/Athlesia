
use athlesia_perception::{perceive, segment, shape_fingerprint_hash};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn same_color_pairs_are_detected() {
    let grid = build_grid([
        [1, 0, 0, 0, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let output = perceive(None, &grid);
    assert_eq!(output.graph.same_color_pairs, vec![(0, 1)]);
}

#[test]
fn symmetry_pairs_are_detected() {
    let grid = build_grid([
        [1, 0, 0, 0, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let output = perceive(None, &grid);
    // Két egymással szimmetrikus (vízszintesen) objektum
    assert_eq!(output.graph.symmetry_pairs, vec![(0, 1)]);
}

#[test]
fn shape_fingerprint_hash_is_translation_invariant() {
    let g1 = build_grid([
        [1, 1, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let g2 = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 1, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let objs1 = segment(&g1);
    let objs2 = segment(&g2);
    assert_eq!(shape_fingerprint_hash(&objs1[0]), shape_fingerprint_hash(&objs2[0]));
}
