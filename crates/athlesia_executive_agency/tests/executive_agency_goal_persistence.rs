use athlesia_executive_agency::{
    ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveSelectionThresholds, ExecutiveUtilityWeights,
    GoalPersistence, GoalPersistenceDecision, GoalPersistencePolicy,
    GroundedExecutiveActionCandidate, PersistentExecutiveCommitment, UniversalGoalPersistence,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(values.iter().copied().map(atom).collect()).unwrap()
}

fn goal(identity: u64, satisfaction: u16) -> ExecutiveGoal {
    ExecutiveGoal::new(atom(identity), signal(1000), signal(satisfaction))
}

fn candidate_with_action(
    goal_identity: CognitiveStructure,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
    alignment: u16,
) -> GroundedExecutiveActionCandidate {
    GroundedExecutiveActionCandidate::new(
        goal_identity,
        action,
        outcome,
        signal(alignment),
        signal(1000),
        signal(1000),
        signal(0),
        signal(0),
    )
}

fn candidate(goal_identity: u64, action: u64, alignment: u16) -> GroundedExecutiveActionCandidate {
    candidate_with_action(
        atom(goal_identity),
        atom(action),
        atom(action + 100),
        alignment,
    )
}

fn agency_policy() -> ExecutiveAgencyPolicy {
    let weights = ExecutiveUtilityWeights::new(1, 0, 0, 0, 0).unwrap();

    let thresholds =
        ExecutiveSelectionThresholds::new(signal(1), signal(1), signal(1), signal(1), signal(1))
            .unwrap();

    ExecutiveAgencyPolicy::new(16, 16, 64, 1, weights, thresholds).unwrap()
}

fn persistence_policy(
    max_stalled_cycles: usize,
    switch_margin: u16,
    max_challengers: usize,
) -> GoalPersistencePolicy {
    GoalPersistencePolicy::new(max_stalled_cycles, signal(switch_margin), max_challengers).unwrap()
}

fn establish(
    goals: &[ExecutiveGoal],
    candidates: &[GroundedExecutiveActionCandidate],
    policy: GoalPersistencePolicy,
) -> PersistentExecutiveCommitment {
    GoalPersistence::select(None, goals, candidates, agency_policy(), policy)
        .commitment()
        .unwrap()
        .clone()
}

#[test]
fn goal_persistence_policy_requires_positive_stall_margin_and_challenger_bounds() {
    assert_eq!(GoalPersistencePolicy::new(0, signal(100,), 1,), None);

    assert_eq!(GoalPersistencePolicy::new(1, signal(0,), 1,), None);

    assert_eq!(GoalPersistencePolicy::new(1, signal(100,), 0,), None);

    assert!(GoalPersistencePolicy::new(2, signal(100,), 8,).is_some());
}

#[test]
fn no_prior_commitment_establishes_best_current_intent() {
    let goals = vec![goal(1, 0)];

    let candidates = vec![candidate(1, 10, 700), candidate(1, 11, 900)];

    let result = GoalPersistence::select(
        None,
        &goals,
        &candidates,
        agency_policy(),
        persistence_policy(3, 100, 8),
    );

    assert_eq!(result.decision(), GoalPersistenceDecision::Established);

    assert_eq!(result.commitment().unwrap().action(), &atom(11,));

    assert_eq!(result.commitment().unwrap().age_cycles(), 1);
}

#[test]
fn small_challenger_advantage_does_not_break_existing_commitment() {
    let goals = vec![goal(1, 0)];

    let initial_candidates = vec![candidate(1, 10, 700)];

    let persistence = persistence_policy(4, 100, 8);

    let incumbent = establish(&goals, &initial_candidates, persistence);

    let current_candidates = vec![candidate(1, 10, 700), candidate(1, 11, 750)];

    let result = GoalPersistence::select(
        Some(&incumbent),
        &goals,
        &current_candidates,
        agency_policy(),
        persistence,
    );

    assert_eq!(result.decision(), GoalPersistenceDecision::Continued);

    assert!(!result.switch_margin_satisfied());

    assert_eq!(result.commitment().unwrap().action(), &atom(10,));
}

#[test]
fn challenger_switches_only_when_required_margin_is_reached() {
    let goals = vec![goal(1, 0)];

    let persistence = persistence_policy(4, 100, 8);

    let incumbent = establish(&goals, &[candidate(1, 10, 700)], persistence);

    let result = GoalPersistence::select(
        Some(&incumbent),
        &goals,
        &[candidate(1, 10, 700), candidate(1, 11, 850)],
        agency_policy(),
        persistence,
    );

    assert_eq!(
        result.decision(),
        GoalPersistenceDecision::SwitchedChallenge
    );

    assert!(result.switch_margin_satisfied());

    assert_eq!(result.commitment().unwrap().action(), &atom(11,));
}

#[test]
fn observed_goal_progress_resets_stall_counter_and_preserves_incumbent() {
    let persistence = persistence_policy(3, 100, 8);

    let initial_goals = vec![goal(1, 0)];

    let candidates = vec![candidate(1, 10, 900)];

    let initial = establish(&initial_goals, &candidates, persistence);

    let stalled_once = GoalPersistence::select(
        Some(&initial),
        &initial_goals,
        &candidates,
        agency_policy(),
        persistence,
    );

    assert_eq!(stalled_once.commitment().unwrap().stalled_cycles(), 1);

    let progressed_goals = vec![goal(1, 100)];

    let progressed = GoalPersistence::select(
        stalled_once.commitment(),
        &progressed_goals,
        &candidates,
        agency_policy(),
        persistence,
    );

    assert_eq!(progressed.decision(), GoalPersistenceDecision::Continued);

    assert!(progressed.progress_observed());

    assert_eq!(progressed.commitment().unwrap().stalled_cycles(), 0);

    assert_eq!(progressed.commitment().unwrap().age_cycles(), 3);
}

#[test]
fn repeated_non_progress_reaches_stall_limit_and_forces_replanning() {
    let persistence = persistence_policy(2, 200, 8);

    let goals = vec![goal(1, 0)];

    let candidates = vec![candidate(1, 10, 800), candidate(1, 11, 600)];

    let initial = establish(&goals, &[candidate(1, 10, 800)], persistence);

    let first = GoalPersistence::select(
        Some(&initial),
        &goals,
        &candidates,
        agency_policy(),
        persistence,
    );

    assert_eq!(first.decision(), GoalPersistenceDecision::Continued);

    assert_eq!(first.commitment().unwrap().stalled_cycles(), 1);

    let second = GoalPersistence::select(
        first.commitment(),
        &goals,
        &candidates,
        agency_policy(),
        persistence,
    );

    assert_eq!(second.decision(), GoalPersistenceDecision::SwitchedStalled);

    assert_eq!(second.commitment().unwrap().action(), &atom(11,));

    assert_eq!(second.commitment().unwrap().stalled_cycles(), 0);
}

#[test]
fn satisfied_incumbent_goal_is_released_and_next_goal_can_take_control() {
    let persistence = persistence_policy(4, 100, 8);

    let incumbent = establish(&[goal(1, 0)], &[candidate(1, 10, 800)], persistence);

    let current_goals = vec![goal(1, 1000), goal(2, 0)];

    let current_candidates = vec![candidate(1, 10, 1000), candidate(2, 20, 700)];

    let result = GoalPersistence::select(
        Some(&incumbent),
        &current_goals,
        &current_candidates,
        agency_policy(),
        persistence,
    );

    assert_eq!(
        result.decision(),
        GoalPersistenceDecision::SwitchedGoalSatisfied
    );

    assert_eq!(result.commitment().unwrap().goal_identity(), &atom(2,));
}

#[test]
fn unavailable_incumbent_action_triggers_switch_to_viable_alternative() {
    let persistence = persistence_policy(4, 100, 8);

    let goals = vec![goal(1, 0)];

    let incumbent = establish(&goals, &[candidate(1, 10, 800)], persistence);

    let result = GoalPersistence::select(
        Some(&incumbent),
        &goals,
        &[candidate(1, 11, 700)],
        agency_policy(),
        persistence,
    );

    assert_eq!(
        result.decision(),
        GoalPersistenceDecision::SwitchedIncumbentUnavailable
    );

    assert!(!result.incumbent_available());

    assert_eq!(result.commitment().unwrap().action(), &atom(11,));
}

#[test]
fn unavailable_incumbent_without_alternative_releases_commitment_and_abstains() {
    let persistence = persistence_policy(4, 100, 8);

    let goals = vec![goal(1, 0)];

    let incumbent = establish(&goals, &[candidate(1, 10, 800)], persistence);

    let result =
        GoalPersistence::select(Some(&incumbent), &goals, &[], agency_policy(), persistence);

    assert_eq!(
        result.decision(),
        GoalPersistenceDecision::ReleasedIncumbentUnavailable
    );

    assert!(result.abstained());
}

#[test]
fn reordered_opaque_action_identity_never_impersonates_existing_incumbent() {
    let persistence = persistence_policy(4, 100, 8);

    let goals = vec![goal(1, 0)];

    let first_action = ordered(&[10, 11]);

    let reordered_action = ordered(&[11, 10]);

    assert_ne!(first_action, reordered_action);

    let incumbent = establish(
        &goals,
        &[candidate_with_action(atom(1), first_action, atom(100), 800)],
        persistence,
    );

    let result = GoalPersistence::select(
        Some(&incumbent),
        &goals,
        &[candidate_with_action(
            atom(1),
            reordered_action.clone(),
            atom(100),
            800,
        )],
        agency_policy(),
        persistence,
    );

    assert_eq!(
        result.decision(),
        GoalPersistenceDecision::SwitchedIncumbentUnavailable
    );

    assert_eq!(result.commitment().unwrap().action(), &reordered_action);
}

#[test]
fn hard_challenger_frontier_uses_best_deterministic_challenger() {
    let persistence = persistence_policy(4, 100, 1);

    let goals = vec![goal(1, 0)];

    let incumbent = establish(&goals, &[candidate(1, 10, 700)], persistence);

    let forward_candidates = vec![
        candidate(1, 10, 700),
        candidate(1, 11, 900),
        candidate(1, 12, 850),
    ];

    let mut reversed_candidates = forward_candidates.clone();

    reversed_candidates.reverse();

    let forward = GoalPersistence::select(
        Some(&incumbent),
        &goals,
        &forward_candidates,
        agency_policy(),
        persistence,
    );

    let reversed = GoalPersistence::select(
        Some(&incumbent),
        &goals,
        &reversed_candidates,
        agency_policy(),
        persistence,
    );

    assert_eq!(forward, reversed);

    assert_eq!(forward.total_challenger_count(), 2);

    assert_eq!(forward.considered_challenger_count(), 1);

    assert!(forward.challenger_frontier_truncated());

    assert_eq!(forward.commitment().unwrap().action(), &atom(11,));
}

#[test]
fn goal_persistence_is_deterministic_non_mutating_and_facade_equivalent() {
    let persistence = persistence_policy(4, 100, 8);

    let goals = vec![goal(1, 0), goal(2, 100)];

    let candidates = vec![
        candidate(1, 10, 800),
        candidate(1, 11, 850),
        candidate(2, 20, 900),
    ];

    let incumbent = establish(&[goal(1, 0)], &[candidate(1, 10, 800)], persistence);

    let goals_before = goals.clone();

    let candidates_before = candidates.clone();

    let incumbent_before = incumbent.clone();

    let mut reversed_goals = goals.clone();

    reversed_goals.reverse();

    let mut reversed_candidates = candidates.clone();

    reversed_candidates.reverse();

    let direct = GoalPersistence::select(
        Some(&incumbent),
        &goals,
        &candidates,
        agency_policy(),
        persistence,
    );

    let reversed = GoalPersistence::select(
        Some(&incumbent),
        &reversed_goals,
        &reversed_candidates,
        agency_policy(),
        persistence,
    );

    let facade = UniversalGoalPersistence::evaluate(
        Some(&incumbent),
        &goals,
        &candidates,
        agency_policy(),
        persistence,
    );

    let repeated = UniversalGoalPersistence::evaluate(
        Some(&incumbent),
        &goals,
        &candidates,
        agency_policy(),
        persistence,
    );

    assert_eq!(direct, reversed);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(goals, goals_before);

    assert_eq!(candidates, candidates_before);

    assert_eq!(incumbent, incumbent_before);
}
