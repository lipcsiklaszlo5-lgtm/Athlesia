use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionTransferValidation,
    RecursiveWorldRevisionAbstractionTransferValidationStatus,
    RecursiveWorldRevisionAbstractionTransferValidator,
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

fn induction_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1], &[10]),
        observation(&[2], &[10]),
        observation(&[1], &[20]),
        observation(&[2], &[20]),
    ])
}

fn deterministic_transfer_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 50], &[10, 60]),
        observation(&[1, 51], &[10, 61]),
    ])
}

#[test]
fn unavailable_transfer_discovery_is_validation_unavailable() {
    let target = rule(&[9], &[99]);

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![target.clone()]),
        target,
        induction_source(),
        observation_set(vec![observation(&[1], &[10]), observation(&[2], &[10])]),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionTransferValidationStatus::DiscoveryUnavailable
    );

    assert!(validation.validation().is_none());

    assert!(validation.accepted_hypothesis().is_none());

    assert!(validation.rejected_hypothesis().is_none());
}

#[test]
fn transfer_noop_is_validation_unavailable() {
    let target = rule(&[1], &[10]);

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![target.clone()]),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionTransferValidationStatus::DiscoveryUnavailable
    );

    assert!(validation.hypothesis().is_none());
}

#[test]
fn valid_transfer_revision_is_accepted() {
    let target = rule(&[9], &[99]);

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![target.clone(), rule(&[30], &[40])]),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionTransferValidationStatus::Accepted
    );

    assert!(validation.is_accepted());

    assert!(!validation.is_rejected());

    assert_eq!(validation.validation().unwrap().accepted_count(), 1);

    assert_eq!(validation.validation().unwrap().rejected_count(), 0);
}

#[test]
fn missing_target_transfer_revision_is_rejected() {
    let target = rule(&[9], &[99]);

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![rule(&[30], &[40])]),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionTransferValidationStatus::Rejected
    );

    assert!(validation.is_rejected());

    assert_eq!(validation.validation().unwrap().accepted_count(), 0);

    assert_eq!(validation.validation().unwrap().rejected_count(), 1);
}

#[test]
fn replacement_collision_transfer_revision_is_rejected() {
    let target = rule(&[9], &[99]);

    let replacement = rule(&[1], &[10]);

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![target.clone(), replacement]),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionAbstractionTransferValidationStatus::Rejected
    );

    assert!(validation.is_rejected());

    assert_eq!(validation.validation().unwrap().rejected_count(), 1);
}

#[test]
fn accepted_validation_preserves_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![target.clone()]),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(validation.accepted_hypothesis(), validation.hypothesis());

    assert_eq!(
        validation.accepted_hypothesis().unwrap().replacement(),
        &rule(&[1], &[10],)
    );
}

#[test]
fn rejected_validation_preserves_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![rule(&[30], &[40])]),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(validation.rejected_hypothesis(), validation.hypothesis());
}

#[test]
fn validation_preserves_target_and_replacement_identity() {
    let target = rule(&[9], &[99]);

    let expected_replacement = rule(&[1], &[10]);

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![target.clone()]),
        target.clone(),
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(validation.target(), &target);

    assert_eq!(validation.replacement(), Some(&expected_replacement,));
}

#[test]
fn validation_preserves_transfer_realization_provenance() {
    let target = rule(&[9], &[99]);

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![target.clone()]),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        validation.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );

    assert_eq!(
        validation.hypothesis().unwrap().observation(),
        &observation(&[1], &[10],)
    );
}

#[test]
fn validation_preserves_learning_and_application_provenance() {
    let target = rule(&[9], &[99]);

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    let induction_before = induction.clone();

    let transfer_before = transfer.clone();

    let validation = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![target.clone()]),
        target,
        induction,
        transfer,
    );

    assert_eq!(validation.induction_observations(), &induction_before);

    assert_eq!(validation.transfer_observations(), &transfer_before);

    assert_eq!(validation.consensus().unwrap().observation_count(), 4);

    assert_eq!(validation.vocabulary().unwrap().classes().len(), 2);
}

#[test]
fn transfer_validator_facade_matches_direct_validation() {
    let target = rule(&[9], &[99]);

    let world = model(vec![target.clone(), rule(&[30], &[40])]);

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionTransferValidator::validate(
            world.clone(),
            target.clone(),
            induction.clone(),
            transfer.clone(),
        ),
        RecursiveWorldRevisionAbstractionTransferValidation::validate(
            world, target, induction, transfer,
        )
    );
}

#[test]
fn transfer_validation_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let world = model(vec![target.clone(), rule(&[30], &[40])]);

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    let world_before = world.clone();

    let induction_before = induction.clone();

    let transfer_before = transfer.clone();

    let left = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        world.clone(),
        target.clone(),
        induction.clone(),
        transfer.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionTransferValidation::validate(
        model(vec![rule(&[30], &[40]), target]),
        rule(&[9], &[99]),
        observation_set(vec![
            observation(&[2], &[20]),
            observation(&[1], &[20]),
            observation(&[2], &[10]),
            observation(&[1], &[10]),
        ]),
        observation_set(vec![
            observation(&[1, 51], &[10, 61]),
            observation(&[1, 50], &[10, 60]),
        ]),
    );

    assert_eq!(left, right);

    assert_eq!(world, world_before);

    assert_eq!(induction, induction_before);

    assert_eq!(transfer, transfer_before);

    assert!(left.is_accepted());
}
