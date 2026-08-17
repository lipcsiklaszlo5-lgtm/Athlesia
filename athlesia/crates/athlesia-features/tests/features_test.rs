
use athlesia_features::extract_features;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn extract_basic_features() {
    let grid = build_grid([
        [1, 1, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 2, 2],
        [0, 0, 0, 2, 0],
    ]);

    let fv = extract_features(&grid);

    assert_eq!(fv.object_count, 2);
    assert_eq!(fv.color_counts[1], 3);
    assert_eq!(fv.color_counts[2], 3);
    assert_eq!(fv.touching_pairs, 0);
}

#[test]
fn detect_touching_pair() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let fv = extract_features(&grid);

    assert_eq!(fv.object_count, 2);
    assert_eq!(fv.touching_pairs, 1);
}
