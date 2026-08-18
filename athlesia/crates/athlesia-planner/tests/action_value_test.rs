
use athlesia_planner::{Planner, PlannerMode, ActionValue};
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, Color, Action, PrimName, Params};

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
fn action_value_information_gain_is_positive() {
    let current = grid_from_rows(vec![vec![1, 0], vec![0, 0]]);
    let wm = WorldModel::new(current.clone());
    let planner = Planner::new(PlannerMode::Exploration);

    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let av = planner.compute_action_value(&current, None, &action, &wm);

    assert!(av.expected_information_gain >= 0.0 && av.expected_information_gain <= 1.0);
    assert_eq!(av.expected_progress, 0.0);
    assert_eq!(av.action_cost, 1.0);
    assert_eq!(av.risk, 0.0);
}

#[test]
fn select_action_uses_beta_for_progress() {
    let current = grid_from_rows(vec![vec![1, 0], vec![0, 0]]);
    let target = grid_from_rows(vec![vec![0, 1], vec![0, 0]]);
    let wm = WorldModel::new(current.clone());
    let planner = Planner::new(PlannerMode::GoalDirected);

    let actions = vec![
        Action { prim: PrimName::Translate, params: Params::Translate(1, 0) }, // javítja a progresszt
        Action { prim: PrimName::Translate, params: Params::Translate(0, 1) }, // rontja
    ];

    // Csak a progresszt nézzük (β=1, α=γ=δ=0)
    let best = planner.select_action(&current, Some(&target), &actions, &wm, 0.0, 1.0, 0.0, 0.0)
        .expect("Kell lennie kiválasztott akciónak");
    assert_eq!(best.params, Params::Translate(1, 0), "A jobb progresszű akciót kell választani");
}
