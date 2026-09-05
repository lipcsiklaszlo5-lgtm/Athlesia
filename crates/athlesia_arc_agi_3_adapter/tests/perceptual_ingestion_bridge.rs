use athlesia_arc_agi_3_adapter::{
    ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId, ArcAgi3GameState, ArcAgi3Grid,
    ArcAgi3Observation,
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
    perceptual_ingestion_bridge::{
        ArcAgi3PerceptualBridgeError, ArcAgi3PerceptualElementSignature,
        ArcAgi3PerceptualIngestionBridge, UniversalArcAgi3PerceptualIngestionBridge,
    },
};
use athlesia_core_knowledge_perceptual_grounding::{
    IntegratedPerceptualWorldContext, IntegratedPerceptualWorldInput, PerceptualElementHandle,
    PerceptualFrame,
};
use athlesia_integrated_cognitive_agent::{
    OnlinePerceptualGroundingRuntime, PerceptualGroundingIngestionPolicy,
    PerceptualGroundingIngestionRequest,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn observation_with_frames(frames: Vec<ArcAgi3Grid>) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new("perceptual-ingestion-test".to_string()).unwrap(),
        ArcAgi3GameState::NotFinished,
        ArcAgi3FrameSequence::new(frames).unwrap(),
        1,
        4,
        ArcAgi3AvailableActions::new(Vec::new()).unwrap(),
        None,
    )
}

fn grid_a() -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![1, 2, 3], vec![4, 5, 6]]).unwrap()
}

fn grid_b() -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![vec![1, 9, 3], vec![4, 5, 6]]).unwrap()
}

#[test]
fn grid_projects_geometry_and_every_cell_without_information_loss() {
    let frame = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 17).unwrap();

    assert_eq!(frame.observation_index(), 17);
    assert_eq!(frame.element_count(), 7);

    let geometry = frame
        .element(ArcAgi3PerceptualIngestionBridge::geometry_handle())
        .unwrap();

    assert_eq!(
        ArcAgi3PerceptualIngestionBridge::decode_element_signature(geometry.signature()).unwrap(),
        ArcAgi3PerceptualElementSignature::Geometry {
            width: 3,
            height: 2,
        }
    );

    for y in 0_u8..2 {
        for x in 0_u8..3 {
            let element = frame
                .element(ArcAgi3PerceptualIngestionBridge::cell_handle(x, y))
                .unwrap();

            let value = grid_a().cell(usize::from(x), usize::from(y)).unwrap();

            assert_eq!(
                ArcAgi3PerceptualIngestionBridge::decode_element_signature(element.signature())
                    .unwrap(),
                ArcAgi3PerceptualElementSignature::Cell { x, y, value }
            );
        }
    }
}

#[test]
fn coordinate_handle_identity_is_stable_across_frame_values() {
    let first = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 1).unwrap();

    let second = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_b(), 2).unwrap();

    let handle = ArcAgi3PerceptualIngestionBridge::cell_handle(1, 0);

    assert!(first.contains_handle(handle));
    assert!(second.contains_handle(handle));

    assert_ne!(
        first.element(handle).unwrap().signature(),
        second.element(handle).unwrap().signature(),
    );

    assert_eq!(
        ArcAgi3PerceptualIngestionBridge::decode_handle_coordinate(handle),
        Some((1, 0)),
    );
}

#[test]
fn geometry_identity_is_distinct_from_all_cell_handles() {
    let geometry = ArcAgi3PerceptualIngestionBridge::geometry_handle();

    for y in 0_u8..64 {
        for x in 0_u8..64 {
            assert_ne!(
                geometry,
                ArcAgi3PerceptualIngestionBridge::cell_handle(x, y),
            );
        }
    }

    assert_eq!(
        ArcAgi3PerceptualIngestionBridge::decode_handle_coordinate(geometry),
        None,
    );
}

#[test]
fn full_animation_sequence_becomes_ordered_perceptual_frames() {
    let observation = observation_with_frames(vec![
        grid_a(),
        grid_b(),
        ArcAgi3Grid::from_rows(vec![vec![7, 8, 9], vec![10, 11, 12]]).unwrap(),
    ]);

    let projection =
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 40, None).unwrap();

    assert_eq!(projection.frame_count(), 3);

    assert_eq!(projection.frames()[0].observation_index(), 40,);

    assert_eq!(projection.frames()[1].observation_index(), 41,);

    assert_eq!(projection.frames()[2].observation_index(), 42,);

    assert_eq!(projection.next_observation_index(), 43);
}

#[test]
fn cross_turn_transition_is_explicitly_identified_as_environment_causal_boundary() {
    let previous = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 90).unwrap();

    let observation = observation_with_frames(vec![grid_b(), grid_a()]);

    let projection =
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 91, Some(&previous))
            .unwrap();

    assert_eq!(projection.transition_count(), 2);

    let causal = projection
        .causal_environment_transition()
        .expect("cross-turn projection must expose its causal environment transition");

    assert_eq!(causal.previous_frame(), &previous);
    assert_eq!(causal.current_frame(), &projection.frames()[0]);

    assert_eq!(
        causal,
        &projection.transitions()[0],
        "the causal environment transition must be the explicit cross-turn transition"
    );

    assert_ne!(
        causal,
        &projection.transitions()[1],
        "response-internal animation must not be labelled as direct action consequence"
    );
}

#[test]
fn response_internal_animation_has_no_direct_environment_causal_transition_without_prior_frame() {
    let observation = observation_with_frames(vec![grid_a(), grid_b(), grid_a()]);

    let projection =
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 100, None).unwrap();

    assert_eq!(projection.transition_count(), 2);

    assert!(
        projection.causal_environment_transition().is_none(),
        "internal response animation alone must never fabricate action causality"
    );
}

#[test]
fn internal_animation_frames_create_exact_consecutive_transitions() {
    let observation = observation_with_frames(vec![grid_a(), grid_b(), grid_a()]);

    let projection =
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 100, None).unwrap();

    assert_eq!(projection.transition_count(), 2);

    assert_eq!(
        projection.transitions()[0]
            .previous_frame()
            .observation_index(),
        100,
    );

    assert_eq!(
        projection.transitions()[0]
            .current_frame()
            .observation_index(),
        101,
    );

    assert_eq!(
        projection.transitions()[1]
            .previous_frame()
            .observation_index(),
        101,
    );

    assert_eq!(
        projection.transitions()[1]
            .current_frame()
            .observation_index(),
        102,
    );
}

#[test]
fn previous_environment_frame_connects_across_turn_boundary() {
    let previous = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 9).unwrap();

    let observation = observation_with_frames(vec![grid_b()]);

    let projection =
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 10, Some(&previous))
            .unwrap();

    assert_eq!(projection.frame_count(), 1);
    assert_eq!(projection.transition_count(), 1);

    let transition = &projection.transitions()[0];

    assert_eq!(transition.previous_frame(), &previous,);

    assert_eq!(transition.current_frame().observation_index(), 10,);
}

#[test]
fn raw_projected_grid_yields_bounded_evidence_neutral_atomic_proposals() {
    let frame = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 500).unwrap();

    let result = ArcAgi3PerceptualIngestionBridge::atomic_object_proposals(&frame, 3)
        .expect("positive proposal bound is valid");

    assert_eq!(result.input_element_count(), frame.element_count());

    assert_eq!(
        result.excluded_element_count(),
        1,
        "geometry metadata must not enter the object proposal frontier"
    );

    assert_eq!(result.eligible_element_count(), frame.element_count() - 1,);

    assert_eq!(
        result.proposal_count(),
        result.eligible_element_count().min(3),
    );

    assert_eq!(
        result.dropped_by_bound_count(),
        result
            .eligible_element_count()
            .saturating_sub(result.proposal_count()),
    );

    for proposal in result.proposals() {
        assert_eq!(
            proposal.member_count(),
            1,
            "P4A creates only atomic candidates; grouping requires later evidence"
        );

        assert!(
            proposal.is_grounded_in(&frame),
            "every proposal member must preserve exact perceptual grounding"
        );

        assert!(
            !proposal.contains(ArcAgi3PerceptualIngestionBridge::geometry_handle()),
            "geometry meta-element must remain outside object proposals"
        );
    }
}

#[test]
fn atomic_proposal_generation_is_deterministic_and_bound_preserving() {
    let frame = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_b(), 700).unwrap();

    let first = ArcAgi3PerceptualIngestionBridge::atomic_object_proposals(&frame, 4)
        .expect("positive proposal bound is valid");

    let repeated = ArcAgi3PerceptualIngestionBridge::atomic_object_proposals(&frame, 4)
        .expect("positive proposal bound is valid");

    assert_eq!(first, repeated);
    assert!(first.proposal_count() <= 4);

    let handles = first
        .proposals()
        .iter()
        .map(|proposal| proposal.members()[0])
        .collect::<Vec<_>>();

    let mut sorted = handles.clone();
    sorted.sort_unstable();

    assert_eq!(
        handles, sorted,
        "proposal order must follow exact canonical perceptual-handle order"
    );
}

#[test]
fn atomic_proposal_generation_rejects_zero_budget_without_fabricating_output() {
    let frame = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 900).unwrap();

    assert!(ArcAgi3PerceptualIngestionBridge::atomic_object_proposals(&frame, 0).is_none());
}

#[test]
fn atomic_transition_evidence_distinguishes_stable_and_changed_cells_without_objecthood() {
    let previous = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 1000).unwrap();

    let current = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_b(), 1001).unwrap();

    let result =
        ArcAgi3PerceptualIngestionBridge::atomic_transition_evidence(&previous, &current, 64)
            .expect("positive proposal budget is valid");

    assert!(result.evidence_count() > 0);

    assert!(
        result.stable_count() > 0,
        "unchanged exact cell signatures must create stable observational evidence"
    );

    assert!(
        result.changed_count() > 0,
        "changed exact cell signatures must create changed observational evidence"
    );

    for evidence in result.evidence() {
        assert_eq!(evidence.member_count(), 1);

        assert!(
            evidence.has_direct_temporal_evidence(),
            "every retained evidence item must be grounded in at least one frame"
        );

        assert!(
            !evidence
                .proposal()
                .contains(ArcAgi3PerceptualIngestionBridge::geometry_handle()),
            "geometry metadata must remain outside object proposal evidence"
        );
    }
}

#[test]
fn identical_frames_produce_only_stable_atomic_transition_evidence() {
    let previous = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 2000).unwrap();

    let current = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 2001).unwrap();

    let result =
        ArcAgi3PerceptualIngestionBridge::atomic_transition_evidence(&previous, &current, 64)
            .expect("positive proposal budget is valid");

    assert_eq!(result.changed_count(), 0);
    assert_eq!(result.appeared_count(), 0);
    assert_eq!(result.disappeared_count(), 0);

    assert_eq!(
        result.stable_count(),
        result.evidence_count(),
        "exactly identical frames must classify every atomic proposal as stable"
    );
}

#[test]
fn atomic_transition_evidence_is_deterministic_and_creates_no_m46_world_candidates() {
    let previous = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 3000).unwrap();

    let current = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_b(), 3001).unwrap();

    let first =
        ArcAgi3PerceptualIngestionBridge::atomic_transition_evidence(&previous, &current, 4)
            .expect("positive proposal budget is valid");

    let repeated =
        ArcAgi3PerceptualIngestionBridge::atomic_transition_evidence(&previous, &current, 4)
            .expect("positive proposal budget is valid");

    assert_eq!(first, repeated);

    let world_candidates = ArcAgi3PerceptualIngestionBridge::empty_world_candidates();

    assert!(world_candidates.previous_scene_candidates().is_empty());
    assert!(world_candidates.current_scene_candidates().is_empty());
    assert!(world_candidates.persistence_candidates().is_empty());
    assert!(world_candidates.topology_candidates().is_empty());
    assert!(world_candidates.change_candidates().is_empty());
    assert!(world_candidates.action_consequence_candidates().is_empty());
}

#[test]
fn repeated_projected_transitions_accumulate_temporal_support_without_objecthood_promotion() {
    use athlesia_core_knowledge_perceptual_grounding::{
        PerceptualObjectProposal, PerceptualProposalTemporalEvidencePolicy,
        PerceptualProposalTemporalEvidenceState, PerceptualProposalTemporalSupportStatus,
    };

    let first = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 4000).unwrap();

    let second = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_b(), 4001).unwrap();

    let third = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 4002).unwrap();

    let proposal =
        PerceptualObjectProposal::new(vec![ArcAgi3PerceptualIngestionBridge::cell_handle(0, 0)])
            .expect("cell proposal is valid");

    let policy = PerceptualProposalTemporalEvidencePolicy::new(2).unwrap();

    let mut state = PerceptualProposalTemporalEvidenceState::new();

    ArcAgi3PerceptualIngestionBridge::accumulate_atomic_transition_evidence(
        &mut state, &first, &second, 64,
    )
    .expect("first transition evidence is available");

    assert_eq!(
        state.support_status(&proposal, policy),
        PerceptualProposalTemporalSupportStatus::InsufficientHistory
    );

    ArcAgi3PerceptualIngestionBridge::accumulate_atomic_transition_evidence(
        &mut state, &second, &third, 64,
    )
    .expect("second transition evidence is available");

    assert_eq!(
        state.support_status(&proposal, policy),
        PerceptualProposalTemporalSupportStatus::Supported
    );

    let record = state
        .record(&proposal)
        .expect("retained temporal proposal record must exist");

    assert_eq!(record.observation_count(), 2);
    assert_eq!(record.consecutive_cross_frame_presence(), 2);

    let world_candidates = ArcAgi3PerceptualIngestionBridge::empty_world_candidates();

    assert!(
        world_candidates.previous_scene_candidates().is_empty()
            && world_candidates.current_scene_candidates().is_empty()
            && world_candidates.persistence_candidates().is_empty()
            && world_candidates.topology_candidates().is_empty()
            && world_candidates.change_candidates().is_empty()
            && world_candidates.action_consequence_candidates().is_empty(),
        "temporal support must not silently promote proposals into semantic M46 candidates"
    );
}

#[test]
fn retained_temporal_support_generates_competing_grid_grouping_candidates_without_objecthood() {
    use athlesia_core_knowledge_perceptual_grounding::{
        PerceptualGroupingCandidateKind, PerceptualGroupingGenerationPolicy,
        PerceptualProposalTemporalEvidencePolicy, PerceptualProposalTemporalEvidenceState,
    };

    let first = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 5_000).unwrap();

    let second = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_b(), 5_001).unwrap();

    let third = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 5_002).unwrap();

    let mut temporal_state = PerceptualProposalTemporalEvidenceState::new();

    ArcAgi3PerceptualIngestionBridge::accumulate_atomic_transition_evidence(
        &mut temporal_state,
        &first,
        &second,
        64,
    )
    .expect("first exact transition evidence exists");

    ArcAgi3PerceptualIngestionBridge::accumulate_atomic_transition_evidence(
        &mut temporal_state,
        &second,
        &third,
        64,
    )
    .expect("second exact transition evidence exists");

    let temporal_policy = PerceptualProposalTemporalEvidencePolicy::new(2).unwrap();

    let grouping_policy = PerceptualGroupingGenerationPolicy::new(64, 64).unwrap();

    let result = ArcAgi3PerceptualIngestionBridge::temporally_supported_grid_grouping_candidates(
        &temporal_state,
        &third,
        temporal_policy,
        grouping_policy,
    );

    /*
     * grid_a is 3x2:
     *
     * horizontal edges = 4
     * vertical edges   = 3
     * total adjacency relations = 7
     *
     * The relation graph is connected, so one additional six-member
     * connected-component proposal competes with the seven pairwise proposals.
     */
    assert_eq!(result.input_relation_count(), 7);
    assert_eq!(result.considered_relation_count(), 7);
    assert_eq!(result.admitted_relation_count(), 7);

    assert_eq!(result.rejected_ungrounded_relation_count(), 0);
    assert_eq!(result.rejected_temporal_support_count(), 0);

    assert_eq!(result.pairwise_candidate_count(), 7);
    assert_eq!(result.component_candidate_count(), 1);
    assert_eq!(result.candidate_count(), 8);

    assert!(!result.relation_frontier_truncated());
    assert!(!result.candidate_frontier_truncated());

    assert!(result.candidates().iter().any(|candidate| {
        candidate.kind() == PerceptualGroupingCandidateKind::PairwiseRelation
    }));

    assert!(result.candidates().iter().any(|candidate| {
        candidate.kind() == PerceptualGroupingCandidateKind::ConnectedComponent
            && candidate.member_count() == 6
    }));

    for candidate in result.candidates() {
        assert!(candidate.member_count() >= 2);
        assert!(candidate.is_grounded_in(&third));

        assert!(
            !candidate.contains(ArcAgi3PerceptualIngestionBridge::geometry_handle()),
            "geometry metadata must never enter structural cell groupings"
        );
    }

    /*
     * P4D still does not cross the semantic promotion boundary.
     */
    let world_candidates = ArcAgi3PerceptualIngestionBridge::empty_world_candidates();

    assert!(world_candidates.previous_scene_candidates().is_empty());
    assert!(world_candidates.current_scene_candidates().is_empty());
    assert!(world_candidates.persistence_candidates().is_empty());
    assert!(world_candidates.topology_candidates().is_empty());
    assert!(world_candidates.change_candidates().is_empty());
    assert!(world_candidates.action_consequence_candidates().is_empty());
}

#[test]
fn grouping_frontier_abstains_before_temporal_support_and_respects_candidate_bound() {
    use athlesia_core_knowledge_perceptual_grounding::{
        PerceptualGroupingGenerationPolicy, PerceptualProposalTemporalEvidencePolicy,
        PerceptualProposalTemporalEvidenceState,
    };

    let first = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 6_000).unwrap();

    let second = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_b(), 6_001).unwrap();

    let third = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 6_002).unwrap();

    let temporal_policy = PerceptualProposalTemporalEvidencePolicy::new(2).unwrap();

    let grouping_policy = PerceptualGroupingGenerationPolicy::new(64, 3).unwrap();

    let mut temporal_state = PerceptualProposalTemporalEvidenceState::new();

    ArcAgi3PerceptualIngestionBridge::accumulate_atomic_transition_evidence(
        &mut temporal_state,
        &first,
        &second,
        64,
    )
    .expect("first exact transition evidence exists");

    let before_support =
        ArcAgi3PerceptualIngestionBridge::temporally_supported_grid_grouping_candidates(
            &temporal_state,
            &second,
            temporal_policy,
            grouping_policy,
        );

    assert_eq!(before_support.input_relation_count(), 0);
    assert_eq!(before_support.candidate_count(), 0);

    ArcAgi3PerceptualIngestionBridge::accumulate_atomic_transition_evidence(
        &mut temporal_state,
        &second,
        &third,
        64,
    )
    .expect("second exact transition evidence exists");

    let after_support =
        ArcAgi3PerceptualIngestionBridge::temporally_supported_grid_grouping_candidates(
            &temporal_state,
            &third,
            temporal_policy,
            grouping_policy,
        );

    assert_eq!(after_support.input_relation_count(), 7);

    assert_eq!(after_support.candidate_count_before_frontier(), 8);

    assert_eq!(after_support.candidate_count(), 3);
    assert!(after_support.candidate_frontier_truncated());
}

#[test]
fn bridge_never_fabricates_m46_world_hypothesis_candidates() {
    let observation = observation_with_frames(vec![grid_a(), grid_b()]);

    let projection =
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 1, None).unwrap();

    let candidates = projection.transitions()[0].candidates();

    assert!(candidates.previous_scene_candidates().is_empty());

    assert!(candidates.current_scene_candidates().is_empty());

    assert!(candidates.persistence_candidates().is_empty());

    assert!(candidates.topology_candidates().is_empty());

    assert!(candidates.change_candidates().is_empty());

    assert!(candidates.action_consequence_candidates().is_empty());
}

#[test]
fn invalid_cross_turn_temporal_order_is_rejected() {
    let previous = ArcAgi3PerceptualIngestionBridge::project_grid(&grid_a(), 50).unwrap();

    let observation = observation_with_frames(vec![grid_b()]);

    assert_eq!(
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 50, Some(&previous),),
        Err(ArcAgi3PerceptualBridgeError::InvalidIntegratedInput),
    );
}

#[test]
fn observation_index_overflow_is_rejected_before_projection() {
    let observation = observation_with_frames(vec![grid_a(), grid_b()]);

    assert_eq!(
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, u64::MAX, None,),
        Err(ArcAgi3PerceptualBridgeError::ObservationIndexOverflow),
    );
}

#[test]
fn ingestion_request_uses_exact_arc_observation_as_grounded_state() {
    let observation = observation_with_frames(vec![grid_a()]);

    let anchor = CognitiveStructure::ordered(vec![CognitiveStructure::atom(77)]).unwrap();

    let request = ArcAgi3PerceptualIngestionBridge::build_ingestion_request(
        anchor.clone(),
        &observation,
        signal(900),
        signal(12),
    )
    .unwrap();

    assert_eq!(request.anchor_state(), &anchor);

    assert_eq!(
        request.grounded_state(),
        &ArcAgi3CognitiveProtocolBridge::encode_observation(&observation),
    );

    assert_eq!(request.confidence(), signal(900));
    assert_eq!(request.compute_cost(), signal(12));
}

#[test]
fn zero_confidence_is_rejected_by_real_m51_ingestion_contract() {
    let observation = observation_with_frames(vec![grid_a()]);

    assert_eq!(
        ArcAgi3PerceptualIngestionBridge::build_ingestion_request(
            CognitiveStructure::atom(1),
            &observation,
            CognitiveSignal::zero(),
            signal(5),
        ),
        Err(ArcAgi3PerceptualBridgeError::InvalidIngestionRequest),
    );
}

#[test]
fn projection_is_deterministic_and_non_mutating() {
    let observation = observation_with_frames(vec![grid_a(), grid_b()]);

    let original = observation.clone();

    let left =
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 200, None).unwrap();

    let right =
        ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 200, None).unwrap();

    assert_eq!(left, right);
    assert_eq!(observation, original);
}

#[test]
fn universal_facade_matches_direct_projection_exactly() {
    let observation = observation_with_frames(vec![grid_a(), grid_b()]);

    let direct = ArcAgi3PerceptualIngestionBridge::project_observation(&observation, 10, None);

    let facade =
        UniversalArcAgi3PerceptualIngestionBridge::project_observation(&observation, 10, None);

    assert_eq!(direct, facade);
}

#[test]
fn bridge_is_compile_time_bound_to_real_m46_and_m51_types() {
    let _project: fn(
        &ArcAgi3Observation,
        u64,
        Option<&PerceptualFrame>,
    ) -> Result<
        athlesia_arc_agi_3_adapter::perceptual_ingestion_bridge::ArcAgi3PerceptualProjection,
        ArcAgi3PerceptualBridgeError,
    > = ArcAgi3PerceptualIngestionBridge::project_observation;

    let _request: fn(
        CognitiveStructure,
        &ArcAgi3Observation,
        CognitiveSignal,
        CognitiveSignal,
    )
        -> Result<PerceptualGroundingIngestionRequest, ArcAgi3PerceptualBridgeError> =
        ArcAgi3PerceptualIngestionBridge::build_ingestion_request;

    let _runtime: for<'a> fn(
        &'a PerceptualGroundingIngestionRequest,
        &'a IntegratedPerceptualWorldInput,
        IntegratedPerceptualWorldContext,
        PerceptualGroundingIngestionPolicy,
    ) -> OnlinePerceptualGroundingRuntime<'a> = ArcAgi3PerceptualIngestionBridge::online_runtime;

    let _handle: PerceptualElementHandle = ArcAgi3PerceptualIngestionBridge::cell_handle(0, 0);
}
