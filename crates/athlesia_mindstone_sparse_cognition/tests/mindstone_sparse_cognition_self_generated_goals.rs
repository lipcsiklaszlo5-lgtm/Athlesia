use athlesia_mindstone_sparse_cognition::{
    CognitiveBudget, CognitiveFingerprint, CognitiveSignal, EpistemicSelfModel,
    EpistemicSelfPolicy, EpistemicSelfState, MindstoneExtendedSignalProfile,
    MindstoneSelfGeneratedGoals, MindstoneSignalProfile, SelfGeneratedGoal,
    SelfGeneratedGoalEngine, SelfGeneratedGoalKind, SelfGeneratedGoalPolicy,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn epistemic_policy() -> EpistemicSelfPolicy {
    EpistemicSelfPolicy::new(signal(200), signal(300), signal(400), signal(600), 2).unwrap()
}

fn goal_policy(max_goals: usize) -> SelfGeneratedGoalPolicy {
    SelfGeneratedGoalPolicy::new(max_goals, 2, 3, 4, 2).unwrap()
}

fn budget(units: u32) -> CognitiveBudget {
    CognitiveBudget::new(units).unwrap()
}

fn profile(
    uncertainty: u16,
    learning_progress: u16,
    compression_gain: u16,
    controllability: u16,
) -> MindstoneExtendedSignalProfile {
    MindstoneExtendedSignalProfile::new(
        MindstoneSignalProfile::new(
            signal(100),
            signal(uncertainty),
            signal(100),
            signal(learning_progress),
            signal(100),
        ),
        signal(compression_gain),
        signal(controllability),
    )
}

fn empty_state(capacity: usize) -> EpistemicSelfState {
    EpistemicSelfState::new(capacity).unwrap()
}

fn observe(
    state: EpistemicSelfState,
    index: u64,
    id: u64,
    input: MindstoneExtendedSignalProfile,
) -> EpistemicSelfState {
    EpistemicSelfModel::observe(
        state,
        index,
        CognitiveFingerprint::new(id),
        input,
        epistemic_policy(),
    )
    .state_after()
    .clone()
}

#[test]
fn self_generated_goal_and_policy_require_nonzero_priority_cost_and_bounds() {
    assert_eq!(
        SelfGeneratedGoal::new(
            CognitiveFingerprint::new(1,),
            SelfGeneratedGoalKind::ResolveUncertainty,
            CognitiveSignal::zero(),
            1,
        ),
        None
    );

    assert_eq!(
        SelfGeneratedGoal::new(
            CognitiveFingerprint::new(1,),
            SelfGeneratedGoalKind::ResolveUncertainty,
            signal(500,),
            0,
        ),
        None
    );

    assert_eq!(SelfGeneratedGoalPolicy::new(0, 2, 3, 4, 2,), None);

    assert_eq!(SelfGeneratedGoalPolicy::new(4, 0, 3, 4, 2,), None);

    assert_eq!(goal_policy(4,).max_goals(), 4);
}

#[test]
fn stable_noncompressible_knowledge_generates_no_internal_goal() {
    let mut state = empty_state(4);

    state = observe(state, 1, 1, profile(100, 0, 100, 100));

    state = observe(state, 2, 1, profile(100, 0, 100, 100));

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), goal_policy(4), budget(100));

    assert_eq!(result.source_record_count(), 1);

    assert_eq!(result.candidate_goal_count(), 0);

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn unresolved_uncertainty_generates_resolution_goal() {
    let state = observe(empty_state(4), 1, 10, profile(900, 100, 100, 100));

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), goal_policy(4), budget(100));

    assert_eq!(result.selected_count(), 1);

    let goal = result.selected()[0];

    assert_eq!(goal.kind(), SelfGeneratedGoalKind::ResolveUncertainty);

    assert_eq!(goal.priority().value(), 900);

    assert_eq!(goal.estimated_cost(), 2);
}

#[test]
fn measurable_learning_progress_generates_continue_learning_goal() {
    let state = observe(empty_state(4), 1, 11, profile(800, 700, 100, 100));

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), goal_policy(4), budget(100));

    let goal = result.selected()[0];

    assert_eq!(goal.kind(), SelfGeneratedGoalKind::ContinueLearning);

    assert_eq!(goal.priority().value(), 700);

    assert_eq!(goal.estimated_cost(), 3);
}

#[test]
fn uncertain_supported_controllable_knowledge_generates_control_test_goal() {
    let mut state = empty_state(4);

    state = observe(state, 1, 12, profile(800, 100, 100, 900));

    state = observe(state, 2, 12, profile(800, 100, 100, 900));

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), goal_policy(4), budget(100));

    let goal = result.selected()[0];

    assert_eq!(goal.kind(), SelfGeneratedGoalKind::TestControl);

    assert_eq!(goal.priority().value(), 900);

    assert_eq!(goal.estimated_cost(), 4);
}

#[test]
fn stable_supported_compressible_knowledge_generates_compression_goal() {
    let mut state = empty_state(4);

    state = observe(state, 1, 13, profile(100, 0, 900, 100));

    state = observe(state, 2, 13, profile(100, 0, 900, 100));

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), goal_policy(4), budget(100));

    let goal = result.selected()[0];

    assert_eq!(goal.kind(), SelfGeneratedGoalKind::CompressRepresentation);

    assert_eq!(goal.priority().value(), 900);

    assert_eq!(goal.estimated_cost(), 2);
}

#[test]
fn one_epistemic_identity_generates_at_most_one_primary_goal_per_cycle() {
    let mut state = empty_state(4);

    state = observe(state, 1, 20, profile(900, 800, 900, 900));

    state = observe(state, 2, 20, profile(900, 800, 900, 900));

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), goal_policy(10), budget(100));

    assert_eq!(result.source_record_count(), 1);

    assert_eq!(result.candidate_goal_count(), 1);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(
        result.selected()[0].kind(),
        SelfGeneratedGoalKind::TestControl
    );
}

#[test]
fn goal_frontier_ranks_higher_primary_signal_first() {
    let mut state = empty_state(4);

    state = observe(state, 1, 1, profile(500, 100, 100, 100));

    state = observe(state, 2, 2, profile(900, 100, 100, 100));

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), goal_policy(4), budget(100));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(2,)
    );

    assert_eq!(result.selected()[0].priority().value(), 900);
}

#[test]
fn equal_priority_goals_prefer_lower_cost_then_deterministic_identity() {
    let mut state = empty_state(6);

    state = observe(state, 1, 30, profile(800, 100, 100, 100));

    state = observe(state, 2, 40, profile(800, 800, 100, 100));

    state = observe(state, 3, 20, profile(800, 100, 100, 100));

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), goal_policy(6), budget(100));

    assert_eq!(result.selected_count(), 3);

    assert_eq!(result.selected()[0].estimated_cost(), 2);

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(20,)
    );

    assert_eq!(
        result.selected()[1].fingerprint(),
        CognitiveFingerprint::new(30,)
    );

    assert_eq!(result.selected()[2].estimated_cost(), 3);
}

#[test]
fn generated_goal_count_has_hard_frontier_bound() {
    let mut state = empty_state(16);

    for index in 1_u64..=10 {
        state = observe(state, index, index, profile(900, 100, 100, 100));
    }

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), goal_policy(3), budget(100));

    assert_eq!(result.source_record_count(), 10);

    assert_eq!(result.candidate_goal_count(), 10);

    assert_eq!(result.selected_count(), 3);

    assert!(result.truncated_by_goal_limit());

    assert!(result.was_truncated());
}

#[test]
fn unaffordable_next_goal_stops_frontier_without_cheaper_tail_substitution() {
    let mut state = empty_state(6);

    state = observe(state, 1, 1, profile(1000, 100, 100, 100));

    state = observe(state, 2, 2, profile(900, 900, 100, 100));

    state = observe(state, 3, 3, profile(800, 100, 100, 100));

    let expensive_policy = SelfGeneratedGoalPolicy::new(6, 1, 5, 4, 1).unwrap();

    let result =
        SelfGeneratedGoalEngine::generate(&state, epistemic_policy(), expensive_policy, budget(4));

    assert_eq!(result.selected_count(), 1);

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(1,)
    );

    assert_eq!(result.total_selected_cost(), 1);

    assert!(result.truncated_by_compute_budget());

    assert!(!result
        .selected()
        .iter()
        .any(|goal| { goal.fingerprint() == CognitiveFingerprint::new(3,) },));
}

#[test]
fn self_generated_goal_engine_is_deterministic_non_mutating_and_facade_equivalent() {
    let mut state = empty_state(8);

    state = observe(state, 1, 1, profile(900, 100, 100, 100));

    state = observe(state, 2, 2, profile(700, 600, 100, 100));

    let state_before = state.clone();

    let self_policy = epistemic_policy();

    let goals = goal_policy(4);

    let compute = budget(5);

    let direct = SelfGeneratedGoalEngine::generate(&state, self_policy, goals, compute);

    let facade = MindstoneSelfGeneratedGoals::evaluate(&state, self_policy, goals, compute);

    let repeated = MindstoneSelfGeneratedGoals::evaluate(&state, self_policy, goals, compute);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(state, state_before);

    assert_eq!(facade.selected_count(), 2);

    assert!(facade.total_selected_cost() <= compute.units());
}
