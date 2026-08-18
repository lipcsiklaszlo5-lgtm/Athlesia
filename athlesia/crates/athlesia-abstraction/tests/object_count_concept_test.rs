
use athlesia_abstraction::AbstractionEngine;
use athlesia_world_model::{Observation, PredictionResidual};
use athlesia_types::{Grid, Color};

fn grid_3x3_with_objects(count: usize) -> Grid {
    let mut g = Grid::new(3, 3);
    for i in 0..count {
        g.set(i as i8, 0, Color(1));
    }
    g
}

#[test]
fn discover_candidate_concept_prefers_object_count_changed() {
    let residual = PredictionResidual {
        expected_observation: Observation { state: grid_3x3_with_objects(1) },
        observed_observation: Observation { state: grid_3x3_with_objects(2) },
        mismatch_score: 0.5,
        unexplained_features: vec!["pixel_mismatch".to_string(), "object_count_changed".to_string()],
    };

    let candidate = AbstractionEngine::discover_candidate_concept(&[residual])
        .expect("Candidate fogalom kell");
    assert_eq!(candidate.sketch.relation_pattern, "object_count_change(A,B)");
    assert!(candidate.sketch.name.contains("object_count_change"));
}
