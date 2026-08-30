use athlesia::{
    Encoder, ExperimentGenerator, PartialStructuralState, PredictionError, PredictionRule,
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
fn predictions_generate_experiment_candidates() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    assert_eq!(experiments.len(), 2);

    assert_eq!(experiments[0].target(), 2);

    assert_eq!(experiments[1].target(), 3);
}

#[test]
fn simple_candidate_has_one_unit_of_information_gain() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    assert_eq!(experiments[0].information_gain(), 1);

    assert_eq!(experiments[1].information_gain(), 1);
}

#[test]
fn candidate_contains_supporting_structural_rule() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    assert_eq!(
        experiments[0].supporting_rules(),
        &[PredictionRule::new_equal(0, 2,),]
    );

    assert_eq!(
        experiments[1].supporting_rules(),
        &[PredictionRule::new_equal(1, 3,),]
    );
}

#[test]
fn multiple_predictions_for_one_target_are_grouped() {
    let model = composite_model();

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let target_two = experiments
        .iter()
        .find(|experiment| experiment.target() == 2)
        .unwrap();

    assert_eq!(target_two.information_gain(), 2);

    assert_eq!(
        target_two.supporting_rules(),
        &[
            PredictionRule::new_equal(0, 2,),
            PredictionRule::new_equal(1, 2,),
        ]
    );
}

#[test]
fn information_gain_is_structural_rule_count() {
    let model = composite_model();

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    for experiment in experiments {
        assert_eq!(
            experiment.information_gain(),
            experiment.supporting_rules().len()
        );
    }
}

#[test]
fn observed_target_generates_no_experiment() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1, 2]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    assert_eq!(experiments.len(), 1);

    assert_eq!(experiments[0].target(), 3);
}

#[test]
fn fully_observed_state_generates_no_experiment() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1, 2, 3, 4]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    assert!(experiments.is_empty());
}

#[test]
fn no_observed_reference_generates_no_experiment() {
    let model = alternating_model();

    let state = PartialStructuralState::new(5);

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    assert!(experiments.is_empty());
}

#[test]
fn experiment_generation_is_deterministic() {
    let model = composite_model();

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let generator = ExperimentGenerator::new();

    let first = generator.generate(&model, &state).unwrap();

    let second = generator.generate(&model, &state).unwrap();

    assert_eq!(first, second);
}

#[test]
fn experiment_order_is_target_canonical() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    let targets: Vec<usize> = experiments
        .iter()
        .map(|experiment| experiment.target())
        .collect();

    assert_eq!(targets, vec![2, 3]);
}

#[test]
fn experiment_generation_is_value_independent() {
    let encoder = Encoder::new();

    let first_sequence = encoder.encode(&[1, 2, 1, 2, 3]);

    let second_sequence = encoder.encode(&[847, 13, 847, 13, 999]);

    let first_structure = RelationalStructure::from_sequence(&first_sequence);

    let second_structure = RelationalStructure::from_sequence(&second_sequence);

    let concept = StructuralConcept::new(vec![PrimitiveSignature::new(RelationKind::Equal, 2)]);

    let first_model = PredictiveStructuralModel::from_example(&concept, &first_structure).unwrap();

    let second_model =
        PredictiveStructuralModel::from_example(&concept, &second_structure).unwrap();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let generator = ExperimentGenerator::new();

    let first = generator.generate(&first_model, &state).unwrap();

    let second = generator.generate(&second_model, &state).unwrap();

    assert_eq!(first, second);
}

#[test]
fn experiment_candidate_contains_no_concrete_value() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(5, &[0, 1]).unwrap();

    let experiments = ExperimentGenerator::new().generate(&model, &state).unwrap();

    for experiment in experiments {
        assert!(experiment.target() < model.sequence_length());

        for rule in experiment.supporting_rules() {
            assert_eq!(rule.kind(), RelationKind::Equal);
        }
    }
}

#[test]
fn length_mismatch_is_propagated() {
    let model = alternating_model();

    let state = PartialStructuralState::from_observed_positions(4, &[0, 1]).unwrap();

    let result = ExperimentGenerator::new().generate(&model, &state);

    assert_eq!(
        result,
        Err(PredictionError::LengthMismatch {
            expected: 5,
            actual: 4,
        })
    );
}
