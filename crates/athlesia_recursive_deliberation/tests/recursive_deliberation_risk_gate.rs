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
    RecursiveDeliberationChoicePolicy, RecursiveDeliberationFoundation,
    RecursiveDeliberationRiskGate, RecursiveDeliberationRiskLimit, RecursiveDeliberationRiskStatus,
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

fn counterfactual_choice(
    cost: usize,
    outcomes: &[&[usize]],
) -> athlesia_recursive_deliberation::RecursiveDeliberationChoice {
    let request = RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        selection(cost, outcomes),
    );

    let bridge = RecursiveDeliberationFoundation::bridge(request);

    RecursiveDeliberationChoicePolicy::choose(bridge)
}

fn no_decision_choice() -> athlesia_recursive_deliberation::RecursiveDeliberationChoice {
    let projections = RecursiveCounterfactualProjectionSet::new(Vec::new());

    let budget = RecursiveCounterfactualBudget::new(10, 6, 15).unwrap();

    let policy = RecursiveCounterfactualSelectionPolicy::new(1).unwrap();

    let selected = RecursiveCounterfactualActiveCycle::evaluate(&projections, budget, policy)
        .selection()
        .clone();

    let request = RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        selected,
    );

    RecursiveDeliberationChoicePolicy::choose(RecursiveDeliberationFoundation::bridge(request))
}

#[test]
fn zero_interaction_risk_limit_is_rejected() {
    assert!(RecursiveDeliberationRiskLimit::new(0, 2,).is_none());
}

#[test]
fn zero_outcome_risk_limit_is_rejected() {
    assert!(RecursiveDeliberationRiskLimit::new(2, 0,).is_none());
}

#[test]
fn risk_limit_preserves_exact_bounds() {
    let limit = RecursiveDeliberationRiskLimit::new(3, 4).unwrap();

    assert_eq!(limit.max_interaction_cost(), 3);

    assert_eq!(limit.max_outcomes(), 4);
}

#[test]
fn candidate_on_exact_risk_boundary_is_allowed() {
    let choice = counterfactual_choice(2, &[&[1, 2], &[1, 3], &[1, 4]]);

    let value = choice.counterfactual().unwrap();

    let limit = RecursiveDeliberationRiskLimit::new(2, 3).unwrap();

    assert!(limit.allows(value,));
}

#[test]
fn interaction_cost_above_risk_limit_is_rejected() {
    let choice = counterfactual_choice(3, &[&[1, 2], &[1, 3]]);

    let assessment = RecursiveDeliberationRiskGate::assess(
        choice,
        RecursiveDeliberationRiskLimit::new(2, 3).unwrap(),
    );

    assert_eq!(
        assessment.status(),
        RecursiveDeliberationRiskStatus::Rejected
    );

    assert!(assessment.is_rejected());
}

#[test]
fn outcome_count_above_risk_limit_is_rejected() {
    let choice = counterfactual_choice(1, &[&[1, 2], &[1, 3], &[1, 4], &[1, 5]]);

    let assessment = RecursiveDeliberationRiskGate::assess(
        choice,
        RecursiveDeliberationRiskLimit::new(2, 3).unwrap(),
    );

    assert_eq!(
        assessment.status(),
        RecursiveDeliberationRiskStatus::Rejected
    );
}

#[test]
fn bounded_counterfactual_is_risk_eligible() {
    let choice = counterfactual_choice(1, &[&[1, 2], &[1, 3]]);

    let assessment = RecursiveDeliberationRiskGate::assess(
        choice,
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert!(assessment.is_eligible());

    assert_eq!(
        assessment.status(),
        RecursiveDeliberationRiskStatus::Eligible
    );
}

#[test]
fn no_decision_is_not_risk_applicable() {
    let assessment = RecursiveDeliberationRiskGate::assess(
        no_decision_choice(),
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert!(assessment.is_not_applicable());

    assert_eq!(
        assessment.status(),
        RecursiveDeliberationRiskStatus::NotApplicable
    );
}

#[test]
fn assessment_preserves_choice_identity() {
    let choice = counterfactual_choice(1, &[&[1, 2], &[1, 3]]);

    let expected = choice.clone();

    let assessment = RecursiveDeliberationRiskGate::assess(
        choice,
        RecursiveDeliberationRiskLimit::new(2, 2).unwrap(),
    );

    assert_eq!(assessment.choice(), &expected);
}

#[test]
fn assessment_preserves_counterfactual_identity() {
    let choice = counterfactual_choice(1, &[&[1, 2], &[1, 3], &[1, 4]]);

    let expected = choice.counterfactual().cloned();

    let assessment = RecursiveDeliberationRiskGate::assess(
        choice,
        RecursiveDeliberationRiskLimit::new(2, 3).unwrap(),
    );

    assert_eq!(assessment.counterfactual(), expected.as_ref());
}

#[test]
fn assessment_preserves_risk_limit_identity() {
    let choice = counterfactual_choice(1, &[&[1, 2], &[1, 3]]);

    let limit = RecursiveDeliberationRiskLimit::new(2, 3).unwrap();

    let assessment = RecursiveDeliberationRiskGate::assess(choice, limit);

    assert_eq!(assessment.limit(), limit);
}

#[test]
fn risk_gate_is_deterministic_and_non_mutating() {
    let choice = counterfactual_choice(1, &[&[1, 2], &[1, 3], &[1, 4]]);

    let before = choice.clone();

    let limit = RecursiveDeliberationRiskLimit::new(2, 3).unwrap();

    let first = RecursiveDeliberationRiskGate::assess(choice.clone(), limit);

    let second = RecursiveDeliberationRiskGate::assess(choice.clone(), limit);

    assert_eq!(first, second);

    assert_eq!(choice, before);
}
