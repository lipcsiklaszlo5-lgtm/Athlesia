use athlesia_executive_agency::{
    ArbitratedExecutiveIntent, ContinuationAssessment, DeviationReplanner,
    DeviationReplanningPolicy, DeviationReplanningThresholds, ExecutableMultiStepIntention,
    ExecutiveAgency, ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveSelectionThresholds,
    ExecutiveUtilityWeights, GoalConflictArbitration, GoalConflictArbitrationPolicy,
    GoalConflictArbitrationThresholds, GroundedExecutionObservation,
    GroundedExecutiveActionCandidate, GroundedIntentionStep, IntentionExecutionMonitor,
    IntentionExecutionMonitoringPolicy, MultiStepIntention, MultiStepIntentionCandidate,
    MultiStepIntentionPolicy, MultiStepIntentionThresholds, ReconsiderationState,
    StopReconsideration, StopReconsiderationDecision, StopReconsiderationPolicy,
    UniversalStopReconsideration,
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

fn goal(identity: CognitiveStructure, satisfaction: u16) -> ExecutiveGoal {
    ExecutiveGoal::new(identity, signal(1000), signal(satisfaction))
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

fn source(
    goal_identity: CognitiveStructure,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
) -> Vec<ArbitratedExecutiveIntent> {
    let current_goal = goal(goal_identity.clone(), 0);

    let candidate = GroundedExecutiveActionCandidate::new(
        goal_identity,
        action,
        outcome,
        signal(1000),
        signal(1000),
        signal(1000),
        signal(0),
        signal(0),
    );

    let executive = ExecutiveAgency::select(
        std::slice::from_ref(&current_goal),
        std::slice::from_ref(&candidate),
        agency_policy(),
    );

    GoalConflictArbitration::arbitrate(executive.selected(), &[], None, arbitration_policy())
        .selected()
        .to_vec()
}

fn plan(
    goal_identity: CognitiveStructure,
    required_state: CognitiveStructure,
    first_action: CognitiveStructure,
    first_outcome: CognitiveStructure,
    second_action: CognitiveStructure,
    second_outcome: CognitiveStructure,
) -> ExecutableMultiStepIntention {
    let sources = source(
        goal_identity.clone(),
        first_action.clone(),
        first_outcome.clone(),
    );

    let candidate = MultiStepIntentionCandidate::new(
        goal_identity,
        vec![
            GroundedIntentionStep::new(
                required_state,
                first_action,
                first_outcome.clone(),
                signal(1000),
                signal(1000),
                signal(0),
            ),
            GroundedIntentionStep::new(
                first_outcome,
                second_action,
                second_outcome,
                signal(1000),
                signal(1000),
                signal(0),
            ),
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

fn default_plan() -> ExecutableMultiStepIntention {
    plan(atom(1), atom(500), atom(10), atom(110), atom(11), atom(111))
}

fn monitoring_policy(max_steps: usize, confidence: u16) -> IntentionExecutionMonitoringPolicy {
    IntentionExecutionMonitoringPolicy::new(max_steps, 16, signal(confidence)).unwrap()
}

fn pending_monitoring(
    intention: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    IntentionExecutionMonitor::monitor(intention, &[], monitoring_policy(8, 500))
}

fn advanced_monitoring(
    intention: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    let observation =
        GroundedExecutionObservation::new(atom(500), atom(10), atom(110), signal(1000));

    IntentionExecutionMonitor::monitor(
        intention,
        std::slice::from_ref(&observation),
        monitoring_policy(8, 500),
    )
}

fn inconclusive_monitoring(
    intention: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    let observation =
        GroundedExecutionObservation::new(atom(500), atom(10), atom(110), signal(400));

    IntentionExecutionMonitor::monitor(
        intention,
        std::slice::from_ref(&observation),
        monitoring_policy(8, 500),
    )
}

fn completed_monitoring(
    intention: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    let observations = vec![
        GroundedExecutionObservation::new(atom(500), atom(10), atom(110), signal(1000)),
        GroundedExecutionObservation::new(atom(110), atom(11), atom(111), signal(1000)),
    ];

    IntentionExecutionMonitor::monitor(intention, &observations, monitoring_policy(8, 500))
}

fn deviation_monitoring(
    intention: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    let observation =
        GroundedExecutionObservation::new(atom(500), atom(10), atom(999), signal(1000));

    IntentionExecutionMonitor::monitor(
        intention,
        std::slice::from_ref(&observation),
        monitoring_policy(8, 500),
    )
}

fn step_bound_monitoring(
    intention: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    IntentionExecutionMonitor::monitor(intention, &[], monitoring_policy(1, 500))
}

fn replanning_policy() -> DeviationReplanningPolicy {
    DeviationReplanningPolicy::new(
        16,
        16,
        8,
        16,
        DeviationReplanningThresholds::new(signal(500), signal(1), signal(1)).unwrap(),
    )
    .unwrap()
}

fn replacement_replanning(
    prior: &ExecutableMultiStepIntention,
    monitoring: &athlesia_executive_agency::IntentionExecutionMonitoringResult,
) -> athlesia_executive_agency::DeviationReplanningResult {
    let replacement = plan(atom(1), atom(999), atom(20), atom(120), atom(21), atom(121));

    DeviationReplanner::replan(
        prior,
        monitoring,
        std::slice::from_ref(&replacement),
        replanning_policy(),
    )
}

fn exhausted_replanning(
    prior: &ExecutableMultiStepIntention,
    monitoring: &athlesia_executive_agency::IntentionExecutionMonitoringResult,
) -> athlesia_executive_agency::DeviationReplanningResult {
    DeviationReplanner::replan(prior, monitoring, &[], replanning_policy())
}

fn assessment(
    goal_identity: CognitiveStructure,
    progress: u16,
    evidence: u16,
    control: u16,
    cost: u16,
) -> ContinuationAssessment {
    ContinuationAssessment::new(
        goal_identity,
        signal(progress),
        signal(evidence),
        signal(control),
        signal(cost),
    )
}

fn policy(
    max_reconsideration_cycles: usize,
    evidence: u16,
    control: u16,
    minimum_value: u16,
) -> StopReconsiderationPolicy {
    StopReconsiderationPolicy::new(
        max_reconsideration_cycles,
        signal(evidence),
        signal(control),
        signal(minimum_value),
    )
    .unwrap()
}

fn default_policy() -> StopReconsiderationPolicy {
    policy(3, 500, 500, 100)
}

#[test]
fn stop_reconsideration_policy_requires_positive_bound_and_thresholds() {
    assert_eq!(
        StopReconsiderationPolicy::new(0, signal(1,), signal(1,), signal(1,),),
        None
    );

    assert_eq!(
        StopReconsiderationPolicy::new(1, signal(0,), signal(1,), signal(1,),),
        None
    );

    assert!(StopReconsiderationPolicy::new(1, signal(1,), signal(1,), signal(1,),).is_some());
}

#[test]
fn satisfied_goal_stops_immediately_even_when_continuation_looks_strong() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 1000);

    let continuation = assessment(atom(1), 1000, 1000, 1000, 0);

    let result = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        ReconsiderationState::new(2),
        default_policy(),
    );

    assert_eq!(
        result.decision(),
        StopReconsiderationDecision::StopGoalSatisfied
    );

    assert!(result.should_stop());

    assert_eq!(result.next_reconsideration_cycles(), 2);
}

#[test]
fn strong_grounded_continuation_preserves_current_intention_and_resets_reconsideration() {
    let intention = default_plan();

    let monitoring = advanced_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let continuation = assessment(atom(1), 1000, 1000, 1000, 100);

    let result = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        ReconsiderationState::new(2),
        default_policy(),
    );

    assert_eq!(
        result.decision(),
        StopReconsiderationDecision::ContinueCurrent
    );

    assert!(result.should_continue());

    assert_eq!(result.net_continuation_value(), Some(signal(900,),));

    assert_eq!(result.next_reconsideration_cycles(), 0);
}

#[test]
fn weak_continuation_evidence_causes_reconsideration_without_blind_execution() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let continuation = assessment(atom(1), 1000, 499, 1000, 0);

    let result = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        ReconsiderationState::new(0),
        default_policy(),
    );

    assert_eq!(
        result.decision(),
        StopReconsiderationDecision::ReconsiderWeakEvidence
    );

    assert!(result.should_reconsider());

    assert_eq!(result.next_reconsideration_cycles(), 1);
}

#[test]
fn weak_controllability_causes_reconsideration_even_with_high_expected_progress() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let continuation = assessment(atom(1), 1000, 1000, 499, 0);

    let result = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        ReconsiderationState::default(),
        default_policy(),
    );

    assert_eq!(
        result.decision(),
        StopReconsiderationDecision::ReconsiderWeakControllability
    );

    assert!(result.should_reconsider());
}

#[test]
fn continuation_stops_when_cost_reduces_expected_net_value_below_threshold() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let continuation = assessment(atom(1), 300, 1000, 1000, 250);

    let result = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        ReconsiderationState::default(),
        policy(3, 500, 500, 100),
    );

    assert_eq!(result.net_continuation_value(), Some(signal(50,),));

    assert_eq!(
        result.decision(),
        StopReconsiderationDecision::StopLowContinuationValue
    );

    assert!(result.should_stop());
}

#[test]
fn inconclusive_execution_evidence_reconsiders_before_any_continuation_assessment() {
    let intention = default_plan();

    let monitoring = inconclusive_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let continuation = assessment(atom(1), 1000, 1000, 1000, 0);

    let result = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        ReconsiderationState::default(),
        default_policy(),
    );

    assert_eq!(
        result.decision(),
        StopReconsiderationDecision::ReconsiderInsufficientObservation
    );

    assert_eq!(result.net_continuation_value(), None);
}

#[test]
fn completed_intention_with_unsatisfied_goal_reconsiders_instead_of_claiming_success() {
    let intention = default_plan();

    let monitoring = completed_monitoring(&intention);

    let current_goal = goal(atom(1), 500);

    let result = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        None,
        ReconsiderationState::default(),
        default_policy(),
    );

    assert_eq!(
        result.decision(),
        StopReconsiderationDecision::ReconsiderIntentionCompleted
    );

    assert!(result.should_reconsider());

    assert!(!result.goal_satisfied());
}

#[test]
fn successful_deviation_replanning_continues_exact_selected_replacement() {
    let intention = default_plan();

    let monitoring = deviation_monitoring(&intention);

    let replanning = replacement_replanning(&intention, &monitoring);

    let current_goal = goal(atom(1), 0);

    let result = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        Some(&replanning),
        None,
        ReconsiderationState::new(2),
        default_policy(),
    );

    assert_eq!(
        result.decision(),
        StopReconsiderationDecision::ContinueReplacement
    );

    assert!(result.should_continue());

    assert_eq!(result.next_reconsideration_cycles(), 0);

    let selected = result.selected_replacement().unwrap();

    assert_eq!(selected.goal_identity(), &atom(1,));

    assert_eq!(selected.first_step().required_state(), &atom(999,));

    assert_eq!(selected.first_step().action(), &atom(20,));
}

#[test]
fn exhausted_replanning_reconsiders_until_hard_cycle_limit_then_stops() {
    let intention = default_plan();

    let monitoring = deviation_monitoring(&intention);

    let replanning = exhausted_replanning(&intention, &monitoring);

    let current_goal = goal(atom(1), 0);

    let first = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        Some(&replanning),
        None,
        ReconsiderationState::new(0),
        policy(2, 500, 500, 100),
    );

    assert_eq!(
        first.decision(),
        StopReconsiderationDecision::ReconsiderRecoveryExhausted
    );

    assert_eq!(first.next_reconsideration_cycles(), 1);

    let limited = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        Some(&replanning),
        None,
        ReconsiderationState::new(2),
        policy(2, 500, 500, 100),
    );

    assert_eq!(
        limited.decision(),
        StopReconsiderationDecision::StopReconsiderationLimit
    );

    assert!(limited.should_stop());

    assert_eq!(limited.next_reconsideration_cycles(), 2);
}

#[test]
fn continuation_goal_matching_uses_exact_opaque_structure_identity() {
    let exact_goal = ordered(&[1, 2]);

    let reordered_goal = ordered(&[2, 1]);

    assert_ne!(exact_goal, reordered_goal);

    let intention = plan(
        exact_goal.clone(),
        atom(500),
        atom(10),
        atom(110),
        atom(11),
        atom(111),
    );

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(exact_goal, 0);

    let continuation = assessment(reordered_goal, 1000, 1000, 1000, 0);

    let result = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        ReconsiderationState::default(),
        default_policy(),
    );

    assert_eq!(
        result.decision(),
        StopReconsiderationDecision::ReconsiderGoalMismatch
    );

    assert!(result.should_reconsider());
}

#[test]
fn execution_bound_reconsideration_is_deterministic_non_mutating_and_facade_equivalent() {
    let intention = default_plan();

    let monitoring = step_bound_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let continuation = assessment(atom(1), 1000, 1000, 1000, 0);

    let state = ReconsiderationState::new(1);

    let decision_policy = default_policy();

    let intention_before = intention.clone();

    let monitoring_before = monitoring.clone();

    let continuation_before = continuation.clone();

    let direct = StopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        state,
        decision_policy,
    );

    let facade = UniversalStopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        state,
        decision_policy,
    );

    let repeated = UniversalStopReconsideration::evaluate(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&continuation),
        state,
        decision_policy,
    );

    assert_eq!(
        direct.decision(),
        StopReconsiderationDecision::ReconsiderExecutionBound
    );

    assert_eq!(direct.next_reconsideration_cycles(), 2);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(intention, intention_before);

    assert_eq!(monitoring, monitoring_before);

    assert_eq!(continuation, continuation_before);
}
