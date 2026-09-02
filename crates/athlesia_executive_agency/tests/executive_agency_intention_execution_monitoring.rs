use athlesia_executive_agency::{
    ArbitratedExecutiveIntent, ExecutableMultiStepIntention, ExecutionDeviationKind,
    ExecutiveAgency, ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveSelectionThresholds,
    ExecutiveUtilityWeights, GoalConflictArbitration, GoalConflictArbitrationPolicy,
    GoalConflictArbitrationThresholds, GroundedExecutionObservation,
    GroundedExecutiveActionCandidate, GroundedIntentionStep, IntentionExecutionMonitor,
    IntentionExecutionMonitoringPolicy, IntentionExecutionStatus, MultiStepIntention,
    MultiStepIntentionCandidate, MultiStepIntentionPolicy, MultiStepIntentionThresholds,
    UniversalIntentionExecutionMonitor,
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

fn goal(identity: u64) -> ExecutiveGoal {
    ExecutiveGoal::new(atom(identity), signal(1000), signal(0))
}

fn executive_candidate(
    goal_identity: u64,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
) -> GroundedExecutiveActionCandidate {
    GroundedExecutiveActionCandidate::new(
        atom(goal_identity),
        action,
        outcome,
        signal(1000),
        signal(1000),
        signal(1000),
        signal(0),
        signal(0),
    )
}

fn agency_policy() -> ExecutiveAgencyPolicy {
    ExecutiveAgencyPolicy::new(
        16,
        16,
        64,
        64,
        ExecutiveUtilityWeights::new(1, 0, 0, 0, 0).unwrap(),
        ExecutiveSelectionThresholds::new(signal(1), signal(1), signal(1), signal(1), signal(1))
            .unwrap(),
    )
    .unwrap()
}

fn arbitration_policy() -> GoalConflictArbitrationPolicy {
    GoalConflictArbitrationPolicy::new(
        16,
        16,
        64,
        16,
        signal(0),
        GoalConflictArbitrationThresholds::new(signal(1), signal(1)).unwrap(),
    )
    .unwrap()
}

fn source_intents(
    action: CognitiveStructure,
    outcome: CognitiveStructure,
) -> Vec<ArbitratedExecutiveIntent> {
    let candidates = vec![executive_candidate(1, action, outcome)];

    let executive = ExecutiveAgency::select(&[goal(1)], &candidates, agency_policy());

    GoalConflictArbitration::arbitrate(executive.selected(), &[], None, arbitration_policy())
        .selected()
        .to_vec()
}

fn grounded_step(
    required_state: CognitiveStructure,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
) -> GroundedIntentionStep {
    GroundedIntentionStep::new(
        required_state,
        action,
        outcome,
        signal(1000),
        signal(1000),
        signal(0),
    )
}

fn intention_policy() -> MultiStepIntentionPolicy {
    MultiStepIntentionPolicy::new(
        16,
        16,
        8,
        64,
        16,
        MultiStepIntentionThresholds::new(signal(1), signal(1), signal(1), signal(1)).unwrap(),
    )
    .unwrap()
}

fn intention_with_structures(
    first_action: CognitiveStructure,
    first_outcome: CognitiveStructure,
    second_action: CognitiveStructure,
    second_outcome: CognitiveStructure,
) -> ExecutableMultiStepIntention {
    let sources = source_intents(first_action.clone(), first_outcome.clone());

    let candidate = MultiStepIntentionCandidate::new(
        atom(1),
        vec![
            grounded_step(atom(500), first_action, first_outcome.clone()),
            grounded_step(first_outcome, second_action, second_outcome),
        ],
        signal(1000),
    )
    .unwrap();

    MultiStepIntention::select(
        &sources,
        std::slice::from_ref(&candidate),
        intention_policy(),
    )
    .selected()[0]
        .clone()
}

fn default_intention() -> ExecutableMultiStepIntention {
    intention_with_structures(atom(10), atom(110), atom(11), atom(111))
}

fn observation(
    state: u64,
    action: u64,
    outcome: u64,
    confidence: u16,
) -> GroundedExecutionObservation {
    GroundedExecutionObservation::new(atom(state), atom(action), atom(outcome), signal(confidence))
}

fn monitoring_policy(
    max_steps: usize,
    max_observations: usize,
    minimum_confidence: u16,
) -> IntentionExecutionMonitoringPolicy {
    IntentionExecutionMonitoringPolicy::new(max_steps, max_observations, signal(minimum_confidence))
        .unwrap()
}

fn default_monitoring_policy() -> IntentionExecutionMonitoringPolicy {
    monitoring_policy(8, 16, 500)
}

#[test]
fn execution_monitoring_policy_requires_positive_bounds_and_confidence() {
    assert_eq!(
        IntentionExecutionMonitoringPolicy::new(0, 1, signal(500,),),
        None
    );

    assert_eq!(
        IntentionExecutionMonitoringPolicy::new(1, 0, signal(500,),),
        None
    );

    assert_eq!(
        IntentionExecutionMonitoringPolicy::new(1, 1, signal(0,),),
        None
    );

    assert!(IntentionExecutionMonitoringPolicy::new(2, 2, signal(500,),).is_some());
}

#[test]
fn absent_execution_observation_leaves_intention_pending_at_first_step() {
    let intention = default_intention();

    let result = IntentionExecutionMonitor::monitor(&intention, &[], default_monitoring_policy());

    assert_eq!(result.status(), IntentionExecutionStatus::Pending);

    assert_eq!(result.confirmed_step_count(), 0);

    assert_eq!(result.next_step_index(), Some(0,));

    assert_eq!(result.remaining_step_count(), 2);
}

#[test]
fn low_confidence_observation_is_retained_as_inconclusive_without_advancing() {
    let intention = default_intention();

    let result = IntentionExecutionMonitor::monitor(
        &intention,
        &[observation(500, 10, 110, 499)],
        default_monitoring_policy(),
    );

    assert_eq!(result.status(), IntentionExecutionStatus::Inconclusive);

    assert_eq!(result.low_confidence_observation_count(), 1);

    assert_eq!(result.confirmed_step_count(), 0);

    assert_eq!(result.next_step_index(), Some(0,));
}

#[test]
fn exact_confident_first_step_observation_advances_to_next_step() {
    let intention = default_intention();

    let result = IntentionExecutionMonitor::monitor(
        &intention,
        &[observation(500, 10, 110, 1000)],
        default_monitoring_policy(),
    );

    assert_eq!(result.status(), IntentionExecutionStatus::Advanced);

    assert_eq!(result.confirmed_step_count(), 1);

    assert_eq!(result.next_step_index(), Some(1,));

    assert_eq!(result.remaining_step_count(), 1);
}

#[test]
fn exact_ordered_observations_complete_entire_multi_step_intention() {
    let intention = default_intention();

    let result = IntentionExecutionMonitor::monitor(
        &intention,
        &[
            observation(500, 10, 110, 1000),
            observation(110, 11, 111, 1000),
        ],
        default_monitoring_policy(),
    );

    assert!(result.completed());

    assert_eq!(result.status(), IntentionExecutionStatus::Completed);

    assert_eq!(result.confirmed_step_count(), 2);

    assert_eq!(result.next_step_index(), None);

    assert_eq!(result.remaining_step_count(), 0);
}

#[test]
fn confident_state_mismatch_creates_explicit_deviation_without_advancing() {
    let intention = default_intention();

    let result = IntentionExecutionMonitor::monitor(
        &intention,
        &[observation(999, 10, 110, 1000)],
        default_monitoring_policy(),
    );

    assert!(result.deviated());

    let deviation = result.deviation().unwrap();

    assert_eq!(deviation.kind(), ExecutionDeviationKind::StateMismatch);

    assert_eq!(deviation.step_index(), 0);

    assert_eq!(deviation.expected_state(), &atom(500,));

    assert_eq!(deviation.observed_state(), &atom(999,));

    assert_eq!(result.confirmed_step_count(), 0);
}

#[test]
fn confident_action_mismatch_creates_explicit_deviation() {
    let intention = default_intention();

    let result = IntentionExecutionMonitor::monitor(
        &intention,
        &[observation(500, 99, 110, 1000)],
        default_monitoring_policy(),
    );

    let deviation = result.deviation().unwrap();

    assert_eq!(deviation.kind(), ExecutionDeviationKind::ActionMismatch);

    assert_eq!(deviation.expected_action(), &atom(10,));

    assert_eq!(deviation.observed_action(), &atom(99,));

    assert_eq!(result.next_step_index(), Some(0,));
}

#[test]
fn confident_outcome_mismatch_creates_explicit_prediction_error() {
    let intention = default_intention();

    let result = IntentionExecutionMonitor::monitor(
        &intention,
        &[observation(500, 10, 999, 1000)],
        default_monitoring_policy(),
    );

    let deviation = result.deviation().unwrap();

    assert_eq!(deviation.kind(), ExecutionDeviationKind::OutcomeMismatch);

    assert_eq!(deviation.expected_outcome(), &atom(110,));

    assert_eq!(deviation.observed_outcome(), &atom(999,));

    assert_eq!(deviation.observation_confidence(), signal(1000,));
}

#[test]
fn first_confident_deviation_halts_monitoring_and_later_match_cannot_rescue_plan() {
    let intention = default_intention();

    let result = IntentionExecutionMonitor::monitor(
        &intention,
        &[
            observation(500, 10, 999, 1000),
            observation(500, 10, 110, 1000),
            observation(110, 11, 111, 1000),
        ],
        default_monitoring_policy(),
    );

    assert!(result.deviated());

    assert_eq!(result.considered_observation_count(), 1);

    assert_eq!(result.confirmed_step_count(), 0);

    assert_eq!(result.next_step_index(), Some(0,));
}

#[test]
fn exact_opaque_execution_identity_rejects_reordered_structures() {
    let expected_action = ordered(&[10, 11]);

    let reordered_action = ordered(&[11, 10]);

    assert_ne!(expected_action, reordered_action);

    let intention = intention_with_structures(expected_action, atom(110), atom(12), atom(111));

    let observation =
        GroundedExecutionObservation::new(atom(500), reordered_action, atom(110), signal(1000));

    let result = IntentionExecutionMonitor::monitor(
        &intention,
        std::slice::from_ref(&observation),
        default_monitoring_policy(),
    );

    assert!(result.deviated());

    assert_eq!(
        result.deviation().unwrap().kind(),
        ExecutionDeviationKind::ActionMismatch
    );
}

#[test]
fn hard_step_and_observation_frontiers_are_enforced_without_hidden_progress() {
    let intention = default_intention();

    let step_limited = IntentionExecutionMonitor::monitor(
        &intention,
        &[observation(500, 10, 110, 1000)],
        monitoring_policy(1, 16, 500),
    );

    assert_eq!(
        step_limited.status(),
        IntentionExecutionStatus::StepBoundExceeded
    );

    assert_eq!(step_limited.considered_observation_count(), 0);

    assert_eq!(step_limited.confirmed_step_count(), 0);

    let observation_limited = IntentionExecutionMonitor::monitor(
        &intention,
        &[
            observation(500, 10, 110, 1000),
            observation(110, 11, 111, 1000),
        ],
        monitoring_policy(8, 1, 500),
    );

    assert!(observation_limited.observation_frontier_truncated());

    assert_eq!(observation_limited.considered_observation_count(), 1);

    assert_eq!(observation_limited.confirmed_step_count(), 1);

    assert_eq!(
        observation_limited.status(),
        IntentionExecutionStatus::Advanced
    );
}

#[test]
fn execution_monitoring_is_deterministic_non_mutating_and_facade_equivalent() {
    let intention = default_intention();

    let observations = vec![
        observation(500, 10, 110, 1000),
        observation(110, 11, 111, 1000),
    ];

    let intention_before = intention.clone();

    let observations_before = observations.clone();

    let policy = default_monitoring_policy();

    let direct = IntentionExecutionMonitor::monitor(&intention, &observations, policy);

    let facade = UniversalIntentionExecutionMonitor::evaluate(&intention, &observations, policy);

    let repeated = UniversalIntentionExecutionMonitor::evaluate(&intention, &observations, policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(intention, intention_before);

    assert_eq!(observations, observations_before);
}
