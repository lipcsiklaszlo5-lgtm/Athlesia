
use athlesia_synthesis::synthesize;
use athlesia_types::{Grid, Color, PrimName, Params, Program};

fn grid_from_rows(rows: Vec<Vec<u8>>) -> Grid {
    let height = rows.len() as u8;
    let width = if height > 0 { rows[0].len() as u8 } else { 0 };
    let mut cells = Vec::new();
    for row in &rows {
        for &c in row {
            cells.push(Color(c));
        }
    }
    Grid { width, height, cells }
}

#[test]
fn synthesizes_repeat_grid_3x3_to_9x9() {
    let input = grid_from_rows(vec![
        vec![1, 2, 3],
        vec![4, 5, 6],
        vec![7, 8, 9],
    ]);
    let target = grid_from_rows(vec![
        vec![1,2,3,1,2,3,1,2,3],
        vec![4,5,6,4,5,6,4,5,6],
        vec![7,8,9,7,8,9,7,8,9],
        vec![1,2,3,1,2,3,1,2,3],
        vec![4,5,6,4,5,6,4,5,6],
        vec![7,8,9,7,8,9,7,8,9],
        vec![1,2,3,1,2,3,1,2,3],
        vec![4,5,6,4,5,6,4,5,6],
        vec![7,8,9,7,8,9,7,8,9],
    ]);

    let program = synthesize(&input, &target, &[]).expect("Meg kell találni a RepeatGrid(3)-at");
    assert_eq!(
        program,
        vec![(PrimName::RepeatGrid, Params::RepeatGrid(3))]
    );
}

#[test]
fn synthesizes_tile_2x2_to_4x4() {
    let input = grid_from_rows(vec![vec![1, 0], vec![0, 1]]);
    let target = grid_from_rows(vec![
        vec![1,1,0,0],
        vec![1,1,0,0],
        vec![0,0,1,1],
        vec![0,0,1,1],
    ]);

    let program = synthesize(&input, &target, &[]).expect("Meg kell találni a Tile(2)-t");
    assert_eq!(
        program,
        vec![(PrimName::Tile, Params::Tile(2))]
    );
}
