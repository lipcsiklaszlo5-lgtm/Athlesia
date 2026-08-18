
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};

fn grid_5x5_with_one_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

fn grid_3x3_zeros() -> Grid {
    Grid { width: 3, height: 3, cells: vec![Color(0); 9] }
}

#[test]
fn residual_zero_for_exact_match() {
    let initial = grid_5x5_with_one_pixel(0, 0, 1);
    let wm = WorldModel::new(initial.clone());
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: initial.clone(), confidence: 0.5 };
    let observation = Observation { state: initial.clone() };

    let residual = wm.compute_prediction_residual(&action, &prediction, &observation);
    assert_eq!(residual.mismatch_score, 0.0);
    assert!(residual.unexplained_features.is_empty());
}

#[test]
fn residual_fraction_for_partial_mismatch() {
    let initial = grid_5x5_with_one_pixel(0, 0, 1);
    let expected = grid_5x5_with_one_pixel(1, 0, 1); // egy eltérő pixel
    let wm = WorldModel::new(initial.clone());
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: expected.clone(), confidence: 0.5 };
    let observation = Observation { state: initial.clone() }; // a tényleges megfigyelés más

    let residual = wm.compute_prediction_residual(&action, &prediction, &observation);
    // 25 pixelből 1 tér el, ha expected[1,0]=1 és initial[0,0]=1, a többi 0.
    assert!((residual.mismatch_score - 0.08).abs() < 0.0001);
    assert_eq!(residual.unexplained_features, vec!["pixel_mismatch"]);
}

#[test]
fn residual_one_for_dimension_mismatch() {
    let initial_5x5 = grid_5x5_with_one_pixel(0, 0, 1);
    let initial_3x3 = grid_3x3_zeros();
    let wm = WorldModel::new(initial_5x5.clone());
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: initial_5x5.clone(), confidence: 0.5 };
    let observation = Observation { state: initial_3x3 };

    let residual = wm.compute_prediction_residual(&action, &prediction, &observation);
    assert_eq!(residual.mismatch_score, 1.0);
    assert_eq!(residual.unexplained_features, vec!["pixel_mismatch"]);
}
