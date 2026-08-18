
use athlesia_world_model::{WorldModel, PredictionError};
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn learn_from_error_falsifies_hypothesis() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let id = wm.add_hypothesis(program);

    let error = PredictionError {
        expected: build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
        observed: build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
        summary: "direction mismatch".to_string(),
        feature_mismatch: 2,
    };

    wm.learn_from_error(id, &error);
    assert_eq!(wm.hypotheses[0].evidence_against, 1);
    assert_eq!(wm.hypotheses[0].status, athlesia_world_model::HypothesisStatus::Falsified);
}
