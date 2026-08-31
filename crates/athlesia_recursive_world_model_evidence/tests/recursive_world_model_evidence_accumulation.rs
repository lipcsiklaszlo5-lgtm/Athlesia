use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceAccumulator, RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord,
    RecursiveWorldEvidenceSet, RecursiveWorldEvidenceState,
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
fn empty_state_is_empty() {
    let state = RecursiveWorldEvidenceState::empty();

    assert!(state.is_empty());

    assert_eq!(state.len(), 0);
}

#[test]
fn state_preserves_initial_evidence_identity() {
    let evidence = record(rule(&[1], &[2]), 2, RecursiveWorldEvidenceKind::Confirming);

    let set = RecursiveWorldEvidenceSet::new(vec![evidence.clone()]);

    let state = RecursiveWorldEvidenceState::new(set.clone());

    assert_eq!(state.evidence(), &set);

    assert!(state.contains(&evidence,));
}

#[test]
fn accumulating_new_record_increases_cardinality() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty();

    let next = state.accumulate(record(source, 2, RecursiveWorldEvidenceKind::Confirming));

    assert_eq!(state.len(), 0);

    assert_eq!(next.len(), 1);
}

#[test]
fn exact_duplicate_accumulation_is_idempotent() {
    let source = rule(&[1], &[2]);

    let evidence = record(source, 2, RecursiveWorldEvidenceKind::Confirming);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence.clone());

    let next = state.accumulate(evidence);

    assert_eq!(next.len(), 1);

    assert_eq!(next, state);
}

#[test]
fn opposite_kind_same_observation_is_preserved() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty()
        .accumulate(record(
            source.clone(),
            4,
            RecursiveWorldEvidenceKind::Confirming,
        ))
        .accumulate(record(source, 4, RecursiveWorldEvidenceKind::Violating));

    assert_eq!(state.len(), 2);

    assert_eq!(state.confirming_count(), 1);

    assert_eq!(state.violating_count(), 1);
}

#[test]
fn accumulation_preserves_prior_records() {
    let source = rule(&[1], &[2]);

    let first = record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming);

    let second = record(source, 3, RecursiveWorldEvidenceKind::Violating);

    let initial = RecursiveWorldEvidenceState::empty().accumulate(first.clone());

    let next = initial.accumulate(second.clone());

    assert!(next.contains(&first,));

    assert!(next.contains(&second,));

    assert_eq!(initial.len(), 1);

    assert_eq!(next.len(), 2);
}

#[test]
fn batch_accumulation_preserves_all_distinct_records() {
    let first_rule = rule(&[1], &[2]);

    let second_rule = rule(&[5], &[6]);

    let first = record(first_rule, 2, RecursiveWorldEvidenceKind::Confirming);

    let second = record(second_rule, 7, RecursiveWorldEvidenceKind::Violating);

    let state =
        RecursiveWorldEvidenceState::empty().accumulate_many(vec![first.clone(), second.clone()]);

    assert_eq!(state.len(), 2);

    assert!(state.contains(&first,));

    assert!(state.contains(&second,));
}

#[test]
fn batch_accumulation_deduplicates_internal_duplicates() {
    let source = rule(&[1], &[2]);

    let evidence = record(source, 2, RecursiveWorldEvidenceKind::Confirming);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence.clone(),
        evidence.clone(),
        evidence,
    ]);

    assert_eq!(state.len(), 1);
}

#[test]
fn accumulation_keeps_rule_scopes_separate() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(first.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(second.clone(), 6, RecursiveWorldEvidenceKind::Confirming),
    ]);

    assert_eq!(state.records_for_rule(&first,).len(), 1);

    assert_eq!(state.records_for_rule(&second,).len(), 1);
}

#[test]
fn accumulator_facade_matches_state_methods() {
    let source = rule(&[1], &[2]);

    let evidence = record(source, 2, RecursiveWorldEvidenceKind::Confirming);

    let state = RecursiveWorldEvidenceState::empty();

    assert_eq!(
        RecursiveWorldEvidenceAccumulator::accumulate(&state, evidence.clone(),),
        state.accumulate(evidence,)
    );
}

#[test]
fn accumulation_is_deterministic_under_batch_order() {
    let source = rule(&[1], &[2]);

    let first = record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming);

    let second = record(source, 3, RecursiveWorldEvidenceKind::Violating);

    let state = RecursiveWorldEvidenceState::empty();

    let left = RecursiveWorldEvidenceAccumulator::accumulate_many(
        &state,
        vec![first.clone(), second.clone()],
    );

    let right = RecursiveWorldEvidenceAccumulator::accumulate_many(&state, vec![second, first]);

    assert_eq!(left, right);
}

#[test]
fn accumulation_does_not_mutate_source_state() {
    let source = rule(&[1], &[2]);

    let initial = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let before = initial.clone();

    let _ = RecursiveWorldEvidenceAccumulator::accumulate(
        &initial,
        record(source, 3, RecursiveWorldEvidenceKind::Violating),
    );

    assert_eq!(initial, before);
}
