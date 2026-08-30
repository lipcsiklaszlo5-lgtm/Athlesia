use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_revision::{RevisionMemory, RevisionObservation, RevisionPolicy, RevisionStatus};

fn concept() -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 2)],
        5,
    )
}

#[test]
fn empty_evidence_is_unsupported() {
    let policy = RevisionPolicy::default();

    assert_eq!(
        policy.classify(Default::default()),
        RevisionStatus::Unsupported
    );
}

#[test]
fn single_confirmation_is_not_enough_by_default() {
    let mut memory = RevisionMemory::new();
    let target = concept();

    let evidence = memory.record(target, RevisionObservation::Confirmed);

    assert_eq!(
        RevisionPolicy::default().classify(evidence),
        RevisionStatus::Unsupported
    );
}

#[test]
fn minimum_support_promotes_concept() {
    let mut memory = RevisionMemory::new();
    let target = concept();

    memory.record(target.clone(), RevisionObservation::Confirmed);

    let evidence = memory.record(target, RevisionObservation::Confirmed);

    assert_eq!(
        RevisionPolicy::default().classify(evidence),
        RevisionStatus::Supported
    );
}

#[test]
fn mixed_evidence_is_contested() {
    let mut memory = RevisionMemory::new();
    let target = concept();

    memory.record(target.clone(), RevisionObservation::Confirmed);

    memory.record(target.clone(), RevisionObservation::Confirmed);

    let evidence = memory.record(target, RevisionObservation::Violated);

    assert_eq!(
        RevisionPolicy::default().classify(evidence),
        RevisionStatus::Contested
    );
}

#[test]
fn contradiction_margin_can_weaken_concept() {
    let mut memory = RevisionMemory::new();
    let target = concept();

    memory.record(target.clone(), RevisionObservation::Confirmed);

    memory.record(target.clone(), RevisionObservation::Violated);

    memory.record(target.clone(), RevisionObservation::Violated);

    let evidence = memory.record(target, RevisionObservation::Violated);

    assert_eq!(
        RevisionPolicy::default().classify(evidence),
        RevisionStatus::Weakened
    );
}

#[test]
fn one_violation_does_not_immediately_weaken() {
    let mut memory = RevisionMemory::new();
    let target = concept();

    memory.record(target.clone(), RevisionObservation::Confirmed);

    let evidence = memory.record(target, RevisionObservation::Violated);

    assert_eq!(
        RevisionPolicy::default().classify(evidence),
        RevisionStatus::Contested
    );
}

#[test]
fn pure_violations_can_weaken() {
    let mut memory = RevisionMemory::new();
    let target = concept();

    memory.record(target.clone(), RevisionObservation::Violated);

    let evidence = memory.record(target, RevisionObservation::Violated);

    assert_eq!(
        RevisionPolicy::default().classify(evidence),
        RevisionStatus::Weakened
    );
}

#[test]
fn policy_thresholds_are_explicit() {
    let policy = RevisionPolicy::new(3, 4);

    assert_eq!(policy.minimum_support(), 3);
    assert_eq!(policy.weakening_margin(), 4);
}

#[test]
fn stricter_support_threshold_delays_support() {
    let mut memory = RevisionMemory::new();
    let target = concept();

    memory.record(target.clone(), RevisionObservation::Confirmed);

    let evidence = memory.record(target, RevisionObservation::Confirmed);

    assert_eq!(
        RevisionPolicy::new(3, 2).classify(evidence),
        RevisionStatus::Unsupported
    );
}

#[test]
fn classification_is_deterministic() {
    let mut memory = RevisionMemory::new();
    let target = concept();

    memory.record(target.clone(), RevisionObservation::Confirmed);

    memory.record(target.clone(), RevisionObservation::Confirmed);

    let evidence = memory.record(target, RevisionObservation::Violated);

    let policy = RevisionPolicy::default();

    assert_eq!(policy.classify(evidence), policy.classify(evidence));
}

#[test]
fn evidence_order_does_not_change_status() {
    let target = concept();

    let mut first = RevisionMemory::new();

    first.record(target.clone(), RevisionObservation::Confirmed);

    first.record(target.clone(), RevisionObservation::Confirmed);

    let first_evidence = first.record(target.clone(), RevisionObservation::Violated);

    let mut second = RevisionMemory::new();

    second.record(target.clone(), RevisionObservation::Violated);

    second.record(target.clone(), RevisionObservation::Confirmed);

    let second_evidence = second.record(target, RevisionObservation::Confirmed);

    let policy = RevisionPolicy::default();

    assert_eq!(first_evidence, second_evidence);

    assert_eq!(
        policy.classify(first_evidence),
        policy.classify(second_evidence)
    );
}

#[test]
fn status_does_not_modify_concept_identity() {
    let mut memory = RevisionMemory::new();
    let target = concept();
    let original = target.clone();

    memory.record(target.clone(), RevisionObservation::Confirmed);

    let evidence = memory.record(target, RevisionObservation::Confirmed);

    assert_eq!(
        RevisionPolicy::default().classify(evidence),
        RevisionStatus::Supported
    );

    let stored = memory.concepts().next().unwrap().0;

    assert_eq!(stored, &original);
}
