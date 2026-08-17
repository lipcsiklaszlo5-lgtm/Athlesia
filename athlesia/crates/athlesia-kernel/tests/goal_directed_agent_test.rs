
use athlesia_kernel::Agent;
use athlesia_types::{Grid, PrimName, Params, Budget};
use athlesia_executor::run_program;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn agent_uses_goal_directed_plan_after_learning() {
    let start = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let mut agent = Agent::new(start.clone());

    // Tanuljuk meg a Translate(1,0) szabályt ugyanazzal a szabállyal,
    // mint amit a környezet használ.
    let rule = vec![(PrimName::Translate, Params::Translate(1, 0))];

    let mut current = start;
    for _ in 0..5 {
        let _action = agent.step(&current, None);
        let mut budget = Budget { max_steps: 1 };
        let next = run_program(&rule, &current, &mut budget).unwrap();
        agent.update(&current, &next);
        current = next;
    }

    // Most egy új, ismeretlen pozíciójú bemenetet és célt adunk.
    let new_start = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 1, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    // A cél-irányított lépésnek azonnal a Translate(1,0) akciót kell adnia.
    let action = agent.step(&new_start, Some(&target));

    assert_eq!(action.prim, PrimName::Translate);
    match action.params {
        Params::Translate(dx, dy) => {
            assert_eq!(dx, 1);
            assert_eq!(dy, 0);
        }
        _ => panic!("Nem Translate akciót kaptunk"),
    }
}
