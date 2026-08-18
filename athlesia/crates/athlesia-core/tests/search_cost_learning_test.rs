
use athlesia_core::CoreEngine;
use athlesia_types::{Grid, Color};
use athlesia_features::extract_features;

fn make_grid(rows: Vec<Vec<u8>>) -> Grid {
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
fn core_learns_search_cost_after_failed_search() {
    let mut engine = CoreEngine::new();
    let input = make_grid(vec![vec![1, 0], vec![0, 0]]);
    let target = make_grid(vec![vec![2, 3], vec![4, 5]]);

    let fv = extract_features(&input);
    let (result, _steps) = engine.solve_with_steps(&input, &target);
    assert!(result.is_none(), "Nincs megoldás erre a feladatra");

    // A keresési költségnek rögzítve kell lennie a meta-learnerben
    let cost = engine.meta.estimated_cost(fv, 0);
    assert!(cost.is_some(), "A keresési költséget meg kellett tanulni");
    assert!(cost.unwrap() > 0.0, "A költség nagyobb mint nulla");
}
