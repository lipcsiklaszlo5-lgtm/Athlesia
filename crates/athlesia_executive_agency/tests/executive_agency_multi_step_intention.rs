use athlesia_executive_agency::{
    ArbitratedExecutiveIntent, ExecutiveAgency, ExecutiveAgencyPolicy, ExecutiveGoal,
    ExecutiveSelectionThresholds, ExecutiveUtilityWeights, GoalConflictArbitration,
    GoalConflictArbitrationPolicy, GoalConflictArbitrationThresholds,
    GroundedExecutiveActionCandidate, GroundedIntentionStep, MultiStepIntention,
    MultiStepIntentionCandidate, MultiStepIntentionPolicy, MultiStepIntentionThresholds,
    UniversalMultiStepIntention,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Copy)]
struct StepSignals {
    evidence: u16,
    control: u16,
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

fn goal(identity: u64) -> ExecutiveGoal {
    ExecutiveGoal::new(atom(identity), signal(1000), signal(0))
}

fn candidate(
    goal_identity: u64,
    action: u64,
    outcome: u64,
    alignment: u16,
) -> GroundedExecutiveActionCandidate {
    GroundedExecutiveActionCandidate::new(
        atom(goal_identity),
        atom(action),
        atom(outcome),
        signal(alignment),
        signal(1000),
        signal(1000),
        signal(0),
        signal(0),
    )
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
        256,
        32,
        signal(0),
        GoalConflictArbitrationThresholds::new(signal(1), signal(1)).unwrap(),
    )
    .unwrap()
}

fn source_intents(
    goals: &[ExecutiveGoal],
    candidates: &[GroundedExecutiveActionCandidate],
) -> Vec<ArbitratedExecutiveIntent> {
    let executive = ExecutiveAgency::select(goals, candidates, agency_policy());

    GoalConflictArbitration::arbitrate(executive.selected(), &[], None, arbitration_policy())
        .selected()
        .to_vec()
}

fn step(
    required_state: u64,
    action: u64,
    predicted_outcome: u64,
    values: StepSignals,
) -> GroundedIntentionStep {
    GroundedIntentionStep::new(
        atom(required_state),
        atom(action),
        atom(predicted_outcome),
        signal(values.evidence),
        signal(values.control),
        signal(values.cost),
    )
}

fn default_step_signals() -> StepSignals {
    StepSignals {
        evidence: 1000,
        control: 1000,
        cost: 10,
    }
}

fn plan(
    goal_identity: u64,
    steps: Vec<GroundedIntentionStep>,
    terminal_alignment: u16,
) -> MultiStepIntentionCandidate {
    MultiStepIntentionCandidate::new(atom(goal_identity), steps, signal(terminal_alignment))
        .unwrap()
}

fn valid_plan(
    goal_identity: u64,
    first_action: u64,
    first_outcome: u64,
    second_action: u64,
) -> MultiStepIntentionCandidate {
    plan(
        goal_identity,
        vec![
            step(500, first_action, first_outcome, default_step_signals()),
            step(
                first_outcome,
                second_action,
                first_outcome + 1,
                default_step_signals(),
            ),
        ],
        1000,
    )
}

fn thresholds() -> MultiStepIntentionThresholds {
    MultiStepIntentionThresholds::new(signal(1), signal(1), signal(1), signal(1)).unwrap()
}

fn policy(
    max_sources: usize,
    max_candidates: usize,
    max_steps: usize,
    max_step_evaluations: usize,
    max_selected: usize,
) -> MultiStepIntentionPolicy {
    MultiStepIntentionPolicy::new(
        max_sources,
        max_candidates,
        max_steps,
        max_step_evaluations,
        max_selected,
        thresholds(),
    )
    .unwrap()
}

fn default_policy() -> MultiStepIntentionPolicy {
    policy(32, 32, 8, 256, 32)
}

#[test]
fn multi_step_policy_requires_positive_bounds_thresholds_and_at_least_two_steps() {
    assert_eq!(
        MultiStepIntentionThresholds::new(signal(0,), signal(1,), signal(1,), signal(1,),),
        None
    );

    assert_eq!(
        MultiStepIntentionPolicy::new(1, 1, 1, 1, 1, thresholds(),),
        None
    );

    assert_eq!(
        MultiStepIntentionCandidate::new(
            atom(1,),
            vec![step(500, 10, 110, default_step_signals(),),],
            signal(1000,),
        ),
        None
    );

    assert!(MultiStepIntentionPolicy::new(1, 1, 2, 2, 1, thresholds(),).is_some());
}

#[test]
fn first_step_must_bind_exactly_to_arbitrated_source_action_and_outcome() {
    let sources = source_intents(&[goal(1)], &[candidate(1, 10, 110, 900)]);

    let mismatched = valid_plan(1, 11, 110, 12);

    let result = MultiStepIntention::select(&sources, &[mismatched], default_policy());

    assert_eq!(result.rejected_source_mismatch_count(), 1);

    assert!(result.abstained());
}

#[test]
fn continuation_requires_exact_previous_predicted_outcome_as_next_required_state() {
    let sources = source_intents(&[goal(1)], &[candidate(1, 10, 110, 900)]);

    let broken = plan(
        1,
        vec![
            step(500, 10, 110, default_step_signals()),
            step(999, 11, 111, default_step_signals()),
        ],
        1000,
    );

    let result = MultiStepIntention::select(&sources, &[broken], default_policy());

    assert_eq!(result.rejected_structural_chain_count(), 1);

    assert!(result.abstained());
}

#[test]
fn valid_ordered_multi_step_intention_is_admitted_with_exact_sequence() {
    let sources = source_intents(&[goal(1)], &[candidate(1, 10, 110, 900)]);

    let candidate_plan = valid_plan(1, 10, 110, 11);

    let result = MultiStepIntention::select(
        &sources,
        std::slice::from_ref(&candidate_plan),
        default_policy(),
    );

    assert_eq!(result.selected_count(), 1);

    let selected = &result.selected()[0];

    assert_eq!(selected.goal_identity(), &atom(1,));

    assert_eq!(selected.step_count(), 2);

    assert_eq!(selected.steps(), candidate_plan.steps());

    assert_eq!(selected.first_step().action(), &atom(10,));
}

#[test]
fn weakest_step_evidence_controls_path_confidence_and_can_reject_plan() {
    let sources = source_intents(&[goal(1)], &[candidate(1, 10, 110, 1000)]);

    let weak = plan(
        1,
        vec![
            step(500, 10, 110, default_step_signals()),
            step(
                110,
                11,
                111,
                StepSignals {
                    evidence: 400,
                    control: 1000,
                    cost: 0,
                },
            ),
        ],
        1000,
    );

    let permissive =
        MultiStepIntention::select(&sources, std::slice::from_ref(&weak), default_policy());

    assert_eq!(
        permissive.selected()[0].weakest_step_evidence_confidence(),
        signal(400,)
    );

    assert_eq!(permissive.selected()[0].path_confidence(), signal(400,));

    let strict_thresholds =
        MultiStepIntentionThresholds::new(signal(500), signal(1), signal(1), signal(1)).unwrap();

    let strict_policy =
        MultiStepIntentionPolicy::new(32, 32, 8, 256, 32, strict_thresholds).unwrap();

    let rejected = MultiStepIntention::select(&sources, &[weak], strict_policy);

    assert_eq!(rejected.rejected_threshold_count(), 1);

    assert!(rejected.abstained());
}

#[test]
fn weakest_step_controllability_is_explicit_and_threshold_gated() {
    let sources = source_intents(&[goal(1)], &[candidate(1, 10, 110, 1000)]);

    let candidate_plan = plan(
        1,
        vec![
            step(500, 10, 110, default_step_signals()),
            step(
                110,
                11,
                111,
                StepSignals {
                    evidence: 1000,
                    control: 399,
                    cost: 0,
                },
            ),
        ],
        1000,
    );

    let strict_thresholds =
        MultiStepIntentionThresholds::new(signal(1), signal(400), signal(1), signal(1)).unwrap();

    let strict_policy =
        MultiStepIntentionPolicy::new(32, 32, 8, 256, 32, strict_thresholds).unwrap();

    let result = MultiStepIntention::select(&sources, &[candidate_plan], strict_policy);

    assert_eq!(result.rejected_threshold_count(), 1);

    assert!(result.abstained());
}

#[test]
fn terminal_goal_alignment_remains_part_of_plan_confidence() {
    let sources = source_intents(&[goal(1)], &[candidate(1, 10, 110, 1000)]);

    let lower = plan(
        1,
        vec![
            step(
                500,
                10,
                110,
                StepSignals {
                    cost: 0,
                    ..default_step_signals()
                },
            ),
            step(
                110,
                11,
                111,
                StepSignals {
                    cost: 0,
                    ..default_step_signals()
                },
            ),
        ],
        600,
    );

    let higher = plan(
        1,
        vec![
            step(
                500,
                10,
                110,
                StepSignals {
                    cost: 0,
                    ..default_step_signals()
                },
            ),
            step(
                110,
                12,
                112,
                StepSignals {
                    cost: 0,
                    ..default_step_signals()
                },
            ),
        ],
        900,
    );

    let result = MultiStepIntention::select(&sources, &[lower, higher], policy(32, 32, 8, 256, 1));

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].terminal_goal_alignment(), signal(900,));

    assert_eq!(result.selected()[0].path_confidence(), signal(900,));
}

#[test]
fn execution_cost_can_reverse_otherwise_equal_multi_step_preference() {
    let sources = source_intents(&[goal(1)], &[candidate(1, 10, 110, 1000)]);

    let expensive = plan(
        1,
        vec![
            step(
                500,
                10,
                110,
                StepSignals {
                    cost: 300,
                    ..default_step_signals()
                },
            ),
            step(
                110,
                11,
                111,
                StepSignals {
                    cost: 300,
                    ..default_step_signals()
                },
            ),
        ],
        1000,
    );

    let efficient = plan(
        1,
        vec![
            step(
                500,
                10,
                110,
                StepSignals {
                    cost: 10,
                    ..default_step_signals()
                },
            ),
            step(
                110,
                12,
                112,
                StepSignals {
                    cost: 10,
                    ..default_step_signals()
                },
            ),
        ],
        1000,
    );

    let result =
        MultiStepIntention::select(&sources, &[expensive, efficient], policy(32, 32, 8, 256, 1));

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].steps()[1].action(), &atom(12,));

    assert_eq!(result.selected()[0].execution_cost_penalty(), signal(20,));
}

#[test]
fn reordered_opaque_step_action_identity_does_not_preserve_source_binding() {
    let source_action = ordered(&[10, 11]);

    let reordered_action = ordered(&[11, 10]);

    assert_ne!(source_action, reordered_action);

    let executive_candidate = GroundedExecutiveActionCandidate::new(
        atom(1),
        source_action,
        atom(110),
        signal(1000),
        signal(1000),
        signal(1000),
        signal(0),
        signal(0),
    );

    let sources = source_intents(&[goal(1)], &[executive_candidate]);

    let candidate_plan = MultiStepIntentionCandidate::new(
        atom(1),
        vec![
            GroundedIntentionStep::new(
                atom(500),
                reordered_action,
                atom(110),
                signal(1000),
                signal(1000),
                signal(0),
            ),
            step(
                110,
                12,
                112,
                StepSignals {
                    cost: 0,
                    ..default_step_signals()
                },
            ),
        ],
        signal(1000),
    )
    .unwrap();

    let result = MultiStepIntention::select(&sources, &[candidate_plan], default_policy());

    assert_eq!(result.rejected_source_mismatch_count(), 1);

    assert!(result.abstained());
}

#[test]
fn overlong_candidate_is_rejected_before_step_evaluation() {
    let sources = source_intents(&[goal(1)], &[candidate(1, 10, 110, 900)]);

    let candidate_plan = plan(
        1,
        vec![
            step(500, 10, 110, default_step_signals()),
            step(110, 11, 111, default_step_signals()),
            step(111, 12, 112, default_step_signals()),
        ],
        1000,
    );

    let result =
        MultiStepIntention::select(&sources, &[candidate_plan], policy(32, 32, 2, 256, 32));

    assert_eq!(result.rejected_over_step_bound_count(), 1);

    assert_eq!(result.step_evaluation_count(), 0);

    assert!(result.abstained());
}

#[test]
fn hard_source_candidate_step_evaluation_and_final_frontiers_are_enforced() {
    let sources = source_intents(
        &[goal(1), goal(2)],
        &[candidate(1, 10, 110, 900), candidate(2, 20, 120, 800)],
    );

    let source_limited = MultiStepIntention::select(
        &sources,
        &[valid_plan(1, 10, 110, 11), valid_plan(2, 20, 120, 21)],
        policy(1, 32, 8, 256, 32),
    );

    assert_eq!(source_limited.considered_source_intent_count(), 1);

    assert!(source_limited.source_frontier_truncated());

    assert_eq!(source_limited.rejected_source_mismatch_count(), 1);

    let candidate_limited = MultiStepIntention::select(
        &sources[0..1],
        &[
            plan(
                1,
                vec![
                    step(
                        500,
                        10,
                        110,
                        StepSignals {
                            cost: 0,
                            ..default_step_signals()
                        },
                    ),
                    step(
                        110,
                        11,
                        111,
                        StepSignals {
                            cost: 0,
                            ..default_step_signals()
                        },
                    ),
                ],
                1000,
            ),
            plan(
                1,
                vec![
                    step(
                        500,
                        10,
                        110,
                        StepSignals {
                            cost: 50,
                            ..default_step_signals()
                        },
                    ),
                    step(
                        110,
                        12,
                        112,
                        StepSignals {
                            cost: 50,
                            ..default_step_signals()
                        },
                    ),
                ],
                900,
            ),
        ],
        policy(32, 1, 8, 256, 32),
    );

    assert_eq!(candidate_limited.unique_candidate_count(), 2);

    assert_eq!(candidate_limited.considered_candidate_count(), 1);

    assert!(candidate_limited.candidate_frontier_truncated());

    let step_limited = MultiStepIntention::select(
        &sources[0..1],
        &[valid_plan(1, 10, 110, 11), valid_plan(1, 10, 110, 12)],
        policy(32, 32, 8, 2, 32),
    );

    assert_eq!(step_limited.step_evaluation_count(), 2);

    assert!(step_limited.step_evaluation_truncated());

    let final_limited = MultiStepIntention::select(
        &sources[0..1],
        &[valid_plan(1, 10, 110, 11), valid_plan(1, 10, 110, 12)],
        policy(32, 32, 8, 256, 1),
    );

    assert_eq!(final_limited.admitted_before_frontier(), 2);

    assert_eq!(final_limited.selected_count(), 1);
}

#[test]
fn multi_step_intention_is_order_invariant_non_mutating_and_facade_equivalent() {
    let sources = source_intents(
        &[goal(1), goal(2)],
        &[candidate(1, 10, 110, 900), candidate(2, 20, 120, 800)],
    );

    let candidates = vec![valid_plan(1, 10, 110, 11), valid_plan(2, 20, 120, 21)];

    let sources_before = sources.clone();

    let candidates_before = candidates.clone();

    let mut reversed_sources = sources.clone();

    reversed_sources.reverse();

    let mut reversed_candidates = candidates.clone();

    reversed_candidates.reverse();

    let intention_policy = default_policy();

    let direct = MultiStepIntention::select(&sources, &candidates, intention_policy);

    let reversed =
        MultiStepIntention::select(&reversed_sources, &reversed_candidates, intention_policy);

    let facade = UniversalMultiStepIntention::evaluate(&sources, &candidates, intention_policy);

    let repeated = UniversalMultiStepIntention::evaluate(&sources, &candidates, intention_policy);

    assert_eq!(direct, reversed);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(sources, sources_before);

    assert_eq!(candidates, candidates_before);
}
