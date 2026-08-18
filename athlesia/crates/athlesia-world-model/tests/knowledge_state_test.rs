
use athlesia_world_model::{WorldModel, KnowledgeState, Observation};
use athlesia_types::{Grid, PrimName, Params, Action};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

fn make_grid(x: usize, y: usize) -> Grid {
    let mut rows = [[0u8; 5]; 5];
    rows[y][x] = 1;
    build_grid(rows)
}

#[test]
fn evaluate_prediction_explained_when_state_matches() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(action.prim, action.params.clone())]);

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: prediction.state.clone() };

    let state = wm.evaluate_prediction(&action, &prediction, &observation);
    assert_eq!(state, KnowledgeState::Explained);
}

#[test]
fn evaluate_prediction_contradicted_when_known_but_wrong() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(action.prim, action.params.clone())]);

    // Predikció jobbra, de megfigyelés balra (vagy ugyanaz a kezdő)
    let prediction = wm.predict(&initial, &action);
    let wrong_observation = Observation { state: initial.clone() }; // nem a predikció

    let state = wm.evaluate_prediction(&action, &prediction, &wrong_observation);
    assert_eq!(state, KnowledgeState::Contradicted);
}

#[test]
fn evaluate_prediction_uncertain_when_no_hypotheses() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let wm = WorldModel::new(initial.clone());

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: initial.clone() }; // rossz

    let state = wm.evaluate_prediction(&action, &prediction, &observation);
    assert_eq!(state, KnowledgeState::Uncertain);
}

#[test]
fn evaluate_prediction_out_of_model_when_no_matching_hypothesis() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    // Van egy másik hipotézis (ReflectH), de nem az akcióra vonatkozik
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: initial.clone() }; // rossz

    let state = wm.evaluate_prediction(&action, &prediction, &observation);
    assert_eq!(state, KnowledgeState::OutOfModel);
}
