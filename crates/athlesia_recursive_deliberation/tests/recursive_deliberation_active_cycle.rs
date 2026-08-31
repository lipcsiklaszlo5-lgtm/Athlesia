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
    RecursiveDeliberationActionKind, RecursiveDeliberationActiveCycle,
    RecursiveDeliberationFoundation, RecursiveDeliberationRiskLimit,
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

fn selection(cost: usize, outcomes: &[&[usize]]) -> RecursiveCounterfactualSelection {
    let projections =
        RecursiveCounterfactualProjectionSet::new(vec![projection(2, cost, outcomes)]);

    let budget = RecursiveCounterfactualBudget::new(10, 6, 15).unwrap();

    let policy = RecursiveCounterfactualSelectionPolicy::new(1).unwrap();

    RecursiveCounterfactualActiveCycle::evaluate(&projections, budget, policy)
        .selection()
        .clone()
}

fn empty_selection() -> RecursiveCounterfactualSelection {
    let projections = RecursiveCounterfactualProjectionSet::new(Vec::new());

    let budget = RecursiveCounterfactualBudget::new(10, 6, 15).unwrap();

    let policy = RecursiveCounterfactualSelectionPolicy::new(1).unwrap();

    RecursiveCounterfactualActiveCycle::evaluate(&projections, budget, policy)
        .selection()
        .clone()
}

fn request(
    selected: RecursiveCounterfactualSelection,
) -> athlesia_recursive_deliberation::RecursiveDeliberationRequest {
    RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        selected,
    )
}

#[test]
fn active_cycle_preserves_request_identity() {
    let source = request(selection(1, &[&[1, 2], &[1, 3]]));

    let expected = source.clone();

    let result = RecursiveDeliberationActiveCycle::evaluate(
        source,
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert_eq!(result.request(), &expected);
}

#[test]
fn active_cycle_builds_matching_control_bridge() {
    let result = RecursiveDeliberationActiveCycle::evaluate(
        request(selection(1, &[&[1, 2], &[1, 3]])),
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert_eq!(result.bridge().request(), result.request());
}

#[test]
fn active_cycle_builds_counterfactual_choice_from_no_decision() {
    let result = RecursiveDeliberationActiveCycle::evaluate(
        request(selection(1, &[&[1, 2], &[1, 3]])),
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert!(result.choice().is_counterfactual());
}

#[test]
fn eligible_counterfactual_becomes_bounded_action() {
    let result = RecursiveDeliberationActiveCycle::evaluate(
        request(selection(1, &[&[1, 2], &[1, 3]])),
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert_eq!(
        result.final_kind(),
        RecursiveDeliberationActionKind::BoundedAction
    );

    assert!(result.decision().is_bounded_action());
}

#[test]
fn excessive_interaction_cost_blocks_bounded_action() {
    let result = RecursiveDeliberationActiveCycle::evaluate(
        request(selection(4, &[&[1, 2], &[1, 3]])),
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert_eq!(
        result.final_kind(),
        RecursiveDeliberationActionKind::NoDecision
    );
}

#[test]
fn excessive_outcome_count_blocks_bounded_action() {
    let result = RecursiveDeliberationActiveCycle::evaluate(
        request(selection(1, &[&[1, 2], &[1, 3], &[1, 4]])),
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert_eq!(
        result.final_kind(),
        RecursiveDeliberationActionKind::NoDecision
    );
}

#[test]
fn deterministic_projection_does_not_become_bounded_action() {
    let result = RecursiveDeliberationActiveCycle::evaluate(
        request(selection(1, &[&[1, 2]])),
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert_eq!(
        result.final_kind(),
        RecursiveDeliberationActionKind::NoDecision
    );

    assert!(result.counterfactual().is_none());
}

#[test]
fn empty_frontier_remains_no_decision() {
    let result = RecursiveDeliberationActiveCycle::evaluate(
        request(empty_selection()),
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert_eq!(
        result.final_kind(),
        RecursiveDeliberationActionKind::NoDecision
    );
}

#[test]
fn active_cycle_preserves_best_counterfactual_identity() {
    let source = request(selection(1, &[&[1, 2], &[1, 3], &[1, 4]]));

    let expected = source.best_counterfactual().cloned();

    let result = RecursiveDeliberationActiveCycle::evaluate(
        source,
        RecursiveDeliberationRiskLimit::new(2, 3).unwrap(),
    );

    assert_eq!(result.counterfactual(), expected.as_ref());
}

#[test]
fn active_cycle_preserves_risk_limit_identity() {
    let limit = RecursiveDeliberationRiskLimit::new(2, 3).unwrap();

    let result = RecursiveDeliberationActiveCycle::evaluate(
        request(selection(1, &[&[1, 2], &[1, 3]])),
        limit,
    );

    assert_eq!(result.assessment().limit(), limit);
}

#[test]
fn active_cycle_is_deterministic() {
    let source = request(selection(1, &[&[1, 2], &[1, 3], &[1, 4]]));

    let limit = RecursiveDeliberationRiskLimit::new(2, 3).unwrap();

    let first = RecursiveDeliberationActiveCycle::evaluate(source.clone(), limit);

    let second = RecursiveDeliberationActiveCycle::evaluate(source, limit);

    assert_eq!(first, second);
}

#[test]
fn active_cycle_does_not_mutate_source_request() {
    let source = request(selection(1, &[&[1, 2], &[1, 3]]));

    let before = source.clone();

    let _ = RecursiveDeliberationActiveCycle::evaluate(
        source.clone(),
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert_eq!(source, before);
}
