use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_hierarchy::{
    HierarchicalConcept, HierarchicalMemory, HierarchyActiveCycle, HierarchyCompletionOutcome,
    HierarchyObservation,
};

fn concept(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        6,
    )
}

fn hierarchy(spans: &[usize]) -> HierarchicalConcept {
    HierarchicalConcept::new(spans.iter().copied().map(concept).collect()).unwrap()
}

fn memory() -> HierarchicalMemory {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 3]));

    memory.insert(hierarchy(&[2, 3]));

    memory
}

#[test]
fn cycle_selects_completion_candidate() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let transition = HierarchyActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.candidate().child(), &concept(3));

    assert_eq!(transition.candidate().supporting_hierarchies(), 2);
}

#[test]
fn cycle_confirms_appearing_target() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let transition = HierarchyActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.outcome(), HierarchyCompletionOutcome::Confirmed);

    assert!(transition.is_confirmed());

    assert!(!transition.is_violated());
}

#[test]
fn cycle_violates_missing_target() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(4)]);

    let transition = HierarchyActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.outcome(), HierarchyCompletionOutcome::Violated);

    assert!(transition.is_violated());
}

#[test]
fn cycle_records_prior_observation() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let transition = HierarchyActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.prior_observation(), &prior);
}

#[test]
fn cycle_records_next_observation() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3), concept(4)]);

    let transition = HierarchyActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.next_observation(), &next);
}

#[test]
fn transition_target_matches_evaluation_target() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let transition = HierarchyActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(
        transition.candidate().child(),
        transition.evaluation().target()
    );
}

#[test]
fn complete_prior_state_stops_cycle() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    assert!(HierarchyActiveCycle::new()
        .step(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn zero_overlap_stops_cycle() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    let prior = HierarchyObservation::new(vec![concept(3), concept(4)]);

    let next = HierarchyObservation::new(vec![concept(2)]);

    assert!(HierarchyActiveCycle::new()
        .step(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn empty_memory_stops_cycle() {
    let memory = HierarchicalMemory::new();

    let prior = HierarchyObservation::new(vec![concept(1)]);

    let next = HierarchyObservation::new(vec![concept(2)]);

    assert!(HierarchyActiveCycle::new()
        .step(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn extra_next_context_preserves_confirmation() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3), concept(4), concept(5)]);

    let transition = HierarchyActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_confirmed());
}

#[test]
fn extent_mismatch_produces_violation() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 3)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 3)],
        6,
    );

    let first = concept(1);

    let mut memory = HierarchicalMemory::new();

    memory.insert(HierarchicalConcept::new(vec![first.clone(), short]).unwrap());

    let prior = HierarchyObservation::new(vec![first]);

    let next = HierarchyObservation::new(vec![long]);

    let transition = HierarchyActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_violated());
}

#[test]
fn active_cycle_is_deterministic() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let cycle = HierarchyActiveCycle::new();

    assert_eq!(
        cycle.step(&memory, &prior, &next,),
        cycle.step(&memory, &prior, &next,)
    );
}

#[test]
fn active_cycle_does_not_mutate_memory() {
    let memory = memory();
    let before = memory.clone();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let _ = HierarchyActiveCycle::new().step(&memory, &prior, &next);

    assert_eq!(memory, before);
}
