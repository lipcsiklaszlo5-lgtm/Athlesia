use athlesia_core_knowledge_perceptual_grounding::{
    CoreKnowledgePersistenceTracking, ObjectHypothesis, ObjectObservation, ObjecthoodEvidence,
    PerceptualElement, PerceptualElementHandle, PerceptualFrame, PersistenceEvidence,
    PersistenceLinkHypothesis, PersistenceTracking, PersistenceTrackingPolicy,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn handle(value: u64) -> PerceptualElementHandle {
    PerceptualElementHandle::new(value)
}

fn frame(observation_index: u64, values: &[u64]) -> PerceptualFrame {
    PerceptualFrame::new(
        observation_index,
        values
            .iter()
            .copied()
            .map(|value| {
                PerceptualElement::new(
                    handle(value),
                    CognitiveStructure::ordered(vec![
                        CognitiveStructure::atom(observation_index),
                        CognitiveStructure::atom(value),
                    ])
                    .unwrap(),
                )
            })
            .collect(),
    )
    .unwrap()
}

fn objecthood() -> ObjecthoodEvidence {
    ObjecthoodEvidence::new(
        signal(500),
        signal(0),
        signal(0),
        signal(0),
        signal(0),
        signal(0),
    )
}

fn object(members: &[u64]) -> ObjectHypothesis {
    ObjectHypothesis::new(members.iter().copied().map(handle).collect(), objecthood()).unwrap()
}

fn observation(input_frame: &PerceptualFrame, members: &[u64]) -> ObjectObservation {
    ObjectObservation::from_hypothesis(input_frame, &object(members)).unwrap()
}

fn evidence(
    structural: u16,
    relational: u16,
    change: u16,
    boundary: u16,
    containment: u16,
    causal: u16,
) -> PersistenceEvidence {
    PersistenceEvidence::new(
        signal(structural),
        signal(relational),
        signal(change),
        signal(boundary),
        signal(containment),
        signal(causal),
    )
}

fn link(
    previous: ObjectObservation,
    current: ObjectObservation,
    support: PersistenceEvidence,
) -> PersistenceLinkHypothesis {
    PersistenceLinkHypothesis::new(previous, current, support).unwrap()
}

fn policy(predecessors: usize, successors: usize, total: usize) -> PersistenceTrackingPolicy {
    PersistenceTrackingPolicy::new(predecessors, successors, total).unwrap()
}

#[test]
fn object_observation_requires_grounded_hypothesis_and_preserves_time_and_membership() {
    let input_frame = frame(5, &[1, 2, 3]);

    let grounded = observation(&input_frame, &[1, 2]);

    assert_eq!(grounded.observation_index(), 5);

    assert_eq!(grounded.members(), &[handle(1,), handle(2,),]);

    assert_eq!(
        ObjectObservation::from_hypothesis(&input_frame, &object(&[1, 99,],),),
        None
    );
}

#[test]
fn persistence_link_requires_forward_time_and_nonzero_continuity_evidence() {
    let first = frame(1, &[1]);

    let second = frame(2, &[2]);

    let previous = observation(&first, &[1]);

    let current = observation(&second, &[2]);

    assert_eq!(
        PersistenceLinkHypothesis::new(
            current.clone(),
            previous.clone(),
            evidence(500, 0, 0, 0, 0, 0,),
        ),
        None
    );

    assert_eq!(
        PersistenceLinkHypothesis::new(
            previous.clone(),
            current.clone(),
            evidence(0, 0, 0, 0, 0, 0,),
        ),
        None
    );

    assert!(
        PersistenceLinkHypothesis::new(previous, current, evidence(0, 500, 0, 0, 0, 0,),).is_some()
    );
}

#[test]
fn no_single_persistence_evidence_axis_is_mandatory() {
    let first = frame(1, &[1]);

    let second = frame(2, &[10]);

    let previous = observation(&first, &[1]);

    let current = observation(&second, &[10]);

    let variants = [
        evidence(600, 0, 0, 0, 0, 0),
        evidence(0, 600, 0, 0, 0, 0),
        evidence(0, 0, 600, 0, 0, 0),
        evidence(0, 0, 0, 600, 0, 0),
        evidence(0, 0, 0, 0, 600, 0),
        evidence(0, 0, 0, 0, 0, 600),
    ];

    for support in variants {
        let candidate =
            PersistenceLinkHypothesis::new(previous.clone(), current.clone(), support).unwrap();

        assert!(candidate.evidence().has_support());

        assert!(candidate.continuity_score() > CognitiveSignal::zero());
    }
}

#[test]
fn changed_membership_and_changed_perceptual_signature_can_still_support_persistence() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10, 11, 12]);

    assert_ne!(
        first.element(handle(1,),).unwrap().signature(),
        second.element(handle(10,),).unwrap().signature()
    );

    let candidate = link(
        observation(&first, &[1, 2]),
        observation(&second, &[10, 11, 12]),
        evidence(0, 800, 700, 0, 0, 0),
    );

    assert_eq!(candidate.previous().member_count(), 2);

    assert_eq!(candidate.current().member_count(), 3);

    assert!(candidate.continuity_score() > CognitiveSignal::zero());
}

#[test]
fn persistence_tracking_policy_requires_all_hard_bounds_to_be_nonzero() {
    assert_eq!(PersistenceTrackingPolicy::new(0, 1, 1,), None);

    assert_eq!(PersistenceTrackingPolicy::new(1, 0, 1,), None);

    assert_eq!(PersistenceTrackingPolicy::new(1, 1, 0,), None);

    let valid = policy(2, 3, 4);

    assert_eq!(valid.max_predecessors_per_current(), 2);

    assert_eq!(valid.max_successors_per_previous(), 3);

    assert_eq!(valid.max_total_links(), 4);
}

#[test]
fn duplicate_transition_keeps_the_best_supported_persistence_variant() {
    let first = frame(1, &[1]);

    let second = frame(2, &[10]);

    let previous = observation(&first, &[1]);

    let current = observation(&second, &[10]);

    let weak = link(
        previous.clone(),
        current.clone(),
        evidence(200, 0, 0, 0, 0, 0),
    );

    let strong = link(previous, current, evidence(900, 900, 900, 0, 0, 0));

    let result = PersistenceTracking::select(&[weak, strong], policy(4, 4, 4));

    assert_eq!(result.input_link_count(), 2);

    assert_eq!(result.canonical_link_count(), 1);

    assert_eq!(result.duplicate_transition_count(), 1);

    assert_eq!(result.selected_count(), 1);

    assert!(result.selected()[0].continuity_score().value() > 200);
}

#[test]
fn multiple_predecessors_can_compete_for_one_current_object() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10]);

    let current = observation(&second, &[10]);

    let result = PersistenceTracking::select(
        &[
            link(
                observation(&first, &[1]),
                current.clone(),
                evidence(800, 0, 0, 0, 0, 0),
            ),
            link(
                observation(&first, &[2]),
                current,
                evidence(700, 0, 0, 0, 0, 0),
            ),
        ],
        policy(2, 2, 4),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.dropped_by_predecessor_bound_count(), 0);
}

#[test]
fn multiple_successors_can_compete_from_one_previous_object() {
    let first = frame(1, &[1]);

    let second = frame(2, &[10, 11]);

    let previous = observation(&first, &[1]);

    let result = PersistenceTracking::select(
        &[
            link(
                previous.clone(),
                observation(&second, &[10]),
                evidence(800, 0, 0, 0, 0, 0),
            ),
            link(
                previous,
                observation(&second, &[11]),
                evidence(700, 0, 0, 0, 0, 0),
            ),
        ],
        policy(2, 2, 4),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.dropped_by_successor_bound_count(), 0);
}

#[test]
fn hard_global_tracking_frontier_retains_only_highest_supported_links() {
    let first = frame(1, &[1, 2, 3, 4]);

    let second = frame(2, &[11, 12, 13, 14]);

    let candidates = [(1, 11, 300), (2, 12, 900), (3, 13, 700), (4, 14, 500)]
        .into_iter()
        .map(|(previous_value, current_value, support)| {
            link(
                observation(&first, &[previous_value]),
                observation(&second, &[current_value]),
                evidence(support, support, 0, 0, 0, 0),
            )
        })
        .collect::<Vec<_>>();

    let result = PersistenceTracking::select(&candidates, policy(4, 4, 2));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.selected()[0].previous().members(), &[handle(2,),]);

    assert_eq!(result.selected()[1].previous().members(), &[handle(3,),]);

    assert_eq!(result.dropped_by_global_bound_count(), 2);
}

#[test]
fn equal_continuity_support_prefers_shorter_temporal_gap_without_requiring_adjacency() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10]);

    let fourth = frame(4, &[40]);

    let support = evidence(600, 600, 0, 0, 0, 0);

    let short_gap = link(
        observation(&first, &[1]),
        observation(&second, &[10]),
        support,
    );

    let long_gap = link(
        observation(&first, &[2]),
        observation(&fourth, &[40]),
        support,
    );

    let result = PersistenceTracking::select(&[long_gap, short_gap], policy(4, 4, 4));

    assert_eq!(result.selected()[0].temporal_gap(), 1);

    assert_eq!(result.selected()[1].temporal_gap(), 3);
}

#[test]
fn merge_and_split_ambiguity_can_coexist_without_forced_one_to_one_assignment() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10, 11]);

    let a = observation(&first, &[1]);

    let b = observation(&first, &[2]);

    let c = observation(&second, &[10]);

    let d = observation(&second, &[11]);

    let support = evidence(700, 500, 0, 0, 0, 0);

    let result = PersistenceTracking::select(
        &[
            link(a.clone(), c.clone(), support),
            link(b, c, support),
            link(a, d, support),
        ],
        policy(2, 2, 6),
    );

    assert_eq!(result.selected_count(), 3);

    assert_eq!(result.dropped_by_predecessor_bound_count(), 0);

    assert_eq!(result.dropped_by_successor_bound_count(), 0);
}

#[test]
fn persistence_tracking_is_deterministic_non_mutating_and_facade_equivalent() {
    let first = frame(10, &[1, 2]);

    let second = frame(11, &[10, 20]);

    let candidates = vec![
        link(
            observation(&first, &[1]),
            observation(&second, &[10]),
            evidence(800, 400, 300, 200, 100, 50),
        ),
        link(
            observation(&first, &[2]),
            observation(&second, &[20]),
            evidence(700, 500, 300, 200, 100, 50),
        ),
    ];

    let before = candidates.clone();

    let tracking_policy = policy(2, 2, 4);

    let direct = PersistenceTracking::select(&candidates, tracking_policy);

    let facade = CoreKnowledgePersistenceTracking::evaluate(&candidates, tracking_policy);

    let repeated = CoreKnowledgePersistenceTracking::evaluate(&candidates, tracking_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(candidates, before);

    assert_eq!(facade.selected_count(), 2);
}
