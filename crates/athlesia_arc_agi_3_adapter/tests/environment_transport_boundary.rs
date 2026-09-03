use std::collections::VecDeque;

use athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3CognitiveInteractionRuntime;
use athlesia_arc_agi_3_adapter::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge;
use athlesia_arc_agi_3_adapter::environment_transport_boundary::*;
use athlesia_arc_agi_3_adapter::*;
use athlesia_mindstone_sparse_cognition::CognitiveSignal;

mod m51_fixture {
    include!("support/m51_online_orchestration_fixture.rs");
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn grid(value: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![value, value], vec![value, value]]).unwrap()
}

fn observation(
    game_id: &str,
    state: ArcAgi3GameState,
    values: &[u8],
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new(game_id.to_string()).unwrap(),
        state,
        ArcAgi3FrameSequence::new(values.iter().copied().map(grid).collect()).unwrap(),
        0,
        3,
        ArcAgi3AvailableActions::new(vec![
            ArcAgi3ActionId::Action1,
            ArcAgi3ActionId::Action6,
            ArcAgi3ActionId::Action7,
        ])
        .unwrap(),
        last_action,
    )
}

fn wire_json(
    game_id: &str,
    guid: &str,
    state: &str,
    frames: serde_json::Value,
    action_input: serde_json::Value,
    available_actions: serde_json::Value,
) -> String {
    serde_json::json!({
        "game_id": game_id,
        "guid": guid,
        "frame": frames,
        "state": state,
        "levels_completed": 2,
        "win_levels": 5,
        "action_input": action_input,
        "available_actions": available_actions,
    })
    .to_string()
}

#[derive(Debug)]
struct ScriptedTransport {
    start: Option<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    responses: VecDeque<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    start_count: usize,
    execute_count: usize,
}

impl ScriptedTransport {
    fn new(initial: ArcAgi3Observation) -> Self {
        Self {
            start: Some(Ok(initial)),
            responses: VecDeque::new(),
            start_count: 0,
            execute_count: 0,
        }
    }

    fn push(&mut self, response: Result<ArcAgi3Observation, ArcAgi3TransportError>) {
        self.responses.push_back(response);
    }
}

impl ArcAgi3EnvironmentTransport for ScriptedTransport {
    fn start_game(
        &mut self,
        _game_id: &ArcAgi3GameId,
        _card_id: &str,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        self.start_count += 1;

        self.start
            .take()
            .unwrap_or(Err(ArcAgi3TransportError::ActiveSessionExists))
    }

    fn execute(
        &mut self,
        _command: &athlesia_arc_agi_3_adapter::interactive_session_runtime::ArcAgi3SessionCommand,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        self.execute_count += 1;

        self.responses
            .pop_front()
            .unwrap_or(Err(ArcAgi3TransportError::NoActiveSession))
    }
}

#[test]
fn rest_decoder_preserves_multiframe_state_metadata_and_reset_echo() {
    let json = wire_json(
        "ls20-version",
        "guid-1",
        "NOT_FINISHED",
        serde_json::json!([[[1, 2], [3, 4]], [[5, 6], [7, 8]]]),
        serde_json::json!({
            "id": 0,
            "data": {}
        }),
        serde_json::json!([1, 6, 7]),
    );

    let decoded = ArcAgi3RestProtocol::decode_observation_json(&json).unwrap();

    let observation = decoded.observation();

    assert_eq!(decoded.guid(), "guid-1");
    assert_eq!(observation.game_id().as_str(), "ls20-version");
    assert_eq!(observation.state(), ArcAgi3GameState::NotFinished);
    assert_eq!(observation.frames().frame_count(), 2);
    assert_eq!(observation.levels_completed(), 2);
    assert_eq!(observation.win_levels(), 5);
    assert_eq!(observation.last_action(), Some(ArcAgi3Action::reset()));
}

#[test]
fn rest_decoder_preserves_action6_coordinates_exactly() {
    let json = wire_json(
        "xy01-version",
        "guid-xy",
        "NOT_FINISHED",
        serde_json::json!([[[1, 1], [1, 1]]]),
        serde_json::json!({
            "id": 6,
            "data": {
                "x": 63,
                "y": 17
            }
        }),
        serde_json::json!([6]),
    );

    let decoded = ArcAgi3RestProtocol::decode_observation_json(&json).unwrap();

    assert_eq!(
        decoded.observation().last_action(),
        ArcAgi3Action::coordinate(63, 17),
    );
}

#[test]
fn rest_decoder_maps_official_not_started_to_internal_not_played() {
    let json = wire_json(
        "ns01-version",
        "guid-ns",
        "NOT_STARTED",
        serde_json::json!([[[0]]]),
        serde_json::json!({
            "id": 0,
            "data": {}
        }),
        serde_json::json!([]),
    );

    let decoded = ArcAgi3RestProtocol::decode_observation_json(&json).unwrap();

    assert_eq!(decoded.observation().state(), ArcAgi3GameState::NotPlayed,);
}

#[test]
fn rest_decoder_rejects_malformed_remote_protocol_values() {
    let bad_state = wire_json(
        "bad1-version",
        "guid-1",
        "PLAYING",
        serde_json::json!([[[0]]]),
        serde_json::json!({"id": 1}),
        serde_json::json!([1]),
    );

    assert!(matches!(
        ArcAgi3RestProtocol::decode_observation_json(&bad_state),
        Err(ArcAgi3RestDecodeError::InvalidState(_)),
    ));

    let bad_payload = wire_json(
        "bad2-version",
        "guid-2",
        "NOT_FINISHED",
        serde_json::json!([[[0]]]),
        serde_json::json!({
            "id": 6,
            "data": {"x": 4}
        }),
        serde_json::json!([6]),
    );

    assert_eq!(
        ArcAgi3RestProtocol::decode_observation_json(&bad_payload),
        Err(ArcAgi3RestDecodeError::InvalidActionPayload),
    );

    let bad_grid = wire_json(
        "bad3-version",
        "guid-3",
        "NOT_FINISHED",
        serde_json::json!([[[0, 1], [2]]]),
        serde_json::json!({"id": 1}),
        serde_json::json!([1]),
    );

    assert_eq!(
        ArcAgi3RestProtocol::decode_observation_json(&bad_grid),
        Err(ArcAgi3RestDecodeError::InvalidFrame),
    );

    let bad_available = wire_json(
        "bad4-version",
        "guid-4",
        "NOT_FINISHED",
        serde_json::json!([[[0]]]),
        serde_json::json!({"id": 1}),
        serde_json::json!([1, 1]),
    );

    assert_eq!(
        ArcAgi3RestProtocol::decode_observation_json(&bad_available),
        Err(ArcAgi3RestDecodeError::InvalidAvailableActions),
    );
}

#[test]
fn rest_protocol_builds_exact_start_reset_and_simple_action_requests() {
    let game_id = ArcAgi3GameId::new("ls20-version".to_string()).unwrap();

    let start = ArcAgi3RestProtocol::start_request(&game_id, "card-1").unwrap();

    assert_eq!(start.endpoint(), "/api/cmd/RESET");
    assert_eq!(start.game_id(), "ls20-version");
    assert_eq!(start.card_id(), Some("card-1"));
    assert_eq!(start.guid(), None);
    assert_eq!(start.coordinate(), None);

    let session =
        ArcAgi3RestSession::new(game_id, "card-1".to_string(), "guid-1".to_string()).unwrap();

    let initial = observation("ls20-version", ArcAgi3GameState::NotFinished, &[0], None);

    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(initial, 100).unwrap();

    let reset_command = runtime.begin_reset().unwrap();

    let reset = ArcAgi3RestProtocol::command_request(&session, &reset_command).unwrap();

    assert_eq!(reset.endpoint(), "/api/cmd/RESET");
    assert_eq!(reset.card_id(), Some("card-1"));
    assert_eq!(reset.guid(), Some("guid-1"));

    let mut action_session =
        athlesia_arc_agi_3_adapter::interactive_session_runtime::ArcAgi3InteractiveSession::new(
            observation("ls20-version", ArcAgi3GameState::NotFinished, &[0], None),
        );

    let action_command = action_session
        .begin_action(ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap())
        .unwrap();

    let action_request = ArcAgi3RestProtocol::command_request(&session, &action_command).unwrap();

    assert_eq!(action_request.endpoint(), "/api/cmd/ACTION1",);

    assert_eq!(action_request.game_id(), "ls20-version",);

    assert_eq!(action_request.card_id(), None,);

    assert_eq!(action_request.guid(), Some("guid-1"),);

    assert_eq!(action_request.coordinate(), None,);
}

#[test]
fn rest_protocol_builds_action6_without_coordinate_enumeration() {
    let game_id = ArcAgi3GameId::new("xy01-version".to_string()).unwrap();

    let session =
        ArcAgi3RestSession::new(game_id, "card-xy".to_string(), "guid-xy".to_string()).unwrap();

    let action = ArcAgi3Action::coordinate(63, 17).unwrap();

    let mut session_runtime =
        athlesia_arc_agi_3_adapter::interactive_session_runtime::ArcAgi3InteractiveSession::new(
            observation("xy01-version", ArcAgi3GameState::NotFinished, &[1], None),
        );

    let command = session_runtime.begin_action(action).unwrap();

    let request = ArcAgi3RestProtocol::command_request(&session, &command).unwrap();

    assert_eq!(request.endpoint(), "/api/cmd/ACTION6");
    assert_eq!(request.guid(), Some("guid-xy"));
    assert_eq!(request.card_id(), None);
    assert_eq!(request.coordinate(), ArcAgi3Coordinate::new(63, 17),);
}

#[test]
fn boundary_starts_runtime_from_transport_observation() {
    let initial = observation(
        "st01-version",
        ArcAgi3GameState::NotFinished,
        &[2, 3],
        Some(ArcAgi3Action::reset()),
    );

    let mut transport = ScriptedTransport::new(initial.clone());

    let requested = ArcAgi3GameId::new("st01".to_string()).unwrap();

    let runtime = ArcAgi3EnvironmentTransportBoundary::start_runtime(
        &mut transport,
        &requested,
        "card-st",
        500,
    )
    .unwrap();

    assert_eq!(transport.start_count, 1);
    assert_eq!(runtime.observation(), &initial);
    assert_eq!(runtime.perception().frame_count(), 2);
    assert_eq!(runtime.next_perceptual_observation_index(), 502);
}

#[test]
fn protocol_reset_round_trip_through_boundary_has_no_cognitive_feedback() {
    let initial = observation("rs01-version", ArcAgi3GameState::NotFinished, &[1], None);

    let mut transport = ScriptedTransport::new(initial);

    let requested = ArcAgi3GameId::new("rs01".to_string()).unwrap();

    let mut runtime = ArcAgi3EnvironmentTransportBoundary::start_runtime(
        &mut transport,
        &requested,
        "card-rs",
        600,
    )
    .unwrap();

    let command = runtime.begin_reset().unwrap();

    transport.push(Ok(observation(
        "rs01-version",
        ArcAgi3GameState::NotFinished,
        &[7, 8],
        Some(ArcAgi3Action::reset()),
    )));

    let completion = ArcAgi3EnvironmentTransportBoundary::complete_pending(
        &mut transport,
        &mut runtime,
        &command,
        signal(900),
    )
    .unwrap();

    assert!(!completion.has_cognitive_feedback());
    assert_eq!(transport.execute_count, 1);
    assert_eq!(runtime.session().completed_turn_count(), 1);
    assert_eq!(runtime.session().completed_reset_count(), 1);
    assert_eq!(runtime.session().completed_action_count(), 0);
    assert_eq!(completion.perception().frame_count(), 2);
}

#[test]
fn transport_failure_preserves_exact_pending_runtime_state() {
    let initial = observation("tf01-version", ArcAgi3GameState::NotFinished, &[1], None);

    let mut transport = ScriptedTransport::new(initial);

    let requested = ArcAgi3GameId::new("tf01".to_string()).unwrap();

    let mut runtime = ArcAgi3EnvironmentTransportBoundary::start_runtime(
        &mut transport,
        &requested,
        "card-tf",
        700,
    )
    .unwrap();

    let command = runtime.begin_reset().unwrap();
    let before = runtime.clone();

    transport.push(Err(ArcAgi3TransportError::HttpTransport {
        message: "connection lost".to_string(),
        disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
    }));

    let result = ArcAgi3EnvironmentTransportBoundary::complete_pending(
        &mut transport,
        &mut runtime,
        &command,
        signal(900),
    );

    assert!(matches!(
        result,
        Err(ArcAgi3TransportError::HttpTransport {
            disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
            ..
        }),
    ));

    assert_eq!(runtime, before);
    assert!(runtime.session().has_pending_command());
}

#[test]
fn command_mismatch_is_rejected_before_transport_side_effect() {
    let initial = observation("cm01-version", ArcAgi3GameState::NotFinished, &[1], None);

    let mut transport = ScriptedTransport::new(initial.clone());

    let requested = ArcAgi3GameId::new("cm01".to_string()).unwrap();

    let mut runtime = ArcAgi3EnvironmentTransportBoundary::start_runtime(
        &mut transport,
        &requested,
        "card-cm",
        800,
    )
    .unwrap();

    let _pending_reset = runtime.begin_reset().unwrap();

    let mut other_session =
        athlesia_arc_agi_3_adapter::interactive_session_runtime::ArcAgi3InteractiveSession::new(
            initial,
        );

    let other_command = other_session
        .begin_action(ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap())
        .unwrap();

    let before = runtime.clone();

    let result = ArcAgi3EnvironmentTransportBoundary::complete_pending(
        &mut transport,
        &mut runtime,
        &other_command,
        signal(900),
    );

    assert_eq!(result, Err(ArcAgi3TransportError::PendingCommandMismatch),);

    assert_eq!(transport.execute_count, 0);
    assert_eq!(runtime, before);
}

#[test]
fn real_m51_executive_command_round_trip_through_transport_boundary_binds_feedback() {
    let initial = observation("e201-version", ArcAgi3GameState::NotFinished, &[1], None);

    let mut transport = ScriptedTransport::new(initial);

    let requested = ArcAgi3GameId::new("e201".to_string()).unwrap();

    let mut runtime = ArcAgi3EnvironmentTransportBoundary::start_runtime(
        &mut transport,
        &requested,
        "card-e2e",
        900,
    )
    .unwrap();

    let arc_action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(arc_action);

    let step = m51_fixture::begin_arc(&mut runtime, cognitive_action.clone()).unwrap();

    assert_eq!(step.command().action(), arc_action);

    transport.push(Ok(observation(
        "e201-version",
        ArcAgi3GameState::NotFinished,
        &[5, 6],
        Some(arc_action),
    )));

    let completion = ArcAgi3EnvironmentTransportBoundary::complete_pending(
        &mut transport,
        &mut runtime,
        step.command(),
        signal(900),
    )
    .unwrap();

    assert!(completion.has_cognitive_feedback());

    let evidence = completion.turn().evidence().unwrap();

    assert_eq!(
        evidence.action_observation().descriptor(),
        &cognitive_action,
    );

    assert_eq!(
        evidence.execution_observation().observed_action(),
        &cognitive_action,
    );

    assert_eq!(
        evidence.experiment_observation().action(),
        &cognitive_action,
    );

    assert_eq!(completion.turn().action(), arc_action,);

    assert_eq!(transport.execute_count, 1);
}

#[test]
fn cognitively_invalid_remote_response_is_atomic_even_after_transport_execution() {
    let initial = observation("ar01-version", ArcAgi3GameState::NotFinished, &[1], None);

    let mut transport = ScriptedTransport::new(initial);

    let requested = ArcAgi3GameId::new("ar01".to_string()).unwrap();

    let mut runtime = ArcAgi3EnvironmentTransportBoundary::start_runtime(
        &mut transport,
        &requested,
        "card-ar",
        1000,
    )
    .unwrap();

    let command = runtime.begin_reset().unwrap();
    let before = runtime.clone();

    transport.push(Ok(observation(
        "different-game",
        ArcAgi3GameState::NotFinished,
        &[2],
        Some(ArcAgi3Action::reset()),
    )));

    let result = ArcAgi3EnvironmentTransportBoundary::complete_pending(
        &mut transport,
        &mut runtime,
        &command,
        signal(900),
    );

    assert!(matches!(
        result,
        Err(ArcAgi3TransportError::CognitiveCompletionRejected {
            disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
            ..
        }),
    ));

    assert_eq!(transport.execute_count, 1);
    assert_eq!(runtime, before);
}

#[test]
fn rest_transport_configuration_is_strict_and_session_safe() {
    assert!(matches!(
        ArcAgi3RestTransport::new("not a url".to_string(), "key".to_string(),),
        Err(ArcAgi3TransportError::InvalidBaseUrl),
    ));

    assert!(matches!(
        ArcAgi3RestTransport::new("http://localhost:8001".to_string(), String::new(),),
        Err(ArcAgi3TransportError::EmptyApiKey),
    ));

    let transport =
        ArcAgi3RestTransport::new("http://localhost:8001/".to_string(), "test-key".to_string())
            .unwrap();

    assert!(transport.session().is_none());
}

#[test]
fn universal_transport_boundary_matches_direct_start_semantics() {
    let initial = observation("uv01-version", ArcAgi3GameState::NotFinished, &[4], None);

    let mut direct_transport = ScriptedTransport::new(initial.clone());

    let mut facade_transport = ScriptedTransport::new(initial);

    let requested = ArcAgi3GameId::new("uv01".to_string()).unwrap();

    let direct = ArcAgi3EnvironmentTransportBoundary::start_runtime(
        &mut direct_transport,
        &requested,
        "card-uv",
        1100,
    )
    .unwrap();

    let facade = UniversalArcAgi3EnvironmentTransportBoundary::start_runtime(
        &mut facade_transport,
        &requested,
        "card-uv",
        1100,
    )
    .unwrap();

    assert_eq!(direct, facade);
}
