use athlesia_mindstone_sparse_cognition::{
    BoundedHypothesisSearchNode, BoundedHypothesisSearchPolicy, CausalControllabilityEvidence,
    CognitiveBudget, CognitiveFingerprint, CognitiveSignal, CognitiveStructure,
    CollisionSafeGoalInformationPrediction, CollisionSafeGoalInformationPredictionSet,
    CollisionSafeStructuralIdentity, EpistemicOutcomePrediction, EpistemicSelfPolicy,
    IntegratedSparseCycleContext, IntegratedSparseCycleState, IntegratedSparseCycleStatus,
    MindstoneExtendedSignalProfile, MindstoneSignalProfile, MindstoneSparseMindstoneFinal,
    SelfGeneratedGoalPolicy, SparseCognitionPolicy, SparseMindstoneFinalContext,
    SparseMindstoneFinalInput, SparseMindstoneFinalOrchestrator,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn profile(
    surprise: u16,
    uncertainty: u16,
    learning_progress: u16,
    information_gain: u16,
    compression_gain: u16,
    raw_controllability: u16,
) -> MindstoneExtendedSignalProfile {
    MindstoneExtendedSignalProfile::new(
        MindstoneSignalProfile::new(
            signal(surprise),
            signal(uncertainty),
            signal(0),
            signal(learning_progress),
            signal(information_gain),
        ),
        signal(compression_gain),
        signal(raw_controllability),
    )
}

fn sparse_policy() -> SparseCognitionPolicy {
    SparseCognitionPolicy::new(signal(200), signal(600), 2, 8).unwrap()
}

fn epistemic_policy() -> EpistemicSelfPolicy {
    EpistemicSelfPolicy::new(signal(200), signal(300), signal(400), signal(600), 2).unwrap()
}

fn goal_policy() -> SelfGeneratedGoalPolicy {
    SelfGeneratedGoalPolicy::new(4, 2, 3, 4, 2).unwrap()
}

fn hypothesis_policy() -> BoundedHypothesisSearchPolicy {
    BoundedHypothesisSearchPolicy::new(4, 4, 3).unwrap()
}

fn budget(units: u32) -> CognitiveBudget {
    CognitiveBudget::new(units).unwrap()
}

fn evidence(
    intervention_successes: u64,
    intervention_attempts: u64,
    passive_successes: u64,
    passive_attempts: u64,
) -> CausalControllabilityEvidence {
    CausalControllabilityEvidence::new(
        intervention_successes,
        intervention_attempts,
        passive_successes,
        passive_attempts,
    )
    .unwrap()
}

fn outcome(weight: u32, uncertainty: u16) -> EpistemicOutcomePrediction {
    EpistemicOutcomePrediction::new(weight, signal(uncertainty)).unwrap()
}

fn prediction(
    structure: CognitiveStructure,
    outcomes: Vec<EpistemicOutcomePrediction>,
) -> CollisionSafeGoalInformationPrediction {
    CollisionSafeGoalInformationPrediction::new(
        CollisionSafeStructuralIdentity::from_structure(structure),
        outcomes,
    )
    .unwrap()
}

fn prediction_set(
    values: Vec<CollisionSafeGoalInformationPrediction>,
) -> CollisionSafeGoalInformationPredictionSet {
    CollisionSafeGoalInformationPredictionSet::new(values).unwrap()
}

fn empty_predictions() -> CollisionSafeGoalInformationPredictionSet {
    CollisionSafeGoalInformationPredictionSet::empty()
}

fn context(
    predictions: CollisionSafeGoalInformationPredictionSet,
    hard_units: u32,
) -> SparseMindstoneFinalContext {
    SparseMindstoneFinalContext::new(
        IntegratedSparseCycleContext::new(
            predictions,
            sparse_policy(),
            epistemic_policy(),
            goal_policy(),
            budget(hard_units),
        ),
        hypothesis_policy(),
    )
}

fn state() -> IntegratedSparseCycleState {
    IntegratedSparseCycleState::new(16, 16).unwrap()
}

fn hypothesis(
    value: u64,
    score: u16,
    cost: u32,
    depth: u16,
    path_length: usize,
) -> BoundedHypothesisSearchNode {
    BoundedHypothesisSearchNode::new(
        CollisionSafeStructuralIdentity::from_structure(atom(value)),
        signal(score),
        cost,
        depth,
        path_length,
    )
    .unwrap()
}

#[test]
fn final_context_preserves_cycle_budget_prediction_and_hypothesis_bounds() {
    let structure = atom(1);

    let final_context = context(
        prediction_set(vec![prediction(structure, vec![outcome(1, 100)])]),
        32,
    );

    assert_eq!(final_context.cycle_context().budget().units(), 32);

    assert_eq!(final_context.cycle_context().predictions().len(), 1);

    assert_eq!(final_context.hypothesis_policy().max_hypotheses(), 4);

    assert_eq!(final_context.hypothesis_policy().max_depth(), 3);
}

#[test]
fn missing_causal_evidence_suppresses_unverified_raw_controllability() {
    let final_context = context(empty_predictions(), 20);

    let input = SparseMindstoneFinalInput::from_structure(
        atom(10),
        profile(0, 500, 0, 0, 0, 900),
        None,
        Vec::new(),
    );

    let result = SparseMindstoneFinalOrchestrator::evaluate(state(), 1, input, &final_context);

    assert!(result.causal_estimate().is_none());

    assert_eq!(
        result.corrected_profile().controllability(),
        CognitiveSignal::zero()
    );

    assert!(result.processed());
}

#[test]
fn causal_evidence_replaces_raw_control_with_baseline_corrected_lift() {
    let final_context = context(empty_predictions(), 20);

    let result = SparseMindstoneFinalOrchestrator::evaluate(
        state(),
        1,
        SparseMindstoneFinalInput::from_structure(
            atom(11),
            profile(0, 500, 0, 0, 0, 950),
            Some(evidence(9, 10, 3, 10)),
            Vec::new(),
        ),
        &final_context,
    );

    let estimate = result.causal_estimate().unwrap();

    assert_eq!(estimate.intervention_rate().value(), 900);

    assert_eq!(estimate.passive_rate().value(), 300);

    assert_eq!(result.corrected_profile().controllability().value(), 600);
}

#[test]
fn spurious_equal_baseline_control_activates_no_adaptive_expensive_compute() {
    let final_context = context(empty_predictions(), 20);

    let result = SparseMindstoneFinalOrchestrator::evaluate(
        state(),
        1,
        SparseMindstoneFinalInput::from_structure(
            atom(12),
            profile(0, 500, 0, 0, 0, 900),
            Some(evidence(9, 10, 9, 10)),
            vec![hypothesis(100, 900, 1, 0, 1)],
        ),
        &final_context,
    );

    assert_eq!(
        result.corrected_profile().controllability(),
        CognitiveSignal::zero()
    );

    let allocation = result.allocation().unwrap();

    assert!(allocation.is_idle());

    assert_eq!(allocation.activated_units(), 0);

    assert_eq!(result.final_goals().selected_count(), 0);

    assert!(result.hypothesis_search().is_none());
}

#[test]
fn forced_hash_collision_remains_separate_through_final_orchestration() {
    let shared = CognitiveFingerprint::new(777);

    let final_context = context(empty_predictions(), 20);

    let first = SparseMindstoneFinalOrchestrator::evaluate(
        state(),
        1,
        SparseMindstoneFinalInput::with_fingerprint_hint(
            shared,
            atom(1),
            profile(0, 800, 0, 0, 0, 0),
            None,
            Vec::new(),
        ),
        &final_context,
    );

    let second = SparseMindstoneFinalOrchestrator::evaluate(
        first.cycle().state_after().clone(),
        2,
        SparseMindstoneFinalInput::with_fingerprint_hint(
            shared,
            atom(2),
            profile(0, 800, 0, 0, 0, 0),
            None,
            Vec::new(),
        ),
        &final_context,
    );

    assert_eq!(
        second
            .cycle()
            .state_after()
            .structural()
            .bucket_len(shared,),
        2
    );

    assert_eq!(
        second.cycle().state_after().epistemic().bucket_len(shared,),
        2
    );
}

#[test]
fn final_goal_frontier_uses_expected_information_gain_and_adaptive_goal_budget() {
    let first_structure = atom(21);

    let second_structure = atom(22);

    let final_context = context(
        prediction_set(vec![
            prediction(first_structure.clone(), vec![outcome(1, 800)]),
            prediction(second_structure.clone(), vec![outcome(1, 0)]),
        ]),
        20,
    );

    let first = SparseMindstoneFinalOrchestrator::evaluate(
        state(),
        1,
        SparseMindstoneFinalInput::from_structure(
            first_structure,
            profile(0, 900, 0, 0, 0, 0),
            None,
            Vec::new(),
        ),
        &final_context,
    );

    let second = SparseMindstoneFinalOrchestrator::evaluate(
        first.cycle().state_after().clone(),
        2,
        SparseMindstoneFinalInput::from_structure(
            second_structure.clone(),
            profile(0, 700, 0, 0, 0, 0),
            None,
            Vec::new(),
        ),
        &final_context,
    );

    let allocation = second.allocation().unwrap();

    assert!(allocation.goal_units() > 0);

    assert_eq!(
        second.final_goals().selected()[0].structure(),
        &second_structure
    );

    assert_eq!(
        second.final_goals().selected()[0]
            .information_gain()
            .value(),
        700
    );

    assert!(second.final_goals().total_selected_cost() <= allocation.goal_units());
}

#[test]
fn final_hypothesis_search_obeys_adaptive_hypothesis_compute_allocation() {
    let final_context = context(empty_predictions(), 20);

    let result = SparseMindstoneFinalOrchestrator::evaluate(
        state(),
        1,
        SparseMindstoneFinalInput::from_structure(
            atom(30),
            profile(1000, 500, 0, 0, 0, 0),
            None,
            vec![
                hypothesis(101, 900, 2, 0, 1),
                hypothesis(102, 800, 2, 1, 2),
                hypothesis(103, 700, 2, 2, 3),
            ],
        ),
        &final_context,
    );

    let allocation = result.allocation().unwrap();

    assert_eq!(allocation.goal_units(), 0);

    assert!(allocation.hypothesis_units() > 0);

    let search = result.hypothesis_search().unwrap();

    assert!(search.total_selected_cost() <= allocation.hypothesis_units());

    assert_eq!(search.selected_count(), 3);
}

#[test]
fn zero_hypothesis_pressure_skips_search_even_when_hypotheses_are_available() {
    let structure = atom(40);

    let final_context = context(
        prediction_set(vec![prediction(structure.clone(), vec![outcome(1, 0)])]),
        20,
    );

    let result = SparseMindstoneFinalOrchestrator::evaluate(
        state(),
        1,
        SparseMindstoneFinalInput::from_structure(
            structure,
            profile(0, 1000, 0, 0, 0, 0),
            None,
            vec![hypothesis(200, 1000, 1, 0, 1)],
        ),
        &final_context,
    );

    let allocation = result.allocation().unwrap();

    assert_eq!(allocation.hypothesis_units(), 0);

    assert!(allocation.goal_units() > 0);

    assert!(result.hypothesis_search().is_none());
}

#[test]
fn final_hypothesis_path_preserves_conservative_unaffordable_next_stop() {
    let final_context = context(empty_predictions(), 12);

    let result = SparseMindstoneFinalOrchestrator::evaluate(
        state(),
        1,
        SparseMindstoneFinalInput::from_structure(
            atom(50),
            profile(1000, 500, 0, 0, 0, 0),
            None,
            vec![
                hypothesis(1, 1000, 2, 0, 1),
                hypothesis(2, 900, 5, 0, 1),
                hypothesis(3, 800, 1, 0, 1),
            ],
        ),
        &final_context,
    );

    let allocation = result.allocation().unwrap();

    assert_eq!(allocation.hypothesis_units(), 4);

    let search = result.hypothesis_search().unwrap();

    assert_eq!(search.selected_count(), 1);

    assert_eq!(search.selected()[0].structure(), &atom(1,));

    assert!(search.truncated_by_compute_budget());

    assert!(!search
        .selected()
        .iter()
        .any(|candidate| { candidate.structure() == &atom(3,) },));
}

#[test]
fn out_of_order_final_cycle_rejects_without_adaptive_goal_or_search_execution() {
    let final_context = context(empty_predictions(), 20);

    let first = SparseMindstoneFinalOrchestrator::evaluate(
        state(),
        10,
        SparseMindstoneFinalInput::from_structure(
            atom(60),
            profile(1000, 800, 0, 0, 0, 0),
            None,
            Vec::new(),
        ),
        &final_context,
    );

    let before = first.cycle().state_after().clone();

    let rejected = SparseMindstoneFinalOrchestrator::evaluate(
        before.clone(),
        10,
        SparseMindstoneFinalInput::from_structure(
            atom(61),
            profile(1000, 1000, 1000, 1000, 1000, 1000),
            Some(evidence(10, 10, 0, 10)),
            vec![hypothesis(900, 1000, 1, 0, 1)],
        ),
        &final_context,
    );

    assert_eq!(
        rejected.cycle().status(),
        IntegratedSparseCycleStatus::RejectedOutOfOrder
    );

    assert_eq!(rejected.cycle().state_after(), &before);

    assert!(rejected.allocation().is_none());

    assert_eq!(rejected.final_goals().selected_count(), 0);

    assert!(rejected.hypothesis_search().is_none());
}

#[test]
fn ignored_repeat_skips_adaptive_expensive_goal_and_hypothesis_paths() {
    let structure = atom(70);

    let final_context = context(empty_predictions(), 20);

    let first = SparseMindstoneFinalOrchestrator::evaluate(
        state(),
        1,
        SparseMindstoneFinalInput::from_structure(
            structure.clone(),
            profile(0, 900, 0, 0, 0, 0),
            None,
            Vec::new(),
        ),
        &final_context,
    );

    let second = SparseMindstoneFinalOrchestrator::evaluate(
        first.cycle().state_after().clone(),
        2,
        SparseMindstoneFinalInput::from_structure(
            structure,
            profile(0, 0, 0, 0, 0, 0),
            None,
            vec![hypothesis(500, 1000, 1, 0, 1)],
        ),
        &final_context,
    );

    assert_eq!(
        second.cycle().status(),
        IntegratedSparseCycleStatus::Ignored
    );

    assert!(second.allocation().is_none());

    assert_eq!(second.final_goals().selected_count(), 0);

    assert!(second.hypothesis_search().is_none());
}

#[test]
fn final_orchestrator_is_deterministic_non_mutating_hard_capped_and_facade_equivalent() {
    let initial = state();

    let initial_before = initial.clone();

    let structure = CognitiveStructure::ordered(vec![atom(91), atom(92)]).unwrap();

    let final_context = context(
        prediction_set(vec![prediction(structure.clone(), vec![outcome(1, 100)])]),
        32,
    );

    let context_before = final_context.clone();

    let input = SparseMindstoneFinalInput::from_structure(
        structure,
        profile(700, 800, 400, 0, 500, 900),
        Some(evidence(8, 10, 3, 10)),
        vec![
            hypothesis(1001, 900, 2, 0, 1),
            hypothesis(1002, 800, 2, 1, 2),
        ],
    );

    let input_before = input.clone();

    let direct = SparseMindstoneFinalOrchestrator::evaluate(
        initial.clone(),
        1,
        input.clone(),
        &final_context,
    );

    let facade =
        MindstoneSparseMindstoneFinal::evaluate(initial.clone(), 1, input.clone(), &final_context);

    let repeated =
        MindstoneSparseMindstoneFinal::evaluate(initial.clone(), 1, input.clone(), &final_context);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(initial, initial_before);

    assert_eq!(input, input_before);

    assert_eq!(final_context, context_before);

    let allocation = facade.allocation().unwrap();

    assert_eq!(
        allocation.total_accounted_units(),
        final_context.cycle_context().budget().units()
    );

    assert!(facade.final_goals().total_selected_cost() <= allocation.goal_units());

    if let Some(search) = facade.hypothesis_search() {
        assert!(search.total_selected_cost() <= allocation.hypothesis_units());
    }
}
