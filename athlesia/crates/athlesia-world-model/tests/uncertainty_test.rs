
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, PrimName, Params, Action};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn uncertainty_initial_is_half() {
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));

    let action = Action {
        prim: PrimName::Translate,
        params: Params::Translate(1, 0),
    };

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    // Nincs hipotézis, a bizonytalanság 1 - 0.5 = 0.5
    assert_eq!(wm.uncertainty(&input, &action), 0.5);
}

#[test]
fn uncertainty_decreases_after_successful_updates() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    let action = Action {
        prim: PrimName::Translate,
        params: Params::Translate(1, 0),
    };

    let program = vec![(action.prim, action.params)];
    wm.add_hypothesis(program.clone());

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    wm.update(&input, &obs);
    wm.update(&input, &obs);
    wm.update(&input, &obs);

    let uncertainty = wm.uncertainty(&input, &action);
    assert!(uncertainty < 0.5);
}
