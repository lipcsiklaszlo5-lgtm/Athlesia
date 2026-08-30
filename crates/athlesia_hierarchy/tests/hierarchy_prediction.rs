use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_hierarchy::{
    HierarchicalConcept, HierarchicalMemory, HierarchyObservation, HierarchyPredictor,
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
fn partial_hierarchy_predicts_missing_child() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let prediction = HierarchyPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.observed_children(), 1);

    assert_eq!(prediction.missing_children(), &[concept(2)]);
}

#[test]
fn complete_hierarchy_produces_no_prediction() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    assert!(HierarchyPredictor::new()
        .predict(&target, &observation,)
        .is_none());
}

#[test]
fn zero_overlap_produces_no_prediction() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(3), concept(4)]);

    assert!(HierarchyPredictor::new()
        .predict(&target, &observation,)
        .is_none());
}

#[test]
fn extra_context_does_not_block_prediction() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(1), concept(3), concept(4)]);

    let prediction = HierarchyPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.missing_children(), &[concept(2)]);
}

#[test]
fn multiple_missing_children_are_reported() {
    let target = hierarchy(&[1, 2, 3]);

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let prediction = HierarchyPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.missing_count(), 2);

    assert_eq!(prediction.missing_children(), &[concept(2), concept(3),]);
}

#[test]
fn single_missing_child_is_single_step_completion() {
    let target = hierarchy(&[1, 2, 3]);

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let prediction = HierarchyPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert!(prediction.is_single_step_completion());

    assert_eq!(prediction.missing_children(), &[concept(3)]);
}

#[test]
fn extent_mismatch_is_not_counted_as_observed_child() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        6,
    );

    let second = concept(2);

    let target = HierarchicalConcept::new(vec![short.clone(), second.clone()]).unwrap();

    let observation = HierarchyObservation::new(vec![long, second]);

    let prediction = HierarchyPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.observed_children(), 1);

    assert_eq!(prediction.missing_children(), &[short]);
}

#[test]
fn memory_can_generate_multiple_predictions() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    memory.insert(hierarchy(&[1, 3]));

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let predictions = HierarchyPredictor::new().predict_memory(&memory, &observation);

    assert_eq!(predictions.len(), 2);
}

#[test]
fn predictions_prefer_fewer_missing_children() {
    let mut memory = HierarchicalMemory::new();

    let closer = hierarchy(&[1, 2]);

    let farther = hierarchy(&[1, 3, 4]);

    memory.insert(farther);

    memory.insert(closer.clone());

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let best = HierarchyPredictor::new()
        .best_prediction(&memory, &observation)
        .unwrap();

    assert_eq!(best.hierarchy(), &closer);

    assert_eq!(best.missing_count(), 1);
}

#[test]
fn equal_missing_count_prefers_more_observed_children() {
    let mut memory = HierarchicalMemory::new();

    let weaker = hierarchy(&[1, 4]);

    let stronger = hierarchy(&[1, 2, 3]);

    memory.insert(weaker);

    memory.insert(stronger.clone());

    let observation = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let best = HierarchyPredictor::new()
        .best_prediction(&memory, &observation)
        .unwrap();

    assert_eq!(best.hierarchy(), &stronger);

    assert_eq!(best.observed_children(), 2);

    assert_eq!(best.missing_count(), 1);
}

#[test]
fn exact_prediction_tie_uses_hierarchy_identity() {
    let mut memory = HierarchicalMemory::new();

    let first = hierarchy(&[1, 2]);

    let second = hierarchy(&[1, 3]);

    memory.insert(second.clone());

    memory.insert(first.clone());

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let best = HierarchyPredictor::new()
        .best_prediction(&memory, &observation)
        .unwrap();

    let expected = if first < second { first } else { second };

    assert_eq!(best.hierarchy(), &expected);
}

#[test]
fn prediction_is_deterministic() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    memory.insert(hierarchy(&[1, 3]));

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let predictor = HierarchyPredictor::new();

    assert_eq!(
        predictor.predict_memory(&memory, &observation,),
        predictor.predict_memory(&memory, &observation,)
    );
}

#[test]
fn prediction_does_not_mutate_memory() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    memory.insert(hierarchy(&[1, 3]));

    let before = memory.clone();

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let predictor = HierarchyPredictor::new();

    let _ = predictor.predict_memory(&memory, &observation);

    let _ = predictor.best_prediction(&memory, &observation);

    assert_eq!(memory, before);
}

#[test]
fn prediction_contains_structural_concepts_only() {
    let target = hierarchy(&[1, 2]);

    let observation = HierarchyObservation::new(vec![concept(1)]);

    let prediction = HierarchyPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.missing_children().len(), 1);

    assert!(target.contains(&prediction.missing_children()[0]));
}
