
use athlesia_kernel::cognitive::{CognitiveController, CompetenceEstimate};
use athlesia_features::FeatureVector;
use athlesia_metalearner::MetaLearner;
use athlesia_types::{Grid, Color};

fn grid_from_rows(rows: Vec<Vec<u8>>) -> Grid {
    let height = rows.len() as u8;
    let width = if height > 0 { rows[0].len() as u8 } else { 0 };
    let mut cells = Vec::new();
    for row in &rows {
        for &c in row {
            cells.push(Color(c));
        }
    }
    Grid { width, height, cells }
}

#[test]
fn structural_match_high_for_repeated_identity_blocks() {
    let input = grid_from_rows(vec![
        vec![1, 2],
        vec![3, 4],
    ]);
    let target = grid_from_rows(vec![
        vec![1,2,1,2],
        vec![3,4,3,4],
        vec![1,2,1,2],
        vec![3,4,3,4],
    ]);

    let meta = MetaLearner::new();
    let fv = FeatureVector::default();
    let estimate = CognitiveController::estimate(&fv, &meta, &input, &target);
    assert!(estimate.structural_match > 0.9, "Az identitás ismétlődésnek magas strukturális egyezést kell adnia, de {} volt", estimate.structural_match);
}

#[test]
fn structural_match_zero_for_trivial_same_size() {
    let input = grid_from_rows(vec![vec![1,2], vec![3,4]]);
    let target = grid_from_rows(vec![vec![1,2], vec![3,4]]);

    let meta = MetaLearner::new();
    let fv = FeatureVector::default();
    let estimate = CognitiveController::estimate(&fv, &meta, &input, &target);
    assert_eq!(estimate.structural_match, 0.0, "Azonos méretű grid nem ad strukturális jelet");
}
