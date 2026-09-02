use athlesia_executive_agency::{
    ExecutiveAgency, ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveSelectionThresholds,
    ExecutiveUtilityWeights, GroundedExecutiveActionCandidate, UniversalExecutiveAgency,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Copy)]
struct CandidateSignals {
    alignment: u16,
    control: u16,
    evidence: u16,
    information: u16,
    cost: u16,
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(values.iter().copied().map(atom).collect()).unwrap()
}

fn goal(identity: u64, priority: u16, satisfaction: u16) -> ExecutiveGoal {
    ExecutiveGoal::new(atom(identity), signal(priority), signal(satisfaction))
}

fn candidate(
    goal_identity: CognitiveStructure,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
    values: CandidateSignals,
) -> GroundedExecutiveActionCandidate {
    GroundedExecutiveActionCandidate::new(
        goal_identity,
        action,
        outcome,
        signal(values.alignment),
        signal(values.control),
        signal(values.evidence),
        signal(values.information),
        signal(values.cost),
    )
}

fn default_values() -> CandidateSignals {
    CandidateSignals {
        alignment: 800,
        control: 800,
        evidence: 800,
        information: 400,
        cost: 100,
    }
}

fn weights(
    alignment: u16,
    control: u16,
    evidence: u16,
    information: u16,
    cost: u16,
) -> ExecutiveUtilityWeights {
    ExecutiveUtilityWeights::new(alignment, control, evidence, information, cost).unwrap()
}

fn thresholds() -> ExecutiveSelectionThresholds {
    ExecutiveSelectionThresholds::new(signal(1), signal(1), signal(1), signal(1), signal(1))
        .unwrap()
}

fn policy(
    max_goals: usize,
    max_actions: usize,
    max_evaluations: usize,
    max_intents: usize,
    utility_weights: ExecutiveUtilityWeights,
) -> ExecutiveAgencyPolicy {
    ExecutiveAgencyPolicy::new(
        max_goals,
        max_actions,
        max_evaluations,
        max_intents,
        utility_weights,
        thresholds(),
    )
    .unwrap()
}

fn default_policy() -> ExecutiveAgencyPolicy {
    policy(16, 16, 64, 16, weights(3, 2, 2, 1, 1))
}

#[test]
fn executive_policy_requires_positive_bounds_benefit_weights_and_thresholds() {
    assert_eq!(ExecutiveUtilityWeights::new(0, 0, 0, 0, 10,), None);

    let valid_weights = weights(1, 1, 1, 1, 1);

    assert_eq!(
        ExecutiveAgencyPolicy::new(0, 1, 1, 1, valid_weights, thresholds(),),
        None
    );

    assert_eq!(
        ExecutiveSelectionThresholds::new(
            signal(0,),
            signal(1,),
            signal(1,),
            signal(1,),
            signal(1,),
        ),
        None
    );

    assert!(ExecutiveAgencyPolicy::new(1, 1, 1, 1, valid_weights, thresholds(),).is_some());
}

#[test]
fn fully_satisfied_goal_produces_explicit_abstention() {
    let goals = vec![goal(1, 1000, 1000)];

    let actions = vec![candidate(atom(1), atom(10), atom(20), default_values())];

    let result = ExecutiveAgency::select(&goals, &actions, default_policy());

    assert!(result.abstained());

    assert_eq!(result.considered_goal_count(), 0);
}

#[test]
fn action_candidate_must_match_goal_by_exact_opaque_identity() {
    let goals = vec![goal(1, 1000, 0)];

    let actions = vec![candidate(atom(2), atom(10), atom(20), default_values())];

    let result = ExecutiveAgency::select(&goals, &actions, default_policy());

    assert_eq!(result.matching_candidate_count(), 0);

    assert!(result.abstained());
}

#[test]
fn evidence_thresholds_reject_weak_action_without_forcing_execution() {
    let strict_thresholds = ExecutiveSelectionThresholds::new(
        signal(100),
        signal(500),
        signal(500),
        signal(700),
        signal(100),
    )
    .unwrap();

    let strict_policy =
        ExecutiveAgencyPolicy::new(8, 8, 8, 8, weights(1, 1, 1, 1, 1), strict_thresholds).unwrap();

    let result = ExecutiveAgency::select(
        &[goal(1, 1000, 0)],
        &[candidate(
            atom(1),
            atom(10),
            atom(20),
            CandidateSignals {
                evidence: 699,
                ..default_values()
            },
        )],
        strict_policy,
    );

    assert_eq!(result.evaluated_candidate_count(), 1);

    assert_eq!(result.rejected_by_threshold_count(), 1);

    assert!(result.abstained());
}

#[test]
fn greater_goal_alignment_wins_when_other_evidence_is_equal() {
    let goals = vec![goal(1, 1000, 0)];

    let weak = candidate(
        atom(1),
        atom(10),
        atom(20),
        CandidateSignals {
            alignment: 600,
            ..default_values()
        },
    );

    let strong = candidate(
        atom(1),
        atom(11),
        atom(21),
        CandidateSignals {
            alignment: 900,
            ..default_values()
        },
    );

    let result = ExecutiveAgency::select(
        &goals,
        &[weak, strong.clone()],
        policy(8, 8, 8, 1, weights(5, 1, 1, 0, 0)),
    );

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].action(), strong.action());
}

#[test]
fn execution_cost_can_reverse_action_preference() {
    let goals = vec![goal(1, 1000, 0)];

    let expensive = candidate(
        atom(1),
        atom(10),
        atom(20),
        CandidateSignals {
            alignment: 1000,
            control: 1000,
            evidence: 1000,
            information: 0,
            cost: 900,
        },
    );

    let efficient = candidate(
        atom(1),
        atom(11),
        atom(21),
        CandidateSignals {
            alignment: 800,
            control: 1000,
            evidence: 1000,
            information: 0,
            cost: 0,
        },
    );

    let result = ExecutiveAgency::select(
        &goals,
        &[expensive, efficient.clone()],
        policy(8, 8, 8, 1, weights(1, 0, 0, 0, 2)),
    );

    assert_eq!(result.selected()[0].action(), efficient.action());
}

#[test]
fn information_gain_can_drive_exploration_when_policy_values_learning() {
    let goals = vec![goal(1, 1000, 0)];

    let exploit = candidate(
        atom(1),
        atom(10),
        atom(20),
        CandidateSignals {
            alignment: 800,
            control: 800,
            evidence: 800,
            information: 100,
            cost: 0,
        },
    );

    let explore = candidate(
        atom(1),
        atom(11),
        atom(21),
        CandidateSignals {
            alignment: 700,
            control: 700,
            evidence: 700,
            information: 1000,
            cost: 0,
        },
    );

    let result = ExecutiveAgency::select(
        &goals,
        &[exploit, explore.clone()],
        policy(8, 8, 8, 1, weights(1, 1, 1, 8, 0)),
    );

    assert_eq!(result.selected()[0].action(), explore.action());
}

#[test]
fn goal_pressure_combines_priority_with_remaining_unsatisfied_need() {
    let nearly_done = goal(1, 1000, 900);

    let unfinished = goal(2, 800, 0);

    assert!(unfinished.pressure().value() > nearly_done.pressure().value());

    let actions = vec![
        candidate(atom(1), atom(10), atom(20), default_values()),
        candidate(atom(2), atom(11), atom(21), default_values()),
    ];

    let result = ExecutiveAgency::select(
        &[nearly_done, unfinished],
        &actions,
        policy(8, 8, 8, 1, weights(1, 1, 1, 1, 0)),
    );

    assert_eq!(result.selected()[0].goal_identity(), &atom(2,));
}

#[test]
fn hard_goal_frontier_prefers_highest_pressure_goal() {
    let goals = vec![goal(1, 500, 0), goal(2, 900, 0)];

    let actions = vec![
        candidate(atom(1), atom(10), atom(20), default_values()),
        candidate(atom(2), atom(11), atom(21), default_values()),
    ];

    let result =
        ExecutiveAgency::select(&goals, &actions, policy(1, 8, 8, 8, weights(1, 1, 1, 1, 0)));

    assert_eq!(result.considered_goal_count(), 1);

    assert!(result.goal_frontier_truncated());

    assert_eq!(result.selected()[0].goal_identity(), &atom(2,));
}

#[test]
fn hard_candidate_evaluation_and_final_intent_frontiers_are_enforced() {
    let goals = vec![goal(1, 1000, 0)];

    let actions = vec![
        candidate(
            atom(1),
            atom(10),
            atom(20),
            CandidateSignals {
                alignment: 900,
                ..default_values()
            },
        ),
        candidate(
            atom(1),
            atom(11),
            atom(21),
            CandidateSignals {
                alignment: 800,
                ..default_values()
            },
        ),
        candidate(
            atom(1),
            atom(12),
            atom(22),
            CandidateSignals {
                alignment: 700,
                ..default_values()
            },
        ),
    ];

    let candidate_limited =
        ExecutiveAgency::select(&goals, &actions, policy(8, 2, 8, 8, weights(1, 1, 1, 1, 0)));

    assert_eq!(candidate_limited.matching_candidate_count(), 3);

    assert!(candidate_limited.candidate_frontier_truncated());

    let evaluation_limited =
        ExecutiveAgency::select(&goals, &actions, policy(8, 3, 1, 8, weights(1, 1, 1, 1, 0)));

    assert_eq!(evaluation_limited.evaluated_candidate_count(), 1);

    assert!(evaluation_limited.evaluation_truncated());

    let final_limited =
        ExecutiveAgency::select(&goals, &actions, policy(8, 3, 3, 1, weights(1, 1, 1, 1, 0)));

    assert_eq!(final_limited.admitted_before_frontier(), 3);

    assert_eq!(final_limited.selected_count(), 1);
}

#[test]
fn reordered_opaque_action_structures_remain_distinct_and_deterministic() {
    let first_action = ordered(&[10, 11]);

    let second_action = ordered(&[11, 10]);

    assert_ne!(first_action, second_action);

    let goals = vec![goal(1, 1000, 0)];

    let first = candidate(atom(1), first_action, atom(20), default_values());

    let second = candidate(atom(1), second_action, atom(21), default_values());

    let forward = ExecutiveAgency::select(
        &goals,
        &[first.clone(), second.clone()],
        policy(8, 8, 8, 2, weights(1, 1, 1, 1, 0)),
    );

    let reverse = ExecutiveAgency::select(
        &goals,
        &[second, first],
        policy(8, 8, 8, 2, weights(1, 1, 1, 1, 0)),
    );

    assert_eq!(forward, reverse);

    assert_eq!(forward.selected_count(), 2);

    assert_ne!(
        forward.selected()[0].action(),
        forward.selected()[1].action()
    );
}

#[test]
fn executive_agency_is_input_order_invariant_non_mutating_and_facade_equivalent() {
    let goals = vec![goal(1, 900, 100), goal(2, 800, 200)];

    let actions = vec![
        candidate(atom(1), atom(10), atom(20), default_values()),
        candidate(
            atom(2),
            atom(11),
            atom(21),
            CandidateSignals {
                information: 700,
                ..default_values()
            },
        ),
    ];

    let goals_before = goals.clone();

    let actions_before = actions.clone();

    let mut reversed_goals = goals.clone();

    reversed_goals.reverse();

    let mut reversed_actions = actions.clone();

    reversed_actions.reverse();

    let agency_policy = default_policy();

    let direct = ExecutiveAgency::select(&goals, &actions, agency_policy);

    let reversed = ExecutiveAgency::select(&reversed_goals, &reversed_actions, agency_policy);

    let facade = UniversalExecutiveAgency::evaluate(&goals, &actions, agency_policy);

    let repeated = UniversalExecutiveAgency::evaluate(&goals, &actions, agency_policy);

    assert_eq!(direct, reversed);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(goals, goals_before);

    assert_eq!(actions, actions_before);

    assert_eq!(facade.input_goal_count(), goals.len());
}
