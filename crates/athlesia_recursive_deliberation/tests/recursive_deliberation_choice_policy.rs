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
    RecursiveDeliberationChoiceKind, RecursiveDeliberationChoicePolicy,
    RecursiveDeliberationControlMode, RecursiveDeliberationFoundation,
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

fn selection(informative: bool) -> RecursiveCounterfactualSelection {
    let projections = if informative {
        RecursiveCounterfactualProjectionSet::new(vec![
            projection(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]),
            projection(5, 2, &[&[1, 5], &[1, 6]]),
        ])
    } else {
        RecursiveCounterfactualProjectionSet::new(vec![projection(2, 1, &[&[1, 2]])])
    };

    let budget = RecursiveCounterfactualBudget::new(5, 4, 6).unwrap();

    let policy = RecursiveCounterfactualSelectionPolicy::new(2).unwrap();

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

fn bridge(
    selected: RecursiveCounterfactualSelection,
) -> athlesia_recursive_deliberation::RecursiveDeliberationControlBridge {
    let request = RecursiveDeliberationFoundation::prepare(
        RecursiveControlUncertaintyDecision::NoDecision,
        selected,
    );

    RecursiveDeliberationFoundation::bridge(request)
}

#[test]
fn act_mode_has_absolute_priority() {
    assert_eq!(
        RecursiveDeliberationChoicePolicy::resolve_kind(
            RecursiveDeliberationControlMode::Act,
            true,
        ),
        RecursiveDeliberationChoiceKind::Act
    );

    assert_eq!(
        RecursiveDeliberationChoicePolicy::resolve_kind(
            RecursiveDeliberationControlMode::Act,
            false,
        ),
        RecursiveDeliberationChoiceKind::Act
    );
}

#[test]
fn experiment_mode_has_absolute_priority() {
    assert_eq!(
        RecursiveDeliberationChoicePolicy::resolve_kind(
            RecursiveDeliberationControlMode::Experiment,
            true,
        ),
        RecursiveDeliberationChoiceKind::Experiment
    );

    assert_eq!(
        RecursiveDeliberationChoicePolicy::resolve_kind(
            RecursiveDeliberationControlMode::Experiment,
            false,
        ),
        RecursiveDeliberationChoiceKind::Experiment
    );
}

#[test]
fn no_decision_with_information_selects_counterfactual() {
    assert_eq!(
        RecursiveDeliberationChoicePolicy::resolve_kind(
            RecursiveDeliberationControlMode::NoDecision,
            true,
        ),
        RecursiveDeliberationChoiceKind::Counterfactual
    );
}

#[test]
fn no_decision_without_information_stays_no_decision() {
    assert_eq!(
        RecursiveDeliberationChoicePolicy::resolve_kind(
            RecursiveDeliberationControlMode::NoDecision,
            false,
        ),
        RecursiveDeliberationChoiceKind::NoDecision
    );
}

#[test]
fn informative_frontier_produces_counterfactual_choice() {
    let choice = RecursiveDeliberationChoicePolicy::choose(bridge(selection(true)));

    assert!(choice.is_counterfactual());

    assert_eq!(
        choice.kind(),
        RecursiveDeliberationChoiceKind::Counterfactual
    );
}

#[test]
fn deterministic_frontier_does_not_fake_information() {
    let choice = RecursiveDeliberationChoicePolicy::choose(bridge(selection(false)));

    assert!(choice.is_no_decision());

    assert!(choice.counterfactual().is_none());
}

#[test]
fn empty_frontier_stays_no_decision() {
    let choice = RecursiveDeliberationChoicePolicy::choose(bridge(empty_selection()));

    assert!(choice.is_no_decision());

    assert!(choice.counterfactual().is_none());
}

#[test]
fn counterfactual_choice_preserves_best_identity() {
    let source = bridge(selection(true));

    let expected = source.best_counterfactual().cloned();

    let choice = RecursiveDeliberationChoicePolicy::choose(source);

    assert_eq!(choice.counterfactual(), expected.as_ref());
}

#[test]
fn choice_preserves_control_bridge_identity() {
    let source = bridge(selection(true));

    let expected = source.clone();

    let choice = RecursiveDeliberationChoicePolicy::choose(source);

    assert_eq!(choice.bridge(), &expected);
}

#[test]
fn choice_preserves_original_control_identity() {
    let choice = RecursiveDeliberationChoicePolicy::choose(bridge(selection(true)));

    assert_eq!(
        choice.control(),
        &RecursiveControlUncertaintyDecision::NoDecision
    );
}

#[test]
fn choice_preserves_budget_and_beam_context() {
    let choice = RecursiveDeliberationChoicePolicy::choose(bridge(selection(true)));

    assert_eq!(choice.bridge().counterfactual().policy().beam_width(), 2);

    assert_eq!(
        choice
            .bridge()
            .counterfactual()
            .budget()
            .max_interaction_cost(),
        5
    );
}

#[test]
fn choice_policy_is_deterministic_and_non_mutating() {
    let source = bridge(selection(true));

    let before = source.clone();

    let first = RecursiveDeliberationChoicePolicy::choose(source.clone());

    let second = RecursiveDeliberationChoicePolicy::choose(source.clone());

    assert_eq!(first, second);

    assert_eq!(source, before);
}
