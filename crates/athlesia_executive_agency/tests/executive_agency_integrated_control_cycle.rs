use athlesia_executive_agency::{
    ArbitratedExecutiveIntent, ContinuationAssessment, DeviationReplanner,
    DeviationReplanningPolicy, DeviationReplanningThresholds, ExecutableMultiStepIntention,
    ExecutiveAgency, ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveSelectionThresholds,
    ExecutiveUtilityWeights, ExplorationExploitationPolicy, ExplorationExploitationThresholds,
    ExplorationSignals, GoalConflictArbitration, GoalConflictArbitrationPolicy,
    GoalConflictArbitrationThresholds, GroundedExecutionObservation,
    GroundedExecutiveActionCandidate, GroundedExplorationCandidate, GroundedIntentionStep,
    IntegratedExecutiveControl, IntegratedExecutiveControlContext,
    IntegratedExecutiveControlDecision, IntegratedExecutiveControlPolicy,
    IntegratedExecutiveSelectionSource, IntentionExecutionMonitor,
    IntentionExecutionMonitoringPolicy, MultiStepIntention, MultiStepIntentionCandidate,
    MultiStepIntentionPolicy, MultiStepIntentionThresholds, ReconsiderationState,
    StopReconsiderationPolicy, UniversalIntegratedExecutiveControl,
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
        32,
        32,
        128,
        128,
        ExecutiveUtilityWeights::new(1, 0, 0, 0, 0).unwrap(),
        ExecutiveSelectionThresholds::new(signal(1), signal(1), signal(1), signal(1), signal(1))
            .unwrap(),
    )
    .unwrap()
}

fn arbitration_policy() -> GoalConflictArbitrationPolicy {
    GoalConflictArbitrationPolicy::new(
        32,
        32,
        128,
        32,
        signal(0),
        GoalConflictArbitrationThresholds::new(signal(1), signal(1)).unwrap(),
    )
    .unwrap()
}

fn intention_policy() -> MultiStepIntentionPolicy {
    MultiStepIntentionPolicy::new(
        32,
        32,
        16,
        256,
        32,
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

fn plan_with(
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
    plan_with(atom(1), atom(500), atom(10), atom(110), atom(11), atom(111))
}

fn monitoring_policy() -> IntentionExecutionMonitoringPolicy {
    IntentionExecutionMonitoringPolicy::new(16, 16, signal(500)).unwrap()
}

fn pending_monitoring(
    intention: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    IntentionExecutionMonitor::monitor(intention, &[], monitoring_policy())
}

fn advanced_monitoring(
    intention: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    let observation =
        GroundedExecutionObservation::new(atom(500), atom(10), atom(110), signal(1000));

    IntentionExecutionMonitor::monitor(
        intention,
        std::slice::from_ref(&observation),
        monitoring_policy(),
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
        monitoring_policy(),
    )
}

fn deviation_monitoring(
    intention: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    let observation =
        GroundedExecutionObservation::new(atom(500), atom(10), atom(999), signal(1000));

    IntentionExecutionMonitor::monitor(
        intention,
        std::slice::from_ref(&observation),
        monitoring_policy(),
    )
}

fn continuation(
    goal_identity: CognitiveStructure,
    progress: u16,
    cost: u16,
) -> ContinuationAssessment {
    ContinuationAssessment::new(
        goal_identity,
        signal(progress),
        signal(1000),
        signal(1000),
        signal(cost),
    )
}

fn stop_policy(minimum_value: u16) -> StopReconsiderationPolicy {
    StopReconsiderationPolicy::new(3, signal(500), signal(500), signal(minimum_value)).unwrap()
}

fn exploration_policy() -> ExplorationExploitationPolicy {
    ExplorationExploitationPolicy::new(
        16,
        16,
        ExplorationExploitationThresholds::new(
            signal(500),
            signal(500),
            signal(100),
            signal(100),
            signal(100),
        )
        .unwrap(),
    )
    .unwrap()
}

fn integrated_policy(minimum_continuation_value: u16) -> IntegratedExecutiveControlPolicy {
    IntegratedExecutiveControlPolicy::new(
        stop_policy(minimum_continuation_value),
        exploration_policy(),
    )
}

fn exploration(
    goal_identity: CognitiveStructure,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
    information_gain: u16,
) -> GroundedExplorationCandidate {
    GroundedExplorationCandidate::new(
        goal_identity,
        action,
        outcome,
        ExplorationSignals::new(
            signal(information_gain),
            signal(1000),
            signal(1000),
            signal(1000),
            signal(0),
        ),
    )
}

fn replanning_policy() -> DeviationReplanningPolicy {
    DeviationReplanningPolicy::new(
        16,
        16,
        16,
        16,
        DeviationReplanningThresholds::new(signal(500), signal(1), signal(1)).unwrap(),
    )
    .unwrap()
}

fn replacement_replanning(
    prior: &ExecutableMultiStepIntention,
    monitoring: &athlesia_executive_agency::IntentionExecutionMonitoringResult,
) -> athlesia_executive_agency::DeviationReplanningResult {
    let replacement = plan_with(atom(1), atom(999), atom(20), atom(120), atom(21), atom(121));

    DeviationReplanner::replan(
        prior,
        monitoring,
        std::slice::from_ref(&replacement),
        replanning_policy(),
    )
}

#[test]
fn integrated_policy_preserves_bounded_stop_and_exploration_subpolicies() {
    let policy = integrated_policy(100);

    assert_eq!(
        policy.stop_reconsideration().max_reconsideration_cycles(),
        3
    );

    assert_eq!(
        policy
            .exploration_exploitation()
            .max_exploration_candidates(),
        16
    );

    assert_eq!(
        policy
            .exploration_exploitation()
            .max_candidate_evaluations(),
        16
    );
}

#[test]
fn satisfied_goal_stops_before_exploration_is_considered() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 1000);

    let current_continuation = continuation(atom(1), 1000, 0);

    let candidate = exploration(atom(1), atom(20), atom(120), 1000);

    let context = IntegratedExecutiveControlContext::new(
        &current_goal,
        Some(&intention),
        &monitoring,
        None,
        Some(&current_continuation),
        ReconsiderationState::default(),
        std::slice::from_ref(&candidate),
    );

    let result = IntegratedExecutiveControl::evaluate(context, integrated_policy(100));

    assert_eq!(result.decision(), IntegratedExecutiveControlDecision::Stop);

    assert!(result.should_stop());

    assert_eq!(result.exploration_exploitation(), None);

    assert_eq!(result.selection(), None);
}

#[test]
fn inconclusive_execution_reconsiders_before_exploration_or_action_selection() {
    let intention = default_plan();

    let monitoring = inconclusive_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let current_continuation = continuation(atom(1), 1000, 0);

    let candidate = exploration(atom(1), atom(20), atom(120), 1000);

    let result = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            std::slice::from_ref(&candidate),
        ),
        integrated_policy(100),
    );

    assert_eq!(
        result.decision(),
        IntegratedExecutiveControlDecision::Reconsider
    );

    assert!(result.should_reconsider());

    assert_eq!(result.exploration_exploitation(), None);
}

#[test]
fn pending_viable_intention_executes_exact_first_step_without_exploration() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let current_continuation = continuation(atom(1), 600, 0);

    let result = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            &[],
        ),
        integrated_policy(100),
    );

    assert_eq!(
        result.decision(),
        IntegratedExecutiveControlDecision::ExecuteCurrent
    );

    assert!(result.should_execute());

    let selection = result.selection().unwrap();

    assert_eq!(
        selection.source(),
        IntegratedExecutiveSelectionSource::CurrentIntention
    );

    assert_eq!(selection.intention_step_index(), Some(0,));

    assert_eq!(selection.action(), &atom(10,));

    assert_eq!(selection.predicted_outcome(), &atom(110,));
}

#[test]
fn advanced_intention_executes_next_unconfirmed_step_not_first_step_again() {
    let intention = default_plan();

    let monitoring = advanced_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let current_continuation = continuation(atom(1), 600, 0);

    let result = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            &[],
        ),
        integrated_policy(100),
    );

    assert_eq!(
        result.decision(),
        IntegratedExecutiveControlDecision::ExecuteCurrent
    );

    let selection = result.selection().unwrap();

    assert_eq!(selection.intention_step_index(), Some(1,));

    assert_eq!(selection.action(), &atom(11,));

    assert_eq!(selection.predicted_outcome(), &atom(111,));
}

#[test]
fn validated_deviation_replan_executes_replacement_from_recovery_state() {
    let intention = default_plan();

    let monitoring = deviation_monitoring(&intention);

    let replanning = replacement_replanning(&intention, &monitoring);

    let current_goal = goal(atom(1), 0);

    let result = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            Some(&replanning),
            None,
            ReconsiderationState::default(),
            &[],
        ),
        integrated_policy(100),
    );

    assert_eq!(
        result.decision(),
        IntegratedExecutiveControlDecision::ExecuteReplacement
    );

    let selection = result.selection().unwrap();

    assert_eq!(
        selection.source(),
        IntegratedExecutiveSelectionSource::ReplacementIntention
    );

    assert_eq!(selection.intention_step_index(), Some(0,));

    assert_eq!(selection.action(), &atom(20,));

    assert_eq!(selection.predicted_outcome(), &atom(120,));
}

#[test]
fn decisive_learning_advantage_selects_exact_grounded_exploration_action() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let current_continuation = continuation(atom(1), 600, 0);

    let candidate = exploration(atom(1), atom(20), atom(120), 800);

    let result = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            std::slice::from_ref(&candidate),
        ),
        integrated_policy(100),
    );

    assert_eq!(
        result.decision(),
        IntegratedExecutiveControlDecision::ExecuteExploration
    );

    let selection = result.selection().unwrap();

    assert_eq!(
        selection.source(),
        IntegratedExecutiveSelectionSource::Exploration
    );

    assert_eq!(selection.intention_step_index(), None);

    assert_eq!(selection.action(), &atom(20,));

    assert_eq!(selection.predicted_outcome(), &atom(120,));

    assert_eq!(selection.control_value(), signal(800,));
}

#[test]
fn exploration_below_advantage_margin_preserves_current_exploitation() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let current_continuation = continuation(atom(1), 600, 0);

    let candidate = exploration(atom(1), atom(20), atom(120), 650);

    let result = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            std::slice::from_ref(&candidate),
        ),
        integrated_policy(100),
    );

    assert_eq!(
        result.decision(),
        IntegratedExecutiveControlDecision::ExecuteCurrent
    );

    assert_eq!(result.selection().unwrap().action(), &atom(10,));
}

#[test]
fn low_net_continuation_value_stops_before_exploration_can_override_stop_gate() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let current_continuation = continuation(atom(1), 300, 250);

    let candidate = exploration(atom(1), atom(20), atom(120), 1000);

    let result = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            std::slice::from_ref(&candidate),
        ),
        integrated_policy(100),
    );

    assert_eq!(
        result.stop_reconsideration().net_continuation_value(),
        Some(signal(50,),)
    );

    assert_eq!(result.decision(), IntegratedExecutiveControlDecision::Stop);

    assert_eq!(result.exploration_exploitation(), None);
}

#[test]
fn missing_current_intention_reconsiders_without_manufacturing_action() {
    let actual_intention = default_plan();

    let monitoring = pending_monitoring(&actual_intention);

    let current_goal = goal(atom(1), 0);

    let current_continuation = continuation(atom(1), 1000, 0);

    let result = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            None,
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            &[],
        ),
        integrated_policy(100),
    );

    assert_eq!(
        result.decision(),
        IntegratedExecutiveControlDecision::Reconsider
    );

    assert_eq!(result.selection(), None);

    assert!(!result.should_execute());
}

#[test]
fn integrated_selection_preserves_exact_opaque_action_identity() {
    let exact_action = ordered(&[10, 11]);

    let reordered_action = ordered(&[11, 10]);

    assert_ne!(exact_action, reordered_action);

    let intention = plan_with(
        atom(1),
        atom(500),
        exact_action.clone(),
        atom(110),
        atom(12),
        atom(112),
    );

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let current_continuation = continuation(atom(1), 600, 0);

    let result = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            &[],
        ),
        integrated_policy(100),
    );

    assert_eq!(result.selection().unwrap().action(), &exact_action);

    assert_ne!(result.selection().unwrap().action(), &reordered_action);
}

#[test]
fn integrated_control_is_deterministic_non_mutating_and_facade_equivalent() {
    let intention = default_plan();

    let monitoring = pending_monitoring(&intention);

    let current_goal = goal(atom(1), 0);

    let current_continuation = continuation(atom(1), 500, 0);

    let candidates = vec![
        exploration(atom(1), atom(20), atom(120), 800),
        exploration(atom(1), atom(30), atom(130), 700),
    ];

    let intention_before = intention.clone();

    let monitoring_before = monitoring.clone();

    let continuation_before = current_continuation.clone();

    let candidates_before = candidates.clone();

    let mut reversed = candidates.clone();

    reversed.reverse();

    let policy = integrated_policy(100);

    let direct = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            &candidates,
        ),
        policy,
    );

    let reordered = IntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            &reversed,
        ),
        policy,
    );

    let facade = UniversalIntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            &candidates,
        ),
        policy,
    );

    let repeated = UniversalIntegratedExecutiveControl::evaluate(
        IntegratedExecutiveControlContext::new(
            &current_goal,
            Some(&intention),
            &monitoring,
            None,
            Some(&current_continuation),
            ReconsiderationState::default(),
            &candidates,
        ),
        policy,
    );

    assert_eq!(direct, reordered);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(intention, intention_before);

    assert_eq!(monitoring, monitoring_before);

    assert_eq!(current_continuation, continuation_before);

    assert_eq!(candidates, candidates_before);
}
