//! Tests live here to assemble the existing mature C16I fixture without exposing
//! mutable cognition or a synthetic authority constructor in production.
use super::*;
use crate::cognitive_interaction_runtime::c16i_successor_informed_two_contract_e2e_tests as c16i;
use crate::cognitive_trace::{
    ArcAgi3CognitiveTraceEvent as Event, ArcAgi3JsonlTraceSink, ArcAgi3TraceStructure,
};
use crate::successor_episode_runtime::{
    ArcAgi3SuccessorEpisodeError as Error, ArcAgi3SuccessorEpisodePolicy as Policy,
    ArcAgi3SuccessorEpisodeRuntime as Driver, ArcAgi3SuccessorEpisodeTermination as Termination,
};
use crate::ArcAgi3Observation;
use std::collections::VecDeque;

struct Transport {
    responses: VecDeque<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    actions: Vec<ArcAgi3Action>,
}

impl ArcAgi3EnvironmentTransport for Transport {
    fn start_game(
        &mut self,
        _: &ArcAgi3GameId,
        _: &str,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        Err(ArcAgi3TransportError::ActiveSessionExists)
    }
    fn execute(
        &mut self,
        command: &ArcAgi3SessionCommand,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        self.actions.push(command.action());
        self.responses
            .pop_front()
            .expect("no unplanned transport call")
    }
}

type Live = ArcAgi3LiveEnvironmentRuntime<Transport>;

struct Fixture {
    live: Live,
    action: ArcAgi3Action,
    cognitive_action: CognitiveStructure,
    source: CognitiveStructure,
    possibilities: Vec<athlesia_autonomous_active_experimentation::GroundedExperimentPossibility>,
    beliefs: Vec<athlesia_autonomous_active_experimentation::HypothesisBeliefState>,
}

fn with_state(observation: &ArcAgi3Observation, state: ArcAgi3GameState) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        observation.game_id().clone(),
        state,
        observation.frames().clone(),
        observation.levels_completed(),
        observation.win_levels(),
        observation.available_actions().clone(),
        observation.last_action(),
    )
}

fn fixture(state: ArcAgi3GameState) -> Fixture {
    let game = "successor-episode";
    let (cognitive_runtime, action, cognitive_action, source, possibilities, beliefs) =
        c16i::fixture(game, 8_300_000).into_live_parts();
    let response = with_state(&c16i::live_response(game, 6, Some(action)), state);
    Fixture {
        live: Live {
            transport: Transport {
                responses: VecDeque::from([Ok(response)]),
                actions: Vec::new(),
            },
            cognitive_runtime,
            completed_cognitive_step_count: 0,
            completed_reset_count: 0,
            faulted_pending: false,
            fault_disposition: None,
        },
        action,
        cognitive_action,
        source,
        possibilities,
        beliefs,
    }
}

fn signal() -> CognitiveSignal {
    CognitiveSignal::new(900).unwrap()
}
fn events(sink: &ArcAgi3JsonlTraceSink<Vec<u8>>) -> Vec<Event> {
    std::str::from_utf8(&sink.writer)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn policy_requires_both_nonzero_limits() {
    assert_eq!(Policy::new(0, 3), None);
    assert_eq!(Policy::new(3, 0), None);
    assert_eq!(Policy::new(0, 0), None);
    let policy = Policy::new(2, 3).unwrap();
    assert_eq!(policy.max_decision_attempts(), 2);
    assert_eq!(policy.max_consecutive_abstentions(), 3);
}

#[test]
fn one_real_successor_step_preserves_exact_provenance_and_stops_at_terminal_or_budget() {
    for (state, termination, status) in [
        (
            ArcAgi3GameState::NotFinished,
            Termination::DecisionBudgetExhausted,
            ArcAgi3LiveEnvironmentStatus::Active,
        ),
        (
            ArcAgi3GameState::Win,
            Termination::Won,
            ArcAgi3LiveEnvironmentStatus::Won,
        ),
        (
            ArcAgi3GameState::GameOver,
            Termination::GameOver,
            ArcAgi3LiveEnvironmentStatus::GameOver,
        ),
    ] {
        let mut fixture = fixture(state);
        let goal = c16i::live_goal();
        let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
            c16i::live_request(&[], &goal, &fixture.possibilities, &fixture.beliefs),
            signal(),
        );
        let mut sink = ArcAgi3JsonlTraceSink::new(Vec::new());
        let max_attempts = if state.is_terminal() { 8 } else { 1 };
        let result = Driver::run_with(
            &mut fixture.live,
            Policy::new(max_attempts, 3).unwrap(),
            |live| live.execute_successor_informed_unified_with_trace(request, &mut sink),
        )
        .unwrap();
        assert_eq!(result.termination(), termination);
        assert_eq!(result.final_status(), status);
        assert_eq!(result.decision_attempts(), 1);
        assert_eq!(result.executed_steps(), 1);
        assert_eq!(result.abstentions(), 0);
        assert_eq!(result.starting_completed_cognitive_step_count(), 0);
        assert_eq!(result.ending_completed_cognitive_step_count(), 1);
        assert_eq!(fixture.live.transport.actions, vec![fixture.action]);
        let records = events(&sink);
        assert_eq!(records.len(), 1);
        let Event::Executed { authority, .. } = &records[0] else {
            panic!("expected execution");
        };
        assert_eq!(
            authority.source_state,
            ArcAgi3TraceStructure::from(&fixture.source)
        );
        assert_eq!(
            authority.cognitive_action,
            ArcAgi3TraceStructure::from(&fixture.cognitive_action)
        );
        if state.is_terminal() {
            let repeated = Driver::run_with(&mut fixture.live, Policy::new(5, 3).unwrap(), |_| {
                panic!("terminal state must not dispatch")
            })
            .unwrap();
            assert_eq!(repeated.termination(), termination);
            assert_eq!(repeated.final_status(), status);
            assert_eq!(repeated.decision_attempts(), 0);
            assert_eq!(repeated.starting_completed_cognitive_step_count(), 1);
            assert_eq!(repeated.ending_completed_cognitive_step_count(), 1);
        }
    }
}

#[test]
fn abstention_budgets_are_exact_and_never_create_an_action() {
    for (attempt_limit, abstention_limit, expected_attempts, termination) in [
        (8, 3, 3, Termination::ConsecutiveAbstentionBudgetExhausted),
        (2, 3, 2, Termination::DecisionBudgetExhausted),
        (3, 3, 3, Termination::ConsecutiveAbstentionBudgetExhausted),
    ] {
        let mut fixture = fixture(ArcAgi3GameState::NotFinished);
        let cognition = fixture.live.cognitive_runtime.clone();
        let goal = c16i::live_goal();
        let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
            c16i::live_request(&[], &goal, &[], &[]),
            signal(),
        );
        let mut sink = ArcAgi3JsonlTraceSink::new(Vec::new());
        let result = Driver::run_with(
            &mut fixture.live,
            Policy::new(attempt_limit, abstention_limit).unwrap(),
            |live| live.execute_successor_informed_unified_with_trace(request, &mut sink),
        )
        .unwrap();
        assert_eq!(result.termination(), termination);
        assert_eq!(result.decision_attempts(), expected_attempts);
        assert_eq!(result.abstentions(), expected_attempts);
        assert_eq!(result.executed_steps(), 0);
        assert_eq!(result.starting_completed_cognitive_step_count(), 0);
        assert_eq!(result.ending_completed_cognitive_step_count(), 0);
        assert_eq!(result.final_status(), ArcAgi3LiveEnvironmentStatus::Active);
        assert_eq!(fixture.live.cognitive_runtime(), &cognition);
        assert!(!fixture
            .live
            .cognitive_runtime()
            .session()
            .has_pending_command());
        assert!(fixture.live.transport.actions.is_empty());
        let records = events(&sink);
        assert_eq!(records.len(), expected_attempts);
        assert!(records
            .iter()
            .all(|event| matches!(event, Event::Abstained { .. })));
        for line in std::str::from_utf8(&sink.writer).unwrap().lines() {
            let json: serde_json::Value = serde_json::from_str(line).unwrap();
            assert!(json.get("authority").is_none());
            assert!(json.get("outcome").is_none());
            assert!(json.get("command_action").is_none());
        }
    }
}

#[test]
fn mixed_attempts_match_manual_traced_calls_and_execution_resets_abstention_streak() {
    // Two abstentions, one genuine M50 execution, two more abstentions: a
    // streak limit of three must not stop this five-attempt episode early.
    let mut fixture = fixture(ArcAgi3GameState::NotFinished);
    let response = fixture.live.transport.responses.pop_front().unwrap();
    let mut manual = Live {
        transport: Transport {
            responses: VecDeque::from([response]),
            actions: Vec::new(),
        },
        cognitive_runtime: fixture.live.cognitive_runtime.clone(),
        completed_cognitive_step_count: 17,
        completed_reset_count: 0,
        faulted_pending: false,
        fault_disposition: None,
    };
    fixture
        .live
        .transport
        .responses
        .push_back(Ok(c16i::live_response(
            "successor-episode",
            6,
            Some(fixture.action),
        )));
    // A nonzero live-counter baseline exercises episode-relative accounting.
    fixture.live.completed_cognitive_step_count = 17;
    let goal = c16i::live_goal();
    let active = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
        c16i::live_request(&[], &goal, &fixture.possibilities, &fixture.beliefs),
        signal(),
    );
    let abstain = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
        c16i::live_request(&[], &goal, &[], &[]),
        signal(),
    );
    let sequence = [abstain, abstain, active, abstain, abstain];
    let mut manual_sink = ArcAgi3JsonlTraceSink::new(Vec::new());
    for (index, request) in sequence.iter().enumerate() {
        let step = manual
            .execute_successor_informed_unified_with_trace(*request, &mut manual_sink)
            .unwrap();
        assert_eq!(step.is_some(), index == 2);
    }
    let mut sink = ArcAgi3JsonlTraceSink::new(Vec::new());
    let mut requests = sequence.into_iter();
    let result = Driver::run_with(&mut fixture.live, Policy::new(5, 3).unwrap(), |live| {
        live.execute_successor_informed_unified_with_trace(
            requests.next().expect("bounded callback"),
            &mut sink,
        )
    })
    .unwrap();
    assert_eq!(result.termination(), Termination::DecisionBudgetExhausted);
    assert_eq!(result.decision_attempts(), 5);
    assert_eq!(result.executed_steps(), 1);
    assert_eq!(result.abstentions(), 4);
    assert_eq!(result.starting_completed_cognitive_step_count(), 17);
    assert_eq!(result.ending_completed_cognitive_step_count(), 18);
    assert_eq!(fixture.live.cognitive_runtime(), manual.cognitive_runtime());
    assert_eq!(fixture.live.transport.actions, manual.transport.actions);
    assert_eq!(
        fixture.live.completed_cognitive_step_count(),
        manual.completed_cognitive_step_count()
    );
    assert_eq!(sink.writer, manual_sink.writer);
    assert_eq!(events(&sink).len(), 5);
}

#[test]
fn transport_failure_is_traced_once_propagated_exactly_and_never_retried() {
    let mut fixture = fixture(ArcAgi3GameState::NotFinished);
    fixture.live.transport.responses =
        VecDeque::from([Err(ArcAgi3TransportError::HttpTransport {
            message: "indeterminate test dispatch".into(),
            disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
        })]);
    let goal = c16i::live_goal();
    let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
        c16i::live_request(&[], &goal, &fixture.possibilities, &fixture.beliefs),
        signal(),
    );
    let mut sink = ArcAgi3JsonlTraceSink::new(Vec::new());
    let error = Driver::run_with(&mut fixture.live, Policy::new(8, 3).unwrap(), |live| {
        live.execute_successor_informed_unified_with_trace(request, &mut sink)
    })
    .unwrap_err();
    assert_eq!(
        error,
        Error::StepFailed {
            decision_attempts: 1,
            executed_steps: 0,
            error: ArcAgi3LiveEnvironmentError::Transport(ArcAgi3TransportError::HttpTransport {
                message: "indeterminate test dispatch".into(),
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
            }),
        }
    );
    assert_eq!(fixture.live.transport.actions.len(), 1);
    assert_eq!(fixture.live.completed_cognitive_step_count(), 0);
    assert_eq!(
        fixture.live.status(),
        ArcAgi3LiveEnvironmentStatus::FaultedPending
    );
    assert!(matches!(
        events(&sink).as_slice(),
        [Event::TransportFailure { .. }]
    ));
    let again = Driver::run_with(&mut fixture.live, Policy::new(8, 3).unwrap(), |_| {
        panic!("no retry")
    });
    assert_eq!(
        again,
        Err(Error::RuntimeFaultedPending(Some(
            ArcAgi3TransportFailureDisposition::DispatchIndeterminate
        )))
    );
    assert_eq!(events(&sink).len(), 1);
}

#[test]
fn invalid_executor_effects_fail_closed() {
    let policy = Policy::new(4, 3).unwrap();
    let mut fixture = fixture(ArcAgi3GameState::NotFinished);
    let error = Driver::run_with(&mut fixture.live, policy, |live| {
        live.completed_cognitive_step_count += 1;
        Ok(None)
    })
    .unwrap_err();
    assert_eq!(
        error,
        Error::AbstentionChangedCompletedStepCount {
            before: 0,
            after: 1
        }
    );

    let mut fixture = self::fixture(ArcAgi3GameState::NotFinished);
    let error = Driver::run_with(&mut fixture.live, policy, |live| {
        live.cognitive_runtime.begin_reset().unwrap();
        Ok(None)
    })
    .unwrap_err();
    assert_eq!(error, Error::AbstentionCreatedPendingCommand);
    assert!(fixture.live.transport.actions.is_empty());
    assert_eq!(
        Driver::run_with(&mut fixture.live, policy, |_| panic!(
            "pending command blocks attempt"
        )),
        Err(Error::RuntimeHasPendingCommand)
    );

    let mut fixture = self::fixture(ArcAgi3GameState::NotFinished);
    let goal = c16i::live_goal();
    let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
        c16i::live_request(&[], &goal, &fixture.possibilities, &fixture.beliefs),
        signal(),
    );
    let error = Driver::run_with(&mut fixture.live, policy, |live| {
        let step = live.execute_successor_informed_unified(request)?;
        live.completed_cognitive_step_count += 1;
        Ok(step)
    })
    .unwrap_err();
    assert_eq!(
        error,
        Error::CompletedCognitiveStepMismatch {
            expected: 1,
            actual: 2
        }
    );
}

#[test]
fn not_started_and_unexpected_post_step_states_are_rejected() {
    let mut fixture = fixture(ArcAgi3GameState::NotFinished);
    let not_started = with_state(
        fixture.live.cognitive_runtime.observation(),
        ArcAgi3GameState::NotPlayed,
    );
    fixture.live.cognitive_runtime =
        ArcAgi3CognitiveInteractionRuntime::new(not_started, 0).unwrap();
    assert_eq!(
        Driver::run_with(&mut fixture.live, Policy::new(1, 1).unwrap(), |_| panic!(
            "not runnable"
        )),
        Err(Error::RuntimeNotRunnable(
            ArcAgi3LiveEnvironmentStatus::NotStarted
        ))
    );

    let mut fixture = self::fixture(ArcAgi3GameState::NotPlayed);
    let goal = c16i::live_goal();
    let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
        c16i::live_request(&[], &goal, &fixture.possibilities, &fixture.beliefs),
        signal(),
    );
    assert_eq!(
        Driver::run_with(&mut fixture.live, Policy::new(1, 1).unwrap(), |live| live
            .execute_successor_informed_unified(request)),
        Err(Error::UnexpectedPostStepStatus(
            ArcAgi3LiveEnvironmentStatus::NotStarted
        ))
    );
}

#[test]
fn repeated_real_executions_advance_only_the_completed_step_counter() {
    let mut fixture = fixture(ArcAgi3GameState::NotFinished);
    // First use the real native M50 authority, then the same fixture's learned
    // exploitation evidence. Caller policies are test inputs, not new evidence.
    fixture.live.transport.responses = [6, 5, 6]
        .into_iter()
        .map(|value| {
            Ok(c16i::live_response(
                "successor-episode",
                value,
                Some(fixture.action),
            ))
        })
        .collect();
    let goal = c16i::live_goal();
    let native = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
        c16i::live_request(&[], &goal, &fixture.possibilities, &fixture.beliefs),
        signal(),
    );
    let actions = [fixture.action];
    let mut executive = c16i::live_request(&actions, &goal, &[], &[]);
    // Existing evidence-authority policy from p4g_functional_live_unified_dispatch.
    use athlesia_executive_agency::{
        ExecutiveAgencyPolicy, ExecutiveSelectionThresholds, ExecutiveUtilityWeights,
    };
    let bounded = |value| CognitiveSignal::new(value).unwrap();
    executive.executive_policy = ExecutiveAgencyPolicy::new(
        1,
        8,
        16,
        1,
        ExecutiveUtilityWeights::new(0, 0, 1000, 0, 0).unwrap(),
        ExecutiveSelectionThresholds::new(
            bounded(100),
            bounded(100),
            bounded(1),
            bounded(600),
            bounded(100),
        )
        .unwrap(),
    )
    .unwrap();
    let exploitation = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(executive, signal());
    let mut requests = [native, exploitation, exploitation].into_iter();
    let mut sink = ArcAgi3JsonlTraceSink::new(Vec::new());
    let result = Driver::run_with(&mut fixture.live, Policy::new(3, 3).unwrap(), |live| {
        live.execute_successor_informed_unified_with_trace(requests.next().unwrap(), &mut sink)
    })
    .unwrap();
    assert_eq!(result.termination(), Termination::DecisionBudgetExhausted);
    assert_eq!(result.decision_attempts(), 3);
    assert_eq!(result.executed_steps(), 3);
    assert_eq!(result.abstentions(), 0);
    assert_eq!(
        result.ending_completed_cognitive_step_count(),
        result.starting_completed_cognitive_step_count() + 3
    );
    assert_eq!(fixture.live.transport.actions.len(), 3);
    assert!(events(&sink)
        .iter()
        .all(|event| matches!(event, Event::Executed { .. })));
    assert_eq!(events(&sink).len(), 3);
}

#[test]
fn competition_game_delegates_successor_execution_without_extra_scorecard_operations() {
    use crate::competition_session_runtime::*;
    struct Scorecard {
        opens: usize,
        closes: usize,
    }
    impl ArcAgi3ScorecardTransport for Scorecard {
        fn open_scorecard(
            &mut self,
            _: &ArcAgi3CompetitionMetadata,
        ) -> Result<ArcAgi3ScorecardId, ArcAgi3CompetitionTransportError> {
            self.opens += 1;
            Ok(ArcAgi3ScorecardId::new("successor-card".into()).unwrap())
        }
        fn get_scorecard(
            &mut self,
            _: &ArcAgi3ScorecardId,
        ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError> {
            panic!("driver must not query scorecards");
        }
        fn close_scorecard(
            &mut self,
            card: &ArcAgi3ScorecardId,
        ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError> {
            self.closes += 1;
            Ok(
                ArcAgi3ScorecardRestProtocol::decode_summary(serde_json::json!({
                    "card_id": card.as_str(), "score": 0.0, "environments": [],
                    "total_environments_completed": 0, "total_environments": 1,
                    "total_levels_completed": 0, "total_levels": 1, "total_actions": 1,
                }))
                .unwrap(),
            )
        }
    }
    struct Startable(Option<ArcAgi3Observation>, Transport);
    impl ArcAgi3EnvironmentTransport for Startable {
        fn start_game(
            &mut self,
            _: &ArcAgi3GameId,
            card: &str,
        ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
            assert_eq!(card, "successor-card");
            Ok(self.0.take().expect("exactly one start"))
        }
        fn execute(
            &mut self,
            command: &ArcAgi3SessionCommand,
        ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
            self.1.execute(command)
        }
    }
    let fixture = fixture(ArcAgi3GameState::Win);
    let observation = fixture.live.cognitive_runtime.observation().clone();
    let game_id = observation.game_id().clone();
    let metadata = ArcAgi3CompetitionMetadata::new(None, Vec::new(), None).unwrap();
    let mut session = ArcAgi3CompetitionSession::open(
        Scorecard {
            opens: 0,
            closes: 0,
        },
        &metadata,
    )
    .unwrap();
    let environment = Startable(Some(observation), fixture.live.transport);
    let mut game = session
        .start_game(environment, &game_id, 8_300_000)
        .unwrap();
    // Test-only assembly with the real mature C16I cognition, as in B3A. No
    // production mutable-cognition access or request provider is introduced.
    game.runtime_mut().cognitive_runtime = fixture.live.cognitive_runtime;
    let goal = c16i::live_goal();
    let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
        c16i::live_request(&[], &goal, &fixture.possibilities, &fixture.beliefs),
        signal(),
    );
    let mut sink = ArcAgi3JsonlTraceSink::new(Vec::new());
    let result = game
        .run_successor_bounded_with(Policy::new(5, 3).unwrap(), |live| {
            live.execute_successor_informed_unified_with_trace(request, &mut sink)
        })
        .unwrap();
    assert_eq!(result.termination(), Termination::Won);
    assert_eq!(result.decision_attempts(), 1);
    assert_eq!(game.runtime().transport().1.actions, vec![fixture.action]);
    assert_eq!(events(&sink).len(), 1);
    let live = game.finish();
    assert_eq!(live.completed_cognitive_step_count(), 1);
    assert_eq!(session.scorecard_transport().opens, 1);
    assert_eq!(session.scorecard_transport().closes, 0);
    assert_eq!(session.status(), ArcAgi3CompetitionSessionStatus::Open);
    session.close().unwrap();
    assert_eq!(session.scorecard_transport().closes, 1);
}
