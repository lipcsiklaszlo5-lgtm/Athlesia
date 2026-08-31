use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_generation::{
    RecursiveWorldRevisionGenerationCandidate, RecursiveWorldRevisionGenerationCandidateSet,
    RecursiveWorldRevisionGenerationValidation, RecursiveWorldRevisionGenerationValidator,
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

fn candidate(
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
    basis: &[usize],
) -> RecursiveWorldRevisionGenerationCandidate {
    RecursiveWorldRevisionGenerationCandidate::new(
        target,
        replacement,
        basis.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

#[test]
fn empty_generation_validation_is_empty() {
    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        RecursiveWorldRevisionGenerationCandidateSet::new(Vec::new()),
    );

    assert_eq!(validation.accepted_count(), 0);

    assert_eq!(validation.rejected_count(), 0);

    assert!(validation.accepted_candidates().is_empty());

    assert!(validation.rejected_candidates().is_empty());
}

#[test]
fn valid_generation_candidate_is_accepted_through_m35() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &model,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target,
            rule(&[1], &[3]),
            &[9],
        )]),
    );

    assert_eq!(validation.accepted_count(), 1);

    assert_eq!(validation.rejected_count(), 0);
}

#[test]
fn missing_target_generation_candidate_is_rejected() {
    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            rule(&[1], &[2]),
            rule(&[1], &[3]),
            &[9],
        )]),
    );

    assert_eq!(validation.accepted_count(), 0);

    assert_eq!(validation.rejected_count(), 1);

    assert_eq!(
        validation.rejected()[0].reason(),
        RecursiveWorldRevisionProposalRejection::TargetMissing
    );
}

#[test]
fn replacement_collision_generation_candidate_is_rejected() {
    let target = rule(&[1], &[2]);

    let collision = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), collision.clone()]);

    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &model,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(target, collision, &[9])]),
    );

    assert_eq!(validation.rejected_count(), 1);

    assert_eq!(
        validation.rejected()[0].reason(),
        RecursiveWorldRevisionProposalRejection::ReplacementCollision
    );
}

#[test]
fn accepted_generation_provenance_is_preserved() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let generated = candidate(target.clone(), replacement, &[9]);

    let model = RecursiveWorldModel::new(vec![target]);

    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &model,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated.clone()]),
    );

    assert_eq!(
        validation.candidates_for_accepted(&validation.accepted()[0],),
        vec![generated,]
    );
}

#[test]
fn rejected_generation_provenance_is_preserved() {
    let generated = candidate(rule(&[1], &[2]), rule(&[1], &[3]), &[9]);

    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated.clone()]),
    );

    assert_eq!(
        validation.candidates_for_rejected(&validation.rejected()[0],),
        vec![generated,]
    );
}

#[test]
fn multiple_basis_candidates_follow_same_accepted_proposal() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let first = candidate(target.clone(), replacement.clone(), &[9]);

    let second = candidate(target.clone(), replacement, &[10]);

    let model = RecursiveWorldModel::new(vec![target]);

    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &model,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![second.clone(), first.clone()]),
    );

    assert_eq!(validation.accepted_count(), 1);

    assert_eq!(
        validation.candidates_for_accepted(&validation.accepted()[0],),
        vec![first, second,]
    );
}

#[test]
fn multiple_basis_candidates_follow_same_rejected_proposal() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let first = candidate(target.clone(), replacement.clone(), &[9]);

    let second = candidate(target, replacement, &[10]);

    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &RecursiveWorldModel::new(Vec::new()),
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![second.clone(), first.clone()]),
    );

    assert_eq!(validation.rejected_count(), 1);

    assert_eq!(
        validation.candidates_for_rejected(&validation.rejected()[0],),
        vec![first, second,]
    );
}

#[test]
fn accepted_and_rejected_candidate_partitions_are_exact() {
    let accepted_target = rule(&[1], &[2]);

    let accepted_candidate = candidate(accepted_target.clone(), rule(&[1], &[3]), &[9]);

    let rejected_candidate = candidate(rule(&[5], &[6]), rule(&[5], &[7]), &[10]);

    let model = RecursiveWorldModel::new(vec![accepted_target]);

    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &model,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![
            rejected_candidate.clone(),
            accepted_candidate.clone(),
        ]),
    );

    assert_eq!(validation.accepted_candidates(), vec![accepted_candidate,]);

    assert_eq!(validation.rejected_candidates(), vec![rejected_candidate,]);
}

#[test]
fn validation_preserves_exact_m35_materialized_revision() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validation = RecursiveWorldRevisionGenerationValidation::new(
        &model,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target.clone(),
            replacement.clone(),
            &[9],
        )]),
    );

    let revision = validation.accepted()[0].revision();

    assert_eq!(revision.target(), &target);

    assert_eq!(revision.replacement(), &replacement);

    assert!(revision.after().contains(&replacement,));
}

#[test]
fn validator_facade_matches_direct_construction() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let candidates = RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
        target,
        rule(&[1], &[3]),
        &[9],
    )]);

    assert_eq!(
        RecursiveWorldRevisionGenerationValidator::validate(&model, candidates.clone(),),
        RecursiveWorldRevisionGenerationValidation::new(&model, candidates,)
    );
}

#[test]
fn generation_validation_does_not_mutate_model_or_candidate_set() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let candidates = RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
        target,
        rule(&[1], &[3]),
        &[9],
    )]);

    let model_before = model.clone();

    let candidates_before = candidates.clone();

    let _ = RecursiveWorldRevisionGenerationValidation::new(&model, candidates.clone());

    assert_eq!(model, model_before);

    assert_eq!(candidates, candidates_before);
}
