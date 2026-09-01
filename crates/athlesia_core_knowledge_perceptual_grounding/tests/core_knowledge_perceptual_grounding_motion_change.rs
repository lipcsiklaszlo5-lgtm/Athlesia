use athlesia_core_knowledge_perceptual_grounding::{
    ChangeEvidence, CoreKnowledgeMotionChange, ObjectHypothesis, ObjectObservation,
    ObjectTransitionObservation, ObjecthoodEvidence, PerceptualChangeCompetition,
    PerceptualChangeHypothesis, PerceptualChangeKind, PerceptualChangePolicy, PerceptualElement,
    PerceptualElementHandle, PerceptualFrame, PersistenceEvidence, PersistenceLinkHypothesis,
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

fn persistence_evidence() -> PersistenceEvidence {
    PersistenceEvidence::new(
        signal(500),
        signal(500),
        signal(0),
        signal(0),
        signal(0),
        signal(0),
    )
}

fn transition(
    previous_frame: &PerceptualFrame,
    previous_members: &[u64],
    current_frame: &PerceptualFrame,
    current_members: &[u64],
) -> ObjectTransitionObservation {
    let link = PersistenceLinkHypothesis::new(
        observation(previous_frame, previous_members),
        observation(current_frame, current_members),
        persistence_evidence(),
    )
    .unwrap();

    ObjectTransitionObservation::from_persistence_link(&link)
}

fn evidence(
    state: u16,
    relational: u16,
    structural: u16,
    temporal: u16,
    commonality: u16,
    causal: u16,
) -> ChangeEvidence {
    ChangeEvidence::new(
        signal(state),
        signal(relational),
        signal(structural),
        signal(temporal),
        signal(commonality),
        signal(causal),
    )
}

fn unary_change(
    transition: ObjectTransitionObservation,
    kind: PerceptualChangeKind,
    descriptor: CognitiveStructure,
    support: ChangeEvidence,
) -> PerceptualChangeHypothesis {
    PerceptualChangeHypothesis::new(transition, kind, None, descriptor, support).unwrap()
}

fn comparative_change(
    transition: ObjectTransitionObservation,
    kind: PerceptualChangeKind,
    reference: ObjectTransitionObservation,
    descriptor: CognitiveStructure,
    support: ChangeEvidence,
) -> PerceptualChangeHypothesis {
    PerceptualChangeHypothesis::new(transition, kind, Some(reference), descriptor, support).unwrap()
}

fn policy(per_transition: usize, total: usize) -> PerceptualChangePolicy {
    PerceptualChangePolicy::new(per_transition, total).unwrap()
}

#[test]
fn transition_is_grounded_in_persistence_link_and_preserves_temporal_endpoints() {
    let first = frame(1, &[1, 2]);

    let fourth = frame(4, &[10, 11, 12]);

    let observed = transition(&first, &[1, 2], &fourth, &[10, 11, 12]);

    assert_eq!(observed.start_index(), 1);

    assert_eq!(observed.end_index(), 4);

    assert_eq!(observed.temporal_gap(), 3);

    assert_eq!(observed.previous().member_count(), 2);

    assert_eq!(observed.current().member_count(), 3);
}

#[test]
fn change_kinds_encode_reference_requirements_without_world_specific_semantics() {
    assert!(!PerceptualChangeKind::StateTransition.requires_reference());

    assert!(!PerceptualChangeKind::Motion.requires_reference());

    assert!(PerceptualChangeKind::RelativeChange.requires_reference());

    assert!(PerceptualChangeKind::CommonChange.requires_reference());

    assert!(!PerceptualChangeKind::RelativeChange.is_symmetric_comparison());

    assert!(PerceptualChangeKind::CommonChange.is_symmetric_comparison());
}

#[test]
fn no_single_change_evidence_axis_is_mandatory() {
    let variants = [
        evidence(600, 0, 0, 0, 0, 0),
        evidence(0, 600, 0, 0, 0, 0),
        evidence(0, 0, 600, 0, 0, 0),
        evidence(0, 0, 0, 600, 0, 0),
        evidence(0, 0, 0, 0, 600, 0),
        evidence(0, 0, 0, 0, 0, 600),
    ];

    for support in variants {
        assert!(support.has_support());

        assert_eq!(support.peak_support().value(), 600);

        assert!(support.change_score() > CognitiveSignal::zero());
    }
}

#[test]
fn comparative_changes_require_distinct_transitions_with_the_same_time_window() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10, 20]);

    let third = frame(3, &[30]);

    let a = transition(&first, &[1], &second, &[10]);

    let b = transition(&first, &[2], &second, &[20]);

    let different_window = transition(&second, &[10], &third, &[30]);

    let support = evidence(0, 500, 0, 500, 0, 0);

    assert_eq!(
        PerceptualChangeHypothesis::new(
            a.clone(),
            PerceptualChangeKind::RelativeChange,
            None,
            CognitiveStructure::atom(1,),
            support,
        ),
        None
    );

    assert_eq!(
        PerceptualChangeHypothesis::new(
            a.clone(),
            PerceptualChangeKind::RelativeChange,
            Some(a.clone(),),
            CognitiveStructure::atom(1,),
            support,
        ),
        None
    );

    assert_eq!(
        PerceptualChangeHypothesis::new(
            a.clone(),
            PerceptualChangeKind::CommonChange,
            Some(different_window,),
            CognitiveStructure::atom(1,),
            support,
        ),
        None
    );

    assert!(
        PerceptualChangeHypothesis::new(
            a,
            PerceptualChangeKind::RelativeChange,
            Some(b,),
            CognitiveStructure::atom(1,),
            support,
        )
        .is_some()
    );
}

#[test]
fn common_change_canonicalizes_transition_pair_but_relative_change_preserves_direction() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10, 20]);

    let a = transition(&first, &[1], &second, &[10]);

    let b = transition(&first, &[2], &second, &[20]);

    let support = evidence(0, 0, 0, 500, 800, 0);

    let common_forward = comparative_change(
        a.clone(),
        PerceptualChangeKind::CommonChange,
        b.clone(),
        CognitiveStructure::atom(7),
        support,
    );

    let common_reverse = comparative_change(
        b.clone(),
        PerceptualChangeKind::CommonChange,
        a.clone(),
        CognitiveStructure::atom(7),
        support,
    );

    assert_eq!(common_forward, common_reverse);

    let relative_forward = comparative_change(
        a.clone(),
        PerceptualChangeKind::RelativeChange,
        b.clone(),
        CognitiveStructure::atom(8),
        support,
    );

    let relative_reverse = comparative_change(
        b,
        PerceptualChangeKind::RelativeChange,
        a,
        CognitiveStructure::atom(8),
        support,
    );

    assert_ne!(relative_forward, relative_reverse);
}

#[test]
fn multiple_change_kinds_and_descriptors_can_coexist_for_one_transition() {
    let first = frame(1, &[1]);

    let second = frame(2, &[10]);

    let observed = transition(&first, &[1], &second, &[10]);

    let candidates = vec![
        unary_change(
            observed.clone(),
            PerceptualChangeKind::StateTransition,
            CognitiveStructure::atom(100),
            evidence(900, 0, 0, 0, 0, 0),
        ),
        unary_change(
            observed.clone(),
            PerceptualChangeKind::Motion,
            CognitiveStructure::atom(200),
            evidence(0, 800, 0, 600, 0, 0),
        ),
        unary_change(
            observed,
            PerceptualChangeKind::Motion,
            CognitiveStructure::atom(201),
            evidence(0, 700, 0, 600, 0, 0),
        ),
    ];

    let result = PerceptualChangeCompetition::select(&candidates, policy(4, 8));

    assert_eq!(result.selected_count(), 3);

    assert_eq!(result.duplicate_hypothesis_count(), 0);
}

#[test]
fn exact_duplicate_change_keeps_only_best_supported_evidence_variant() {
    let first = frame(1, &[1]);

    let second = frame(2, &[10]);

    let observed = transition(&first, &[1], &second, &[10]);

    let weak = unary_change(
        observed.clone(),
        PerceptualChangeKind::StateTransition,
        CognitiveStructure::atom(42),
        evidence(200, 0, 0, 0, 0, 0),
    );

    let strong = unary_change(
        observed,
        PerceptualChangeKind::StateTransition,
        CognitiveStructure::atom(42),
        evidence(900, 900, 0, 0, 0, 0),
    );

    let result = PerceptualChangeCompetition::select(&[weak, strong], policy(4, 8));

    assert_eq!(result.input_hypothesis_count(), 2);

    assert_eq!(result.canonical_hypothesis_count(), 1);

    assert_eq!(result.duplicate_hypothesis_count(), 1);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].evidence().peak_support().value(), 900);
}

#[test]
fn hard_per_transition_bound_counts_comparative_hypotheses_against_every_involved_transition() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10, 20]);

    let a = transition(&first, &[1], &second, &[10]);

    let b = transition(&first, &[2], &second, &[20]);

    let candidates = vec![
        unary_change(
            a.clone(),
            PerceptualChangeKind::StateTransition,
            CognitiveStructure::atom(1),
            evidence(1000, 0, 0, 0, 0, 0),
        ),
        comparative_change(
            a.clone(),
            PerceptualChangeKind::RelativeChange,
            b.clone(),
            CognitiveStructure::atom(2),
            evidence(0, 900, 0, 0, 0, 0),
        ),
        unary_change(
            b,
            PerceptualChangeKind::Motion,
            CognitiveStructure::atom(3),
            evidence(0, 800, 0, 0, 0, 0),
        ),
    ];

    let result = PerceptualChangeCompetition::select(&candidates, policy(1, 8));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.dropped_by_transition_bound_count(), 1);
}

#[test]
fn hard_global_change_frontier_retains_only_highest_scoring_hypotheses() {
    let first = frame(1, &[1, 2, 3, 4]);

    let second = frame(2, &[11, 12, 13, 14]);

    let scores = [(1, 11, 300), (2, 12, 900), (3, 13, 700), (4, 14, 500)];

    let candidates = scores
        .into_iter()
        .map(|(previous, current, support)| {
            unary_change(
                transition(&first, &[previous], &second, &[current]),
                PerceptualChangeKind::StateTransition,
                CognitiveStructure::atom(previous),
                evidence(support, support, 0, 0, 0, 0),
            )
        })
        .collect::<Vec<_>>();

    let result = PerceptualChangeCompetition::select(&candidates, policy(4, 2));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.selected()[0].evidence().peak_support().value(), 900);

    assert_eq!(result.selected()[1].evidence().peak_support().value(), 700);

    assert_eq!(result.dropped_by_global_bound_count(), 2);
}

#[test]
fn changed_membership_and_signature_can_support_motion_without_encoded_direction() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[100, 101, 102]);

    assert_ne!(
        first.element(handle(1,),).unwrap().signature(),
        second.element(handle(100,),).unwrap().signature()
    );

    let observed = transition(&first, &[1, 2], &second, &[100, 101, 102]);

    let motion = unary_change(
        observed,
        PerceptualChangeKind::Motion,
        CognitiveStructure::unordered(vec![
            CognitiveStructure::atom(700),
            CognitiveStructure::atom(701),
        ])
        .unwrap(),
        evidence(0, 800, 700, 900, 0, 0),
    );

    assert_eq!(motion.kind(), PerceptualChangeKind::Motion);

    assert!(motion.change_score() > CognitiveSignal::zero());

    assert_eq!(motion.transition().previous().member_count(), 2);

    assert_eq!(motion.transition().current().member_count(), 3);
}

#[test]
fn relative_and_common_change_can_coexist_without_generating_each_other() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10, 20]);

    let a = transition(&first, &[1], &second, &[10]);

    let b = transition(&first, &[2], &second, &[20]);

    let relative = comparative_change(
        a.clone(),
        PerceptualChangeKind::RelativeChange,
        b.clone(),
        CognitiveStructure::atom(50),
        evidence(0, 800, 0, 700, 0, 0),
    );

    let common = comparative_change(
        a,
        PerceptualChangeKind::CommonChange,
        b,
        CognitiveStructure::atom(51),
        evidence(0, 0, 0, 700, 900, 0),
    );

    let result = PerceptualChangeCompetition::select(&[relative, common], policy(4, 8));

    assert_eq!(result.selected_count(), 2);

    assert!(
        result
            .selected()
            .iter()
            .any(|candidate| { candidate.kind() == PerceptualChangeKind::RelativeChange },)
    );

    assert!(
        result
            .selected()
            .iter()
            .any(|candidate| { candidate.kind() == PerceptualChangeKind::CommonChange },)
    );
}

#[test]
fn motion_change_competition_is_deterministic_non_mutating_and_facade_equivalent() {
    let first = frame(10, &[1, 2]);

    let second = frame(11, &[10, 20]);

    let a = transition(&first, &[1], &second, &[10]);

    let b = transition(&first, &[2], &second, &[20]);

    let candidates = vec![
        unary_change(
            a.clone(),
            PerceptualChangeKind::StateTransition,
            CognitiveStructure::atom(1),
            evidence(800, 300, 200, 700, 0, 100),
        ),
        unary_change(
            b.clone(),
            PerceptualChangeKind::Motion,
            CognitiveStructure::atom(2),
            evidence(0, 700, 500, 800, 0, 100),
        ),
        comparative_change(
            a,
            PerceptualChangeKind::CommonChange,
            b,
            CognitiveStructure::atom(3),
            evidence(0, 0, 0, 600, 900, 100),
        ),
    ];

    let before = candidates.clone();

    let change_policy = policy(4, 6);

    let direct = PerceptualChangeCompetition::select(&candidates, change_policy);

    let facade = CoreKnowledgeMotionChange::evaluate(&candidates, change_policy);

    let repeated = CoreKnowledgeMotionChange::evaluate(&candidates, change_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(candidates, before);

    assert_eq!(facade.selected_count(), 3);
}
