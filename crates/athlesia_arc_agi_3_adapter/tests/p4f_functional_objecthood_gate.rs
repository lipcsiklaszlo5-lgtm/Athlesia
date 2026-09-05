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

fn top_pair_is_eligible(runtime: &ArcAgi3CognitiveInteractionRuntime) -> bool {
    let left = ArcAgi3PerceptualIngestionBridge::cell_handle(0, 0);

    let right = ArcAgi3PerceptualIngestionBridge::cell_handle(1, 0);

    runtime
        .current_objecthood_eligible_groupings()
        .iter()
        .any(|candidate| {
            candidate.member_count() == 2 && candidate.contains(left) && candidate.contains(right)
        })
}

#[test]
fn holdout_compact_coherent_pair_becomes_objecthood_eligible() {
    let game = "p4f-positive-holdout";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1, 1, 8, 9), None), 30_000)
            .unwrap();

    /*
     * The first two consequences establish temporal eligibility.
     *
     * The next two are genuinely new common-change validation.
     *
     * Throughout, the top pair shares one observed appearance feature and
     * remains contrast-separated from the bottom cells.
     */
    for frame in [grid(2, 2, 8, 9), grid(3, 3, 8, 9), grid(4, 4, 8, 9)] {
        real_turn(&mut runtime, game, frame);

        assert!(
            !top_pair_is_eligible(&runtime),
            "objecthood eligibility must not appear before sufficient independent evidence"
        );
    }

    real_turn(&mut runtime, game, grid(5, 5, 8, 9));

    assert!(
        top_pair_is_eligible(&runtime),
        "repeated behavior plus temporal persistence plus appearance cohesion plus boundary must become eligible"
    );

    assert_eq!(
        runtime.current_objecthood_eligible_groupings().len(),
        1,
        "the holdout world contains only one grouping satisfying all evidence families"
    );
}

#[test]
fn global_synchronous_flash_is_not_mistaken_for_an_object() {
    let game = "p4f-global-flash-holdout";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1, 1, 1, 1), None), 40_000)
            .unwrap();

    /*
     * Every cell is adjacent, persistent, visually equal and changes
     * synchronously.
     *
     * A naive common-change detector would call this one object.
     *
     * It has no local contrast boundary, so the multi-axis gate must abstain.
     */
    for value in [2_u8, 3, 4, 5, 6, 7] {
        real_turn(&mut runtime, game, grid(value, value, value, value));

        assert!(
            runtime.current_objecthood_eligible_groupings().is_empty(),
            "global synchronous change without a local boundary must never become objecthood-eligible"
        );
    }
}

#[test]
fn synchronized_but_visually_incoherent_neighbors_are_not_objecthood_eligible() {
    let game = "p4f-incoherent-holdout";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1, 2, 8, 9), None), 50_000)
            .unwrap();

    /*
     * Top cells always change together but remain perceptually distinct.
     *
     * Therefore temporal persistence + adjacency + common change alone are
     * insufficient.
     */
    for frame in [
        grid(3, 4, 8, 9),
        grid(5, 6, 8, 9),
        grid(7, 10, 8, 9),
        grid(11, 12, 8, 9),
        grid(13, 14, 8, 9),
    ] {
        real_turn(&mut runtime, game, frame);

        assert!(
            !top_pair_is_eligible(&runtime),
            "synchronous change without appearance cohesion must not cross the objecthood gate"
        );
    }

    assert!(runtime.current_objecthood_eligible_groupings().is_empty());
}
