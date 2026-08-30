use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_hierarchy::{
    HierarchicalConcept, HierarchicalMemory, HierarchyCompletionEvaluator,
    HierarchyCompletionOutcome, HierarchyCompletionSelector, HierarchyObservation,
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

fn memory() -> HierarchicalMemory {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 3]));

    memory.insert(hierarchy(&[2, 3]));

    memory
}

#[test]
fn appearing_target_confirms_completion() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let candidate = HierarchyCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    assert_eq!(candidate.child(), &concept(3));

    let next = HierarchyObservation::new(vec![concept(3)]);

    let evaluation = HierarchyCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(evaluation.outcome(), HierarchyCompletionOutcome::Confirmed);

    assert!(evaluation.is_confirmed());

    assert!(!evaluation.is_violated());
}

#[test]
fn absent_target_violates_completion() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let candidate = HierarchyCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = HierarchyObservation::new(vec![concept(4)]);

    let evaluation = HierarchyCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(evaluation.outcome(), HierarchyCompletionOutcome::Violated);

    assert!(evaluation.is_violated());

    assert!(!evaluation.is_confirmed());
}

#[test]
fn evaluation_preserves_target_identity() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let candidate = HierarchyCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = HierarchyObservation::new(vec![concept(3)]);

    let evaluation = HierarchyCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(evaluation.target(), candidate.child());
}

#[test]
fn evaluation_preserves_support_snapshot() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let candidate = HierarchyCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = HierarchyObservation::new(vec![concept(3)]);

    let evaluation = HierarchyCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(
        evaluation.supporting_hierarchies(),
        candidate.supporting_hierarchies()
    );

    assert_eq!(
        evaluation.single_step_support(),
        candidate.single_step_support()
    );
}

#[test]
fn unrelated_context_does_not_confirm_target() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let candidate = HierarchyCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = HierarchyObservation::new(vec![concept(4), concept(5)]);

    let evaluation = HierarchyCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_violated());
}

#[test]
fn extra_context_does_not_block_confirmation() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let candidate = HierarchyCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = HierarchyObservation::new(vec![concept(3), concept(4), concept(5)]);

    let evaluation = HierarchyCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_confirmed());
}

#[test]
fn extent_mismatch_is_violation() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 3)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 3)],
        6,
    );

    let first = concept(1);

    let mut memory = HierarchicalMemory::new();

    memory.insert(HierarchicalConcept::new(vec![first.clone(), short.clone()]).unwrap());

    let prior = HierarchyObservation::new(vec![first]);

    let candidate = HierarchyCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    assert_eq!(candidate.child(), &short);

    let next = HierarchyObservation::new(vec![long]);

    let evaluation = HierarchyCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_violated());
}

#[test]
fn select_and_evaluate_runs_complete_path() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let result = HierarchyCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next)
        .unwrap();

    let (candidate, evaluation) = result;

    assert_eq!(candidate.child(), evaluation.target());

    assert!(evaluation.is_confirmed());
}

#[test]
fn no_candidate_means_no_evaluation() {
    let mut memory = HierarchicalMemory::new();

    memory.insert(hierarchy(&[1, 2]));

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    assert!(HierarchyCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn empty_memory_means_no_evaluation() {
    let memory = HierarchicalMemory::new();

    let prior = HierarchyObservation::new(vec![concept(1)]);

    let next = HierarchyObservation::new(vec![concept(2)]);

    assert!(HierarchyCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn evaluation_is_deterministic() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let evaluator = HierarchyCompletionEvaluator::new();

    assert_eq!(
        evaluator.select_and_evaluate(&memory, &prior, &next,),
        evaluator.select_and_evaluate(&memory, &prior, &next,)
    );
}

#[test]
fn evaluation_does_not_mutate_memory() {
    let memory = memory();
    let before = memory.clone();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let _ = HierarchyCompletionEvaluator::new().select_and_evaluate(&memory, &prior, &next);

    assert_eq!(memory, before);
}

#[test]
fn evaluation_contains_structural_information_only() {
    let memory = memory();

    let prior = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let next = HierarchyObservation::new(vec![concept(3)]);

    let (candidate, evaluation) = HierarchyCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next)
        .unwrap();

    assert_eq!(candidate.child(), evaluation.target());

    assert!(evaluation.supporting_hierarchies() > 0);
}
