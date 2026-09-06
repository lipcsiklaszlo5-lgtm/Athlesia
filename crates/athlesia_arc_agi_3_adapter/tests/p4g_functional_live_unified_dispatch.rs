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

#[test]
fn multiframe_first_response_is_causal_but_latest_response_is_current_state() {
    let game = "c16hb0-multiframe-authority";

    let mut runtime = live_runtime(game, 620_000);

    /*
     * Reuse the canonical P4G maturity path.
     *
     * No duplicate grounding fixture is introduced here.
     */
    mature_runtime(&mut runtime, game);

    let pre_action_current = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect(
            "canonical mature P4G runtime must expose a grounded current state",
        );

    let episode_count_before = runtime
        .cognitive_runtime()
        .cognition()
        .transition_episode_count();

    assert!(
        episode_count_before > 0,
        "canonical mature runtime must retain causal transition evidence",
    );

    let action_one = action(ArcAgi3ActionId::Action1);

    /*
     * Build each frame through the existing normal_observation fixture,
     * then combine only their already-valid grids into one multiframe
     * environment response.
     */
    let causal_template =
        normal_observation(game, 6_u8, Some(action_one));

    let latest_template =
        normal_observation(game, 7_u8, Some(action_one));

    let causal_grid =
        causal_template.frames().latest().clone();

    let latest_grid =
        latest_template.frames().latest().clone();

    let multiframe = ArcAgi3Observation::new(
        causal_template.game_id().clone(),
        causal_template.state(),
        ArcAgi3FrameSequence::new(vec![
            causal_grid,
            latest_grid,
        ])
        .expect("two canonical grids form a valid multiframe response"),
        causal_template.levels_completed(),
        causal_template.win_levels(),
        causal_template.available_actions().clone(),
        Some(action_one),
    );

    runtime
        .transport()
        .push(Ok(multiframe));

    let cognitive_action =
        ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let completed = runtime
        .execute_with(signal(900), |cognitive| {
            m51_fixture::begin_arc(
                cognitive,
                cognitive_action,
            )
        })
        .expect("grounded multiframe ACTION1 turn must execute");

    assert!(
        completed.completion().has_cognitive_feedback(),
        "real ACTION1 must retain causal feedback",
    );

    assert_eq!(
        completed.completion().perception().frame_count(),
        2,
        "response must retain exactly two perceptual frames",
    );

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .transition_episode_count(),
        episode_count_before + 1,
        "one real action must create exactly one new direct causal episode",
    );

    let frames =
        completed.completion().perception().frames();

    let causal_frame = &frames[0];
    let latest_frame = &frames[1];

    assert!(
        causal_frame.observation_index()
            < latest_frame.observation_index(),
        "first and latest response frames must have ordered observation indices",
    );

    /*
     * Current scene authority is evaluated against the latest frame.
     *
     * Project it through the canonical M46 single-scene projector for both
     * frames. If it cannot ground the first frame, the test fails rather than
     * manufacturing a separate scene or state encoder.
     */
    let scene = runtime
        .cognitive_runtime()
        .current_best_scene_interpretation()
        .expect(
            "latest multiframe response must expose a grounded current scene",
        );

    let causal_state =
        athlesia_core_knowledge_perceptual_grounding::
            GroundedPerceptualStateProjector::scene_facts(
                causal_frame,
                &scene,
            )
            .and_then(
                athlesia_universal_domain_learning::
                    GroundedStateSnapshot::new,
            )
            .expect(
                "canonical current scene must remain grounded in causal response frame",
            );

    let latest_state =
        athlesia_core_knowledge_perceptual_grounding::
            GroundedPerceptualStateProjector::scene_facts(
                latest_frame,
                &scene,
            )
            .and_then(
                athlesia_universal_domain_learning::
                    GroundedStateSnapshot::new,
            )
            .expect(
                "canonical current scene must project latest response frame",
            );

    assert_ne!(
        causal_state,
        latest_state,
        "causal response state and latest response state must be observably distinct",
    );

    assert_ne!(
        pre_action_current,
        latest_state,
        "multiframe response must genuinely advance the represented current state",
    );

    /*
     * M47 causal evidence authority is first response frame.
     */
    let retained_after = runtime
        .cognitive_runtime()
        .cognition()
        .transition_schema_learning()
        .episodes()
        .last()
        .expect("new causal episode must be retained")
        .after();

    assert_eq!(
        retained_after,
        &causal_state,
        "retained direct action consequence must terminate at FIRST response frame",
    );

    /*
     * Live current state authority is latest response frame.
     */
    let live_current = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect(
            "latest multiframe response must expose current grounded state",
        );

    assert_eq!(
        live_current,
        latest_state,
        "live current state must represent LATEST response frame",
    );

    assert_ne!(
        live_current,
        causal_state,
        "FIRST causal response cannot masquerade as latest live state",
    );

    assert_ne!(
        &live_current,
        retained_after,
        "retained transition history cannot alias live current-state authority",
    );
}
#[test]
fn transition_capacity_saturation_does_not_freeze_live_current_grounded_state() {
    const LIVE_TRANSITION_EPISODE_CAP: usize = 256;

    let game = "c16hb0-transition-capacity";

    let mut runtime = live_runtime(game, 630_000);

    /*
     * Establish the canonical grounded P4G state first.
     */
    mature_runtime(&mut runtime, game);

    let action_one = action(ArcAgi3ActionId::Action1);
    let action_two = action(ArcAgi3ActionId::Action2);

    let initial_count = runtime
        .cognitive_runtime()
        .cognition()
        .transition_episode_count();

    assert!(
        initial_count > 0
            && initial_count < LIVE_TRANSITION_EPISODE_CAP,
        "fixture must begin with real evidence below the live frontier",
    );

    assert!(
        runtime
            .cognitive_runtime()
            .current_grounded_world_state()
            .is_some(),
        "fixture must begin from a genuinely grounded live state",
    );

    /*
     * mature_runtime ends on the familiar grounded value 5.
     *
     * Repeated ACTION2 -> 5 self-loops are already part of the canonical
     * maturity evidence class. Fill the real M51 store to its exact live
     * frontier, checking every admission so a rejected turn cannot produce
     * a false saturation result.
     */
    let remaining =
        LIVE_TRANSITION_EPISODE_CAP - initial_count;

    for index in 0..remaining {
        let before = runtime
            .cognitive_runtime()
            .cognition()
            .transition_episode_count();

        real_training_turn(
            &mut runtime,
            game,
            action_two,
            5_u8,
        );

        let after = runtime
            .cognitive_runtime()
            .cognition()
            .transition_episode_count();

        assert_eq!(
            after,
            before + 1,
            "every pre-frontier controlled turn must be admitted; failure at fill index {index}",
        );
    }

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .transition_episode_count(),
        LIVE_TRANSITION_EPISODE_CAP,
        "fixture must reach the exact production evidence frontier",
    );

    let pre_saturation_current = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect(
            "full transition memory must still expose a grounded current state",
        );

    let historical_last_before = runtime
        .cognitive_runtime()
        .cognition()
        .transition_schema_learning()
        .episodes()
        .last()
        .expect("full transition memory must have a final retained episode")
        .clone();

    /*
     * Now execute one MORE genuine controlled action.
     *
     * ACTION1 -> 6 is a familiar grounded transition from the canonical
     * mature fixture. M51 cannot retain episode 257, but perception and
     * current-state authority must still advance.
     */
    real_training_turn(
        &mut runtime,
        game,
        action_one,
        6_u8,
    );

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .transition_episode_count(),
        LIVE_TRANSITION_EPISODE_CAP,
        "transition evidence memory must remain bounded at 256",
    );

    let historical_last_after = runtime
        .cognitive_runtime()
        .cognition()
        .transition_schema_learning()
        .episodes()
        .last()
        .expect("bounded history remains nonempty");

    assert_eq!(
        historical_last_after,
        &historical_last_before,
        "frontier-exceeded turn must not overwrite or fabricate retained transition history",
    );

    /*
     * Derive expected live state solely from the LATEST perception using
     * the canonical M46 scene->facts authority.
     */
    let current_scene = runtime
        .cognitive_runtime()
        .current_best_scene_interpretation()
        .expect(
            "familiar post-saturation response must retain a grounded current scene",
        );

    let expected_current =
        athlesia_core_knowledge_perceptual_grounding::
            GroundedPerceptualStateProjector::scene_facts(
                runtime
                    .cognitive_runtime()
                    .perception()
                    .latest_frame(),
                &current_scene,
            )
            .and_then(
                athlesia_universal_domain_learning::
                    GroundedStateSnapshot::new,
            )
            .expect(
                "latest perception must project through canonical M46 authority",
            );

    let live_current = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect(
            "transition-memory saturation must not erase live current state",
        );

    assert_eq!(
        live_current,
        expected_current,
        "current state must continue to follow latest perception after transition-memory saturation",
    );

    assert_ne!(
        live_current,
        pre_saturation_current,
        "post-frontier ACTION1 -> 6 must genuinely advance current state",
    );

    assert_ne!(
        live_current,
        historical_last_before.after().clone(),
        "frozen retained transition history must not masquerade as the new live current state",
    );

    assert_eq!(
        runtime
            .cognitive_runtime()
            .observation()
            .last_action(),
        Some(action_one),
        "anti-vacuity: the post-frontier environment action must actually complete",
    );
}
#[test]
fn live_completed_turn_event_identity_is_bound_one_to_one_to_retained_transition_episode() {
    let game = "c16hb1-live-transition-provenance";

    let mut runtime = live_runtime(game, 650_000);

    /*
     * Reuse the canonical P4G maturity trajectory so the next transition
     * is genuinely grounded and therefore admissible into M47/M51 memory.
     */
    mature_runtime(&mut runtime, game);

    let cognition_before =
        runtime.cognitive_runtime().cognition();

    let episode_count_before =
        cognition_before.transition_episode_count();

    let provenance_count_before =
        cognition_before
            .transition_schema_learning()
            .event_provenance_count();

    assert!(
        episode_count_before > 0,
        "fixture must already contain real retained transition evidence",
    );

    assert_eq!(
        episode_count_before,
        provenance_count_before,
        "every pre-existing retained episode must already have exactly one provenance record",
    );

    let action_one =
        action(ArcAgi3ActionId::Action1);

    let cognitive_action =
        ArcAgi3CognitiveProtocolBridge::encode_action(
            action_one,
        );

    /*
     * Execute one REAL live environment turn directly instead of using
     * real_training_turn(), because B1-B must inspect the returned
     * ArcAgi3CompletedTurn identity itself.
     */
    runtime
        .transport()
        .push(Ok(normal_observation(
            game,
            6_u8,
            Some(action_one),
        )));

    let completed = runtime
        .execute_with(signal(900), |cognitive| {
            m51_fixture::begin_arc(
                cognitive,
                cognitive_action.clone(),
            )
        })
        .expect(
            "live grounded ACTION1 consequence must execute",
        );

    assert!(
        completed.completion().has_cognitive_feedback(),
        "real live action must produce canonical environment evidence",
    );

    let completed_turn =
        completed.completion().turn();

    let completed_event_index =
        completed_turn.event_index();

    let evidence =
        completed_turn
            .evidence()
            .expect(
                "real completed action must retain environment interaction evidence",
            );

    let evidence_event_index =
        evidence
            .action_observation()
            .event_index();

    assert_eq!(
        evidence_event_index,
        completed_event_index,
        "session completed-turn identity must survive unchanged into EnvironmentInteractionEvidence",
    );

    let causal_transition =
        completed
            .completion()
            .perception()
            .causal_environment_transition()
            .expect(
                "real live action response must expose its direct causal perceptual transition",
            );

    let cognition_after =
        runtime.cognitive_runtime().cognition();

    let learning_state =
        cognition_after.transition_schema_learning();

    assert_eq!(
        cognition_after.transition_episode_count(),
        episode_count_before + 1,
        "one newly admitted real event must add exactly one retained transition episode",
    );

    assert_eq!(
        learning_state.event_provenance_count(),
        provenance_count_before + 1,
        "one newly admitted real event must add exactly one provenance record",
    );

    assert_eq!(
        cognition_after.transition_episode_count(),
        learning_state.event_provenance_count(),
        "retained episode and provenance frontiers must remain exactly one-to-one",
    );

    let retained_episode =
        learning_state
            .episodes()
            .last()
            .expect(
                "newly admitted live event must retain its M47 episode",
            );

    let retained_provenance =
        learning_state
            .event_provenance()
            .last()
            .expect(
                "newly admitted live event must retain provenance beside its episode",
            );

    assert_eq!(
        retained_provenance.event_index(),
        completed_event_index,
        "retained M51 provenance must preserve the exact completed-turn event identity",
    );

    assert_eq!(
        retained_provenance.previous_observation_index(),
        causal_transition
            .previous_frame()
            .observation_index(),
        "retained provenance must bind to the exact causal predecessor frame",
    );

    assert_eq!(
        retained_provenance.current_observation_index(),
        causal_transition
            .current_frame()
            .observation_index(),
        "retained provenance must bind to the exact first causal response frame",
    );

    assert_eq!(
        retained_episode.transformation(),
        &cognitive_action,
        "the episode paired with this provenance must preserve the exact executed cognitive action",
    );

    /*
     * The two clocks are independent authorities.
     *
     * This test verifies exact identities independently and intentionally
     * makes no numeric equality/order assertion between event_index and
     * perceptual observation_index.
     */
    assert_eq!(
        learning_state
            .episodes()
            .len(),
        learning_state
            .event_provenance()
            .len(),
        "index-aligned episode/provenance storage must remain structurally one-to-one",
    );
}

#[test]
fn live_reset_cannot_append_transition_event_provenance() {
    let game = "c16hb1-reset-provenance";

    let mut runtime = live_runtime(game, 650_000);

    mature_runtime(&mut runtime, game);

    let action_two = action(ArcAgi3ActionId::Action2);

    real_training_turn(
        &mut runtime,
        game,
        action_two,
        7_u8,
    );

    let state_before = runtime
        .cognitive_runtime()
        .cognition()
        .transition_schema_learning();

    let episodes_before = state_before.episode_count();
    let provenance_before = state_before.event_provenance().to_vec();

    assert!(
        episodes_before > 0,
        "RESET provenance test requires retained transition evidence",
    );

    assert_eq!(
        episodes_before,
        provenance_before.len(),
        "precondition: every retained episode must already have provenance",
    );

    runtime.transport().push(Ok(normal_observation(
        game,
        5_u8,
        Some(ArcAgi3Action::reset()),
    )));

    let completion = runtime
        .reset(signal(900))
        .expect("live protocol RESET must complete");

    assert!(
        !completion.has_cognitive_feedback(),
        "RESET must not become an environment-learning consequence",
    );

    let state_after = runtime
        .cognitive_runtime()
        .cognition()
        .transition_schema_learning();

    assert_eq!(
        state_after.episode_count(),
        episodes_before,
        "RESET must not append a grounded transformation episode",
    );

    assert_eq!(
        state_after.event_provenance(),
        provenance_before.as_slice(),
        "RESET must not append, rewrite, or consume transition-event provenance",
    );

    assert_eq!(
        state_after.episode_count(),
        state_after.event_provenance_count(),
        "episode/provenance cardinality must remain atomic after RESET",
    );
}


#[test]
fn live_transition_capacity_saturates_episode_and_provenance_atomically() {
    const LIVE_TRANSITION_EPISODE_CAP: usize = 256;

    let game = "c16hb1-provenance-capacity";

    let mut runtime = live_runtime(game, 660_000);

    mature_runtime(&mut runtime, game);

    let action_one = action(ArcAgi3ActionId::Action1);
    let action_two = action(ArcAgi3ActionId::Action2);

    let initial = runtime
        .cognitive_runtime()
        .cognition()
        .transition_schema_learning();

    let initial_episode_count = initial.episode_count();

    assert_eq!(
        initial_episode_count,
        initial.event_provenance_count(),
        "mature live owner must begin with exact episode/provenance cardinality",
    );

    assert!(
        initial_episode_count > 0
            && initial_episode_count < LIVE_TRANSITION_EPISODE_CAP,
        "fixture must begin below the production transition frontier",
    );

    let remaining =
        LIVE_TRANSITION_EPISODE_CAP - initial_episode_count;

    for index in 0..remaining {
        let before = runtime
            .cognitive_runtime()
            .cognition()
            .transition_schema_learning();

        let before_episodes = before.episode_count();
        let before_provenance = before.event_provenance_count();

        assert_eq!(
            before_episodes,
            before_provenance,
            "pre-admission owner must remain one-to-one at fill index {index}",
        );

        real_training_turn(
            &mut runtime,
            game,
            action_two,
            5_u8,
        );

        let after = runtime
            .cognitive_runtime()
            .cognition()
            .transition_schema_learning();

        assert_eq!(
            after.episode_count(),
            before_episodes + 1,
            "pre-frontier real event must append one episode at fill index {index}",
        );

        assert_eq!(
            after.event_provenance_count(),
            before_provenance + 1,
            "pre-frontier real event must append one provenance record at fill index {index}",
        );
    }

    let full = runtime
        .cognitive_runtime()
        .cognition()
        .transition_schema_learning();

    assert_eq!(
        full.episode_count(),
        LIVE_TRANSITION_EPISODE_CAP,
    );

    assert_eq!(
        full.event_provenance_count(),
        LIVE_TRANSITION_EPISODE_CAP,
    );

    let last_episode_before = full
        .episodes()
        .last()
        .expect("full transition store must be nonempty")
        .clone();

    let last_provenance_before = *full
        .event_provenance()
        .last()
        .expect("full provenance store must be nonempty");

    /*
     * A 257th genuine live event completes in the environment but cannot
     * partially append either side of the retained transition record.
     */
    real_training_turn(
        &mut runtime,
        game,
        action_one,
        6_u8,
    );

    let after_frontier = runtime
        .cognitive_runtime()
        .cognition()
        .transition_schema_learning();

    assert_eq!(
        after_frontier.episode_count(),
        LIVE_TRANSITION_EPISODE_CAP,
        "episode owner must remain hard-bounded",
    );

    assert_eq!(
        after_frontier.event_provenance_count(),
        LIVE_TRANSITION_EPISODE_CAP,
        "provenance owner must freeze atomically with episode owner",
    );

    assert_eq!(
        after_frontier.episodes().last(),
        Some(&last_episode_before),
        "frontier overflow must not rewrite the last retained episode",
    );

    assert_eq!(
        after_frontier.event_provenance().last(),
        Some(&last_provenance_before),
        "frontier overflow must not create orphan provenance",
    );

    assert_eq!(
        after_frontier.episode_count(),
        after_frontier.event_provenance_count(),
        "episode/provenance one-to-one invariant must survive saturation",
    );
}
