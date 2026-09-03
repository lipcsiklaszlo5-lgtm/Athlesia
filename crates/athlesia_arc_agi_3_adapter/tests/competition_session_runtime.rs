use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use athlesia_arc_agi_3_adapter::bounded_episode_runtime::{
    ArcAgi3BoundedEpisodePolicy, ArcAgi3BoundedEpisodeTermination,
};
use athlesia_arc_agi_3_adapter::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge;
use athlesia_arc_agi_3_adapter::competition_session_runtime::*;
use athlesia_arc_agi_3_adapter::environment_transport_boundary::{
    ArcAgi3EnvironmentTransport, ArcAgi3TransportError,
};
use athlesia_arc_agi_3_adapter::live_environment_runtime::{
    ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError,
};
use athlesia_arc_agi_3_adapter::*;
use athlesia_mindstone_sparse_cognition::CognitiveSignal;
use serde_json::{json, Value};

mod m51_fixture {
    include!("support/m51_online_orchestration_fixture.rs");
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn game_id(value: &str) -> ArcAgi3GameId {
    ArcAgi3GameId::new(value.to_string()).unwrap()
}

fn grid(value: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![value, value], vec![value, value]]).unwrap()
}

fn action1() -> ArcAgi3Action {
    ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap()
}

fn observation(
    game: &str,
    state: ArcAgi3GameState,
    value: u8,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        game_id(game),
        state,
        ArcAgi3FrameSequence::new(vec![grid(value)]).unwrap(),
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

fn metadata() -> ArcAgi3CompetitionMetadata {
    ArcAgi3CompetitionMetadata::new(
        Some("https://example.com/athlesia".to_string()),
        vec!["athlesia".to_string(), "m52".to_string()],
        Some(json!({
            "commit": "test",
            "runtime": "rust",
        })),
    )
    .unwrap()
}

fn summary_value(card_id: &str, published: bool) -> Value {
    let mut value = json!({
        "card_id": card_id,
        "score": 12.5,
        "environments": [],
        "total_environments_completed": 1,
        "total_environments": 2,
        "total_levels_completed": 3,
        "total_levels": 7,
        "total_actions": 41,
        "competition_mode": true,
        "opaque": {
            "commit": "test"
        },
        "unknown_future_field": {
            "preserved": true
        }
    });

    if published {
        value.as_object_mut().unwrap().insert(
            "published_at".to_string(),
            Value::String("2026-09-03T12:00:00Z".to_string()),
        );
    }

    value
}

fn summary(card_id: &str, published: bool) -> ArcAgi3ScorecardSummary {
    ArcAgi3ScorecardRestProtocol::decode_summary(summary_value(card_id, published)).unwrap()
}

#[derive(Debug)]
struct ScriptedScorecardTransport {
    open_results: VecDeque<Result<ArcAgi3ScorecardId, ArcAgi3CompetitionTransportError>>,
    get_results: VecDeque<Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError>>,
    close_results: VecDeque<Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError>>,
    open_count: Cell<usize>,
    get_count: Cell<usize>,
    close_count: Cell<usize>,
}

impl ScriptedScorecardTransport {
    fn new(
        open_results: Vec<Result<ArcAgi3ScorecardId, ArcAgi3CompetitionTransportError>>,
        get_results: Vec<Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError>>,
        close_results: Vec<Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError>>,
    ) -> Self {
        Self {
            open_results: open_results.into_iter().collect(),
            get_results: get_results.into_iter().collect(),
            close_results: close_results.into_iter().collect(),
            open_count: Cell::new(0),
            get_count: Cell::new(0),
            close_count: Cell::new(0),
        }
    }

    fn open_count(&self) -> usize {
        self.open_count.get()
    }

    fn get_count(&self) -> usize {
        self.get_count.get()
    }

    fn close_count(&self) -> usize {
        self.close_count.get()
    }
}

impl ArcAgi3ScorecardTransport for ScriptedScorecardTransport {
    fn open_scorecard(
        &mut self,
        _metadata: &ArcAgi3CompetitionMetadata,
    ) -> Result<ArcAgi3ScorecardId, ArcAgi3CompetitionTransportError> {
        self.open_count.set(
            self.open_count
                .get()
                .checked_add(1)
                .expect("scripted scorecard open counter remains bounded"),
        );

        self.open_results.pop_front().unwrap_or_else(|| {
            Err(ArcAgi3CompetitionTransportError::InvalidConfiguration(
                "missing scripted open response".to_string(),
            ))
        })
    }

    fn get_scorecard(
        &mut self,
        _card_id: &ArcAgi3ScorecardId,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError> {
        self.get_count.set(
            self.get_count
                .get()
                .checked_add(1)
                .expect("scripted scorecard get counter remains bounded"),
        );

        self.get_results.pop_front().unwrap_or_else(|| {
            Err(ArcAgi3CompetitionTransportError::InvalidConfiguration(
                "missing scripted get response".to_string(),
            ))
        })
    }

    fn close_scorecard(
        &mut self,
        _card_id: &ArcAgi3ScorecardId,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError> {
        self.close_count.set(
            self.close_count
                .get()
                .checked_add(1)
                .expect("scripted scorecard close counter remains bounded"),
        );

        self.close_results.pop_front().unwrap_or_else(|| {
            Err(ArcAgi3CompetitionTransportError::InvalidConfiguration(
                "missing scripted close response".to_string(),
            ))
        })
    }
}

#[derive(Debug)]
struct ScriptedEnvironmentTransport {
    initial: Option<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    responses: VecDeque<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    observed_card_id: RefCell<Option<String>>,
    execute_count: Cell<usize>,
}

impl ScriptedEnvironmentTransport {
    fn new(
        initial: ArcAgi3Observation,
        responses: Vec<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    ) -> Self {
        Self {
            initial: Some(Ok(initial)),
            responses: responses.into_iter().collect(),
            observed_card_id: RefCell::new(None),
            execute_count: Cell::new(0),
        }
    }

    fn observed_card_id(&self) -> Option<String> {
        self.observed_card_id.borrow().clone()
    }

    fn execute_count(&self) -> usize {
        self.execute_count.get()
    }
}

impl ArcAgi3EnvironmentTransport for ScriptedEnvironmentTransport {
    fn start_game(
        &mut self,
        _game_id: &ArcAgi3GameId,
        card_id: &str,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        *self.observed_card_id.borrow_mut() = Some(card_id.to_string());

        self.initial
            .take()
            .unwrap_or(Err(ArcAgi3TransportError::ActiveSessionExists))
    }

    fn execute(
        &mut self,
        _command: &athlesia_arc_agi_3_adapter::interactive_session_runtime::ArcAgi3SessionCommand,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        self.execute_count.set(
            self.execute_count
                .get()
                .checked_add(1)
                .expect("scripted environment execute counter remains bounded"),
        );

        self.responses
            .pop_front()
            .unwrap_or(Err(ArcAgi3TransportError::NoActiveSession))
    }
}

fn execute_action1(
    live: &mut athlesia_arc_agi_3_adapter::live_environment_runtime::ArcAgi3LiveEnvironmentRuntime<
        ScriptedEnvironmentTransport,
    >,
) -> Result<ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError> {
    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action1());

    live.execute_with(signal(900), |cognitive| {
        m51_fixture::begin_arc(cognitive, cognitive_action)
    })
}

#[test]
fn competition_metadata_preserves_exact_metadata_and_forces_competition_mode() {
    let metadata = metadata();

    assert_eq!(metadata.source_url(), Some("https://example.com/athlesia"),);

    assert_eq!(
        metadata.tags(),
        &["athlesia".to_string(), "m52".to_string(),],
    );

    assert_eq!(
        metadata.opaque(),
        Some(&json!({
            "commit": "test",
            "runtime": "rust",
        }),),
    );

    assert!(metadata.competition_mode());
}

#[test]
fn competition_metadata_rejects_invalid_source_url() {
    assert_eq!(
        ArcAgi3CompetitionMetadata::new(Some("not a uri".to_string(),), Vec::new(), None,),
        Err(ArcAgi3CompetitionProtocolError::InvalidSourceUrl,),
    );
}

#[test]
fn competition_metadata_enforces_official_opaque_size_limit() {
    let large = Value::String("x".repeat(ARC_AGI_3_SCORECARD_OPAQUE_MAX_BYTES));

    assert!(matches!(
        ArcAgi3CompetitionMetadata::new(None, Vec::new(), Some(large),),
        Err(ArcAgi3CompetitionProtocolError::OpaqueTooLarge {
            maximum_bytes: ARC_AGI_3_SCORECARD_OPAQUE_MAX_BYTES,
            ..
        },),
    ));
}

#[test]
fn rest_protocol_builds_official_competition_open_and_close_payloads() {
    let metadata = metadata();

    let open = ArcAgi3ScorecardRestProtocol::open_request(&metadata);

    assert_eq!(open["source_url"], json!("https://example.com/athlesia"),);

    assert_eq!(open["tags"], json!(["athlesia", "m52"]),);

    assert_eq!(open["competition_mode"], json!(true),);

    let card_id = ArcAgi3ScorecardId::new("card-123".to_string()).unwrap();

    assert_eq!(
        ArcAgi3ScorecardRestProtocol::close_request(&card_id),
        json!({
            "card_id": "card-123",
        }),
    );

    assert_eq!(
        ArcAgi3ScorecardRestProtocol::OPEN_PATH,
        "api/scorecard/open",
    );

    assert_eq!(
        ArcAgi3ScorecardRestProtocol::CLOSE_PATH,
        "api/scorecard/close",
    );
}

#[test]
fn open_response_requires_nonempty_server_card_identity() {
    assert_eq!(
        ArcAgi3ScorecardRestProtocol::decode_open_response(json!({
            "card_id": ""
        }),),
        Err(ArcAgi3CompetitionProtocolError::InvalidCardId,),
    );

    let card = ArcAgi3ScorecardRestProtocol::decode_open_response(json!({
        "card_id":
            "server-card-1"
    }))
    .unwrap();

    assert_eq!(card.as_str(), "server-card-1",);
}

#[test]
fn scorecard_summary_preserves_server_score_and_unknown_fields_without_recalculation() {
    let raw = summary_value("scorecard-1", true);

    let summary = ArcAgi3ScorecardRestProtocol::decode_summary(raw.clone()).unwrap();

    assert_eq!(summary.card_id().as_str(), "scorecard-1",);

    assert_eq!(summary.score(), 12.5,);

    assert_eq!(summary.total_actions(), 41,);

    assert_eq!(summary.total_levels_completed(), 3,);

    assert_eq!(summary.competition_mode(), Some(true),);

    assert_eq!(summary.raw(), &raw,);

    assert_eq!(
        summary.raw()["unknown_future_field"]["preserved"],
        json!(true),
    );
}

#[test]
fn malformed_scorecard_summary_is_rejected_instead_of_fabricated() {
    let malformed = json!({
        "card_id": "scorecard-1",
        "score": 12.5,
        "environments": [],
        "total_environments_completed": 1,
        "total_environments": 2,
        "total_levels_completed": 3,
        "total_levels": 7
    });

    assert!(matches!(
        ArcAgi3ScorecardRestProtocol::decode_summary(malformed),
        Err(ArcAgi3CompetitionProtocolError::MalformedResponse(_),),
    ));
}

#[test]
fn competition_session_opens_exactly_one_server_scorecard() {
    let transport = ScriptedScorecardTransport::new(
        vec![Ok(
            ArcAgi3ScorecardId::new("competition-card".to_string()).unwrap()
        )],
        Vec::new(),
        Vec::new(),
    );

    let session = ArcAgi3CompetitionSession::open(transport, &metadata()).unwrap();

    assert_eq!(session.card_id().as_str(), "competition-card",);

    assert_eq!(session.status(), ArcAgi3CompetitionSessionStatus::Open,);

    assert_eq!(session.scorecard_transport().open_count(), 1,);
}

#[test]
fn competition_game_forwards_exact_server_card_id_to_environment_reset() {
    let scorecard_transport = ScriptedScorecardTransport::new(
        vec![Ok(
            ArcAgi3ScorecardId::new("exact-card-id".to_string()).unwrap()
        )],
        Vec::new(),
        Vec::new(),
    );

    let mut session = ArcAgi3CompetitionSession::open(scorecard_transport, &metadata()).unwrap();

    let game_id = game_id("competition-game");

    let environment = ScriptedEnvironmentTransport::new(
        observation("competition-game", ArcAgi3GameState::NotFinished, 1, None),
        Vec::new(),
    );

    let game = session.start_game(environment, &game_id, 100).unwrap();

    assert_eq!(game.card_id().as_str(), "exact-card-id",);

    assert_eq!(
        game.runtime().transport().observed_card_id(),
        Some("exact-card-id".to_string(),),
    );
}

#[test]
fn competition_game_runs_real_bounded_m51_episode_under_same_card() {
    let action = action1();

    let scorecard_transport = ScriptedScorecardTransport::new(
        vec![Ok(
            ArcAgi3ScorecardId::new("bounded-card".to_string()).unwrap()
        )],
        Vec::new(),
        Vec::new(),
    );

    let mut session = ArcAgi3CompetitionSession::open(scorecard_transport, &metadata()).unwrap();

    let game_id = game_id("bounded-competition-game");

    let environment = ScriptedEnvironmentTransport::new(
        observation(
            "bounded-competition-game",
            ArcAgi3GameState::NotFinished,
            1,
            None,
        ),
        vec![Ok(observation(
            "bounded-competition-game",
            ArcAgi3GameState::Win,
            2,
            Some(action),
        ))],
    );

    let mut game = session.start_game(environment, &game_id, 200).unwrap();

    let result = game
        .run_bounded_with(
            ArcAgi3BoundedEpisodePolicy::new(4).unwrap(),
            execute_action1,
        )
        .unwrap();

    assert_eq!(result.termination(), ArcAgi3BoundedEpisodeTermination::Won,);

    assert_eq!(result.completed_steps_in_episode(), 1,);

    assert_eq!(game.runtime().transport().execute_count(), 1,);

    assert_eq!(
        game.runtime().transport().observed_card_id(),
        Some("bounded-card".to_string(),),
    );

    let live = game.finish();

    assert_eq!(live.completed_cognitive_step_count(), 1,);

    assert_eq!(session.status(), ArcAgi3CompetitionSessionStatus::Open,);
}

#[test]
fn successful_close_uses_server_summary_and_locks_session() {
    let card_id = ArcAgi3ScorecardId::new("close-card".to_string()).unwrap();

    let transport = ScriptedScorecardTransport::new(
        vec![Ok(card_id)],
        Vec::new(),
        vec![Ok(summary("close-card", true))],
    );

    let mut session = ArcAgi3CompetitionSession::open(transport, &metadata()).unwrap();

    let closed = session.close().unwrap();

    assert_eq!(closed.score(), 12.5,);

    assert_eq!(session.status(), ArcAgi3CompetitionSessionStatus::Closed,);

    assert_eq!(session.scorecard_transport().close_count(), 1,);

    assert_eq!(
        session.close(),
        Err(ArcAgi3CompetitionSessionError::SessionNotOpen(
            ArcAgi3CompetitionSessionStatus::Closed,
        ),),
    );

    assert_eq!(session.scorecard_transport().close_count(), 1,);
}

#[test]
fn indeterminate_close_failure_is_never_automatically_retried() {
    let transport = ScriptedScorecardTransport::new(
        vec![Ok(
            ArcAgi3ScorecardId::new("faulted-card".to_string()).unwrap()
        )],
        Vec::new(),
        vec![Err(ArcAgi3CompetitionTransportError::HttpTransport {
            message: "timeout".to_string(),
            disposition: ArcAgi3CompetitionFailureDisposition::DispatchIndeterminate,
        })],
    );

    let mut session = ArcAgi3CompetitionSession::open(transport, &metadata()).unwrap();

    assert!(matches!(
        session.close(),
        Err(ArcAgi3CompetitionSessionError::ScorecardTransport(
            ArcAgi3CompetitionTransportError::HttpTransport {
                disposition: ArcAgi3CompetitionFailureDisposition::DispatchIndeterminate,
                ..
            },
        ),),
    ));

    assert_eq!(
        session.status(),
        ArcAgi3CompetitionSessionStatus::CloseFaulted,
    );

    assert_eq!(
        session.close_failure_disposition(),
        Some(ArcAgi3CompetitionFailureDisposition::DispatchIndeterminate,),
    );

    assert_eq!(
        session.close(),
        Err(ArcAgi3CompetitionSessionError::SessionNotOpen(
            ArcAgi3CompetitionSessionStatus::CloseFaulted,
        ),),
    );

    assert_eq!(session.scorecard_transport().close_count(), 1,);
}

#[test]
fn close_failure_reconciliation_uses_safe_get_instead_of_second_post() {
    let transport = ScriptedScorecardTransport::new(
        vec![Ok(
            ArcAgi3ScorecardId::new("reconcile-card".to_string()).unwrap()
        )],
        vec![Ok(summary("reconcile-card", true))],
        vec![Err(ArcAgi3CompetitionTransportError::HttpTransport {
            message: "connection lost".to_string(),
            disposition: ArcAgi3CompetitionFailureDisposition::DispatchIndeterminate,
        })],
    );

    let mut session = ArcAgi3CompetitionSession::open(transport, &metadata()).unwrap();

    assert!(session.close().is_err());

    let reconciled = session.reconcile_close_failure().unwrap();

    assert_eq!(reconciled.published_at(), Some("2026-09-03T12:00:00Z"),);

    assert_eq!(session.status(), ArcAgi3CompetitionSessionStatus::Closed,);

    assert_eq!(session.scorecard_transport().close_count(), 1,);

    assert_eq!(session.scorecard_transport().get_count(), 1,);
}

#[test]
fn polling_detects_server_auto_closed_scorecard_without_local_close_post() {
    let transport = ScriptedScorecardTransport::new(
        vec![Ok(
            ArcAgi3ScorecardId::new("auto-close-card".to_string()).unwrap()
        )],
        vec![Ok(summary("auto-close-card", true))],
        Vec::new(),
    );

    let mut session = ArcAgi3CompetitionSession::open(transport, &metadata()).unwrap();

    let polled = session.poll().unwrap();

    assert!(polled.published_at().is_some());

    assert_eq!(session.status(), ArcAgi3CompetitionSessionStatus::Closed,);

    assert_eq!(session.scorecard_transport().close_count(), 0,);

    assert_eq!(session.scorecard_transport().get_count(), 1,);
}

#[test]
fn universal_facade_matches_direct_competition_session_open() {
    let direct_transport = ScriptedScorecardTransport::new(
        vec![Ok(
            ArcAgi3ScorecardId::new("facade-card".to_string()).unwrap()
        )],
        Vec::new(),
        Vec::new(),
    );

    let facade_transport = ScriptedScorecardTransport::new(
        vec![Ok(
            ArcAgi3ScorecardId::new("facade-card".to_string()).unwrap()
        )],
        Vec::new(),
        Vec::new(),
    );

    let direct = ArcAgi3CompetitionSession::open(direct_transport, &metadata()).unwrap();

    let facade =
        UniversalArcAgi3CompetitionSessionRuntime::open(facade_transport, &metadata()).unwrap();

    assert_eq!(direct.card_id(), facade.card_id(),);

    assert_eq!(direct.status(), facade.status(),);
}

#[test]
fn competition_runtime_never_calculates_score_or_invents_action_policy() {
    let source = include_str!("../src/competition_session_runtime.rs");

    for forbidden in [
        "RHAE",
        "baseline_actions",
        ".powi(",
        "Action1",
        "Action2",
        "Action3",
        "Action4",
        "Action5",
        "Action6",
        "Action7",
        "begin_reset",
        ".reset(",
        "retry(",
        "sleep(",
    ] {
        assert!(
            !source.contains(forbidden),
            "competition runtime leaked forbidden policy or score logic: {forbidden}",
        );
    }

    assert!(source.contains("\"competition_mode\""));

    assert!(source.contains("ArcAgi3BoundedEpisodeRuntime::run_with"));

    assert!(source.contains("ArcAgi3LiveEnvironmentRuntime::start"));
}
