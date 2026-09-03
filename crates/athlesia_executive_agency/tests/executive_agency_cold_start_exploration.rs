use athlesia_executive_agency::{
    ColdStartExplorationController, ColdStartExplorationPolicy, ColdStartExplorationStatus,
    ExecutiveGoal, ExplorationSignals, GroundedExplorationCandidate,
    UniversalColdStartExplorationController,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn goal(identity: u64, satisfaction: u16) -> ExecutiveGoal {
    ExecutiveGoal::new(atom(identity), signal(1000), signal(satisfaction))
}

fn signals(
    information_gain: u16,
    learning_progress: u16,
    controllability: u16,
    evidence_confidence: u16,
    execution_cost: u16,
) -> ExplorationSignals {
    ExplorationSignals::new(
        signal(information_gain),
        signal(learning_progress),
        signal(controllability),
        signal(evidence_confidence),
        signal(execution_cost),
    )
}

fn candidate(
    goal_identity: u64,
    action: u64,
    outcome: u64,
    information_gain: u16,
    controllability: u16,
    evidence_confidence: u16,
    execution_cost: u16,
) -> GroundedExplorationCandidate {
    GroundedExplorationCandidate::new(
        atom(goal_identity),
        atom(action),
        atom(outcome),
        signals(
            information_gain,
            0,
            controllability,
            evidence_confidence,
            execution_cost,
        ),
    )
}

fn policy() -> ColdStartExplorationPolicy {
    ColdStartExplorationPolicy::new(16, 16, signal(500), signal(500), signal(500), signal(1))
        .unwrap()
}

#[test]
fn cold_start_policy_requires_positive_bounds_and_grounding_thresholds() {
    assert_eq!(
        ColdStartExplorationPolicy::new(0, 1, signal(1), signal(1), signal(1), signal(1),),
        None,
    );

    assert_eq!(
        ColdStartExplorationPolicy::new(1, 0, signal(1), signal(1), signal(1), signal(1),),
        None,
    );

    assert_eq!(
        ColdStartExplorationPolicy::new(1, 1, signal(0), signal(1), signal(1), signal(1),),
        None,
    );
}

#[test]
fn satisfied_goal_blocks_cold_start_before_candidate_evaluation() {
    let result = ColdStartExplorationController::evaluate(
        &goal(1, 1000),
        &[candidate(1, 10, 100, 900, 900, 900, 0)],
        policy(),
    );

    assert_eq!(result.status(), ColdStartExplorationStatus::GoalSatisfied,);

    assert_eq!(result.candidate_evaluation_count(), 0,);

    assert_eq!(result.selected_exploration(), None,);
}

#[test]
fn cold_start_requires_exact_goal_identity() {
    let result = ColdStartExplorationController::evaluate(
        &goal(1, 0),
        &[candidate(2, 10, 100, 900, 900, 900, 0)],
        policy(),
    );

    assert_eq!(result.rejected_goal_mismatch_count(), 1,);

    assert!(!result.selected());
}

#[test]
fn weak_information_gain_cannot_start_exploration() {
    let result = ColdStartExplorationController::evaluate(
        &goal(1, 0),
        &[candidate(1, 10, 100, 499, 900, 900, 0)],
        policy(),
    );

    assert_eq!(result.rejected_threshold_count(), 1,);

    assert!(!result.selected());
}

#[test]
fn weak_evidence_confidence_cannot_start_exploration() {
    let result = ColdStartExplorationController::evaluate(
        &goal(1, 0),
        &[candidate(1, 10, 100, 900, 900, 499, 0)],
        policy(),
    );

    assert_eq!(result.rejected_threshold_count(), 1,);

    assert!(!result.selected());
}

#[test]
fn weak_controllability_cannot_start_exploration() {
    let result = ColdStartExplorationController::evaluate(
        &goal(1, 0),
        &[candidate(1, 10, 100, 900, 499, 900, 0)],
        policy(),
    );

    assert_eq!(result.rejected_threshold_count(), 1,);

    assert!(!result.selected());
}

#[test]
fn cold_start_does_not_fabricate_prior_learning_progress() {
    let candidate = candidate(1, 10, 100, 800, 900, 900, 100);

    assert_eq!(candidate.signals().learning_progress(), signal(0),);

    let result = ColdStartExplorationController::evaluate(
        &goal(1, 0),
        std::slice::from_ref(&candidate),
        policy(),
    );

    assert_eq!(result.status(), ColdStartExplorationStatus::Selected,);

    assert_eq!(
        result
            .selected_exploration()
            .unwrap()
            .net_exploration_value(),
        signal(700),
    );
}

#[test]
fn execution_cost_can_reverse_cold_start_preference() {
    let expensive = candidate(1, 10, 100, 900, 900, 900, 700);

    let efficient = candidate(1, 20, 200, 800, 800, 800, 100);

    let result =
        ColdStartExplorationController::evaluate(&goal(1, 0), &[expensive, efficient], policy());

    assert_eq!(
        result.selected_exploration().unwrap().candidate().action(),
        &atom(20),
    );

    assert_eq!(
        result
            .selected_exploration()
            .unwrap()
            .net_exploration_value(),
        signal(700),
    );
}

#[test]
fn selected_cold_start_preserves_exact_action_and_prediction() {
    let result = ColdStartExplorationController::evaluate(
        &goal(1, 0),
        &[candidate(1, 10, 100, 900, 900, 900, 100)],
        policy(),
    );

    let selected = result.selected_exploration().unwrap().candidate();

    assert_eq!(selected.action(), &atom(10),);

    assert_eq!(selected.predicted_outcome(), &atom(100),);
}

#[test]
fn exact_duplicate_candidates_are_deduplicated() {
    let value = candidate(1, 10, 100, 900, 900, 900, 100);

    let result =
        ColdStartExplorationController::evaluate(&goal(1, 0), &[value.clone(), value], policy());

    assert_eq!(result.input_candidate_count(), 2,);

    assert_eq!(result.unique_candidate_count(), 1,);

    assert_eq!(result.duplicate_candidate_count(), 1,);
}

#[test]
fn cold_start_frontiers_are_hard_and_explicit() {
    let bounded =
        ColdStartExplorationPolicy::new(1, 1, signal(1), signal(1), signal(1), signal(1)).unwrap();

    let result = ColdStartExplorationController::evaluate(
        &goal(1, 0),
        &[
            candidate(1, 10, 100, 900, 900, 900, 0),
            candidate(1, 20, 200, 800, 800, 800, 0),
        ],
        bounded,
    );

    assert_eq!(result.unique_candidate_count(), 2,);

    assert_eq!(result.considered_candidate_count(), 1,);

    assert!(result.candidate_frontier_truncated());

    assert_eq!(result.candidate_evaluation_count(), 1,);
}

#[test]
fn cold_start_is_order_invariant_non_mutating_and_facade_equivalent() {
    let first = candidate(1, 10, 100, 800, 900, 900, 100);

    let second = candidate(1, 20, 200, 900, 900, 900, 100);

    let candidates = vec![first, second];

    let before = candidates.clone();

    let mut reversed = candidates.clone();

    reversed.reverse();

    let direct = ColdStartExplorationController::evaluate(&goal(1, 0), &candidates, policy());

    let reordered = ColdStartExplorationController::evaluate(&goal(1, 0), &reversed, policy());

    let facade =
        UniversalColdStartExplorationController::evaluate(&goal(1, 0), &candidates, policy());

    assert_eq!(direct, reordered,);

    assert_eq!(direct, facade,);

    assert_eq!(candidates, before,);
}
