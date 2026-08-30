use athlesia::{
    ActiveInferenceEngine, ActiveInferenceError, Encoder, PartialStructuralState,
    PredictionOutcome, PredictiveStructuralModel, PrimitiveSignature, RelationKind,
    RelationalStructure, StructuralConcept,
};

fn alternating_model() -> PredictiveStructuralModel {
    let encoder = Encoder::new();

    let sequence = encoder.encode(&[1, 2, 1, 2, 3]);

    let structure = RelationalStructure::from_sequence(&sequence);

    let concept = StructuralConcept::new(vec![PrimitiveSignature::new(RelationKind::Equal, 2)]);

    PredictiveStructuralModel::from_example(&concept, &structure).unwrap()
}

#[test]
fn active_step_selects_expected_target() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let transition = ActiveInferenceEngine::new()
        .step(&model, &state, &observation)
        .unwrap();

    assert_eq!(transition.selected().target(), 2);
}

#[test]
fn active_step_confirms_prediction() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let transition = ActiveInferenceEngine::new()
        .step(&model, &state, &observation)
        .unwrap();

    assert_eq!(transition.confirmed_count(), 1);

    assert_eq!(transition.violated_count(), 0);

    assert_eq!(
        transition.evaluations()[0].outcome(),
        PredictionOutcome::Confirmed
    );
}

#[test]
fn active_step_preserves_prediction_violation() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 999, 13, 777]);

    let transition = ActiveInferenceEngine::new()
        .step(&model, &state, &observation)
        .unwrap();

    assert_eq!(transition.selected().target(), 2);

    assert_eq!(transition.confirmed_count(), 0);

    assert_eq!(transition.violated_count(), 1);

    assert_eq!(
        transition.evaluations()[0].outcome(),
        PredictionOutcome::Violated
    );
}

#[test]
fn active_step_advances_observation_state() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let transition = ActiveInferenceEngine::new()
        .step(&model, &state, &observation)
        .unwrap();

    assert_eq!(transition.before().observed_count(), 2);

    assert_eq!(transition.after().observed_count(), 3);

    assert_eq!(transition.after().is_observed(2), Some(true));
}

#[test]
fn active_step_does_not_mutate_input_state() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let original = state.clone();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let _transition = ActiveInferenceEngine::new()
        .step(&model, &state, &observation)
        .unwrap();

    assert_eq!(state, original);
}

#[test]
fn repeated_active_step_is_deterministic() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let engine = ActiveInferenceEngine::new();

    let first = engine.step(&model, &state, &observation);

    let second = engine.step(&model, &state, &observation);

    assert_eq!(first, second);
}

#[test]
fn active_transition_is_value_invariant() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let first_observation = encoder.encode(&[1, 2, 1, 2, 3]);

    let second_observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let engine = ActiveInferenceEngine::new();

    let first = engine.step(&model, &state, &first_observation);

    let second = engine.step(&model, &state, &second_observation);

    assert_eq!(first, second);
}

#[test]
fn second_active_step_can_select_next_target() {
    let model = alternating_model();

    let initial = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let engine = ActiveInferenceEngine::new();

    let first = engine.step(&model, &initial, &observation).unwrap();

    let second = engine.step(&model, first.after(), &observation).unwrap();

    assert_eq!(first.selected().target(), 2);

    assert_eq!(second.selected().target(), 3);

    assert_eq!(second.after().observed_count(), 4);
}

#[test]
fn no_experiment_is_explicit() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1, 2, 3, 4]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[1, 2, 1, 2, 3]);

    let result = ActiveInferenceEngine::new().step(&model, &state, &observation);

    assert_eq!(result, Err(ActiveInferenceError::NoExperimentAvailable));
}

#[test]
fn state_length_mismatch_is_explicit() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[1, 2, 1, 2, 3]);

    let result = ActiveInferenceEngine::new().step(&model, &state, &observation);

    assert_eq!(
        result,
        Err(ActiveInferenceError::StateLengthMismatch {
            expected: 5,
            actual: 4,
        })
    );
}

#[test]
fn observation_length_mismatch_is_explicit() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[1, 2, 1, 2]);

    let result = ActiveInferenceEngine::new().step(&model, &state, &observation);

    assert_eq!(
        result,
        Err(ActiveInferenceError::ObservationLengthMismatch {
            expected: 5,
            actual: 4,
        })
    );
}

#[test]
fn active_transition_contains_structural_semantics_only() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let transition = ActiveInferenceEngine::new()
        .step(&model, &state, &observation)
        .unwrap();

    assert_eq!(transition.selected().target(), 2);

    assert_eq!(transition.selected().information_gain(), 1);

    assert_eq!(transition.evaluations().len(), 1);
}

#[test]
fn violation_still_advances_observation_state() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[1, 2, 3, 2, 4]);

    let transition = ActiveInferenceEngine::new()
        .step(&model, &state, &observation)
        .unwrap();

    assert_eq!(transition.violated_count(), 1);

    assert_eq!(transition.after().is_observed(2), Some(true));

    assert_eq!(transition.after().observed_count(), 3);
}

#[test]
fn active_state_transition_matches_module_23_loop() {
    let model = alternating_model();

    let initial = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let encoder = Encoder::new();

    let observation = encoder.encode(&[847, 13, 847, 13, 999]);

    let engine = ActiveInferenceEngine::new();

    let first = engine.step(&model, &initial, &observation).unwrap();

    assert_eq!(first.selected().target(), 2);

    assert_eq!(first.confirmed_count(), 1);

    let second = engine.step(&model, first.after(), &observation).unwrap();

    assert_eq!(second.selected().target(), 3);

    assert_eq!(second.confirmed_count(), 1);

    assert_eq!(second.after().observed_count(), 4);
}
