use athlesia_arc_agi_3_adapter::*;

fn grid(value: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![value, value], vec![value, value]]).unwrap()
}

fn observation(state: ArcAgi3GameState, actions: Vec<ArcAgi3ActionId>) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new("ls20-testversion".to_string()).unwrap(),
        state,
        ArcAgi3FrameSequence::new(vec![grid(1)]).unwrap(),
        2,
        6,
        ArcAgi3AvailableActions::new(actions).unwrap(),
        None,
    )
}

#[test]
fn game_states_preserve_exact_protocol_lifecycle() {
    assert!(!ArcAgi3GameState::NotPlayed.is_terminal());
    assert!(ArcAgi3GameState::NotFinished.awaiting_action());
    assert!(ArcAgi3GameState::Win.is_terminal());
    assert!(ArcAgi3GameState::GameOver.is_terminal());
    assert!(!ArcAgi3GameState::Win.awaiting_action());
}

#[test]
fn action_ids_round_trip_exact_protocol_identity() {
    let ids = [
        ArcAgi3ActionId::Action1,
        ArcAgi3ActionId::Action2,
        ArcAgi3ActionId::Action3,
        ArcAgi3ActionId::Action4,
        ArcAgi3ActionId::Action5,
        ArcAgi3ActionId::Action6,
        ArcAgi3ActionId::Action7,
    ];

    for (index, id) in ids.into_iter().enumerate() {
        let numeric = (index + 1) as u8;

        assert_eq!(id.numeric_id(), Some(numeric));
        assert_eq!(ArcAgi3ActionId::from_numeric_id(numeric), Some(id));
        assert_eq!(
            ArcAgi3ActionId::from_protocol_name(id.protocol_name()),
            Some(id)
        );
    }

    assert_eq!(ArcAgi3ActionId::Reset.numeric_id(), None);
    assert_eq!(
        ArcAgi3ActionId::from_protocol_name("RESET"),
        Some(ArcAgi3ActionId::Reset)
    );
    assert_eq!(ArcAgi3ActionId::from_numeric_id(0), None);
    assert_eq!(ArcAgi3ActionId::from_numeric_id(8), None);
}

#[test]
fn coordinate_action_enforces_competition_safe_grid_bounds() {
    assert!(ArcAgi3Coordinate::new(0, 0).is_some());
    assert!(ArcAgi3Coordinate::new(63, 63).is_some());
    assert!(ArcAgi3Coordinate::new(64, 0).is_none());
    assert!(ArcAgi3Coordinate::new(0, 64).is_none());

    let action = ArcAgi3Action::coordinate(63, 17).unwrap();

    assert_eq!(action.id(), ArcAgi3ActionId::Action6);
    assert_eq!(action.coordinate_data(), ArcAgi3Coordinate::new(63, 17));
}

#[test]
fn action6_requires_coordinates_and_other_actions_forbid_them() {
    let coordinate = ArcAgi3Coordinate::new(4, 9).unwrap();

    assert!(
        ArcAgi3Action::from_parts(ArcAgi3ActionId::Action6, ArcAgi3ActionPayload::None,).is_none()
    );

    assert!(ArcAgi3Action::from_parts(
        ArcAgi3ActionId::Action1,
        ArcAgi3ActionPayload::Coordinate(coordinate),
    )
    .is_none());

    assert!(
        ArcAgi3Action::from_parts(ArcAgi3ActionId::Action7, ArcAgi3ActionPayload::None,).is_some()
    );

    assert!(ArcAgi3Action::discrete(ArcAgi3ActionId::Action6).is_none());
    assert!(ArcAgi3Action::discrete(ArcAgi3ActionId::Reset).is_none());
}

#[test]
fn reset_is_explicit_and_carries_no_coordinate_payload() {
    let reset = ArcAgi3Action::reset();

    assert_eq!(reset.id(), ArcAgi3ActionId::Reset);
    assert_eq!(reset.payload(), ArcAgi3ActionPayload::None);
    assert_eq!(reset.coordinate_data(), None);
}

#[test]
fn available_actions_are_unique_canonical_and_reset_free() {
    let actions = ArcAgi3AvailableActions::new(vec![
        ArcAgi3ActionId::Action7,
        ArcAgi3ActionId::Action2,
        ArcAgi3ActionId::Action6,
        ArcAgi3ActionId::Action1,
    ])
    .unwrap();

    assert_eq!(
        actions.actions(),
        &[
            ArcAgi3ActionId::Action1,
            ArcAgi3ActionId::Action2,
            ArcAgi3ActionId::Action6,
            ArcAgi3ActionId::Action7,
        ]
    );

    assert_eq!(actions.action_count(), 4);

    assert!(ArcAgi3AvailableActions::new(
        vec![ArcAgi3ActionId::Action2, ArcAgi3ActionId::Action2,]
    )
    .is_none());

    assert!(ArcAgi3AvailableActions::new(vec![ArcAgi3ActionId::Reset,]).is_none());
}

#[test]
fn grid_rejects_empty_ragged_oversized_and_invalid_cells() {
    assert!(ArcAgi3Grid::from_rows(Vec::new()).is_none());

    assert!(ArcAgi3Grid::from_rows(vec![vec![1, 2], vec![3],]).is_none());

    assert!(ArcAgi3Grid::from_rows(vec![vec![16],]).is_none());

    assert!(ArcAgi3Grid::from_rows(vec![vec![0]; ARC_AGI_3_MAX_GRID_DIMENSION + 1]).is_none());

    assert!(ArcAgi3Grid::from_rows(vec![vec![0; ARC_AGI_3_MAX_GRID_DIMENSION + 1],]).is_none());
}

#[test]
fn grid_preserves_exact_row_major_coordinates() {
    let grid = ArcAgi3Grid::from_rows(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap();

    assert_eq!(grid.width(), 3);
    assert_eq!(grid.height(), 2);
    assert_eq!(grid.cells(), &[1, 2, 3, 4, 5, 6]);

    assert_eq!(grid.cell(0, 0), Some(1));
    assert_eq!(grid.cell(2, 0), Some(3));
    assert_eq!(grid.cell(0, 1), Some(4));
    assert_eq!(grid.cell(2, 1), Some(6));

    assert_eq!(grid.row(0), Some(&[1, 2, 3][..]));
    assert_eq!(grid.row(1), Some(&[4, 5, 6][..]));
    assert_eq!(grid.cell(3, 0), None);
    assert_eq!(grid.cell(0, 2), None);
}

#[test]
fn flat_grid_constructor_requires_exact_shape_and_cell_domain() {
    assert!(ArcAgi3Grid::from_flat(2, 2, vec![1, 2, 3, 4],).is_some());

    assert!(ArcAgi3Grid::from_flat(2, 2, vec![1, 2, 3],).is_none());

    assert!(ArcAgi3Grid::from_flat(1, 1, vec![16],).is_none());

    assert!(ArcAgi3Grid::from_flat(0, 1, Vec::new()).is_none());
}

#[test]
fn frame_sequence_preserves_transition_animation_order() {
    let first = grid(2);
    let middle = grid(7);
    let last = grid(14);

    let sequence =
        ArcAgi3FrameSequence::new(vec![first.clone(), middle.clone(), last.clone()]).unwrap();

    assert_eq!(sequence.frame_count(), 3);
    assert_eq!(sequence.frames(), &[first, middle, last]);
    assert_eq!(sequence.latest(), &grid(14));

    assert!(ArcAgi3FrameSequence::new(Vec::new()).is_none());
}

#[test]
fn game_identifier_rejects_empty_or_whitespace_identity() {
    assert!(ArcAgi3GameId::new(String::new()).is_none());
    assert!(ArcAgi3GameId::new("bad id".to_string()).is_none());

    let id = ArcAgi3GameId::new("ls20-016295f7601e".to_string()).unwrap();

    assert_eq!(id.as_str(), "ls20-016295f7601e");
}

#[test]
fn observation_preserves_exact_protocol_metadata() {
    let last_action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap();

    let frames = ArcAgi3FrameSequence::new(vec![grid(3), grid(4)]).unwrap();

    let available = ArcAgi3AvailableActions::new(vec![
        ArcAgi3ActionId::Action1,
        ArcAgi3ActionId::Action3,
        ArcAgi3ActionId::Action6,
    ])
    .unwrap();

    let observation = ArcAgi3Observation::new(
        ArcAgi3GameId::new("vc33-version".to_string()).unwrap(),
        ArcAgi3GameState::NotFinished,
        frames.clone(),
        3,
        6,
        available.clone(),
        Some(last_action),
    );

    assert_eq!(observation.game_id().as_str(), "vc33-version");
    assert_eq!(observation.state(), ArcAgi3GameState::NotFinished);
    assert_eq!(observation.frames(), &frames);
    assert_eq!(observation.levels_completed(), 3);
    assert_eq!(observation.win_levels(), 6);
    assert_eq!(observation.available_actions(), &available);
    assert_eq!(observation.last_action(), Some(last_action));
    assert!(!observation.terminal());
    assert!(observation.awaiting_action());
}

#[test]
fn active_game_authorizes_only_explicit_available_actions() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action2, ArcAgi3ActionId::Action6],
    );

    let allowed = ArcAgi3Action::discrete(ArcAgi3ActionId::Action2).unwrap();

    let unavailable = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let coordinate = ArcAgi3Action::coordinate(20, 30).unwrap();

    assert_eq!(
        ArcAgi3Protocol::authorize_action(&observation, allowed,).status(),
        ArcAgi3ActionAuthorizationStatus::AuthorizedAction
    );

    assert_eq!(
        ArcAgi3Protocol::authorize_action(&observation, coordinate,).status(),
        ArcAgi3ActionAuthorizationStatus::AuthorizedAction
    );

    assert_eq!(
        ArcAgi3Protocol::authorize_action(&observation, unavailable,).status(),
        ArcAgi3ActionAuthorizationStatus::ActionUnavailable
    );
}

#[test]
fn terminal_and_not_played_states_block_environment_actions_but_allow_reset() {
    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    for state in [
        ArcAgi3GameState::NotPlayed,
        ArcAgi3GameState::Win,
        ArcAgi3GameState::GameOver,
    ] {
        let observation = observation(state, vec![ArcAgi3ActionId::Action1]);

        let blocked = ArcAgi3Protocol::authorize_action(&observation, action);

        assert_eq!(
            blocked.status(),
            ArcAgi3ActionAuthorizationStatus::GameNotActive
        );
        assert!(!blocked.authorized());

        let reset = ArcAgi3Protocol::authorize_action(&observation, ArcAgi3Action::reset());

        assert_eq!(
            reset.status(),
            ArcAgi3ActionAuthorizationStatus::AuthorizedReset
        );
        assert!(reset.authorized());
    }
}

#[test]
fn protocol_is_deterministic_non_mutating_and_facade_equivalent() {
    let observation = observation(
        ArcAgi3GameState::NotFinished,
        vec![ArcAgi3ActionId::Action1, ArcAgi3ActionId::Action7],
    );

    let before = observation.clone();

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action7).unwrap();

    let direct_a = ArcAgi3Protocol::authorize_action(&observation, action);

    let direct_b = ArcAgi3Protocol::authorize_action(&observation, action);

    let facade = UniversalArcAgi3Protocol::authorize_action(&observation, action);

    assert_eq!(direct_a, direct_b);
    assert_eq!(direct_a, facade);
    assert_eq!(observation, before);
}
