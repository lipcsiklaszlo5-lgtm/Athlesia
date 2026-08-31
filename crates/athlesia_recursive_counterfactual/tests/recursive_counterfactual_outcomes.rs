use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_counterfactual::{
    RecursiveCounterfactualCandidate, RecursiveCounterfactualOutcome,
    RecursiveCounterfactualProjection, RecursiveCounterfactualProjectionSet,
};

use athlesia_recursive_planning::{RecursivePlanningState, RecursivePlanningTransition};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
    )
}

fn base_abstraction(span: usize) -> AbstractionUnit {
    AbstractionUnit::Structural(structural(span))
}

fn hierarchy(spans: &[usize]) -> HierarchicalConcept {
    HierarchicalConcept::new(spans.iter().copied().map(structural).collect()).unwrap()
}

fn cross_level(span: usize) -> CrossLevelConcept {
    CrossLevelConcept::new(vec![
        base_abstraction(span),
        AbstractionUnit::Hierarchical(hierarchy(&[span + 1, span + 2])),
    ])
    .unwrap()
}

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(base_abstraction(span))
}

fn cross_unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::CrossLevel(cross_level(span))
}

fn valid_recursive_concept(base_span: usize, cross_span: usize) -> RecursiveConcept {
    RecursiveConcept::new(vec![unit(base_span), cross_unit(cross_span)]).unwrap()
}

fn state(spans: &[usize]) -> RecursivePlanningState {
    RecursivePlanningState::new(spans.iter().copied().map(unit).collect())
}

fn transition(from: &[usize], to: &[usize]) -> RecursivePlanningTransition {
    RecursivePlanningTransition::new(state(from), state(to)).unwrap()
}

fn candidate(from: &[usize], to: &[usize], cost: usize) -> RecursiveCounterfactualCandidate {
    RecursiveCounterfactualCandidate::new(transition(from, to), cost).unwrap()
}

fn outcome(spans: &[usize]) -> RecursiveCounterfactualOutcome {
    RecursiveCounterfactualOutcome::new(state(spans))
}

#[test]
fn outcome_preserves_state_identity() {
    let expected = state(&[1, 2]);

    let outcome = RecursiveCounterfactualOutcome::new(expected.clone());

    assert_eq!(outcome.state(), &expected);
}

#[test]
fn empty_projection_is_rejected() {
    assert!(
        RecursiveCounterfactualProjection::new(candidate(&[1], &[1, 2], 1,), Vec::new(),).is_none()
    );
}

#[test]
fn single_outcome_projection_is_deterministic() {
    let projection =
        RecursiveCounterfactualProjection::new(candidate(&[1], &[1, 2], 1), vec![outcome(&[1, 2])])
            .unwrap();

    assert!(projection.is_deterministic());

    assert!(!projection.is_branching());

    assert_eq!(projection.outcome_count(), 1);
}

#[test]
fn multiple_outcomes_form_branching_projection() {
    let projection = RecursiveCounterfactualProjection::new(
        candidate(&[1], &[1, 2], 1),
        vec![outcome(&[1, 2]), outcome(&[1, 3])],
    )
    .unwrap();

    assert!(projection.is_branching());

    assert_eq!(projection.outcome_count(), 2);
}

#[test]
fn duplicate_outcomes_are_deduplicated() {
    let projection = RecursiveCounterfactualProjection::new(
        candidate(&[1], &[1, 2], 1),
        vec![outcome(&[1, 2]), outcome(&[1, 2])],
    )
    .unwrap();

    assert_eq!(projection.outcome_count(), 1);
}

#[test]
fn outcome_order_is_canonical() {
    let left = RecursiveCounterfactualProjection::new(
        candidate(&[1], &[1, 2], 1),
        vec![outcome(&[1, 3]), outcome(&[1, 2])],
    )
    .unwrap();

    let right = RecursiveCounterfactualProjection::new(
        candidate(&[1], &[1, 2], 1),
        vec![outcome(&[1, 2]), outcome(&[1, 3])],
    )
    .unwrap();

    assert_eq!(left, right);
}

#[test]
fn projection_preserves_candidate_identity() {
    let expected = candidate(&[1], &[1, 2], 3);

    let projection =
        RecursiveCounterfactualProjection::new(expected.clone(), vec![outcome(&[1, 2])]).unwrap();

    assert_eq!(projection.candidate(), &expected);
}

#[test]
fn projection_supports_exact_state_membership() {
    let projection = RecursiveCounterfactualProjection::new(
        candidate(&[1], &[1, 2], 1),
        vec![outcome(&[1, 2]), outcome(&[1, 3])],
    )
    .unwrap();

    assert!(projection.contains_state(&state(&[1, 3]),));

    assert!(!projection.contains_state(&state(&[1, 4]),));
}

#[test]
fn projection_set_is_deterministic_under_input_order() {
    let first =
        RecursiveCounterfactualProjection::new(candidate(&[1], &[1, 2], 1), vec![outcome(&[1, 2])])
            .unwrap();

    let second =
        RecursiveCounterfactualProjection::new(candidate(&[1], &[1, 3], 2), vec![outcome(&[1, 3])])
            .unwrap();

    let left = RecursiveCounterfactualProjectionSet::new(vec![first.clone(), second.clone()]);

    let right = RecursiveCounterfactualProjectionSet::new(vec![second, first]);

    assert_eq!(left, right);
}

#[test]
fn projection_set_deduplicates_exact_projections() {
    let projection =
        RecursiveCounterfactualProjection::new(candidate(&[1], &[1, 2], 1), vec![outcome(&[1, 2])])
            .unwrap();

    let set = RecursiveCounterfactualProjectionSet::new(vec![projection.clone(), projection]);

    assert_eq!(set.len(), 1);
}

#[test]
fn recursive_depth_identity_survives_outcome_projection() {
    let child = valid_recursive_concept(1, 2);

    let deep =
        RecursiveConcept::new(vec![unit(8), RecursiveUnit::Recursive(Box::new(child))]).unwrap();

    let predicted = RecursivePlanningState::new(vec![RecursiveUnit::Recursive(Box::new(deep))]);

    let projection = RecursiveCounterfactualProjection::new(
        candidate(&[1], &[1, 2], 1),
        vec![RecursiveCounterfactualOutcome::new(predicted.clone())],
    )
    .unwrap();

    assert!(projection.contains_state(&predicted,));

    assert_eq!(projection.outcomes().first().unwrap().state(), &predicted);
}

#[test]
fn projection_construction_does_not_mutate_inputs() {
    let source_candidate = candidate(&[1], &[1, 2], 2);

    let source_outcomes = vec![outcome(&[1, 3]), outcome(&[1, 2])];

    let candidate_before = source_candidate.clone();

    let outcomes_before = source_outcomes.clone();

    let _ =
        RecursiveCounterfactualProjection::new(source_candidate.clone(), source_outcomes.clone())
            .unwrap();

    assert_eq!(source_candidate, candidate_before);

    assert_eq!(source_outcomes, outcomes_before);
}
