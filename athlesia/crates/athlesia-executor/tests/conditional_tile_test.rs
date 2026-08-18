use athlesia_executor::apply_primitive;
use athlesia_types::{Grid, Color, PrimName, Params};

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
fn conditional_tile_places_only_on_foreground() {
    let mask = grid_from_rows(vec![
        vec![1, 0],
        vec![0, 1],
    ]);
    let output = apply_primitive(&mask, &PrimName::ConditionalTile, &Params::ConditionalTile);

    let expected = grid_from_rows(vec![
        vec![1,0,0,0],
        vec![0,1,0,0],
        vec![0,0,1,0],
        vec![0,0,0,1],
    ]);

    assert_eq!(output, expected);
}
