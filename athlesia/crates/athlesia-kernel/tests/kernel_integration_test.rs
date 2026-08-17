use athlesia_kernel::solve_with_kernel;
use athlesia_memory::Memory;
use athlesia_knowledge::KnowledgeBase;
use athlesia_planner::{Planner, PlannerMode};
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
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
fn kernel_solves_multiple_positions_and_updates_state() {
    let mut kb = KnowledgeBase::new();
    let mut mem = Memory::new();
    let planner = Planner::new(PlannerMode::GoalDirected);
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));

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
            2,
        );

        assert!(program.is_some(), "A kernelnek minden pozícióban meg kell oldania a feladatot");
        assert_eq!(program.unwrap(), expected_program, "A megoldásnak mindig Translate(1,0)-nak kell lennie");
    }

    // Három megoldott feladat után a memóriában és a tudásbázisban is nyoma kell legyen
    assert_eq!(mem.episodic.len(), 3);
    assert_eq!(mem.get_known_programs().len(), 1, "Ugyanaz a program csak egyszer tárolódik");
    assert_eq!(kb.get_all_macros().len(), 1, "A tudásbázisban egy makró legyen");
}
