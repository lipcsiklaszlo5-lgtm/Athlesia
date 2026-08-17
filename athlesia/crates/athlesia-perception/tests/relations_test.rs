
use athlesia_perception::{segment, centroid, distance_between, relative_direction, contains};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn centroid_of_square_object() {
    let grid = build_grid([
        [1, 1, 0, 0, 0],
        [1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 1);
    let (cx, cy) = centroid(&objects[0]);
    // 2x2-es négyzet a (0,0)-nál: centroid (0.5, 0.5)
    assert_eq!(cx, 0.5);
    assert_eq!(cy, 0.5);
}

#[test]
fn distance_between_two_objects() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 2],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);
    // (0,0) és (4,4): távolság = sqrt(32) = 5.656...
    let d = distance_between(&objects[0], &objects[1]);
    assert!(d > 5.0 && d < 6.0);
}

#[test]
fn relative_direction_basic() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 2],
    ]);
    let objects = segment(&grid);
    let dir = relative_direction(&objects[0], &objects[1]);
    // mindkét komponens pozitív, mert B jobbra és lefelé van
    assert_eq!(dir, (1, 1));
}

#[test]
fn contains_detects_bbox_containment() {
    // B bounding box-át A bounding box-a tartalmazza
    let grid = build_grid([
        [1, 1, 1, 0, 0],
        [1, 2, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);
    // A az 1-es, B a 2-es
    let (a, b) = if objects[0].color == 1 { (&objects[0], &objects[1]) } else { (&objects[1], &objects[0]) };
    assert!(contains(a, b));
    assert!(!contains(b, a));
}
