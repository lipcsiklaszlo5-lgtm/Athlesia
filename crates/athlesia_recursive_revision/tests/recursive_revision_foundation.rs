use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_revision::{
    RecursiveEvidenceState, RecursiveRevisionMemory, RecursiveRevisionObservation,
};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        6,
    )
}

fn hierarchy(spans: &[usize]) -> HierarchicalConcept {
    HierarchicalConcept::new(spans.iter().copied().map(structural).collect()).unwrap()
}

fn structural_unit(span: usize) -> AbstractionUnit {
    AbstractionUnit::Structural(structural(span))
}

fn hierarchical_unit(spans: &[usize]) -> AbstractionUnit {
    AbstractionUnit::Hierarchical(hierarchy(spans))
}

fn cross_level(structural_span: usize, hierarchy_spans: &[usize]) -> CrossLevelConcept {
    CrossLevelConcept::new(vec![
        structural_unit(structural_span),
        hierarchical_unit(hierarchy_spans),
    ])
    .unwrap()
}

fn base(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(structural_unit(span))
}

fn cross(structural_span: usize, hierarchy_spans: &[usize]) -> RecursiveUnit {
    RecursiveUnit::CrossLevel(cross_level(structural_span, hierarchy_spans))
}

fn concept(base_span: usize, cross_span: usize) -> RecursiveConcept {
    RecursiveConcept::new(vec![
        base(base_span),
        cross(cross_span, &[cross_span + 1, cross_span + 2]),
    ])
    .unwrap()
}

#[test]
fn evidence_state_starts_empty() {
    let state = RecursiveEvidenceState::new();

    assert_eq!(state.confirmations(), 0);

    assert_eq!(state.violations(), 0);

    assert_eq!(state.observations(), 0);

    assert_eq!(state.balance(), 0);

    assert!(state.is_unobserved());
}

#[test]
fn confirmation_updates_only_confirmation_count() {
    let mut state = RecursiveEvidenceState::new();

    state.confirm();

    assert_eq!(state.confirmations(), 1);

    assert_eq!(state.violations(), 0);

    assert_eq!(state.balance(), 1);
}

#[test]
fn violation_updates_only_violation_count() {
    let mut state = RecursiveEvidenceState::new();

    state.violate();

    assert_eq!(state.confirmations(), 0);

    assert_eq!(state.violations(), 1);

    assert_eq!(state.balance(), -1);
}

#[test]
fn repeated_evidence_accumulates() {
    let mut state = RecursiveEvidenceState::new();

    state.confirm();
    state.confirm();
    state.violate();

    assert_eq!(state.confirmations(), 2);

    assert_eq!(state.violations(), 1);

    assert_eq!(state.observations(), 3);

    assert_eq!(state.balance(), 1);
}

#[test]
fn boolean_confirmation_maps_to_observation() {
    assert_eq!(
        RecursiveRevisionObservation::from_confirmation(true,),
        RecursiveRevisionObservation::Confirmed
    );

    assert_eq!(
        RecursiveRevisionObservation::from_confirmation(false,),
        RecursiveRevisionObservation::Violated
    );
}

#[test]
fn memory_starts_empty() {
    let memory = RecursiveRevisionMemory::new();

    assert!(memory.is_empty());

    assert_eq!(memory.len(), 0);
}

#[test]
fn memory_confirmation_creates_evidence_entry() {
    let target = concept(1, 2);

    let mut memory = RecursiveRevisionMemory::new();

    let state = memory.confirm(target.clone());

    assert_eq!(state.confirmations(), 1);

    assert!(memory.contains(&target));

    assert_eq!(memory.evidence(&target), Some(&state));
}

#[test]
fn memory_violation_creates_evidence_entry() {
    let target = concept(1, 2);

    let mut memory = RecursiveRevisionMemory::new();

    let state = memory.violate(target.clone());

    assert_eq!(state.violations(), 1);

    assert_eq!(memory.evidence(&target), Some(&state));
}

#[test]
fn evidence_is_keyed_by_exact_recursive_identity() {
    let first = concept(1, 2);

    let second = concept(1, 3);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first.clone());

    memory.violate(second.clone());

    assert_eq!(memory.evidence(&first).unwrap().confirmations(), 1);

    assert_eq!(memory.evidence(&first).unwrap().violations(), 0);

    assert_eq!(memory.evidence(&second).unwrap().confirmations(), 0);

    assert_eq!(memory.evidence(&second).unwrap().violations(), 1);
}

#[test]
fn recursive_depth_remains_part_of_revision_identity() {
    let child = concept(1, 2);

    let shallow = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(child.clone())),
    ])
    .unwrap();

    let deeper_child =
        RecursiveConcept::new(vec![base(6), RecursiveUnit::Recursive(Box::new(child))]).unwrap();

    let deep = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(deeper_child)),
    ])
    .unwrap();

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(shallow.clone());

    memory.violate(deep.clone());

    assert_eq!(memory.evidence(&shallow).unwrap().confirmations(), 1);

    assert_eq!(memory.evidence(&deep).unwrap().violations(), 1);

    assert_eq!(memory.len(), 2);
}

#[test]
fn repeated_observation_updates_existing_entry() {
    let target = concept(1, 2);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(target.clone());

    memory.confirm(target.clone());

    memory.violate(target.clone());

    let state = memory.evidence(&target).unwrap();

    assert_eq!(state.confirmations(), 2);

    assert_eq!(state.violations(), 1);

    assert_eq!(memory.len(), 1);
}

#[test]
fn observation_api_matches_convenience_methods() {
    let first = concept(1, 2);

    let second = concept(5, 6);

    let mut explicit = RecursiveRevisionMemory::new();

    explicit.observe(first.clone(), RecursiveRevisionObservation::Confirmed);

    explicit.observe(second.clone(), RecursiveRevisionObservation::Violated);

    let mut convenience = RecursiveRevisionMemory::new();

    convenience.confirm(first);

    convenience.violate(second);

    assert_eq!(explicit, convenience);
}

#[test]
fn memory_iteration_is_deterministic() {
    let first = concept(1, 2);

    let second = concept(5, 6);

    let mut left = RecursiveRevisionMemory::new();

    left.confirm(first.clone());

    left.violate(second.clone());

    let mut right = RecursiveRevisionMemory::new();

    right.violate(second);

    right.confirm(first);

    let left_items: Vec<_> = left
        .iter()
        .map(|(concept, evidence)| (concept.clone(), *evidence))
        .collect();

    let right_items: Vec<_> = right
        .iter()
        .map(|(concept, evidence)| (concept.clone(), *evidence))
        .collect();

    assert_eq!(left_items, right_items);
}
