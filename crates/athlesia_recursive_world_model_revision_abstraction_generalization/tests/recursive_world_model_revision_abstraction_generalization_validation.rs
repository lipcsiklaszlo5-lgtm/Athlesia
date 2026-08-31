use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_abstraction_generalization::{
    RecursiveWorldRevisionAbstractionGeneralizationThreshold,
    RecursiveWorldRevisionAbstractionGeneralizationValidation,
    RecursiveWorldRevisionAbstractionGeneralizationValidationStatus,
    RecursiveWorldRevisionAbstractionGeneralizationValidator,
    RecursiveWorldRevisionAbstractionGeneralizedClassSet,
};

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInducedClassSet,
    RecursiveWorldRevisionAbstractionSubstitutionWitness,
    RecursiveWorldRevisionAbstractionSubstitutionWitnessSet,
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

fn model(rules: Vec<RecursiveWorldRule>) -> RecursiveWorldModel {
    RecursiveWorldModel::new(rules)
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

fn premise_witness(
    first: usize,
    second: usize,
    shared: usize,
    fixed_conclusion: usize,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[first, shared], &[fixed_conclusion]),
        observation(&[second, shared], &[fixed_conclusion]),
    )
    .unwrap()
}

fn conclusion_witness(
    first: usize,
    second: usize,
    shared: usize,
    fixed_premise: usize,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[fixed_premise], &[first, shared]),
        observation(&[fixed_premise], &[second, shared]),
    )
    .unwrap()
}

fn induced(
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
) -> RecursiveWorldRevisionAbstractionInducedClassSet {
    RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(witnesses).unwrap(),
    )
    .unwrap()
}

fn generalized(
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
) -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        induced(witnesses),
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap()
}

fn generalized_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    generalized(vec![
        premise_witness(1, 2, 30, 40),
        premise_witness(1, 2, 31, 41),
        conclusion_witness(10, 20, 50, 60),
        conclusion_witness(10, 20, 51, 61),
    ])
}

fn deterministic_application() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 70], &[10, 80]),
        observation(&[1, 71], &[10, 81]),
    ])
}

fn ambiguous_application() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 70], &[10, 80]),
        observation(&[2, 71], &[10, 81]),
    ])
}

#[test]
fn ambiguous_discovery_is_validation_unavailable() {
    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        model(vec![rule(&[9], &[99])]),
        rule(&[9], &[99]),
        generalized_source(),
        ambiguous_application(),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionGeneralizationValidationStatus::DiscoveryUnavailable
    );

    assert!(validation.validation().is_none());
}

#[test]
fn noop_discovery_is_validation_unavailable() {
    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        model(vec![rule(&[1], &[10])]),
        rule(&[1], &[10]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionGeneralizationValidationStatus::DiscoveryUnavailable
    );

    assert!(validation.validation().is_none());
}

#[test]
fn missing_target_is_rejected() {
    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        model(vec![rule(&[8], &[88])]),
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionGeneralizationValidationStatus::Rejected
    );

    assert!(validation.is_rejected());

    assert!(validation.rejected_hypothesis().is_some());
}

#[test]
fn replacement_collision_is_rejected() {
    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        model(vec![rule(&[9], &[99]), rule(&[1], &[10])]),
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionGeneralizationValidationStatus::Rejected
    );

    assert!(validation.rejected_hypothesis().is_some());
}

#[test]
fn valid_generalized_discovery_is_accepted() {
    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        model(vec![rule(&[9], &[99])]),
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionGeneralizationValidationStatus::Accepted
    );

    assert!(validation.is_accepted());

    assert!(validation.accepted_hypothesis().is_some());
}

#[test]
fn accepted_validation_preserves_hypothesis_identity() {
    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        model(vec![rule(&[9], &[99])]),
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(validation.accepted_hypothesis(), validation.hypothesis());
}

#[test]
fn rejected_validation_preserves_hypothesis_identity() {
    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        model(vec![rule(&[9], &[99]), rule(&[1], &[10])]),
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(validation.rejected_hypothesis(), validation.hypothesis());
}

#[test]
fn validation_preserves_target_and_replacement_identity() {
    let target = rule(&[9], &[99]);

    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        model(vec![target.clone()]),
        target.clone(),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(validation.target(), &target);

    assert_eq!(validation.replacement(), Some(&rule(&[1], &[10],),));
}

#[test]
fn validation_preserves_model_identity() {
    let world = model(vec![rule(&[9], &[99])]);

    let before = world.clone();

    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        world,
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(validation.model(), &before);
}

#[test]
fn validation_preserves_generalization_and_application_provenance() {
    let source = generalized_source();

    let application = deterministic_application();

    let source_before = source.clone();

    let application_before = application.clone();

    let validation = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        model(vec![rule(&[9], &[99])]),
        rule(&[9], &[99]),
        source,
        application,
    );

    assert_eq!(validation.generalized_source(), &source_before);

    assert_eq!(validation.application_observations(), &application_before);

    assert_eq!(
        validation.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );

    assert!(validation.consensus().is_some());

    assert!(validation.vocabulary().is_some());
}

#[test]
fn generalized_validator_facade_matches_direct_validation() {
    let world = model(vec![rule(&[9], &[99])]);

    let target = rule(&[9], &[99]);

    let source = generalized_source();

    let application = deterministic_application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionGeneralizationValidator::validate(
            world.clone(),
            target.clone(),
            source.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
            world,
            target,
            source,
            application,
        )
    );
}

#[test]
fn generalized_validation_is_canonical_deterministic_and_non_mutating() {
    let world = model(vec![rule(&[9], &[99])]);

    let target = rule(&[9], &[99]);

    let source = generalized_source();

    let application = deterministic_application();

    let world_before = world.clone();

    let target_before = target.clone();

    let source_before = source.clone();

    let application_before = application.clone();

    let left = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        world.clone(),
        target.clone(),
        source.clone(),
        application.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionGeneralizationValidation::validate(
        world.clone(),
        target.clone(),
        generalized(vec![
            conclusion_witness(10, 20, 51, 61),
            premise_witness(1, 2, 31, 41),
            conclusion_witness(10, 20, 50, 60),
            premise_witness(1, 2, 30, 40),
        ]),
        observation_set(vec![
            observation(&[1, 71], &[10, 81]),
            observation(&[1, 70], &[10, 80]),
        ]),
    );

    assert_eq!(left, right);

    assert_eq!(world, world_before);

    assert_eq!(target, target_before);

    assert_eq!(source, source_before);

    assert_eq!(application, application_before);

    assert!(left.is_accepted());
}
