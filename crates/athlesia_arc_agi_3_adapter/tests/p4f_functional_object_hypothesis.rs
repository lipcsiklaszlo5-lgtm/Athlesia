use athlesia_arc_agi_3_adapter::{
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
    cognitive_interaction_runtime::ArcAgi3CognitiveInteractionRuntime,
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
    perceptual_ingestion_bridge::ArcAgi3PerceptualIngestionBridge,
};
use athlesia_mindstone_sparse_cognition::CognitiveSignal;

mod m51_fixture {
    include!("support/m51_online_orchestration_fixture.rs");
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn grid(a: u8, b: u8, c: u8, d: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![a, b], vec![c, d]]).unwrap()
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
        ArcAgi3AvailableActions::new(vec![ArcAgi3ActionId::Action1, ArcAgi3ActionId::Action6])
            .unwrap(),
        last_action,
    )
}

fn real_turn(runtime: &mut ArcAgi3CognitiveInteractionRuntime, game: &str, frame: ArcAgi3Grid) {
    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action);

    let step = m51_fixture::begin_arc(runtime, cognitive_action).expect("real M51 step must begin");

    assert!(step.orchestration().advanced());

    let completion = runtime
        .complete_environment_turn(observation(game, frame, Some(action)), signal(900))
        .expect("valid environment consequence must commit");

    assert!(completion.has_cognitive_feedback());
}

fn top_pair_is_object(runtime: &ArcAgi3CognitiveInteractionRuntime) -> bool {
    let left = ArcAgi3PerceptualIngestionBridge::cell_handle(0, 0);

    let right = ArcAgi3PerceptualIngestionBridge::cell_handle(1, 0);

    runtime
        .current_provisional_object_hypotheses()
        .iter()
        .any(|hypothesis| {
            hypothesis.member_count() == 2
                && hypothesis.contains(left)
                && hypothesis.contains(right)
        })
}

#[test]
fn repeated_independent_evidence_produces_real_grounded_object_hypothesis() {
    let game = "p4fb-positive-holdout";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1, 1, 8, 9), None), 60_000)
            .unwrap();

    for frame in [grid(2, 2, 8, 9), grid(3, 3, 8, 9), grid(4, 4, 8, 9)] {
        real_turn(&mut runtime, game, frame);

        assert!(
            !top_pair_is_object(&runtime),
            "a provisional object must not appear before multiple retained evidence families mature"
        );
    }

    real_turn(&mut runtime, game, grid(5, 5, 8, 9));

    let hypotheses = runtime.current_provisional_object_hypotheses();

    assert_eq!(
        hypotheses.len(),
        1,
        "the holdout world contains exactly one grouping with sufficient independent empirical evidence"
    );

    assert!(
        top_pair_is_object(&runtime),
        "the repeatedly coherent compact pair must become a real ObjectHypothesis"
    );

    assert!(
        hypotheses[0].is_grounded_in(runtime.perception().latest_frame(),),
        "emitted object hypothesis must remain grounded in the actual current observation"
    );
}

#[test]
fn global_flash_never_produces_object_hypothesis() {
    let game = "p4fb-global-flash";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1, 1, 1, 1), None), 70_000)
            .unwrap();

    for value in [2_u8, 3, 4, 5, 6, 7] {
        real_turn(&mut runtime, game, grid(value, value, value, value));

        assert!(
            runtime.current_provisional_object_hypotheses().is_empty(),
            "global synchronous flashing has no local object boundary and must never become an object hypothesis"
        );
    }
}

#[test]
fn one_late_visual_match_cannot_overwrite_repeated_visual_contradiction() {
    let game = "p4fb-late-match";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1, 2, 8, 9), None), 80_000)
            .unwrap();

    /*
     * Temporal persistence and common change are genuine.
     *
     * But the top pair is visually incoherent during the first retained
     * appearance opportunities.
     */
    for frame in [
        grid(3, 4, 8, 9),
        grid(5, 6, 8, 9),
        grid(7, 10, 8, 9),
        grid(11, 12, 8, 9),
    ] {
        real_turn(&mut runtime, game, frame);

        assert!(runtime.current_provisional_object_hypotheses().is_empty());
    }

    /*
     * A single late same-valued frame looks locally eligible NOW.
     *
     * It must not erase the retained contradictory visual history.
     */
    real_turn(&mut runtime, game, grid(13, 13, 8, 9));

    assert!(
        runtime.current_provisional_object_hypotheses().is_empty(),
        "one accidental current-frame match must not rewrite repeated contrary appearance evidence"
    );
}

#[test]
fn learned_object_is_retracted_when_current_boundary_disappears() {
    let game = "p4fb-retraction";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1, 1, 8, 8), None), 90_000)
            .unwrap();

    for frame in [
        grid(2, 2, 8, 8),
        grid(3, 3, 8, 8),
        grid(4, 4, 8, 8),
        grid(5, 5, 8, 8),
    ] {
        real_turn(&mut runtime, game, frame);
    }

    assert!(
        top_pair_is_object(&runtime),
        "the compact top pair must first become a provisional object"
    );

    /*
     * Current local contrast boundary vanishes.
     *
     * Historical evidence remains retained, but the present object hypothesis
     * must no longer be emitted as grounded objecthood.
     */
    real_turn(&mut runtime, game, grid(6, 6, 6, 6));

    assert!(
        runtime.current_provisional_object_hypotheses().is_empty(),
        "loss of the current independent boundary axis must retract the grounded object hypothesis"
    );
}
