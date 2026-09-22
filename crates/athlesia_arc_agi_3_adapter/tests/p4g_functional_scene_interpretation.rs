use athlesia_arc_agi_3_adapter::{
    cognitive_interaction_runtime::ArcAgi3CognitiveInteractionRuntime,
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
    perceptual_ingestion_bridge::ArcAgi3PerceptualIngestionBridge, ArcAgi3Action, ArcAgi3ActionId,
    ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId, ArcAgi3GameState, ArcAgi3Grid,
    ArcAgi3Observation,
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

fn scene_members(
    scene: &athlesia_core_knowledge_perceptual_grounding::SceneInterpretation,
) -> Vec<Vec<(u8, u8)>> {
    let mut objects = scene
        .hypotheses()
        .iter()
        .map(|hypothesis| {
            let mut members = hypothesis
                .members()
                .iter()
                .map(|handle| {
                    ArcAgi3PerceptualIngestionBridge::decode_handle_coordinate(*handle)
                        .expect("scene objects must contain grid cells")
                })
                .collect::<Vec<_>>();
            members.sort_unstable();
            members
        })
        .collect::<Vec<_>>();
    objects.sort();
    objects
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

    assert!(runtime.current_best_scene_interpretation().is_none());

    real_turn(&mut runtime, game, grid([3, 3, 0, 4, 4], bottom));

    assert!(runtime.current_provisional_object_hypotheses().is_empty());
    assert!(
        runtime.current_best_scene_interpretation().is_none(),
        "one observation must not fabricate persistent objects or a scene"
    );

    real_turn(&mut runtime, game, grid([5, 5, 0, 6, 6], bottom));

    let provisional = runtime.current_provisional_object_hypotheses();
    for coordinates in [[(0, 0), (1, 0)], [(2, 0), (2, 1)], [(3, 0), (4, 0)]] {
        let mut members =
            coordinates.map(|(x, y)| ArcAgi3PerceptualIngestionBridge::cell_handle(x, y));
        members.sort_unstable();
        assert!(
            provisional
                .iter()
                .any(|hypothesis| hypothesis.members() == members),
            "persistent cohesive bounded components remain legitimate provisional objects"
        );
    }
    for hypothesis in &provisional {
        assert!(hypothesis.is_grounded_in(runtime.perception().latest_frame()));
        let evidence = hypothesis.evidence();
        assert!(evidence.persistence() > CognitiveSignal::zero());
        assert!(evidence.cohesion() > CognitiveSignal::zero());
        assert!(evidence.boundary() > CognitiveSignal::zero());
        assert_eq!(evidence.common_change(), CognitiveSignal::zero());
    }
    assert!(runtime.current_empirically_coherent_groupings().is_empty());
    assert!(
        runtime.current_best_scene_interpretation().is_none(),
        "appearance-only candidates do not establish a unique scene membership"
    );
    let alternatives = runtime.current_competing_scene_interpretations();
    let separator = vec![(2, 0), (2, 1)];
    assert!(
        alternatives
            .selected()
            .iter()
            .any(|scene| scene_members(scene).contains(&separator)),
        "the provisional separator may participate in an alternative explanation"
    );
    assert!(
        alternatives
            .selected()
            .iter()
            .any(|scene| !scene_members(scene).contains(&separator)),
        "coverage must not force the provisional separator into every explanation"
    );

    real_turn(&mut runtime, game, grid([7, 7, 0, 12, 12], bottom));

    assert!(
        runtime.current_empirically_coherent_groupings().is_empty(),
        "one behavioral observation must not fabricate mature common-change support"
    );
    let immature = runtime.current_provisional_object_hypotheses();
    for expected in &provisional {
        let hypothesis = immature
            .iter()
            .find(|hypothesis| hypothesis.members() == expected.members())
            .expect("provisional objecthood must survive immature behavioral evidence");
        assert_eq!(
            hypothesis.evidence().common_change(),
            CognitiveSignal::zero(),
            "one behavioral observation must not become independent scene support"
        );
    }
    assert!(
        runtime.current_best_scene_interpretation().is_none(),
        "immature behavioral evidence must not resolve scene membership ambiguity"
    );

    real_turn(&mut runtime, game, grid([13, 13, 0, 14, 14], bottom));

    let scene = runtime
        .current_best_scene_interpretation()
        .expect("independently supported objects must retain a grounded scene explanation");

    assert!(scene.is_grounded_in(runtime.perception().latest_frame(),));

    assert_eq!(
        scene_members(&scene),
        vec![vec![(0, 0), (1, 0)], vec![(3, 0), (4, 0)]],
        "independent behavior confirms both changing pairs without requiring the static separator"
    );

    let behaviorally_supported = runtime.current_empirically_coherent_groupings();
    for hypothesis in scene.hypotheses() {
        assert!(hypothesis.evidence().common_change() > CognitiveSignal::zero());
        assert!(
            behaviorally_supported
                .iter()
                .any(|grouping| grouping.members() == hypothesis.members()),
            "scene authority must have retained behavioral evidence for the same physical members"
        );
    }

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
        .expect("the independently behavior-confirmed objects must first be represented");

    assert_eq!(
        scene_members(&before),
        vec![vec![(0, 0), (1, 0)], vec![(3, 0), (4, 0)]],
        "static appearance support must not force extra scene membership"
    );

    /*
     * The separator now takes the left object's value.
     *
     * This removes the LEFT pair's current contrast boundary.
     * The vertical separator also loses its appearance cohesion.
     * The right pair remains coherent and locally bounded.
     *
     * Historical evidence is not erased, but the current grounded scene must
     * revise instead of carrying stale object identities forward.
     */
    real_turn(&mut runtime, game, grid([15, 15, 15, 1, 1], bottom));

    let after = runtime
        .current_best_scene_interpretation()
        .expect("the still-grounded right object must preserve a partial current scene");

    assert_eq!(
        scene_members(&after),
        vec![vec![(3, 0), (4, 0)]],
        "scene revision must retain only the supported right object; the expanded left region lacks retained appearance evidence"
    );

    assert!(after.is_grounded_in(runtime.perception().latest_frame()));
}

mod behavior_evidence_binding {
    use super::*;
    use athlesia_core_knowledge_perceptual_grounding::{
        EmpiricalObjecthoodSignalCalibration, PerceptualElement, PerceptualFrame,
        PerceptualGroupingAppearanceRetentionPolicy, PerceptualGroupingBehaviorObservation,
        PerceptualGroupingBehaviorRetentionPolicy, PerceptualGroupingBehaviorStatus,
        PerceptualGroupingCandidate, PerceptualGroupingCandidateKind, PerceptualObjectProposal,
        PerceptualProposalObservation,
    };
    use athlesia_integrated_cognitive_agent::OnlinePersistentCognitiveState;
    use athlesia_mindstone_sparse_cognition::CognitiveStructure;

    fn fixture() -> (
        OnlinePersistentCognitiveState,
        PerceptualFrame,
        PerceptualGroupingCandidate,
    ) {
        let game = "p4g-two-object-holdout";
        let bottom = [8, 9, 0, 10, 11];
        let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(
            observation(game, grid([1, 1, 0, 2, 2], bottom), None),
            100_000,
        )
        .unwrap();
        real_turn(&mut runtime, game, grid([3, 3, 0, 4, 4], bottom));
        real_turn(&mut runtime, game, grid([5, 5, 0, 6, 6], bottom));
        assert_eq!(
            runtime
                .cognition()
                .perceptual_grouping_behavior_record_count(),
            0
        );
        let grouping = runtime
            .current_objecthood_eligible_groupings()
            .into_iter()
            .find(|grouping| grouping.contains(ArcAgi3PerceptualIngestionBridge::cell_handle(0, 0)))
            .unwrap();
        (
            runtime.cognition().clone(),
            runtime.perception().latest_frame().clone(),
            grouping,
        )
    }

    // Each retained count comes from an observed transition of these exact
    // members. Separate proposal histories are never merged in the fixture.
    fn retain_history(
        state: &mut OnlinePersistentCognitiveState,
        grouping: &PerceptualGroupingCandidate,
        history: &[PerceptualGroupingBehaviorStatus],
    ) {
        use PerceptualGroupingBehaviorStatus::*;
        let atomic = grouping
            .members()
            .iter()
            .map(|handle| PerceptualObjectProposal::new(vec![*handle]).unwrap())
            .collect::<Vec<_>>();
        let make_frame = |index, values: [u64; 2], interrupted: bool| {
            PerceptualFrame::new(
                index,
                grouping
                    .members()
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| !interrupted || *index != 0)
                    .map(|(index, handle)| {
                        PerceptualElement::new(*handle, CognitiveStructure::atom(values[index]))
                    })
                    .collect(),
            )
            .unwrap()
        };
        let mut values = [0, 0];
        for (index, status) in history.iter().enumerate() {
            let previous = make_frame(index as u64, values, false);
            match status {
                UniformChanged => {
                    values[0] += 1;
                    values[1] += 1;
                }
                Mixed => {
                    values[0] += 1;
                }
                UniformStable | BoundaryInterrupted => {}
            }
            let current = make_frame(index as u64 + 1, values, *status == BoundaryInterrupted);
            let observation = PerceptualProposalObservation::observe(&previous, &current, &atomic);
            let result = PerceptualGroupingBehaviorObservation::observe(
                std::slice::from_ref(grouping),
                &observation,
            );
            assert_eq!(result.evidence()[0].status(), *status);
            state.retain_perceptual_grouping_behavior_result(&result);
        }
    }

    fn common_change(
        state: &OnlinePersistentCognitiveState,
        frame: &PerceptualFrame,
        grouping: &PerceptualGroupingCandidate,
        policy: PerceptualGroupingBehaviorRetentionPolicy,
    ) -> CognitiveSignal {
        let before = state.perceptual_grouping_behavior_evidence().clone();
        let objects = state.current_provisional_object_hypotheses_from_groupings(
            frame,
            std::slice::from_ref(grouping),
            PerceptualGroupingAppearanceRetentionPolicy::new(2).unwrap(),
            policy,
        );
        assert_eq!(state.perceptual_grouping_behavior_evidence(), &before);
        assert_eq!(
            objects.len(),
            1,
            "behavioral uncertainty must preserve provisional objecthood"
        );
        objects[0].evidence().common_change()
    }

    #[test]
    fn same_member_histories_cannot_pool_their_way_to_policy_maturity() {
        use PerceptualGroupingBehaviorStatus::*;
        let (mut state, frame, grouping) = fixture();
        let alias = PerceptualGroupingCandidate::new(
            grouping.members().to_vec(),
            PerceptualGroupingCandidateKind::PairwiseRelation,
        )
        .unwrap();
        let policy = PerceptualGroupingBehaviorRetentionPolicy::new(3, 2).unwrap();
        retain_history(&mut state, &grouping, &[UniformChanged]);
        retain_history(&mut state, &alias, &[UniformChanged, UniformChanged]);
        assert_eq!(
            common_change(&state, &frame, &grouping, policy),
            CognitiveSignal::zero()
        );

        retain_history(&mut state, &alias, &[Mixed, UniformChanged]);
        assert_eq!(
            common_change(&state, &frame, &grouping, policy),
            EmpiricalObjecthoodSignalCalibration::from_counts(3, 4).unwrap()
        );
        // The supplied policy, including its advantage over mixed observations,
        // is authoritative; the binding must not bake in the live defaults.
        let stricter = PerceptualGroupingBehaviorRetentionPolicy::new(3, 3).unwrap();
        assert_eq!(
            common_change(&state, &frame, &grouping, stricter),
            CognitiveSignal::zero()
        );
        assert_eq!(state.perceptual_grouping_behavior_record_count(), 2);
    }

    #[test]
    fn equal_length_histories_fail_closed_on_conflicts_regardless_of_provenance_or_insertion() {
        use PerceptualGroupingBehaviorStatus::*;
        let (baseline, frame, grouping) = fixture();
        let alias = PerceptualGroupingCandidate::new(
            grouping.members().to_vec(),
            PerceptualGroupingCandidateKind::PairwiseRelation,
        )
        .unwrap();
        let policy = PerceptualGroupingBehaviorRetentionPolicy::new(2, 2).unwrap();
        let calibrated = EmpiricalObjecthoodSignalCalibration::from_counts(3, 4).unwrap();
        let cases: &[(&[_], &[_], CognitiveSignal)] = &[
            (
                &[UniformChanged, UniformChanged],
                &[Mixed, Mixed],
                CognitiveSignal::zero(),
            ),
            (
                &[UniformChanged; 4],
                &[Mixed, UniformChanged, UniformChanged, UniformChanged],
                CognitiveSignal::zero(),
            ),
            (
                &[UniformChanged, UniformChanged, UniformStable],
                &[UniformChanged, UniformChanged, BoundaryInterrupted],
                CognitiveSignal::zero(),
            ),
            (
                &[Mixed, UniformChanged, UniformChanged, UniformChanged],
                &[Mixed, UniformChanged, UniformChanged, UniformChanged],
                calibrated,
            ),
        ];
        for &(first, second, expected) in cases {
            for swap_provenance in [false, true] {
                let proposals = if swap_provenance {
                    [&alias, &grouping]
                } else {
                    [&grouping, &alias]
                };
                for reverse_insertion in [false, true] {
                    let mut state = baseline.clone();
                    let order = if reverse_insertion { [1, 0] } else { [0, 1] };
                    for index in order {
                        retain_history(&mut state, proposals[index], [first, second][index]);
                    }
                    assert_eq!(common_change(&state, &frame, &grouping, policy), expected);
                    assert_eq!(common_change(&state, &frame, &grouping, policy), expected);
                    assert_eq!(state.perceptual_grouping_behavior_record_count(), 2);
                }
            }
        }
    }
}
