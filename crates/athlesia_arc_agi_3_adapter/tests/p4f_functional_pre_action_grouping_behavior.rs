use athlesia_arc_agi_3_adapter::{
    cognitive_interaction_runtime::ArcAgi3CognitiveInteractionRuntime,
    cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge,
    perceptual_ingestion_bridge::ArcAgi3PerceptualIngestionBridge, ArcAgi3Action, ArcAgi3ActionId,
    ArcAgi3AvailableActions, ArcAgi3FrameSequence, ArcAgi3GameId, ArcAgi3GameState, ArcAgi3Grid,
    ArcAgi3Observation,
};
use athlesia_core_knowledge_perceptual_grounding::{
    PerceptualGroupingBehaviorEvidenceRecord, PerceptualGroupingBehaviorRetentionPolicy,
    PerceptualGroupingBehaviorStatus, PerceptualGroupingBehaviorSupportStatus,
    PerceptualGroupingCandidate, PerceptualGroupingCandidateKind,
    PerceptualGroupingGenerationPolicy, PerceptualProposalTemporalEvidencePolicy,
};
use athlesia_mindstone_sparse_cognition::CognitiveSignal;

mod m51_fixture {
    include!("support/m51_online_orchestration_fixture.rs");
}

fn grid(left: u8, right: u8) -> ArcAgi3Grid {
    ArcAgi3Grid::from_rows(vec![
        vec![left, left, 0, right, right],
        vec![left, left, 0, 8, 9],
    ])
    .unwrap()
}

fn observation(frames: Vec<ArcAgi3Grid>, action: Option<ArcAgi3Action>) -> ArcAgi3Observation {
    ArcAgi3Observation::new(
        ArcAgi3GameId::new("p4f-pre-action-behavior".to_string()).unwrap(),
        ArcAgi3GameState::NotFinished,
        ArcAgi3FrameSequence::new(frames).unwrap(),
        0,
        3,
        ArcAgi3AvailableActions::new(vec![ArcAgi3ActionId::Action1]).unwrap(),
        action,
    )
}

fn real_turn(runtime: &mut ArcAgi3CognitiveInteractionRuntime, frames: Vec<ArcAgi3Grid>) {
    let action = ArcAgi3Action::discrete(ArcAgi3ActionId::Action1).unwrap();
    let step = m51_fixture::begin_arc(
        runtime,
        ArcAgi3CognitiveProtocolBridge::encode_action(action),
    )
    .expect("a real M51 executive step must begin");
    assert!(step.orchestration().advanced());
    let completion = runtime
        .complete_environment_turn(
            observation(frames, Some(action)),
            CognitiveSignal::new(900).unwrap(),
        )
        .unwrap();
    assert!(completion.has_cognitive_feedback());
}

fn mature_appearance() -> ArcAgi3CognitiveInteractionRuntime {
    let mut runtime =
        ArcAgi3CognitiveInteractionRuntime::new(observation(vec![grid(1, 2)], None), 130_000)
            .unwrap();
    real_turn(&mut runtime, vec![grid(3, 4)]);
    real_turn(&mut runtime, vec![grid(5, 6)]);
    runtime
}

fn grouping(coordinates: &[(u8, u8)]) -> PerceptualGroupingCandidate {
    PerceptualGroupingCandidate::new(
        coordinates
            .iter()
            .map(|&(x, y)| ArcAgi3PerceptualIngestionBridge::cell_handle(x, y))
            .collect(),
        PerceptualGroupingCandidateKind::ConnectedComponent,
    )
    .unwrap()
}

fn left_object() -> PerceptualGroupingCandidate {
    grouping(&[(0, 0), (1, 0), (0, 1), (1, 1)])
}

fn temporal_candidates(
    runtime: &ArcAgi3CognitiveInteractionRuntime,
) -> Vec<PerceptualGroupingCandidate> {
    runtime
        .current_perceptual_grouping_frontier(
            PerceptualProposalTemporalEvidencePolicy::new(2).unwrap(),
            PerceptualGroupingGenerationPolicy::new(256, 256).unwrap(),
        )
        .candidates()
        .to_vec()
}

fn records<'a>(
    runtime: &'a ArcAgi3CognitiveInteractionRuntime,
    subject: &PerceptualGroupingCandidate,
) -> Vec<&'a PerceptualGroupingBehaviorEvidenceRecord> {
    runtime
        .cognition()
        .perceptual_grouping_behavior_evidence()
        .records()
        .iter()
        .filter(|record| record.candidate().members() == subject.members())
        .collect()
}

fn common_change(
    runtime: &ArcAgi3CognitiveInteractionRuntime,
    subject: &PerceptualGroupingCandidate,
) -> CognitiveSignal {
    runtime
        .current_provisional_object_hypotheses()
        .iter()
        .find(|object| object.members() == subject.members())
        .expect("appearance-grounded provisional object must exist")
        .evidence()
        .common_change()
}

#[test]
fn pre_action_appearance_object_outside_temporal_frontier_receives_mature_behavior() {
    let mut runtime = mature_appearance();
    let subject = left_object();
    assert!(!temporal_candidates(&runtime)
        .iter()
        .any(|candidate| candidate.members() == subject.members()));
    assert_eq!(common_change(&runtime, &subject), CognitiveSignal::zero());
    assert!(records(&runtime, &subject).is_empty());
    assert!(
        runtime.current_best_scene_interpretation().is_none(),
        "behavior observation cannot require a unique scene"
    );

    let policy = PerceptualGroupingBehaviorRetentionPolicy::new(2, 2).unwrap();
    for (count, left, right) in [(1, 7, 12), (2, 13, 14)] {
        real_turn(&mut runtime, vec![grid(left, right)]);
        let histories = records(&runtime, &subject);
        assert_eq!(
            histories.len(),
            1,
            "pre-action object must receive exactly one physical-member history"
        );
        assert_eq!(histories[0].observation_count(), count);
        assert_eq!(histories[0].uniform_changed_count(), count);
        let status = runtime
            .cognition()
            .perceptual_grouping_behavior_evidence()
            .support_status(histories[0].candidate(), policy);
        if count == 1 {
            assert_eq!(
                status,
                PerceptualGroupingBehaviorSupportStatus::InsufficientCommonChangeEvidence
            );
            assert_eq!(common_change(&runtime, &subject), CognitiveSignal::zero());
            assert!(runtime.current_best_scene_interpretation().is_none());
        } else {
            assert_eq!(status, PerceptualGroupingBehaviorSupportStatus::Supported);
            assert!(common_change(&runtime, &subject) > CognitiveSignal::zero());
            assert!(runtime
                .current_best_scene_interpretation()
                .unwrap()
                .hypotheses()
                .iter()
                .any(|object| object.members() == subject.members()));
        }
    }
}

#[test]
fn consequence_only_component_receives_no_retroactive_behavior() {
    let incoherent =
        ArcAgi3Grid::from_rows(vec![vec![1, 2, 0, 4, 4], vec![3, 5, 0, 8, 9]]).unwrap();
    let mut runtime = ArcAgi3CognitiveInteractionRuntime::new(
        observation(vec![incoherent.clone()], None),
        140_000,
    )
    .unwrap();
    real_turn(&mut runtime, vec![incoherent.clone()]);
    real_turn(&mut runtime, vec![incoherent]);
    let subject = left_object();
    assert!(!runtime
        .current_perceptual_grouping_candidates()
        .iter()
        .any(|candidate| candidate.members() == subject.members()));

    real_turn(&mut runtime, vec![grid(6, 7)]);
    assert!(runtime
        .current_objecthood_eligible_groupings()
        .iter()
        .any(|candidate| candidate.members() == subject.members()));
    assert!(
        records(&runtime, &subject).is_empty(),
        "a consequence cannot validate an identity discovered from itself"
    );

    real_turn(&mut runtime, vec![grid(10, 11)]);
    let histories = records(&runtime, &subject);
    assert_eq!(histories.len(), 1);
    assert_eq!(
        histories[0].observation_count(),
        1,
        "only the later independent action evaluates the new grouping"
    );
    assert_eq!(common_change(&runtime, &subject), CognitiveSignal::zero());
}

#[test]
fn proposal_aliases_and_animation_do_not_duplicate_or_fabricate_behavior() {
    let mut runtime = mature_appearance();
    let subject = grouping(&[(3, 0), (4, 0)]);
    assert!(temporal_candidates(&runtime).iter().any(|candidate| {
        candidate.members() == subject.members()
            && candidate.kind() == PerceptualGroupingCandidateKind::PairwiseRelation
    }));
    assert!(runtime
        .current_objecthood_eligible_groupings()
        .contains(&subject));

    // The first response leaves the subject stable. Its later animation change
    // is not a second independent action consequence.
    real_turn(&mut runtime, vec![grid(7, 6), grid(7, 12)]);
    let histories = records(&runtime, &subject);
    assert_eq!(histories.len(), 1);
    assert_eq!(histories[0].observation_count(), 1);
    assert_eq!(histories[0].uniform_stable_count(), 1);
    assert_eq!(histories[0].uniform_changed_count(), 0);
    let retained_provenance = histories[0].candidate().clone();

    real_turn(&mut runtime, vec![grid(13, 14)]);
    let histories = records(&runtime, &subject);
    assert_eq!(histories.len(), 1);
    assert_eq!(histories[0].candidate(), &retained_provenance);
    assert_eq!(histories[0].observation_count(), 2);
    assert_eq!(histories[0].uniform_changed_count(), 1);
    assert_eq!(common_change(&runtime, &subject), CognitiveSignal::zero());
}

#[test]
fn prior_objects_receive_contradiction_and_disappearance_evidence() {
    let baseline = mature_appearance();
    let subject = left_object();
    for (frame, expected) in [
        (
            ArcAgi3Grid::from_rows(vec![vec![7, 5, 0, 6, 6], vec![5, 5, 0, 8, 9]]).unwrap(),
            PerceptualGroupingBehaviorStatus::Mixed,
        ),
        (
            ArcAgi3Grid::from_rows(vec![vec![7, 7, 0, 6, 6]]).unwrap(),
            PerceptualGroupingBehaviorStatus::BoundaryInterrupted,
        ),
    ] {
        let mut runtime = baseline.clone();
        real_turn(&mut runtime, vec![frame]);
        let histories = records(&runtime, &subject);
        assert_eq!(
            histories.len(),
            1,
            "post-action loss must not erase the pre-action subject of observation"
        );
        assert_eq!(histories[0].observation_count(), 1);
        assert_eq!(histories[0].last_status(), expected);
        assert_eq!(histories[0].uniform_changed_count(), 0);
        assert!(!runtime
            .current_provisional_object_hypotheses()
            .iter()
            .any(|object| object.members() == subject.members()));
        if expected == PerceptualGroupingBehaviorStatus::BoundaryInterrupted {
            let new_component = grouping(&[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]);
            assert!(!temporal_candidates(&baseline)
                .iter()
                .any(|candidate| candidate.members() == new_component.members()));
            assert!(temporal_candidates(&runtime)
                .iter()
                .any(|candidate| candidate.members() == new_component.members()));
            assert!(records(&runtime, &new_component).is_empty(),
                "post-action geometry cannot create a temporal component and validate it retroactively");
        }
    }
}
