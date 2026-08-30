use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{
    AbstractionUnit, CrossLevelConcept, CrossLevelMemory, CrossLevelObservation,
    CrossLevelPredictor,
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
fn structural_unit_can_predict_hierarchical_unit() {
    let target = cross_level(1, &[2, 3]);

    let observation = CrossLevelObservation::new(vec![structural_unit(1)]);

    let prediction = CrossLevelPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.observed_units(), 1);

    assert_eq!(prediction.missing_units(), &[hierarchical_unit(&[2, 3],),]);
}

#[test]
fn hierarchical_unit_can_predict_structural_unit() {
    let target = cross_level(1, &[2, 3]);

    let observation = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3])]);

    let prediction = CrossLevelPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.missing_units(), &[structural_unit(1),]);
}

#[test]
fn complete_cross_level_concept_produces_no_prediction() {
    let target = cross_level(1, &[2, 3]);

    let observation =
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]);

    assert!(CrossLevelPredictor::new()
        .predict(&target, &observation,)
        .is_none());
}

#[test]
fn zero_overlap_produces_no_prediction() {
    let target = cross_level(1, &[2, 3]);

    let observation =
        CrossLevelObservation::new(vec![structural_unit(4), hierarchical_unit(&[5, 6])]);

    assert!(CrossLevelPredictor::new()
        .predict(&target, &observation,)
        .is_none());
}

#[test]
fn extra_context_does_not_block_prediction() {
    let target = cross_level(1, &[2, 3]);

    let observation = CrossLevelObservation::new(vec![
        structural_unit(1),
        structural_unit(4),
        hierarchical_unit(&[5, 6]),
    ]);

    let prediction = CrossLevelPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.missing_units(), &[hierarchical_unit(&[2, 3],),]);
}

#[test]
fn multiple_missing_units_are_reported() {
    let target = CrossLevelConcept::new(vec![
        structural_unit(1),
        structural_unit(2),
        hierarchical_unit(&[3, 4]),
    ])
    .unwrap();

    let observation = CrossLevelObservation::new(vec![structural_unit(1)]);

    let prediction = CrossLevelPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.missing_count(), 2);
}

#[test]
fn single_missing_unit_is_single_step_completion() {
    let target = CrossLevelConcept::new(vec![
        structural_unit(1),
        structural_unit(2),
        hierarchical_unit(&[3, 4]),
    ])
    .unwrap();

    let observation = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let prediction = CrossLevelPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert!(prediction.is_single_step_completion());

    assert_eq!(prediction.missing_units(), &[hierarchical_unit(&[3, 4],),]);
}

#[test]
fn structural_extent_mismatch_remains_missing() {
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
        AbstractionUnit::Structural(short.clone()),
        hierarchical.clone(),
    ])
    .unwrap();

    let observation =
        CrossLevelObservation::new(vec![AbstractionUnit::Structural(long), hierarchical]);

    let prediction = CrossLevelPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(
        prediction.missing_units(),
        &[AbstractionUnit::Structural(short,),]
    );
}

#[test]
fn hierarchy_identity_mismatch_remains_missing() {
    let target = cross_level(1, &[2, 3]);

    let observation =
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 4])]);

    let prediction = CrossLevelPredictor::new()
        .predict(&target, &observation)
        .unwrap();

    assert_eq!(prediction.missing_units(), &[hierarchical_unit(&[2, 3],),]);
}

#[test]
fn memory_can_generate_multiple_predictions() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[3, 4]));

    memory.insert(cross_level(2, &[3, 4]));

    let observation = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let predictions = CrossLevelPredictor::new().predict_memory(&memory, &observation);

    assert_eq!(predictions.len(), 2);
}

#[test]
fn predictions_prefer_fewer_missing_units() {
    let mut memory = CrossLevelMemory::new();

    let closer = cross_level(1, &[3, 4]);

    let farther = CrossLevelConcept::new(vec![
        structural_unit(1),
        structural_unit(2),
        hierarchical_unit(&[3, 4]),
    ])
    .unwrap();

    memory.insert(farther);

    memory.insert(closer.clone());

    let observation = CrossLevelObservation::new(vec![structural_unit(1)]);

    let best = CrossLevelPredictor::new()
        .best_prediction(&memory, &observation)
        .unwrap();

    assert_eq!(best.concept(), &closer);

    assert_eq!(best.missing_count(), 1);
}

#[test]
fn equal_missing_count_prefers_more_observed_units() {
    let mut memory = CrossLevelMemory::new();

    let weaker = cross_level(1, &[4, 5]);

    let stronger = CrossLevelConcept::new(vec![
        structural_unit(1),
        structural_unit(2),
        hierarchical_unit(&[3, 4]),
    ])
    .unwrap();

    memory.insert(weaker);

    memory.insert(stronger.clone());

    let observation = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let best = CrossLevelPredictor::new()
        .best_prediction(&memory, &observation)
        .unwrap();

    assert_eq!(best.concept(), &stronger);

    assert_eq!(best.observed_units(), 2);

    assert_eq!(best.missing_count(), 1);
}

#[test]
fn exact_prediction_tie_uses_concept_identity() {
    let mut memory = CrossLevelMemory::new();

    let first = cross_level(1, &[3, 4]);

    let second = cross_level(2, &[3, 4]);

    memory.insert(second.clone());

    memory.insert(first.clone());

    let observation = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let best = CrossLevelPredictor::new()
        .best_prediction(&memory, &observation)
        .unwrap();

    let expected = if first < second { first } else { second };

    assert_eq!(best.concept(), &expected);
}

#[test]
fn prediction_is_deterministic_and_non_mutating() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[3, 4]));

    memory.insert(cross_level(2, &[3, 4]));

    let before = memory.clone();

    let observation = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let predictor = CrossLevelPredictor::new();

    let first = predictor.predict_memory(&memory, &observation);

    let second = predictor.predict_memory(&memory, &observation);

    assert_eq!(first, second);

    assert_eq!(memory, before);
}
