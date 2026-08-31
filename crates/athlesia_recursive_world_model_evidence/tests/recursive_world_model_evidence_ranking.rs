use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceAssessment, RecursiveWorldEvidenceAssessor, RecursiveWorldEvidenceKind,
    RecursiveWorldEvidenceRanking, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
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

fn assess(
    state: &RecursiveWorldEvidenceState,
    source: RecursiveWorldRule,
) -> RecursiveWorldEvidenceAssessment {
    RecursiveWorldEvidenceAssessor::assess(state, source)
}

#[test]
fn empty_ranking_is_empty() {
    let ranking = RecursiveWorldEvidenceRanking::new(Vec::new());

    assert!(ranking.is_empty());

    assert_eq!(ranking.len(), 0);

    assert!(ranking.highest_revision_pressure().is_none());
}

#[test]
fn single_assessment_is_highest_revision_pressure() {
    let source = rule(&[1], &[2]);

    let assessment = assess(&RecursiveWorldEvidenceState::empty(), source);

    let ranking = RecursiveWorldEvidenceRanking::new(vec![assessment.clone()]);

    assert_eq!(ranking.highest_revision_pressure(), Some(&assessment,));
}

#[test]
fn more_violating_evidence_ranks_first() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(first.clone(), 3, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 7, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 8, RecursiveWorldEvidenceKind::Violating),
    ]);

    let first_assessment = assess(&state, first);

    let second_assessment = assess(&state, second);

    let ranking =
        RecursiveWorldEvidenceRanking::new(vec![first_assessment, second_assessment.clone()]);

    assert_eq!(
        ranking.highest_revision_pressure(),
        Some(&second_assessment,)
    );
}

#[test]
fn more_negative_balance_breaks_violation_tie() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(first.clone(), 3, RecursiveWorldEvidenceKind::Violating),
        record(first.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(second.clone(), 7, RecursiveWorldEvidenceKind::Violating),
    ]);

    let first_assessment = assess(&state, first);

    let second_assessment = assess(&state, second);

    assert_eq!(
        first_assessment.violating_count(),
        second_assessment.violating_count()
    );

    assert!(second_assessment.balance() < first_assessment.balance());

    let ranking =
        RecursiveWorldEvidenceRanking::new(vec![first_assessment, second_assessment.clone()]);

    assert_eq!(
        ranking.highest_revision_pressure(),
        Some(&second_assessment,)
    );
}

#[test]
fn greater_evidence_count_breaks_remaining_tie() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(first.clone(), 3, RecursiveWorldEvidenceKind::Violating),
        record(first.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(second.clone(), 7, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 6, RecursiveWorldEvidenceKind::Confirming),
    ]);

    let first_assessment = assess(&state, first);

    let second_assessment = assess(&state, second);

    assert_eq!(
        first_assessment.violating_count(),
        second_assessment.violating_count()
    );

    assert_eq!(first_assessment.balance(), second_assessment.balance());

    assert_eq!(
        first_assessment.evidence_count(),
        second_assessment.evidence_count()
    );

    let ranking = RecursiveWorldEvidenceRanking::new(vec![
        second_assessment.clone(),
        first_assessment.clone(),
    ]);

    let expected = if first_assessment.rule() < second_assessment.rule() {
        first_assessment
    } else {
        second_assessment
    };

    assert_eq!(ranking.highest_revision_pressure(), Some(&expected,));
}

#[test]
fn higher_evidence_count_is_respected_after_equal_violation_and_balance() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(first.clone(), 3, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 7, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 6, RecursiveWorldEvidenceKind::Confirming),
        record(second.clone(), 8, RecursiveWorldEvidenceKind::Confirming),
    ]);

    let first_assessment = assess(&state, first);

    let second_assessment = assess(&state, second);

    assert_eq!(
        first_assessment.violating_count(),
        second_assessment.violating_count()
    );

    assert!(first_assessment.balance() < second_assessment.balance());

    let ranking =
        RecursiveWorldEvidenceRanking::new(vec![second_assessment, first_assessment.clone()]);

    assert_eq!(
        ranking.highest_revision_pressure(),
        Some(&first_assessment,)
    );
}

#[test]
fn exact_duplicate_assessments_are_deduplicated() {
    let source = rule(&[1], &[2]);

    let assessment = assess(&RecursiveWorldEvidenceState::empty(), source);

    let ranking = RecursiveWorldEvidenceRanking::new(vec![assessment.clone(), assessment]);

    assert_eq!(ranking.len(), 1);
}

#[test]
fn distinct_assessments_are_preserved() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty();

    let first_assessment = assess(&state, first);

    let second_assessment = assess(&state, second);

    let ranking = RecursiveWorldEvidenceRanking::new(vec![
        first_assessment.clone(),
        second_assessment.clone(),
    ]);

    assert_eq!(ranking.len(), 2);

    assert!(ranking.assessments().contains(&first_assessment,));

    assert!(ranking.assessments().contains(&second_assessment,));
}

#[test]
fn ranking_is_monotonic_by_violation_pressure() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let third = rule(&[8], &[9]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(first.clone(), 3, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 7, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 8, RecursiveWorldEvidenceKind::Violating),
        record(third.clone(), 10, RecursiveWorldEvidenceKind::Violating),
        record(third.clone(), 11, RecursiveWorldEvidenceKind::Violating),
        record(third.clone(), 12, RecursiveWorldEvidenceKind::Violating),
    ]);

    let ranking = RecursiveWorldEvidenceRanking::new(RecursiveWorldEvidenceAssessor::assess_many(
        &state,
        vec![first, second, third],
    ));

    for pair in ranking.assessments().windows(2) {
        assert!(pair[0].violating_count() >= pair[1].violating_count());
    }
}

#[test]
fn ranking_is_deterministic_under_input_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty();

    let left = assess(&state, first);

    let right = assess(&state, second);

    assert_eq!(
        RecursiveWorldEvidenceRanking::new(vec![left.clone(), right.clone(),],),
        RecursiveWorldEvidenceRanking::new(vec![right, left,],)
    );
}

#[test]
fn ranking_preserves_assessment_identity() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        3,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let assessment = assess(&state, source);

    let ranking = RecursiveWorldEvidenceRanking::new(vec![assessment.clone()]);

    assert_eq!(ranking.highest_revision_pressure(), Some(&assessment,));
}

#[test]
fn ranking_does_not_mutate_source_assessments() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty();

    let source = RecursiveWorldEvidenceAssessor::assess_many(&state, vec![first, second]);

    let before = source.clone();

    let _ = RecursiveWorldEvidenceRanking::new(source.clone());

    assert_eq!(source, before);
}
