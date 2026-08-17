
use athlesia_core::CoreEngine;
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn learns_from_empty_and_reuses_without_resynthesis() {
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

    // Első alkalom: üres CoreEngine-nek szintetizálnia kell
    let program1 = core.solve(&input, &target)
        .expect("Üres motornak is meg kell oldania a feladatot");
    assert_eq!(core.known_programs.len(), 1);

    // Második alkalom: már használja a megtanult programot, nem szintetizál újat
    let program2 = core.solve(&input, &target)
        .expect("Másodjára is meg kell oldania");
    assert_eq!(core.known_programs.len(), 1, "Nem szabad új programot szintetizálnia");
    assert_eq!(program1, program2);
}
