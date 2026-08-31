use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceAssessor, RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRanking,
    RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_proposal::{
    RecursiveWorldRevisionProposal, RecursiveWorldRevisionProposalEvidenceScope,
    RecursiveWorldRevisionProposalEvidenceScoper, RecursiveWorldRevisionProposalSet,
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

fn evidence(
    source: RecursiveWorldRule,
    observation: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(source, unit(observation), kind)
}

fn ranking(
    state: &RecursiveWorldEvidenceState,
    rules: Vec<RecursiveWorldRule>,
) -> RecursiveWorldEvidenceRanking {
    RecursiveWorldEvidenceRanking::new(RecursiveWorldEvidenceAssessor::assess_many(state, rules))
}

#[test]
fn empty_ranking_and_validations_produce_empty_scope() {
    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &RecursiveWorldEvidenceRanking::new(Vec::new()),
        &RecursiveWorldRevisionProposalValidator::validate_set(
            &RecursiveWorldModel::new(Vec::new()),
            &RecursiveWorldRevisionProposalSet::new(Vec::new()),
        ),
    );

    assert!(scope.pressure().is_none());

    assert_eq!(scope.active_count(), 0);

    assert_eq!(scope.inactive_count(), 0);

    assert_eq!(scope.rejected_count(), 0);
}

#[test]
fn no_evidence_keeps_valid_proposal_inactive() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target.clone(), rule(&[1], &[3]))]),
    );

    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &ranking(&RecursiveWorldEvidenceState::empty(), vec![target]),
        &validations,
    );

    assert!(!scope.has_negative_pressure());

    assert_eq!(scope.active_count(), 0);

    assert_eq!(scope.inactive_count(), 1);
}

#[test]
fn confirming_evidence_keeps_valid_proposal_inactive() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target.clone(), rule(&[1], &[3]))]),
    );

    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &ranking(&state, vec![target]),
        &validations,
    );

    assert_eq!(scope.active_count(), 0);

    assert_eq!(scope.inactive_count(), 1);
}

#[test]
fn balanced_evidence_keeps_valid_proposal_inactive() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 3, RecursiveWorldEvidenceKind::Violating),
    ]);

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target.clone(), rule(&[1], &[4]))]),
    );

    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &ranking(&state, vec![target]),
        &validations,
    );

    assert!(!scope.has_negative_pressure());

    assert_eq!(scope.active_count(), 0);
}

#[test]
fn negative_pressure_activates_matching_valid_proposal() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target.clone(), rule(&[1], &[3]))]),
    );

    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &ranking(&state, vec![target.clone()]),
        &validations,
    );

    assert!(scope.has_negative_pressure());

    assert_eq!(scope.pressured_rule(), Some(&target,));

    assert_eq!(scope.active_count(), 1);

    assert_eq!(scope.inactive_count(), 0);
}

#[test]
fn nonmatching_valid_proposal_remains_inactive() {
    let pressured = rule(&[1], &[2]);

    let other = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![pressured.clone(), other.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(other.clone(), rule(&[5], &[7]))]),
    );

    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &ranking(&state, vec![pressured, other]),
        &validations,
    );

    assert_eq!(scope.active_count(), 0);

    assert_eq!(scope.inactive_count(), 1);
}

#[test]
fn multiple_matching_valid_proposals_are_active() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![
            proposal(target.clone(), rule(&[1], &[3])),
            proposal(target.clone(), rule(&[1], &[4])),
        ]),
    );

    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &ranking(&state, vec![target]),
        &validations,
    );

    assert_eq!(scope.active_count(), 2);

    assert_eq!(scope.inactive_count(), 0);
}

#[test]
fn rejected_proposals_are_preserved_independently_of_pressure() {
    let target = rule(&[1], &[2]);

    let existing = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), existing.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target.clone(), existing)]),
    );

    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &ranking(&state, vec![target]),
        &validations,
    );

    assert_eq!(scope.rejected_count(), 1);

    assert_eq!(scope.active_count(), 0);
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

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![
            proposal(pressured.clone(), rule(&[1], &[3])),
            proposal(other.clone(), rule(&[5], &[7])),
        ]),
    );

    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &ranking(&state, vec![pressured, other]),
        &validations,
    );

    assert_eq!(scope.accepted_count(), validations.accepted_count());

    assert_eq!(scope.active_count() + scope.inactive_count(), 2);
}

#[test]
fn active_revisions_preserve_m33_revision_identity() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(
            target.clone(),
            replacement.clone(),
        )]),
    );

    let scope = RecursiveWorldRevisionProposalEvidenceScope::new(
        &ranking(&state, vec![target.clone()]),
        &validations,
    );

    let revisions = scope.active_revisions();

    assert_eq!(revisions.len(), 1);

    assert_eq!(revisions[0].target(), &target);

    assert_eq!(revisions[0].replacement(), &replacement);
}

#[test]
fn evidence_scoper_matches_direct_construction() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let evidence_ranking = ranking(&state, vec![target.clone()]);

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target, rule(&[1], &[3]))]),
    );

    assert_eq!(
        RecursiveWorldRevisionProposalEvidenceScoper::scope(&evidence_ranking, &validations,),
        RecursiveWorldRevisionProposalEvidenceScope::new(&evidence_ranking, &validations,)
    );
}

#[test]
fn evidence_scope_does_not_mutate_ranking_or_validation_set() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let evidence_ranking = ranking(&state, vec![target.clone()]);

    let validations = RecursiveWorldRevisionProposalValidator::validate_set(
        &model,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target, rule(&[1], &[3]))]),
    );

    let ranking_before = evidence_ranking.clone();

    let validations_before = validations.clone();

    let _ = RecursiveWorldRevisionProposalEvidenceScope::new(&evidence_ranking, &validations);

    assert_eq!(evidence_ranking, ranking_before);

    assert_eq!(validations, validations_before);
}
