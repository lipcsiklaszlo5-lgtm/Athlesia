use athlesia_mindstone_sparse_cognition::{
    CognitiveBudget, CognitiveFingerprint, CognitiveSignal, EpistemicOutcomePrediction,
    EpistemicSelfModel, EpistemicSelfPolicy, EpistemicSelfState, GoalInformationPrediction,
    GoalInformationPredictionSet, InformationGainGoalRanking, MindstoneExtendedSignalProfile,
    MindstoneInformationGainGoalRanking, MindstoneSignalProfile, SelfGeneratedGoalKind,
    SelfGeneratedGoalPolicy,
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

fn custom_goal_policy(
    max_goals: usize,
    resolve_cost: u32,
    learning_cost: u32,
    control_cost: u32,
    compression_cost: u32,
) -> SelfGeneratedGoalPolicy {
    SelfGeneratedGoalPolicy::new(
        max_goals,
        resolve_cost,
        learning_cost,
        control_cost,
        compression_cost,
    )
    .unwrap()
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

fn outcome(weight: u32, uncertainty: u16) -> EpistemicOutcomePrediction {
    EpistemicOutcomePrediction::new(weight, signal(uncertainty)).unwrap()
}

fn prediction(id: u64, outcomes: Vec<EpistemicOutcomePrediction>) -> GoalInformationPrediction {
    GoalInformationPrediction::new(CognitiveFingerprint::new(id), outcomes).unwrap()
}

fn predictions(values: Vec<GoalInformationPrediction>) -> GoalInformationPredictionSet {
    GoalInformationPredictionSet::new(values).unwrap()
}

#[test]
fn goal_information_prediction_requires_nonempty_outcomes() {
    assert_eq!(
        GoalInformationPrediction::new(CognitiveFingerprint::new(1,), Vec::new(),),
        None
    );

    let valid = prediction(1, vec![outcome(1, 200)]);

    assert_eq!(valid.fingerprint(), CognitiveFingerprint::new(1,));

    assert_eq!(valid.outcomes().len(), 1);
}

#[test]
fn prediction_set_rejects_duplicate_fingerprint_ambiguity() {
    let first = prediction(1, vec![outcome(1, 100)]);

    let second = prediction(1, vec![outcome(1, 200)]);

    assert_eq!(
        GoalInformationPredictionSet::new(vec![first, second,],),
        None
    );

    let empty = GoalInformationPredictionSet::empty();

    assert!(empty.is_empty());
}

#[test]
fn goal_without_prediction_retains_zero_information_gain() {
    let state = observe(empty_state(4), 1, 10, profile(900, 100, 100, 100));

    let result = InformationGainGoalRanking::rank(
        &state,
        epistemic_policy(),
        goal_policy(4),
        &GoalInformationPredictionSet::empty(),
        budget(100),
    );

    assert_eq!(result.selected_count(), 1);

    let ranked = result.selected()[0];

    assert!(!ranked.has_prediction());

    assert_eq!(ranked.information_gain(), CognitiveSignal::zero());

    assert_eq!(ranked.expected_uncertainty(), None);
}

#[test]
fn matched_prediction_derives_gain_from_self_model_uncertainty() {
    let state = observe(empty_state(4), 1, 11, profile(900, 100, 100, 100));

    let prediction_set = predictions(vec![prediction(11, vec![outcome(1, 300)])]);

    let result = InformationGainGoalRanking::rank(
        &state,
        epistemic_policy(),
        goal_policy(4),
        &prediction_set,
        budget(100),
    );

    let ranked = result.selected()[0];

    assert!(ranked.has_prediction());

    assert_eq!(ranked.information_gain().value(), 600);

    assert_eq!(ranked.expected_uncertainty().unwrap().value(), 300);

    assert_eq!(result.matched_prediction_count(), 1);
}

#[test]
fn higher_information_gain_outranks_higher_base_uncertainty_priority() {
    let mut state = empty_state(4);

    state = observe(state, 1, 1, profile(900, 100, 100, 100));

    state = observe(state, 2, 2, profile(700, 100, 100, 100));

    let prediction_set = predictions(vec![
        prediction(1, vec![outcome(1, 800)]),
        prediction(2, vec![outcome(1, 0)]),
    ]);

    let result = InformationGainGoalRanking::rank(
        &state,
        epistemic_policy(),
        goal_policy(4),
        &prediction_set,
        budget(100),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(2,)
    );

    assert_eq!(result.selected()[0].information_gain().value(), 700);

    assert_eq!(result.selected()[1].information_gain().value(), 100);
}

#[test]
fn equal_information_gain_falls_back_to_higher_base_priority() {
    let mut state = empty_state(4);

    state = observe(state, 1, 1, profile(900, 100, 100, 100));

    state = observe(state, 2, 2, profile(700, 100, 100, 100));

    let prediction_set = predictions(vec![
        prediction(1, vec![outcome(1, 500)]),
        prediction(2, vec![outcome(1, 300)]),
    ]);

    let result = InformationGainGoalRanking::rank(
        &state,
        epistemic_policy(),
        goal_policy(4),
        &prediction_set,
        budget(100),
    );

    assert_eq!(result.selected()[0].information_gain().value(), 400);

    assert_eq!(result.selected()[1].information_gain().value(), 400);

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(1,)
    );

    assert_eq!(result.selected()[0].base_priority().value(), 900);
}

#[test]
fn equal_gain_and_priority_prefer_lower_goal_cost() {
    let mut state = empty_state(4);

    state = observe(state, 1, 1, profile(800, 100, 100, 100));

    state = observe(state, 2, 2, profile(800, 800, 100, 100));

    let prediction_set = predictions(vec![
        prediction(1, vec![outcome(1, 400)]),
        prediction(2, vec![outcome(1, 400)]),
    ]);

    let ranking_policy = custom_goal_policy(4, 5, 2, 4, 2);

    let result = InformationGainGoalRanking::rank(
        &state,
        epistemic_policy(),
        ranking_policy,
        &prediction_set,
        budget(100),
    );

    assert_eq!(result.selected()[0].information_gain().value(), 400);

    assert_eq!(result.selected()[1].information_gain().value(), 400);

    assert_eq!(result.selected()[0].base_priority().value(), 800);

    assert_eq!(result.selected()[1].base_priority().value(), 800);

    assert_eq!(
        result.selected()[0].kind(),
        SelfGeneratedGoalKind::ContinueLearning
    );

    assert_eq!(result.selected()[0].estimated_cost(), 2);

    assert_eq!(result.selected()[1].estimated_cost(), 5);
}

#[test]
fn predicted_uncertainty_worsening_produces_zero_gain_without_removing_goal() {
    let state = observe(empty_state(4), 1, 5, profile(500, 100, 100, 100));

    let prediction_set = predictions(vec![prediction(5, vec![outcome(1, 900)])]);

    let result = InformationGainGoalRanking::rank(
        &state,
        epistemic_policy(),
        goal_policy(4),
        &prediction_set,
        budget(100),
    );

    assert_eq!(result.selected_count(), 1);

    assert!(result.selected()[0].has_prediction());

    assert_eq!(
        result.selected()[0].information_gain(),
        CognitiveSignal::zero()
    );

    assert_eq!(
        result.selected()[0].expected_uncertainty().unwrap().value(),
        900
    );
}

#[test]
fn irrelevant_prediction_is_ignored_but_reported_as_unmatched() {
    let state = observe(empty_state(4), 1, 1, profile(700, 100, 100, 100));

    let prediction_set = predictions(vec![prediction(999, vec![outcome(1, 0)])]);

    let result = InformationGainGoalRanking::rank(
        &state,
        epistemic_policy(),
        goal_policy(4),
        &prediction_set,
        budget(100),
    );

    assert_eq!(result.prediction_count(), 1);

    assert_eq!(result.matched_prediction_count(), 0);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(
        result.selected()[0].information_gain(),
        CognitiveSignal::zero()
    );
}

#[test]
fn information_gain_frontier_has_hard_goal_count_bound() {
    let mut state = empty_state(8);

    for index in 1_u64..=5 {
        state = observe(state, index, index, profile(900, 100, 100, 100));
    }

    let prediction_set = predictions(vec![
        prediction(1, vec![outcome(1, 800)]),
        prediction(2, vec![outcome(1, 700)]),
        prediction(3, vec![outcome(1, 600)]),
        prediction(4, vec![outcome(1, 500)]),
        prediction(5, vec![outcome(1, 0)]),
    ]);

    let result = InformationGainGoalRanking::rank(
        &state,
        epistemic_policy(),
        goal_policy(2),
        &prediction_set,
        budget(100),
    );

    assert_eq!(result.candidate_goal_count(), 5);

    assert_eq!(result.selected_count(), 2);

    assert!(result.truncated_by_goal_limit());

    assert!(result.was_truncated());

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(5,)
    );

    assert_eq!(
        result.selected()[1].fingerprint(),
        CognitiveFingerprint::new(4,)
    );
}

#[test]
fn unaffordable_next_information_ranked_goal_stops_without_cheaper_tail() {
    let mut state = empty_state(6);

    state = observe(state, 1, 1, profile(1000, 100, 100, 100));

    state = observe(state, 2, 2, profile(900, 900, 100, 100));

    state = observe(state, 3, 3, profile(800, 100, 100, 100));

    let prediction_set = predictions(vec![
        prediction(1, vec![outcome(1, 100)]),
        prediction(2, vec![outcome(1, 100)]),
        prediction(3, vec![outcome(1, 100)]),
    ]);

    let ranking_policy = custom_goal_policy(6, 1, 5, 4, 1);

    let result = InformationGainGoalRanking::rank(
        &state,
        epistemic_policy(),
        ranking_policy,
        &prediction_set,
        budget(4),
    );

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
fn information_gain_goal_ranking_is_deterministic_non_mutating_and_facade_equivalent() {
    let mut state = empty_state(6);

    state = observe(state, 1, 1, profile(900, 100, 100, 100));

    state = observe(state, 2, 2, profile(700, 600, 100, 100));

    let prediction_set = predictions(vec![
        prediction(1, vec![outcome(1, 400)]),
        prediction(2, vec![outcome(1, 100)]),
    ]);

    let state_before = state.clone();

    let predictions_before = prediction_set.clone();

    let self_policy = epistemic_policy();

    let goals = goal_policy(4);

    let compute = budget(5);

    let direct =
        InformationGainGoalRanking::rank(&state, self_policy, goals, &prediction_set, compute);

    let facade = MindstoneInformationGainGoalRanking::evaluate(
        &state,
        self_policy,
        goals,
        &prediction_set,
        compute,
    );

    let repeated = MindstoneInformationGainGoalRanking::evaluate(
        &state,
        self_policy,
        goals,
        &prediction_set,
        compute,
    );

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(state, state_before);

    assert_eq!(prediction_set, predictions_before);

    assert!(facade.total_selected_cost() <= compute.units());
}
