
use athlesia_perception::pattern::{periodicity_score, detect_periodicity};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn detects_periodic_pattern() {
    // Sakktábla-szerű 2x2 minta
    let grid = build_grid([
        [1, 2, 1, 2, 1],
        [2, 1, 2, 1, 2],
        [1, 2, 1, 2, 1],
        [2, 1, 2, 1, 2],
        [1, 2, 1, 2, 1],
    ]);
    let period = detect_periodicity(&grid, 1.0);
    assert!(period.is_some());
    let (px, py) = period.unwrap();
    // A sakktábla 1x1-es eltolásra is invariáns, így a legkisebb periódus (1,1).
    assert_eq!((px, py), (1, 1));
}

#[test]
fn periodicity_score_is_one_for_exact_repeat() {
    let grid = build_grid([
        [1, 1, 2, 2, 1],
        [1, 1, 2, 2, 1],
        [2, 2, 1, 1, 2],
        [2, 2, 1, 1, 2],
        [1, 1, 2, 2, 1],
    ]);
    // 2x2 periódus tökéletesen illeszkedik
    let score = periodicity_score(&grid, (2, 2));
    assert_eq!(score, 1.0);
}

#[test]
fn no_periodicity_on_random_grid() {
    let grid = build_grid([
        [1, 2, 3, 4, 5],
        [5, 4, 3, 2, 1],
        [1, 3, 2, 4, 5],
        [5, 2, 4, 3, 1],
        [2, 4, 1, 5, 3],
    ]);
    // Itt 4-es, 5-ös színek is vannak, de a rács 5x5
    let period = detect_periodicity(&grid, 0.99);
    assert!(period.is_none());
}
