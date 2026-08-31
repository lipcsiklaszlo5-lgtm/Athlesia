use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_proposal::{
    RecursiveWorldRevisionProposal, RecursiveWorldRevisionProposalRejection,
    RecursiveWorldRevisionProposalSet, RecursiveWorldRevisionProposalValidator,
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
fn existing_target_and_new_replacement_are_accepted() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation =
        RecursiveWorldRevisionProposalValidator::validate(&model, proposal(target, replacement));

    assert!(validation.is_accepted());

    assert!(!validation.is_rejected());
}

#[test]
fn missing_target_is_rejected() {
    let existing = rule(&[5], &[6]);

    let missing = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![existing]);

    let validation = RecursiveWorldRevisionProposalValidator::validate(
        &model,
        proposal(missing, rule(&[1], &[3])),
    );

    assert_eq!(
        validation.rejection_reason(),
        Some(RecursiveWorldRevisionProposalRejection::TargetMissing,)
    );
}

#[test]
fn existing_replacement_is_rejected_as_collision() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), replacement.clone()]);

    let validation =
        RecursiveWorldRevisionProposalValidator::validate(&model, proposal(target, replacement));

    assert_eq!(
        validation.rejection_reason(),
        Some(RecursiveWorldRevisionProposalRejection::ReplacementCollision,)
    );
}

#[test]
fn accepted_validation_preserves_proposal_identity() {
    let target = rule(&[1], &[2]);

    let source = proposal(target.clone(), rule(&[1], &[3]));

    let model = RecursiveWorldModel::new(vec![target]);

    let validation = RecursiveWorldRevisionProposalValidator::validate(&model, source.clone());

    assert_eq!(validation.proposal(), &source);
}

#[test]
fn rejected_validation_preserves_proposal_identity() {
    let source = proposal(rule(&[1], &[2]), rule(&[1], &[3]));

    let validation = RecursiveWorldRevisionProposalValidator::validate(
        &RecursiveWorldModel::new(Vec::new()),
        source.clone(),
    );

    assert_eq!(validation.proposal(), &source);
}

#[test]
fn accepted_validation_preserves_target_and_replacement_identity() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation = RecursiveWorldRevisionProposalValidator::validate(
        &model,
        proposal(target.clone(), replacement.clone()),
    );

    let validated = validation.validated().unwrap();

    assert_eq!(validated.target(), &target);

    assert_eq!(validated.replacement(), &replacement);
}

#[test]
fn accepted_validation_materializes_exact_m33_revision() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let unrelated = rule(&[8], &[9]);

    let model = RecursiveWorldModel::new(vec![target.clone(), unrelated.clone()]);

    let validation = RecursiveWorldRevisionProposalValidator::validate(
        &model,
        proposal(target.clone(), replacement.clone()),
    );

    let revision = validation.validated().unwrap().revision();

    assert_eq!(revision.target(), &target);

    assert_eq!(revision.replacement(), &replacement);

    assert!(revision.after().contains(&replacement,));

    assert!(revision.after().contains(&unrelated,));

    assert!(!revision.after().contains(&target,));
}

#[test]
fn accepted_validation_has_no_rejection_reason() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation = RecursiveWorldRevisionProposalValidator::validate(
        &model,
        proposal(target, rule(&[1], &[3])),
    );

    assert_eq!(validation.rejection_reason(), None);
}

#[test]
fn rejected_validation_has_no_validated_revision() {
    let validation = RecursiveWorldRevisionProposalValidator::validate(
        &RecursiveWorldModel::new(Vec::new()),
        proposal(rule(&[1], &[2]), rule(&[1], &[3])),
    );

    assert!(validation.validated().is_none());
}

#[test]
fn validate_many_preserves_canonical_proposal_order() {
    let first_target = rule(&[1], &[2]);

    let second_target = rule(&[5], &[6]);

    let first = proposal(first_target.clone(), rule(&[1], &[3]));

    let second = proposal(second_target.clone(), rule(&[5], &[7]));

    let model = RecursiveWorldModel::new(vec![first_target, second_target]);

    let set = RecursiveWorldRevisionProposalSet::new(vec![second.clone(), first.clone()]);

    let validations = RecursiveWorldRevisionProposalValidator::validate_many(&model, &set);

    assert_eq!(validations.len(), 2);

    assert_eq!(validations[0].proposal(), set.proposals().first().unwrap());

    assert_eq!(validations[1].proposal(), set.proposals().get(1,).unwrap());
}

#[test]
fn validation_is_deterministic() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let source = proposal(target, rule(&[1], &[3]));

    assert_eq!(
        RecursiveWorldRevisionProposalValidator::validate(&model, source.clone(),),
        RecursiveWorldRevisionProposalValidator::validate(&model, source,)
    );
}

#[test]
fn validation_does_not_mutate_model_or_proposal_set() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let set = RecursiveWorldRevisionProposalSet::new(vec![proposal(target, rule(&[1], &[3]))]);

    let model_before = model.clone();

    let set_before = set.clone();

    let _ = RecursiveWorldRevisionProposalValidator::validate_many(&model, &set);

    assert_eq!(model, model_before);

    assert_eq!(set, set_before);
}
