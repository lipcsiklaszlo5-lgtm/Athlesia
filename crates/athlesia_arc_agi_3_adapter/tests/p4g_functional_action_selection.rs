use athlesia_arc_agi_3_adapter::{
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
    cognitive_interaction_runtime::ArcAgi3CognitiveInteractionRuntime,
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
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

fn action(id: ArcAgi3ActionId) -> ArcAgi3Action {
    ArcAgi3Action::discrete(id).unwrap()
}

fn object_grid(value: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![value, value], vec![8, 9]]).unwrap()
}

fn global_grid(value: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![value, value], vec![value, value]]).unwrap()
}

fn observation(
    game: &str,
    frame: ArcAgi3Grid,
    last_action: Option<ArcAgi3Action>,
) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new(game.to_string()).unwrap(),
        ArcAgi3GameState::NotFinished,
        ArcAgi3FrameSequence::new(vec![frame]).unwrap(),
        0,
        3,
        ArcAgi3AvailableActions::new(vec![
            ArcAgi3ActionId::Action1,
            ArcAgi3ActionId::Action2,
            ArcAgi3ActionId::Action6,
        ])
        .unwrap(),
        last_action,
    )
}

fn real_turn(
    runtime: &mut ArcAgi3CognitiveInteractionRuntime,
    game: &str,
    selected_action: ArcAgi3Action,
    frame: ArcAgi3Grid,
) {
    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(selected_action);

    let step = m51_fixture::begin_arc(runtime, cognitive_action)
        .expect("real M51 executive action must begin");

    assert!(step.orchestration().advanced());

    let completion = runtime
        .complete_environment_turn(observation(game, frame, Some(selected_action)), signal(900))
        .expect("real environment consequence must commit");

    assert!(completion.has_cognitive_feedback());
}

fn goal() -> ExecutiveGoal {
    /*
     * Goal desirability is explicit executive authority.
     * It is deliberately not inferred from the learned world model.
     */
    ExecutiveGoal::new(
        CognitiveStructure::atom(0x5034_4742_3247_4f41),
        signal(900),
        CognitiveSignal::zero(),
    )
}

fn executive_policy() -> ExecutiveAgencyPolicy {
    let weights = ExecutiveUtilityWeights::new(
        300, 350, 350,
        /*
         * Model-grounded exploitation does not receive
         * an invented information-gain advantage.
         */
        0, 0,
    )
    .unwrap();

    let thresholds = ExecutiveSelectionThresholds::new(
        signal(100),
        signal(100),
        signal(1),
        signal(600),
        signal(100),
    )
    .unwrap();

    ExecutiveAgencyPolicy::new(1, 8, 16, 1, weights, thresholds).unwrap()
}

fn candidate_actions() -> [ArcAgi3Action; 2] {
    [
        action(ArcAgi3ActionId::Action1),
        action(ArcAgi3ActionId::Action2),
    ]
}

fn selected_action(runtime: &ArcAgi3CognitiveInteractionRuntime) -> Option<ArcAgi3Action> {
    runtime.current_model_grounded_action_selection(
        &candidate_actions(),
        &goal(),
        signal(900),
        CognitiveSignal::zero(),
        executive_policy(),
    )
}

fn mature_scene(runtime: &mut ArcAgi3CognitiveInteractionRuntime, game: &str) {
    let action_one = action(ArcAgi3ActionId::Action1);

    for value in [2_u8, 3, 4, 5] {
        real_turn(runtime, game, action_one, object_grid(value));
    }

    assert!(
        runtime.current_best_scene_interpretation().is_some(),
        "precondition: real perceptual experience must first mature into a grounded scene"
    );

    assert_eq!(
        selected_action(runtime),
        None,
        "a grounded scene without an admitted causal action model must still abstain from model-grounded action selection"
    );
}

fn teach_action_one_toggle(runtime: &mut ArcAgi3CognitiveInteractionRuntime, game: &str) {
    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    /*
     * ACTION2 supplies real contrast evidence.
     *
     * ACTION1 repeatedly changes the learned object:
     *     5 -> 6
     *     6 -> 5
     *
     * No rule is injected into the cognitive state.
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
        real_turn(runtime, game, selected, object_grid(value));
    }
}

#[test]
fn real_experience_changes_later_selected_action_through_same_runtime_owner() {
    let game = "p4gb2-action-causal-closure";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, object_grid(1), None), 160_000)
            .unwrap();

    /*
     * SAME action opportunity frontier before and after learning.
     */
    assert_eq!(
        selected_action(&runtime),
        None,
        "without retained causal knowledge the runtime must not fabricate a model-grounded action"
    );

    mature_scene(&mut runtime, game);

    assert_eq!(
        selected_action(&runtime),
        None,
        "perception alone must not manufacture causal executive authority"
    );

    teach_action_one_toggle(&mut runtime, game);

    assert_eq!(
        selected_action(&runtime),
        Some(action(ArcAgi3ActionId::Action1,),),
        "real retained causal experience must change the later M48-selected ARC action"
    );

    /*
     * Candidate input order cannot determine learned behavior.
     */
    let reversed = [
        action(ArcAgi3ActionId::Action2),
        action(ArcAgi3ActionId::Action1),
    ];

    assert_eq!(
        runtime.current_model_grounded_action_selection(
            &reversed,
            &goal(),
            signal(900),
            CognitiveSignal::zero(),
            executive_policy(),
        ),
        Some(action(ArcAgi3ActionId::Action1,),),
        "action selection must be evidence-determined rather than caller-order-determined"
    );
}

#[test]
fn transient_contradiction_preserves_selected_action_but_repeated_contradiction_retracts_it() {
    let game = "p4gb2-action-revision";

    let action_one = action(ArcAgi3ActionId::Action1);

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, object_grid(1), None), 170_000)
            .unwrap();

    mature_scene(&mut runtime, game);

    teach_action_one_toggle(&mut runtime, game);

    assert_eq!(
        selected_action(&runtime),
        Some(action_one),
        "precondition: retained causal model must first authorize ACTION1"
    );

    /*
     * First transient causal contradiction:
     * ACTION1 unexpectedly leaves the object unchanged.
     */
    real_turn(&mut runtime, game, action_one, object_grid(5));

    assert!(
        runtime.current_best_scene_interpretation().is_some(),
        "causal contradiction must not be confused with perceptual scene loss"
    );

    assert_eq!(
        selected_action(&runtime),
        Some(action_one),
        "one transient contradiction must not destroy a twice-supported action policy"
    );

    /*
     * Repeated contradiction at the same grounded relation.
     */
    real_turn(&mut runtime, game, action_one, object_grid(5));

    assert!(
        runtime.current_best_scene_interpretation().is_some(),
        "behavioral retraction must happen while the perceptual scene remains grounded"
    );

    assert_eq!(
        selected_action(&runtime),
        None,
        "repeated structural contradiction must retract stale M48 action authority instead of continuing ACTION1"
    );
}

#[test]
fn global_flash_never_bootstraps_model_grounded_action_selection() {
    let game = "p4gb2-global-flash-negative";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, global_grid(1), None), 180_000)
            .unwrap();

    for (index, value) in [2_u8, 3, 4, 5, 6, 7, 8, 9].into_iter().enumerate() {
        let selected = if index % 2 == 0 {
            action_one
        } else {
            action_two
        };

        real_turn(&mut runtime, game, selected, global_grid(value));

        assert!(
            runtime.current_best_scene_interpretation().is_none(),
            "global synchronous change must remain scene-level abstention"
        );

        assert_eq!(
            selected_action(&runtime),
            None,
            "without a grounded scene/global causal model the executive must not manufacture an action"
        );
    }
}
