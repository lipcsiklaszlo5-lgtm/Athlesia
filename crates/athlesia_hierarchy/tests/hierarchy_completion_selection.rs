use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_hierarchy::{
    HierarchicalConcept, HierarchicalMemory, HierarchyCompletionSelector, HierarchyObservation,
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
fn no_predictions_produce_no_completion_candidate() {
    let memory = HierarchicalMemory::new();

    let observation = HierarchyObservation::new(vec![concept(1)]);

    assert!(HierarchyCompletionSelector::new()
        .select(&memory, &observation,)
        .is_none());
}

#[test]
fn complete_hierarchy_produces_no_candidate() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    assert!(HierarchyCompletionSelector::new()
        .select(&memory, &observation,)
        .is_none());
}

#[test]
fn zero_overlap_produces_no_candidate() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    let observation = HierarchyObservation::new(vec![concept(3), concept(4)]);

    assert!(HierarchyCompletionSelector::new()
        .select(&memory, &observation,)
        .is_none());
}

#[test]
fn one_prediction_generates_missing_child_candidate() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let candidate = HierarchyCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(candidate.child(), &concept(2));

    assert_eq!(candidate.supporting_hierarchies(), 1);
}

#[test]
fn shared_missing_child_accumulates_support() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 3]));

    memory.insert(hierarchy(&[2, 3]));

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let candidate = HierarchyCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(candidate.child(), &concept(3));

    assert_eq!(candidate.supporting_hierarchies(), 2);

    assert_eq!(candidate.single_step_support(), 2);
}

#[test]
fn selector_prefers_more_hierarchy_support() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 3]));

    memory.insert(hierarchy(&[2, 3]));

    memory.insert(hierarchy(&[1, 4]));

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let selected = HierarchyCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.child(), &concept(3));

    assert_eq!(selected.supporting_hierarchies(), 2);
}

#[test]
fn single_step_support_breaks_equal_support_tie() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 3]));

    memory.insert(hierarchy(&[1, 2, 4]));

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let candidates = HierarchyCompletionSelector::new().generate(&memory, &observation);

    let three = candidates
        .iter()
        .find(|candidate| candidate.child() == &concept(3))
        .unwrap();

    let four = candidates
        .iter()
        .find(|candidate| candidate.child() == &concept(4))
        .unwrap();

    assert_eq!(three.supporting_hierarchies(), 1);

    assert_eq!(four.supporting_hierarchies(), 1);

    assert_eq!(three.single_step_support(), 1);

    assert_eq!(four.single_step_support(), 0);

    assert_eq!(candidates[0].child(), &concept(3));
}

#[test]
fn exact_tie_uses_structural_identity() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    memory.insert(hierarchy(&[1, 3]));

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let selected = HierarchyCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    let expected = if concept(2) < concept(3) {
        concept(2)
    } else {
        concept(3)
    };

    assert_eq!(selected.child(), &expected);
}

#[test]
fn multi_missing_prediction_supports_each_missing_child() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2, 3]));

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let candidates = HierarchyCompletionSelector::new().generate(&memory, &observation);

    assert_eq!(candidates.len(), 2);

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.child() == &concept(2) }));

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.child() == &concept(3) }));
}

#[test]
fn extra_context_does_not_block_selection() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    let observation = HierarchyObservation::new(vec![concept(1), concept(3), concept(4)]);

    let selected = HierarchyCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.child(), &concept(2));
}

#[test]
fn extent_identity_is_preserved_in_selection() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 2)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 2)],
        6,
    );

    let first = concept(1);

    let mut memory = HierarchicalMemory::new();

    memory.insert(HierarchicalConcept::new(vec![first.clone(), short.clone()]).unwrap());

    memory.insert(HierarchicalConcept::new(vec![first.clone(), long.clone()]).unwrap());

    let observation = HierarchyObservation::new(vec![first]);

    let candidates = HierarchyCompletionSelector::new().generate(&memory, &observation);

    assert_eq!(candidates.len(), 2);

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.child() == &short }));

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.child() == &long }));
}

#[test]
fn candidate_generation_is_deterministic() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    memory.insert(hierarchy(&[1, 3]));

    memory.insert(hierarchy(&[2, 3]));

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let selector = HierarchyCompletionSelector::new();

    assert_eq!(
        selector.generate(&memory, &observation,),
        selector.generate(&memory, &observation,)
    );
}

#[test]
fn insertion_order_does_not_change_selection() {
    let first_hierarchy = hierarchy(&[1, 2]);

    let second_hierarchy = hierarchy(&[1, 3]);

    let mut first = HierarchicalMemory::new();

    first.insert(first_hierarchy.clone());

    first.insert(second_hierarchy.clone());

    let mut second = HierarchicalMemory::new();

    second.insert(second_hierarchy);

    second.insert(first_hierarchy);

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let selector = HierarchyCompletionSelector::new();

    assert_eq!(
        selector.select(&first, &observation,),
        selector.select(&second, &observation,)
    );
}

#[test]
fn selection_does_not_mutate_memory() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    memory.insert(hierarchy(&[1, 3]));

    let before = memory.clone();

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let selector = HierarchyCompletionSelector::new();

    let _ = selector.generate(&memory, &observation);

    let _ = selector.select(&memory, &observation);

    assert_eq!(memory, before);
}
