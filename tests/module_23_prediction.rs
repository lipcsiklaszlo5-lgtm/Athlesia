use athlesia::{
    Encoder, PartialStructuralState, PredictionEngine, PredictionError, PredictionRule,
    PredictiveStructuralModel, PrimitiveSignature, RelationKind, RelationalStructure,
    StructuralConcept,
};

fn model<T>(values: &[T]) -> PredictiveStructuralModel
where
    T: Eq + std::hash::Hash,
{
    let encoder = Encoder::new();

    let sequence = encoder.encode(values);

    let relations = RelationalStructure::from_sequence(&sequence);

    let concept = StructuralConcept::new(vec![PrimitiveSignature::new(RelationKind::Equal, 2)]);

    PredictiveStructuralModel::from_example(&concept, &relations)
        .expect("structural concept must be supported")
}

#[test]
fn predictive_model_extracts_structural_rules() {
    let model = model(&[1, 2, 1, 2, 3]);

    assert_eq!(model.rule_count(), 2);

    assert_eq!(
        model.rules(),
        &[
            PredictionRule::new_equal(0, 2,),
            PredictionRule::new_equal(1, 3,),
        ]
    );
}

#[test]
fn predictive_model_is_value_invariant() {
    let first = model(&[1, 2, 1, 2, 3]);

    let second = model(&[847, 13, 847, 13, 999]);

    assert_eq!(first, second);
}

#[test]
fn predictive_model_is_deterministic() {
    let first = model(&[1, 2, 1, 2, 3]);

    let second = model(&[1, 2, 1, 2, 3]);

    assert_eq!(first, second);
}

#[test]
fn predictive_model_preserves_structural_length() {
    let model = model(&[1, 2, 1, 2, 3]);

    assert_eq!(model.sequence_length(), 5);
}

#[test]
fn unsupported_concept_cannot_build_model() {
    let encoder = Encoder::new();

    let sequence = encoder.encode(&[1, 2, 1, 2, 3]);

    let relations = RelationalStructure::from_sequence(&sequence);

    let unsupported = StructuralConcept::new(vec![PrimitiveSignature::new(RelationKind::Equal, 3)]);

    assert!(PredictiveStructuralModel::from_example(&unsupported, &relations,).is_none());
}

#[test]
fn partial_prefix_generates_two_predictions() {
    let model = model(&[1, 2, 1, 2, 3]);

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let predictions = PredictionEngine::new().predict(&model, &state).unwrap();

    assert_eq!(
        predictions,
        vec![
            PredictionRule::new_equal(0, 2,),
            PredictionRule::new_equal(1, 3,),
        ]
    );
}

#[test]
fn prediction_requires_observed_reference() {
    let model = model(&[1, 2, 1, 2, 3]);

    let state = PartialStructuralState::from_observed_positions(5, &[0]).unwrap();

    let predictions = PredictionEngine::new().predict(&model, &state).unwrap();

    assert_eq!(predictions, vec![PredictionRule::new_equal(0, 2,),]);
}

#[test]
fn observed_target_is_not_predicted_again() {
    let model = model(&[1, 2, 1, 2, 3]);

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1, 2]).unwrap();

    let predictions = PredictionEngine::new().predict(&model, &state).unwrap();

    assert_eq!(predictions, vec![PredictionRule::new_equal(1, 3,),]);
}

#[test]
fn fully_observed_state_generates_no_prediction() {
    let model = model(&[1, 2, 1, 2, 3]);

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1, 2, 3, 4]).unwrap();

    let predictions = PredictionEngine::new().predict(&model, &state).unwrap();

    assert!(predictions.is_empty());
}

#[test]
fn prediction_is_structural_only() {
    let model = model(&[847, 13, 847, 13, 999]);

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let predictions = PredictionEngine::new().predict(&model, &state).unwrap();

    for prediction in predictions {
        assert_eq!(prediction.kind(), RelationKind::Equal);

        assert!(prediction.reference() < prediction.target());
    }
}

#[test]
fn prediction_order_is_deterministic() {
    let model = model(&[1, 2, 1, 2, 3]);

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let engine = PredictionEngine::new();

    let first = engine.predict(&model, &state).unwrap();

    let second = engine.predict(&model, &state).unwrap();

    assert_eq!(first, second);
}

#[test]
fn state_length_mismatch_is_explicit() {
    let model = model(&[1, 2, 1, 2, 3]);

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let result = PredictionEngine::new().predict(&model, &state);

    assert_eq!(
        result,
        Err(PredictionError::LengthMismatch {
            expected: 5,
            actual: 4,
        })
    );
}

#[test]
fn invalid_observed_position_is_rejected() {
    let state = PartialStructuralState::from_observed_positions(5, &[0, 5]);

    assert!(state.is_none());
}

#[test]
fn empty_partial_state_is_valid() {
    let state = PartialStructuralState::new(0);

    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

#[test]
fn model_contains_concept_not_training_values() {
    let model = model(&[847, 13, 847, 13, 999]);

    assert_eq!(
        model.concept().signatures(),
        &[PrimitiveSignature::new(RelationKind::Equal, 2,)]
    );

    assert_eq!(
        model.rules(),
        &[
            PredictionRule::new_equal(0, 2,),
            PredictionRule::new_equal(1, 3,),
        ]
    );
}
