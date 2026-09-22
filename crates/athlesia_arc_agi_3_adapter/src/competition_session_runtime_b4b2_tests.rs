use super::*;

use std::collections::VecDeque;

use crate::cognitive_interaction_runtime::c16i_successor_informed_two_contract_e2e_tests as c16i;

use crate::cognitive_trace::{
    ArcAgi3CognitiveTraceEvent, ArcAgi3CognitiveTraceSink, ArcAgi3TraceAuthorityKind,
};

use crate::environment_transport_boundary::ArcAgi3TransportError;

use crate::interactive_session_runtime::ArcAgi3SessionCommand;

use crate::successor_episode_runtime::ArcAgi3SuccessorEpisodeTermination;

use crate::{
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3FrameSequence,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
};

use athlesia_mindstone_sparse_cognition::CognitiveSignal;

#[derive(Debug)]
struct B4b2ScorecardTransport {
    card_id: ArcAgi3ScorecardId,
    open_count: usize,
    get_count: usize,
    close_count: usize,
}

impl B4b2ScorecardTransport {
    fn new(card: &str) -> Self {
        Self {
            card_id: ArcAgi3ScorecardId::new(card.to_string()).unwrap(),
            open_count: 0,
            get_count: 0,
            close_count: 0,
        }
    }

    fn summary(&self, published: bool) -> ArcAgi3ScorecardSummary {
        let published_at = published.then(|| "2026-09-22T00:00:00Z".to_string());

        let raw = serde_json::json!({
            "card_id": self.card_id.as_str(),
            "score": 0.0,
            "environments": [],
            "total_environments_completed": 0,
            "total_environments": 1,
            "total_levels_completed": 0,
            "total_levels": 1,
            "total_actions": 0,
            "competition_mode": true,
            "published_at": published_at,
        });

        ArcAgi3ScorecardSummary {
            card_id: self.card_id.clone(),
            score: 0.0,
            environments: Vec::new(),
            total_environments_completed: 0,
            total_environments: 1,
            total_levels_completed: 0,
            total_levels: 1,
            total_actions: 0,
            competition_mode: Some(true),
            published_at,
            raw,
        }
    }
}

impl ArcAgi3ScorecardTransport for B4b2ScorecardTransport {
    fn open_scorecard(
        &mut self,
        _metadata: &ArcAgi3CompetitionMetadata,
    ) -> Result<ArcAgi3ScorecardId, ArcAgi3CompetitionTransportError> {
        self.open_count += 1;

        Ok(self.card_id.clone())
    }

    fn get_scorecard(
        &mut self,
        _card_id: &ArcAgi3ScorecardId,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError> {
        self.get_count += 1;

        Ok(self.summary(false))
    }

    fn close_scorecard(
        &mut self,
        _card_id: &ArcAgi3ScorecardId,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError> {
        self.close_count += 1;

        Ok(self.summary(true))
    }
}

#[derive(Debug)]
struct B4b2EnvironmentTransport {
    initial: Option<ArcAgi3Observation>,
    responses: VecDeque<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    start_count: usize,
    execute_count: usize,
    started_card_id: Option<String>,
    executed_actions: Vec<ArcAgi3Action>,
}

impl B4b2EnvironmentTransport {
    fn new(initial: ArcAgi3Observation, responses: Vec<ArcAgi3Observation>) -> Self {
        Self {
            initial: Some(initial),
            responses: responses.into_iter().map(Ok).collect(),
            start_count: 0,
            execute_count: 0,
            started_card_id: None,
            executed_actions: Vec::new(),
        }
    }

    fn execute_count(&self) -> usize {
        self.execute_count
    }

    fn started_card_id(&self) -> Option<&str> {
        self.started_card_id.as_deref()
    }

    fn executed_actions(&self) -> &[ArcAgi3Action] {
        &self.executed_actions
    }
}

impl ArcAgi3EnvironmentTransport for B4b2EnvironmentTransport {
    fn start_game(
        &mut self,
        _game_id: &ArcAgi3GameId,
        card_id: &str,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        self.start_count += 1;

        self.started_card_id = Some(card_id.to_string());

        self.initial
            .take()
            .ok_or(ArcAgi3TransportError::ActiveSessionExists)
    }

    fn execute(
        &mut self,
        command: &ArcAgi3SessionCommand,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        self.execute_count += 1;

        self.executed_actions.push(command.action());

        self.responses
            .pop_front()
            .unwrap_or(Err(ArcAgi3TransportError::NoActiveSession))
    }
}

#[derive(Default)]
struct B4b2TraceCollector(Vec<ArcAgi3CognitiveTraceEvent>);

impl ArcAgi3CognitiveTraceSink for B4b2TraceCollector {
    fn record(&mut self, event: &ArcAgi3CognitiveTraceEvent) -> std::io::Result<()> {
        self.0.push(event.clone());

        Ok(())
    }
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn metadata() -> ArcAgi3CompetitionMetadata {
    ArcAgi3CompetitionMetadata::new(None, vec!["b4b2".to_string()], None).unwrap()
}

fn observation(
    game: &str,
    state: ArcAgi3GameState,
    value: u8,
    available: Vec<ArcAgi3ActionId>,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new(game.to_string()).unwrap(),
        state,
        ArcAgi3FrameSequence::new(vec![ArcAgi3Grid::from_rows(vec![vec![value]]).unwrap()])
            .unwrap(),
        if state == ArcAgi3GameState::Win { 1 } else { 0 },
        1,
        ArcAgi3AvailableActions::new(available).unwrap(),
        last_action,
    )
}

fn production_policy<'a>(
    goal: &'a athlesia_executive_agency::ExecutiveGoal,
) -> ArcAgi3ProductionSuccessorPolicy<'a> {
    let empty: [ArcAgi3Action; 0] = [];

    let seed = c16i::live_evidence_faithful_request(&empty, goal);

    ArcAgi3ProductionSuccessorPolicy::new(
        goal,
        seed.goal_alignment,
        seed.exploitation_execution_cost,
        seed.version_policy,
        seed.discrimination_policy,
        seed.expectation_policy,
        seed.exploitation_policy,
        signal(900),
    )
    .unwrap()
}

#[test]
fn b4b2_competition_production_runner_executes_real_terminal_step_and_preserves_lifecycle() {
    let game_name = "b4b2-production-terminal";

    let game_id = ArcAgi3GameId::new(game_name.to_string()).unwrap();

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap();

    let initial = observation(
        game_name,
        ArcAgi3GameState::NotFinished,
        7,
        vec![ArcAgi3ActionId::Action3],
        None,
    );

    let terminal = observation(
        game_name,
        ArcAgi3GameState::Win,
        8,
        vec![ArcAgi3ActionId::Action3],
        Some(action),
    );

    let scorecard = B4b2ScorecardTransport::new("b4b2-card-terminal");

    let mut session = ArcAgi3CompetitionSession::open(scorecard, &metadata()).unwrap();

    assert_eq!(session.card_id().as_str(), "b4b2-card-terminal");

    let environment = B4b2EnvironmentTransport::new(initial, vec![terminal]);

    let mut game = session
        .start_game(environment, &game_id, 9_400_000)
        .unwrap();

    assert_eq!(game.card_id().as_str(), "b4b2-card-terminal");

    assert_eq!(
        game.runtime().transport().started_card_id(),
        Some("b4b2-card-terminal"),
        "competition card identity must be forwarded unchanged to the environment session",
    );

    /*
     * This is a genuine zero-history competition start.
     *
     * One observation alone must NOT magically become a fully
     * scene-grounded B0 world representation.
     */
    assert!(
        game.runtime()
            .cognitive_runtime()
            .current_grounded_world_state()
            .is_none(),
        "fresh competition start must not fabricate strict B0 scene grounding",
    );

    let goal = c16i::live_goal();

    let production_policy = production_policy(&goal);

    let episode_policy = ArcAgi3SuccessorEpisodePolicy::new(4, 2).unwrap();

    let transitions_before = game
        .runtime()
        .cognitive_runtime()
        .cognition()
        .transition_episode_count();

    let perceptual_records_before = game
        .runtime()
        .cognitive_runtime()
        .cognition()
        .perceptual_temporal_record_count();

    let mut trace = B4b2TraceCollector::default();

    /*
     * B4B1/B4B2 production execution must be able to bootstrap from
     * directly observed frame facts without pretending a learned
     * scene/world model already exists.
     */
    let result = game
        .run_production_successor_bounded_with_trace(episode_policy, production_policy, &mut trace)
        .unwrap();

    assert_eq!(
        result.termination(),
        ArcAgi3SuccessorEpisodeTermination::Won,
    );

    assert_eq!(result.decision_attempts(), 1);
    assert_eq!(result.executed_steps(), 1);
    assert_eq!(result.abstentions(), 0);

    assert_eq!(
        result.final_status(),
        crate::live_environment_runtime::ArcAgi3LiveEnvironmentStatus::Won,
    );

    assert_eq!(game.runtime().transport().execute_count(), 1,);

    assert_eq!(
        game.runtime().transport().executed_actions(),
        &[action],
        "the only concrete B4A affordance must be the real executed command",
    );

    /*
     * Because the source was not yet a strict scene-grounded B0
     * representation, this first intervention may not be promoted
     * retroactively into a fabricated M47 transformation episode.
     */
    assert_eq!(
        game.runtime()
            .cognitive_runtime()
            .cognition()
            .transition_episode_count(),
        transitions_before,
        "cold-start bootstrap must not fabricate a scene-grounded M47 transition episode",
    );

    /*
     * The intervention was nevertheless real. Its cross-frame
     * perceptual consequence must become retained empirical evidence.
     */
    assert!(
        game.runtime()
            .cognitive_runtime()
            .cognition()
            .perceptual_temporal_record_count()
            > perceptual_records_before,
        "the real bootstrap intervention must retain real cross-frame perceptual evidence",
    );

    assert_eq!(trace.0.len(), 1);

    match &trace.0[0] {
        ArcAgi3CognitiveTraceEvent::Executed { authority, .. } => {
            assert_eq!(
                authority.kind,
                ArcAgi3TraceAuthorityKind::IgnoranceExploration,
            );
        }

        other => {
            panic!("expected production ignorance execution trace, got {other:?}");
        }
    }

    let runtime = game.finish();

    assert_eq!(runtime.completed_cognitive_step_count(), 1,);

    /*
     * Finishing the environment runtime must not implicitly mutate
     * the competition scorecard lifecycle.
     */
    assert_eq!(session.status(), ArcAgi3CompetitionSessionStatus::Open,);

    assert_eq!(session.card_id().as_str(), "b4b2-card-terminal",);

    let summary = session.close().unwrap();

    assert_eq!(summary.card_id().as_str(), "b4b2-card-terminal",);

    assert_eq!(summary.competition_mode(), Some(true),);

    assert!(summary.published_at().is_some(),);

    assert_eq!(session.status(), ArcAgi3CompetitionSessionStatus::Closed,);

    assert_eq!(session.scorecard_transport().open_count, 1,);

    assert_eq!(session.scorecard_transport().close_count, 1,);
}

#[test]
fn b4b2_production_runner_obeys_successor_decision_budget_without_hidden_retry() {
    let game_name = "b4b2-production-budget";

    let game_id = ArcAgi3GameId::new(game_name.to_string()).unwrap();

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action4).unwrap();

    let initial = observation(
        game_name,
        ArcAgi3GameState::NotFinished,
        4,
        vec![ArcAgi3ActionId::Action4],
        None,
    );

    let active_response = observation(
        game_name,
        ArcAgi3GameState::NotFinished,
        5,
        vec![ArcAgi3ActionId::Action4],
        Some(action),
    );

    let mut session = ArcAgi3CompetitionSession::open(
        B4b2ScorecardTransport::new("b4b2-card-budget"),
        &metadata(),
    )
    .unwrap();

    let environment = B4b2EnvironmentTransport::new(initial, vec![active_response]);

    let mut game = session
        .start_game(environment, &game_id, 9_500_000)
        .unwrap();

    let goal = c16i::live_goal();

    let result = game
        .run_production_successor_bounded(
            ArcAgi3SuccessorEpisodePolicy::new(1, 2).unwrap(),
            production_policy(&goal),
        )
        .unwrap();

    assert_eq!(
        result.termination(),
        ArcAgi3SuccessorEpisodeTermination::DecisionBudgetExhausted,
    );

    assert_eq!(
        result.decision_attempts(),
        1,
        "production competition runner must not exceed the inherited successor decision budget",
    );

    assert!(
        game.runtime().transport().execute_count() <= 1,
        "one decision attempt can cause at most one environment command",
    );

    assert_eq!(
        result.executed_steps() + result.abstentions(),
        1,
        "every bounded decision attempt must resolve as exactly one execution or abstention",
    );
}

#[test]
fn b4b2_terminal_start_never_invokes_production_callback_or_fake_command() {
    let game_name = "b4b2-terminal-start";

    let game_id = ArcAgi3GameId::new(game_name.to_string()).unwrap();

    let terminal = observation(
        game_name,
        ArcAgi3GameState::Win,
        9,
        vec![ArcAgi3ActionId::Action3],
        None,
    );

    let mut session = ArcAgi3CompetitionSession::open(
        B4b2ScorecardTransport::new("b4b2-card-terminal-start"),
        &metadata(),
    )
    .unwrap();

    let environment = B4b2EnvironmentTransport::new(terminal, Vec::new());

    let mut game = session
        .start_game(environment, &game_id, 9_600_000)
        .unwrap();

    let goal = c16i::live_goal();

    let result = game
        .run_production_successor_bounded(
            ArcAgi3SuccessorEpisodePolicy::new(4, 2).unwrap(),
            production_policy(&goal),
        )
        .unwrap();

    assert_eq!(
        result.termination(),
        ArcAgi3SuccessorEpisodeTermination::Won,
    );

    assert_eq!(result.decision_attempts(), 0);
    assert_eq!(result.executed_steps(), 0);
    assert_eq!(result.abstentions(), 0);

    assert_eq!(
        game.runtime().transport().execute_count(),
        0,
        "terminal competition observation must not generate a production command",
    );
}
