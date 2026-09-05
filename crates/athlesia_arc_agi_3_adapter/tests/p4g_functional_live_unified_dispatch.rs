use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use athlesia_arc_agi_3_adapter::{
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
    environment_transport_boundary::{ArcAgi3EnvironmentTransport, ArcAgi3TransportError},
    interactive_session_runtime::ArcAgi3SessionCommand,
    live_environment_runtime::{ArcAgi3LiveEnvironmentRuntime, ArcAgi3LiveUnifiedActionRequest},
};
use athlesia_executive_agency::{
    ExecutiveAgencyPolicy, ExecutiveGoal, ExecutiveSelectionThresholds, ExecutiveUtilityWeights,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

mod m51_fixture {
    include!("support/m51_online_orchestration_fixture.rs");
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn action(id: ArcAgi3ActionId) -> ArcAgi3Action {
    ArcAgi3Action::discrete(id).unwrap()
}

fn object_grid(value: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![value, value], vec![8, 9]]).unwrap()
}

fn observation(
    game: &str,
    value: u8,
    available: Vec<ArcAgi3ActionId>,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new(game.to_string()).unwrap(),
        ArcAgi3GameState::NotFinished,
        ArcAgi3FrameSequence::new(vec![object_grid(value)]).unwrap(),
        0,
        3,
        ArcAgi3AvailableActions::new(available).unwrap(),
        last_action,
    )
}

fn normal_observation(
    game: &str,
    value: u8,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    observation(
        game,
        value,
        vec![ArcAgi3ActionId::Action1, ArcAgi3ActionId::Action2],
        last_action,
    )
}

#[derive(Debug)]
struct RecordingTransport {
    initial: Option<Result<ArcAgi3Observation, ArcAgi3TransportError>>,
    responses: RefCell<VecDeque<Result<ArcAgi3Observation, ArcAgi3TransportError>>>,
    execute_count: Cell<usize>,
    executed_actions: RefCell<Vec<ArcAgi3Action>>,
}

impl RecordingTransport {
    fn new(initial: ArcAgi3Observation) -> Self {
        Self {
            initial: Some(Ok(initial)),
            responses: RefCell::new(VecDeque::new()),
            execute_count: Cell::new(0),
            executed_actions: RefCell::new(Vec::new()),
        }
    }

    fn push(&self, response: Result<ArcAgi3Observation, ArcAgi3TransportError>) {
        self.responses.borrow_mut().push_back(response);
    }

    fn execute_count(&self) -> usize {
        self.execute_count.get()
    }

    fn last_executed_action(&self) -> Option<ArcAgi3Action> {
        self.executed_actions.borrow().last().copied()
    }
}

impl ArcAgi3EnvironmentTransport for RecordingTransport {
    fn start_game(
        &mut self,
        _game_id: &ArcAgi3GameId,
        _card_id: &str,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        self.initial
            .take()
            .unwrap_or(Err(ArcAgi3TransportError::ActiveSessionExists))
    }

    fn execute(
        &mut self,
        command: &ArcAgi3SessionCommand,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        self.execute_count.set(
            self.execute_count
                .get()
                .checked_add(1)
                .expect("transport counter remains bounded"),
        );

        self.executed_actions.borrow_mut().push(command.action());

        self.responses
            .borrow_mut()
            .pop_front()
            .unwrap_or(Err(ArcAgi3TransportError::NoActiveSession))
    }
}

fn live_runtime(game: &str, first_index: u64) -> ArcAgi3LiveEnvironmentRuntime<RecordingTransport> {
    ArcAgi3LiveEnvironmentRuntime::start(
        RecordingTransport::new(normal_observation(game, 1, None)),
        &ArcAgi3GameId::new(game.to_string()).unwrap(),
        "p4g-c2",
        first_index,
    )
    .unwrap()
}

fn real_training_turn(
    runtime: &mut ArcAgi3LiveEnvironmentRuntime<RecordingTransport>,
    game: &str,
    selected_action: ArcAgi3Action,
    value: u8,
) {
    runtime
        .transport()
        .push(Ok(normal_observation(game, value, Some(selected_action))));

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(selected_action);

    let result = runtime
        .execute_with(signal(900), |cognitive| {
            m51_fixture::begin_arc(cognitive, cognitive_action)
        })
        .expect("training environment turn must execute");

    assert!(result.completion().has_cognitive_feedback(),);
}

fn mature_runtime(runtime: &mut ArcAgi3LiveEnvironmentRuntime<RecordingTransport>, game: &str) {
    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    /*
     * Repeated coherent ACTION1 changes establish object/scene and
     * action-specific transition evidence.
     */
    for value in [2_u8, 3, 4, 5] {
        real_training_turn(runtime, game, action_one, value);
    }

    /*
     * ACTION2 is explicitly contrasted as a self-loop while ACTION1
     * continues to change the same grounded object.
     */
    for (selected, value) in [
        (action_two, 5_u8),
        (action_one, 6_u8),
        (action_two, 6_u8),
        (action_one, 5_u8),
        (action_two, 5_u8),
        (action_one, 6_u8),
        (action_two, 6_u8),
        (action_one, 5_u8),
    ] {
        real_training_turn(runtime, game, selected, value);
    }
}

fn goal() -> ExecutiveGoal {
    ExecutiveGoal::new(
        atom(0x5034_4743_3247_4f41),
        signal(900),
        CognitiveSignal::zero(),
    )
}

fn evidence_policy() -> ExecutiveAgencyPolicy {
    ExecutiveAgencyPolicy::new(
        1,
        8,
        16,
        1,
        ExecutiveUtilityWeights::new(0, 0, 1000, 0, 0).unwrap(),
        ExecutiveSelectionThresholds::new(
            signal(100),
            signal(100),
            signal(1),
            signal(600),
            signal(100),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn retained_cognition_selects_real_live_action_without_caller_supplying_final_action() {
    let game = "p4gc2-live-causal";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 300_000);

    mature_runtime(&mut runtime, game);

    let before_transport = runtime.transport().execute_count();

    /*
     * Environment consequence is prepared for the expected learned
     * ACTION1. The execute_unified call receives only the affordance
     * frontier, not a selected action and not a cognitive source state.
     */
    runtime
        .transport()
        .push(Ok(normal_observation(game, 6, Some(action_one))));

    let candidates = [action_one, action_two];

    let goal = goal();

    let request = ArcAgi3LiveUnifiedActionRequest::new(
        &candidates,
        &goal,
        signal(900),
        CognitiveSignal::zero(),
        None,
        evidence_policy(),
        signal(900),
    );

    let step = runtime
        .execute_unified(request)
        .expect("live unified execution must not error")
        .expect("retained model must authorize one action");

    assert_eq!(step.action(), action_one,);

    assert_eq!(step.command().action(), action_one,);

    assert_eq!(
        runtime.transport().last_executed_action(),
        Some(action_one,),
    );

    assert_eq!(runtime.transport().execute_count(), before_transport + 1,);

    assert!(
        step.completion().has_cognitive_feedback(),
        "autonomously selected live action must retain cognitive feedback",
    );

    let evidence = step
        .completion()
        .turn()
        .evidence()
        .expect("unified live completion must carry environment evidence");

    assert_eq!(
        evidence.action_observation().descriptor(),
        step.cognitive_action(),
    );

    assert_eq!(
        evidence.execution_observation().observed_action(),
        step.cognitive_action(),
    );

    assert_eq!(
        evidence.execution_observation().observed_state(),
        step.source_state(),
    );

    assert_eq!(
        evidence.experiment_observation().source_state(),
        step.source_state(),
    );

    assert_eq!(
        evidence.experiment_observation().action(),
        step.cognitive_action(),
    );
}

#[test]
fn epistemic_abstention_has_zero_transport_and_zero_hidden_pending_state() {
    let game = "p4gc2-live-abstain";

    let mut runtime = live_runtime(game, 310_000);

    let candidates = [
        action(ArcAgi3ActionId::Action1),
        action(ArcAgi3ActionId::Action2),
    ];

    let goal = goal();

    let before_transport = runtime.transport().execute_count();

    let before_steps = runtime.completed_cognitive_step_count();

    let request = ArcAgi3LiveUnifiedActionRequest::new(
        &candidates,
        &goal,
        signal(900),
        CognitiveSignal::zero(),
        None,
        evidence_policy(),
        signal(900),
    );

    let result = runtime
        .execute_unified(request)
        .expect("abstention is not a live execution error");

    assert!(
        result.is_none(),
        "fresh runtime must not fabricate action authority",
    );

    assert_eq!(runtime.transport().execute_count(), before_transport,);

    assert_eq!(runtime.completed_cognitive_step_count(), before_steps,);

    assert!(
        !runtime.cognitive_runtime().session().has_pending_command(),
        "abstention must not create hidden pending work",
    );
}

#[test]
fn learned_but_now_unavailable_action_never_reaches_transport() {
    let game = "p4gc2-live-unavailable";

    let mut runtime = live_runtime(game, 320_000);

    mature_runtime(&mut runtime, game);

    let action_one = action(ArcAgi3ActionId::Action1);

    /*
     * One more genuine ACTION1 consequence is observed, but the next
     * environment observation removes ACTION1 from its affordances.
     */
    runtime.transport().push(Ok(observation(
        game,
        6,
        vec![ArcAgi3ActionId::Action2],
        Some(action_one),
    )));

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    runtime
        .execute_with(signal(900), |cognitive| {
            m51_fixture::begin_arc(cognitive, cognitive_action)
        })
        .expect("final real grounding turn must execute");

    let before_transport = runtime.transport().execute_count();

    let candidates = [action_one];

    let goal = goal();

    let request = ArcAgi3LiveUnifiedActionRequest::new(
        &candidates,
        &goal,
        signal(900),
        CognitiveSignal::zero(),
        None,
        evidence_policy(),
        signal(900),
    );

    let result = runtime
        .execute_unified(request)
        .expect("unavailable candidate must fail closed as abstention");

    assert!(
        result.is_none(),
        "ARC availability must remove stale executable authority",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        before_transport,
        "unavailable learned action must never reach transport",
    );

    assert!(!runtime.cognitive_runtime().session().has_pending_command(),);
}

mod p4g_c2_real_m50_live_provenance {
    include!("p4g_functional_unified_dispatch_authority.rs");

    #[test]
    fn real_m50_winner_preserves_exact_source_state_through_live_transport_feedback() {
        let game = "p4gc1-m50-through-m48";

        let action_two = action(ArcAgi3ActionId::Action2);

        let runtime =
            ArcAgi3CognitiveInteractionRuntime::new(normal_observation(game, 1, None), 190_000)
                .unwrap();

        assert_eq!(
            runtime.current_unified_executive_action_selection(
                &[],
                &goal(),
                signal(900),
                CognitiveSignal::zero(),
                None,
                information_authority_policy(),
            ),
            None,
            "without M47 exploitation or M50 continuation there is no dispatch authority"
        );

        let experimentation = continuing_experiment(action_two);

        assert_eq!(
            runtime.current_unified_executive_action_selection(
                    &[],
                    &goal(),
                    signal(900),
                    CognitiveSignal::zero(),
                    Some(athlesia_arc_agi_3_adapter::cognitive_interaction_runtime::ArcAgi3ExperimentDispatchAuthority::new(&experimentation, &experiment_source_state())),
                    information_authority_policy(),
                ),
            Some(action_two),
            "real M50 continuation may act only after entering and winning the common M48 frontier"
        );

        /*
         * P4G-C2 final provenance oracle.
         *
         * The frozen C1 setup above has already created the real M50
         * continuation and proved that it wins only through the common
         * M48 authority.
         *
         * Now execute that exact authority through the REAL live
         * transport path.
         */

        let live_initial_observation = runtime.observation().clone();

        assert!(
            live_initial_observation.last_action().is_none(),
            "frozen C1 fixture must begin from an observation without a fabricated reported action",
        );

        let live_expected_source_state = experiment_source_state();

        let live_experiment_authority =
athlesia_arc_agi_3_adapter::
                        cognitive_interaction_runtime::
                        ArcAgi3ExperimentDispatchAuthority::new(
                            &experimentation,
                            &live_expected_source_state,
                        );

        let live_expected_experiment = live_experiment_authority
            .result()
            .next_experiment()
            .expect("continuing M50 authority has an exact next experiment");

        let live_expected_action_structure = live_expected_experiment.action().clone();

        let live_expected_arc_action =
                athlesia_arc_agi_3_adapter::
                    cognitive_protocol_bridge::
                    ArcAgi3CognitiveProtocolBridge::
                    decode_action(
                        &live_expected_action_structure,
                    )
                    .expect(
                        "real M50 action must preserve exact ARC identity",
                    );

        let mut live_runtime =
                athlesia_arc_agi_3_adapter::
                    live_environment_runtime::
                    ArcAgi3LiveEnvironmentRuntime::start(
                        super::RecordingTransport::new(
                            live_initial_observation.clone(),
                        ),
                        live_initial_observation.game_id(),
                        "p4g-c2-m50-live-provenance",
                        390_000,
                    )
                    .expect(
                        "live runtime must start from frozen C1 observation",
                    );

        /*
         * No reported last_action is required by the session contract.
         * The pending self-generated action remains the causal authority.
         */
        live_runtime
            .transport()
            .push(Ok(live_initial_observation.clone()));

        let before_transport = live_runtime.transport().execute_count();

        let live_goal = goal();

        let live_request =
                athlesia_arc_agi_3_adapter::
                    live_environment_runtime::
                    ArcAgi3LiveUnifiedActionRequest::new(
                        &[],
                        &live_goal,
                        signal(900),
                        CognitiveSignal::zero(),
                        Some(live_experiment_authority),
                        information_authority_policy(),
                        athlesia_mindstone_sparse_cognition::
                            CognitiveSignal::new(900)
                            .expect(
                                "positive live feedback confidence",
                            ),
                    );

        let live_step = live_runtime
            .execute_unified(live_request)
            .expect("real M50 live execution must not fail")
            .expect("real continuing M50 authority must produce one live action");

        assert_eq!(
            live_step.action(),
            live_expected_arc_action,
            "the live transport action must be the exact real M50 experiment selected through M48",
        );

        assert_eq!(
            live_step.cognitive_action(),
            &live_expected_action_structure,
            "the live authority must preserve the exact M50 cognitive action",
        );

        assert_eq!(
            live_step.source_state(),
            &live_expected_source_state,
            "the live authority must preserve the exact M50 source state",
        );

        assert_eq!(
            live_runtime.transport().execute_count(),
            before_transport + 1,
            "real M50 winner must produce exactly one transport side effect",
        );

        assert_eq!(
            live_runtime.transport().last_executed_action(),
            Some(live_expected_arc_action,),
            "transport must receive exactly the M48-selected M50 action",
        );

        assert!(
            live_step.completion().has_cognitive_feedback(),
            "real M50 live execution must close through cognitive feedback",
        );

        let live_evidence = live_step
            .completion()
            .turn()
            .evidence()
            .expect("real M50 live completion must bind self-generated environment evidence");

        assert_eq!(
            live_evidence.execution_observation().observed_state(),
            &live_expected_source_state,
            "execution feedback must retain exact M50 source-state provenance",
        );

        assert_eq!(
            live_evidence.experiment_observation().source_state(),
            &live_expected_source_state,
            "M50 feedback must retain its exact experiment source state",
        );

        assert_eq!(
            live_evidence.execution_observation().observed_action(),
            &live_expected_action_structure,
            "execution feedback must retain exact M50 action identity",
        );

        assert_eq!(
            live_evidence.experiment_observation().action(),
            &live_expected_action_structure,
            "experiment feedback must retain exact M50 action identity",
        );
    }
}
