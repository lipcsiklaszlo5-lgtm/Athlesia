use athlesia_autonomous_active_experimentation::{
    ActiveExperimentBounds, ActiveExperimentPolicy, ActiveExperimentThresholds,
    AutonomousActiveExperimentationFoundation, AutonomousExperimentProposal, ExperimentEvidence,
    UniversalAutonomousActiveExperimentationFoundation,
};

use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn s(value: u16) -> CognitiveSignal {
    if value == 0 {
        CognitiveSignal::zero()
    } else {
        CognitiveSignal::new(value).unwrap()
    }
}

fn a(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

#[derive(Clone, Copy)]
struct Spec {
    state: u64,
    action: u64,
    outcome: u64,
    uncertainty: u16,
    information: u16,
    control: u16,
    grounding: u16,
    cost: u16,
}

fn proposal(spec: Spec) -> AutonomousExperimentProposal {
    AutonomousExperimentProposal::new(
        a(spec.state),
        a(spec.action),
        a(spec.outcome),
        ExperimentEvidence::new(
            s(spec.uncertainty),
            s(spec.information),
            s(spec.control),
            s(spec.grounding),
            s(spec.cost),
        )
        .unwrap(),
    )
}

fn good(action: u64, outcome: u64) -> AutonomousExperimentProposal {
    proposal(Spec {
        state: 1,
        action,
        outcome,
        uncertainty: 800,
        information: 800,
        control: 800,
        grounding: 900,
        cost: 200,
    })
}

fn policy() -> ActiveExperimentPolicy {
    ActiveExperimentPolicy::new(
        ActiveExperimentBounds::new(32, 32, 32).unwrap(),
        ActiveExperimentThresholds::new(s(500), s(500), s(500), s(500)).unwrap(),
    )
}

#[test]
fn foundation_requires_positive_bounds_and_epistemic_evidence() {
    assert_eq!(ActiveExperimentBounds::new(0, 1, 1), None);

    assert_eq!(ExperimentEvidence::new(s(0), s(1), s(1), s(1), s(1),), None);

    assert_eq!(
        ActiveExperimentThresholds::new(s(0), s(1), s(1), s(1),),
        None
    );

    assert!(ActiveExperimentBounds::new(1, 1, 1).is_some());
}

#[test]
fn grounded_informative_controllable_low_cost_experiment_is_selected() {
    let p = good(10, 20);

    let result =
        AutonomousActiveExperimentationFoundation::select(std::slice::from_ref(&p), policy());

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].proposal(), &p);
}

#[test]
fn high_uncertainty_without_information_gain_does_not_justify_experiment() {
    let p = proposal(Spec {
        state: 1,
        action: 10,
        outcome: 20,
        uncertainty: 1000,
        information: 400,
        control: 900,
        grounding: 900,
        cost: 100,
    });

    let result = AutonomousActiveExperimentationFoundation::select(&[p], policy());

    assert!(result.abstained());

    assert_eq!(result.rejected_information_gain_count(), 1);
}

#[test]
fn weak_controllability_blocks_active_intervention() {
    let p = proposal(Spec {
        state: 1,
        action: 10,
        outcome: 20,
        uncertainty: 900,
        information: 900,
        control: 400,
        grounding: 900,
        cost: 100,
    });

    let result = AutonomousActiveExperimentationFoundation::select(&[p], policy());

    assert!(result.abstained());

    assert_eq!(result.rejected_controllability_count(), 1);
}

#[test]
fn weak_grounding_blocks_high_information_experiment() {
    let p = proposal(Spec {
        state: 1,
        action: 10,
        outcome: 20,
        uncertainty: 1000,
        information: 1000,
        control: 1000,
        grounding: 400,
        cost: 100,
    });

    let result = AutonomousActiveExperimentationFoundation::select(&[p], policy());

    assert!(result.abstained());

    assert_eq!(result.rejected_grounding_count(), 1);
}

#[test]
fn excessive_execution_cost_blocks_experiment() {
    let p = proposal(Spec {
        state: 1,
        action: 10,
        outcome: 20,
        uncertainty: 900,
        information: 900,
        control: 900,
        grounding: 900,
        cost: 600,
    });

    let result = AutonomousActiveExperimentationFoundation::select(&[p], policy());

    assert!(result.abstained());

    assert_eq!(result.rejected_cost_count(), 1);
}

#[test]
fn exact_semantic_duplicate_keeps_stronger_evidence_once() {
    let weak = proposal(Spec {
        state: 1,
        action: 10,
        outcome: 20,
        uncertainty: 600,
        information: 600,
        control: 600,
        grounding: 600,
        cost: 300,
    });

    let strong = proposal(Spec {
        state: 1,
        action: 10,
        outcome: 20,
        uncertainty: 900,
        information: 900,
        control: 900,
        grounding: 900,
        cost: 100,
    });

    let result =
        AutonomousActiveExperimentationFoundation::select(&[weak, strong.clone()], policy());

    assert_eq!(result.input_proposal_count(), 2);
    assert_eq!(result.unique_proposal_count(), 1);
    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].proposal(), &strong);
}

#[test]
fn same_action_with_distinct_predicted_outcomes_remains_distinct() {
    let one = good(10, 20);
    let two = good(10, 21);

    let result = AutonomousActiveExperimentationFoundation::select(&[one, two], policy());

    assert_eq!(result.unique_proposal_count(), 2);
    assert_eq!(result.selected_count(), 2);
}

#[test]
fn distinct_actions_with_same_outcome_remain_distinct() {
    let one = good(10, 20);
    let two = good(11, 20);

    let result = AutonomousActiveExperimentationFoundation::select(&[one, two], policy());

    assert_eq!(result.unique_proposal_count(), 2);
    assert_eq!(result.selected_count(), 2);
}

#[test]
fn information_gain_ranks_before_raw_uncertainty() {
    let uncertainty = proposal(Spec {
        state: 1,
        action: 10,
        outcome: 20,
        uncertainty: 1000,
        information: 700,
        control: 900,
        grounding: 900,
        cost: 100,
    });

    let information = proposal(Spec {
        state: 1,
        action: 11,
        outcome: 21,
        uncertainty: 700,
        information: 900,
        control: 900,
        grounding: 900,
        cost: 100,
    });

    let result = AutonomousActiveExperimentationFoundation::select(
        &[uncertainty, information.clone()],
        policy(),
    );

    assert_eq!(result.selected()[0].proposal(), &information);
}

#[test]
fn hard_input_evaluation_and_selection_frontiers_are_enforced() {
    let items = vec![good(10, 20), good(11, 21), good(12, 22)];

    let input_policy = ActiveExperimentPolicy::new(
        ActiveExperimentBounds::new(1, 32, 32).unwrap(),
        policy().thresholds(),
    );

    let input = AutonomousActiveExperimentationFoundation::select(&items, input_policy);

    assert_eq!(input.unique_proposal_count(), 3);
    assert_eq!(input.considered_proposal_count(), 1);
    assert!(input.input_frontier_truncated());

    let eval_policy = ActiveExperimentPolicy::new(
        ActiveExperimentBounds::new(32, 1, 32).unwrap(),
        policy().thresholds(),
    );

    let eval = AutonomousActiveExperimentationFoundation::select(&items, eval_policy);

    assert_eq!(eval.evaluation_count(), 1);
    assert!(eval.evaluation_frontier_truncated());

    let selection_policy = ActiveExperimentPolicy::new(
        ActiveExperimentBounds::new(32, 32, 1).unwrap(),
        policy().thresholds(),
    );

    let selected = AutonomousActiveExperimentationFoundation::select(&items, selection_policy);

    assert_eq!(selected.selected_before_frontier(), 3);
    assert_eq!(selected.selected_count(), 1);
    assert!(selected.selection_frontier_truncated());
}

#[test]
fn foundation_is_deterministic_non_mutating_and_facade_equivalent() {
    let items = vec![good(10, 20), good(11, 21), good(12, 22)];

    let before = items.clone();

    let mut reversed = items.clone();
    reversed.reverse();

    let p = policy();

    let direct = AutonomousActiveExperimentationFoundation::select(&items, p);

    let reordered = AutonomousActiveExperimentationFoundation::select(&reversed, p);

    let facade = UniversalAutonomousActiveExperimentationFoundation::evaluate(&items, p);

    let repeated = UniversalAutonomousActiveExperimentationFoundation::evaluate(&items, p);

    assert_eq!(direct, reordered);
    assert_eq!(direct, facade);
    assert_eq!(facade, repeated);
    assert_eq!(items, before);
}
