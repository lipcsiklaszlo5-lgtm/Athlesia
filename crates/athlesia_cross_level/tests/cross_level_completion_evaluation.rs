use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{
    AbstractionUnit, CrossLevelCompletionEvaluator, CrossLevelCompletionOutcome,
    CrossLevelCompletionSelector, CrossLevelConcept, CrossLevelMemory, CrossLevelObservation,
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

fn memory() -> CrossLevelMemory {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[3, 4]));

    memory.insert(cross_level(2, &[3, 4]));

    memory
}

#[test]
fn appearing_hierarchical_target_confirms_completion() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let prior = CrossLevelObservation::new(vec![structural_unit(1)]);

    let candidate = CrossLevelCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    assert_eq!(candidate.unit(), &hierarchical_unit(&[2, 3],));

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3])]);

    let evaluation = CrossLevelCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(evaluation.outcome(), CrossLevelCompletionOutcome::Confirmed);

    assert!(evaluation.is_confirmed());

    assert!(!evaluation.is_violated());
}

#[test]
fn appearing_structural_target_confirms_completion() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let prior = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3])]);

    let candidate = CrossLevelCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    assert_eq!(candidate.unit(), &structural_unit(1));

    let next = CrossLevelObservation::new(vec![structural_unit(1)]);

    let evaluation = CrossLevelCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_confirmed());
}

#[test]
fn absent_target_violates_completion() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let candidate = CrossLevelCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[5, 6])]);

    let evaluation = CrossLevelCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(evaluation.outcome(), CrossLevelCompletionOutcome::Violated);

    assert!(evaluation.is_violated());

    assert!(!evaluation.is_confirmed());
}

#[test]
fn evaluation_preserves_target_identity() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let candidate = CrossLevelCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let evaluation = CrossLevelCompletionEvaluator::new().evaluate(&candidate, &next);

    assert_eq!(evaluation.target(), candidate.unit());
}

#[test]
fn evaluation_preserves_support_snapshot() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let candidate = CrossLevelCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    assert_eq!(candidate.supporting_concepts(), 2);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let evaluation = CrossLevelCompletionEvaluator::new().evaluate(&candidate, &next);

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
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let candidate = CrossLevelCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = CrossLevelObservation::new(vec![structural_unit(5), hierarchical_unit(&[6, 7])]);

    let evaluation = CrossLevelCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_violated());
}

#[test]
fn extra_context_does_not_block_confirmation() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let candidate = CrossLevelCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = CrossLevelObservation::new(vec![
        hierarchical_unit(&[3, 4]),
        structural_unit(5),
        hierarchical_unit(&[6, 7]),
    ]);

    let evaluation = CrossLevelCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_confirmed());
}

#[test]
fn structural_extent_mismatch_is_violation() {
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

    let prior = CrossLevelObservation::new(vec![anchor]);

    let candidate = CrossLevelCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    assert_eq!(candidate.unit(), &AbstractionUnit::Structural(short,));

    let next = CrossLevelObservation::new(vec![AbstractionUnit::Structural(long)]);

    let evaluation = CrossLevelCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_violated());
}

#[test]
fn hierarchy_identity_mismatch_is_violation() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let prior = CrossLevelObservation::new(vec![structural_unit(1)]);

    let candidate = CrossLevelCompletionSelector::new()
        .select(&memory, &prior)
        .unwrap();

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 4])]);

    let evaluation = CrossLevelCompletionEvaluator::new().evaluate(&candidate, &next);

    assert!(evaluation.is_violated());
}

#[test]
fn select_and_evaluate_runs_complete_path() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let (candidate, evaluation) = CrossLevelCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next)
        .unwrap();

    assert_eq!(candidate.unit(), evaluation.target());

    assert!(evaluation.is_confirmed());
}

#[test]
fn no_candidate_means_no_evaluation() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let prior = CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]);

    let next = CrossLevelObservation::new(vec![structural_unit(4)]);

    assert!(CrossLevelCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn empty_memory_means_no_evaluation() {
    let memory = CrossLevelMemory::new();

    let prior = CrossLevelObservation::new(vec![structural_unit(1)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3])]);

    assert!(CrossLevelCompletionEvaluator::new()
        .select_and_evaluate(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn evaluation_is_deterministic() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let evaluator = CrossLevelCompletionEvaluator::new();

    assert_eq!(
        evaluator.select_and_evaluate(&memory, &prior, &next,),
        evaluator.select_and_evaluate(&memory, &prior, &next,)
    );
}

#[test]
fn evaluation_does_not_mutate_memory() {
    let memory = memory();
    let before = memory.clone();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let _ = CrossLevelCompletionEvaluator::new().select_and_evaluate(&memory, &prior, &next);

    assert_eq!(memory, before);
}
