use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_proposal::{
    RecursiveWorldRevisionProposal, RecursiveWorldRevisionProposalRejection,
    RecursiveWorldRevisionProposalSet, RecursiveWorldRevisionProposalValidationSet,
    RecursiveWorldRevisionProposalValidator,
};

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

fn proposal(
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
) -> RecursiveWorldRevisionProposal {
    RecursiveWorldRevisionProposal::new(target, replacement).unwrap()
}

#[test]
fn empty_validation_set_is_empty() {
    let set = RecursiveWorldRevisionProposalValidationSet::new(Vec::new());

    assert!(set.is_empty());

    assert_eq!(set.len(), 0);
}

#[test]
fn accepted_validation_is_partitioned_into_accepted_set() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation = RecursiveWorldRevisionProposalValidator::validate(
        &model,
        proposal(target, rule(&[1], &[3])),
    );

    let set = RecursiveWorldRevisionProposalValidationSet::new(vec![validation]);

    assert_eq!(set.accepted_count(), 1);

    assert_eq!(set.rejected_count(), 0);
}

#[test]
fn rejected_validation_is_partitioned_into_rejected_set() {
    let validation = RecursiveWorldRevisionProposalValidator::validate(
        &RecursiveWorldModel::new(Vec::new()),
        proposal(rule(&[1], &[2]), rule(&[1], &[3])),
    );

    let set = RecursiveWorldRevisionProposalValidationSet::new(vec![validation]);

    assert_eq!(set.accepted_count(), 0);

    assert_eq!(set.rejected_count(), 1);
}

#[test]
fn mixed_validations_preserve_partition_cardinality() {
    let accepted_target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![accepted_target.clone()]);

    let accepted = RecursiveWorldRevisionProposalValidator::validate(
        &model,
        proposal(accepted_target, rule(&[1], &[3])),
    );

    let rejected = RecursiveWorldRevisionProposalValidator::validate(
        &model,
        proposal(rule(&[5], &[6]), rule(&[5], &[7])),
    );

    let set = RecursiveWorldRevisionProposalValidationSet::new(vec![rejected, accepted]);

    assert_eq!(set.accepted_count(), 1);

    assert_eq!(set.rejected_count(), 1);

    assert_eq!(set.len(), 2);
}

#[test]
fn rejected_reason_is_preserved() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), replacement.clone()]);

    let set = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target, replacement)]),
    );

    assert_eq!(
        set.rejected()[0].reason(),
        RecursiveWorldRevisionProposalRejection::ReplacementCollision
    );
}

#[test]
fn accepted_revision_materialization_is_preserved() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let set = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(
            target.clone(),
            replacement.clone(),
        )]),
    );

    let revisions = set.revisions();

    assert_eq!(revisions.len(), 1);

    assert_eq!(revisions[0].target(), &target);

    assert_eq!(revisions[0].replacement(), &replacement);
}

#[test]
fn accepted_target_query_is_exact() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let set = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![
            proposal(first.clone(), rule(&[1], &[3])),
            proposal(second.clone(), rule(&[5], &[7])),
        ]),
    );

    assert_eq!(set.accepted_for_target(&first,).len(), 1);

    assert_eq!(set.accepted_for_target(&second,).len(), 1);
}

#[test]
fn rejected_target_query_is_exact() {
    let missing = rule(&[1], &[2]);

    let other_missing = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(Vec::new());

    let set = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![
            proposal(missing.clone(), rule(&[1], &[3])),
            proposal(other_missing.clone(), rule(&[5], &[7])),
        ]),
    );

    assert_eq!(set.rejected_for_target(&missing,).len(), 1);

    assert_eq!(set.rejected_for_target(&other_missing,).len(), 1);
}

#[test]
fn duplicate_validations_are_deduplicated() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let source = proposal(target, rule(&[1], &[3]));

    let validation = RecursiveWorldRevisionProposalValidator::validate(&model, source);

    let set =
        RecursiveWorldRevisionProposalValidationSet::new(vec![validation.clone(), validation]);

    assert_eq!(set.accepted_count(), 1);

    assert_eq!(set.len(), 1);
}

#[test]
fn validation_set_is_deterministic_under_input_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone()]);

    let accepted = RecursiveWorldRevisionProposalValidator::validate(
        &model,
        proposal(first, rule(&[1], &[3])),
    );

    let rejected = RecursiveWorldRevisionProposalValidator::validate(
        &model,
        proposal(second, rule(&[5], &[7])),
    );

    assert_eq!(
        RecursiveWorldRevisionProposalValidationSet::new(vec![accepted.clone(), rejected.clone(),],),
        RecursiveWorldRevisionProposalValidationSet::new(vec![rejected, accepted,],)
    );
}

#[test]
fn validator_set_matches_manual_partition() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let proposals = RecursiveWorldRevisionProposalSet::new(vec![
        proposal(target, rule(&[1], &[3])),
        proposal(rule(&[5], &[6]), rule(&[5], &[7])),
    ]);

    assert_eq!(
        RecursiveWorldRevisionProposalValidator::validate_set(&model, &proposals,),
        RecursiveWorldRevisionProposalValidationSet::new(
            RecursiveWorldRevisionProposalValidator::validate_many(&model, &proposals,),
        )
    );
}

#[test]
fn validation_set_construction_does_not_mutate_inputs() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let proposals =
        RecursiveWorldRevisionProposalSet::new(vec![proposal(target, rule(&[1], &[3]))]);

    let validations = RecursiveWorldRevisionProposalValidator::validate_many(&model, &proposals);

    let before = validations.clone();

    let _ = RecursiveWorldRevisionProposalValidationSet::new(validations.clone());

    assert_eq!(validations, before);
}
