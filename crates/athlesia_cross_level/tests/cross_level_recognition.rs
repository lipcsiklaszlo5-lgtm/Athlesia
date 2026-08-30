use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{
    AbstractionUnit, CrossLevelConcept, CrossLevelMemory, CrossLevelObservation,
    CrossLevelRecognizer,
};

use athlesia_hierarchy::HierarchicalConcept;

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

#[test]
fn exact_mixed_units_are_recognized() {
    let target = cross_level(1, &[2, 3]);

    let observation =
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]);

    assert!(CrossLevelRecognizer::new().recognizes(&target, &observation,));
}

#[test]
fn unit_order_does_not_affect_recognition() {
    let target = cross_level(1, &[2, 3]);

    let observation =
        CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3]), structural_unit(1)]);

    assert!(CrossLevelRecognizer::new().recognizes(&target, &observation,));
}

#[test]
fn extra_structural_context_is_allowed() {
    let target = cross_level(1, &[2, 3]);

    let observation = CrossLevelObservation::new(vec![
        structural_unit(1),
        structural_unit(4),
        hierarchical_unit(&[2, 3]),
    ]);

    let matched = CrossLevelRecognizer::new()
        .recognize(&target, &observation)
        .unwrap();

    assert_eq!(matched.matched_units(), 2);

    assert_eq!(matched.observation_size(), 3);

    assert!(!matched.is_exact_context());
}

#[test]
fn extra_hierarchical_context_is_allowed() {
    let target = cross_level(1, &[2, 3]);

    let observation = CrossLevelObservation::new(vec![
        structural_unit(1),
        hierarchical_unit(&[2, 3]),
        hierarchical_unit(&[4, 5]),
    ]);

    assert!(CrossLevelRecognizer::new()
        .recognize(&target, &observation,)
        .is_some());
}

#[test]
fn exact_context_is_reported() {
    let target = cross_level(1, &[2, 3]);

    let observation =
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]);

    let matched = CrossLevelRecognizer::new()
        .recognize(&target, &observation)
        .unwrap();

    assert!(matched.is_exact_context());
}

#[test]
fn missing_structural_unit_rejects_match() {
    let target = cross_level(1, &[2, 3]);

    let observation = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3])]);

    assert!(CrossLevelRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn missing_hierarchical_unit_rejects_match() {
    let target = cross_level(1, &[2, 3]);

    let observation = CrossLevelObservation::new(vec![structural_unit(1)]);

    assert!(CrossLevelRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn partial_overlap_is_not_recognition() {
    let target = CrossLevelConcept::new(vec![
        structural_unit(1),
        structural_unit(2),
        hierarchical_unit(&[3, 4]),
    ])
    .unwrap();

    let observation =
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[3, 5])]);

    assert!(CrossLevelRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn structural_extent_mismatch_rejects_recognition() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        6,
    );

    let hierarchical = hierarchical_unit(&[2, 3]);

    let target = CrossLevelConcept::new(vec![
        AbstractionUnit::Structural(short),
        hierarchical.clone(),
    ])
    .unwrap();

    let observation =
        CrossLevelObservation::new(vec![AbstractionUnit::Structural(long), hierarchical]);

    assert!(CrossLevelRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn hierarchy_identity_mismatch_rejects_recognition() {
    let target = cross_level(1, &[2, 3]);

    let observation =
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 4])]);

    assert!(CrossLevelRecognizer::new()
        .recognize(&target, &observation,)
        .is_none());
}

#[test]
fn memory_recognition_returns_matching_concepts() {
    let mut memory = CrossLevelMemory::new();

    let first = cross_level(1, &[2, 3]);

    let second = cross_level(4, &[5, 6]);

    memory.insert(first.clone());

    memory.insert(second);

    let observation =
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]);

    let matches = CrossLevelRecognizer::new().recognize_memory(&memory, &observation);

    assert_eq!(matches.len(), 1);

    assert_eq!(matches[0].concept(), &first);
}

#[test]
fn one_observation_can_match_multiple_cross_level_concepts() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[3, 4]));

    memory.insert(cross_level(2, &[3, 4]));

    let observation = CrossLevelObservation::new(vec![
        structural_unit(1),
        structural_unit(2),
        hierarchical_unit(&[3, 4]),
    ]);

    let matches = CrossLevelRecognizer::new().recognize_memory(&memory, &observation);

    assert_eq!(matches.len(), 2);
}

#[test]
fn empty_memory_returns_no_matches() {
    let memory = CrossLevelMemory::new();

    let observation =
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]);

    assert!(CrossLevelRecognizer::new()
        .recognize_memory(&memory, &observation,)
        .is_empty());
}

#[test]
fn empty_observation_returns_no_matches() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let observation = CrossLevelObservation::new(Vec::new());

    assert!(CrossLevelRecognizer::new()
        .recognize_memory(&memory, &observation,)
        .is_empty());
}

#[test]
fn recognition_is_deterministic() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[3, 4]));

    memory.insert(cross_level(2, &[3, 4]));

    let observation = CrossLevelObservation::new(vec![
        structural_unit(1),
        structural_unit(2),
        hierarchical_unit(&[3, 4]),
    ]);

    let recognizer = CrossLevelRecognizer::new();

    assert_eq!(
        recognizer.recognize_memory(&memory, &observation,),
        recognizer.recognize_memory(&memory, &observation,)
    );
}

#[test]
fn recognition_does_not_mutate_memory() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[3, 4]));

    memory.insert(cross_level(2, &[3, 4]));

    let before = memory.clone();

    let observation = CrossLevelObservation::new(vec![
        structural_unit(1),
        structural_unit(2),
        hierarchical_unit(&[3, 4]),
    ]);

    let _ = CrossLevelRecognizer::new().recognize_memory(&memory, &observation);

    assert_eq!(memory, before);
}
