use athlesia_arc_agi_3_adapter::{
    interactive_session_runtime::{
        ArcAgi3InteractiveSession, ArcAgi3InteractiveSessionError, ArcAgi3InteractiveSessionStatus,
        UniversalArcAgi3InteractiveSession,
    },
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
};
use athlesia_integrated_cognitive_agent::{
    EnvironmentActionDispatch, EnvironmentInteractionEvidence,
};
use athlesia_mindstone_sparse_cognition::CognitiveSignal;

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn frame(value: u8) -> ArcAgi3FrameSequence {
    ArcAgi3FrameSequence::new(vec![ArcAgi3Grid::from_rows(vec![
        vec![value, value],
        vec![value, value],
    ])
    .unwrap()])
    .unwrap()
}

fn observation(
    state: ArcAgi3GameState,
    last_action: Option<ArcAgi3Action>,
    value: u8,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new("session-runtime-test".to_string()).unwrap(),
        state,
        frame(value),
        0,
        3,
        ArcAgi3AvailableActions::new(vec![
            ArcAgi3ActionId::Action1,
            ArcAgi3ActionId::Action3,
            ArcAgi3ActionId::Action6,
            ArcAgi3ActionId::Action7,
        ])
        .unwrap(),
        last_action,
    )
}

fn different_game_observation(
    state: ArcAgi3GameState,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new("different-game-version".to_string()).unwrap(),
        state,
        frame(9),
        0,
        3,
        ArcAgi3AvailableActions::new(vec![ArcAgi3ActionId::Action1]).unwrap(),
        last_action,
    )
}

#[test]
fn session_status_tracks_exact_game_lifecycle() {
    for (state, expected) in [
        (
            ArcAgi3GameState::NotPlayed,
            ArcAgi3InteractiveSessionStatus::NotStarted,
        ),
        (
            ArcAgi3GameState::NotFinished,
            ArcAgi3InteractiveSessionStatus::Active,
        ),
        (ArcAgi3GameState::Win, ArcAgi3InteractiveSessionStatus::Won),
        (
            ArcAgi3GameState::GameOver,
            ArcAgi3InteractiveSessionStatus::GameOver,
        ),
    ] {
        let session = ArcAgi3InteractiveSession::new(observation(state, None, 1));

        assert_eq!(session.status(), expected);
    }
}

#[test]
fn reset_starts_not_played_session_and_waits_for_environment_response() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotPlayed, None, 1));

    let command = session.begin_reset().unwrap();

    assert_eq!(command.action_id(), ArcAgi3ActionId::Reset);
    assert!(command.is_reset());
    assert!(session.has_pending_command());
    assert_eq!(
        session.status(),
        ArcAgi3InteractiveSessionStatus::AwaitingEnvironmentResponse
    );

    let completed = session
        .complete_turn(
            observation(
                ArcAgi3GameState::NotFinished,
                Some(ArcAgi3Action::reset()),
                2,
            ),
            signal(1000),
        )
        .unwrap();

    assert_eq!(completed.event_index(), 1);
    assert!(completed.action().id() == ArcAgi3ActionId::Reset);
    assert!(!completed.has_cognitive_feedback());

    assert_eq!(session.status(), ArcAgi3InteractiveSessionStatus::Active);
    assert_eq!(session.completed_turn_count(), 1);
    assert_eq!(session.completed_action_count(), 0);
    assert_eq!(session.completed_reset_count(), 1);
}

#[test]
fn pending_command_blocks_second_command_until_response_arrives() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotFinished, None, 1));

    let first = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    session.begin_action(first).unwrap();

    assert_eq!(
        session.begin_action(ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap()),
        Err(ArcAgi3InteractiveSessionError::PendingCommandExists)
    );

    assert_eq!(session.pending_action(), Some(first));
    assert_eq!(session.completed_turn_count(), 0);
}

#[test]
fn active_session_accepts_only_explicitly_available_actions() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotFinished, None, 1));

    let allowed = ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap();

    let denied = ArcAgi3Action::discrete(ArcAgi3ActionId::Action2).unwrap();

    assert_eq!(session.begin_action(allowed).unwrap().action(), allowed);

    let mut second =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotFinished, None, 1));

    assert_eq!(
        second.begin_action(denied),
        Err(ArcAgi3InteractiveSessionError::ActionUnavailable)
    );

    assert!(!second.has_pending_command());
}

#[test]
fn terminal_session_rejects_non_reset_environment_action() {
    for state in [
        ArcAgi3GameState::NotPlayed,
        ArcAgi3GameState::Win,
        ArcAgi3GameState::GameOver,
    ] {
        let mut session = ArcAgi3InteractiveSession::new(observation(state, None, 1));

        assert_eq!(
            session.begin_action(ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap()),
            Err(ArcAgi3InteractiveSessionError::GameNotActive)
        );

        assert!(!session.has_pending_command());
    }
}

#[test]
fn coordinate_action_survives_pending_and_completion_exactly() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotFinished, None, 1));

    let action = ArcAgi3Action::coordinate(17, 31).unwrap();

    let command = session.begin_action(action).unwrap();

    assert_eq!(command.action(), action);
    assert_eq!(session.pending_action(), Some(action));

    let completed = session
        .complete_turn(
            observation(ArcAgi3GameState::NotFinished, Some(action), 2),
            signal(900),
        )
        .unwrap();

    assert_eq!(completed.action(), action);
    assert_eq!(session.completed_action_count(), 1);
    assert_eq!(session.completed_reset_count(), 0);
}

#[test]
fn response_game_identity_mismatch_is_rejected_atomically() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotFinished, None, 1));

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    session.begin_action(action).unwrap();

    let before = session.clone();

    assert_eq!(
        session.complete_turn(
            different_game_observation(ArcAgi3GameState::NotFinished, Some(action),),
            signal(1000),
        ),
        Err(ArcAgi3InteractiveSessionError::GameIdentityMismatch)
    );

    assert_eq!(session, before);
}

#[test]
fn reported_action_mismatch_is_rejected_atomically() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotFinished, None, 1));

    let submitted = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let reported = ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap();

    session.begin_action(submitted).unwrap();

    let before = session.clone();

    assert_eq!(
        session.complete_turn(
            observation(ArcAgi3GameState::NotFinished, Some(reported), 2,),
            signal(1000),
        ),
        Err(ArcAgi3InteractiveSessionError::ReportedActionMismatch)
    );

    assert_eq!(session, before);
}

#[test]
fn response_without_reported_action_is_accepted_without_fabricating_identity() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotFinished, None, 1));

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    session.begin_action(action).unwrap();

    let completed = session
        .complete_turn(
            observation(ArcAgi3GameState::NotFinished, None, 2),
            signal(1000),
        )
        .unwrap();

    assert_eq!(completed.action(), action);
    assert_eq!(completed.observation().last_action(), None);
}

#[test]
fn completed_turn_preserves_full_exact_environment_observation() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotFinished, None, 1));

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action7).unwrap();

    session.begin_action(action).unwrap();

    let response = observation(ArcAgi3GameState::Win, Some(action), 15);

    let completed = session
        .complete_turn(response.clone(), signal(1000))
        .unwrap();

    assert_eq!(completed.observation(), &response);
    assert_eq!(session.observation(), &response);
    assert_eq!(session.status(), ArcAgi3InteractiveSessionStatus::Won);
}

#[test]
fn turn_action_and_reset_counters_have_separate_exact_semantics() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotPlayed, None, 1));

    session.begin_reset().unwrap();

    session
        .complete_turn(
            observation(
                ArcAgi3GameState::NotFinished,
                Some(ArcAgi3Action::reset()),
                2,
            ),
            signal(1000),
        )
        .unwrap();

    let first_action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    session.begin_action(first_action).unwrap();

    let first = session
        .complete_turn(
            observation(ArcAgi3GameState::NotFinished, Some(first_action), 3),
            signal(1000),
        )
        .unwrap();

    let second_action = ArcAgi3Action::coordinate(5, 6).unwrap();

    session.begin_action(second_action).unwrap();

    let second = session
        .complete_turn(
            observation(ArcAgi3GameState::GameOver, Some(second_action), 4),
            signal(1000),
        )
        .unwrap();

    assert_eq!(first.event_index(), 2);
    assert_eq!(second.event_index(), 3);

    assert_eq!(session.completed_turn_count(), 3);
    assert_eq!(session.completed_action_count(), 2);
    assert_eq!(session.completed_reset_count(), 1);
}

#[test]
fn completing_without_pending_command_is_rejected_without_mutation() {
    let mut session =
        ArcAgi3InteractiveSession::new(observation(ArcAgi3GameState::NotFinished, None, 1));

    let before = session.clone();

    assert_eq!(
        session.complete_turn(
            observation(ArcAgi3GameState::NotFinished, None, 2,),
            signal(1000),
        ),
        Err(ArcAgi3InteractiveSessionError::NoPendingCommand)
    );

    assert_eq!(session, before);
}

#[test]
fn two_equal_sessions_follow_identical_deterministic_transition_history() {
    let initial = observation(ArcAgi3GameState::NotFinished, None, 1);

    let mut left = ArcAgi3InteractiveSession::new(initial.clone());
    let mut right = ArcAgi3InteractiveSession::new(initial);

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap();

    assert_eq!(left.begin_action(action), right.begin_action(action));

    let response = observation(ArcAgi3GameState::NotFinished, Some(action), 8);

    assert_eq!(
        left.complete_turn(response.clone(), signal(850)),
        right.complete_turn(response, signal(850))
    );

    assert_eq!(left, right);
}

#[test]
fn runtime_is_compile_time_bound_to_real_m51_dispatch_and_feedback_contracts() {
    let _begin_dispatch: fn(
        &mut ArcAgi3InteractiveSession,
        &EnvironmentActionDispatch,
    ) -> Result<
        athlesia_arc_agi_3_adapter::interactive_session_runtime::ArcAgi3SessionCommand,
        ArcAgi3InteractiveSessionError,
    > = ArcAgi3InteractiveSession::begin_dispatch;

    let _facade_begin_dispatch: fn(
        &mut ArcAgi3InteractiveSession,
        &EnvironmentActionDispatch,
    ) -> Result<
        athlesia_arc_agi_3_adapter::interactive_session_runtime::ArcAgi3SessionCommand,
        ArcAgi3InteractiveSessionError,
    > = UniversalArcAgi3InteractiveSession::begin_dispatch;

    let _evidence_type: Option<EnvironmentInteractionEvidence> = None;
}

#[test]
fn universal_facade_complete_turn_matches_direct_runtime_semantics() {
    let initial = observation(ArcAgi3GameState::NotFinished, None, 1);

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let response = observation(ArcAgi3GameState::NotFinished, Some(action), 2);

    let mut direct = ArcAgi3InteractiveSession::new(initial.clone());
    let mut facade = ArcAgi3InteractiveSession::new(initial);

    direct.begin_action(action).unwrap();
    facade.begin_action(action).unwrap();

    let direct_result = direct.complete_turn(response.clone(), signal(900));

    let facade_result =
        UniversalArcAgi3InteractiveSession::complete_turn(&mut facade, response, signal(900));

    assert_eq!(direct_result, facade_result);
    assert_eq!(direct, facade);
}
