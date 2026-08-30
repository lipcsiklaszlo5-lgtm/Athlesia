use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{
    RecursiveConcept, RecursiveMemory, RecursiveObservation, RecursivePredictor, RecursiveUnit,
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
fn base_anchor_predicts_cross_level_unit() {
    let concept = child();

    let observation = RecursiveObservation::new(vec![base(1)]);

    let prediction = RecursivePredictor::new()
        .predict(&concept, &observation)
        .unwrap();

    assert_eq!(prediction.observed_units(), 1);

    assert_eq!(prediction.missing_units(), &[cross(2, &[3, 4],),]);
}

#[test]
fn cross_level_anchor_predicts_base_unit() {
    let concept = child();

    let observation = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let prediction = RecursivePredictor::new()
        .predict(&concept, &observation)
        .unwrap();

    assert_eq!(prediction.missing_units(), &[base(1),]);
}

#[test]
fn recursive_anchor_can_predict_base_unit() {
    let nested = child();

    let concept = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(nested.clone())),
    ])
    .unwrap();

    let observation = RecursiveObservation::new(vec![RecursiveUnit::Recursive(Box::new(nested))]);

    let prediction = RecursivePredictor::new()
        .predict(&concept, &observation)
        .unwrap();

    assert_eq!(prediction.missing_units(), &[base(5),]);
}

#[test]
fn base_anchor_can_predict_recursive_unit() {
    let nested = child();

    let recursive_unit = RecursiveUnit::Recursive(Box::new(nested));

    let concept = RecursiveConcept::new(vec![base(5), recursive_unit.clone()]).unwrap();

    let observation = RecursiveObservation::new(vec![base(5)]);

    let prediction = RecursivePredictor::new()
        .predict(&concept, &observation)
        .unwrap();

    assert_eq!(prediction.missing_units(), &[recursive_unit,]);
}

#[test]
fn complete_recursive_concept_produces_no_prediction() {
    let concept = child();

    let observation = RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]);

    assert!(RecursivePredictor::new()
        .predict(&concept, &observation,)
        .is_none());
}

#[test]
fn zero_overlap_produces_no_prediction() {
    let concept = child();

    let observation = RecursiveObservation::new(vec![base(5), cross(6, &[7, 8])]);

    assert!(RecursivePredictor::new()
        .predict(&concept, &observation,)
        .is_none());
}

#[test]
fn extra_context_does_not_block_prediction() {
    let concept = child();

    let observation = RecursiveObservation::new(vec![base(1), base(5), cross(6, &[7, 8])]);

    let prediction = RecursivePredictor::new()
        .predict(&concept, &observation)
        .unwrap();

    assert_eq!(prediction.missing_units(), &[cross(2, &[3, 4],),]);
}

#[test]
fn multiple_missing_units_are_reported() {
    let concept = RecursiveConcept::new(vec![base(1), base(2), cross(3, &[4, 5])]).unwrap();

    let observation = RecursiveObservation::new(vec![base(1)]);

    let prediction = RecursivePredictor::new()
        .predict(&concept, &observation)
        .unwrap();

    assert_eq!(prediction.missing_count(), 2);
}

#[test]
fn single_missing_unit_is_single_step_completion() {
    let concept = RecursiveConcept::new(vec![base(1), base(2), cross(3, &[4, 5])]).unwrap();

    let observation = RecursiveObservation::new(vec![base(1), base(2)]);

    let prediction = RecursivePredictor::new()
        .predict(&concept, &observation)
        .unwrap();

    assert!(prediction.is_single_step_completion());

    assert_eq!(prediction.missing_units(), &[cross(3, &[4, 5],),]);
}

#[test]
fn cross_level_identity_mismatch_remains_missing() {
    let concept = child();

    let observation = RecursiveObservation::new(vec![base(1), cross(2, &[3, 5])]);

    let prediction = RecursivePredictor::new()
        .predict(&concept, &observation)
        .unwrap();

    assert_eq!(prediction.missing_units(), &[cross(2, &[3, 4],),]);
}

#[test]
fn recursive_depth_mismatch_remains_missing() {
    let level_one = child();

    let level_two = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(level_one.clone())),
    ])
    .unwrap();

    let target_recursive = RecursiveUnit::Recursive(Box::new(level_two));

    let concept = RecursiveConcept::new(vec![base(6), target_recursive.clone()]).unwrap();

    let observation =
        RecursiveObservation::new(vec![base(6), RecursiveUnit::Recursive(Box::new(level_one))]);

    let prediction = RecursivePredictor::new()
        .predict(&concept, &observation)
        .unwrap();

    assert_eq!(prediction.missing_units(), &[target_recursive,]);
}

#[test]
fn memory_can_generate_multiple_predictions() {
    let first = RecursiveConcept::new(vec![base(1), cross(3, &[4, 5])]).unwrap();

    let second = RecursiveConcept::new(vec![base(2), cross(3, &[4, 5])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(first);

    memory.insert(second);

    let observation = RecursiveObservation::new(vec![cross(3, &[4, 5])]);

    let predictions = RecursivePredictor::new().predict_memory(&memory, &observation);

    assert_eq!(predictions.len(), 2);
}

#[test]
fn predictions_prefer_fewer_missing_units() {
    let closer = child();

    let farther = RecursiveConcept::new(vec![base(1), base(5), cross(2, &[3, 4])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(farther);

    memory.insert(closer.clone());

    let observation = RecursiveObservation::new(vec![base(1)]);

    let best = RecursivePredictor::new()
        .best_prediction(&memory, &observation)
        .unwrap();

    assert_eq!(best.concept(), &closer);

    assert_eq!(best.missing_count(), 1);
}

#[test]
fn equal_missing_count_prefers_more_observed_units() {
    let weaker = RecursiveConcept::new(vec![base(1), cross(5, &[6, 7])]).unwrap();

    let stronger = RecursiveConcept::new(vec![base(1), base(2), cross(3, &[4, 5])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(weaker);

    memory.insert(stronger.clone());

    let observation = RecursiveObservation::new(vec![base(1), base(2)]);

    let best = RecursivePredictor::new()
        .best_prediction(&memory, &observation)
        .unwrap();

    assert_eq!(best.concept(), &stronger);

    assert_eq!(best.observed_units(), 2);

    assert_eq!(best.missing_count(), 1);
}

#[test]
fn exact_prediction_tie_uses_concept_identity() {
    let first = RecursiveConcept::new(vec![base(1), cross(3, &[4, 5])]).unwrap();

    let second = RecursiveConcept::new(vec![base(2), cross(3, &[4, 5])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(second.clone());

    memory.insert(first.clone());

    let observation = RecursiveObservation::new(vec![cross(3, &[4, 5])]);

    let best = RecursivePredictor::new()
        .best_prediction(&memory, &observation)
        .unwrap();

    let expected = if first < second { first } else { second };

    assert_eq!(best.concept(), &expected);
}

#[test]
fn prediction_is_deterministic_and_non_mutating() {
    let first = RecursiveConcept::new(vec![base(1), cross(3, &[4, 5])]).unwrap();

    let second = RecursiveConcept::new(vec![base(2), cross(3, &[4, 5])]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(first);

    memory.insert(second);

    let before = memory.clone();

    let observation = RecursiveObservation::new(vec![cross(3, &[4, 5])]);

    let predictor = RecursivePredictor::new();

    let first_run = predictor.predict_memory(&memory, &observation);

    let second_run = predictor.predict_memory(&memory, &observation);

    assert_eq!(first_run, second_run);

    assert_eq!(memory, before);
}
