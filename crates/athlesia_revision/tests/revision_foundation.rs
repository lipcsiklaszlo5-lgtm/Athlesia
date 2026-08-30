use athlesia::{
    ConceptConsolidator, ConceptMemory, Encoder, HypothesisInducer, PredictionOutcome,
    PrimitiveDiscovery, PrimitiveSignature, RelationKind, RelationalStructure, StructuralConcept,
};

use athlesia_revision::{RevisionMemory, RevisionObservation};

fn concept(length: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 2)],
        length,
    )
}

fn learned_concept(values: &[i32]) -> StructuralConcept {
    let encoder = Encoder::new();
    let sequence = encoder.encode(values);

    let relations = RelationalStructure::from_sequence(&sequence);

    let primitives = PrimitiveDiscovery::default().discover(&relations);

    let hypotheses = HypothesisInducer::default().induce(&primitives);

    let mut memory = ConceptMemory::new();

    ConceptConsolidator::new().consolidate_into(&hypotheses, &mut memory);

    let result = memory.concepts().next().unwrap().clone();

    result
}

#[test]
fn revision_memory_starts_empty() {
    let memory = RevisionMemory::new();

    assert!(memory.is_empty());
    assert_eq!(memory.len(), 0);
}

#[test]
fn confirmation_is_recorded() {
    let mut memory = RevisionMemory::new();
    let target = concept(5);

    let evidence = memory.record(target.clone(), RevisionObservation::Confirmed);

    assert_eq!(evidence.confirmations(), 1);
    assert_eq!(evidence.violations(), 0);
    assert_eq!(evidence.total(), 1);

    assert_eq!(memory.evidence_for(&target), Some(evidence));
}

#[test]
fn violation_is_recorded() {
    let mut memory = RevisionMemory::new();
    let target = concept(5);

    let evidence = memory.record(target.clone(), RevisionObservation::Violated);

    assert_eq!(evidence.confirmations(), 0);
    assert_eq!(evidence.violations(), 1);
    assert_eq!(evidence.total(), 1);

    assert_eq!(memory.evidence_for(&target), Some(evidence));
}

#[test]
fn support_and_contradiction_can_coexist() {
    let mut memory = RevisionMemory::new();
    let target = concept(5);

    memory.record(target.clone(), RevisionObservation::Confirmed);

    let evidence = memory.record(target, RevisionObservation::Violated);

    assert_eq!(evidence.confirmations(), 1);
    assert_eq!(evidence.violations(), 1);
    assert!(evidence.is_contested());
}

#[test]
fn repeated_evidence_accumulates() {
    let mut memory = RevisionMemory::new();
    let target = concept(5);

    memory.record(target.clone(), RevisionObservation::Confirmed);

    memory.record(target.clone(), RevisionObservation::Confirmed);

    let evidence = memory.record(target, RevisionObservation::Violated);

    assert_eq!(evidence.confirmations(), 2);
    assert_eq!(evidence.violations(), 1);
    assert_eq!(evidence.total(), 3);
}

#[test]
fn same_concept_uses_one_evidence_record() {
    let mut memory = RevisionMemory::new();
    let target = concept(5);

    memory.record(target.clone(), RevisionObservation::Confirmed);

    memory.record(target, RevisionObservation::Violated);

    assert_eq!(memory.len(), 1);
}

#[test]
fn structural_extent_separates_evidence_records() {
    let mut memory = RevisionMemory::new();

    memory.record(concept(4), RevisionObservation::Confirmed);

    memory.record(concept(5), RevisionObservation::Confirmed);

    assert_eq!(memory.len(), 2);
}

#[test]
fn evidence_does_not_change_concept_identity() {
    let mut memory = RevisionMemory::new();
    let target = concept(5);
    let original = target.clone();

    memory.record(target.clone(), RevisionObservation::Confirmed);

    memory.record(target, RevisionObservation::Violated);

    let stored = memory.concepts().next().unwrap().0;

    assert_eq!(stored, &original);
}

#[test]
fn prediction_outcome_maps_to_revision_evidence() {
    let mut memory = RevisionMemory::new();
    let target = concept(5);

    memory.record_prediction_outcome(target.clone(), PredictionOutcome::Confirmed);

    let evidence = memory.record_prediction_outcome(target, PredictionOutcome::Violated);

    assert_eq!(evidence.confirmations(), 1);
    assert_eq!(evidence.violations(), 1);
    assert_eq!(evidence.total(), 2);
    assert!(evidence.is_contested());
}

#[test]
fn violation_does_not_delete_concept() {
    let mut memory = RevisionMemory::new();
    let target = concept(5);

    memory.record(target.clone(), RevisionObservation::Confirmed);

    memory.record(target.clone(), RevisionObservation::Violated);

    memory.record(target.clone(), RevisionObservation::Violated);

    assert_eq!(memory.len(), 1);
    assert!(memory.evidence_for(&target).is_some());
}

#[test]
fn update_order_does_not_change_aggregate_evidence() {
    let target = concept(5);

    let mut first = RevisionMemory::new();

    first.record(target.clone(), RevisionObservation::Confirmed);

    first.record(target.clone(), RevisionObservation::Violated);

    let mut second = RevisionMemory::new();

    second.record(target.clone(), RevisionObservation::Violated);

    second.record(target, RevisionObservation::Confirmed);

    assert_eq!(first, second);
}

#[test]
fn concept_iteration_is_deterministic() {
    let mut first = RevisionMemory::new();
    let mut second = RevisionMemory::new();

    first.record(concept(6), RevisionObservation::Confirmed);

    first.record(concept(4), RevisionObservation::Confirmed);

    first.record(concept(5), RevisionObservation::Confirmed);

    second.record(concept(5), RevisionObservation::Confirmed);

    second.record(concept(6), RevisionObservation::Confirmed);

    second.record(concept(4), RevisionObservation::Confirmed);

    let first_items: Vec<_> = first.concepts().collect();

    let second_items: Vec<_> = second.concepts().collect();

    assert_eq!(first_items, second_items);
}

#[test]
fn evidence_transfers_across_concrete_values() {
    let first = learned_concept(&[1, 2, 1, 2, 3]);

    let second = learned_concept(&[847, 13, 847, 13, 999]);

    assert_eq!(first, second);

    let mut memory = RevisionMemory::new();

    memory.record(first, RevisionObservation::Confirmed);

    memory.record(second.clone(), RevisionObservation::Violated);

    let evidence = memory.evidence_for(&second).unwrap();

    assert_eq!(memory.len(), 1);
    assert_eq!(evidence.confirmations(), 1);
    assert_eq!(evidence.violations(), 1);
}
