
use athlesia_core::CoreEngine;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn solves_two_step_program_with_search() {
    let mut core = CoreEngine::new();

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    // Cél: két lépés jobbra (Translate(1,0) + Translate(1,0)) = Translate(2,0)
    let target = build_grid([
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let (program, _steps) = core.solve_with_steps(&input, &target);
    assert!(program.is_some(), "A motornak meg kell oldania a kétlépéses feladatot");
    // A megoldás hossza legalább 2, mert két Translate lépés kell
    let program = program.unwrap();
    assert!(program.len() >= 2, "A programnak legalább 2 lépésből kell állnia, de hossza: {}", program.len());

    // A megtanult programot a motor másodjára már azonnal előveszi
    let (_, steps_second) = core.solve_with_steps(&input, &target);
    assert_eq!(steps_second, 1, "Másodjára már csak 1 lépés kell");
}
