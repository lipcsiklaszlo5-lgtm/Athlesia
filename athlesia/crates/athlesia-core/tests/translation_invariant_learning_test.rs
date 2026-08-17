use athlesia_core::CoreEngine;
use athlesia_types::{Grid, PrimName, Params, Color};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

/// Segédfüggvény: egyetlen 1-es cellát rak a megadott pozícióba,
/// és az 5x5-ös rácson belül marad.
fn make_input(x: usize, y: usize) -> Grid {
    let mut rows = [[0u8; 5]; 5];
    rows[x][y] = 1;
    build_grid(rows)
}

/// A szabály: jobbra tolás (Translate(1,0)). A cél az 1-es cella eggyel jobbra.
/// Ha a cella a jobb szélen van, a rácsból kicsúszik, ezért az ilyen eseteket kerüljük.
fn make_target(x: usize, y: usize) -> Grid {
    let mut rows = [[0u8; 5]; 5];
    rows[x][y + 1] = 1;
    build_grid(rows)
}

#[test]
fn learning_transfers_across_positions() {
    let mut core = CoreEngine::new();

    // Félrevezető hipotézisek
    core.known_programs.push(vec![(PrimName::ReflectH, Params::None)]);
    core.known_programs.push(vec![(PrimName::ReflectV, Params::None)]);
    core.known_programs.push(vec![(PrimName::Rotate90, Params::None)]);
    core.known_programs.push(vec![(PrimName::Recolor, Params::Recolor([Color(1), Color(0), Color(2), Color(3)]))]);

    // Különböző pozíciók, ahol a cella nincs a jobb szélen
    let positions = [(0, 0), (1, 1), (2, 2), (0, 3), (1, 0)];
    let mut steps_history = Vec::new();

    for (x, y) in positions {
        let input = make_input(x, y);
        let target = make_target(x, y);

        let (program, steps) = core.solve_with_steps(&input, &target);
        assert!(program.is_some(), "A kernelnek meg kell oldania a feladatot");
        steps_history.push(steps);
    }

    // Az első feladatnál a motor nem tudja a szabályt, tehát sok hipotézist próbál ki
    assert!(steps_history[0] >= 5, "Első lépésszám: {}", steps_history[0]);

    // A második feladattól a motor már ismeri a szabályt, még akkor is,
    // ha az objektum más pozícióban van (eltolás-invariáns jellemzők).
    for i in 1..steps_history.len() {
        assert_eq!(
            steps_history[i], 1,
            "A(z) {}. feladatnál a lépésszám: {} (pozíció: {:?})",
            i, steps_history[i], positions[i]
        );
    }
}
