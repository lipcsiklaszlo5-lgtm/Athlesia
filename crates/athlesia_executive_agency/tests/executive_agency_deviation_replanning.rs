use athlesia_executive_agency::{
    ArbitratedExecutiveIntent, DeviationReplanner, DeviationReplanningPolicy,
    DeviationReplanningStatus, DeviationReplanningThresholds, ExecutableMultiStepIntention,
    ExecutiveAgency, ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveSelectionThresholds,
    ExecutiveUtilityWeights, GoalConflictArbitration, GoalConflictArbitrationPolicy,
    GoalConflictArbitrationThresholds, GroundedExecutionObservation,
    GroundedExecutiveActionCandidate, GroundedIntentionStep, IntentionExecutionMonitor,
    IntentionExecutionMonitoringPolicy, MultiStepIntention, MultiStepIntentionCandidate,
    MultiStepIntentionPolicy, MultiStepIntentionThresholds, UniversalDeviationReplanner,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone)]
struct PlanSpec {
    goal: CognitiveStructure,
    required_state: CognitiveStructure,
    first_action: CognitiveStructure,
    first_outcome: CognitiveStructure,
    second_action: CognitiveStructure,
    second_outcome: CognitiveStructure,
    terminal_alignment: u16,
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
    let goals = vec![ExecutiveGoal::new(
        goal_identity.clone(),
        signal(1000),
        signal(0),
    )];

    let candidates = vec![GroundedExecutiveActionCandidate::new(
        goal_identity,
        action,
        outcome,
        signal(1000),
        signal(1000),
        signal(1000),
        signal(0),
        signal(0),
    )];

    let executive = ExecutiveAgency::select(&goals, &candidates, agency_policy());

    GoalConflictArbitration::arbitrate(executive.selected(), &[], None, arbitration_policy())
        .selected()
        .to_vec()
}

fn build_plan(spec: &PlanSpec) -> ExecutableMultiStepIntention {
    let sources = source(
        spec.goal.clone(),
        spec.first_action.clone(),
        spec.first_outcome.clone(),
    );

    let candidate = MultiStepIntentionCandidate::new(
        spec.goal.clone(),
        vec![
            GroundedIntentionStep::new(
                spec.required_state.clone(),
                spec.first_action.clone(),
                spec.first_outcome.clone(),
                signal(1000),
                signal(1000),
                signal(0),
            ),
            GroundedIntentionStep::new(
                spec.first_outcome.clone(),
                spec.second_action.clone(),
                spec.second_outcome.clone(),
                signal(1000),
                signal(1000),
                signal(0),
            ),
        ],
        signal(spec.terminal_alignment),
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

fn build_three_step_replacement(
    recovery_state: CognitiveStructure,
) -> ExecutableMultiStepIntention {
    let goal_identity = atom(1);

    let first_action = atom(40);

    let first_outcome = atom(140);

    let sources = source(
        goal_identity.clone(),
        first_action.clone(),
        first_outcome.clone(),
    );

    let candidate = MultiStepIntentionCandidate::new(
        goal_identity,
        vec![
            GroundedIntentionStep::new(
                recovery_state,
                first_action,
                first_outcome.clone(),
                signal(1000),
                signal(1000),
                signal(0),
            ),
            GroundedIntentionStep::new(
                first_outcome,
                atom(41),
                atom(141),
                signal(1000),
                signal(1000),
                signal(0),
            ),
            GroundedIntentionStep::new(
                atom(141),
                atom(42),
                atom(142),
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

fn prior_plan() -> ExecutableMultiStepIntention {
    build_plan(&PlanSpec {
        goal: atom(1),
        required_state: atom(500),
        first_action: atom(10),
        first_outcome: atom(110),
        second_action: atom(11),
        second_outcome: atom(111),
        terminal_alignment: 1000,
    })
}

fn replacement(
    required_state: CognitiveStructure,
    first_action: u64,
    first_outcome: u64,
    terminal_alignment: u16,
) -> ExecutableMultiStepIntention {
    build_plan(&PlanSpec {
        goal: atom(1),
        required_state,
        first_action: atom(first_action),
        first_outcome: atom(first_outcome),
        second_action: atom(first_action + 1),
        second_outcome: atom(first_outcome + 1),
        terminal_alignment,
    })
}

fn different_goal_replacement(recovery_state: CognitiveStructure) -> ExecutableMultiStepIntention {
    build_plan(&PlanSpec {
        goal: atom(2),
        required_state: recovery_state,
        first_action: atom(30),
        first_outcome: atom(130),
        second_action: atom(31),
        second_outcome: atom(131),
        terminal_alignment: 1000,
    })
}

fn monitor_policy() -> IntentionExecutionMonitoringPolicy {
    IntentionExecutionMonitoringPolicy::new(16, 16, signal(500)).unwrap()
}

fn deviation_monitoring(
    prior: &ExecutableMultiStepIntention,
    observed_outcome: CognitiveStructure,
    confidence: u16,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    let observation = GroundedExecutionObservation::new(
        atom(500),
        atom(10),
        observed_outcome,
        signal(confidence),
    );

    IntentionExecutionMonitor::monitor(prior, std::slice::from_ref(&observation), monitor_policy())
}

fn advanced_monitoring(
    prior: &ExecutableMultiStepIntention,
) -> athlesia_executive_agency::IntentionExecutionMonitoringResult {
    let observation =
        GroundedExecutionObservation::new(atom(500), atom(10), atom(110), signal(1000));

    IntentionExecutionMonitor::monitor(prior, std::slice::from_ref(&observation), monitor_policy())
}

fn thresholds(observation: u16, path: u16, score: u16) -> DeviationReplanningThresholds {
    DeviationReplanningThresholds::new(signal(observation), signal(path), signal(score)).unwrap()
}

fn policy(
    max_candidates: usize,
    max_evaluations: usize,
    max_steps: usize,
    max_selected: usize,
    replanning_thresholds: DeviationReplanningThresholds,
) -> DeviationReplanningPolicy {
    DeviationReplanningPolicy::new(
        max_candidates,
        max_evaluations,
        max_steps,
        max_selected,
        replanning_thresholds,
    )
    .unwrap()
}

fn default_policy() -> DeviationReplanningPolicy {
    policy(32, 32, 16, 32, thresholds(500, 1, 1))
}

#[test]
fn deviation_replanning_policy_requires_positive_bounds_and_thresholds() {
    assert_eq!(
        DeviationReplanningThresholds::new(signal(0,), signal(1,), signal(1,),),
        None
    );

    assert_eq!(
        DeviationReplanningPolicy::new(0, 1, 2, 1, thresholds(1, 1, 1,),),
        None
    );

    assert_eq!(
        DeviationReplanningPolicy::new(1, 1, 1, 1, thresholds(1, 1, 1,),),
        None
    );

    assert!(DeviationReplanningPolicy::new(1, 1, 2, 1, thresholds(1, 1, 1,),).is_some());
}

#[test]
fn replanning_does_not_trigger_without_explicit_execution_deviation() {
    let prior = prior_plan();

    let monitoring = advanced_monitoring(&prior);

    let candidate = replacement(atom(110), 20, 120, 1000);

    let result = DeviationReplanner::replan(
        &prior,
        &monitoring,
        std::slice::from_ref(&candidate),
        default_policy(),
    );

    assert_eq!(result.status(), DeviationReplanningStatus::NotTriggered);

    assert_eq!(result.candidate_evaluation_count(), 0);

    assert!(result.abstained());
}

#[test]
fn stricter_replanning_confidence_can_refuse_an_otherwise_detected_deviation() {
    let prior = prior_plan();

    let monitoring = deviation_monitoring(&prior, atom(999), 600);

    let candidate = replacement(atom(999), 20, 120, 1000);

    let result = DeviationReplanner::replan(
        &prior,
        &monitoring,
        std::slice::from_ref(&candidate),
        policy(32, 32, 16, 32, thresholds(700, 1, 1)),
    );

    assert_eq!(
        result.status(),
        DeviationReplanningStatus::EvidenceInsufficient
    );

    assert_eq!(result.candidate_evaluation_count(), 0);
}

#[test]
fn confident_deviation_selects_replacement_anchored_to_observed_outcome() {
    let prior = prior_plan();

    let monitoring = deviation_monitoring(&prior, atom(999), 1000);

    let candidate = replacement(atom(999), 20, 120, 1000);

    let result = DeviationReplanner::replan(
        &prior,
        &monitoring,
        std::slice::from_ref(&candidate),
        default_policy(),
    );

    assert_eq!(
        result.status(),
        DeviationReplanningStatus::ReplacementSelected
    );

    assert!(result.triggered());

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].recovery_state(), &atom(999,));

    assert_eq!(
        result.selected()[0]
            .replacement()
            .first_step()
            .required_state(),
        &atom(999,)
    );
}

#[test]
fn replanning_preserves_exact_original_goal_identity() {
    let prior = prior_plan();

    let monitoring = deviation_monitoring(&prior, atom(999), 1000);

    let wrong_goal = different_goal_replacement(atom(999));

    let result = DeviationReplanner::replan(
        &prior,
        &monitoring,
        std::slice::from_ref(&wrong_goal),
        default_policy(),
    );

    assert_eq!(result.rejected_goal_mismatch_count(), 1);

    assert_eq!(
        result.status(),
        DeviationReplanningStatus::NoViableReplacement
    );
}

#[test]
fn old_expected_suffix_cannot_continue_after_observed_outcome_deviation() {
    let prior = prior_plan();

    let monitoring = deviation_monitoring(&prior, atom(999), 1000);

    let stale_suffix = replacement(atom(110), 11, 111, 1000);

    let result = DeviationReplanner::replan(
        &prior,
        &monitoring,
        std::slice::from_ref(&stale_suffix),
        default_policy(),
    );

    assert_eq!(result.rejected_recovery_anchor_count(), 1);

    assert_eq!(
        result.status(),
        DeviationReplanningStatus::NoViableReplacement
    );

    assert!(result.abstained());
}

#[test]
fn recovery_anchor_uses_exact_opaque_structure_identity() {
    let prior = prior_plan();

    let observed = ordered(&[9, 99]);

    let reordered = ordered(&[99, 9]);

    assert_ne!(observed, reordered);

    let monitoring = deviation_monitoring(&prior, observed, 1000);

    let candidate = replacement(reordered, 20, 120, 1000);

    let result = DeviationReplanner::replan(
        &prior,
        &monitoring,
        std::slice::from_ref(&candidate),
        default_policy(),
    );

    assert_eq!(result.rejected_recovery_anchor_count(), 1);

    assert!(result.abstained());
}

#[test]
fn replacement_path_confidence_remains_threshold_gated() {
    let prior = prior_plan();

    let monitoring = deviation_monitoring(&prior, atom(999), 1000);

    let weak = replacement(atom(999), 20, 120, 400);

    let result = DeviationReplanner::replan(
        &prior,
        &monitoring,
        std::slice::from_ref(&weak),
        policy(32, 32, 16, 32, thresholds(500, 500, 1)),
    );

    assert_eq!(result.rejected_threshold_count(), 1);

    assert_eq!(
        result.status(),
        DeviationReplanningStatus::NoViableReplacement
    );
}

#[test]
fn deviation_observation_confidence_discounts_replan_score() {
    let prior = prior_plan();

    let monitoring = deviation_monitoring(&prior, atom(999), 800);

    let candidate = replacement(atom(999), 20, 120, 1000);

    let result = DeviationReplanner::replan(
        &prior,
        &monitoring,
        std::slice::from_ref(&candidate),
        default_policy(),
    );

    assert_eq!(
        result.selected()[0].deviation_observation_confidence(),
        signal(800,)
    );

    assert_eq!(result.selected()[0].adjusted_replan_score(), signal(800,));
}

#[test]
fn stronger_viable_replacement_outranks_weaker_recovery_plan() {
    let prior = prior_plan();

    let monitoring = deviation_monitoring(&prior, atom(999), 1000);

    let weaker = replacement(atom(999), 20, 120, 700);

    let stronger = replacement(atom(999), 30, 130, 950);

    let result = DeviationReplanner::replan(
        &prior,
        &monitoring,
        &[weaker, stronger],
        policy(32, 32, 16, 1, thresholds(500, 1, 1)),
    );

    assert_eq!(result.admitted_before_frontier(), 2);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(
        result.selected()[0].replacement().first_step().action(),
        &atom(30,)
    );
}

#[test]
fn hard_candidate_evaluation_step_and_final_replan_frontiers_are_enforced() {
    let prior = prior_plan();

    let monitoring = deviation_monitoring(&prior, atom(999), 1000);

    let first = replacement(atom(999), 20, 120, 1000);

    let second = replacement(atom(999), 30, 130, 900);

    let candidate_limited = DeviationReplanner::replan(
        &prior,
        &monitoring,
        &[first.clone(), second.clone()],
        policy(1, 32, 16, 32, thresholds(500, 1, 1)),
    );

    assert_eq!(candidate_limited.unique_candidate_count(), 2);

    assert_eq!(candidate_limited.considered_candidate_count(), 1);

    assert!(candidate_limited.candidate_frontier_truncated());

    let evaluation_limited = DeviationReplanner::replan(
        &prior,
        &monitoring,
        &[first.clone(), second.clone()],
        policy(32, 1, 16, 32, thresholds(500, 1, 1)),
    );

    assert_eq!(evaluation_limited.candidate_evaluation_count(), 1);

    assert!(evaluation_limited.candidate_evaluation_truncated());

    let overlong = build_three_step_replacement(atom(999));

    assert_eq!(overlong.step_count(), 3);

    let step_limited = DeviationReplanner::replan(
        &prior,
        &monitoring,
        std::slice::from_ref(&overlong),
        policy(32, 32, 2, 32, thresholds(500, 1, 1)),
    );

    assert_eq!(step_limited.rejected_step_bound_count(), 1);

    assert_eq!(step_limited.selected_count(), 0);

    let final_limited = DeviationReplanner::replan(
        &prior,
        &monitoring,
        &[first, second],
        policy(32, 32, 16, 1, thresholds(500, 1, 1)),
    );

    assert_eq!(final_limited.admitted_before_frontier(), 2);

    assert_eq!(final_limited.selected_count(), 1);
}

#[test]
fn deviation_replanning_is_order_invariant_non_mutating_and_facade_equivalent() {
    let prior = prior_plan();

    let monitoring = deviation_monitoring(&prior, atom(999), 1000);

    let candidates = vec![
        replacement(atom(999), 20, 120, 900),
        replacement(atom(999), 30, 130, 800),
    ];

    let prior_before = prior.clone();

    let monitoring_before = monitoring.clone();

    let candidates_before = candidates.clone();

    let mut reversed = candidates.clone();

    reversed.reverse();

    let replanning_policy = default_policy();

    let direct = DeviationReplanner::replan(&prior, &monitoring, &candidates, replanning_policy);

    let reversed_result =
        DeviationReplanner::replan(&prior, &monitoring, &reversed, replanning_policy);

    let facade =
        UniversalDeviationReplanner::evaluate(&prior, &monitoring, &candidates, replanning_policy);

    let repeated =
        UniversalDeviationReplanner::evaluate(&prior, &monitoring, &candidates, replanning_policy);

    assert_eq!(direct, reversed_result);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(prior, prior_before);

    assert_eq!(monitoring, monitoring_before);

    assert_eq!(candidates, candidates_before);
}
