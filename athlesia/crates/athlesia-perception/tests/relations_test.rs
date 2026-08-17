
use athlesia_perception::{segment, centroid, distance_between, relative_direction, contains, relative_size, shares_row, shares_col, color_histogram};
use athlesia_types::{Grid, Color};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
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
    let (a, b) = if objects[0].color == Color(1) { (&objects[0], &objects[1]) } else { (&objects[1], &objects[0]) };
    assert!(contains(a, b));
    assert!(!contains(b, a));
}


#[test]
fn relative_size_between_two_objects() {
    let grid = build_grid([
        [1, 1, 0, 0, 0],
        [1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 2, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);

    let rel = relative_size(&objects[0], &objects[1]);
    assert_eq!(rel, 4.0);
}

#[test]
fn shares_row_and_col_detect_overlap() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    assert_eq!(objects.len(), 2);
    assert!(shares_row(&objects[0], &objects[1]));
    assert!(!shares_col(&objects[0], &objects[1]));
}

#[test]
fn color_histogram_counts_colors() {
    let grid = build_grid([
        [1, 2, 3, 0, 0],
        [1, 2, 3, 0, 0],
        [1, 2, 3, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let hist = color_histogram(&grid);
    assert_eq!(hist[1], 3);
    assert_eq!(hist[2], 3);
    assert_eq!(hist[3], 3);
}
