use athlesia_arc_agi_3_adapter::{
    cognitive_protocol_bridge::{
        ArcAgi3CognitiveBridgeError, ArcAgi3CognitiveCodecError, ArcAgi3CognitiveProtocolBridge,
    },
    ArcAgi3Action, ArcAgi3ActionAuthorizationStatus, ArcAgi3ActionId, ArcAgi3AvailableActions,
    ArcAgi3FrameSequence, ArcAgi3GameId, ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
};
use athlesia_integrated_cognitive_agent::{
    EnvironmentActionDispatch, EnvironmentInteractionEvidence, EnvironmentInteractionObservation,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn grid(rows: Vec<Vec<u8>>) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(rows).unwrap()
}

fn rich_observation() -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new("ls20-bridge-test".to_string()).unwrap(),
        ArcAgi3GameState::NotFinished,
        ArcAgi3FrameSequence::new(vec![
            grid(vec![vec![1, 2, 3], vec![4, 5, 6]]),
            grid(vec![vec![6, 5, 4], vec![3, 2, 1]]),
        ])
        .unwrap(),
        3,
        7,
        ArcAgi3AvailableActions::new(vec![
            ArcAgi3ActionId::Action1,
            ArcAgi3ActionId::Action3,
            ArcAgi3ActionId::Action6,
            ArcAgi3ActionId::Action7,
        ])
        .unwrap(),
        Some(ArcAgi3Action::coordinate(17, 23).unwrap()),
    )
}

#[test]
fn every_valid_arc_action_has_exact_cognitive_round_trip() {
    let actions = [
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action2).unwrap(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action4).unwrap(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action5).unwrap(),
        ArcAgi3Action::coordinate(0, 0).unwrap(),
        ArcAgi3Action::coordinate(63, 63).unwrap(),
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action7).unwrap(),
        ArcAgi3Action::reset(),
    ];

    for action in actions {
        let encoded = ArcAgi3CognitiveProtocolBridge::encode_action(action);
        let decoded = ArcAgi3CognitiveProtocolBridge::decode_action(&encoded).unwrap();

        assert_eq!(decoded, action);
    }
}

#[test]
fn action_codec_preserves_coordinate_identity_without_hash_authority() {
    let first = ArcAgi3Action::coordinate(12, 34).unwrap();
    let second = ArcAgi3Action::coordinate(34, 12).unwrap();

    let first_encoded = ArcAgi3CognitiveProtocolBridge::encode_action(first);
    let second_encoded = ArcAgi3CognitiveProtocolBridge::encode_action(second);

    assert_ne!(first_encoded, second_encoded);

    assert_eq!(
        ArcAgi3CognitiveProtocolBridge::decode_action(&first_encoded).unwrap(),
        first
    );

    assert_eq!(
        ArcAgi3CognitiveProtocolBridge::decode_action(&second_encoded).unwrap(),
        second
    );
}

#[test]
fn malformed_cognitive_action_is_rejected_instead_of_reinterpreted() {
    let malformed = CognitiveStructure::atom(1);

    assert_eq!(
        ArcAgi3CognitiveProtocolBridge::decode_action(&malformed),
        Err(ArcAgi3CognitiveCodecError::ExpectedOrderedStructure)
    );
}

#[test]
fn complete_arc_observation_round_trips_without_information_loss() {
    let observation = rich_observation();

    let encoded = ArcAgi3CognitiveProtocolBridge::encode_observation(&observation);
    let decoded = ArcAgi3CognitiveProtocolBridge::decode_observation(&encoded).unwrap();

    assert_eq!(decoded, observation);
}

#[test]
fn observation_codec_preserves_entire_frame_sequence_not_only_latest_frame() {
    let first = rich_observation();

    let second = ArcAgi3Observation::new(
        first.game_id().clone(),
        first.state(),
        ArcAgi3FrameSequence::new(vec![
            grid(vec![vec![1, 2, 3], vec![4, 5, 6]]),
            grid(vec![vec![0, 0, 0], vec![0, 0, 0]]),
        ])
        .unwrap(),
        first.levels_completed(),
        first.win_levels(),
        first.available_actions().clone(),
        first.last_action(),
    );

    let first_encoded = ArcAgi3CognitiveProtocolBridge::encode_observation(&first);
    let second_encoded = ArcAgi3CognitiveProtocolBridge::encode_observation(&second);

    assert_ne!(first_encoded, second_encoded);

    assert_eq!(
        ArcAgi3CognitiveProtocolBridge::decode_observation(&first_encoded).unwrap(),
        first
    );

    assert_eq!(
        ArcAgi3CognitiveProtocolBridge::decode_observation(&second_encoded).unwrap(),
        second
    );
}

#[test]
fn observation_codec_preserves_game_state_and_progress_metadata() {
    let base = rich_observation();

    let win = ArcAgi3Observation::new(
        base.game_id().clone(),
        ArcAgi3GameState::Win,
        base.frames().clone(),
        4,
        7,
        base.available_actions().clone(),
        base.last_action(),
    );

    let encoded = ArcAgi3CognitiveProtocolBridge::encode_observation(&win);
    let decoded = ArcAgi3CognitiveProtocolBridge::decode_observation(&encoded).unwrap();

    assert_eq!(decoded.state(), ArcAgi3GameState::Win);
    assert_eq!(decoded.levels_completed(), 4);
    assert_eq!(decoded.win_levels(), 7);
}

#[test]
fn available_action_set_survives_exact_observation_round_trip() {
    let observation = rich_observation();

    let encoded = ArcAgi3CognitiveProtocolBridge::encode_observation(&observation);
    let decoded = ArcAgi3CognitiveProtocolBridge::decode_observation(&encoded).unwrap();

    assert_eq!(decoded.available_actions(), observation.available_actions());
}

#[test]
fn encoded_action_authorization_uses_real_arc_available_action_contract() {
    let observation = rich_observation();

    let allowed = ArcAgi3CognitiveProtocolBridge::encode_action(
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap(),
    );

    let denied = ArcAgi3CognitiveProtocolBridge::encode_action(
        ArcAgi3Action::discrete(ArcAgi3ActionId::Action2).unwrap(),
    );

    let allowed_result =
        ArcAgi3CognitiveProtocolBridge::authorize_cognitive_action(&observation, &allowed).unwrap();

    let denied_result =
        ArcAgi3CognitiveProtocolBridge::authorize_cognitive_action(&observation, &denied).unwrap();

    assert_eq!(
        allowed_result.status(),
        ArcAgi3ActionAuthorizationStatus::AuthorizedAction
    );

    assert_eq!(
        denied_result.status(),
        ArcAgi3ActionAuthorizationStatus::ActionUnavailable
    );
}

#[test]
fn environment_observation_contains_exact_full_arc_observation_structure() {
    let observation = rich_observation();
    let expected = ArcAgi3CognitiveProtocolBridge::encode_observation(&observation);

    let bridged =
        ArcAgi3CognitiveProtocolBridge::environment_observation(41, &observation, signal(900))
            .unwrap();

    assert_eq!(bridged.event_index(), 41);
    assert_eq!(bridged.observed_outcome(), &expected);
    assert_eq!(bridged.confidence(), signal(900));
}

#[test]
fn zero_confidence_feedback_is_rejected_at_m51_boundary_contract() {
    let observation = rich_observation();

    assert_eq!(
        ArcAgi3CognitiveProtocolBridge::environment_observation(
            1,
            &observation,
            CognitiveSignal::zero(),
        ),
        Err(ArcAgi3CognitiveBridgeError::InvalidConfidence)
    );
}

#[test]
fn codec_is_deterministic_and_non_mutating() {
    let observation = rich_observation();
    let before = observation.clone();

    let first = ArcAgi3CognitiveProtocolBridge::encode_observation(&observation);
    let second = ArcAgi3CognitiveProtocolBridge::encode_observation(&observation);

    assert_eq!(first, second);
    assert_eq!(observation, before);

    assert_eq!(
        ArcAgi3CognitiveProtocolBridge::decode_observation(&first).unwrap(),
        before
    );
}

#[test]
fn bridge_is_compile_time_bound_to_real_m51_environment_dispatch_and_feedback_types() {
    let _decode_dispatch: fn(
        &EnvironmentActionDispatch,
    ) -> Result<ArcAgi3Action, ArcAgi3CognitiveCodecError> =
        ArcAgi3CognitiveProtocolBridge::decode_dispatch;

    let _environment_observation: fn(
        u64,
        &ArcAgi3Observation,
        CognitiveSignal,
    ) -> Result<
        EnvironmentInteractionObservation,
        ArcAgi3CognitiveBridgeError,
    > = ArcAgi3CognitiveProtocolBridge::environment_observation;

    let _bind_feedback: fn(
        &EnvironmentActionDispatch,
        u64,
        &ArcAgi3Observation,
        CognitiveSignal,
    )
        -> Result<EnvironmentInteractionEvidence, ArcAgi3CognitiveBridgeError> =
        ArcAgi3CognitiveProtocolBridge::bind_feedback;
}
