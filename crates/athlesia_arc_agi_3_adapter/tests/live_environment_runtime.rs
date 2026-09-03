use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use athlesia_arc_agi_3_adapter::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge;
use athlesia_arc_agi_3_adapter::environment_transport_boundary::{
    ArcAgi3EnvironmentTransport, ArcAgi3TransportError, ArcAgi3TransportFailureDisposition,
};
use athlesia_arc_agi_3_adapter::live_environment_runtime::*;
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

#[derive(Debug)]
struct ScriptedTransport {
    initial: Option<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    responses: RefCell<VecDeque<Result<ArcAgi3Observation, ArcAgi3TransportError>>>,
    start_count: Cell<usize>,
    execute_count: Cell<usize>,
}

impl ScriptedTransport {
    fn new(initial: ArcAgi3Observation) -> Self {
        Self {
            initial: Some(Ok(initial)),
            responses: RefCell::new(VecDeque::new()),
            start_count: Cell::new(0),
            execute_count: Cell::new(0),
        }
    }

    fn push(&self, response: Result<ArcAgi3Observation, ArcAgi3TransportError>) {
        self.responses.borrow_mut().push_back(response);
    }

    fn start_count(&self) -> usize {
        self.start_count.get()
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
            .borrow_mut()
            .pop_front()
            .unwrap_or(Err(ArcAgi3TransportError::NoActiveSession))
    }
}

fn active_runtime(
    game_id: &str,
    first_index: u64,
) -> ArcAgi3LiveEnvironmentRuntime<ScriptedTransport> {
    let initial = observation(
        game_id,
        ArcAgi3GameState::NotFinished,
        &[1],
        Some(ArcAgi3Action::reset()),
    );

    let transport = ScriptedTransport::new(initial);

    ArcAgi3LiveEnvironmentRuntime::start(
        transport,
        &ArcAgi3GameId::new("requested".to_string()).unwrap(),
        "card-live",
        first_index,
    )
    .unwrap()
}

fn begin_action1(
    runtime:
        &mut athlesia_arc_agi_3_adapter::
            cognitive_interaction_runtime::
            ArcAgi3CognitiveInteractionRuntime,
) -> Result<
    athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3CognitiveInteractionStep,
    athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3CognitiveInteractionError,
> {
    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action);

    m51_fixture::begin_arc(runtime, cognitive_action)
}

#[test]
fn live_runtime_starts_from_exact_transport_observation() {
    let initial = observation(
        "live-start",
        ArcAgi3GameState::NotFinished,
        &[2, 3],
        Some(ArcAgi3Action::reset()),
    );

    let transport = ScriptedTransport::new(initial.clone());

    let runtime = ArcAgi3LiveEnvironmentRuntime::start(
        transport,
        &ArcAgi3GameId::new("requested".to_string()).unwrap(),
        "card-start",
        100,
    )
    .unwrap();

    assert_eq!(runtime.status(), ArcAgi3LiveEnvironmentStatus::Active,);

    assert_eq!(runtime.cognitive_runtime().observation(), &initial,);

    assert_eq!(runtime.cognitive_runtime().perception().frame_count(), 2,);

    assert_eq!(
        runtime
            .cognitive_runtime()
            .next_perceptual_observation_index(),
        102,
    );

    assert_eq!(runtime.transport().start_count(), 1,);

    assert_eq!(runtime.completed_cognitive_step_count(), 0,);

    assert_eq!(runtime.completed_reset_count(), 0,);
}

#[test]
fn real_m51_step_executes_through_transport_and_binds_feedback() {
    let mut runtime = active_runtime("live-e2e", 200);

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    runtime.transport().push(Ok(observation(
        "live-e2e",
        ArcAgi3GameState::NotFinished,
        &[4, 5],
        Some(action),
    )));

    let result = runtime.execute_with(signal(900), begin_action1).unwrap();

    assert_eq!(result.cognitive_step().command().action(), action,);

    assert!(result.completion().has_cognitive_feedback());

    let evidence = result.completion().turn().evidence().unwrap();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action);

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

    assert_eq!(runtime.completed_cognitive_step_count(), 1,);

    assert_eq!(runtime.transport().execute_count(), 1,);
}

#[test]
fn live_step_preserves_complete_multiframe_environment_response() {
    let mut runtime = active_runtime("live-frames", 300);

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    runtime.transport().push(Ok(observation(
        "live-frames",
        ArcAgi3GameState::NotFinished,
        &[5, 6, 7],
        Some(action),
    )));

    let result = runtime.execute_with(signal(900), begin_action1).unwrap();

    assert_eq!(result.completion().perception().frame_count(), 3,);

    assert_eq!(result.completion().perception().transition_count(), 3,);

    assert_eq!(
        runtime
            .cognitive_runtime()
            .next_perceptual_observation_index(),
        304,
    );
}

#[test]
fn protocol_reset_is_live_transport_control_without_cognitive_feedback() {
    let mut runtime = active_runtime("live-reset", 400);

    runtime.transport().push(Ok(observation(
        "live-reset",
        ArcAgi3GameState::NotFinished,
        &[8, 9],
        Some(ArcAgi3Action::reset()),
    )));

    let completion = runtime.reset(signal(900)).unwrap();

    assert!(!completion.has_cognitive_feedback());

    assert_eq!(runtime.completed_reset_count(), 1,);

    assert_eq!(runtime.completed_cognitive_step_count(), 0,);

    assert_eq!(
        runtime
            .cognitive_runtime()
            .session()
            .completed_reset_count(),
        1,
    );

    assert_eq!(runtime.transport().execute_count(), 1,);
}

#[test]
fn successful_cognitive_steps_increment_only_after_environment_completion() {
    let mut runtime = active_runtime("live-count", 500);

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    runtime.transport().push(Ok(observation(
        "live-count",
        ArcAgi3GameState::NotFinished,
        &[2],
        Some(action),
    )));

    let result = runtime.execute_with(signal(800), begin_action1).unwrap();

    assert_eq!(result.completed_cognitive_step_count(), 1,);

    assert_eq!(runtime.completed_cognitive_step_count(), 1,);

    assert_eq!(runtime.completed_reset_count(), 0,);
}

#[test]
fn transport_failure_preserves_pending_cognitive_runtime_and_faults_live_loop() {
    let mut runtime = active_runtime("live-fault", 600);

    let before_observation = runtime.cognitive_runtime().observation().clone();

    runtime
        .transport()
        .push(Err(ArcAgi3TransportError::HttpTransport {
            message: "connection lost".to_string(),
            disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
        }));

    let result = runtime.execute_with(signal(900), begin_action1);

    assert!(matches!(
        result,
        Err(ArcAgi3LiveEnvironmentError::Transport(
            ArcAgi3TransportError::HttpTransport {
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
                ..
            }
        )),
    ));

    assert_eq!(
        runtime.status(),
        ArcAgi3LiveEnvironmentStatus::FaultedPending,
    );

    assert_eq!(
        runtime.fault_disposition(),
        Some(ArcAgi3TransportFailureDisposition::DispatchIndeterminate,),
    );

    assert!(runtime.cognitive_runtime().session().has_pending_command());

    assert_eq!(
        runtime.cognitive_runtime().observation(),
        &before_observation,
    );

    assert_eq!(runtime.completed_cognitive_step_count(), 0,);
}

#[test]
fn faulted_runtime_never_automatically_retries_indeterminate_action() {
    let mut runtime = active_runtime("live-no-retry", 700);

    runtime
        .transport()
        .push(Err(ArcAgi3TransportError::HttpTransport {
            message: "timeout".to_string(),
            disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
        }));

    let first = runtime.execute_with(signal(900), begin_action1);

    assert!(first.is_err());

    assert_eq!(runtime.transport().execute_count(), 1,);

    let mut begin_calls = 0usize;

    let second = runtime.execute_with(signal(900), |cognitive| {
        begin_calls += 1;
        begin_action1(cognitive)
    });

    assert_eq!(
        second,
        Err(ArcAgi3LiveEnvironmentError::FaultedPending(Some(
            ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
        ),),),
    );

    assert_eq!(begin_calls, 0,);

    assert_eq!(runtime.transport().execute_count(), 1,);
}

#[test]
fn rejected_environment_response_never_causes_hidden_retry() {
    let mut runtime = active_runtime("live-rejected", 800);

    runtime
        .transport()
        .push(Err(ArcAgi3TransportError::HttpStatus {
            status: 400,
            body: "rejected".to_string(),
            disposition: ArcAgi3TransportFailureDisposition::RejectedByEnvironment,
        }));

    let result = runtime.execute_with(signal(900), begin_action1);

    assert!(result.is_err());

    assert_eq!(
        runtime.status(),
        ArcAgi3LiveEnvironmentStatus::FaultedPending,
    );

    assert_eq!(
        runtime.fault_disposition(),
        Some(ArcAgi3TransportFailureDisposition::RejectedByEnvironment,),
    );

    assert_eq!(runtime.transport().execute_count(), 1,);
}

#[test]
fn cognitively_invalid_post_dispatch_response_faults_without_committing_observation() {
    let mut runtime = active_runtime("live-atomic", 900);

    let before = runtime.cognitive_runtime().clone();

    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    runtime.transport().push(Ok(observation(
        "wrong-game",
        ArcAgi3GameState::NotFinished,
        &[9],
        Some(action),
    )));

    let result = runtime.execute_with(signal(900), begin_action1);

    assert!(matches!(
        result,
        Err(ArcAgi3LiveEnvironmentError::Transport(
            ArcAgi3TransportError::CognitiveCompletionRejected {
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
                ..
            }
        )),
    ));

    assert_eq!(
        runtime.status(),
        ArcAgi3LiveEnvironmentStatus::FaultedPending,
    );

    assert_eq!(
        runtime.cognitive_runtime().observation(),
        before.observation(),
    );

    assert_eq!(runtime.completed_cognitive_step_count(), 0,);
}

#[test]
fn terminal_win_blocks_next_cognitive_begin_before_transport_execution() {
    let initial = observation(
        "live-win",
        ArcAgi3GameState::Win,
        &[3],
        Some(ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap()),
    );

    let transport = ScriptedTransport::new(initial);

    let mut runtime = ArcAgi3LiveEnvironmentRuntime::start(
        transport,
        &ArcAgi3GameId::new("requested".to_string()).unwrap(),
        "card-win",
        1000,
    )
    .unwrap();

    assert_eq!(runtime.status(), ArcAgi3LiveEnvironmentStatus::Won,);

    let mut begin_calls = 0usize;

    let result = runtime.execute_with(signal(900), |cognitive| {
        begin_calls += 1;
        begin_action1(cognitive)
    });

    assert_eq!(result, Err(ArcAgi3LiveEnvironmentError::GameNotActive,),);

    assert_eq!(begin_calls, 0,);

    assert_eq!(runtime.transport().execute_count(), 0,);
}

#[test]
fn terminal_game_over_blocks_next_cognitive_begin_before_transport_execution() {
    let initial = observation("live-over", ArcAgi3GameState::GameOver, &[3], None);

    let transport = ScriptedTransport::new(initial);

    let mut runtime = ArcAgi3LiveEnvironmentRuntime::start(
        transport,
        &ArcAgi3GameId::new("requested".to_string()).unwrap(),
        "card-over",
        1100,
    )
    .unwrap();

    assert_eq!(runtime.status(), ArcAgi3LiveEnvironmentStatus::GameOver,);

    let mut begin_calls = 0usize;

    let result = runtime.execute_with(signal(900), |cognitive| {
        begin_calls += 1;
        begin_action1(cognitive)
    });

    assert_eq!(result, Err(ArcAgi3LiveEnvironmentError::GameNotActive,),);

    assert_eq!(begin_calls, 0,);

    assert_eq!(runtime.transport().execute_count(), 0,);
}

#[test]
fn reset_remains_available_after_terminal_state() {
    let initial = observation(
        "live-terminal-reset",
        ArcAgi3GameState::GameOver,
        &[2],
        None,
    );

    let transport = ScriptedTransport::new(initial);

    let mut runtime = ArcAgi3LiveEnvironmentRuntime::start(
        transport,
        &ArcAgi3GameId::new("requested".to_string()).unwrap(),
        "card-terminal-reset",
        1200,
    )
    .unwrap();

    runtime.transport().push(Ok(observation(
        "live-terminal-reset",
        ArcAgi3GameState::NotFinished,
        &[4],
        Some(ArcAgi3Action::reset()),
    )));

    let completion = runtime.reset(signal(900)).unwrap();

    assert!(!completion.has_cognitive_feedback());

    assert_eq!(runtime.status(), ArcAgi3LiveEnvironmentStatus::Active,);

    assert_eq!(runtime.completed_reset_count(), 1,);
}

#[test]
fn faulted_runtime_blocks_protocol_reset_without_second_transport_call() {
    let mut runtime = active_runtime("live-fault-reset", 1300);

    runtime
        .transport()
        .push(Err(ArcAgi3TransportError::HttpTransport {
            message: "ambiguous failure".to_string(),
            disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
        }));

    assert!(runtime.execute_with(signal(900), begin_action1,).is_err());

    assert_eq!(runtime.transport().execute_count(), 1,);

    let reset = runtime.reset(signal(900));

    assert_eq!(
        reset,
        Err(ArcAgi3LiveEnvironmentError::FaultedPending(Some(
            ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
        ),),),
    );

    assert_eq!(runtime.transport().execute_count(), 1,);
}

#[test]
fn universal_facade_matches_direct_live_runtime_start() {
    let initial = observation("live-universal", ArcAgi3GameState::NotFinished, &[5], None);

    let direct_transport = ScriptedTransport::new(initial.clone());

    let facade_transport = ScriptedTransport::new(initial);

    let requested = ArcAgi3GameId::new("requested".to_string()).unwrap();

    let direct =
        ArcAgi3LiveEnvironmentRuntime::start(direct_transport, &requested, "card-universal", 1400)
            .unwrap();

    let facade = UniversalArcAgi3LiveEnvironmentRuntime::start(
        facade_transport,
        &requested,
        "card-universal",
        1400,
    )
    .unwrap();

    assert_eq!(direct.status(), facade.status(),);

    assert_eq!(
        direct.cognitive_runtime().observation(),
        facade.cognitive_runtime().observation(),
    );

    assert_eq!(
        direct
            .cognitive_runtime()
            .next_perceptual_observation_index(),
        facade
            .cognitive_runtime()
            .next_perceptual_observation_index(),
    );
}

#[test]
fn live_runtime_does_not_rewrite_exact_m51_action_identity() {
    let mut runtime = active_runtime("live-identity", 1500);

    let arc_action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(arc_action);

    runtime.transport().push(Ok(observation(
        "live-identity",
        ArcAgi3GameState::NotFinished,
        &[6],
        Some(arc_action),
    )));

    let result = runtime
        .execute_with(signal(900), |cognitive| {
            m51_fixture::begin_arc(cognitive, cognitive_action.clone())
        })
        .unwrap();

    assert_eq!(
        result.cognitive_step().dispatch().action(),
        &cognitive_action,
    );

    assert_eq!(result.cognitive_step().command().action(), arc_action,);

    assert_eq!(result.completion().turn().action(), arc_action,);
}

#[test]
fn live_runtime_into_parts_preserves_transport_and_cognitive_state_exactly() {
    let runtime = active_runtime("live-parts", 1600);

    let expected_observation = runtime.cognitive_runtime().observation().clone();

    let expected_index = runtime
        .cognitive_runtime()
        .next_perceptual_observation_index();

    let (transport, cognitive) = runtime.into_parts();

    assert_eq!(transport.start_count(), 1,);

    assert_eq!(transport.execute_count(), 0,);

    assert_eq!(cognitive.observation(), &expected_observation,);

    assert_eq!(
        cognitive.next_perceptual_observation_index(),
        expected_index,
    );
}
