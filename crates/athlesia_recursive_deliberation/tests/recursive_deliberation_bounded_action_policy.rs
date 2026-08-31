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
    RecursiveDeliberationActionKind, RecursiveDeliberationBoundedActionPolicy,
    RecursiveDeliberationChoiceKind, RecursiveDeliberationChoicePolicy,
    RecursiveDeliberationFoundation, RecursiveDeliberationRiskGate, RecursiveDeliberationRiskLimit,
    RecursiveDeliberationRiskStatus,
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

fn assessment(
    cost: usize,
    outcomes: &[&[usize]],
    max_cost: usize,
    max_outcomes: usize,
) -> athlesia_recursive_deliberation::RecursiveDeliberationRiskAssessment {
    let request = RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        selection(cost, outcomes),
    );

    let bridge = RecursiveDeliberationFoundation::bridge(request);

    let choice = RecursiveDeliberationChoicePolicy::choose(bridge);

    RecursiveDeliberationRiskGate::assess(
        choice,
        RecursiveDeliberationRiskLimit::new(max_cost, max_outcomes).unwrap(),
    )
}

fn no_decision_assessment() -> athlesia_recursive_deliberation::RecursiveDeliberationRiskAssessment
{
    let request = RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        empty_selection(),
    );

    let bridge = RecursiveDeliberationFoundation::bridge(request);

    let choice = RecursiveDeliberationChoicePolicy::choose(bridge);

    RecursiveDeliberationRiskGate::assess(
        choice,
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    )
}

#[test]
fn act_choice_is_preserved_as_act() {
    assert_eq!(
        RecursiveDeliberationBoundedActionPolicy::resolve_kind(
            RecursiveDeliberationChoiceKind::Act,
            RecursiveDeliberationRiskStatus::NotApplicable,
        ),
        RecursiveDeliberationActionKind::Act
    );
}

#[test]
fn experiment_choice_is_preserved_as_experiment() {
    assert_eq!(
        RecursiveDeliberationBoundedActionPolicy::resolve_kind(
            RecursiveDeliberationChoiceKind::Experiment,
            RecursiveDeliberationRiskStatus::NotApplicable,
        ),
        RecursiveDeliberationActionKind::Experiment
    );
}

#[test]
fn eligible_counterfactual_becomes_bounded_action() {
    assert_eq!(
        RecursiveDeliberationBoundedActionPolicy::resolve_kind(
            RecursiveDeliberationChoiceKind::Counterfactual,
            RecursiveDeliberationRiskStatus::Eligible,
        ),
        RecursiveDeliberationActionKind::BoundedAction
    );
}

#[test]
fn rejected_counterfactual_becomes_no_decision() {
    assert_eq!(
        RecursiveDeliberationBoundedActionPolicy::resolve_kind(
            RecursiveDeliberationChoiceKind::Counterfactual,
            RecursiveDeliberationRiskStatus::Rejected,
        ),
        RecursiveDeliberationActionKind::NoDecision
    );
}

#[test]
fn no_decision_remains_no_decision() {
    assert_eq!(
        RecursiveDeliberationBoundedActionPolicy::resolve_kind(
            RecursiveDeliberationChoiceKind::NoDecision,
            RecursiveDeliberationRiskStatus::NotApplicable,
        ),
        RecursiveDeliberationActionKind::NoDecision
    );
}

#[test]
fn eligible_counterfactual_produces_bounded_action_decision() {
    let decision =
        RecursiveDeliberationBoundedActionPolicy::decide(assessment(1, &[&[1, 2], &[1, 3]], 2, 2));

    assert!(decision.is_bounded_action());

    assert_eq!(
        decision.kind(),
        RecursiveDeliberationActionKind::BoundedAction
    );
}

#[test]
fn over_cost_counterfactual_does_not_act() {
    let decision =
        RecursiveDeliberationBoundedActionPolicy::decide(assessment(3, &[&[1, 2], &[1, 3]], 2, 2));

    assert!(decision.is_no_decision());

    assert!(!decision.is_bounded_action());
}

#[test]
fn over_outcome_counterfactual_does_not_act() {
    let decision = RecursiveDeliberationBoundedActionPolicy::decide(assessment(
        1,
        &[&[1, 2], &[1, 3], &[1, 4]],
        2,
        2,
    ));

    assert!(decision.is_no_decision());
}

#[test]
fn bounded_action_preserves_counterfactual_identity() {
    let source = assessment(1, &[&[1, 2], &[1, 3]], 2, 2);

    let expected = source.counterfactual().cloned();

    let decision = RecursiveDeliberationBoundedActionPolicy::decide(source);

    assert_eq!(decision.counterfactual(), expected.as_ref());
}

#[test]
fn bounded_action_preserves_risk_assessment_identity() {
    let source = assessment(1, &[&[1, 2], &[1, 3]], 2, 2);

    let expected = source.clone();

    let decision = RecursiveDeliberationBoundedActionPolicy::decide(source);

    assert_eq!(decision.assessment(), &expected);
}

#[test]
fn empty_frontier_cannot_produce_bounded_action() {
    let decision = RecursiveDeliberationBoundedActionPolicy::decide(no_decision_assessment());

    assert!(decision.is_no_decision());

    assert!(decision.counterfactual().is_none());
}

#[test]
fn bounded_action_policy_is_deterministic_and_non_mutating() {
    let source = assessment(1, &[&[1, 2], &[1, 3], &[1, 4]], 2, 3);

    let before = source.clone();

    let first = RecursiveDeliberationBoundedActionPolicy::decide(source.clone());

    let second = RecursiveDeliberationBoundedActionPolicy::decide(source.clone());

    assert_eq!(first, second);

    assert_eq!(source, before);
}
