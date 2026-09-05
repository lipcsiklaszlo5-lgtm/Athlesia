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

fn grid(top: [u8; 5], bottom: [u8; 5]) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![top.to_vec(), bottom.to_vec()]).unwrap()
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

    let step = m51_fixture::begin_arc(runtime, cognitive_action)
        .expect("real M51 executive step must begin");

    assert!(step.orchestration().advanced());

    let completion = runtime
        .complete_environment_turn(observation(game, frame, Some(action)), signal(900))
        .expect("valid environment consequence must commit");

    assert!(completion.has_cognitive_feedback());
}

fn contains_exact_object(
    scene: &athlesia_core_knowledge_perceptual_grounding::SceneInterpretation,
    coordinates: &[(u8, u8)],
) -> bool {
    let expected = coordinates
        .iter()
        .map(|&(x, y)| ArcAgi3PerceptualIngestionBridge::cell_handle(x, y))
        .collect::<std::collections::BTreeSet<_>>();

    scene.hypotheses().iter().any(|hypothesis| {
        hypothesis
            .members()
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            == expected
    })
}

#[test]
fn two_independently_learned_objects_form_one_better_grounded_scene() {
    let game = "p4g-two-object-holdout";

    let bottom = [8, 9, 0, 10, 11];

    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(
        observation(game, grid([1, 1, 0, 2, 2], bottom), None),
        100_000,
    )
    .unwrap();

    /*
     * Two spatially separated pairs undergo their own coherent changes.
     *
     * The separator and lower row remain stable, so cross-object and
     * object-background relations accumulate contradiction rather than
     * common-change support.
     */
    for top in [[3, 3, 0, 4, 4], [5, 5, 0, 6, 6], [7, 7, 0, 12, 12]] {
        real_turn(&mut runtime, game, grid(top, bottom));

        assert!(
            runtime.current_best_scene_interpretation().is_none(),
            "scene semantics must not appear before object evidence itself matures"
        );
    }

    real_turn(&mut runtime, game, grid([13, 13, 0, 14, 14], bottom));

    let scene = runtime
        .current_best_scene_interpretation()
        .expect("two mature provisional objects must yield a grounded scene explanation");

    assert!(scene.is_grounded_in(runtime.perception().latest_frame(),));

    assert_eq!(
        scene.hypothesis_count(),
        2,
        "the best explanation must cover both compatible learned objects rather than arbitrarily choosing only one"
    );

    assert!(
        contains_exact_object(&scene, &[(0, 0), (1, 0)],),
        "left learned object must participate in the best scene"
    );

    assert!(
        contains_exact_object(&scene, &[(3, 0), (4, 0)],),
        "right learned object must participate in the best scene"
    );

    assert!(
        !scene.contains_overlapping_hypotheses(),
        "one scene cannot simultaneously assert mutually overlapping object identities"
    );
}

#[test]
fn global_flash_still_produces_no_scene_interpretation() {
    let game = "p4g-global-flash-holdout";

    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(
        observation(game, grid([1, 1, 1, 1, 1], [1, 1, 1, 1, 1]), None),
        110_000,
    )
    .unwrap();

    for value in [2_u8, 3, 4, 5, 6, 7] {
        real_turn(&mut runtime, game, grid([value; 5], [value; 5]));

        assert!(
            runtime.current_best_scene_interpretation().is_none(),
            "global common change without local object boundary must not be promoted indirectly through scene construction"
        );
    }
}

#[test]
fn losing_one_objects_current_boundary_revises_scene_instead_of_preserving_stale_structure() {
    let game = "p4g-scene-revision-holdout";

    let bottom = [8, 9, 0, 10, 11];

    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(
        observation(game, grid([1, 1, 0, 2, 2], bottom), None),
        120_000,
    )
    .unwrap();

    for top in [
        [3, 3, 0, 4, 4],
        [5, 5, 0, 6, 6],
        [7, 7, 0, 12, 12],
        [13, 13, 0, 14, 14],
    ] {
        real_turn(&mut runtime, game, grid(top, bottom));
    }

    let before = runtime
        .current_best_scene_interpretation()
        .expect("both objects must first be represented");

    assert_eq!(before.hypothesis_count(), 2);

    /*
     * The separator now takes the left object's value.
     *
     * This removes the LEFT pair's current contrast boundary.
     * The right pair remains coherent and locally bounded.
     *
     * Historical evidence is not erased, but the current grounded scene must
     * revise instead of carrying a stale two-object interpretation forward.
     */
    real_turn(&mut runtime, game, grid([15, 15, 15, 1, 1], bottom));

    let after = runtime
        .current_best_scene_interpretation()
        .expect("the still-grounded right object must preserve a partial current scene");

    assert_eq!(
        after.hypothesis_count(),
        1,
        "scene revision must remove the currently unsupported object rather than preserving stale structure"
    );

    assert!(
        !contains_exact_object(&after, &[(0, 0), (1, 0)],),
        "left object must disappear from the current scene when its independent boundary evidence disappears"
    );

    assert!(
        contains_exact_object(&after, &[(3, 0), (4, 0)],),
        "right object must survive scene revision because its evidence remains grounded"
    );
}
