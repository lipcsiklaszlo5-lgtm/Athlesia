
use athlesia_features::extract_features;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn detects_contains_relation_feature() {
    let grid = build_grid([
        [1, 1, 1, 0, 0],
        [1, 2, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let fv = extract_features(&grid);
    assert_eq!(fv.contains_pairs, 1);
    assert_eq!(fv.object_count, 2);
}

#[test]
fn detects_distance_category_touching() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let fv = extract_features(&grid);
    assert_eq!(fv.min_distance_category, 1);
}

#[test]
fn detects_dominant_direction() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 2],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let fv = extract_features(&grid);
    // A 2-es objektum jobbra és lefelé van az 1-eshez képest
    assert_eq!(fv.dominant_direction, (1, 1));
}
