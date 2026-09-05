use athlesia_arc_agi_3_adapter::{
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
    cognitive_interaction_runtime::ArcAgi3CognitiveInteractionRuntime,
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
};
use athlesia_mindstone_sparse_cognition::CognitiveSignal;
use athlesia_universal_domain_learning::GroundedStructuralPredictionStatus;

mod m51_fixture {
    include!("support/m51_online_orchestration_fixture.rs");
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn discrete(action_id: ArcAgi3ActionId) -> ArcAgi3Action {
    ArcAgi3Action::discrete(action_id).unwrap()
}

fn grid(object_value: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![object_value, object_value], vec![8, 9]]).unwrap()
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
    action: ArcAgi3Action,
    object_value: u8,
) {
    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action);

    let step = m51_fixture::begin_arc(runtime, cognitive_action)
        .expect("real M51 executive step must begin");

    assert!(step.orchestration().advanced());

    let completion = runtime
        .complete_environment_turn(
            observation(game, grid(object_value), Some(action)),
            signal(900),
        )
        .expect("valid environment consequence must commit");

    assert!(completion.has_cognitive_feedback());
}

fn predicts_effect(runtime: &ArcAgi3CognitiveInteractionRuntime, action: ArcAgi3Action) -> bool {
    runtime
        .current_structural_prediction_for_action(action)
        .map(|prediction| prediction.status() == GroundedStructuralPredictionStatus::Predicted)
        .unwrap_or(false)
}

fn mature_object_scene(runtime: &mut ArcAgi3CognitiveInteractionRuntime, game: &str) {
    let action_one = discrete(ArcAgi3ActionId::Action1);

    for value in [2_u8, 3, 4, 5] {
        real_turn(runtime, game, action_one, value);
    }

    assert!(
        runtime.current_best_scene_interpretation().is_some(),
        "precondition: perceptual experience must first mature into a grounded current scene"
    );

    assert!(
        !predicts_effect(runtime, action_one,),
        "a scene alone must not fabricate an executable causal prediction"
    );
}

#[test]
fn real_scene_experience_changes_later_executable_prediction_through_same_runtime_owner() {
    let game = "p4gb1-live-prediction";

    let action_one = discrete(ArcAgi3ActionId::Action1);

    let action_two = discrete(ArcAgi3ActionId::Action2);

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1), None), 130_000).unwrap();

    mature_object_scene(&mut runtime, game);

    /*
     * World mechanics presented only through real environment consequences:
     *
     * ACTION2: no change.
     * ACTION1: 5 <-> 6.
     *
     * Two full supported cycles are supplied.
     *
     * ACTION2 provides the contrast opportunities needed to distinguish an
     * ACTION1-specific effect from a global visual transition.
     */
    for (action, value) in [
        (action_two, 5_u8),
        (action_one, 6_u8),
        (action_two, 6_u8),
        (action_one, 5_u8),
        (action_two, 5_u8),
        (action_one, 6_u8),
        (action_two, 6_u8),
        (action_one, 5_u8),
    ] {
        real_turn(&mut runtime, game, action, value);
    }

    assert!(
        runtime.current_best_scene_interpretation().is_some(),
        "causal learning must be downstream of a still-grounded perceptual scene"
    );

    assert!(
        predicts_effect(&runtime, action_one,),
        "repeated contrasted real experience must make ACTION1 executable in the learned M47 world model"
    );

    assert!(
        !predicts_effect(&runtime, action_two,),
        "the contrast action must remain epistemically non-predictive when it has never produced the learned effect"
    );
}

#[test]
fn transient_failure_preserves_strong_prediction_but_repeated_contradiction_retracts_it() {
    let game = "p4gb1-contradiction-revision";

    let action_one = discrete(ArcAgi3ActionId::Action1);

    let action_two = discrete(ArcAgi3ActionId::Action2);

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1), None), 140_000).unwrap();

    mature_object_scene(&mut runtime, game);

    for (action, value) in [
        (action_two, 5_u8),
        (action_one, 6_u8),
        (action_two, 6_u8),
        (action_one, 5_u8),
        (action_two, 5_u8),
        (action_one, 6_u8),
        (action_two, 6_u8),
        (action_one, 5_u8),
    ] {
        real_turn(&mut runtime, game, action, value);
    }

    assert!(
        predicts_effect(&runtime, action_one,),
        "precondition: two supported cycles must establish the prediction"
    );

    /*
     * First transient failure:
     *
     * ACTION1 unexpectedly leaves the mature object unchanged.
     *
     * The scene itself remains perfectly grounded. Therefore any prediction
     * change is causal-model revision, not perceptual disappearance.
     */
    real_turn(&mut runtime, game, action_one, 5);

    assert!(
        runtime.current_best_scene_interpretation().is_some(),
        "transient causal failure must not erase the still-visible object scene"
    );

    assert!(
        predicts_effect(&runtime, action_one,),
        "one transient contradiction must weaken but not destroy a twice-supported causal prediction"
    );

    /*
     * Repeated contradiction at the same grounded state/action relation.
     *
     * Now empirical precision falls below the production admission threshold.
     */
    real_turn(&mut runtime, game, action_one, 5);

    assert!(
        runtime.current_best_scene_interpretation().is_some(),
        "prediction retraction must occur while perceptual scene evidence remains intact"
    );

    assert!(
        !predicts_effect(&runtime, action_one,),
        "repeated structural contradiction must retract the formerly executable prediction"
    );
}

#[test]
fn global_flash_never_bootstraps_an_executable_world_model() {
    let game = "p4gb1-global-flash";

    let action_one = discrete(ArcAgi3ActionId::Action1);

    let action_two = discrete(ArcAgi3ActionId::Action2);

    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(
        ArcAgi3Observation::new(
            ArcAgi3GameId::new(game.to_string()).unwrap(),
            ArcAgi3GameState::NotFinished,
            ArcAgi3FrameSequence::new(vec![
                ArcAgi3Grid::from_rows(vec![vec![1, 1], vec![1, 1]]).unwrap(),
            ])
            .unwrap(),
            0,
            3,
            ArcAgi3AvailableActions::new(vec![ArcAgi3ActionId::Action1, ArcAgi3ActionId::Action2])
                .unwrap(),
            None,
        ),
        150_000,
    )
    .unwrap();

    for (index, value) in [2_u8, 3, 4, 5, 6, 7, 8, 9].into_iter().enumerate() {
        let action = if index % 2 == 0 {
            action_one
        } else {
            action_two
        };

        let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action);

        let step = m51_fixture::begin_arc(&mut runtime, cognitive_action)
            .expect("real M51 executive step must begin");

        assert!(step.orchestration().advanced());

        let response = ArcAgi3Observation::new(
            ArcAgi3GameId::new(game.to_string()).unwrap(),
            ArcAgi3GameState::NotFinished,
            ArcAgi3FrameSequence::new(vec![
                ArcAgi3Grid::from_rows(vec![vec![value, value], vec![value, value]]).unwrap(),
            ])
            .unwrap(),
            0,
            3,
            ArcAgi3AvailableActions::new(vec![ArcAgi3ActionId::Action1, ArcAgi3ActionId::Action2])
                .unwrap(),
            Some(action),
        );

        let runtime_ref = &mut runtime;

        runtime_ref
            .complete_environment_turn(response, signal(900))
            .expect("valid global-flash consequence must commit");

        assert!(
            runtime_ref.current_best_scene_interpretation().is_none(),
            "global flash must remain scene-level abstention"
        );

        assert!(
            runtime_ref
                .current_structural_prediction_for_action(action_one,)
                .is_none(),
            "without a grounded scene no executable causal world model may be bootstrapped"
        );
    }
}
