
use athlesia_world_model::WorldModel;
use athlesia_types::{Grid, PrimName, Params, Action};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn predict_translate_returns_expected_grid() {
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

    let (predicted, conf) = wm.predict(&input, &action);

    let expected = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    assert_eq!(predicted, expected);
    // Nincs még hipotézis, ezért semleges konfidencia
    assert_eq!(conf, 0.5);
}

#[test]
fn predict_uses_hypothesis_confidence() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    let action = Action {
        prim: PrimName::Translate,
        params: Params::Translate(1, 0),
    };

    // Adjunk hozzá egy hipotézist, ami pontosan ezt az akciót ismeri
    let program = vec![(action.prim, action.params)];
    wm.add_hypothesis(program.clone());

    // Megerősítjük a hipotézist néhány jó predikcióval
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

    let (_, conf) = wm.predict(&input, &action);
    // 3 sikeres predikció után a konfidencia magasabb 0.5-nél
    assert!(conf > 0.5);
}
