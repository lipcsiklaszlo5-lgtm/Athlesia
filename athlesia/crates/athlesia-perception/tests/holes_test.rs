
use athlesia_perception::holes::{hole_count, hole_sizes, max_hole_depth};
use athlesia_perception::segment;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn detects_single_hole_in_frame() {
    let grid = build_grid([
        [1, 1, 1, 1, 1],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
        [1, 1, 1, 1, 1],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    let obj = &objects[0];

    assert_eq!(hole_count(obj), 1);
    assert_eq!(hole_sizes(obj), vec![9]);
    assert_eq!(max_hole_depth(obj), 1);
}

#[test]
fn no_hole_in_solid_square() {
    let grid = build_grid([
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    let obj = &objects[0];

    assert_eq!(hole_count(obj), 0);
    assert!(hole_sizes(obj).is_empty());
    assert_eq!(max_hole_depth(obj), 0);
}

#[test]
fn no_hole_in_l_shape() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    let obj = &objects[0];

    assert_eq!(hole_count(obj), 0);
    assert!(hole_sizes(obj).is_empty());
    assert_eq!(max_hole_depth(obj), 0);
}
