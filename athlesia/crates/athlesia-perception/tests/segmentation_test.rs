
use athlesia_perception::{segment, touches};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn segment_two_separate_objects() {
    let grid = build_grid([
        [1, 1, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 2, 2],
        [0, 0, 0, 2, 0],
    ]);

    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);
    assert_eq!(objects[0].color, 1);
    assert_eq!(objects[1].color, 2);
}

#[test]
fn segment_ignores_background() {
    let grid = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].cells.len(), 1);
}

#[test]
fn touches_false_for_diagonal() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [0, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);
    assert!(!touches(&objects[0], &objects[1]));
}

#[test]
fn touches_true_for_side_by_side() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);
    assert!(touches(&objects[0], &objects[1]));
}
