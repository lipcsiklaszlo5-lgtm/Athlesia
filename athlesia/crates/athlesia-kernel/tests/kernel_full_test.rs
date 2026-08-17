
use athlesia_kernel::{solve_with_kernel, Agent};
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
fn solve_with_kernel_finds_simple_translate() {
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

    let program = solve_with_kernel(&input, &target, &mut kb, &mut mem, &planner, &wm, &mut core, 2);
    assert!(program.is_some());
    assert_eq!(program.unwrap(), vec![(PrimName::Translate, Params::Translate(1, 0))]);
}

#[test]
fn agent_step_and_update_work() {
    let initial = build_grid([[1, 0, 0, 0, 0], [0,0,0,0,0], [0,0,0,0,0], [0,0,0,0,0], [0,0,0,0,0]]);
    let mut agent = Agent::new(initial.clone());

    let action = agent.step(&initial, None);
    assert_eq!(action.prim, PrimName::Translate); // A feltáró mód leggyakrabban mozgat

    let observed = build_grid([[0, 1, 0, 0, 0], [0,0,0,0,0], [0,0,0,0,0], [0,0,0,0,0], [0,0,0,0,0]]);
    agent.update(&observed);
    assert!(agent.wm.tick > 0);
}
