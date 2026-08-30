use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{
    RecursiveActiveCycle, RecursiveCompletionOutcome, RecursiveConcept, RecursiveMemory,
    RecursiveObservation, RecursiveUnit,
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

fn child() -> RecursiveConcept {
    RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap()
}

#[test]
fn cycle_selects_recursive_completion_target() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.candidate().unit(), &cross(2, &[3, 4],));
}

#[test]
fn base_anchor_can_complete_cross_level_target() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_confirmed());
}

#[test]
fn cross_level_anchor_can_complete_base_target() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let next = RecursiveObservation::new(vec![base(1)]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_confirmed());

    assert_eq!(transition.candidate().unit(), &base(1));
}

#[test]
fn base_anchor_can_complete_recursive_target() {
    let nested = child();

    let recursive_target = RecursiveUnit::Recursive(Box::new(nested));

    let concept = RecursiveConcept::new(vec![base(5), recursive_target.clone()]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(concept);

    let prior = RecursiveObservation::new(vec![base(5)]);

    let next = RecursiveObservation::new(vec![recursive_target]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_confirmed());
}

#[test]
fn cycle_confirms_appearing_target() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.outcome(), RecursiveCompletionOutcome::Confirmed);

    assert!(transition.is_confirmed());

    assert!(!transition.is_violated());
}

#[test]
fn cycle_violates_missing_target() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(5, &[6, 7])]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.outcome(), RecursiveCompletionOutcome::Violated);

    assert!(transition.is_violated());
}

#[test]
fn transition_records_prior_observation() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.prior_observation(), &prior);
}

#[test]
fn transition_records_next_observation() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4]), base(5)]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.next_observation(), &next);
}

#[test]
fn transition_target_matches_evaluation_target() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(
        transition.candidate().unit(),
        transition.evaluation().target()
    );
}

#[test]
fn complete_prior_state_stops_cycle() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]);

    let next = RecursiveObservation::new(vec![base(5)]);

    assert!(RecursiveActiveCycle::new()
        .step(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn zero_overlap_stops_cycle() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(5), cross(6, &[7, 8])]);

    let next = RecursiveObservation::new(vec![base(1)]);

    assert!(RecursiveActiveCycle::new()
        .step(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn empty_memory_stops_cycle() {
    let memory = RecursiveMemory::new();

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    assert!(RecursiveActiveCycle::new()
        .step(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn extra_next_context_preserves_confirmation() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4]), base(5), cross(6, &[7, 8])]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_confirmed());
}

#[test]
fn cross_level_identity_mismatch_produces_violation() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 5])]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_violated());
}

#[test]
fn recursive_depth_mismatch_produces_violation() {
    let level_one = child();

    let level_two = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(level_one.clone())),
    ])
    .unwrap();

    let target = RecursiveUnit::Recursive(Box::new(level_two));

    let concept = RecursiveConcept::new(vec![base(6), target.clone()]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(concept);

    let prior = RecursiveObservation::new(vec![base(6)]);

    let next = RecursiveObservation::new(vec![RecursiveUnit::Recursive(Box::new(level_one))]);

    let transition = RecursiveActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.candidate().unit(), &target);

    assert!(transition.is_violated());
}

#[test]
fn active_cycle_is_deterministic_and_non_mutating() {
    let shared = cross(3, &[4, 5]);

    let mut memory = RecursiveMemory::new();

    memory.insert(RecursiveConcept::new(vec![base(1), shared.clone()]).unwrap());

    memory.insert(RecursiveConcept::new(vec![base(2), shared.clone()]).unwrap());

    let before = memory.clone();

    let prior = RecursiveObservation::new(vec![base(1), base(2)]);

    let next = RecursiveObservation::new(vec![shared]);

    let cycle = RecursiveActiveCycle::new();

    let first = cycle.step(&memory, &prior, &next);

    let second = cycle.step(&memory, &prior, &next);

    assert_eq!(first, second);

    assert_eq!(memory, before);
}
