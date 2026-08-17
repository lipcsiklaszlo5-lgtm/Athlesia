use athlesia_core::CoreEngine;
use athlesia_types::{Grid, PrimName, Params, Color};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn search_steps_decrease_after_learning_same_context() {
    let mut core = CoreEngine::new();

    // Félrevezető, rossz programok
    core.known_programs.push(vec![(PrimName::ReflectH, Params::None)]);
    core.known_programs.push(vec![(PrimName::ReflectV, Params::None)]);
    core.known_programs.push(vec![(PrimName::Rotate90, Params::None)]);
    core.known_programs.push(vec![(PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3)]))]);

    // Ugyanaz a bemenet-cél pár, így a FeatureVector azonos
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

    let mut steps_history = Vec::new();

    for _ in 0..5 {
        let (program, steps) = core.solve_with_steps(&input, &target);
        assert!(program.is_some(), "A kernelnek meg kell oldania a feladatot");
        steps_history.push(steps);
    }

    // Első alkalom: legalább a 4 rossz program + szintézis = 5 lépés
    assert!(steps_history[0] >= 5, "Első lépésszám: {}", steps_history[0]);
    // Második alkalomtól a megtanult program az első helyen áll
    for i in 1..5 {
        assert_eq!(steps_history[i], 1, "A(z) {}. lépésszám: {}", i, steps_history[i]);
    }

    // A tanulási görbe csökkenése
    assert!(steps_history[4] < steps_history[0], "Nem csökkent a lépésszám: {:?}", steps_history);
}
