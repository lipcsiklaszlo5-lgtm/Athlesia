use athlesia_core_knowledge_perceptual_grounding::{
    CompetingSceneInterpretations, CoreKnowledgePerceptualGrounding, ObjectHypothesis,
    ObjecthoodEvidence, PerceptualElement, PerceptualElementHandle, PerceptualFrame,
    PerceptualGroundingPolicy, SceneInterpretation,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn handle(value: u64) -> PerceptualElementHandle {
    PerceptualElementHandle::new(value)
}

fn element(value: u64) -> PerceptualElement {
    PerceptualElement::new(
        handle(value),
        CognitiveStructure::atom(value.saturating_mul(10)),
    )
}

fn frame() -> PerceptualFrame {
    PerceptualFrame::new(1, vec![element(1), element(2), element(3), element(4)]).unwrap()
}

fn evidence(
    cohesion: u16,
    persistence: u16,
    common_change: u16,
    boundary: u16,
    containment: u16,
    topology: u16,
) -> ObjecthoodEvidence {
    ObjecthoodEvidence::new(
        signal(cohesion),
        signal(persistence),
        signal(common_change),
        signal(boundary),
        signal(containment),
        signal(topology),
    )
}

fn hypothesis(members: &[u64], support: ObjecthoodEvidence) -> ObjectHypothesis {
    ObjectHypothesis::new(members.iter().copied().map(handle).collect(), support).unwrap()
}

fn scene(hypotheses: Vec<ObjectHypothesis>, support: u16) -> SceneInterpretation {
    SceneInterpretation::new(hypotheses, signal(support)).unwrap()
}

fn policy(max_objects: usize, max_scenes: usize) -> PerceptualGroundingPolicy {
    PerceptualGroundingPolicy::new(max_objects, max_scenes).unwrap()
}

#[test]
fn perceptual_frame_requires_nonempty_unique_observation_local_handles() {
    assert_eq!(PerceptualFrame::new(1, Vec::new(),), None);

    assert_eq!(
        PerceptualFrame::new(1, vec![element(1,), element(1,),],),
        None
    );

    let valid = frame();

    assert_eq!(valid.element_count(), 4);

    assert_eq!(valid.observation_index(), 1);
}

#[test]
fn perceptual_frame_canonicalizes_handle_order_without_interpreting_signatures() {
    let constructed = PerceptualFrame::new(7, vec![element(3), element(1), element(2)]).unwrap();

    assert_eq!(constructed.elements()[0].handle(), handle(1,));

    assert_eq!(constructed.elements()[1].handle(), handle(2,));

    assert_eq!(constructed.elements()[2].handle(), handle(3,));

    assert_eq!(
        constructed.element(handle(2,),).unwrap().signature(),
        &CognitiveStructure::atom(20,)
    );
}

#[test]
fn object_hypothesis_requires_members_and_at_least_one_evidence_channel() {
    assert_eq!(
        ObjectHypothesis::new(Vec::new(), evidence(1000, 0, 0, 0, 0, 0,),),
        None
    );

    assert_eq!(
        ObjectHypothesis::new(vec![handle(1,),], evidence(0, 0, 0, 0, 0, 0,),),
        None
    );

    assert!(ObjectHypothesis::new(vec![handle(1,),], evidence(0, 0, 500, 0, 0, 0,),).is_some());
}

#[test]
fn no_single_objecthood_evidence_axis_is_mandatory() {
    let evidence_variants = [
        evidence(500, 0, 0, 0, 0, 0),
        evidence(0, 500, 0, 0, 0, 0),
        evidence(0, 0, 500, 0, 0, 0),
        evidence(0, 0, 0, 500, 0, 0),
        evidence(0, 0, 0, 0, 500, 0),
        evidence(0, 0, 0, 0, 0, 500),
    ];

    for support in evidence_variants {
        let candidate = ObjectHypothesis::new(vec![handle(1), handle(2)], support).unwrap();

        assert!(candidate.evidence().has_support());

        assert_eq!(candidate.evidence().peak_support().value(), 500);
    }
}

#[test]
fn object_membership_is_canonical_set_and_unknown_frame_members_are_not_grounded() {
    let support = evidence(500, 0, 0, 0, 0, 0);

    let candidate = hypothesis(&[3, 1, 3, 2, 1], support);

    assert_eq!(candidate.members(), &[handle(1,), handle(2,), handle(3,),]);

    assert!(candidate.is_grounded_in(&frame(),));

    let unknown = hypothesis(&[1, 99], support);

    assert!(!unknown.is_grounded_in(&frame(),));
}

#[test]
fn scene_interpretation_allows_overlapping_object_hypotheses() {
    let support = evidence(500, 0, 0, 0, 0, 0);

    let interpretation = scene(
        vec![hypothesis(&[1, 2], support), hypothesis(&[2, 3], support)],
        700,
    );

    assert!(interpretation.contains_overlapping_hypotheses());

    assert!(interpretation.is_grounded_in(&frame(),));
}

#[test]
fn scene_interpretation_rejects_zero_support_and_duplicate_membership_hypotheses() {
    let support = evidence(500, 0, 0, 0, 0, 0);

    assert_eq!(
        SceneInterpretation::new(vec![hypothesis(&[1,], support,),], CognitiveSignal::zero(),),
        None
    );

    assert_eq!(
        SceneInterpretation::new(
            vec![
                hypothesis(&[1, 2,], support,),
                hypothesis(&[2, 1,], evidence(0, 500, 0, 0, 0, 0,),),
            ],
            signal(800,),
        ),
        None
    );
}

#[test]
fn perceptual_grounding_policy_requires_nonzero_object_and_scene_bounds() {
    assert_eq!(PerceptualGroundingPolicy::new(0, 4,), None);

    assert_eq!(PerceptualGroundingPolicy::new(4, 0,), None);

    let valid = policy(3, 2);

    assert_eq!(valid.max_object_hypotheses_per_scene(), 3);

    assert_eq!(valid.max_scene_interpretations(), 2);
}

#[test]
fn scene_competition_rejects_ungrounded_and_overwide_interpretations() {
    let support = evidence(500, 0, 0, 0, 0, 0);

    let valid = scene(vec![hypothesis(&[1, 2], support)], 700);

    let unknown = scene(vec![hypothesis(&[1, 99], support)], 900);

    let too_many = scene(
        vec![
            hypothesis(&[1], support),
            hypothesis(&[2], support),
            hypothesis(&[3], support),
        ],
        1000,
    );

    let result =
        CompetingSceneInterpretations::select(&frame(), &[valid, unknown, too_many], policy(2, 4));

    assert_eq!(result.input_scene_count(), 3);

    assert_eq!(result.valid_scene_count(), 1);

    assert_eq!(result.rejected_scene_count(), 2);

    assert_eq!(result.selected_count(), 1);
}

#[test]
fn hard_scene_frontier_retains_only_highest_supported_interpretations() {
    let support = evidence(500, 0, 0, 0, 0, 0);

    let candidates = vec![
        scene(vec![hypothesis(&[1], support)], 300),
        scene(vec![hypothesis(&[2], support)], 900),
        scene(vec![hypothesis(&[3], support)], 700),
        scene(vec![hypothesis(&[4], support)], 500),
    ];

    let result = CompetingSceneInterpretations::select(&frame(), &candidates, policy(4, 2));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.selected()[0].explanatory_support().value(), 900);

    assert_eq!(result.selected()[1].explanatory_support().value(), 700);

    assert_eq!(result.dropped_by_scene_bound_count(), 2);
}

#[test]
fn equal_support_prefers_simpler_scene_and_selection_is_input_order_invariant() {
    let support = evidence(500, 0, 0, 0, 0, 0);

    let simple = scene(vec![hypothesis(&[1, 2, 3], support)], 800);

    let complex = scene(
        vec![hypothesis(&[1, 2], support), hypothesis(&[3], support)],
        800,
    );

    let first = CompetingSceneInterpretations::select(
        &frame(),
        &[complex.clone(), simple.clone()],
        policy(4, 1),
    );

    let second = CompetingSceneInterpretations::select(&frame(), &[simple, complex], policy(4, 1));

    assert_eq!(first, second);

    assert_eq!(first.selected()[0].hypothesis_count(), 1);
}

#[test]
fn perceptual_grounding_is_deterministic_non_mutating_and_facade_equivalent() {
    let input_frame = frame();

    let frame_before = input_frame.clone();

    let support = evidence(400, 300, 200, 100, 50, 25);

    let candidates = vec![
        scene(vec![hypothesis(&[1, 2], support)], 900),
        scene(
            vec![hypothesis(&[1], support), hypothesis(&[2], support)],
            700,
        ),
    ];

    let candidates_before = candidates.clone();

    let grounding_policy = policy(4, 4);

    let direct = CompetingSceneInterpretations::select(&input_frame, &candidates, grounding_policy);

    let facade =
        CoreKnowledgePerceptualGrounding::evaluate(&input_frame, &candidates, grounding_policy);

    let repeated =
        CoreKnowledgePerceptualGrounding::evaluate(&input_frame, &candidates, grounding_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(input_frame, frame_before);

    assert_eq!(candidates, candidates_before);

    assert_eq!(facade.selected_count(), 2);
}
