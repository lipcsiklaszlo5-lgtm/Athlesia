use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_generation::{
    RecursiveWorldRevisionGenerationCandidate, RecursiveWorldRevisionGenerationCandidateSet,
    RecursiveWorldRevisionGenerationEvidenceScope, RecursiveWorldRevisionGenerationEvidenceScoper,
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

fn evidence(
    source: RecursiveWorldRule,
    observation: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(source, unit(observation), kind)
}

#[test]
fn empty_inputs_produce_empty_generation_scope() {
    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        RecursiveWorldRevisionGenerationCandidateSet::new(Vec::new()),
    );

    assert_eq!(scope.active_candidate_count(), 0);

    assert_eq!(scope.inactive_candidate_count(), 0);

    assert_eq!(scope.rejected_candidate_count(), 0);

    assert!(scope.pressured_rule().is_none());
}

#[test]
fn valid_candidate_without_evidence_is_inactive() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let generated = candidate(target, rule(&[1], &[3]), &[9]);

    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated.clone()]),
    );

    assert_eq!(scope.active_candidate_count(), 0);

    assert_eq!(scope.inactive_candidates(), &[generated,]);
}

#[test]
fn confirming_evidence_keeps_candidate_inactive() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let generated = candidate(target, rule(&[1], &[3]), &[9]);

    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated.clone()]),
    );

    assert!(!scope.has_negative_pressure());

    assert_eq!(scope.inactive_candidates(), &[generated,]);
}

#[test]
fn balanced_evidence_keeps_candidate_inactive() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 3, RecursiveWorldEvidenceKind::Violating),
    ]);

    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target,
            rule(&[1], &[4]),
            &[9],
        )]),
    );

    assert!(!scope.has_negative_pressure());

    assert_eq!(scope.active_candidate_count(), 0);

    assert_eq!(scope.inactive_candidate_count(), 1);
}

#[test]
fn negative_pressure_activates_matching_candidate() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let generated = candidate(target.clone(), rule(&[1], &[3]), &[10]);

    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated.clone()]),
    );

    assert_eq!(scope.pressured_rule(), Some(&target,));

    assert_eq!(scope.active_candidates(), &[generated,]);

    assert_eq!(scope.inactive_candidate_count(), 0);
}

#[test]
fn accepted_candidate_for_other_target_remains_inactive() {
    let pressured = rule(&[1], &[2]);

    let other = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![pressured.clone(), other.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let generated = candidate(other, rule(&[5], &[7]), &[10]);

    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated.clone()]),
    );

    assert_eq!(scope.active_candidate_count(), 0);

    assert_eq!(scope.inactive_candidates(), &[generated,]);
}

#[test]
fn rejected_generation_candidate_never_becomes_active() {
    let target = rule(&[1], &[2]);

    let collision = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), collision.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let rejected = candidate(target, collision, &[10]);

    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![rejected.clone()]),
    );

    assert_eq!(scope.active_candidate_count(), 0);

    assert_eq!(scope.inactive_candidate_count(), 0);

    assert_eq!(scope.rejected_candidates(), &[rejected,]);
}

#[test]
fn multiple_basis_candidates_for_active_proposal_are_preserved() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let first = candidate(target.clone(), replacement.clone(), &[10]);

    let second = candidate(target, replacement, &[11]);

    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![second.clone(), first.clone()]),
    );

    assert_eq!(scope.active_candidates(), &[first, second,]);
}

#[test]
fn accepted_partition_cardinality_is_preserved() {
    let pressured = rule(&[1], &[2]);

    let other = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![pressured.clone(), other.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![
            candidate(pressured, rule(&[1], &[3]), &[10]),
            candidate(other, rule(&[5], &[7]), &[11]),
        ]),
    );

    assert_eq!(
        scope.accepted_candidate_count(),
        scope.validation().accepted_candidates().len()
    );

    assert_eq!(
        scope.active_candidate_count() + scope.inactive_candidate_count(),
        2
    );
}

#[test]
fn highest_pressure_rule_controls_generation_scope() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(first.clone(), 9, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 10, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 11, RecursiveWorldEvidenceKind::Violating),
    ]);

    let first_candidate = candidate(first, rule(&[1], &[3]), &[12]);

    let second_candidate = candidate(second.clone(), rule(&[5], &[7]), &[13]);

    let scope = RecursiveWorldRevisionGenerationEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![
            first_candidate.clone(),
            second_candidate.clone(),
        ]),
    );

    assert_eq!(scope.pressured_rule(), Some(&second,));

    assert_eq!(scope.active_candidates(), &[second_candidate,]);

    assert_eq!(scope.inactive_candidates(), &[first_candidate,]);
}

#[test]
fn evidence_scoper_facade_matches_direct_construction() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let candidates = RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
        target,
        rule(&[1], &[3]),
        &[10],
    )]);

    assert_eq!(
        RecursiveWorldRevisionGenerationEvidenceScoper::scope(&model, &state, candidates.clone(),),
        RecursiveWorldRevisionGenerationEvidenceScope::new(&model, &state, candidates,)
    );
}

#[test]
fn generation_evidence_scope_does_not_mutate_inputs() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let candidates = RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
        target,
        rule(&[1], &[3]),
        &[10],
    )]);

    let model_before = model.clone();

    let state_before = state.clone();

    let candidates_before = candidates.clone();

    let _ = RecursiveWorldRevisionGenerationEvidenceScope::new(&model, &state, candidates.clone());

    assert_eq!(model, model_before);

    assert_eq!(state, state_before);

    assert_eq!(candidates, candidates_before);
}
