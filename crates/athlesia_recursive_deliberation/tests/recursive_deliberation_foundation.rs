use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_control::RecursiveControlUncertaintyDecision;

use athlesia_recursive_counterfactual::{
    RecursiveCounterfactualActiveCycle, RecursiveCounterfactualBudget,
    RecursiveCounterfactualCandidate, RecursiveCounterfactualOutcome,
    RecursiveCounterfactualProjection, RecursiveCounterfactualProjectionSet,
    RecursiveCounterfactualSelection, RecursiveCounterfactualSelectionPolicy,
};

use athlesia_recursive_deliberation::{
    RecursiveDeliberationFoundation, RecursiveDeliberationRequest,
};

use athlesia_recursive_planning::{RecursivePlanningState, RecursivePlanningTransition};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
    )
}

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(structural(span)))
}

fn state(spans: &[usize]) -> RecursivePlanningState {
    RecursivePlanningState::new(spans.iter().copied().map(unit).collect())
}

fn projection(
    target: usize,
    cost: usize,
    outcomes: &[&[usize]],
) -> RecursiveCounterfactualProjection {
    let transition = RecursivePlanningTransition::new(state(&[1]), state(&[1, target])).unwrap();

    let candidate = RecursiveCounterfactualCandidate::new(transition, cost).unwrap();

    RecursiveCounterfactualProjection::new(
        candidate,
        outcomes
            .iter()
            .map(|spans| RecursiveCounterfactualOutcome::new(state(spans)))
            .collect(),
    )
    .unwrap()
}

fn selection(beam_width: usize) -> RecursiveCounterfactualSelection {
    let projections = RecursiveCounterfactualProjectionSet::new(vec![
        projection(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]),
        projection(5, 2, &[&[1, 5], &[1, 6]]),
        projection(8, 3, &[&[1, 8], &[1, 9]]),
    ]);

    let budget = RecursiveCounterfactualBudget::new(5, 4, 6).unwrap();

    let policy = RecursiveCounterfactualSelectionPolicy::new(beam_width).unwrap();

    RecursiveCounterfactualActiveCycle::evaluate(&projections, budget, policy)
        .selection()
        .clone()
}

fn empty_selection() -> RecursiveCounterfactualSelection {
    let projections = RecursiveCounterfactualProjectionSet::new(Vec::new());

    let budget = RecursiveCounterfactualBudget::new(5, 4, 6).unwrap();

    let policy = RecursiveCounterfactualSelectionPolicy::new(2).unwrap();

    RecursiveCounterfactualActiveCycle::evaluate(&projections, budget, policy)
        .selection()
        .clone()
}

#[test]
fn request_preserves_control_identity() {
    let request = RecursiveDeliberationRequest::new(
        RecursiveControlUncertaintyDecision::NoDecision,
        selection(2),
    );

    assert_eq!(
        request.control(),
        &RecursiveControlUncertaintyDecision::NoDecision
    );
}

#[test]
fn request_preserves_counterfactual_selection() {
    let expected = selection(2);

    let request = RecursiveDeliberationRequest::new(
        RecursiveControlUncertaintyDecision::NoDecision,
        expected.clone(),
    );

    assert_eq!(request.counterfactual(), &expected);
}

#[test]
fn request_reports_counterfactual_frontier_length() {
    let request = RecursiveDeliberationRequest::new(
        RecursiveControlUncertaintyDecision::NoDecision,
        selection(2),
    );

    assert_eq!(request.counterfactual_len(), 2);
}

#[test]
fn non_empty_frontier_is_detected() {
    let request = RecursiveDeliberationRequest::new(
        RecursiveControlUncertaintyDecision::NoDecision,
        selection(2),
    );

    assert!(request.has_counterfactual_frontier());
}

#[test]
fn empty_frontier_is_detected() {
    let request = RecursiveDeliberationRequest::new(
        RecursiveControlUncertaintyDecision::NoDecision,
        empty_selection(),
    );

    assert!(!request.has_counterfactual_frontier());

    assert_eq!(request.counterfactual_len(), 0);
}

#[test]
fn best_counterfactual_matches_selection_head() {
    let selected = selection(2);

    let expected = selected.best().cloned();

    let request = RecursiveDeliberationRequest::new(
        RecursiveControlUncertaintyDecision::NoDecision,
        selected,
    );

    assert_eq!(request.best_counterfactual(), expected.as_ref());
}

#[test]
fn empty_frontier_has_no_best_counterfactual() {
    let request = RecursiveDeliberationRequest::new(
        RecursiveControlUncertaintyDecision::NoDecision,
        empty_selection(),
    );

    assert!(request.best_counterfactual().is_none());
}

#[test]
fn foundation_prepare_matches_direct_request_construction() {
    let control = RecursiveControlUncertaintyDecision::NoDecision;

    let selected = selection(2);

    let direct = RecursiveDeliberationRequest::new(control.clone(), selected.clone());

    let prepared = RecursiveDeliberationFoundation::prepare(control, selected);

    assert_eq!(prepared, direct);
}

#[test]
fn foundation_prepare_preserves_beam_bound() {
    let request = RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        selection(1),
    );

    assert_eq!(request.counterfactual_len(), 1);

    assert_eq!(request.counterfactual().policy().beam_width(), 1);
}

#[test]
fn foundation_prepare_preserves_budget_identity() {
    let selected = selection(2);

    let expected_budget = selected.budget();

    let request = RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        selected,
    );

    assert_eq!(request.counterfactual().budget(), expected_budget);
}

#[test]
fn deliberation_request_clone_preserves_exact_value() {
    let request = RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        selection(2),
    );

    assert_eq!(request.clone(), request);
}

#[test]
fn foundation_prepare_is_deterministic_and_non_mutating() {
    let control = RecursiveControlUncertaintyDecision::NoDecision;

    let selected = selection(2);

    let control_before = control.clone();

    let selection_before = selected.clone();

    let first = RecursiveDeliberationFoundation::prepare(control.clone(), selected.clone());

    let second = RecursiveDeliberationFoundation::prepare(control.clone(), selected.clone());

    assert_eq!(first, second);

    assert_eq!(control, control_before);

    assert_eq!(selected, selection_before);
}
