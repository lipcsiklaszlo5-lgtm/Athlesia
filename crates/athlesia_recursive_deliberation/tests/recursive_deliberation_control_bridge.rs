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
    RecursiveDeliberationControlBridge, RecursiveDeliberationControlMode,
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

fn no_decision_request(beam_width: usize) -> RecursiveDeliberationRequest {
    RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        selection(beam_width),
    )
}

#[test]
fn no_decision_control_is_classified_exactly() {
    assert_eq!(
        RecursiveDeliberationControlBridge::classify(
            &RecursiveControlUncertaintyDecision::NoDecision,
        ),
        RecursiveDeliberationControlMode::NoDecision
    );
}

#[test]
fn control_modes_have_distinct_identity() {
    assert_ne!(
        RecursiveDeliberationControlMode::Act,
        RecursiveDeliberationControlMode::Experiment
    );

    assert_ne!(
        RecursiveDeliberationControlMode::Act,
        RecursiveDeliberationControlMode::NoDecision
    );

    assert_ne!(
        RecursiveDeliberationControlMode::Experiment,
        RecursiveDeliberationControlMode::NoDecision
    );
}

#[test]
fn bridge_preserves_request_identity() {
    let request = no_decision_request(2);

    let expected = request.clone();

    let bridge = RecursiveDeliberationControlBridge::new(request);

    assert_eq!(bridge.request(), &expected);
}

#[test]
fn bridge_preserves_control_identity() {
    let bridge = RecursiveDeliberationControlBridge::new(no_decision_request(2));

    assert_eq!(
        bridge.control(),
        &RecursiveControlUncertaintyDecision::NoDecision
    );
}

#[test]
fn bridge_preserves_counterfactual_frontier() {
    let request = no_decision_request(2);

    let expected = request.counterfactual().clone();

    let bridge = RecursiveDeliberationControlBridge::new(request);

    assert_eq!(bridge.counterfactual(), &expected);
}

#[test]
fn bridge_preserves_best_counterfactual() {
    let request = no_decision_request(2);

    let expected = request.best_counterfactual().cloned();

    let bridge = RecursiveDeliberationControlBridge::new(request);

    assert_eq!(bridge.best_counterfactual(), expected.as_ref());
}

#[test]
fn no_decision_bridge_is_undecided() {
    let bridge = RecursiveDeliberationControlBridge::new(no_decision_request(2));

    assert!(bridge.is_undecided());

    assert!(!bridge.has_action());

    assert!(!bridge.has_experiment());
}

#[test]
fn empty_frontier_survives_control_bridge() {
    let request = RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        empty_selection(),
    );

    let bridge = RecursiveDeliberationControlBridge::new(request);

    assert!(bridge.counterfactual().is_empty());

    assert!(bridge.best_counterfactual().is_none());
}

#[test]
fn bridge_preserves_beam_width() {
    let bridge = RecursiveDeliberationControlBridge::new(no_decision_request(1));

    assert_eq!(bridge.counterfactual().policy().beam_width(), 1);

    assert_eq!(bridge.counterfactual().len(), 1);
}

#[test]
fn bridge_preserves_budget_identity() {
    let request = no_decision_request(2);

    let expected = request.counterfactual().budget();

    let bridge = RecursiveDeliberationControlBridge::new(request);

    assert_eq!(bridge.counterfactual().budget(), expected);
}

#[test]
fn foundation_bridge_matches_direct_bridge() {
    let request = no_decision_request(2);

    let direct = RecursiveDeliberationControlBridge::new(request.clone());

    let via_foundation = RecursiveDeliberationFoundation::bridge(request);

    assert_eq!(via_foundation, direct);
}

#[test]
fn control_bridge_is_deterministic_and_non_mutating() {
    let request = no_decision_request(2);

    let before = request.clone();

    let first = RecursiveDeliberationControlBridge::new(request.clone());

    let second = RecursiveDeliberationControlBridge::new(request.clone());

    assert_eq!(first, second);

    assert_eq!(request, before);
}
