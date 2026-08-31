use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_counterfactual::{
    RecursiveCounterfactualCandidate, RecursiveCounterfactualSet,
};

use athlesia_recursive_planning::{RecursivePlanningState, RecursivePlanningTransition};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
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

fn recursive_concept(base_span: usize, cross_span: usize) -> RecursiveConcept {
    RecursiveConcept::new(vec![
        base(base_span),
        cross(cross_span, &[cross_span + 1, cross_span + 2]),
    ])
    .unwrap()
}

fn state(units: Vec<RecursiveUnit>) -> RecursivePlanningState {
    RecursivePlanningState::new(units)
}

fn transition(from_span: usize, to_span: usize) -> RecursivePlanningTransition {
    RecursivePlanningTransition::new(
        state(vec![base(from_span)]),
        state(vec![base(from_span), base(to_span)]),
    )
    .unwrap()
}

#[test]
fn zero_interaction_cost_is_rejected() {
    assert!(RecursiveCounterfactualCandidate::new(transition(1, 2), 0,).is_none());
}

#[test]
fn positive_interaction_cost_is_accepted() {
    assert!(RecursiveCounterfactualCandidate::new(transition(1, 2), 1,).is_some());
}

#[test]
fn candidate_preserves_transition_identity() {
    let expected = transition(1, 2);

    let candidate = RecursiveCounterfactualCandidate::new(expected.clone(), 3).unwrap();

    assert_eq!(candidate.transition(), &expected);
}

#[test]
fn candidate_preserves_interaction_cost() {
    let candidate = RecursiveCounterfactualCandidate::new(transition(1, 2), 7).unwrap();

    assert_eq!(candidate.interaction_cost(), 7);
}

#[test]
fn empty_candidate_set_is_empty() {
    let set = RecursiveCounterfactualSet::new(Vec::new());

    assert!(set.is_empty());

    assert_eq!(set.len(), 0);
}

#[test]
fn distinct_transitions_are_preserved() {
    let first = RecursiveCounterfactualCandidate::new(transition(1, 2), 1).unwrap();

    let second = RecursiveCounterfactualCandidate::new(transition(1, 3), 1).unwrap();

    let set = RecursiveCounterfactualSet::new(vec![first, second]);

    assert_eq!(set.len(), 2);
}

#[test]
fn duplicate_transition_keeps_cheapest_interaction_cost() {
    let shared = transition(1, 2);

    let set = RecursiveCounterfactualSet::new(vec![
        RecursiveCounterfactualCandidate::new(shared.clone(), 9).unwrap(),
        RecursiveCounterfactualCandidate::new(shared.clone(), 2).unwrap(),
    ]);

    assert_eq!(set.len(), 1);

    assert_eq!(set.candidates()[0].interaction_cost(), 2);

    assert!(set.contains_transition(&shared,));
}

#[test]
fn candidate_set_is_deterministic_under_input_order() {
    let first = RecursiveCounterfactualCandidate::new(transition(1, 2), 3).unwrap();

    let second = RecursiveCounterfactualCandidate::new(transition(1, 4), 2).unwrap();

    let left = RecursiveCounterfactualSet::new(vec![first.clone(), second.clone()]);

    let right = RecursiveCounterfactualSet::new(vec![second, first]);

    assert_eq!(left, right);
}

#[test]
fn set_contains_exact_transition_identity() {
    let expected = transition(2, 5);

    let other = transition(2, 6);

    let set = RecursiveCounterfactualSet::new(vec![RecursiveCounterfactualCandidate::new(
        expected.clone(),
        1,
    )
    .unwrap()]);

    assert!(set.contains_transition(&expected,));

    assert!(!set.contains_transition(&other,));
}

#[test]
fn recursive_depth_identity_is_preserved() {
    let child = recursive_concept(1, 2);

    let deep = RecursiveConcept::new(vec![
        base(8),
        RecursiveUnit::Recursive(Box::new(child.clone())),
    ])
    .unwrap();

    let shallow_state = state(vec![RecursiveUnit::Recursive(Box::new(child))]);

    let deep_state = state(vec![RecursiveUnit::Recursive(Box::new(deep))]);

    let deep_transition = RecursivePlanningTransition::new(shallow_state, deep_state).unwrap();

    let set = RecursiveCounterfactualSet::new(vec![RecursiveCounterfactualCandidate::new(
        deep_transition.clone(),
        1,
    )
    .unwrap()]);

    assert!(set.contains_transition(&deep_transition,));
}

#[test]
fn set_construction_does_not_mutate_source_candidates() {
    let candidates = vec![
        RecursiveCounterfactualCandidate::new(transition(1, 2), 3).unwrap(),
        RecursiveCounterfactualCandidate::new(transition(1, 4), 2).unwrap(),
    ];

    let before = candidates.clone();

    let _ = RecursiveCounterfactualSet::new(candidates.clone());

    assert_eq!(candidates, before);
}

#[test]
fn candidate_and_set_clone_preserve_exact_value() {
    let candidate = RecursiveCounterfactualCandidate::new(transition(3, 7), 4).unwrap();

    let set = RecursiveCounterfactualSet::new(vec![candidate.clone()]);

    assert_eq!(candidate.clone(), candidate);

    assert_eq!(set.clone(), set);
}
