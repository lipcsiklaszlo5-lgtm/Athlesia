use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceSet,
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
    rule: RecursiveWorldRule,
    observation: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(rule, unit(observation), kind)
}

#[test]
fn confirming_evidence_kind_identity() {
    let evidence = record(rule(&[1], &[2]), 2, RecursiveWorldEvidenceKind::Confirming);

    assert_eq!(evidence.kind(), RecursiveWorldEvidenceKind::Confirming);

    assert!(evidence.is_confirming());

    assert!(!evidence.is_violating());
}

#[test]
fn violating_evidence_kind_identity() {
    let evidence = record(rule(&[1], &[2]), 3, RecursiveWorldEvidenceKind::Violating);

    assert_eq!(evidence.kind(), RecursiveWorldEvidenceKind::Violating);

    assert!(evidence.is_violating());

    assert!(!evidence.is_confirming());
}

#[test]
fn evidence_preserves_rule_identity() {
    let source = rule(&[1], &[2]);

    let evidence = record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming);

    assert_eq!(evidence.rule(), &source);
}

#[test]
fn evidence_preserves_observation_identity() {
    let observation = unit(7);

    let evidence = RecursiveWorldEvidenceRecord::new(
        rule(&[1], &[2]),
        observation.clone(),
        RecursiveWorldEvidenceKind::Confirming,
    );

    assert_eq!(evidence.observation(), &observation);
}

#[test]
fn empty_evidence_set_is_empty() {
    let set = RecursiveWorldEvidenceSet::new(Vec::new());

    assert!(set.is_empty());

    assert_eq!(set.len(), 0);
}

#[test]
fn evidence_set_preserves_distinct_records() {
    let source = rule(&[1], &[2]);

    let confirming = record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming);

    let violating = record(source, 3, RecursiveWorldEvidenceKind::Violating);

    let set = RecursiveWorldEvidenceSet::new(vec![confirming.clone(), violating.clone()]);

    assert_eq!(set.len(), 2);

    assert!(set.contains(&confirming,));

    assert!(set.contains(&violating,));
}

#[test]
fn exact_duplicate_evidence_is_deduplicated() {
    let evidence = record(rule(&[1], &[2]), 2, RecursiveWorldEvidenceKind::Confirming);

    let set = RecursiveWorldEvidenceSet::new(vec![evidence.clone(), evidence]);

    assert_eq!(set.len(), 1);
}

#[test]
fn same_observation_with_different_kind_remains_distinct() {
    let source = rule(&[1], &[2]);

    let set = RecursiveWorldEvidenceSet::new(vec![
        record(source.clone(), 4, RecursiveWorldEvidenceKind::Confirming),
        record(source, 4, RecursiveWorldEvidenceKind::Violating),
    ]);

    assert_eq!(set.len(), 2);

    assert_eq!(set.confirming_count(), 1);

    assert_eq!(set.violating_count(), 1);
}

#[test]
fn evidence_kind_counts_are_exact() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let set = RecursiveWorldEvidenceSet::new(vec![
        record(first.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(first, 3, RecursiveWorldEvidenceKind::Violating),
        record(second, 6, RecursiveWorldEvidenceKind::Confirming),
    ]);

    assert_eq!(set.confirming_count(), 2);

    assert_eq!(set.violating_count(), 1);
}

#[test]
fn rule_scoped_evidence_query_is_exact() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let first_record = record(first.clone(), 2, RecursiveWorldEvidenceKind::Confirming);

    let second_record = record(second.clone(), 6, RecursiveWorldEvidenceKind::Confirming);

    let set = RecursiveWorldEvidenceSet::new(vec![second_record, first_record.clone()]);

    assert_eq!(set.records_for_rule(&first,), vec![first_record,]);

    assert!(set.records_for_rule(&rule(&[8], &[9],),).is_empty());
}

#[test]
fn evidence_set_is_deterministic_under_input_order() {
    let source = rule(&[1], &[2]);

    let first = record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming);

    let second = record(source, 3, RecursiveWorldEvidenceKind::Violating);

    let left = RecursiveWorldEvidenceSet::new(vec![first.clone(), second.clone()]);

    let right = RecursiveWorldEvidenceSet::new(vec![second, first]);

    assert_eq!(left, right);
}

#[test]
fn evidence_construction_does_not_mutate_source_vector() {
    let source_rule = rule(&[1], &[2]);

    let source = vec![
        record(
            source_rule.clone(),
            2,
            RecursiveWorldEvidenceKind::Confirming,
        ),
        record(source_rule, 3, RecursiveWorldEvidenceKind::Violating),
    ];

    let before = source.clone();

    let _ = RecursiveWorldEvidenceSet::new(source.clone());

    assert_eq!(source, before);
}
