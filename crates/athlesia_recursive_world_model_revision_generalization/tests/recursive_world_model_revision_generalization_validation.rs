use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_generalization::{
    RecursiveWorldRevisionGeneralizationInput, RecursiveWorldRevisionGeneralizationThreshold,
    RecursiveWorldRevisionGeneralizationValidation,
    RecursiveWorldRevisionGeneralizationValidationStatus,
    RecursiveWorldRevisionGeneralizationValidator, RecursiveWorldRevisionGeneralizedStructure,
};

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

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

fn threshold(
    minimum_support: usize,
    observation_count: usize,
) -> RecursiveWorldRevisionGeneralizationThreshold {
    RecursiveWorldRevisionGeneralizationThreshold::new(minimum_support, observation_count).unwrap()
}

fn generalized(
    target: RecursiveWorldRule,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    minimum_support: usize,
) -> RecursiveWorldRevisionGeneralizedStructure {
    let set = observation_set(observations);

    let count = set.len();

    RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            target,
            set,
            threshold(minimum_support, count),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn noop_generalization_is_discovery_unavailable() {
    let target = rule(&[1], &[2]);

    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        generalized(
            target,
            vec![
                observation(&[1, 3], &[2, 4]),
                observation(&[1, 5], &[2, 6]),
                observation(&[1, 7], &[2, 8]),
            ],
            3,
        ),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionGeneralizationValidationStatus::DiscoveryUnavailable
    );

    assert!(validation.is_discovery_unavailable());

    assert!(validation.bridge().is_none());

    assert!(validation.discovery_validation().is_none());
}

#[test]
fn valid_generalized_revision_is_accepted() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
    );

    assert_eq!(
        validation.status(),
        RecursiveWorldRevisionGeneralizationValidationStatus::Accepted
    );

    assert!(validation.is_accepted());

    assert!(!validation.is_rejected());
}

#[test]
fn missing_target_generalized_revision_is_rejected() {
    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        generalized(
            rule(&[9], &[10]),
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
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
fn replacement_collision_generalized_revision_is_rejected() {
    let target = rule(&[9], &[10]);

    let collision = rule(&[1, 2], &[3]);

    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(vec![target.clone(), collision]),
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
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
fn accepted_generalization_preserves_hypothesis_identity() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        generalized(
            target.clone(),
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
    );

    let hypothesis = validation.accepted_hypothesis().unwrap();

    assert_eq!(hypothesis.target(), &target);

    assert_eq!(hypothesis.replacement(), &rule(&[1, 2], &[3],));
}

#[test]
fn rejected_generalization_preserves_hypothesis_identity() {
    let structure = generalized(
        rule(&[9], &[10]),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3, 5]),
            observation(&[1, 6], &[3]),
        ],
        2,
    );

    let expected = structure.generalized_observation().clone();

    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        structure,
    );

    assert_eq!(
        validation.rejected_hypothesis().unwrap().observation(),
        &expected
    );
}

#[test]
fn validation_preserves_threshold_identity() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
    );

    assert_eq!(validation.threshold().minimum_support(), 2);
}

#[test]
fn validation_preserves_distinct_observation_support_count() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
    );

    assert_eq!(validation.support_count(), 3);
}

#[test]
fn validation_preserves_source_provenance() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 2, 4], &[3, 5]);

    let third = observation(&[1, 6], &[3]);

    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        generalized(
            target,
            vec![third.clone(), first.clone(), second.clone()],
            2,
        ),
    );

    assert!(validation.source_observations().contains(&first,));

    assert!(validation.source_observations().contains(&second,));

    assert!(validation.source_observations().contains(&third,));
}

#[test]
fn validation_preserves_unit_support_counts() {
    let target = rule(&[9], &[10]);

    let validation = RecursiveWorldRevisionGeneralizationValidation::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
    );

    assert_eq!(validation.premise_support(&unit(1,),), 3);

    assert_eq!(validation.premise_support(&unit(2,),), 2);

    assert_eq!(validation.conclusion_support(&unit(3,),), 3);
}

#[test]
fn generalization_validator_facade_matches_direct_validation() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let structure = generalized(
        target,
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3, 5]),
            observation(&[1, 6], &[3]),
        ],
        2,
    );

    assert_eq!(
        RecursiveWorldRevisionGeneralizationValidator::validate(&model, structure.clone(),),
        RecursiveWorldRevisionGeneralizationValidation::new(&model, structure,)
    );
}

#[test]
fn generalization_validation_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 2, 4], &[3, 5]);

    let third = observation(&[1, 6], &[3]);

    let structure = generalized(
        target.clone(),
        vec![third.clone(), first.clone(), second.clone()],
        2,
    );

    let model_before = model.clone();

    let structure_before = structure.clone();

    let left = RecursiveWorldRevisionGeneralizationValidation::new(&model, structure.clone());

    let right = RecursiveWorldRevisionGeneralizationValidation::new(
        &model,
        generalized(target, vec![second, third, first], 2),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(structure, structure_before);
}
