use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_generalization::{
    RecursiveWorldRevisionGeneralizationInput, RecursiveWorldRevisionGeneralizationThreshold,
    RecursiveWorldRevisionGeneralizedStructure, RecursiveWorldRevisionGeneralizer,
};

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
    )
}

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(structural(span)))
}

fn rule(premises: &[usize], conclusions: &[usize]) -> RecursiveWorldRule {
    RecursiveWorldRule::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn observation(
    premises: &[usize],
    conclusions: &[usize],
) -> RecursiveWorldRevisionDiscoveryObservation {
    RecursiveWorldRevisionDiscoveryObservation::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn observation_set(
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionInductionObservationSet {
    RecursiveWorldRevisionInductionObservationSet::new(observations).unwrap()
}

fn threshold(
    minimum_support: usize,
    observation_count: usize,
) -> RecursiveWorldRevisionGeneralizationThreshold {
    RecursiveWorldRevisionGeneralizationThreshold::new(minimum_support, observation_count).unwrap()
}

#[test]
fn generalization_threshold_requires_support_of_at_least_two() {
    assert!(RecursiveWorldRevisionGeneralizationThreshold::new(1, 3,).is_none());
}

#[test]
fn generalization_threshold_cannot_exceed_observation_count() {
    assert!(RecursiveWorldRevisionGeneralizationThreshold::new(4, 3,).is_none());
}

#[test]
fn generalization_input_preserves_target_identity() {
    let target = rule(&[9], &[10]);

    let observations = observation_set(vec![observation(&[1], &[2]), observation(&[1, 3], &[2])]);

    let input = RecursiveWorldRevisionGeneralizationInput::new(
        target.clone(),
        observations,
        threshold(2, 2),
    )
    .unwrap();

    assert_eq!(input.target(), &target);
}

#[test]
fn generalization_keeps_premise_meeting_support_threshold() {
    let input = RecursiveWorldRevisionGeneralizationInput::new(
        rule(&[9], &[10]),
        observation_set(vec![
            observation(&[1, 2], &[7]),
            observation(&[1, 3], &[7, 8]),
            observation(&[1, 2, 4], &[7, 9]),
        ]),
        threshold(2, 3),
    )
    .unwrap();

    let generalized = RecursiveWorldRevisionGeneralizedStructure::generalize(input).unwrap();

    assert_eq!(generalized.generalized_premises(), &[unit(1,), unit(2,),]);

    assert_eq!(generalized.premise_support(&unit(2,),), 2);
}

#[test]
fn generalization_drops_premise_below_support_threshold() {
    let generalized = RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1, 2], &[7]),
                observation(&[1, 3], &[7]),
                observation(&[1, 4], &[7]),
            ]),
            threshold(2, 3),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(generalized.generalized_premises(), &[unit(1,),]);
}

#[test]
fn generalization_keeps_conclusion_meeting_support_threshold() {
    let generalized = RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1], &[2, 3]),
                observation(&[1, 5], &[2, 4]),
                observation(&[1, 6], &[2, 3, 7]),
            ]),
            threshold(2, 3),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        generalized.generalized_conclusions(),
        &[unit(2,), unit(3,),]
    );

    assert_eq!(generalized.conclusion_support(&unit(3,),), 2);
}

#[test]
fn generalization_requires_nonempty_supported_premises() {
    let result = RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1], &[7]),
                observation(&[2], &[7]),
                observation(&[3], &[7]),
            ]),
            threshold(2, 3),
        )
        .unwrap(),
    );

    assert!(result.is_none());
}

#[test]
fn generalization_requires_nonempty_supported_conclusions() {
    let result = RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1], &[2]),
                observation(&[1], &[3]),
                observation(&[1], &[4]),
            ]),
            threshold(2, 3),
        )
        .unwrap(),
    );

    assert!(result.is_none());
}

#[test]
fn full_support_threshold_matches_strict_intersection() {
    let generalized = RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1, 2], &[3, 4]),
                observation(&[1, 5], &[3, 6]),
                observation(&[1, 7], &[3, 8]),
            ]),
            threshold(3, 3),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        generalized.generalized_observation(),
        &observation(&[1], &[3],)
    );
}

#[test]
fn generalization_exposes_exact_unit_support_counts() {
    let generalized = RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1, 2], &[3]),
                observation(&[1], &[3, 4]),
                observation(&[1, 2], &[5]),
            ]),
            threshold(2, 3),
        )
        .unwrap(),
    )
    .unwrap();

    assert_eq!(generalized.premise_support(&unit(1,),), 3);

    assert_eq!(generalized.premise_support(&unit(2,),), 2);

    assert_eq!(generalized.conclusion_support(&unit(3,),), 2);

    assert_eq!(generalized.conclusion_support(&unit(99,),), 0);
}

#[test]
fn generalizer_facade_matches_direct_generalization() {
    let input = RecursiveWorldRevisionGeneralizationInput::new(
        rule(&[9], &[10]),
        observation_set(vec![
            observation(&[1, 2], &[3]),
            observation(&[1], &[3, 4]),
            observation(&[1, 2], &[3, 5]),
        ]),
        threshold(2, 3),
    )
    .unwrap();

    assert_eq!(
        RecursiveWorldRevisionGeneralizer::generalize(input.clone(),),
        RecursiveWorldRevisionGeneralizedStructure::generalize(input,)
    );
}

#[test]
fn generalization_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1], &[3, 4]);

    let third = observation(&[1, 2], &[3, 5]);

    let observations = observation_set(vec![third.clone(), first.clone(), second.clone()]);

    let observations_before = observations.clone();

    let left = RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            target.clone(),
            observations.clone(),
            threshold(2, 3),
        )
        .unwrap(),
    );

    let right = RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            target,
            observation_set(vec![second, third, first]),
            threshold(2, 3),
        )
        .unwrap(),
    );

    assert_eq!(left, right);

    assert_eq!(observations, observations_before);

    assert_eq!(left.unwrap().support_count(), 3);
}
