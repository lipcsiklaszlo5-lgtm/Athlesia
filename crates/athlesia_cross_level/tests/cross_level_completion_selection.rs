use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{
    AbstractionUnit, CrossLevelCompletionSelector, CrossLevelConcept, CrossLevelMemory,
    CrossLevelObservation,
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
fn no_predictions_produce_no_candidate() {
    let memory = CrossLevelMemory::new();

    let observation = CrossLevelObservation::new(vec![structural_unit(1)]);

    assert!(CrossLevelCompletionSelector::new()
        .select(&memory, &observation,)
        .is_none());
}

#[test]
fn complete_concept_produces_no_candidate() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let observation =
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]);

    assert!(CrossLevelCompletionSelector::new()
        .select(&memory, &observation,)
        .is_none());
}

#[test]
fn zero_overlap_produces_no_candidate() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let observation =
        CrossLevelObservation::new(vec![structural_unit(4), hierarchical_unit(&[5, 6])]);

    assert!(CrossLevelCompletionSelector::new()
        .select(&memory, &observation,)
        .is_none());
}

#[test]
fn structural_anchor_selects_hierarchical_target() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let observation = CrossLevelObservation::new(vec![structural_unit(1)]);

    let selected = CrossLevelCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &hierarchical_unit(&[2, 3],));
}

#[test]
fn hierarchical_anchor_selects_structural_target() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let observation = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3])]);

    let selected = CrossLevelCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &structural_unit(1));
}

#[test]
fn shared_missing_unit_accumulates_support() {
    let mut memory = CrossLevelMemory::new();

    let shared = hierarchical_unit(&[3, 4]);

    memory.insert(CrossLevelConcept::new(vec![structural_unit(1), shared.clone()]).unwrap());

    memory.insert(CrossLevelConcept::new(vec![structural_unit(2), shared.clone()]).unwrap());

    let observation = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let selected = CrossLevelCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &shared);

    assert_eq!(selected.supporting_concepts(), 2);

    assert_eq!(selected.single_step_support(), 2);
}

#[test]
fn selector_prefers_more_support() {
    let mut memory = CrossLevelMemory::new();

    let shared = hierarchical_unit(&[3, 4]);

    memory.insert(CrossLevelConcept::new(vec![structural_unit(1), shared.clone()]).unwrap());

    memory.insert(CrossLevelConcept::new(vec![structural_unit(2), shared.clone()]).unwrap());

    memory.insert(cross_level(1, &[5, 6]));

    let observation = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let selected = CrossLevelCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &shared);

    assert_eq!(selected.supporting_concepts(), 2);
}

#[test]
fn single_step_support_breaks_equal_support_tie() {
    let mut memory = CrossLevelMemory::new();

    let preferred = hierarchical_unit(&[3, 4]);

    let weaker = hierarchical_unit(&[5, 6]);

    memory.insert(CrossLevelConcept::new(vec![structural_unit(1), preferred.clone()]).unwrap());

    memory.insert(
        CrossLevelConcept::new(vec![structural_unit(1), structural_unit(2), weaker.clone()])
            .unwrap(),
    );

    let observation = CrossLevelObservation::new(vec![structural_unit(1)]);

    let candidates = CrossLevelCompletionSelector::new().generate(&memory, &observation);

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
fn exact_tie_uses_unit_identity() {
    let mut memory = CrossLevelMemory::new();

    let first = structural_unit(1);

    let second = structural_unit(2);

    let hierarchical = hierarchical_unit(&[3, 4]);

    memory.insert(CrossLevelConcept::new(vec![first.clone(), hierarchical.clone()]).unwrap());

    memory.insert(CrossLevelConcept::new(vec![second.clone(), hierarchical.clone()]).unwrap());

    let observation = CrossLevelObservation::new(vec![hierarchical]);

    let selected = CrossLevelCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    let expected = if first < second { first } else { second };

    assert_eq!(selected.unit(), &expected);
}

#[test]
fn multi_missing_prediction_supports_each_unit() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(
        CrossLevelConcept::new(vec![
            structural_unit(1),
            structural_unit(2),
            hierarchical_unit(&[3, 4]),
        ])
        .unwrap(),
    );

    let observation = CrossLevelObservation::new(vec![structural_unit(1)]);

    let candidates = CrossLevelCompletionSelector::new().generate(&memory, &observation);

    assert_eq!(candidates.len(), 2);

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &structural_unit(2) }));

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &hierarchical_unit(&[3, 4],) }));
}

#[test]
fn extra_context_does_not_block_selection() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let observation = CrossLevelObservation::new(vec![
        structural_unit(1),
        structural_unit(4),
        hierarchical_unit(&[5, 6]),
    ]);

    let selected = CrossLevelCompletionSelector::new()
        .select(&memory, &observation)
        .unwrap();

    assert_eq!(selected.unit(), &hierarchical_unit(&[2, 3],));
}

#[test]
fn structural_extent_identity_is_preserved() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        6,
    );

    let anchor = hierarchical_unit(&[2, 3]);

    let mut memory = CrossLevelMemory::new();

    memory.insert(
        CrossLevelConcept::new(vec![
            AbstractionUnit::Structural(short.clone()),
            anchor.clone(),
        ])
        .unwrap(),
    );

    memory.insert(
        CrossLevelConcept::new(vec![
            AbstractionUnit::Structural(long.clone()),
            anchor.clone(),
        ])
        .unwrap(),
    );

    let observation = CrossLevelObservation::new(vec![anchor]);

    let candidates = CrossLevelCompletionSelector::new().generate(&memory, &observation);

    assert_eq!(candidates.len(), 2);

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &AbstractionUnit::Structural(short.clone(),) }));

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &AbstractionUnit::Structural(long.clone(),) }));
}

#[test]
fn hierarchy_identity_is_preserved() {
    let structural = structural_unit(1);

    let first = hierarchical_unit(&[2, 3]);

    let second = hierarchical_unit(&[2, 4]);

    let mut memory = CrossLevelMemory::new();

    memory.insert(CrossLevelConcept::new(vec![structural.clone(), first.clone()]).unwrap());

    memory.insert(CrossLevelConcept::new(vec![structural.clone(), second.clone()]).unwrap());

    let observation = CrossLevelObservation::new(vec![structural]);

    let candidates = CrossLevelCompletionSelector::new().generate(&memory, &observation);

    assert_eq!(candidates.len(), 2);

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &first }));

    assert!(candidates
        .iter()
        .any(|candidate| { candidate.unit() == &second }));
}

#[test]
fn generation_is_deterministic_and_non_mutating() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[3, 4]));

    memory.insert(cross_level(2, &[3, 4]));

    let before = memory.clone();

    let observation = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let selector = CrossLevelCompletionSelector::new();

    let first = selector.generate(&memory, &observation);

    let second = selector.generate(&memory, &observation);

    assert_eq!(first, second);

    assert_eq!(memory, before);
}

#[test]
fn insertion_order_does_not_change_selection() {
    let first_concept = cross_level(1, &[3, 4]);

    let second_concept = cross_level(2, &[3, 4]);

    let mut first = CrossLevelMemory::new();

    first.insert(first_concept.clone());

    first.insert(second_concept.clone());

    let mut second = CrossLevelMemory::new();

    second.insert(second_concept);

    second.insert(first_concept);

    let observation = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let selector = CrossLevelCompletionSelector::new();

    assert_eq!(
        selector.select(&first, &observation,),
        selector.select(&second, &observation,)
    );
}
