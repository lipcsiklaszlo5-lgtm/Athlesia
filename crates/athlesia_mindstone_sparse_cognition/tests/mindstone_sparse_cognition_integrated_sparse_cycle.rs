use athlesia_mindstone_sparse_cognition::{
    CognitiveAdmissionClass, CognitiveBudget, CognitiveFingerprint, CognitiveSignal,
    CognitiveStructure, CollisionSafeGoalInformationPrediction,
    CollisionSafeGoalInformationPredictionSet, CollisionSafeInformationGoalEngine,
    CollisionSafeStructuralIdentity, EpistemicOutcomePrediction, EpistemicSelfPolicy,
    IntegratedSparseCycle, IntegratedSparseCycleContext, IntegratedSparseCycleState,
    IntegratedSparseCycleStatus, MindstoneExtendedSignalProfile, MindstoneIntegratedSparseCycle,
    MindstoneSignalProfile, SelfGeneratedGoalKind, SelfGeneratedGoalPolicy, SparseCognitionPolicy,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn profile(
    uncertainty: u16,
    learning_progress: u16,
    compression_gain: u16,
    controllability: u16,
) -> MindstoneExtendedSignalProfile {
    MindstoneExtendedSignalProfile::new(
        MindstoneSignalProfile::new(
            signal(0),
            signal(uncertainty),
            signal(0),
            signal(learning_progress),
            signal(0),
        ),
        signal(compression_gain),
        signal(controllability),
    )
}

fn sparse_policy() -> SparseCognitionPolicy {
    SparseCognitionPolicy::new(signal(200), signal(600), 2, 8).unwrap()
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

fn outcome(weight: u32, uncertainty: u16) -> EpistemicOutcomePrediction {
    EpistemicOutcomePrediction::new(weight, signal(uncertainty)).unwrap()
}

fn prediction(
    identity: CollisionSafeStructuralIdentity,
    outcomes: Vec<EpistemicOutcomePrediction>,
) -> CollisionSafeGoalInformationPrediction {
    CollisionSafeGoalInformationPrediction::new(identity, outcomes).unwrap()
}

fn predictions(
    values: Vec<CollisionSafeGoalInformationPrediction>,
) -> CollisionSafeGoalInformationPredictionSet {
    CollisionSafeGoalInformationPredictionSet::new(values).unwrap()
}

fn empty_predictions() -> CollisionSafeGoalInformationPredictionSet {
    CollisionSafeGoalInformationPredictionSet::empty()
}

fn context(
    predictions: CollisionSafeGoalInformationPredictionSet,
    units: u32,
) -> IntegratedSparseCycleContext {
    IntegratedSparseCycleContext::new(
        predictions,
        sparse_policy(),
        epistemic_policy(),
        goal_policy(4),
        budget(units),
    )
}

fn state() -> IntegratedSparseCycleState {
    IntegratedSparseCycleState::new(16, 16).unwrap()
}

#[test]
fn integrated_state_and_exact_prediction_set_require_valid_bounded_shape() {
    assert_eq!(IntegratedSparseCycleState::new(0, 4,), None);

    assert_eq!(IntegratedSparseCycleState::new(4, 0,), None);

    let shared = CognitiveFingerprint::new(99);

    let first_identity = CollisionSafeStructuralIdentity::with_fingerprint_hint(shared, atom(1));

    let second_identity = CollisionSafeStructuralIdentity::with_fingerprint_hint(shared, atom(2));

    let collision_set = predictions(vec![
        prediction(first_identity, vec![outcome(1, 100)]),
        prediction(second_identity, vec![outcome(1, 200)]),
    ]);

    assert_eq!(collision_set.len(), 2);

    let cycle_context = context(collision_set, 20);

    assert_eq!(cycle_context.predictions().len(), 2);

    assert_eq!(cycle_context.budget().units(), 20);

    assert_eq!(cycle_context.goal_policy().max_goals(), 4);
}

#[test]
fn new_structure_flows_from_novelty_into_admission_and_exact_epistemic_state() {
    let cycle_context = context(empty_predictions(), 10);

    let result = IntegratedSparseCycle::evaluate(
        state(),
        1,
        atom(10),
        profile(400, 0, 0, 0),
        &cycle_context,
    );

    assert_eq!(result.status(), IntegratedSparseCycleStatus::Processed);

    assert_eq!(result.novelty(), CognitiveSignal::maximum());

    assert_eq!(result.state_after().structural().len(), 1);

    assert_eq!(result.state_after().epistemic().len(), 1);

    assert!(result.epistemic_record().is_some());
}

#[test]
fn exact_repeat_has_zero_novelty_and_updates_both_retained_statistics() {
    let structure = atom(20);

    let cycle_context = context(empty_predictions(), 20);

    let first = IntegratedSparseCycle::evaluate(
        state(),
        1,
        structure.clone(),
        profile(800, 0, 0, 0),
        &cycle_context,
    );

    let second = IntegratedSparseCycle::evaluate(
        first.state_after().clone(),
        2,
        structure.clone(),
        profile(800, 0, 0, 0),
        &cycle_context,
    );

    assert_eq!(second.novelty(), CognitiveSignal::zero());

    assert_eq!(
        second
            .state_after()
            .structural()
            .record_for_structure(&structure,)
            .unwrap()
            .observation_count(),
        2
    );

    assert_eq!(second.epistemic_record().unwrap().observation_count(), 2);
}

#[test]
fn forced_hash_collision_remains_separate_through_structural_and_epistemic_layers() {
    let shared = CognitiveFingerprint::new(777);

    let cycle_context = context(empty_predictions(), 20);

    let first = IntegratedSparseCycle::evaluate_with_fingerprint_hint(
        state(),
        1,
        shared,
        atom(1),
        profile(800, 0, 0, 0),
        &cycle_context,
    );

    let second = IntegratedSparseCycle::evaluate_with_fingerprint_hint(
        first.state_after().clone(),
        2,
        shared,
        atom(2),
        profile(800, 0, 0, 0),
        &cycle_context,
    );

    assert_eq!(second.state_after().structural().bucket_len(shared,), 2);

    assert_eq!(second.state_after().epistemic().bucket_len(shared,), 2);

    assert_eq!(second.state_after().epistemic().len(), 2);
}

#[test]
fn exact_prediction_matching_does_not_leak_information_gain_across_collision() {
    let shared = CognitiveFingerprint::new(888);

    let first_structure = atom(11);

    let second_structure = atom(22);

    let second_identity =
        CollisionSafeStructuralIdentity::with_fingerprint_hint(shared, second_structure.clone());

    let cycle_context = context(
        predictions(vec![prediction(second_identity, vec![outcome(1, 0)])]),
        20,
    );

    let first = IntegratedSparseCycle::evaluate_with_fingerprint_hint(
        state(),
        1,
        shared,
        first_structure,
        profile(700, 0, 0, 0),
        &cycle_context,
    );

    assert_eq!(first.current_information_gain(), CognitiveSignal::zero());

    let second = IntegratedSparseCycle::evaluate_with_fingerprint_hint(
        first.state_after().clone(),
        2,
        shared,
        second_structure,
        profile(700, 0, 0, 0),
        &cycle_context,
    );

    assert_eq!(second.current_information_gain().value(), 700);
}

#[test]
fn ignored_repeat_updates_structural_stream_but_not_epistemic_self_record() {
    let structure = atom(30);

    let cycle_context = context(empty_predictions(), 20);

    let first = IntegratedSparseCycle::evaluate(
        state(),
        1,
        structure.clone(),
        profile(900, 0, 0, 0),
        &cycle_context,
    );

    let second = IntegratedSparseCycle::evaluate(
        first.state_after().clone(),
        2,
        structure.clone(),
        profile(0, 0, 0, 0),
        &cycle_context,
    );

    assert_eq!(second.status(), IntegratedSparseCycleStatus::Ignored);

    assert_eq!(
        second.admission().unwrap().class(),
        CognitiveAdmissionClass::Ignore
    );

    assert_eq!(
        second
            .state_after()
            .structural()
            .record_for_structure(&structure,)
            .unwrap()
            .observation_count(),
        2
    );

    assert_eq!(second.state_after().epistemic().len(), 1);
}

#[test]
fn nonstable_supported_controllable_learning_prefers_control_test_in_integrated_frontier() {
    let structure = atom(40);

    let cycle_context = context(empty_predictions(), 20);

    let first = IntegratedSparseCycle::evaluate(
        state(),
        1,
        structure.clone(),
        profile(900, 800, 0, 900),
        &cycle_context,
    );

    let second = IntegratedSparseCycle::evaluate(
        first.state_after().clone(),
        2,
        structure,
        profile(900, 800, 0, 900),
        &cycle_context,
    );

    assert!(second.goals().selected_count() >= 1);

    assert_eq!(
        second.goals().selected()[0].kind(),
        SelfGeneratedGoalKind::TestControl
    );
}

#[test]
fn integrated_goal_frontier_prefers_higher_expected_information_gain_over_uncertainty() {
    let first_structure = atom(51);

    let second_structure = atom(52);

    let first_identity = CollisionSafeStructuralIdentity::from_structure(first_structure.clone());

    let second_identity = CollisionSafeStructuralIdentity::from_structure(second_structure.clone());

    let cycle_context = context(
        predictions(vec![
            prediction(first_identity, vec![outcome(1, 800)]),
            prediction(second_identity, vec![outcome(1, 0)]),
        ]),
        20,
    );

    let first = IntegratedSparseCycle::evaluate(
        state(),
        1,
        first_structure,
        profile(900, 0, 0, 0),
        &cycle_context,
    );

    let second = IntegratedSparseCycle::evaluate(
        first.state_after().clone(),
        2,
        second_structure,
        profile(700, 0, 0, 0),
        &cycle_context,
    );

    assert_eq!(second.goals().selected()[0].structure(), &atom(52));

    assert_eq!(second.goals().selected()[0].information_gain().value(), 700);
}

#[test]
fn integrated_budget_reserves_admission_compute_before_goal_frontier() {
    let cycle_context = context(empty_predictions(), 4);

    let result = IntegratedSparseCycle::evaluate(
        state(),
        1,
        atom(60),
        profile(400, 0, 0, 0),
        &cycle_context,
    );

    let admission = result.admission().unwrap();

    assert_eq!(admission.class(), CognitiveAdmissionClass::CheapUpdate);

    assert_eq!(admission.granted_units(), 2);

    assert_eq!(result.goal_compute_units(), 2);

    assert_eq!(result.goals().selected_count(), 1);

    assert_eq!(result.goals().total_selected_cost(), 2);
}

#[test]
fn exact_goal_frontier_keeps_conservative_no_skip_budget_semantics() {
    let cycle_context = context(empty_predictions(), 20);

    let first = IntegratedSparseCycle::evaluate(
        state(),
        1,
        atom(71),
        profile(1000, 0, 0, 0),
        &cycle_context,
    );

    let second = IntegratedSparseCycle::evaluate(
        first.state_after().clone(),
        2,
        atom(72),
        profile(900, 900, 0, 0),
        &cycle_context,
    );

    let third = IntegratedSparseCycle::evaluate(
        second.state_after().clone(),
        3,
        atom(73),
        profile(800, 0, 0, 0),
        &cycle_context,
    );

    let expensive_policy = SelfGeneratedGoalPolicy::new(6, 1, 5, 4, 1).unwrap();

    let frontier = CollisionSafeInformationGoalEngine::rank(
        third.state_after().epistemic(),
        epistemic_policy(),
        expensive_policy,
        &empty_predictions(),
        4,
    );

    assert_eq!(frontier.selected_count(), 1);

    assert_eq!(frontier.total_selected_cost(), 1);

    assert!(frontier.truncated_by_compute_budget());

    assert_eq!(frontier.selected()[0].structure(), &atom(71));

    assert!(!frontier
        .selected()
        .iter()
        .any(|goal| { goal.structure() == &atom(73) },));
}

#[test]
fn non_monotonic_integrated_cycle_is_rejected_without_any_state_mutation() {
    let cycle_context = context(empty_predictions(), 20);

    let first = IntegratedSparseCycle::evaluate(
        state(),
        10,
        atom(80),
        profile(800, 0, 0, 0),
        &cycle_context,
    );

    let before = first.state_after().clone();

    let rejected = IntegratedSparseCycle::evaluate(
        before.clone(),
        10,
        atom(81),
        profile(1000, 1000, 1000, 1000),
        &cycle_context,
    );

    assert_eq!(
        rejected.status(),
        IntegratedSparseCycleStatus::RejectedOutOfOrder
    );

    assert!(!rejected.accepted());

    assert_eq!(rejected.state_before(), &before);

    assert_eq!(rejected.state_after(), &before);

    assert_eq!(rejected.admission(), None);

    assert_eq!(rejected.goals().selected_count(), 0);
}

#[test]
fn integrated_sparse_cycle_is_deterministic_non_mutating_and_facade_equivalent() {
    let initial = state();

    let initial_before = initial.clone();

    let structure = CognitiveStructure::ordered(vec![atom(91), atom(92)]).unwrap();

    let structure_before = structure.clone();

    let input_profile = profile(700, 400, 500, 600);

    let profile_before = input_profile;

    let cycle_context = context(empty_predictions(), 20);

    let context_before = cycle_context.clone();

    let direct = IntegratedSparseCycle::evaluate(
        initial.clone(),
        1,
        structure.clone(),
        input_profile,
        &cycle_context,
    );

    let facade = MindstoneIntegratedSparseCycle::evaluate(
        initial.clone(),
        1,
        structure.clone(),
        input_profile,
        &cycle_context,
    );

    let repeated = MindstoneIntegratedSparseCycle::evaluate(
        initial.clone(),
        1,
        structure.clone(),
        input_profile,
        &cycle_context,
    );

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(initial, initial_before);

    assert_eq!(structure, structure_before);

    assert_eq!(input_profile, profile_before);

    assert_eq!(cycle_context, context_before);

    assert_eq!(cycle_context.sparse_policy(), sparse_policy());

    assert_eq!(cycle_context.epistemic_policy(), epistemic_policy());
}
