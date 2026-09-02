use athlesia_executive_agency::{
    ExecutiveAgency, ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveIntent,
    ExecutiveSelectionThresholds, ExecutiveUtilityWeights, GoalConflictArbitration,
    GoalConflictArbitrationPolicy, GoalConflictArbitrationThresholds, GoalConflictEvidence,
    GoalPersistence, GoalPersistencePolicy, GroundedExecutiveActionCandidate,
    PersistentExecutiveCommitment, UniversalGoalConflictArbitration,
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

fn goal_structure(identity: CognitiveStructure) -> ExecutiveGoal {
    ExecutiveGoal::new(identity, signal(1000), signal(0))
}

fn goal(identity: u64) -> ExecutiveGoal {
    goal_structure(atom(identity))
}

fn candidate_structure(
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
    candidate_structure(
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

    ExecutiveAgencyPolicy::new(32, 32, 128, 128, weights, thresholds).unwrap()
}

fn intents(
    goals: &[ExecutiveGoal],
    candidates: &[GroundedExecutiveActionCandidate],
) -> Vec<ExecutiveIntent> {
    ExecutiveAgency::select(goals, candidates, agency_policy())
        .selected()
        .to_vec()
}

fn thresholds(strength: u16, confidence: u16) -> GoalConflictArbitrationThresholds {
    GoalConflictArbitrationThresholds::new(signal(strength), signal(confidence)).unwrap()
}

fn arbitration_policy(
    max_conflicts: usize,
    max_intents: usize,
    max_pairs: usize,
    max_selected: usize,
    continuity_bonus: u16,
) -> GoalConflictArbitrationPolicy {
    GoalConflictArbitrationPolicy::new(
        max_conflicts,
        max_intents,
        max_pairs,
        max_selected,
        signal(continuity_bonus),
        thresholds(500, 500),
    )
    .unwrap()
}

fn conflict(
    left: CognitiveStructure,
    right: CognitiveStructure,
    strength: u16,
    confidence: u16,
) -> GoalConflictEvidence {
    GoalConflictEvidence::new(left, right, signal(strength), signal(confidence)).unwrap()
}

fn persistence_policy() -> GoalPersistencePolicy {
    GoalPersistencePolicy::new(4, signal(100), 16).unwrap()
}

fn commitment(
    goals: &[ExecutiveGoal],
    candidates: &[GroundedExecutiveActionCandidate],
) -> PersistentExecutiveCommitment {
    GoalPersistence::select(
        None,
        goals,
        candidates,
        agency_policy(),
        persistence_policy(),
    )
    .commitment()
    .unwrap()
    .clone()
}

#[test]
fn conflict_policy_requires_positive_bounds_thresholds_and_distinct_goal_pair() {
    assert_eq!(
        GoalConflictArbitrationThresholds::new(signal(0,), signal(500,),),
        None
    );

    assert_eq!(
        GoalConflictArbitrationPolicy::new(0, 1, 1, 1, signal(0,), thresholds(500, 500,),),
        None
    );

    assert!(
        GoalConflictArbitrationPolicy::new(1, 1, 1, 1, signal(0,), thresholds(500, 500,),)
            .is_some()
    );

    assert_eq!(
        GoalConflictEvidence::new(atom(1,), atom(1,), signal(900,), signal(900,),),
        None
    );
}

#[test]
fn absent_conflict_evidence_preserves_multiple_viable_intents() {
    let current_intents = intents(
        &[goal(1), goal(2)],
        &[candidate(1, 10, 900), candidate(2, 20, 800)],
    );

    let result = GoalConflictArbitration::arbitrate(
        &current_intents,
        &[],
        None,
        arbitration_policy(16, 16, 64, 16, 0),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.suppressed_count(), 0);

    assert_eq!(result.selected()[0].goal_identity(), &atom(1,));
}

#[test]
fn exact_symmetric_conflict_suppresses_weaker_intent() {
    let current_intents = intents(
        &[goal(1), goal(2)],
        &[candidate(1, 10, 900), candidate(2, 20, 700)],
    );

    let evidence = vec![conflict(atom(2), atom(1), 900, 900)];

    let result = GoalConflictArbitration::arbitrate(
        &current_intents,
        &evidence,
        None,
        arbitration_policy(16, 16, 64, 16, 0),
    );

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].goal_identity(), &atom(1,));

    assert_eq!(result.suppressed_count(), 1);

    assert_eq!(result.suppressed()[0].loser_goal(), &atom(2,));
}

#[test]
fn subthreshold_conflict_evidence_cannot_suppress_viable_intent() {
    let current_intents = intents(
        &[goal(1), goal(2)],
        &[candidate(1, 10, 900), candidate(2, 20, 800)],
    );

    let evidence = vec![
        conflict(atom(1), atom(2), 499, 1000),
        conflict(atom(1), atom(2), 1000, 499),
    ];

    let result = GoalConflictArbitration::arbitrate(
        &current_intents,
        &evidence,
        None,
        arbitration_policy(16, 16, 64, 16, 0),
    );

    assert_eq!(result.eligible_conflict_count(), 0);

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.suppressed_count(), 0);
}

#[test]
fn conflict_matching_uses_exact_opaque_goal_identity() {
    let declared_goal = ordered(&[1, 2]);

    let reordered_goal = ordered(&[2, 1]);

    assert_ne!(declared_goal, reordered_goal);

    let other_goal = atom(3);

    let current_intents = intents(
        &[
            goal_structure(reordered_goal.clone()),
            goal_structure(other_goal.clone()),
        ],
        &[
            candidate_structure(reordered_goal, atom(10), atom(110), 900),
            candidate_structure(other_goal.clone(), atom(20), atom(120), 800),
        ],
    );

    let evidence = vec![conflict(declared_goal, other_goal, 1000, 1000)];

    let result = GoalConflictArbitration::arbitrate(
        &current_intents,
        &evidence,
        None,
        arbitration_policy(16, 16, 64, 16, 0),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.suppressed_count(), 0);
}

#[test]
fn continuity_bonus_preserves_incumbent_against_small_conflicting_advantage() {
    let incumbent_goals = vec![goal(1)];

    let incumbent_candidates = vec![candidate(1, 10, 800)];

    let prior = commitment(&incumbent_goals, &incumbent_candidates);

    let current_intents = intents(
        &[goal(1), goal(2)],
        &[candidate(1, 10, 800), candidate(2, 20, 850)],
    );

    let result = GoalConflictArbitration::arbitrate(
        &current_intents,
        &[conflict(atom(1), atom(2), 1000, 1000)],
        Some(&prior),
        arbitration_policy(16, 16, 64, 16, 100),
    );

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].goal_identity(), &atom(1,));

    assert!(result.selected()[0].continuity_applied());

    assert_eq!(result.selected()[0].arbitration_score(), signal(900,));
}

#[test]
fn continuity_bonus_cannot_override_substantially_stronger_conflicting_challenger() {
    let prior = commitment(&[goal(1)], &[candidate(1, 10, 700)]);

    let current_intents = intents(
        &[goal(1), goal(2)],
        &[candidate(1, 10, 700), candidate(2, 20, 950)],
    );

    let result = GoalConflictArbitration::arbitrate(
        &current_intents,
        &[conflict(atom(1), atom(2), 1000, 1000)],
        Some(&prior),
        arbitration_policy(16, 16, 64, 16, 100),
    );

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].goal_identity(), &atom(2,));

    assert!(!result.selected()[0].continuity_applied());
}

#[test]
fn non_conflicting_goal_survives_alongside_winner_of_conflicting_pair() {
    let current_intents = intents(
        &[goal(1), goal(2), goal(3)],
        &[
            candidate(1, 10, 900),
            candidate(2, 20, 800),
            candidate(3, 30, 700),
        ],
    );

    let result = GoalConflictArbitration::arbitrate(
        &current_intents,
        &[conflict(atom(1), atom(2), 900, 900)],
        None,
        arbitration_policy(16, 16, 64, 16, 0),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.selected()[0].goal_identity(), &atom(1,));

    assert_eq!(result.selected()[1].goal_identity(), &atom(3,));
}

#[test]
fn hard_conflict_frontier_prefers_strongest_evidence_deterministically() {
    let current_intents = intents(
        &[goal(1), goal(2), goal(3)],
        &[
            candidate(1, 10, 900),
            candidate(2, 20, 800),
            candidate(3, 30, 700),
        ],
    );

    let forward_evidence = vec![
        conflict(atom(1), atom(2), 800, 900),
        conflict(atom(1), atom(3), 1000, 900),
    ];

    let mut reversed_evidence = forward_evidence.clone();

    reversed_evidence.reverse();

    let policy = arbitration_policy(1, 16, 64, 16, 0);

    let forward =
        GoalConflictArbitration::arbitrate(&current_intents, &forward_evidence, None, policy);

    let reversed =
        GoalConflictArbitration::arbitrate(&current_intents, &reversed_evidence, None, policy);

    assert_eq!(forward, reversed);

    assert_eq!(forward.eligible_conflict_count(), 2);

    assert_eq!(forward.considered_conflict_count(), 1);

    assert!(forward.conflict_frontier_truncated());

    assert_eq!(forward.selected_count(), 2);

    assert_eq!(forward.suppressed()[0].loser_goal(), &atom(3,));
}

#[test]
fn hard_intent_pair_evaluation_and_final_selection_frontiers_are_enforced() {
    let current_intents = intents(
        &[goal(1), goal(2), goal(3)],
        &[
            candidate(1, 10, 900),
            candidate(2, 20, 800),
            candidate(3, 30, 700),
        ],
    );

    let conflict_evidence = vec![conflict(atom(1), atom(3), 900, 900)];

    let intent_limited = GoalConflictArbitration::arbitrate(
        &current_intents,
        &conflict_evidence,
        None,
        arbitration_policy(16, 2, 64, 16, 0),
    );

    assert_eq!(intent_limited.considered_intent_count(), 2);

    assert!(intent_limited.intent_frontier_truncated());

    let pair_limited = GoalConflictArbitration::arbitrate(
        &current_intents,
        &conflict_evidence,
        None,
        arbitration_policy(16, 3, 1, 16, 0),
    );

    assert_eq!(pair_limited.pair_evaluation_count(), 1);

    assert!(pair_limited.pair_evaluation_truncated());

    assert_eq!(pair_limited.evaluated_intent_count(), 2);

    let final_limited = GoalConflictArbitration::arbitrate(
        &current_intents,
        &[],
        None,
        arbitration_policy(16, 3, 64, 1, 0),
    );

    assert_eq!(final_limited.admitted_before_frontier(), 3);

    assert_eq!(final_limited.selected_count(), 1);
}

#[test]
fn suppression_retains_exact_winner_loser_and_conflict_evidence() {
    let current_intents = intents(
        &[goal(1), goal(2)],
        &[candidate(1, 10, 900), candidate(2, 20, 800)],
    );

    let result = GoalConflictArbitration::arbitrate(
        &current_intents,
        &[conflict(atom(1), atom(2), 777, 888)],
        None,
        arbitration_policy(16, 16, 64, 16, 0),
    );

    assert_eq!(result.suppressed_count(), 1);

    let suppression = &result.suppressed()[0];

    assert_eq!(suppression.winner_goal(), &atom(1,));

    assert_eq!(suppression.loser_goal(), &atom(2,));

    assert_eq!(suppression.conflict_strength(), signal(777,));

    assert_eq!(suppression.evidence_confidence(), signal(888,));
}

#[test]
fn goal_conflict_arbitration_is_order_invariant_non_mutating_and_facade_equivalent() {
    let current_intents = intents(
        &[goal(1), goal(2), goal(3)],
        &[
            candidate(1, 10, 900),
            candidate(2, 20, 850),
            candidate(3, 30, 700),
        ],
    );

    let evidence = vec![
        conflict(atom(1), atom(2), 900, 900),
        conflict(atom(2), atom(3), 700, 800),
    ];

    let intents_before = current_intents.clone();

    let evidence_before = evidence.clone();

    let mut reversed_intents = current_intents.clone();

    reversed_intents.reverse();

    let mut reversed_evidence = evidence.clone();

    reversed_evidence.reverse();

    let policy = arbitration_policy(16, 16, 64, 16, 50);

    let direct = GoalConflictArbitration::arbitrate(&current_intents, &evidence, None, policy);

    let reversed =
        GoalConflictArbitration::arbitrate(&reversed_intents, &reversed_evidence, None, policy);

    let facade =
        UniversalGoalConflictArbitration::evaluate(&current_intents, &evidence, None, policy);

    let repeated =
        UniversalGoalConflictArbitration::evaluate(&current_intents, &evidence, None, policy);

    assert_eq!(direct, reversed);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(current_intents, intents_before);

    assert_eq!(evidence, evidence_before);
}
