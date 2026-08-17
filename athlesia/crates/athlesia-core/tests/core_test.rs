
use athlesia_core::CoreEngine;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn learns_new_program_and_reuses_it() {
    let mut core = CoreEngine::new();

    // Két kezdő hipotézis, amelyek rosszak erre a feladatra
    core.known_programs.push(vec![(PrimName::ReflectH, Params::None)]);
    core.known_programs.push(vec![(PrimName::ReflectV, Params::None)]);

    // A valódi szabály: jobbra tolás (Translate(1,0))
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

    // Első megoldás: szintetizálnia kell a Translate programot
    let program = core.solve(&input, &target).expect("Meg kell oldania a feladatot");
    assert_eq!(core.known_programs.len(), 3); // 2 kezdeti + 1 új

    // Második megoldás ugyanarra a feladatra: már ne szintetizáljon újat,
    // hanem a megtanultat használja
    let program2 = core.solve(&input, &target).expect("Másodjára is meg kell oldania");
    assert_eq!(core.known_programs.len(), 3); // nem nőtt a programok száma
    assert_eq!(program, program2); // ugyanazt a programot adta vissza
}

#[test]
fn fails_gracefully_when_no_solution_exists() {
    let mut core = CoreEngine::new();

    // Olyan target, amit a jelenlegi primitívek nem tudnak előállítani
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);

    // 9-es szín nem létezik (csak 0-3), tehát a szintézis nem találhatja meg
    let program = core.solve(&input, &target);
    assert!(program.is_none());
}
