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

    let bootstrap_coverage_before = game
        .runtime()
        .cognitive_runtime()
        .cognition()
        .bootstrap_action_coverage_event_count();

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

    assert_eq!(
        game.runtime()
            .cognitive_runtime()
            .cognition()
            .bootstrap_action_coverage_event_count(),
        bootstrap_coverage_before + 1,
        "a real bootstrap intervention must become one retained global action-coverage event",
    );

    assert_eq!(trace.0.len(), 1);

    match &trace.0[0] {
        ArcAgi3CognitiveTraceEvent::Executed { authority, .. } => {
            assert_eq!(
                authority.kind,
                ArcAgi3TraceAuthorityKind::BootstrapIgnoranceExploration,
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
fn b4b3_multistep_cold_start_uses_global_coverage_before_repeating_action() {
    let game_name = "b4b3-multistep-cold-start";

    let game_id = ArcAgi3GameId::new(game_name.to_string()).unwrap();

    let action_three = ArcAgi3Action::discrete(ArcAgi3ActionId::Action3).unwrap();

    let action_four = ArcAgi3Action::discrete(ArcAgi3ActionId::Action4).unwrap();

    /*
     * Deliberately reverse the supplied protocol order.
     *
     * The test must not depend on caller order. M48 owns the
     * deterministic tie-break.
     */
    let available = vec![ArcAgi3ActionId::Action4, ArcAgi3ActionId::Action3];

    let initial = observation(
        game_name,
        ArcAgi3GameState::NotFinished,
        1,
        available.clone(),
        None,
    );

    /*
     * No last_action echo is supplied.
     *
     * The session still has exact self-generated action provenance
     * from the pending unified executive command.
     */
    let first_response = observation(
        game_name,
        ArcAgi3GameState::NotFinished,
        2,
        available.clone(),
        None,
    );

    let second_response = observation(game_name, ArcAgi3GameState::NotFinished, 3, available, None);

    let mut session = ArcAgi3CompetitionSession::open(
        B4b2ScorecardTransport::new("b4b3-card-multistep"),
        &metadata(),
    )
    .unwrap();

    let environment = B4b2EnvironmentTransport::new(initial, vec![first_response, second_response]);

    let mut game = session
        .start_game(environment, &game_id, 9_700_000)
        .unwrap();

    /*
     * Absolute cold start:
     * there is no justified B0 scene/world representation yet.
     */
    assert!(
        game.runtime()
            .cognitive_runtime()
            .current_grounded_world_state()
            .is_none(),
        "multi-step regression must begin before strict B0 grounding",
    );

    assert_eq!(
        game.runtime()
            .cognitive_runtime()
            .cognition()
            .bootstrap_action_coverage_event_count(),
        0,
    );

    let goal = c16i::live_goal();

    let mut trace = B4b2TraceCollector::default();

    let result = game
        .run_production_successor_bounded_with_trace(
            ArcAgi3SuccessorEpisodePolicy::new(2, 2).unwrap(),
            production_policy(&goal),
            &mut trace,
        )
        .unwrap();

    /*
     * Both bounded decisions must become real commands.
     */
    assert_eq!(
        result.termination(),
        ArcAgi3SuccessorEpisodeTermination::DecisionBudgetExhausted,
    );

    assert_eq!(result.decision_attempts(), 2);
    assert_eq!(result.executed_steps(), 2);
    assert_eq!(result.abstentions(), 0);

    let executed = game.runtime().transport().executed_actions();

    assert_eq!(
        executed.len(),
        2,
        "two production decisions must produce exactly two real commands",
    );

    /*
     * This is the central behavioral invariant.
     *
     * After the first real action has count 1, the other available
     * action still has global count 0. Therefore the second bootstrap
     * selection must move to the globally untried action rather than
     * repeating the first one.
     */
    assert_ne!(
        executed[0],
        executed[1],
        "bootstrap global coverage must prevent immediate repetition while an untried action exists",
    );

    assert!(
        executed.contains(&action_three),
        "Action3 must be covered exactly once across the two-action frontier",
    );

    assert!(
        executed.contains(&action_four),
        "Action4 must be covered exactly once across the two-action frontier",
    );

    assert_eq!(
        game.runtime()
            .cognitive_runtime()
            .cognition()
            .bootstrap_action_coverage_event_count(),
        2,
        "both real bootstrap interventions must be retained",
    );

    let cognitive_three =
        crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
            action_three,
        );

    let cognitive_four =
        crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
            action_four,
        );

    let coverage = game
        .runtime()
        .cognitive_runtime()
        .cognition()
        .bootstrap_action_coverage();

    assert_eq!(
        coverage.global_action_sample_count(&cognitive_three,),
        Some(1),
        "Action3 global coverage must equal exactly one real self-generated event",
    );

    assert_eq!(
        coverage.global_action_sample_count(&cognitive_four,),
        Some(1),
        "Action4 global coverage must equal exactly one real self-generated event",
    );

    /*
     * Most importantly, both decisions must have occurred through
     * the PRE-GROUNDING authority path.
     *
     * If the second decision silently falls through to ordinary
     * state-grounded ignorance, this regression must fail.
     */
    assert_eq!(trace.0.len(), 2);

    for event in &trace.0 {
        match event {
            ArcAgi3CognitiveTraceEvent::Executed { authority, .. } => {
                assert_eq!(
                    authority.kind,
                    ArcAgi3TraceAuthorityKind::BootstrapIgnoranceExploration,
                    "both decisions must remain explicit bootstrap ignorance authority",
                );
            }

            other => {
                panic!("expected only executed bootstrap events, got {other:?}");
            }
        }
    }

    /*
     * The observation identity changed 1 -> 2 -> 3, but the action
     * coverage survived that change. This is exactly the B4A5A
     * global-coverage contract.
     */
    assert_eq!(game.runtime().completed_cognitive_step_count(), 2,);

    let _runtime = game.finish();

    assert_eq!(
        session.status(),
        ArcAgi3CompetitionSessionStatus::Open,
        "bounded cognitive execution must not close the competition scorecard",
    );
}

#[test]
fn b4b4_strict_grounding_disables_bootstrap_authority_even_when_global_coverage_disagrees() {
    let game = "b4b4-grounded-handoff";

    /*
     * Canonical mature fixture:
     *
     * - strict B0 grounding already exists;
     * - ACTION3/ACTION4 are both unseen in the exact current grounded
     *   source state;
     * - therefore normal grounded ignorance has a clean 0-vs-0
     *   deterministic decision.
     */
    let (mut runtime, actions, expected_source) =
        c16i::live_production_ignorance_fixture(game, 9_800_000);

    let current = runtime
        .current_grounded_world_state()
        .expect("B4B4 fixture must begin with strict B0 grounding");

    let cognitive_actions = actions
        .iter()
        .copied()
        .map(crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action)
        .collect::<Vec<_>>();

    let ignorance_policy =
        athlesia_executive_agency::IgnoranceExplorationSelectionPolicy::new(8).unwrap();

    /*
     * Establish the normal grounded winner before touching bootstrap
     * coverage.
     */
    let grounded_before = runtime
        .cognition()
        .current_selected_ignorance_exploration_action(
            &current,
            &cognitive_actions,
            ignorance_policy,
        )
        .expect("exact-state ignorance must select one unseen action");

    assert_eq!(
        grounded_before.source_state(),
        &expected_source,
        "grounded ignorance must preserve exact B0 source identity",
    );

    let grounded_action =
        crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::decode_action(
            grounded_before.action(),
        )
        .expect("selected grounded cognitive action must decode");

    let opposite_action = actions
        .iter()
        .copied()
        .find(|candidate| *candidate != grounded_action)
        .expect("two-action fixture must have an opposite action");

    /*
     * Deliberately make GLOBAL bootstrap coverage disagree with the
     * exact grounded selector.
     *
     * Give the grounded winner three historical bootstrap samples.
     * Leave the opposite action globally untried.
     *
     * If bootstrap coverage leaked across the B0 boundary, bootstrap
     * selection would now prefer `opposite_action`.
     */
    for event_index in 90_001_u64..=90_003_u64 {
        c16i::retain_bootstrap_coverage_for_test(&mut runtime, event_index, grounded_action);
    }

    assert_eq!(
        runtime
            .cognition()
            .bootstrap_action_coverage()
            .global_action_sample_count(grounded_before.action(),),
        Some(3),
    );

    let opposite_cognitive =
        crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
            opposite_action,
        );

    assert_eq!(
        runtime
            .cognition()
            .bootstrap_action_coverage()
            .global_action_sample_count(&opposite_cognitive,),
        Some(0),
    );

    /*
     * Prove the counterfactual explicitly:
     * if B4A5A bootstrap authority were consulted here, it would pick
     * the opposite action.
     */
    let bootstrap_counterfactual = runtime
        .cognition()
        .current_selected_bootstrap_ignorance_action(
            &CognitiveSignal::new(777)
                .map(|signal| {
                    athlesia_mindstone_sparse_cognition::CognitiveStructure::Ordered(vec![
                        athlesia_mindstone_sparse_cognition::CognitiveStructure::atom(
                            0x4234_4234_4F42_53,
                        ),
                        athlesia_mindstone_sparse_cognition::CognitiveStructure::atom(u64::from(
                            signal.value(),
                        )),
                    ])
                })
                .unwrap(),
            &cognitive_actions,
            ignorance_policy,
        )
        .expect("counterfactual bootstrap selector must have authority");

    let bootstrap_action =
        crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::decode_action(
            bootstrap_counterfactual.action(),
        )
        .unwrap();

    assert_eq!(
        bootstrap_action, opposite_action,
        "test setup must force bootstrap coverage to disagree with grounded ignorance",
    );

    /*
     * Re-evaluate normal exact-state ignorance after corrupting nothing
     * except the independent GLOBAL bootstrap-coverage memory.
     *
     * Its result must remain unchanged.
     */
    let grounded_after = runtime
        .cognition()
        .current_selected_ignorance_exploration_action(
            &current,
            &cognitive_actions,
            ignorance_policy,
        )
        .expect("grounded ignorance must remain available");

    assert_eq!(
        grounded_after, grounded_before,
        "global bootstrap coverage must not alter exact-state grounded ignorance",
    );

    let goal = c16i::live_goal();

    let request = c16i::live_evidence_faithful_request(&actions, &goal);

    /*
     * This is the real adapter authority boundary.
     *
     * Because strict B0 grounding exists, bootstrap authority must be
     * bypassed regardless of the contradictory global coverage counts.
     */
    let authority = runtime
        .current_evidence_faithful_successor_executive_authority(request)
        .expect("grounded evidence-faithful path must produce ignorance authority");

    assert_eq!(
        authority.kind(),
        crate::cognitive_interaction_runtime::
            ArcAgi3UnifiedExecutiveAuthorityKind::
                IgnoranceExploration,
        "strict B0 grounding must disable bootstrap authority",
    );

    assert!(
        authority.bootstrap_ignorance_selection().is_none(),
        "grounded authority must carry no bootstrap selection payload",
    );

    let grounded_selection = authority
        .ignorance_selection()
        .expect("grounded handoff must expose ordinary exact-state ignorance provenance");

    assert_eq!(
        grounded_selection, &grounded_before,
        "production authority must use the exact-state winner, not global bootstrap coverage",
    );

    assert_eq!(authority.action(), grounded_action,);

    assert_ne!(
        authority.action(),
        bootstrap_action,
        "a conflicting global bootstrap preference must have zero authority after B0 grounding",
    );

    assert_eq!(
        authority.source_state(),
        &expected_source,
        "post-bootstrap authority must return to strict grounded source provenance",
    );
}

#[test]
fn b4b5_real_production_bootstrap_autonomously_reaches_strict_grounding_and_hands_off() {
    fn groundable_observation(
        game: &str,
        value: u8,
        available: Vec<ArcAgi3ActionId>,
    ) -> ArcAgi3Observation {
        /*
         * Same deliberately simple visual structure used by the
         * established C16I grounding regressions:
         *
         *     value value
         *       8     9
         *
         * Only raw protocol-visible pixels are supplied.
         * No object, scene or world-state annotation enters here.
         */
        ArcAgi3Observation::new(
            ArcAgi3GameId::new(game.to_string()).unwrap(),
            ArcAgi3GameState::NotFinished,
            ArcAgi3FrameSequence::new(vec![ArcAgi3Grid::from_rows(vec![
                vec![value, value],
                vec![8, 9],
            ])
            .unwrap()])
            .unwrap(),
            0,
            3,
            ArcAgi3AvailableActions::new(available).unwrap(),
            /*
             * Deliberately omit environment action echo.
             *
             * Self-generated provenance still comes from the exact
             * pending unified executive command. The fixture therefore
             * does not need to know in advance which action M48 chose.
             */
            None,
        )
    }

    let game_name = "b4b5-autonomous-grounding";

    let game_id = ArcAgi3GameId::new(game_name.to_string()).unwrap();

    let available = vec![ArcAgi3ActionId::Action2, ArcAgi3ActionId::Action1];

    let initial = groundable_observation(game_name, 1, available.clone());

    /*
     * Reproduce the perceptual evidence progression used by the
     * established mature-runtime fixture, but this time every action
     * is selected by the real production authority path.
     *
     * Extra post-maturation observations give the live runtime room
     * to execute after strict grounding has appeared.
     */
    let values = [2_u8, 3, 4, 5, 5, 6, 6, 5, 5, 6, 6, 5, 6, 5];

    let responses = values
        .into_iter()
        .map(|value| groundable_observation(game_name, value, available.clone()))
        .collect::<Vec<_>>();

    let mut session = ArcAgi3CompetitionSession::open(
        B4b2ScorecardTransport::new("b4b5-card-autonomous-grounding"),
        &metadata(),
    )
    .unwrap();

    let environment = B4b2EnvironmentTransport::new(initial, responses);

    let mut game = session
        .start_game(environment, &game_id, 10_000_000)
        .unwrap();

    /*
     * This must be a genuine zero-history start.
     */
    assert!(
        game.runtime()
            .cognitive_runtime()
            .current_grounded_world_state()
            .is_none(),
        "B4B5 must begin without fabricated strict B0 grounding",
    );

    assert_eq!(
        game.runtime()
            .cognitive_runtime()
            .cognition()
            .bootstrap_action_coverage_event_count(),
        0,
    );

    assert_eq!(
        game.runtime()
            .cognitive_runtime()
            .cognition()
            .transition_episode_count(),
        0,
    );

    let goal = c16i::live_goal();

    let mut trace = B4b2TraceCollector::default();

    let result = game
        .run_production_successor_bounded_with_trace(
            ArcAgi3SuccessorEpisodePolicy::new(14, 14).unwrap(),
            production_policy(&goal),
            &mut trace,
        )
        .unwrap();

    assert_eq!(
        result.termination(),
        ArcAgi3SuccessorEpisodeTermination::DecisionBudgetExhausted,
        "non-terminal fixture must finish only because the explicit decision budget is exhausted",
    );

    assert_eq!(result.decision_attempts(), 14,);

    /*
     * This fixture is deliberately designed to remain actionable
     * throughout. An unexplained abstention would hide whether the
     * autonomous handoff itself works.
     */
    assert_eq!(
        result.executed_steps(),
        14,
        "every bounded production decision must become one real intervention in this fixture",
    );

    assert_eq!(
        result.abstentions(),
        0,
        "autonomous grounding fixture must not rely on abstention to cross the authority boundary",
    );

    assert_eq!(game.runtime().transport().execute_count(), 14,);

    /*
     * Critical result #1:
     *
     * production interaction alone must have accumulated enough
     * perceptual evidence for strict B0 grounding.
     */
    assert!(
        game.runtime()
            .cognitive_runtime()
            .current_grounded_world_state()
            .is_some(),
        "real production bootstrap interaction must autonomously produce strict B0 grounding",
    );

    /*
     * Extract only executed authority kinds.
     */
    let executed_kinds = trace
        .0
        .iter()
        .filter_map(|event| match event {
            ArcAgi3CognitiveTraceEvent::Executed { authority, .. } => Some(authority.kind),

            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(executed_kinds.len(), 14,);

    assert_eq!(
        executed_kinds.first(),
        Some(&ArcAgi3TraceAuthorityKind::BootstrapIgnoranceExploration,),
        "the first real decision must use explicit pre-grounding bootstrap authority",
    );

    let first_grounded_authority = executed_kinds
        .iter()
        .position(|kind| *kind != ArcAgi3TraceAuthorityKind::BootstrapIgnoranceExploration)
        .expect("production loop must eventually leave bootstrap authority");

    assert!(
        first_grounded_authority > 0,
        "handoff cannot occur before at least one real bootstrap intervention",
    );

    /*
     * Once this stable fixture reaches strict grounding, bootstrap must
     * not reappear. That would indicate authority oscillation or leaked
     * pre-grounding policy.
     */
    assert!(
        executed_kinds[
            first_grounded_authority..
        ]
        .iter()
        .all(
            |kind| {
                *kind
                    != ArcAgi3TraceAuthorityKind::
                        BootstrapIgnoranceExploration
            },
        ),
        "bootstrap authority must not reappear after autonomous strict grounding in the stable fixture",
    );

    let bootstrap_execution_count = executed_kinds
        .iter()
        .filter(|kind| **kind == ArcAgi3TraceAuthorityKind::BootstrapIgnoranceExploration)
        .count();

    assert!(bootstrap_execution_count > 0,);

    assert!(
        bootstrap_execution_count < 14,
        "the episode must contain both bootstrap and post-grounding decisions",
    );

    /*
     * Critical result #2:
     *
     * only pre-grounding actions enter the GLOBAL bootstrap coverage
     * owner. Grounded actions must not continue incrementing it.
     */
    assert_eq!(
        game.runtime()
            .cognitive_runtime()
            .cognition()
            .bootstrap_action_coverage_event_count(),
        bootstrap_execution_count,
        "global bootstrap coverage must stop growing exactly when bootstrap authority ends",
    );

    /*
     * Critical result #3:
     *
     * after the handoff there is enough strict scene continuity for
     * the normal M47 grounded transition-learning path to begin
     * retaining real transformation episodes.
     */
    assert!(
        game.runtime()
            .cognitive_runtime()
            .cognition()
            .transition_episode_count()
            > 0,
        "autonomous bootstrap -> B0 handoff must reach real grounded transition learning",
    );

    /*
     * We did not fabricate terminal state or mutate competition
     * lifecycle while proving cognition behavior.
     */
    assert_eq!(
        result.final_status(),
        crate::live_environment_runtime::ArcAgi3LiveEnvironmentStatus::Active,
    );

    let runtime = game.finish();

    assert_eq!(runtime.completed_cognitive_step_count(), 14,);

    assert_eq!(session.status(), ArcAgi3CompetitionSessionStatus::Open,);
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
