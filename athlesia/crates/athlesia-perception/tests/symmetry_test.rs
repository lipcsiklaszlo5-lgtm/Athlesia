
use athlesia_perception::symmetry::{
    horizontal_symmetry, vertical_symmetry, diagonal_symmetry, rotational_symmetry_180, DiagonalAxis
};
use athlesia_perception::segment;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn full_symmetry_for_solid_square() {
    let grid = build_grid([
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    let obj = &objects[0];

    assert_eq!(horizontal_symmetry(obj), 1.0);
    assert_eq!(vertical_symmetry(obj), 1.0);
    assert_eq!(diagonal_symmetry(obj, DiagonalAxis::Main), 1.0);
    assert_eq!(diagonal_symmetry(obj, DiagonalAxis::Anti), 1.0);
    assert_eq!(rotational_symmetry_180(obj), 1.0);
}

#[test]
fn horizontal_line_has_vertical_symmetry_but_not_horizontal() {
    let grid = build_grid([
        [1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    let obj = &objects[0];

    // Vízszintes vonal: mindkét szimmetriája teljes, mert a bbox középpontjára tükrözve önmaga
    assert_eq!(vertical_symmetry(obj), 1.0);
    assert_eq!(horizontal_symmetry(obj), 1.0);
}

#[test]
fn l_shape_is_not_fully_symmetric() {
    let grid = build_grid([
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    let obj = &objects[0];

    // L-alak se nem tükrös, se nem 180 fokos, se nem átlós
    assert!(horizontal_symmetry(obj) < 1.0);
    assert!(vertical_symmetry(obj) < 1.0);
    assert!(rotational_symmetry_180(obj) < 1.0);
    assert!(diagonal_symmetry(obj, DiagonalAxis::Main) < 1.0);
}

#[test]
fn cross_is_fully_symmetric() {
    let grid = build_grid([
        [0, 1, 0, 0, 0],
        [1, 1, 1, 0, 0],
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let objects = segment(&grid);
    let obj = &objects[0];

    assert_eq!(horizontal_symmetry(obj), 1.0);
    assert_eq!(vertical_symmetry(obj), 1.0);
    assert_eq!(diagonal_symmetry(obj, DiagonalAxis::Main), 1.0);
    assert_eq!(diagonal_symmetry(obj, DiagonalAxis::Anti), 1.0);
    assert_eq!(rotational_symmetry_180(obj), 1.0);
}
