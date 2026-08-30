use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{
    RecursiveCompletionSelector, RecursiveConcept, RecursiveMemory, RecursiveObservation,
    RecursiveUnit,
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
fn no_predictions_produce_no_candidate() {
    let memory = RecursiveMemory::new();

    let observation = RecursiveObservation::new(vec![base(1)]);

    assert!(RecursiveCompletionSelector::new()
        .select(&memory, &observation,)
        .is_none());
}

#[test]
fn complete_concept_produces_no_candidate() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let observation = RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]);

    assert!(RecursiveCompletionSelector::new()
        .select(&memory, &observation,)
        .is_none());
}

#[test]
fn zero_overlap_produces_no_candidate() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let observation = RecursiveObservation::new(vec![base(5), cross(6, &[7, 8])]);

    assert!(RecursiveCompletionSelector::new()
        .select(&memory, &observation,)
        .is_none());
}

#[test]
fn base_anchor_selects_cross_level_target() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let observation = RecursiveObservation::new(vec![base(1)]);

    let selected = RecursiveCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &cross(2, &[3, 4],));
}

#[test]
fn cross_level_anchor_selects_base_target() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let observation = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let selected = RecursiveCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &base(1));
}

#[test]
fn base_anchor_can_select_recursive_target() {
    let nested = child();

    let recursive_target = RecursiveUnit::Recursive(Box::new(nested));

    let concept = RecursiveConcept::new(vec![base(5), recursive_target.clone()]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(concept);

    let observation = RecursiveObservation::new(vec![base(5)]);

    let selected = RecursiveCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &recursive_target);
}

#[test]
fn recursive_anchor_can_select_base_target() {
    let nested = child();

    let recursive_anchor = RecursiveUnit::Recursive(Box::new(nested));

    let concept = RecursiveConcept::new(vec![base(5), recursive_anchor.clone()]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(concept);

    let observation = RecursiveObservation::new(vec![recursive_anchor]);

    let selected = RecursiveCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &base(5));
}

#[test]
fn shared_missing_unit_accumulates_support() {
    let shared = cross(3, &[4, 5]);

    let first = RecursiveConcept::new(vec![base(1), shared.clone()]).unwrap();

    let second = RecursiveConcept::new(vec![base(2), shared.clone()]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(first);

    memory.insert(second);

    let observation = RecursiveObservation::new(vec![base(1), base(2)]);

    let selected = RecursiveCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &shared);

    assert_eq!(selected.supporting_concepts(), 2);

    assert_eq!(selected.single_step_support(), 2);
}

#[test]
fn selector_prefers_more_support() {
    let shared = cross(3, &[4, 5]);

    let competing = cross(6, &[7, 8]);

    let mut memory = RecursiveMemory::new();

    memory.insert(RecursiveConcept::new(vec![base(1), shared.clone()]).unwrap());

    memory.insert(RecursiveConcept::new(vec![base(2), shared.clone()]).unwrap());

    memory.insert(RecursiveConcept::new(vec![base(1), competing]).unwrap());

    let observation = RecursiveObservation::new(vec![base(1), base(2)]);

    let selected = RecursiveCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &shared);

    assert_eq!(selected.supporting_concepts(), 2);
}

#[test]
fn single_step_support_breaks_equal_support_tie() {
    let preferred = cross(3, &[4, 5]);

    let weaker = cross(6, &[7, 8]);

    let mut memory = RecursiveMemory::new();

    memory.insert(RecursiveConcept::new(vec![base(1), preferred.clone()]).unwrap());

    memory.insert(RecursiveConcept::new(vec![base(1), base(2), weaker.clone()]).unwrap());

    let observation = RecursiveObservation::new(vec![base(1)]);

    let candidates = RecursiveCompletionSelector::new().generate(&memory, &observation);

    let preferred_candidate = candidates
        .iter()
        .find(|candidate| candidate.unit() == &preferred)
        .unwrap();

    let weaker_candidate = candidates
        .iter()
        .find(|candidate| candidate.unit() == &weaker)
        .unwrap();

    assert_eq!(preferred_candidate.supporting_concepts(), 1);

    assert_eq!(weaker_candidate.supporting_concepts(), 1);

    assert_eq!(preferred_candidate.single_step_support(), 1);

    assert_eq!(weaker_candidate.single_step_support(), 0);

    assert_eq!(candidates[0].unit(), &preferred);
}

#[test]
fn exact_tie_uses_recursive_unit_identity() {
    let first = base(1);

    let second = base(2);

    let shared = cross(3, &[4, 5]);

    let mut memory = RecursiveMemory::new();

    memory.insert(RecursiveConcept::new(vec![first.clone(), shared.clone()]).unwrap());

    memory.insert(RecursiveConcept::new(vec![second.clone(), shared.clone()]).unwrap());

    let observation = RecursiveObservation::new(vec![shared]);

    let selected = RecursiveCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    let expected = if first < second { first } else { second };

    assert_eq!(selected.unit(), &expected);
}

#[test]
fn multi_missing_prediction_supports_each_unit() {
    let concept = RecursiveConcept::new(vec![base(1), base(2), cross(3, &[4, 5])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(concept);

    let observation = RecursiveObservation::new(vec![base(1)]);

    let candidates = RecursiveCompletionSelector::new().generate(&memory, &observation);

    assert_eq!(candidates.len(), 2);

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &base(2) }));

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &cross(3, &[4, 5],) }));
}

#[test]
fn extra_context_does_not_block_selection() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let observation = RecursiveObservation::new(vec![base(1), base(5), cross(6, &[7, 8])]);

    let selected = RecursiveCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &cross(2, &[3, 4],));
}

#[test]
fn recursive_depth_identity_is_preserved() {
    let level_one = child();

    let level_two = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(level_one.clone())),
    ])
    .unwrap();

    let shallow = RecursiveUnit::Recursive(Box::new(level_one));

    let deep = RecursiveUnit::Recursive(Box::new(level_two));

    let anchor = base(6);

    let mut memory = RecursiveMemory::new();

    memory.insert(RecursiveConcept::new(vec![anchor.clone(), shallow.clone()]).unwrap());

    memory.insert(RecursiveConcept::new(vec![anchor.clone(), deep.clone()]).unwrap());

    let observation = RecursiveObservation::new(vec![anchor]);

    let candidates = RecursiveCompletionSelector::new().generate(&memory, &observation);

    assert_eq!(candidates.len(), 2);

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &shallow }));

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &deep }));
}

#[test]
fn generation_is_deterministic_and_non_mutating() {
    let first = RecursiveConcept::new(vec![base(1), cross(3, &[4, 5])]).unwrap();

    let second = RecursiveConcept::new(vec![base(2), cross(3, &[4, 5])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(first);

    memory.insert(second);

    let before = memory.clone();

    let observation = RecursiveObservation::new(vec![cross(3, &[4, 5])]);

    let selector = RecursiveCompletionSelector::new();

    let first_run = selector.generate(&memory, &observation);

    let second_run = selector.generate(&memory, &observation);

    assert_eq!(first_run, second_run);

    assert_eq!(memory, before);
}
