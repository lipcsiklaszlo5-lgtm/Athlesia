use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceAssessment, RecursiveWorldEvidenceAssessor, RecursiveWorldEvidenceKind,
    RecursiveWorldEvidenceProfile, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
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

fn record(
    source: RecursiveWorldRule,
    observation: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(source, unit(observation), kind)
}

#[test]
fn no_evidence_produces_none_profile() {
    let source = rule(&[1], &[2]);

    let assessment =
        RecursiveWorldEvidenceAssessment::evaluate(&RecursiveWorldEvidenceState::empty(), source);

    assert_eq!(assessment.profile(), RecursiveWorldEvidenceProfile::None);

    assert_eq!(assessment.evidence_count(), 0);

    assert!(!assessment.has_evidence());
}

#[test]
fn confirming_only_profile_is_classified() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let assessment = RecursiveWorldEvidenceAssessment::evaluate(&state, source);

    assert_eq!(
        assessment.profile(),
        RecursiveWorldEvidenceProfile::ConfirmingOnly
    );

    assert_eq!(assessment.confirming_count(), 1);

    assert_eq!(assessment.violating_count(), 0);
}

#[test]
fn violating_only_profile_is_classified() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        3,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let assessment = RecursiveWorldEvidenceAssessment::evaluate(&state, source);

    assert_eq!(
        assessment.profile(),
        RecursiveWorldEvidenceProfile::ViolatingOnly
    );

    assert_eq!(assessment.confirming_count(), 0);

    assert_eq!(assessment.violating_count(), 1);
}

#[test]
fn mixed_profile_preserves_both_kinds() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(source.clone(), 3, RecursiveWorldEvidenceKind::Violating),
    ]);

    let assessment = RecursiveWorldEvidenceAssessment::evaluate(&state, source);

    assert_eq!(assessment.profile(), RecursiveWorldEvidenceProfile::Mixed);

    assert!(assessment.is_mixed());

    assert_eq!(assessment.confirming_count(), 1);

    assert_eq!(assessment.violating_count(), 1);
}

#[test]
fn positive_balance_is_exact() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(source.clone(), 4, RecursiveWorldEvidenceKind::Confirming),
        record(source.clone(), 3, RecursiveWorldEvidenceKind::Violating),
    ]);

    let assessment = RecursiveWorldEvidenceAssessment::evaluate(&state, source);

    assert_eq!(assessment.balance(), 1);
}

#[test]
fn negative_balance_is_exact() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(source.clone(), 3, RecursiveWorldEvidenceKind::Violating),
        record(source.clone(), 4, RecursiveWorldEvidenceKind::Violating),
    ]);

    let assessment = RecursiveWorldEvidenceAssessment::evaluate(&state, source);

    assert_eq!(assessment.balance(), -1);
}

#[test]
fn balanced_mixed_evidence_has_zero_balance() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(source.clone(), 3, RecursiveWorldEvidenceKind::Violating),
    ]);

    let assessment = RecursiveWorldEvidenceAssessment::evaluate(&state, source);

    assert_eq!(assessment.balance(), 0);

    assert_eq!(assessment.evidence_count(), 2);
}

#[test]
fn assessment_is_rule_scoped() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(first.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(second.clone(), 7, RecursiveWorldEvidenceKind::Violating),
    ]);

    let first_assessment = RecursiveWorldEvidenceAssessment::evaluate(&state, first);

    assert_eq!(first_assessment.confirming_count(), 1);

    assert_eq!(first_assessment.violating_count(), 0);
}

#[test]
fn assessment_preserves_rule_identity() {
    let source = rule(&[1], &[2]);

    let assessment = RecursiveWorldEvidenceAssessment::evaluate(
        &RecursiveWorldEvidenceState::empty(),
        source.clone(),
    );

    assert_eq!(assessment.rule(), &source);
}

#[test]
fn assessor_facade_matches_direct_evaluation() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    assert_eq!(
        RecursiveWorldEvidenceAssessor::assess(&state, source.clone(),),
        RecursiveWorldEvidenceAssessment::evaluate(&state, source,)
    );
}

#[test]
fn assess_many_is_canonical_and_deduplicated() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty();

    let left = RecursiveWorldEvidenceAssessor::assess_many(
        &state,
        vec![second.clone(), first.clone(), first.clone()],
    );

    let right = RecursiveWorldEvidenceAssessor::assess_many(&state, vec![first, second]);

    assert_eq!(left, right);

    assert_eq!(left.len(), 2);
}

#[test]
fn assessment_does_not_mutate_evidence_state() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let before = state.clone();

    let _ = RecursiveWorldEvidenceAssessor::assess(&state, source);

    assert_eq!(state, before);
}
