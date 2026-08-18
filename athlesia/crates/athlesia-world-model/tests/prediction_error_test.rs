
use athlesia_world_model::{WorldModel, PredictionError};
use athlesia_types::{Grid, Color};

fn grid_5x5_filled(value: u8) -> Grid {
    Grid {
        width: 5,
        height: 5,
        cells: vec![Color(value); 25],
    }
}

#[test]
fn record_prediction_error_stores_error() {
    let mut wm = WorldModel::new(grid_5x5_filled(0));
    let error = PredictionError {
        expected: grid_5x5_filled(1),
        observed: grid_5x5_filled(2),
        summary: "test mismatch".to_string(),
        feature_mismatch: 25,
    };
    wm.record_prediction_error(error);
    assert_eq!(wm.recent_errors.len(), 1);
    assert_eq!(wm.recent_errors[0].summary, "test mismatch");
}
