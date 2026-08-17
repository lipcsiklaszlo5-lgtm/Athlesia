
use athlesia_executor::apply_primitive;
use athlesia_types::{Grid, PrimName, Params, Color};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn rotate180_works() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [3, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 4, 0],
        [0, 0, 0, 0, 5],
    ]);
    let result = apply_primitive(&grid, &PrimName::Rotate180, &Params::None);

    let expected = build_grid([
        [5, 0, 0, 0, 0],
        [0, 4, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 3],
        [0, 0, 0, 2, 1],
    ]);

    assert_eq!(result, expected);
}

#[test]
fn rotate270_works() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [3, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 4, 0],
        [0, 0, 0, 0, 5],
    ]);
    let result = apply_primitive(&grid, &PrimName::Rotate270, &Params::None);

    let expected = build_grid([
        [0, 0, 0, 3, 1],
        [0, 0, 0, 0, 2],
        [0, 0, 0, 0, 0],
        [0, 4, 0, 0, 0],
        [5, 0, 0, 0, 0],
    ]);

    assert_eq!(result, expected);
}

#[test]
fn add_border_increases_size() {
    let grid = build_grid([[1; 5]; 5]);
    let result = apply_primitive(&grid, &PrimName::AddBorder, &Params::None);
    assert_eq!(result.width, 7);
    assert_eq!(result.height, 7);
    assert_eq!(result.get(0, 0), Some(Color(0)));
    assert_eq!(result.get(1, 1), Some(Color(1)));
}

#[test]
fn remove_border_decreases_size() {
    let grid = build_grid([
        [0, 0, 0, 0, 0],
        [0, 1, 1, 1, 0],
        [0, 1, 1, 1, 0],
        [0, 1, 1, 1, 0],
        [0, 0, 0, 0, 0],
    ]);
    let result = apply_primitive(&grid, &PrimName::RemoveBorder, &Params::None);
    assert_eq!(result.width, 3);
    assert_eq!(result.height, 3);
    assert_eq!(result.get(0, 0), Some(Color(1)));
}

#[test]
fn swap_colors_works() {
    let grid = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let result = apply_primitive(&grid, &PrimName::SwapColors, &Params::SwapColors(1, 2));

    let expected = build_grid([
        [2, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    assert_eq!(result, expected);
}

#[test]
fn translate_wrap_wraps_around() {
    let grid = build_grid([
        [0, 0, 0, 0, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let result = apply_primitive(&grid, &PrimName::TranslateWrap, &Params::TranslateWrap(1, 0));

    let expected = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    assert_eq!(result, expected);
}
