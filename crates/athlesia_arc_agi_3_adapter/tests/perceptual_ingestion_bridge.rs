use athlesia_arc_agi_3_adapter::{
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
    perceptual_ingestion_bridge::{
        ArcAgi3PerceptualBridgeError, ArcAgi3PerceptualElementSignature,
        ArcAgi3PerceptualIngestionBridge, UniversalArcAgi3PerceptualIngestionBridge,
    },
    ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId, ArcAgi3GameState, ArcAgi3Grid,
    ArcAgi3Observation,
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
