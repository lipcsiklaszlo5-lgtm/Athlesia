use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_hierarchy::{
    HierarchicalConcept, HierarchicalMemory, HierarchyObservation, HierarchyRecognizer,
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

#[test]
fn exact_children_are_recognized() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    assert!(HierarchyRecognizer::new().recognizes(&target, &observation,));
}

#[test]
fn child_order_does_not_affect_recognition() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(2), concept(1)]);

    assert!(HierarchyRecognizer::new().recognizes(&target, &observation,));
}

#[test]
fn extra_context_is_allowed() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(1), concept(2), concept(3)]);

    let matched = HierarchyRecognizer::new()
        .recognize(&target, &observation)
        .unwrap();

    assert_eq!(matched.matched_children(), 2);

    assert_eq!(matched.observation_size(), 3);

    assert!(!matched.is_exact_context());
}

#[test]
fn exact_context_is_reported() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let matched = HierarchyRecognizer::new()
        .recognize(&target, &observation)
        .unwrap();

    assert!(matched.is_exact_context());
}

#[test]
fn missing_child_rejects_hierarchy() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(1)]);

    assert!(HierarchyRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn unrelated_children_do_not_match() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(3), concept(4)]);

    assert!(!HierarchyRecognizer::new().recognizes(&target, &observation,));
}

#[test]
fn partial_overlap_is_not_recognition() {
    let target = hierarchy(&[1, 2, 3]);

    let observation = HierarchyObservation::new(vec![concept(1), concept(2), concept(4)]);

    assert!(HierarchyRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn child_extent_mismatch_rejects_recognition() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        6,
    );

    let second = concept(2);

    let target = HierarchicalConcept::new(vec![short, second.clone()]).unwrap();

    let observation = HierarchyObservation::new(vec![long, second]);

    assert!(HierarchyRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn memory_recognition_returns_matching_hierarchies() {
    let mut memory = HierarchicalMemory::new();

    let first = hierarchy(&[1, 2]);

    let second = hierarchy(&[2, 3]);

    memory.insert(first.clone());
    memory.insert(second);

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let matches = HierarchyRecognizer::new().recognize_memory(&memory, &observation);

    assert_eq!(matches.len(), 1);

    assert_eq!(matches[0].concept(), &first);
}

#[test]
fn one_observation_can_recognize_multiple_hierarchies() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    memory.insert(hierarchy(&[2, 3]));

    let observation = HierarchyObservation::new(vec![concept(1), concept(2), concept(3)]);

    let matches = HierarchyRecognizer::new().recognize_memory(&memory, &observation);

    assert_eq!(matches.len(), 2);
}

#[test]
fn empty_memory_returns_no_matches() {
    let memory = HierarchicalMemory::new();

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    assert!(HierarchyRecognizer::new()
        .recognize_memory(&memory, &observation,)
        .is_empty());
}

#[test]
fn empty_observation_returns_no_matches() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    let observation = HierarchyObservation::new(Vec::new());

    assert!(HierarchyRecognizer::new()
        .recognize_memory(&memory, &observation,)
        .is_empty());
}

#[test]
fn memory_recognition_is_deterministic() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    memory.insert(hierarchy(&[2, 3]));

    let observation = HierarchyObservation::new(vec![concept(1), concept(2), concept(3)]);

    let recognizer = HierarchyRecognizer::new();

    assert_eq!(
        recognizer.recognize_memory(&memory, &observation,),
        recognizer.recognize_memory(&memory, &observation,)
    );
}

#[test]
fn recognition_does_not_mutate_memory() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    memory.insert(hierarchy(&[2, 3]));

    let before = memory.clone();

    let observation = HierarchyObservation::new(vec![concept(1), concept(2), concept(3)]);

    let _ = HierarchyRecognizer::new().recognize_memory(&memory, &observation);

    assert_eq!(memory, before);
}
