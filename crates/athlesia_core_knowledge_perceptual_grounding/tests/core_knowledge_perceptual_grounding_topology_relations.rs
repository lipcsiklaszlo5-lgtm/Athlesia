use athlesia_core_knowledge_perceptual_grounding::{
    CoreKnowledgeTopologicalRelations, ObjectHypothesis, ObjectObservation, ObjecthoodEvidence,
    PerceptualElement, PerceptualElementHandle, PerceptualFrame, TopologicalRelationCompetition,
    TopologicalRelationHypothesis, TopologicalRelationKind, TopologicalRelationPolicy,
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

fn relation(
    subject: ObjectObservation,
    kind: TopologicalRelationKind,
    object: ObjectObservation,
    support: u16,
) -> TopologicalRelationHypothesis {
    TopologicalRelationHypothesis::new(subject, kind, object, signal(support)).unwrap()
}

fn policy(per_pair: usize, total: usize) -> TopologicalRelationPolicy {
    TopologicalRelationPolicy::new(per_pair, total).unwrap()
}

#[test]
fn topological_relation_kind_distinguishes_symmetric_and_directional_capacity() {
    assert!(TopologicalRelationKind::Adjacent.is_symmetric());

    assert!(TopologicalRelationKind::Contact.is_symmetric());

    assert!(TopologicalRelationKind::Overlap.is_symmetric());

    assert!(TopologicalRelationKind::Separate.is_symmetric());

    assert!(!TopologicalRelationKind::Contains.is_symmetric());
}

#[test]
fn relation_hypothesis_requires_same_observation_distinct_objects_and_positive_support() {
    let first_frame = frame(1, &[1, 2]);

    let second_frame = frame(2, &[10]);

    let first = observation(&first_frame, &[1]);

    let second = observation(&first_frame, &[2]);

    let future = observation(&second_frame, &[10]);

    assert_eq!(
        TopologicalRelationHypothesis::new(
            first.clone(),
            TopologicalRelationKind::Adjacent,
            future,
            signal(500,),
        ),
        None
    );

    assert_eq!(
        TopologicalRelationHypothesis::new(
            first.clone(),
            TopologicalRelationKind::Adjacent,
            first,
            signal(500,),
        ),
        None
    );

    assert_eq!(
        TopologicalRelationHypothesis::new(
            second.clone(),
            TopologicalRelationKind::Contact,
            observation(&first_frame, &[1,],),
            CognitiveSignal::zero(),
        ),
        None
    );
}

#[test]
fn symmetric_relations_canonicalize_endpoint_order() {
    let input_frame = frame(3, &[1, 2]);

    let first = observation(&input_frame, &[1]);

    let second = observation(&input_frame, &[2]);

    let forward = relation(
        first.clone(),
        TopologicalRelationKind::Contact,
        second.clone(),
        700,
    );

    let reverse = relation(second, TopologicalRelationKind::Contact, first, 700);

    assert_eq!(forward, reverse);

    assert!(!forward.is_directional());
}

#[test]
fn containment_preserves_direction_and_reverse_containment_is_distinct() {
    let input_frame = frame(4, &[1, 2]);

    let first = observation(&input_frame, &[1]);

    let second = observation(&input_frame, &[2]);

    let forward = relation(
        first.clone(),
        TopologicalRelationKind::Contains,
        second.clone(),
        800,
    );

    let reverse = relation(second, TopologicalRelationKind::Contains, first, 800);

    assert_ne!(forward, reverse);

    assert!(forward.is_directional());

    assert!(reverse.is_directional());
}

#[test]
fn different_relation_kinds_for_the_same_pair_remain_distinct_hypotheses() {
    let input_frame = frame(5, &[1, 2]);

    let first = observation(&input_frame, &[1]);

    let second = observation(&input_frame, &[2]);

    let result = TopologicalRelationCompetition::select(
        &[
            relation(
                first.clone(),
                TopologicalRelationKind::Adjacent,
                second.clone(),
                800,
            ),
            relation(
                first.clone(),
                TopologicalRelationKind::Contact,
                second.clone(),
                700,
            ),
            relation(first, TopologicalRelationKind::Overlap, second, 600),
        ],
        policy(4, 8),
    );

    assert_eq!(result.selected_count(), 3);

    assert_eq!(result.duplicate_relation_count(), 0);
}

#[test]
fn duplicate_exact_relation_keeps_only_the_highest_supported_variant() {
    let input_frame = frame(6, &[1, 2]);

    let first = observation(&input_frame, &[1]);

    let second = observation(&input_frame, &[2]);

    let weak = relation(
        first.clone(),
        TopologicalRelationKind::Adjacent,
        second.clone(),
        300,
    );

    let strong = relation(first, TopologicalRelationKind::Adjacent, second, 900);

    let result = TopologicalRelationCompetition::select(&[weak, strong], policy(4, 8));

    assert_eq!(result.input_relation_count(), 2);

    assert_eq!(result.canonical_relation_count(), 1);

    assert_eq!(result.duplicate_relation_count(), 1);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].support().value(), 900);
}

#[test]
fn hard_per_pair_bound_retains_only_best_competing_relations_for_a_pair() {
    let input_frame = frame(7, &[1, 2]);

    let first = observation(&input_frame, &[1]);

    let second = observation(&input_frame, &[2]);

    let result = TopologicalRelationCompetition::select(
        &[
            relation(
                first.clone(),
                TopologicalRelationKind::Adjacent,
                second.clone(),
                500,
            ),
            relation(
                first.clone(),
                TopologicalRelationKind::Contact,
                second.clone(),
                900,
            ),
            relation(
                first.clone(),
                TopologicalRelationKind::Overlap,
                second.clone(),
                700,
            ),
            relation(first, TopologicalRelationKind::Separate, second, 300),
        ],
        policy(2, 8),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.selected()[0].support().value(), 900);

    assert_eq!(result.selected()[1].support().value(), 700);

    assert_eq!(result.dropped_by_pair_bound_count(), 2);
}

#[test]
fn hard_global_relation_frontier_retains_only_highest_supported_hypotheses() {
    let input_frame = frame(8, &[1, 2, 3, 4]);

    let first = observation(&input_frame, &[1]);

    let second = observation(&input_frame, &[2]);

    let third = observation(&input_frame, &[3]);

    let fourth = observation(&input_frame, &[4]);

    let result = TopologicalRelationCompetition::select(
        &[
            relation(
                first.clone(),
                TopologicalRelationKind::Adjacent,
                second,
                300,
            ),
            relation(
                first.clone(),
                TopologicalRelationKind::Contact,
                third.clone(),
                900,
            ),
            relation(first, TopologicalRelationKind::Overlap, fourth.clone(), 700),
            relation(third, TopologicalRelationKind::Separate, fourth, 500),
        ],
        policy(4, 2),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.selected()[0].support().value(), 900);

    assert_eq!(result.selected()[1].support().value(), 700);

    assert_eq!(result.dropped_by_global_bound_count(), 2);
}

#[test]
fn reversed_symmetric_relations_share_one_pair_and_one_exact_identity() {
    let input_frame = frame(9, &[1, 2]);

    let first = observation(&input_frame, &[1]);

    let second = observation(&input_frame, &[2]);

    let result = TopologicalRelationCompetition::select(
        &[
            relation(
                first.clone(),
                TopologicalRelationKind::Adjacent,
                second.clone(),
                600,
            ),
            relation(
                second.clone(),
                TopologicalRelationKind::Adjacent,
                first.clone(),
                800,
            ),
            relation(second, TopologicalRelationKind::Contact, first, 700),
        ],
        policy(1, 8),
    );

    assert_eq!(result.canonical_relation_count(), 2);

    assert_eq!(result.duplicate_relation_count(), 1);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].support().value(), 800);

    assert_eq!(result.dropped_by_pair_bound_count(), 1);
}

#[test]
fn overlapping_object_hypotheses_can_participate_in_topological_relations() {
    let input_frame = frame(10, &[1, 2, 3]);

    let overlapping_left = observation(&input_frame, &[1, 2]);

    let overlapping_right = observation(&input_frame, &[2, 3]);

    let candidate = relation(
        overlapping_left,
        TopologicalRelationKind::Overlap,
        overlapping_right,
        900,
    );

    assert_eq!(candidate.relation(), TopologicalRelationKind::Overlap);

    assert_eq!(candidate.observation_index(), 10);
}

#[test]
fn competition_does_not_force_one_mutually_exclusive_relation_kind_per_pair() {
    let input_frame = frame(11, &[1, 2]);

    let first = observation(&input_frame, &[1]);

    let second = observation(&input_frame, &[2]);

    let candidates = vec![
        relation(
            first.clone(),
            TopologicalRelationKind::Adjacent,
            second.clone(),
            800,
        ),
        relation(
            first.clone(),
            TopologicalRelationKind::Contact,
            second.clone(),
            800,
        ),
        relation(first, TopologicalRelationKind::Separate, second, 800),
    ];

    let result = TopologicalRelationCompetition::select(&candidates, policy(3, 8));

    assert_eq!(result.selected_count(), 3);

    assert_eq!(result.canonical_relation_count(), 3);

    assert_eq!(result.dropped_by_pair_bound_count(), 0);
}

#[test]
fn topological_relation_competition_is_deterministic_non_mutating_and_facade_equivalent() {
    let input_frame = frame(12, &[1, 2, 3]);

    let first = observation(&input_frame, &[1]);

    let second = observation(&input_frame, &[2]);

    let third = observation(&input_frame, &[3]);

    let candidates = vec![
        relation(
            first.clone(),
            TopologicalRelationKind::Contact,
            second.clone(),
            900,
        ),
        relation(second, TopologicalRelationKind::Adjacent, first, 700),
        relation(
            observation(&input_frame, &[1]),
            TopologicalRelationKind::Contains,
            third,
            800,
        ),
    ];

    let before = candidates.clone();

    let relation_policy = policy(3, 4);

    let direct = TopologicalRelationCompetition::select(&candidates, relation_policy);

    let facade = CoreKnowledgeTopologicalRelations::evaluate(&candidates, relation_policy);

    let repeated = CoreKnowledgeTopologicalRelations::evaluate(&candidates, relation_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(candidates, before);

    assert_eq!(facade.selected_count(), 3);
}
