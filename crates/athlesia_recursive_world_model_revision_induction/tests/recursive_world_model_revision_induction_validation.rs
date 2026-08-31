use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_induction::{
    RecursiveWorldRevisionInducedStructure, RecursiveWorldRevisionInductionInput,
    RecursiveWorldRevisionInductionObservationSet, RecursiveWorldRevisionInductionValidation,
    RecursiveWorldRevisionInductionValidationStatus, RecursiveWorldRevisionInductionValidator,
};

use athlesia_recursive_world_model_revision_proposal::RecursiveWorldRevisionProposalRejection;

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

fn induced(
    target: RecursiveWorldRule,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionInducedStructure {
    RecursiveWorldRevisionInducedStructure::induce(RecursiveWorldRevisionInductionInput::new(
        target,
        observation_set(observations),
    ))
    .unwrap()
}

#[test]
fn noop_induction_is_discovery_unavailable() {
    let target = rule(&[1], &[2]);

    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        induced(
            target,
            vec![observation(&[1, 3], &[2, 4]), observation(&[1, 5], &[2, 6])],
        ),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionInductionValidationStatus::DiscoveryUnavailable
    );

    assert!(validation.is_discovery_unavailable());

    assert!(validation.bridge().is_none());

    assert!(validation.discovery_validation().is_none());
}

#[test]
fn valid_induced_revision_is_accepted() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        induced(
            target,
            vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
        ),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionInductionValidationStatus::Accepted
    );

    assert!(validation.is_accepted());

    assert!(!validation.is_rejected());
}

#[test]
fn missing_target_induced_revision_is_rejected() {
    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        induced(
            rule(&[9], &[10]),
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
    );

    assert!(validation.is_rejected());

    assert_eq!(
        validation
            .discovery_validation()
            .unwrap()
            .generation_validation()
            .rejected()[0]
            .reason(),
        RecursiveWorldRevisionProposalRejection::TargetMissing
    );
}

#[test]
fn replacement_collision_induced_revision_is_rejected() {
    let target = rule(&[9], &[10]);

    let collision = rule(&[1], &[3]);

    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone(), collision]),
        induced(
            target,
            vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
        ),
    );

    assert!(validation.is_rejected());

    assert_eq!(
        validation
            .discovery_validation()
            .unwrap()
            .generation_validation()
            .rejected()[0]
            .reason(),
        RecursiveWorldRevisionProposalRejection::ReplacementCollision
    );
}

#[test]
fn accepted_induction_preserves_discovery_hypothesis_identity() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        induced(
            target.clone(),
            vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
        ),
    );

    let hypothesis = validation.accepted_hypothesis().unwrap();

    assert_eq!(hypothesis.target(), &target);

    assert_eq!(hypothesis.replacement(), &rule(&[1], &[3],));
}

#[test]
fn rejected_induction_preserves_discovery_hypothesis_identity() {
    let target = rule(&[9], &[10]);

    let structure = induced(
        target,
        vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
    );

    let expected = structure.induced_observation().clone();

    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        structure,
    );

    assert_eq!(
        validation.rejected_hypothesis().unwrap().observation(),
        &expected
    );
}

#[test]
fn induction_validation_preserves_support_count() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        induced(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 4], &[3]),
                observation(&[1, 5], &[3]),
            ],
        ),
    );

    assert_eq!(validation.support_count(), 3);
}

#[test]
fn induction_validation_preserves_source_provenance() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 4], &[3]);

    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        induced(target, vec![first.clone(), second.clone()]),
    );

    assert!(validation.source_observations().contains(&first,));

    assert!(validation.source_observations().contains(&second,));
}

#[test]
fn induction_validation_preserves_frozen_m37_validation_identity() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        induced(
            target,
            vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
        ),
    );

    assert_eq!(
        validation.discovery_validation().unwrap().accepted_count(),
        1
    );

    assert_eq!(
        validation.discovery_validation().unwrap().rejected_count(),
        0
    );
}

#[test]
fn induction_validation_preserves_target_identity() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionInductionValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        induced(
            target.clone(),
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
    );

    assert_eq!(validation.induced().target(), &target);

    assert_eq!(validation.bridge().unwrap().target(), &target);
}

#[test]
fn induction_validator_facade_matches_direct_validation() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let structure = induced(
        target,
        vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
    );

    assert_eq!(
        RecursiveWorldRevisionInductionValidator::validate(&model, structure.clone(),),
        RecursiveWorldRevisionInductionValidation::new(&model, structure,)
    );
}

#[test]
fn induction_validation_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let first = observation(&[1, 2], &[3, 4]);

    let second = observation(&[1, 5], &[3, 6]);

    let structure = induced(target.clone(), vec![second.clone(), first.clone()]);

    let model_before = model.clone();

    let structure_before = structure.clone();

    let left = RecursiveWorldRevisionInductionValidation::new(&model, structure.clone());

    let right = RecursiveWorldRevisionInductionValidation::new(
        &model,
        induced(target, vec![first, second]),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(structure, structure_before);
}
