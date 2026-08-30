use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{
    RecursiveCompletionEvaluator, RecursiveCompletionOutcome, RecursiveCompletionSelector,
    RecursiveConcept, RecursiveMemory, RecursiveObservation, RecursiveUnit,
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
fn appearing_cross_level_target_confirms_completion() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(evaluation.outcome(), RecursiveCompletionOutcome::Confirmed);

    assert!(evaluation.is_confirmed());

    assert!(!evaluation.is_violated());
}

#[test]
fn appearing_base_target_confirms_completion() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = RecursiveObservation::new(vec![base(1)]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_confirmed());
}

#[test]
fn appearing_recursive_target_confirms_completion() {
    let nested = child();

    let recursive_target = RecursiveUnit::Recursive(Box::new(nested));

    let concept = RecursiveConcept::new(vec![base(5), recursive_target.clone()]).unwrap();

    let mut memory = RecursiveMemory::new();

    memory.insert(concept);

    let prior = RecursiveObservation::new(vec![base(5)]);

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = RecursiveObservation::new(vec![recursive_target]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_confirmed());
}

#[test]
fn absent_target_violates_completion() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = RecursiveObservation::new(vec![cross(5, &[6, 7])]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(evaluation.outcome(), RecursiveCompletionOutcome::Violated);

    assert!(evaluation.is_violated());
}

#[test]
fn evaluation_preserves_target_identity() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(evaluation.target(), candidate.unit());
}

#[test]
fn evaluation_preserves_support_snapshot() {
    let shared = cross(3, &[4, 5]);

    let mut memory = RecursiveMemory::new();

    memory.insert(RecursiveConcept::new(vec![base(1), shared.clone()]).unwrap());

    memory.insert(RecursiveConcept::new(vec![base(2), shared.clone()]).unwrap());

    let prior = RecursiveObservation::new(vec![base(1), base(2)]);

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    assert_eq!(candidate.supporting_concepts(), 2);

    let next = RecursiveObservation::new(vec![shared]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(
        evaluation.supporting_concepts(),
        candidate.supporting_concepts()
    );

    assert_eq!(
        evaluation.single_step_support(),
        candidate.single_step_support()
    );
}

#[test]
fn unrelated_context_does_not_confirm_target() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = RecursiveObservation::new(vec![base(5), cross(6, &[7, 8])]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_violated());
}

#[test]
fn extra_context_does_not_block_confirmation() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4]), base(5), cross(6, &[7, 8])]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_confirmed());
}

#[test]
fn cross_level_identity_mismatch_is_violation() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = RecursiveObservation::new(vec![cross(2, &[3, 5])]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_violated());
}

#[test]
fn recursive_depth_mismatch_is_violation() {
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

    let candidate = RecursiveCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    assert_eq!(candidate.unit(), &target);

    let next = RecursiveObservation::new(vec![RecursiveUnit::Recursive(Box::new(level_one))]);

    let evaluation = RecursiveCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_violated());
}

#[test]
fn select_and_evaluate_runs_complete_path() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    let (candidate, evaluation) = RecursiveCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next)
        .unwrap();

    assert_eq!(candidate.unit(), evaluation.target());

    assert!(evaluation.is_confirmed());
}

#[test]
fn no_candidate_means_no_evaluation() {
    let mut memory = RecursiveMemory::new();

    memory.insert(child());

    let prior = RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]);

    let next = RecursiveObservation::new(vec![base(5)]);

    assert!(RecursiveCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn empty_memory_means_no_evaluation() {
    let memory = RecursiveMemory::new();

    let prior = RecursiveObservation::new(vec![base(1)]);

    let next = RecursiveObservation::new(vec![cross(2, &[3, 4])]);

    assert!(RecursiveCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn evaluation_is_deterministic_and_non_mutating() {
    let shared = cross(3, &[4, 5]);

    let mut memory = RecursiveMemory::new();

    memory.insert(RecursiveConcept::new(vec![base(1), shared.clone()]).unwrap());

    memory.insert(RecursiveConcept::new(vec![base(2), shared.clone()]).unwrap());

    let before = memory.clone();

    let prior = RecursiveObservation::new(vec![base(1), base(2)]);

    let next = RecursiveObservation::new(vec![shared]);

    let evaluator = RecursiveCompletionEvaluator::new();

    let first = evaluator.select_and_evaluate(&memory, &prior, &next);

    let second = evaluator.select_and_evaluate(&memory, &prior, &next);

    assert_eq!(first, second);

    assert_eq!(memory, before);
}
