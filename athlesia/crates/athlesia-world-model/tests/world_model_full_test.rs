use athlesia_world_model::{WorldModel, HypothesisStatus, Observation, UpdateResult, Query};
use athlesia_types::{Grid, PrimName, Params, Action};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

fn make_input(x: usize, y: usize) -> Grid {
    let mut rows = [[0u8; 5]; 5];
    rows[y][x] = 1;
    build_grid(rows)
}

#[test]
fn predict_returns_prediction_with_confidence() {
    let wm = WorldModel::new(make_input(0, 0));
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let expected = make_input(1, 0);

    let pred = wm.predict(&make_input(0, 0), &action);
    assert_eq!(pred.state, expected);
    assert_eq!(pred.confidence, 0.5); // nincs hipotézis
}

#[test]
fn update_confirms_hypothesis_after_three_successes() {
    let step0 = make_input(0, 0);
    let step1 = make_input(1, 0);
    let step2 = make_input(2, 0);
    let step3 = make_input(3, 0);

    let mut wm = WorldModel::new(step0.clone());
    let program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    wm.add_hypothesis(program);

    wm.update(&Observation { state: step1.clone() });
    wm.update(&Observation { state: step2.clone() });
    wm.update(&Observation { state: step3.clone() });

    assert_eq!(wm.hypotheses[0].evidence_for, 3);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Confirmed);
}

#[test]
fn update_falsifies_wrong_hypothesis() {
    let step0 = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = step0.clone(); // nem tükrözés

    let mut wm = WorldModel::new(step0.clone());
    let program = vec![(PrimName::ReflectH, Params::None)];
    wm.add_hypothesis(program);

    let result = wm.update(&Observation { state: obs });
    assert_eq!(result, UpdateResult::Falsified);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Falsified);
}

#[test]
fn uncertainty_decreases_with_confidence() {
    let step0 = make_input(0, 0);
    let step1 = make_input(1, 0);
    let step2 = make_input(2, 0);
    let step3 = make_input(3, 0);

    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let program = vec![(action.prim, action.params)];
    let mut wm = WorldModel::new(step0.clone());
    wm.add_hypothesis(program);

    wm.update(&Observation { state: step1 });
    wm.update(&Observation { state: step2 });
    wm.update(&Observation { state: step3.clone() });

    // A jelenlegi állapot step3, a predikció step4-re mutatna,
    // de a rács szélén kívül esik, ezért az unwrap_or_else az eredeti step3-at adja.
    // A konfidencia a hipotézisből jön: evidence_for=3, evidence_against=0.
    let query = Query { state: step3, action };
    let uncertainty = wm.uncertainty(&query);
    assert!(uncertainty < 0.5);
}
