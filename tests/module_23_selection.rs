use athlesia::{
    Encoder, ExperimentGenerator, ExperimentSelector, PartialStructuralState,
    PredictiveStructuralModel, PrimitiveSignature, RelationKind, RelationalStructure,
    StructuralConcept,
};

fn alternating_model() -> PredictiveStructuralModel {
    let encoder = Encoder::new();

    let sequence = encoder.encode(&[1, 2, 1, 2, 3]);

    let structure = RelationalStructure::from_sequence(&sequence);

    let concept = StructuralConcept::new(vec![PrimitiveSignature::new(RelationKind::Equal, 2)]);

    PredictiveStructuralModel::from_example(&concept, &structure).unwrap()
}

fn composite_model() -> PredictiveStructuralModel {
    let encoder = Encoder::new();

    let sequence = encoder.encode(&[1, 1, 1, 1]);

    let structure = RelationalStructure::from_sequence(&sequence);

    let concept = StructuralConcept::new(vec![
        PrimitiveSignature::new(RelationKind::Equal, 1),
        PrimitiveSignature::new(RelationKind::Equal, 2),
    ]);

    PredictiveStructuralModel::from_example(&concept, &structure).unwrap()
}

#[test]
fn selector_chooses_highest_information_gain() {
    let model = composite_model();

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let candidates = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let selected = ExperimentSelector::new().select(&candidates).unwrap();

    assert_eq!(selected.target(), 2);
    assert_eq!(selected.information_gain(), 2);
}

#[test]
fn selector_uses_lowest_target_as_tie_break() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let candidates = ExperimentGenerator::new().generate(&model, &state).unwrap();

    assert_eq!(
        candidates[0].information_gain(),
        candidates[1].information_gain()
    );

    let selected = ExperimentSelector::new().select(&candidates).unwrap();

    assert_eq!(selected.target(), 2);
}

#[test]
fn selector_is_deterministic() {
    let model = composite_model();

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let candidates = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let selector = ExperimentSelector::new();

    let first = selector.select(&candidates);

    let second = selector.select(&candidates);

    assert_eq!(first, second);
}

#[test]
fn selector_returns_none_for_empty_candidates() {
    let selected = ExperimentSelector::new().select(&[]);

    assert!(selected.is_none());
}

#[test]
fn selection_exposes_selected_candidate() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let candidates = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let selected = ExperimentSelector::new().select(&candidates).unwrap();

    assert_eq!(selected.candidate(), &candidates[0]);
}

#[test]
fn selector_does_not_mutate_candidates() {
    let model = composite_model();

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let candidates = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let before = candidates.clone();

    let _selected = ExperimentSelector::new().select(&candidates);

    assert_eq!(before, candidates);
}

#[test]
fn selection_depends_only_on_structural_information_gain() {
    let first_model = {
        let encoder = Encoder::new();

        let sequence = encoder.encode(&[1, 1, 1, 1]);

        let structure = RelationalStructure::from_sequence(&sequence);

        let concept = StructuralConcept::new(vec![
            PrimitiveSignature::new(RelationKind::Equal, 1),
            PrimitiveSignature::new(RelationKind::Equal, 2),
        ]);

        PredictiveStructuralModel::from_example(&concept, &structure).unwrap()
    };

    let second_model = {
        let encoder = Encoder::new();

        let sequence = encoder.encode(&[847, 847, 847, 847]);

        let structure = RelationalStructure::from_sequence(&sequence);

        let concept = StructuralConcept::new(vec![
            PrimitiveSignature::new(RelationKind::Equal, 1),
            PrimitiveSignature::new(RelationKind::Equal, 2),
        ]);

        PredictiveStructuralModel::from_example(&concept, &structure).unwrap()
    };

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let first_candidates = ExperimentGenerator::new()
        .generate(&first_model, &state)
        .unwrap();

    let second_candidates = ExperimentGenerator::new()
        .generate(&second_model, &state)
        .unwrap();

    let selector = ExperimentSelector::new();

    let first = selector.select(&first_candidates).unwrap();

    let second = selector.select(&second_candidates).unwrap();

    assert_eq!(first, second);
}

#[test]
fn selector_prefers_two_rules_over_one_rule() {
    let model = composite_model();

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let candidates = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let selected = ExperimentSelector::new().select(&candidates).unwrap();

    let selected_gain = selected.information_gain();

    for candidate in candidates {
        assert!(selected_gain >= candidate.information_gain());
    }

    assert_eq!(selected_gain, 2);
}

#[test]
fn selector_ignores_candidate_input_order() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let candidates = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let mut reversed = candidates.clone();

    reversed.reverse();

    let selector = ExperimentSelector::new();

    let first = selector.select(&candidates).unwrap();

    let second = selector.select(&reversed).unwrap();

    assert_eq!(first, second);
}

#[test]
fn selection_contains_no_concrete_observation_value() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let candidates = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let selected = ExperimentSelector::new().select(&candidates).unwrap();

    assert_eq!(selected.target(), 2);
    assert_eq!(selected.information_gain(), 1);

    for rule in selected.candidate().supporting_rules() {
        assert_eq!(rule.kind(), RelationKind::Equal);
    }
}
