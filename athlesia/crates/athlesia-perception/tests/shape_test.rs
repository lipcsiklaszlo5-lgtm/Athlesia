
use athlesia_perception::shape::{cell_count, bbox_dimensions, fill_ratio, linearity};
use athlesia_perception::segment;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn shape_metrics_for_solid_square() {
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

    assert_eq!(cell_count(obj), 9);
    assert_eq!(bbox_dimensions(obj), (3, 3));
    assert_eq!(fill_ratio(obj), 1.0);
    // A négyzet kompakt, a linearitás 0-hoz közeli
    assert!(linearity(obj) < 0.1, "linearitás négyzetre: {}", linearity(obj));
}

#[test]
fn shape_metrics_for_horizontal_line() {
    let grid = build_grid([
        [1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    let obj = &objects[0];

    assert_eq!(cell_count(obj), 5);
    assert_eq!(bbox_dimensions(obj), (5, 1));
    assert_eq!(fill_ratio(obj), 1.0);
    // Egy vízszintes vonal erősen vonalszerű, linearitás közel 1
    assert!(linearity(obj) > 0.9, "linearitás vonalra: {}", linearity(obj));
}

#[test]
fn shape_metrics_for_l_shape() {
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

    assert_eq!(cell_count(obj), 4);
    assert_eq!(bbox_dimensions(obj), (2, 3));
    // L-alak nem tölti ki a bboxot
    assert!(fill_ratio(obj) < 1.0);
    // L-alak se nem teljesen vonal, se nem négyzet
    let lin = linearity(obj);
    assert!(lin > 0.1 && lin < 0.9, "linearitás L-alakra: {}", lin);
}
