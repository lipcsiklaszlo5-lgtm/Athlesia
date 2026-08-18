
use athlesia_world_model::{WorldModel, KnowledgeState, Observation, Prediction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};

fn grid_5x5_with_one_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

fn make_grid(x: usize, y: usize) -> Grid {
    grid_5x5_with_one_pixel(x, y, 1)
}

#[test]
fn evaluate_with_residual_explained() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(action.prim, action.params.clone())]);

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: prediction.state.clone() };

    let (state, residual) = wm.evaluate_with_residual(&action, &prediction, &observation);
    assert_eq!(state, KnowledgeState::Explained);
    assert_eq!(residual.mismatch_score, 0.0);
    assert!(residual.unexplained_features.is_empty());
}

#[test]
fn evaluate_with_residual_contradicted() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(action.prim, action.params.clone())]);

    let prediction = wm.predict(&initial, &action);
    let wrong_observation = Observation { state: initial.clone() }; // nem a predikció

    let (state, residual) = wm.evaluate_with_residual(&action, &prediction, &wrong_observation);
    assert_eq!(state, KnowledgeState::Contradicted);
    assert!(residual.mismatch_score > 0.0);
    assert!(residual.unexplained_features.contains(&"pixel_mismatch".to_string()));
}

#[test]
fn evaluate_with_residual_uncertain() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let wm = WorldModel::new(initial.clone()); // nincs hipotézis

    let prediction = wm.predict(&initial, &action);
    let wrong_observation = Observation { state: initial.clone() };

    let (state, residual) = wm.evaluate_with_residual(&action, &prediction, &wrong_observation);
    assert_eq!(state, KnowledgeState::Uncertain);
    assert!(residual.mismatch_score > 0.0);
}

#[test]
fn evaluate_with_residual_out_of_model() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    // Van egy másik hipotézis (ReflectH), de nem az akcióra vonatkozik
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let prediction = wm.predict(&initial, &action);
    let wrong_observation = Observation { state: initial.clone() };

    let (state, residual) = wm.evaluate_with_residual(&action, &prediction, &wrong_observation);
    assert_eq!(state, KnowledgeState::OutOfModel);
    assert!(residual.mismatch_score > 0.0);
}
