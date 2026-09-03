use std::cell::Cell;
use std::collections::VecDeque;

use athlesia_arc_agi_3_adapter::bounded_episode_runtime::*;
use athlesia_arc_agi_3_adapter::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge;
use athlesia_arc_agi_3_adapter::environment_transport_boundary::{
    ArcAgi3EnvironmentTransport, ArcAgi3TransportError, ArcAgi3TransportFailureDisposition,
};
use athlesia_arc_agi_3_adapter::live_environment_runtime::{
    ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError, ArcAgi3LiveEnvironmentRuntime,
    ArcAgi3LiveEnvironmentStatus,
};
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
    value: u8,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new(game_id.to_string()).unwrap(),
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

#[derive(Debug)]
struct ScriptedTransport {
    initial: Option<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    responses: VecDeque<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    start_count: Cell<usize>,
    execute_count: Cell<usize>,
}

impl ScriptedTransport {
    fn new(
        initial: ArcAgi3Observation,
        responses: Vec<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    ) -> Self {
        Self {
            initial: Some(Ok(initial)),
            responses: responses.into_iter().collect(),
            start_count: Cell::new(0),
            execute_count: Cell::new(0),
        }
    }

    fn execute_count(&self) -> usize {
        self.execute_count.get()
    }
}

impl ArcAgi3EnvironmentTransport for ScriptedTransport {
    fn start_game(
        &mut self,
        _game_id: &ArcAgi3GameId,
        _card_id: &str,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        self.start_count.set(
            self.start_count
                .get()
                .checked_add(1)
                .expect("scripted start counter remains bounded"),
        );

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
                .expect("scripted execute counter remains bounded"),
        );

        self.responses
            .pop_front()
            .unwrap_or(Err(ArcAgi3TransportError::NoActiveSession))
    }
}

fn action1() -> ArcAgi3Action {
    ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap()
}

fn runtime(
    game_id: &str,
    initial_state: ArcAgi3GameState,
    responses: Vec<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
) -> ArcAgi3LiveEnvironmentRuntime<ScriptedTransport> {
    let initial = observation(game_id, initial_state, 1, None);

    ArcAgi3LiveEnvironmentRuntime::start(
        ScriptedTransport::new(initial, responses),
        &ArcAgi3GameId::new("requested".to_string()).unwrap(),
        "episode-card",
        100,
    )
    .unwrap()
}

fn execute_action1(
    live: &mut ArcAgi3LiveEnvironmentRuntime<ScriptedTransport>,
) -> Result<ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError> {
    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action1());

    live.execute_with(signal(900), |cognitive| {
        m51_fixture::begin_arc(cognitive, cognitive_action)
    })
}

#[test]
fn bounded_episode_policy_rejects_zero_and_preserves_exact_limit() {
    assert_eq!(ArcAgi3BoundedEpisodePolicy::new(0), None,);

    let policy = ArcAgi3BoundedEpisodePolicy::new(17).unwrap();

    assert_eq!(policy.max_cognitive_steps(), 17,);
}

#[test]
fn episode_terminates_immediately_on_existing_win_without_executing_action() {
    let mut live = runtime("episode-initial-win", ArcAgi3GameState::Win, Vec::new());

    let mut calls = 0usize;

    let result = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(5).unwrap(),
        |_runtime| {
            calls += 1;
            unreachable!()
        },
    )
    .unwrap();

    assert_eq!(result.termination(), ArcAgi3BoundedEpisodeTermination::Won,);

    assert_eq!(result.completed_steps_in_episode(), 0,);

    assert_eq!(calls, 0);

    assert_eq!(live.transport().execute_count(), 0,);
}

#[test]
fn episode_terminates_immediately_on_existing_game_over_without_auto_reset() {
    let mut live = runtime(
        "episode-initial-over",
        ArcAgi3GameState::GameOver,
        Vec::new(),
    );

    let result = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(3).unwrap(),
        |_runtime| unreachable!(),
    )
    .unwrap();

    assert_eq!(
        result.termination(),
        ArcAgi3BoundedEpisodeTermination::GameOver,
    );

    assert_eq!(result.completed_steps_in_episode(), 0,);

    assert_eq!(live.completed_reset_count(), 0,);

    assert_eq!(live.transport().execute_count(), 0,);
}

#[test]
fn not_started_runtime_is_rejected_before_step_execution() {
    let mut live = runtime(
        "episode-not-started",
        ArcAgi3GameState::NotPlayed,
        Vec::new(),
    );

    let mut calls = 0usize;

    let result = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(2).unwrap(),
        |_runtime| {
            calls += 1;
            unreachable!()
        },
    );

    assert_eq!(
        result,
        Err(ArcAgi3BoundedEpisodeError::RuntimeNotRunnable(
            ArcAgi3LiveEnvironmentStatus::NotStarted,
        ),),
    );

    assert_eq!(calls, 0);
}

#[test]
fn real_m51_episode_runs_until_environment_win_and_preserves_step_history() {
    let action = action1();

    let mut live = runtime(
        "episode-win",
        ArcAgi3GameState::NotFinished,
        vec![
            Ok(observation(
                "episode-win",
                ArcAgi3GameState::NotFinished,
                2,
                Some(action),
            )),
            Ok(observation(
                "episode-win",
                ArcAgi3GameState::Win,
                3,
                Some(action),
            )),
        ],
    );

    let result = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(5).unwrap(),
        execute_action1,
    )
    .unwrap();

    assert_eq!(result.termination(), ArcAgi3BoundedEpisodeTermination::Won,);

    assert_eq!(result.final_status(), ArcAgi3LiveEnvironmentStatus::Won,);

    assert_eq!(result.completed_steps_in_episode(), 2,);

    assert_eq!(result.starting_completed_cognitive_step_count(), 0,);

    assert_eq!(result.ending_completed_cognitive_step_count(), 2,);

    assert_eq!(live.transport().execute_count(), 2,);

    assert_eq!(
        result.steps()[0].cognitive_step().command().action(),
        action,
    );

    assert_eq!(
        result.steps()[1].completion().turn().observation().state(),
        ArcAgi3GameState::Win,
    );
}

#[test]
fn episode_stops_on_game_over_without_hidden_reset() {
    let action = action1();

    let mut live = runtime(
        "episode-over",
        ArcAgi3GameState::NotFinished,
        vec![Ok(observation(
            "episode-over",
            ArcAgi3GameState::GameOver,
            2,
            Some(action),
        ))],
    );

    let result = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(4).unwrap(),
        execute_action1,
    )
    .unwrap();

    assert_eq!(
        result.termination(),
        ArcAgi3BoundedEpisodeTermination::GameOver,
    );

    assert_eq!(result.completed_steps_in_episode(), 1,);

    assert_eq!(live.completed_reset_count(), 0,);

    assert_eq!(live.transport().execute_count(), 1,);
}

#[test]
fn episode_step_budget_is_hard_upper_bound_with_no_extra_transport_call() {
    let action = action1();

    let mut live = runtime(
        "episode-budget",
        ArcAgi3GameState::NotFinished,
        vec![
            Ok(observation(
                "episode-budget",
                ArcAgi3GameState::NotFinished,
                2,
                Some(action),
            )),
            Ok(observation(
                "episode-budget",
                ArcAgi3GameState::NotFinished,
                3,
                Some(action),
            )),
            Ok(observation(
                "episode-budget",
                ArcAgi3GameState::Win,
                4,
                Some(action),
            )),
        ],
    );

    let result = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(2).unwrap(),
        execute_action1,
    )
    .unwrap();

    assert_eq!(
        result.termination(),
        ArcAgi3BoundedEpisodeTermination::StepBudgetExhausted,
    );

    assert_eq!(result.final_status(), ArcAgi3LiveEnvironmentStatus::Active,);

    assert_eq!(result.completed_steps_in_episode(), 2,);

    assert_eq!(live.transport().execute_count(), 2,);

    assert_eq!(live.completed_cognitive_step_count(), 2,);
}

#[test]
fn transport_failure_aborts_episode_at_exact_completed_step_count() {
    let action = action1();

    let mut live = runtime(
        "episode-failure",
        ArcAgi3GameState::NotFinished,
        vec![
            Ok(observation(
                "episode-failure",
                ArcAgi3GameState::NotFinished,
                2,
                Some(action),
            )),
            Err(ArcAgi3TransportError::HttpTransport {
                message: "ambiguous timeout".to_string(),
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
            }),
        ],
    );

    let result = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(5).unwrap(),
        execute_action1,
    );

    assert!(matches!(
        result,
        Err(ArcAgi3BoundedEpisodeError::StepFailed {
            completed_steps_in_episode: 1,
            error: ArcAgi3LiveEnvironmentError::Transport(ArcAgi3TransportError::HttpTransport {
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
                ..
            },),
        },),
    ));

    assert_eq!(live.status(), ArcAgi3LiveEnvironmentStatus::FaultedPending,);

    assert_eq!(live.completed_cognitive_step_count(), 1,);

    assert_eq!(live.transport().execute_count(), 2,);
}

#[test]
fn rerunning_faulted_episode_never_retries_pending_transport_action() {
    let mut live = runtime(
        "episode-faulted",
        ArcAgi3GameState::NotFinished,
        vec![Err(ArcAgi3TransportError::HttpTransport {
            message: "timeout".to_string(),
            disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
        })],
    );

    assert!(ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(3).unwrap(),
        execute_action1,
    )
    .is_err());

    assert_eq!(live.transport().execute_count(), 1,);

    let mut calls = 0usize;

    let second = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(3).unwrap(),
        |_runtime| {
            calls += 1;
            unreachable!()
        },
    );

    assert_eq!(
        second,
        Err(ArcAgi3BoundedEpisodeError::RuntimeFaultedPending(Some(
            ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
        ),),),
    );

    assert_eq!(calls, 0);

    assert_eq!(live.transport().execute_count(), 1,);
}

#[test]
fn episode_accounting_is_relative_to_existing_live_runtime_history() {
    let action = action1();

    let mut live = runtime(
        "episode-existing-history",
        ArcAgi3GameState::NotFinished,
        vec![
            Ok(observation(
                "episode-existing-history",
                ArcAgi3GameState::NotFinished,
                2,
                Some(action),
            )),
            Ok(observation(
                "episode-existing-history",
                ArcAgi3GameState::Win,
                3,
                Some(action),
            )),
        ],
    );

    execute_action1(&mut live).unwrap();

    assert_eq!(live.completed_cognitive_step_count(), 1,);

    let result = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(4).unwrap(),
        execute_action1,
    )
    .unwrap();

    assert_eq!(result.starting_completed_cognitive_step_count(), 1,);

    assert_eq!(result.ending_completed_cognitive_step_count(), 2,);

    assert_eq!(result.completed_steps_in_episode(), 1,);

    assert_eq!(result.termination(), ArcAgi3BoundedEpisodeTermination::Won,);
}

#[test]
fn every_completed_episode_step_contains_real_cognitive_feedback() {
    let action = action1();

    let mut live = runtime(
        "episode-feedback",
        ArcAgi3GameState::NotFinished,
        vec![
            Ok(observation(
                "episode-feedback",
                ArcAgi3GameState::NotFinished,
                2,
                Some(action),
            )),
            Ok(observation(
                "episode-feedback",
                ArcAgi3GameState::Win,
                3,
                Some(action),
            )),
        ],
    );

    let result = ArcAgi3BoundedEpisodeRuntime::run_with(
        &mut live,
        ArcAgi3BoundedEpisodePolicy::new(4).unwrap(),
        execute_action1,
    )
    .unwrap();

    assert_eq!(result.steps().len(), 2,);

    assert!(result
        .steps()
        .iter()
        .all(|step| step.completion().has_cognitive_feedback()));
}

#[test]
fn bounded_episode_runtime_contains_no_arc_action_semantics_or_auto_reset_policy() {
    let source = include_str!("../src/bounded_episode_runtime.rs");

    for forbidden in [
        "Action1",
        "Action2",
        "Action3",
        "Action4",
        "Action5",
        "Action6",
        "Action7",
        "begin_reset",
        ".reset(",
        "coordinate",
    ] {
        assert!(
            !source.contains(forbidden),
            "bounded episode runtime leaked forbidden semantic: {forbidden}",
        );
    }
}

#[test]
fn universal_facade_matches_direct_bounded_execution() {
    let action = action1();

    let responses = vec![Ok(observation(
        "episode-universal",
        ArcAgi3GameState::Win,
        2,
        Some(action),
    ))];

    let mut direct = runtime(
        "episode-universal",
        ArcAgi3GameState::NotFinished,
        vec![Ok(observation(
            "episode-universal",
            ArcAgi3GameState::Win,
            2,
            Some(action),
        ))],
    );

    let mut facade = runtime(
        "episode-universal",
        ArcAgi3GameState::NotFinished,
        responses,
    );

    let policy = ArcAgi3BoundedEpisodePolicy::new(3).unwrap();

    let direct_result =
        ArcAgi3BoundedEpisodeRuntime::run_with(&mut direct, policy, execute_action1).unwrap();

    let facade_result =
        UniversalArcAgi3BoundedEpisodeRuntime::run_with(&mut facade, policy, execute_action1)
            .unwrap();

    assert_eq!(direct_result, facade_result,);

    assert_eq!(direct.status(), facade.status(),);

    assert_eq!(
        direct.completed_cognitive_step_count(),
        facade.completed_cognitive_step_count(),
    );
}
