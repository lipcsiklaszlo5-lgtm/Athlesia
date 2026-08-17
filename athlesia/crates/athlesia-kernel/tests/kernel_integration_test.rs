
use athlesia_kernel::solve_with_kernel;
use athlesia_core::CoreEngine;
use athlesia_memory::Memory;
use athlesia_knowledge::KnowledgeBase;
use athlesia_planner::{Planner, PlannerMode};
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

fn make_input(x: usize, y: usize) -> Grid {
    let mut rows = [[0u8; 5]; 5];
    rows[x][y] = 1;
    build_grid(rows)
}

fn make_target(x: usize, y: usize) -> Grid {
    let mut rows = [[0u8; 5]; 5];
    rows[x][y + 1] = 1;
    build_grid(rows)
}

#[test]
fn kernel_uses_core_engine_for_learning() {
    let mut kb = KnowledgeBase::new();
    let mut mem = Memory::new();
    let planner = Planner::new(PlannerMode::GoalDirected);
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let mut core = CoreEngine::new();

    let positions = [(0, 0), (1, 1), (2, 2)];
    let expected_program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    for (x, y) in positions {
        let input = make_input(x, y);
        let target = make_target(x, y);

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

        assert!(program.is_some(), "A kernelnek minden pozícióban meg kell oldania a feladatot");
        assert_eq!(program.unwrap(), expected_program, "A megoldásnak mindig Translate(1,0)-nak kell lennie");
    }

    assert_eq!(mem.episodic.len(), 3);
    assert_eq!(mem.get_known_programs().len(), 1, "Ugyanaz a program csak egyszer tárolódik");
    assert_eq!(kb.get_all_macros().len(), 1, "A tudásbázisban egy makró legyen");
}

#[test]
fn kernel_solves_two_step_program_with_core() {
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
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let program = solve_with_kernel(&input, &target, &mut kb, &mut mem, &planner, &wm, &mut core, 3);
    assert!(program.is_some(), "A kernelnek meg kell oldania a kétlépéses feladatot");
    assert!(program.unwrap().len() >= 2);
}
