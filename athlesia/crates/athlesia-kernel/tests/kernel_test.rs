
use athlesia_kernel::solve_with_kernel;
use athlesia_core::CoreEngine;
use athlesia_memory::Memory;
use athlesia_knowledge::KnowledgeBase;
use athlesia_planner::{Planner, PlannerMode};
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn kernel_solves_simple_translate() {
    let mut kb = KnowledgeBase::new();
    let mut mem = Memory::new();
    let planner = Planner::new(PlannerMode::GoalDirected);
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let mut core = CoreEngine::new();

    let input = build_grid([
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

    let program = solve_with_kernel(
        &input,
        &target,
        &mut kb,
        &mut mem,
        &planner,
        &wm,
        &mut core,
        2,
    );

    assert!(program.is_some());
    // A megoldásnak Translate(1,0)-nek kell lennie
    assert_eq!(
        program.unwrap(),
        vec![(PrimName::Translate, Params::Translate(1, 0))]
    );
}
