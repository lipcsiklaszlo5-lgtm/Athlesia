
use athlesia_abstraction::AbstractionEngine;
use athlesia_world_model::{Observation, PredictionResidual};
use athlesia_types::{Grid, Color};

fn grid_3x3_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 9];
    cells[y * 3 + x] = Color(val);
    Grid { width: 3, height: 3, cells }
}

fn residual_with_mismatch() -> PredictionResidual {
    PredictionResidual {
        expected_observation: Observation { state: grid_3x3_with_pixel(0, 0, 1) },
        observed_observation: Observation { state: grid_3x3_with_pixel(1, 0, 2) },
        mismatch_score: 0.5,
        unexplained_features: vec!["pixel_mismatch".to_string()],
    }
}

#[test]
fn discover_candidate_concept_returns_none_for_empty() {
    let result = AbstractionEngine::discover_candidate_concept(&[]);
    assert!(result.is_none());
}

#[test]
fn discover_candidate_concept_returns_candidate_for_mismatch() {
    let residuals = vec![residual_with_mismatch()];
    let candidate = AbstractionEngine::discover_candidate_concept(&residuals)
        .expect("Candidate conceptet kell generálni");
    assert!(!candidate.sketch.name.is_empty());
    assert!(!candidate.sketch.relation_pattern.is_empty());
    assert!(candidate.confidence > 0.0);
    assert!(!candidate.evidence.is_empty());
}
