use athlesia_arc_agi_3_adapter::{
    cognitive_interaction_runtime::{
        ArcAgi3CognitiveInteractionError, ArcAgi3CognitiveInteractionRuntime,
        UniversalArcAgi3CognitiveInteractionRuntime,
    },
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
    interactive_session_runtime::ArcAgi3InteractiveSessionError,
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
};
use athlesia_mindstone_sparse_cognition::CognitiveSignal;

mod m51_fixture {
    include!("support/m51_online_orchestration_fixture.rs");
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn grid(value: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![value, 2], vec![3, 4]]).unwrap()
}

fn observation(
    game: &str,
    state: ArcAgi3GameState,
    frames: Vec<ArcAgi3Grid>,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new(game.to_string()).unwrap(),
        state,
        ArcAgi3FrameSequence::new(frames).unwrap(),
        0,
        3,
        ArcAgi3AvailableActions::new(vec![ArcAgi3ActionId::Action1, ArcAgi3ActionId::Action6])
            .unwrap(),
        last_action,
    )
}

fn initial() -> ArcAgi3Observation {
    observation(
        "runtime-test",
        ArcAgi3GameState::NotPlayed,
        vec![grid(1)],
        None,
    )
}

fn reset_response(frames: Vec<ArcAgi3Grid>) -> ArcAgi3Observation {
    observation(
        "runtime-test",
        ArcAgi3GameState::NotFinished,
        frames,
        Some(ArcAgi3Action::reset()),
    )
}

#[test]
fn constructor_projects_initial_environment_observation_exactly() {
    let runtime = ArcAgi3CognitiveInteractionRuntime::new(initial(), 40).unwrap();

    assert_eq!(runtime.perception().frame_count(), 1,);

    assert_eq!(runtime.perception().latest_frame().observation_index(), 40,);

    assert_eq!(runtime.next_perceptual_observation_index(), 41,);

    assert_eq!(runtime.observation(), &initial(),);
}

#[test]
fn protocol_reset_and_real_response_advance_session_and_perception_together() {
    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial(), 10).unwrap();

    let command = runtime.begin_reset().unwrap();

    assert!(command.is_reset());
    assert!(runtime.session().has_pending_command());

    let completion = runtime
        .complete_environment_turn(reset_response(vec![grid(5)]), signal(900))
        .unwrap();

    assert_eq!(completion.turn().event_index(), 1,);

    assert_eq!(completion.turn().action(), ArcAgi3Action::reset(),);

    assert!(!completion.has_cognitive_feedback());

    assert_eq!(
        completion.perception().latest_frame().observation_index(),
        11,
    );

    assert_eq!(runtime.next_perceptual_observation_index(), 12,);

    assert_eq!(runtime.session().completed_turn_count(), 1,);

    assert_eq!(runtime.session().completed_reset_count(), 1,);

    assert!(!runtime.session().has_pending_command());
}

#[test]
fn multi_frame_environment_response_preserves_cross_turn_and_internal_transitions() {
    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial(), 100).unwrap();

    runtime.begin_reset().unwrap();

    let completion = runtime
        .complete_environment_turn(reset_response(vec![grid(5), grid(6)]), signal(800))
        .unwrap();

    assert_eq!(completion.perception().frame_count(), 2,);

    /*
     * previous environment frame -> response frame 1
     * response frame 1 -> response frame 2
     */
    assert_eq!(completion.perception().transition_count(), 2,);

    assert_eq!(completion.perception().frames()[0].observation_index(), 101,);

    assert_eq!(completion.perception().frames()[1].observation_index(), 102,);

    assert_eq!(runtime.next_perceptual_observation_index(), 103,);
}

#[test]
fn game_identity_failure_is_transactionally_atomic() {
    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial(), 20).unwrap();

    runtime.begin_reset().unwrap();

    let before = runtime.clone();

    let wrong_game = observation(
        "different-game",
        ArcAgi3GameState::NotFinished,
        vec![grid(9)],
        Some(ArcAgi3Action::reset()),
    );

    assert_eq!(
        runtime.complete_environment_turn(wrong_game, signal(900),),
        Err(ArcAgi3CognitiveInteractionError::Session(
            ArcAgi3InteractiveSessionError::GameIdentityMismatch
        )),
    );

    assert_eq!(runtime, before);

    assert!(runtime.session().has_pending_command());
}

#[test]
fn reported_action_failure_is_transactionally_atomic() {
    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial(), 30).unwrap();

    runtime.begin_reset().unwrap();

    let before = runtime.clone();

    let wrong_action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let response = observation(
        "runtime-test",
        ArcAgi3GameState::NotFinished,
        vec![grid(8)],
        Some(wrong_action),
    );

    assert_eq!(
        runtime.complete_environment_turn(response, signal(900),),
        Err(ArcAgi3CognitiveInteractionError::Session(
            ArcAgi3InteractiveSessionError::ReportedActionMismatch
        )),
    );

    assert_eq!(runtime, before);
}

#[test]
fn perceptual_index_overflow_is_transactionally_atomic() {
    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial(), u64::MAX - 1).unwrap();

    assert_eq!(runtime.next_perceptual_observation_index(), u64::MAX,);

    runtime.begin_reset().unwrap();

    let before = runtime.clone();

    let result = runtime.complete_environment_turn(reset_response(vec![grid(7)]), signal(900));

    assert!(matches!(
        result,
        Err(ArcAgi3CognitiveInteractionError::Perception(_))
    ));

    /*
     * The cloned session may have completed internally, but
     * the real runtime must still retain its pending command.
     */
    assert_eq!(runtime, before);

    assert!(runtime.session().has_pending_command());

    assert_eq!(runtime.session().completed_turn_count(), 0,);
}

#[test]
fn second_command_is_blocked_while_environment_response_is_pending() {
    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial(), 1).unwrap();

    runtime.begin_reset().unwrap();

    assert_eq!(
        runtime.begin_reset(),
        Err(ArcAgi3CognitiveInteractionError::Session(
            ArcAgi3InteractiveSessionError::PendingCommandExists
        )),
    );
}

#[test]
fn perceptual_cursor_remains_monotonic_across_multiple_turns() {
    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial(), 500).unwrap();

    assert_eq!(runtime.next_perceptual_observation_index(), 501,);

    runtime.begin_reset().unwrap();

    runtime
        .complete_environment_turn(reset_response(vec![grid(5)]), signal(800))
        .unwrap();

    assert_eq!(runtime.next_perceptual_observation_index(), 502,);

    /*
     * RESET remains protocol-authorized in an active game.
     * This tests cursor continuity without bypassing M51 with
     * an ordinary environment action.
     */
    runtime.begin_reset().unwrap();

    runtime
        .complete_environment_turn(reset_response(vec![grid(6), grid(7)]), signal(800))
        .unwrap();

    assert_eq!(runtime.next_perceptual_observation_index(), 504,);

    assert_eq!(runtime.session().completed_turn_count(), 2,);

    assert_eq!(runtime.session().completed_reset_count(), 2,);
}

#[test]
fn universal_facade_matches_direct_runtime_construction() {
    let direct = ArcAgi3CognitiveInteractionRuntime::new(initial(), 77);

    let facade = UniversalArcAgi3CognitiveInteractionRuntime::create_runtime(initial(), 77);

    assert_eq!(direct, facade);
}

#[test]
fn environment_response_without_pending_command_is_rejected_without_state_change() {
    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial(), 900).unwrap();

    let before = runtime.clone();

    assert_eq!(
        runtime.complete_environment_turn(reset_response(vec![grid(9)]), signal(900),),
        Err(ArcAgi3CognitiveInteractionError::Session(
            ArcAgi3InteractiveSessionError::NoPendingCommand
        )),
    );

    assert_eq!(runtime, before);
}

#[test]
fn real_m51_orchestration_dispatches_exact_arc_action_and_binds_canonical_feedback() {
    let initial_observation = observation(
        "runtime-e2e",
        ArcAgi3GameState::NotFinished,
        vec![grid(1)],
        None,
    );

    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial_observation, 2_000).unwrap();

    let arc_action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(arc_action);

    let step = m51_fixture::begin_arc(&mut runtime, cognitive_action.clone()).unwrap();

    assert!(step.orchestration().advanced());

    assert_eq!(
        step.dispatch().source_anchor_state(),
        &athlesia_mindstone_sparse_cognition::CognitiveStructure::atom(1000),
    );

    assert_eq!(step.dispatch().action(), &cognitive_action,);

    assert_eq!(
        step.dispatch().predicted_outcome(),
        &athlesia_mindstone_sparse_cognition::CognitiveStructure::atom(110),
    );

    assert_eq!(step.command().action(), arc_action,);

    assert!(runtime.session().has_pending_command());

    assert_eq!(runtime.session().pending_action(), Some(arc_action),);

    let response = observation(
        "runtime-e2e",
        ArcAgi3GameState::NotFinished,
        vec![grid(8), grid(9)],
        Some(arc_action),
    );

    let completion = runtime
        .complete_environment_turn(response.clone(), signal(900))
        .unwrap();

    assert_eq!(completion.turn().action(), arc_action,);

    assert_eq!(completion.turn().observation(), &response,);

    assert!(completion.has_cognitive_feedback());

    let evidence = completion.turn().evidence().unwrap();

    assert_eq!(evidence.action_observation().event_index(), 1,);

    assert_eq!(
        evidence.action_observation().descriptor(),
        &cognitive_action,
    );

    let execution = evidence.execution_observation();

    assert_eq!(
        execution.observed_state(),
        step.dispatch().source_anchor_state(),
    );

    assert_eq!(execution.observed_action(), &cognitive_action,);

    assert_eq!(execution.observation_confidence(), signal(900),);

    let experiment = evidence.experiment_observation();

    assert_eq!(
        experiment.source_state(),
        step.dispatch().source_anchor_state(),
    );

    assert_eq!(experiment.action(), &cognitive_action,);

    assert_eq!(experiment.confidence(), signal(900),);

    assert_eq!(execution.observed_outcome(), experiment.observed_outcome(),);

    assert_eq!(completion.perception().frame_count(), 2,);

    assert_eq!(completion.perception().transition_count(), 2,);

    assert_eq!(runtime.session().completed_turn_count(), 1,);

    assert_eq!(runtime.session().completed_action_count(), 1,);

    assert_eq!(runtime.session().completed_reset_count(), 0,);

    assert!(!runtime.session().has_pending_command());
}
