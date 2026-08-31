use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryHypothesis, RecursiveWorldRevisionDiscoveryHypothesisSet,
    RecursiveWorldRevisionDiscoveryObservation, RecursiveWorldRevisionDiscoveryValidation,
    RecursiveWorldRevisionDiscoveryValidator,
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

fn hypothesis(
    target: RecursiveWorldRule,
    premises: &[usize],
    conclusions: &[usize],
) -> RecursiveWorldRevisionDiscoveryHypothesis {
    RecursiveWorldRevisionDiscoveryHypothesis::discover(target, observation(premises, conclusions))
        .unwrap()
}

#[test]
fn empty_discovery_validation_is_empty() {
    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(Vec::new()),
    );

    assert_eq!(validation.hypothesis_count(), 0);

    assert_eq!(validation.candidate_count(), 0);

    assert_eq!(validation.accepted_count(), 0);

    assert_eq!(validation.rejected_count(), 0);
}

#[test]
fn valid_discovered_hypothesis_is_accepted_through_m36() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &model,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(target, &[1], &[3])]),
    );

    assert_eq!(validation.accepted_count(), 1);

    assert_eq!(validation.rejected_count(), 0);
}

#[test]
fn missing_target_discovered_hypothesis_is_rejected() {
    let discovered = hypothesis(rule(&[1], &[2]), &[1], &[3]);

    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    assert_eq!(validation.accepted_count(), 0);

    assert_eq!(validation.rejected_hypotheses(), &[discovered,]);

    assert_eq!(
        validation.generation_validation().rejected()[0].reason(),
        RecursiveWorldRevisionProposalRejection::TargetMissing
    );
}

#[test]
fn replacement_collision_discovered_hypothesis_is_rejected() {
    let target = rule(&[1], &[2]);

    let collision = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), collision.clone()]);

    let discovered = hypothesis(target, &[5], &[6]);

    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &model,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    assert_eq!(validation.rejected_hypotheses(), &[discovered,]);

    assert_eq!(
        validation.generation_validation().rejected()[0].reason(),
        RecursiveWorldRevisionProposalRejection::ReplacementCollision
    );
}

#[test]
fn accepted_discovery_hypothesis_preserves_target_identity() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &model,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
            target.clone(),
            &[1],
            &[3],
        )]),
    );

    assert_eq!(validation.accepted_hypotheses()[0].target(), &target);
}

#[test]
fn accepted_discovery_hypothesis_preserves_replacement_identity() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &model,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(target, &[1], &[3])]),
    );

    assert_eq!(
        validation.accepted_hypotheses()[0].replacement(),
        &replacement
    );
}

#[test]
fn accepted_discovery_hypothesis_preserves_observation_identity() {
    let target = rule(&[1], &[2]);

    let observed = observation(&[1], &[3]);

    let discovered =
        RecursiveWorldRevisionDiscoveryHypothesis::discover(target.clone(), observed.clone())
            .unwrap();

    let model = RecursiveWorldModel::new(vec![target]);

    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &model,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered]),
    );

    assert_eq!(validation.accepted_hypotheses()[0].observation(), &observed);
}

#[test]
fn accepted_and_rejected_hypotheses_partition_exactly() {
    let accepted_target = rule(&[1], &[2]);

    let accepted = hypothesis(accepted_target.clone(), &[1], &[3]);

    let rejected = hypothesis(rule(&[5], &[6]), &[5], &[7]);

    let model = RecursiveWorldModel::new(vec![accepted_target]);

    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &model,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![rejected.clone(), accepted.clone()]),
    );

    assert_eq!(validation.accepted_hypotheses(), &[accepted,]);

    assert_eq!(validation.rejected_hypotheses(), &[rejected,]);

    assert_eq!(
        validation.accepted_count() + validation.rejected_count(),
        validation.hypothesis_count()
    );
}

#[test]
fn discovery_validation_preserves_generation_candidate_identity() {
    let target = rule(&[1], &[2]);

    let discovered = hypothesis(target.clone(), &[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target]);

    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &model,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    let expected_candidate = validation
        .bridge()
        .candidate_for_hypothesis(&discovered)
        .unwrap();

    assert_eq!(
        validation.generation_validation().accepted_candidates(),
        vec![expected_candidate,]
    );
}

#[test]
fn discovery_validation_preserves_frozen_m35_revision_identity() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation = RecursiveWorldRevisionDiscoveryValidation::new(
        &model,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
            target.clone(),
            &[1],
            &[3],
        )]),
    );

    let revision = validation.generation_validation().accepted()[0].revision();

    assert_eq!(revision.target(), &target);

    assert_eq!(revision.replacement(), &replacement);

    assert!(revision.after().contains(&replacement,));
}

#[test]
fn discovery_validator_facade_matches_direct_construction() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let hypotheses =
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(target, &[1], &[3])]);

    assert_eq!(
        RecursiveWorldRevisionDiscoveryValidator::validate(&model, hypotheses.clone(),),
        RecursiveWorldRevisionDiscoveryValidation::new(&model, hypotheses,)
    );
}

#[test]
fn discovery_validation_is_deterministic_and_non_mutating() {
    let first_target = rule(&[1], &[2]);

    let second_target = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first_target.clone(), second_target.clone()]);

    let first = hypothesis(first_target, &[1], &[3]);

    let second = hypothesis(second_target, &[5], &[7]);

    let hypotheses =
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![second.clone(), first.clone()]);

    let model_before = model.clone();

    let hypotheses_before = hypotheses.clone();

    let left = RecursiveWorldRevisionDiscoveryValidation::new(&model, hypotheses.clone());

    let right = RecursiveWorldRevisionDiscoveryValidation::new(
        &model,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![first, second]),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(hypotheses, hypotheses_before);
}
