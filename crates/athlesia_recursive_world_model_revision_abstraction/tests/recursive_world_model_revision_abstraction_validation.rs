use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_abstraction::{
    RecursiveWorldRevisionAbstractionClass, RecursiveWorldRevisionAbstractionConsensus,
    RecursiveWorldRevisionAbstractionProjection, RecursiveWorldRevisionAbstractionRealization,
    RecursiveWorldRevisionAbstractionValidation, RecursiveWorldRevisionAbstractionValidationStatus,
    RecursiveWorldRevisionAbstractionValidator, RecursiveWorldRevisionAbstractionVocabulary,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(
        StructuralConcept::with_sequence_length(
            vec![PrimitiveSignature::new(RelationKind::Equal, span)],
            8,
        ),
    ))
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

fn class(members: &[usize]) -> RecursiveWorldRevisionAbstractionClass {
    RecursiveWorldRevisionAbstractionClass::new(members.iter().copied().map(unit).collect())
        .unwrap()
}

fn realization(
    classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionAbstractionRealization {
    let vocabulary = RecursiveWorldRevisionAbstractionVocabulary::new(classes).unwrap();

    let observations = RecursiveWorldRevisionInductionObservationSet::new(observations).unwrap();

    let projection =
        RecursiveWorldRevisionAbstractionProjection::project(vocabulary, observations).unwrap();

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection).unwrap();

    RecursiveWorldRevisionAbstractionRealization::realize(consensus)
}

fn deterministic() -> RecursiveWorldRevisionAbstractionRealization {
    realization(
        vec![class(&[1, 2]), class(&[20, 21])],
        vec![
            observation(&[1, 50], &[20, 60]),
            observation(&[1, 51], &[20, 61]),
        ],
    )
}

fn ambiguous() -> RecursiveWorldRevisionAbstractionRealization {
    realization(
        vec![class(&[1, 2]), class(&[20, 21])],
        vec![observation(&[1], &[20]), observation(&[2], &[21])],
    )
}

#[test]
fn ambiguous_abstraction_is_discovery_unavailable() {
    let target = rule(&[9], &[10]);

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        target,
        ambiguous(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionValidationStatus::DiscoveryUnavailable
    );

    assert!(result.is_discovery_unavailable());

    assert!(result.bridge().is_none());

    assert!(result.discovery_validation().is_none());
}

#[test]
fn deterministic_noop_is_discovery_unavailable() {
    let target = rule(&[1], &[20]);

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        target,
        deterministic(),
    );

    assert!(result.is_discovery_unavailable());

    assert!(result.accepted_hypothesis().is_none());
}

#[test]
fn valid_abstraction_revision_is_accepted() {
    let target = rule(&[9], &[10]);

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        target,
        deterministic(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionValidationStatus::Accepted
    );

    assert!(result.is_accepted());

    assert!(!result.is_rejected());

    assert_eq!(result.discovery_validation().unwrap().accepted_count(), 1);
}

#[test]
fn missing_target_abstraction_revision_is_rejected() {
    let target = rule(&[9], &[10]);

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        target.clone(),
        deterministic(),
    );

    assert!(result.is_rejected());

    assert!(result.accepted_hypothesis().is_none());

    assert_eq!(result.discovery_validation().unwrap().rejected_count(), 1);

    let rejected = result.rejected_hypothesis().unwrap();

    assert_eq!(rejected.target(), &target);

    assert_eq!(rejected.replacement(), &rule(&[1], &[20],));
}

#[test]
fn replacement_collision_abstraction_revision_is_rejected() {
    let target = rule(&[9], &[10]);

    let replacement = rule(&[1], &[20]);

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone(), replacement.clone()]),
        target.clone(),
        deterministic(),
    );

    assert!(result.is_rejected());

    assert!(result.accepted_hypothesis().is_none());

    assert_eq!(result.discovery_validation().unwrap().rejected_count(), 1);

    let rejected = result.rejected_hypothesis().unwrap();

    assert_eq!(rejected.target(), &target);

    assert_eq!(rejected.replacement(), &replacement);
}

#[test]
fn accepted_abstraction_preserves_hypothesis_identity() {
    let target = rule(&[9], &[10]);

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        target.clone(),
        deterministic(),
    );

    let hypothesis = result.accepted_hypothesis().unwrap();

    assert_eq!(hypothesis.target(), &target);

    assert_eq!(hypothesis.replacement(), &rule(&[1], &[20],));
}

#[test]
fn rejected_abstraction_preserves_hypothesis_identity() {
    let target = rule(&[9], &[10]);

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        target.clone(),
        deterministic(),
    );

    let hypothesis = result.rejected_hypothesis().unwrap();

    assert_eq!(hypothesis.target(), &target);

    assert_eq!(hypothesis.observation(), &observation(&[1], &[20],));
}

#[test]
fn validation_preserves_target_and_realization_identity() {
    let target = rule(&[9], &[10]);

    let realized = deterministic();

    let before = realized.clone();

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        target.clone(),
        realized,
    );

    assert_eq!(result.target(), &target);

    assert_eq!(result.realization(), &before);
}

#[test]
fn validation_preserves_source_and_vocabulary_provenance() {
    let target = rule(&[9], &[10]);

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        target,
        deterministic(),
    );

    assert_eq!(result.observation_count(), 2);

    assert!(result
        .source_observations()
        .contains(&observation(&[1, 50], &[20, 60],),));

    assert!(result
        .source_observations()
        .contains(&observation(&[1, 51], &[20, 61],),));

    assert!(result.vocabulary().class_for(&unit(1,),).is_some());

    assert!(result.vocabulary().class_for(&unit(20,),).is_some());
}

#[test]
fn validation_preserves_unique_witness_provenance() {
    let target = rule(&[9], &[10]);

    let premise = class(&[1, 2]);

    let conclusion = class(&[20, 21]);

    let result = RecursiveWorldRevisionAbstractionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        target,
        deterministic(),
    );

    assert_eq!(
        result.premise_witnesses(&premise,),
        std::slice::from_ref(&unit(1,),)
    );

    assert_eq!(
        result.conclusion_witnesses(&conclusion,),
        std::slice::from_ref(&unit(20,),)
    );
}

#[test]
fn abstraction_validator_facade_matches_direct_validation() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let realized = deterministic();

    assert_eq!(
        RecursiveWorldRevisionAbstractionValidator::validate(
            &model,
            target.clone(),
            realized.clone(),
        ),
        RecursiveWorldRevisionAbstractionValidation::new(&model, target, realized,)
    );
}

#[test]
fn abstraction_validation_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let realized = deterministic();

    let model_before = model.clone();

    let realized_before = realized.clone();

    let left =
        RecursiveWorldRevisionAbstractionValidation::new(&model, target.clone(), realized.clone());

    let right = RecursiveWorldRevisionAbstractionValidation::new(&model, target, deterministic());

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(realized, realized_before);
}
