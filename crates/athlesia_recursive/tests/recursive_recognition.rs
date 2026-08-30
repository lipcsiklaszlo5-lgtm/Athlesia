use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{
    RecursiveConcept, RecursiveMemory, RecursiveObservation, RecursiveRecognizer, RecursiveUnit,
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

fn recursive_child() -> RecursiveConcept {
    RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap()
}

#[test]
fn exact_recursive_units_are_recognized() {
    let target = recursive_child();

    let observation = RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]);

    assert!(RecursiveRecognizer::new().recognizes(&target, &observation,));
}

#[test]
fn unit_order_does_not_affect_recognition() {
    let target = recursive_child();

    let observation = RecursiveObservation::new(vec![cross(2, &[3, 4]), base(1)]);

    assert!(RecursiveRecognizer::new().recognizes(&target, &observation,));
}

#[test]
fn extra_base_context_is_allowed() {
    let target = recursive_child();

    let observation = RecursiveObservation::new(vec![base(1), base(5), cross(2, &[3, 4])]);

    let matched = RecursiveRecognizer::new()
        .recognize(&target, &observation)
        .unwrap();

    assert_eq!(matched.matched_units(), 2);

    assert_eq!(matched.observation_size(), 3);

    assert!(!matched.is_exact_context());
}

#[test]
fn extra_higher_order_context_is_allowed() {
    let target = recursive_child();

    let observation =
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4]), cross(5, &[6, 7])]);

    assert!(RecursiveRecognizer::new()
        .recognize(&target, &observation,)
        .is_some());
}

#[test]
fn exact_context_is_reported() {
    let target = recursive_child();

    let observation = RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]);

    let matched = RecursiveRecognizer::new()
        .recognize(&target, &observation)
        .unwrap();

    assert!(matched.is_exact_context());
}

#[test]
fn missing_base_unit_rejects_recognition() {
    let target = recursive_child();

    let observation = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    assert!(RecursiveRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn missing_cross_level_unit_rejects_recognition() {
    let target = recursive_child();

    let observation = RecursiveObservation::new(vec![base(1)]);

    assert!(RecursiveRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn partial_overlap_is_not_recognition() {
    let target = RecursiveConcept::new(vec![base(1), base(2), cross(3, &[4, 5])]).unwrap();

    let observation = RecursiveObservation::new(vec![base(1), cross(3, &[4, 6])]);

    assert!(RecursiveRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn cross_level_identity_mismatch_rejects_recognition() {
    let target = recursive_child();

    let observation = RecursiveObservation::new(vec![base(1), cross(2, &[3, 5])]);

    assert!(RecursiveRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn recursive_child_is_recognized_at_exact_depth() {
    let child = recursive_child();

    let target = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(child.clone())),
    ])
    .unwrap();

    let observation =
        RecursiveObservation::new(vec![base(5), RecursiveUnit::Recursive(Box::new(child))]);

    assert!(RecursiveRecognizer::new()
        .recognize(&target, &observation,)
        .is_some());
}

#[test]
fn recursive_depth_mismatch_rejects_recognition() {
    let level_one = recursive_child();

    let level_two = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(level_one.clone())),
    ])
    .unwrap();

    let target =
        RecursiveConcept::new(vec![base(6), RecursiveUnit::Recursive(Box::new(level_two))])
            .unwrap();

    let observation =
        RecursiveObservation::new(vec![base(6), RecursiveUnit::Recursive(Box::new(level_one))]);

    assert!(RecursiveRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn memory_recognition_returns_matching_concepts() {
    let first = recursive_child();

    let second = RecursiveConcept::new(vec![base(5), cross(6, &[7, 8])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(first.clone());

    memory.insert(second);

    let observation = RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]);

    let matches = RecursiveRecognizer::new().recognize_memory(&memory, &observation);

    assert_eq!(matches.len(), 1);

    assert_eq!(matches[0].concept(), &first);
}

#[test]
fn one_observation_can_match_multiple_recursive_concepts() {
    let first = RecursiveConcept::new(vec![base(1), cross(3, &[4, 5])]).unwrap();

    let second = RecursiveConcept::new(vec![base(2), cross(3, &[4, 5])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(first);

    memory.insert(second);

    let observation = RecursiveObservation::new(vec![base(1), base(2), cross(3, &[4, 5])]);

    let matches = RecursiveRecognizer::new().recognize_memory(&memory, &observation);

    assert_eq!(matches.len(), 2);
}

#[test]
fn empty_memory_returns_no_matches() {
    let memory = RecursiveMemory::new();

    let observation = RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]);

    assert!(RecursiveRecognizer::new()
        .recognize_memory(&memory, &observation,)
        .is_empty());
}

#[test]
fn empty_observation_returns_no_matches() {
    let mut memory = RecursiveMemory::new();

    memory.insert(recursive_child());

    let observation = RecursiveObservation::new(Vec::new());

    assert!(RecursiveRecognizer::new()
        .recognize_memory(&memory, &observation,)
        .is_empty());
}

#[test]
fn recognition_is_deterministic_and_non_mutating() {
    let first = RecursiveConcept::new(vec![base(1), cross(3, &[4, 5])]).unwrap();

    let second = RecursiveConcept::new(vec![base(2), cross(3, &[4, 5])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(first);

    memory.insert(second);

    let before = memory.clone();

    let observation = RecursiveObservation::new(vec![base(1), base(2), cross(3, &[4, 5])]);

    let recognizer = RecursiveRecognizer::new();

    let first_run = recognizer.recognize_memory(&memory, &observation);

    let second_run = recognizer.recognize_memory(&memory, &observation);

    assert_eq!(first_run, second_run);

    assert_eq!(memory, before);
}
