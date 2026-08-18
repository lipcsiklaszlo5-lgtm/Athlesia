
use athlesia_search::{search_with_budget, SearchTelemetry};
use athlesia_types::{Grid, Color};

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
fn search_with_budget_aborts_when_stuck() {
    // Olyan cél, amely nem érhető el a primitívekkel, hogy a keresés elakadjon.
    let input = make_grid(vec![vec![1, 0], vec![0, 0]]);
    let target = make_grid(vec![vec![2, 3], vec![4, 5]]);

    let max_score = (target.width as usize * target.height as usize) as f32;
    let mut telemetry = SearchTelemetry::new(max_score);
    let result = search_with_budget(&input, &target, 3, &mut telemetry);

    assert!(result.is_none(), "Nem kellett volna megoldást találni");
    assert!(telemetry.should_abort(), "A keresésnek le kellett volna állnia");
    assert!(telemetry.hypotheses_tested > 0, "Telemetriának gyűjtenie kell adatot");
}
