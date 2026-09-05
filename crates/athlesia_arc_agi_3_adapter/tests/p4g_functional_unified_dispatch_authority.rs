use athlesia_arc_agi_3_adapter::{
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
    cognitive_interaction_runtime::ArcAgi3CognitiveInteractionRuntime,
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
};
use athlesia_autonomous_active_experimentation::{
    ActiveExperimentBounds, ActiveExperimentPolicy, ActiveExperimentThresholds,
    BeliefDrivenExperimentProposalBounds, BeliefDrivenExperimentProposalPolicy,
    CompetingHypothesisPrediction, ExperimentSequencePlanningBounds,
    ExperimentSequencePlanningPolicy, GroundedExperimentPossibility, HypothesisBeliefState,
    IntegratedAutonomousExperimentationPolicy, IntegratedAutonomousExperimentationResult,
    LearningProgressBounds, LearningProgressPolicy, LearningProgressThresholds,
    StopContinueExperimentationBounds, StopContinueExperimentationPolicy,
    StopContinueExperimentationThresholds, UniversalAutonomousIntegratedExperimentationCycle,
};
use athlesia_executive_agency::{
    ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveSelectionThresholds, ExecutiveUtilityWeights,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

mod m51_fixture {
    include!("support/m51_online_orchestration_fixture.rs");
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn action(id: ArcAgi3ActionId) -> ArcAgi3Action {
    ArcAgi3Action::discrete(id).unwrap()
}

fn object_grid(value: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![value, value], vec![8, 9]]).unwrap()
}

fn observation(
    game: &str,
    frame: ArcAgi3Grid,
    available: Vec<ArcAgi3ActionId>,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new(game.to_string()).unwrap(),
        ArcAgi3GameState::NotFinished,
        ArcAgi3FrameSequence::new(vec![frame]).unwrap(),
        0,
        3,
        ArcAgi3AvailableActions::new(available).unwrap(),
        last_action,
    )
}

fn normal_observation(
    game: &str,
    value: u8,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    observation(
        game,
        object_grid(value),
        vec![
            ArcAgi3ActionId::Action1,
            ArcAgi3ActionId::Action2,
            ArcAgi3ActionId::Action6,
        ],
        last_action,
    )
}

fn real_turn(
    runtime: &mut ArcAgi3CognitiveInteractionRuntime,
    game: &str,
    selected_action: ArcAgi3Action,
    value: u8,
) {
    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(selected_action);

    let step = m51_fixture::begin_arc(runtime, cognitive_action)
        .expect("real M51 executive action must begin");

    assert!(step.orchestration().advanced());

    runtime
        .complete_environment_turn(
            normal_observation(game, value, Some(selected_action)),
            signal(900),
        )
        .expect("real environment consequence must commit");
}

fn goal() -> ExecutiveGoal {
    ExecutiveGoal::new(
        atom(0x5034_4743_3147_4f41),
        signal(900),
        CognitiveSignal::zero(),
    )
}

fn executive_thresholds() -> ExecutiveSelectionThresholds {
    ExecutiveSelectionThresholds::new(
        signal(100),
        signal(100),
        signal(1),
        signal(600),
        signal(100),
    )
    .unwrap()
}

fn evidence_authority_policy() -> ExecutiveAgencyPolicy {
    ExecutiveAgencyPolicy::new(
        1,
        8,
        16,
        1,
        ExecutiveUtilityWeights::new(0, 0, 1000, 0, 0).unwrap(),
        executive_thresholds(),
    )
    .unwrap()
}

fn information_authority_policy() -> ExecutiveAgencyPolicy {
    ExecutiveAgencyPolicy::new(
        1,
        8,
        16,
        1,
        ExecutiveUtilityWeights::new(0, 0, 0, 1000, 0).unwrap(),
        executive_thresholds(),
    )
    .unwrap()
}

fn experiment_source_state() -> CognitiveStructure {
    atom(0x5034_4743_3153_5243)
}

fn belief(id: u64, confidence: u16) -> HypothesisBeliefState {
    HypothesisBeliefState::new(atom(id), signal(confidence)).unwrap()
}

fn competing_prediction(hypothesis: u64, outcome: u64) -> CompetingHypothesisPrediction {
    CompetingHypothesisPrediction::new(atom(hypothesis), atom(outcome), signal(900)).unwrap()
}

fn m50_policy() -> IntegratedAutonomousExperimentationPolicy {
    let foundation = ActiveExperimentPolicy::new(
        ActiveExperimentBounds::new(32, 32, 32).unwrap(),
        ActiveExperimentThresholds::new(signal(500), signal(500), signal(500), signal(500))
            .unwrap(),
    );

    let proposal = BeliefDrivenExperimentProposalPolicy::new(
        foundation,
        BeliefDrivenExperimentProposalBounds::new(16, 16, 16, 16).unwrap(),
        signal(500),
        signal(500),
    )
    .unwrap();

    let learning = LearningProgressPolicy::new(
        LearningProgressBounds::new(32, 16, 8).unwrap(),
        LearningProgressThresholds::new(signal(500), 2, signal(50)).unwrap(),
    )
    .unwrap();

    let sequence = ExperimentSequencePlanningPolicy::new(
        ExperimentSequencePlanningBounds::new(16, 16, 4, 64, 8).unwrap(),
        signal(500),
    )
    .unwrap();

    let control = StopContinueExperimentationPolicy::new(
        StopContinueExperimentationBounds::new(16, 8, 8).unwrap(),
        StopContinueExperimentationThresholds::new(
            signal(500),
            signal(850),
            signal(250),
            signal(100),
            signal(600),
            signal(500),
        )
        .unwrap(),
    );

    IntegratedAutonomousExperimentationPolicy::new(proposal, learning, sequence, control).unwrap()
}

fn continuing_experiment(
    selected_action: ArcAgi3Action,
) -> IntegratedAutonomousExperimentationResult {
    let source = experiment_source_state();

    let possibility = GroundedExperimentPossibility::new(
        source.clone(),
        ArcAgi3CognitiveProtocolBridge::encode_action(selected_action),
        vec![competing_prediction(1, 100), competing_prediction(2, 101)],
        signal(900),
        signal(900),
        signal(100),
    )
    .unwrap();

    let result = UniversalAutonomousIntegratedExperimentationCycle::evaluate(
        &source,
        &[belief(1, 700), belief(2, 680)],
        &[possibility],
        &[],
        0,
        m50_policy(),
    );

    assert!(
        result.continuing(),
        "fixture must contain a real M50 continuation decision"
    );

    assert_eq!(
        result.next_experiment().unwrap().action(),
        &ArcAgi3CognitiveProtocolBridge::encode_action(selected_action),
    );

    result
}

fn stopped_experimentation() -> IntegratedAutonomousExperimentationResult {
    let source = experiment_source_state();

    let result = UniversalAutonomousIntegratedExperimentationCycle::evaluate(
        &source,
        &[belief(1, 700)],
        &[],
        &[],
        0,
        m50_policy(),
    );

    assert!(result.stopped());
    assert!(result.next_experiment().is_none());

    result
}

fn mature_scene_and_model(runtime: &mut ArcAgi3CognitiveInteractionRuntime, game: &str) {
    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    for value in [2_u8, 3, 4, 5] {
        real_turn(runtime, game, action_one, value);
    }

    for (selected, value) in [
        (action_two, 5_u8),
        (action_one, 6_u8),
        (action_two, 6_u8),
        (action_one, 5_u8),
        (action_two, 5_u8),
        (action_one, 6_u8),
        (action_two, 6_u8),
        (action_one, 5_u8),
    ] {
        real_turn(runtime, game, selected, value);
    }

    assert!(runtime.current_best_scene_interpretation().is_some());

    assert_eq!(
        runtime.current_model_grounded_action_selection(
            &[action_one, action_two,],
            &goal(),
            signal(900),
            CognitiveSignal::zero(),
            evidence_authority_policy(),
        ),
        Some(action_one),
        "fixture must establish a real learned exploitation candidate"
    );
}

#[test]
fn continuing_m50_experiment_reaches_action_only_through_common_m48_authority() {
    let game = "p4gc1-m50-through-m48";

    let action_two = action(ArcAgi3ActionId::Action2);

    let runtime =
        ArcAgi3CognitiveInteractionRuntime::new(normal_observation(game, 1, None), 190_000)
            .unwrap();

    assert_eq!(
        runtime.current_unified_executive_action_selection(
            &[],
            &goal(),
            signal(900),
            CognitiveSignal::zero(),
            None,
            information_authority_policy(),
        ),
        None,
        "without M47 exploitation or M50 continuation there is no dispatch authority"
    );

    let experimentation = continuing_experiment(action_two);

    assert_eq!(
        runtime.current_unified_executive_action_selection(
                &[],
                &goal(),
                signal(900),
                CognitiveSignal::zero(),
                Some(athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3ExperimentDispatchAuthority::new(&experimentation, &experiment_source_state())),
                information_authority_policy(),
            ),
        Some(action_two),
        "real M50 continuation may act only after entering and winning the common M48 frontier"
    );
}

#[test]
fn learned_exploitation_and_m50_experiment_are_ranked_by_the_same_m48_policy() {
    let game = "p4gc1-common-authority";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(normal_observation(game, 1, None), 200_000)
            .unwrap();

    mature_scene_and_model(&mut runtime, game);

    let experimentation = continuing_experiment(action_two);

    /*
     * SAME retained M47 model.
     * SAME M50 continuation.
     * SAME ARC availability.
     * SAME candidate frontier.
     *
     * Only the explicit M48 utility authority changes.
     */
    assert_eq!(
        runtime.current_unified_executive_action_selection(
                &[action_one, action_two,],
                &goal(),
                signal(900),
                CognitiveSignal::zero(),
                Some(athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3ExperimentDispatchAuthority::new(&experimentation, &experiment_source_state())),
                evidence_authority_policy(),
            ),
        Some(action_one),
        "evidence-confidence authority must favor the already learned causal action"
    );

    assert_eq!(
        runtime.current_unified_executive_action_selection(
                &[action_one, action_two,],
                &goal(),
                signal(900),
                CognitiveSignal::zero(),
                Some(athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3ExperimentDispatchAuthority::new(&experimentation, &experiment_source_state())),
                information_authority_policy(),
            ),
        Some(action_two),
        "information-gain authority must allow the real M50 experiment to win without bypassing M48"
    );
}

#[test]
fn m50_stop_removes_experiment_authority_without_blocking_valid_exploitation() {
    let game = "p4gc1-m50-stop";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(normal_observation(game, 1, None), 210_000)
            .unwrap();

    mature_scene_and_model(&mut runtime, game);

    let stopped = stopped_experimentation();

    assert_eq!(
        runtime.current_unified_executive_action_selection(
                &[action_one, action_two,],
                &goal(),
                signal(900),
                CognitiveSignal::zero(),
                Some(athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3ExperimentDispatchAuthority::new(&stopped, &experiment_source_state())),
                evidence_authority_policy(),
            ),
        Some(action_one),
        "stopping experimentation must veto only experimental dispatch, not erase a valid learned exploitation candidate"
    );
}

#[test]
fn unavailable_m50_action_cannot_bypass_arc_grounding_or_m48() {
    let game = "p4gc1-unavailable-experiment";

    let action_two = action(ArcAgi3ActionId::Action2);

    let runtime = ArcAgi3CognitiveInteractionRuntime::new(
        observation(game, object_grid(1), vec![ArcAgi3ActionId::Action1], None),
        220_000,
    )
    .unwrap();

    let experimentation = continuing_experiment(action_two);

    assert_eq!(
        runtime.current_unified_executive_action_selection(
                &[],
                &goal(),
                signal(900),
                CognitiveSignal::zero(),
                Some(athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3ExperimentDispatchAuthority::new(&experimentation, &experiment_source_state())),
                information_authority_policy(),
            ),
        None,
        "M50 cannot dispatch an ARC action that the current environment does not authorize"
    );
}

#[test]
fn m50_source_state_mismatch_fails_closed_instead_of_reinterpreting_experiment() {
    let game = "p4gc1-source-state-authority";

    let action_two = action(ArcAgi3ActionId::Action2);

    let runtime =
        ArcAgi3CognitiveInteractionRuntime::new(normal_observation(game, 1, None), 230_000)
            .unwrap();

    let experimentation = continuing_experiment(action_two);

    let wrong_source = atom(0x5034_4743_3157_524f);

    assert_ne!(wrong_source, experiment_source_state());

    assert_eq!(
        runtime.current_unified_executive_action_selection(
                &[],
                &goal(),
                signal(900),
                CognitiveSignal::zero(),
                Some(athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3ExperimentDispatchAuthority::new(&experimentation, &wrong_source)),
                information_authority_policy(),
            ),
        None,
        "exact M50 source-state authority must survive common arbitration unchanged"
    );
}
