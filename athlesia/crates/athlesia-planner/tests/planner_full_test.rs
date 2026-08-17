
use athlesia_planner::{Planner, PlannerMode};
use athlesia_world_model::WorldModel;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn goal_directed_planner_finds_reachable_target() {
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let planner = Planner::new(PlannerMode::GoalDirected);

    let current = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let plan = planner.plan(&current, Some(&target), &wm, 2);
    assert!(plan.is_some());
    assert!(!plan.unwrap().is_empty());
}

#[test]
fn goal_directed_planner_returns_none_when_no_solution() {
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let planner = Planner::new(PlannerMode::GoalDirected);

    let current = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);

    let plan = planner.plan(&current, Some(&target), &wm, 2);
    assert!(plan.is_none());
}

#[test]
fn exploration_planner_selects_uncertain_action() {
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let planner = Planner::new(PlannerMode::Exploration);

    let current = build_grid([[0; 5]; 5]);
    let plan = planner.plan(&current, None, &wm, 1);
    assert!(plan.is_some());
    assert_eq!(plan.unwrap().len(), 1);
}
