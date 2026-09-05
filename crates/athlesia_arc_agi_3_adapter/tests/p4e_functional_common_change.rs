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

fn grid(top_left: u8, top_right: u8, bottom_left: u8, bottom_right: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![
        vec![top_left, top_right],
        vec![bottom_left, bottom_right],
    ])
    .unwrap()
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

fn execute_real_turn(
    runtime: &mut ArcAgi3CognitiveInteractionRuntime,
    game: &str,
    frame: ArcAgi3Grid,
) {
    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action);

    let step = m51_fixture::begin_arc(runtime, cognitive_action)
        .expect("real M51 executive step must begin");

    assert!(
        step.orchestration().advanced(),
        "functional environment driver requires a real executable M51 step"
    );

    let completion = runtime
        .complete_environment_turn(observation(game, frame, Some(action)), signal(900))
        .expect("valid real environment consequence must commit");

    assert!(
        completion.has_cognitive_feedback(),
        "functional experience must be executive-origin feedback"
    );
}

#[test]
fn repeated_coherent_change_is_learned_then_contradicted_then_recovered() {
    let game = "p4e-functional-coherent";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1, 2, 3, 4), None), 10_000)
            .unwrap();

    let top_left = ArcAgi3PerceptualIngestionBridge::cell_handle(0, 0);

    let top_right = ArcAgi3PerceptualIngestionBridge::cell_handle(1, 0);

    let top_pair_is_present = |runtime: &ArcAgi3CognitiveInteractionRuntime| {
        runtime
            .current_empirically_coherent_groupings()
            .iter()
            .any(|candidate| {
                candidate.member_count() == 2
                    && candidate.contains(top_left)
                    && candidate.contains(top_right)
            })
    };

    /*
     * First two consequences establish only temporal eligibility.
     * They must NOT already count as validation of a grouping that did not
     * exist before those observations.
     */
    execute_real_turn(&mut runtime, game, grid(5, 6, 3, 4));

    assert!(
        !top_pair_is_present(&runtime),
        "one experience must not manufacture a coherent grouping"
    );

    execute_real_turn(&mut runtime, game, grid(7, 8, 3, 4));

    assert!(
        !top_pair_is_present(&runtime),
        "the transition that first establishes eligibility must not self-confirm the grouping"
    );

    /*
     * Now the grouping is eligible from PRIOR history.
     * Two NEW consequences make both members change together.
     */
    execute_real_turn(&mut runtime, game, grid(9, 10, 3, 4));

    assert!(
        !top_pair_is_present(&runtime),
        "one common-change validation remains insufficient"
    );

    execute_real_turn(&mut runtime, game, grid(11, 12, 3, 4));

    let learned = runtime.current_empirically_coherent_groupings();

    assert_eq!(
        learned.len(),
        1,
        "only the repeatedly co-changing top pair should emerge as coherent"
    );

    assert!(
        top_pair_is_present(&runtime),
        "repeated new common-change evidence must change externally visible grouping behavior"
    );

    /*
     * Counterexample: only the left member changes.
     * Adjacency and temporal persistence remain true, but group behavior is
     * now contradictory.
     */
    execute_real_turn(&mut runtime, game, grid(13, 12, 3, 4));

    assert!(
        !top_pair_is_present(&runtime),
        "mixed member behavior must retract the previously supported grouping"
    );

    assert!(
        runtime.current_empirically_coherent_groupings().is_empty(),
        "no alternative stable adjacency may masquerade as common-change competence"
    );

    /*
     * A later genuinely coordinated consequence can recover the hypothesis.
     * Historical support is weakened, not destructively forgotten.
     */
    execute_real_turn(&mut runtime, game, grid(14, 15, 3, 4));

    assert!(
        top_pair_is_present(&runtime),
        "renewed coherent evidence must be able to recover a previously contradicted grouping"
    );

    assert_eq!(runtime.current_empirically_coherent_groupings().len(), 1);
}

#[test]
fn adjacency_and_persistence_without_common_behavior_never_become_coherent_grouping() {
    let game = "p4e-functional-adversarial";

    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(game, grid(1, 2, 3, 4), None), 20_000)
            .unwrap();

    let top_left = ArcAgi3PerceptualIngestionBridge::cell_handle(0, 0);

    let top_right = ArcAgi3PerceptualIngestionBridge::cell_handle(1, 0);

    let false_pair_is_present = |runtime: &ArcAgi3CognitiveInteractionRuntime| {
        runtime
            .current_empirically_coherent_groupings()
            .iter()
            .any(|candidate| {
                candidate.member_count() == 2
                    && candidate.contains(top_left)
                    && candidate.contains(top_right)
            })
    };

    /*
     * The pair remains adjacent and present forever.
     * Its members change independently in alternation.
     *
     * This is an explicit counterexample to:
     *
     *     adjacency + persistence == object-like common behavior
     */
    for frame in [
        grid(5, 2, 3, 4),
        grid(5, 6, 3, 4),
        grid(7, 6, 3, 4),
        grid(7, 8, 3, 4),
        grid(9, 8, 3, 4),
        grid(9, 10, 3, 4),
    ] {
        execute_real_turn(&mut runtime, game, frame);

        assert!(
            !false_pair_is_present(&runtime),
            "independent alternating changes must never be accepted as coherent common-change behavior"
        );
    }

    assert!(
        runtime.current_empirically_coherent_groupings().is_empty(),
        "persistent geometry alone must produce no empirically coherent grouping"
    );
}
