
use athlesia_planner::{Planner, PlannerMode};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn goal_directed_planner_finds_reachable_target() {
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

    let plan = planner.plan(&current, Some(&target), 2);
    assert!(plan.is_some());
    assert!(!plan.unwrap().is_empty());
}

#[test]
fn goal_directed_planner_returns_none_when_no_solution() {
    let planner = Planner::new(PlannerMode::GoalDirected);

    let current = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);

    let plan = planner.plan(&current, Some(&target), 2);
    assert!(plan.is_none());
}

#[test]
fn exploration_planner_returns_default_action() {
    let planner = Planner::new(PlannerMode::Exploration);

    let current = build_grid([[0; 5]; 5]);
    let plan = planner.plan(&current, None, 1);
    assert!(plan.is_some());
    assert_eq!(plan.unwrap().len(), 1);
}
