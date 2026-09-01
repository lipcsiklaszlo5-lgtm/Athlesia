use athlesia_core_knowledge_perceptual_grounding::{
    ActionConsequenceEvidence, ActionConsequenceHypothesis, ActionConsequencePolicy,
    ActionObservation, ActionSource, ChangeEvidence, CoreKnowledgePerceptualWorld,
    IntegratedPerceptualWorld, IntegratedPerceptualWorldCandidates,
    IntegratedPerceptualWorldContext, IntegratedPerceptualWorldInput, ObjectHypothesis,
    ObjectObservation, ObjectTransitionObservation, ObjecthoodEvidence, PerceptualChangeHypothesis,
    PerceptualChangeKind, PerceptualChangePolicy, PerceptualElement, PerceptualElementHandle,
    PerceptualFrame, PerceptualGroundingPolicy, PersistenceEvidence, PersistenceLinkHypothesis,
    PersistenceTrackingPolicy, SceneInterpretation, TopologicalRelationHypothesis,
    TopologicalRelationKind, TopologicalRelationPolicy,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn handle(value: u64) -> PerceptualElementHandle {
    PerceptualElementHandle::new(value)
}

fn frame(index: u64, values: &[u64]) -> PerceptualFrame {
    PerceptualFrame::new(
        index,
        values
            .iter()
            .copied()
            .map(|value| {
                PerceptualElement::new(
                    handle(value),
                    CognitiveStructure::ordered(vec![
                        CognitiveStructure::atom(index),
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
        signal(600),
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

fn scene(objects: Vec<ObjectHypothesis>, support: u16) -> SceneInterpretation {
    SceneInterpretation::new(objects, signal(support)).unwrap()
}

fn observation(input_frame: &PerceptualFrame, members: &[u64]) -> ObjectObservation {
    ObjectObservation::from_hypothesis(input_frame, &object(members)).unwrap()
}

fn persistence_evidence(support: u16) -> PersistenceEvidence {
    PersistenceEvidence::new(
        signal(support),
        signal(support),
        signal(0),
        signal(0),
        signal(0),
        signal(0),
    )
}

fn link(
    previous_frame: &PerceptualFrame,
    previous_members: &[u64],
    current_frame: &PerceptualFrame,
    current_members: &[u64],
    support: u16,
) -> PersistenceLinkHypothesis {
    PersistenceLinkHypothesis::new(
        observation(previous_frame, previous_members),
        observation(current_frame, current_members),
        persistence_evidence(support),
    )
    .unwrap()
}

fn transition(input_link: &PersistenceLinkHypothesis) -> ObjectTransitionObservation {
    ObjectTransitionObservation::from_persistence_link(input_link)
}

fn change_evidence(support: u16) -> ChangeEvidence {
    ChangeEvidence::new(
        signal(support),
        signal(0),
        signal(0),
        signal(support),
        signal(0),
        signal(0),
    )
}

fn change(
    input_transition: ObjectTransitionObservation,
    descriptor: u64,
    support: u16,
) -> PerceptualChangeHypothesis {
    PerceptualChangeHypothesis::new(
        input_transition,
        PerceptualChangeKind::StateTransition,
        None,
        CognitiveStructure::atom(descriptor),
        change_evidence(support),
    )
    .unwrap()
}

fn topology(
    subject: ObjectObservation,
    object: ObjectObservation,
    support: u16,
) -> TopologicalRelationHypothesis {
    TopologicalRelationHypothesis::new(
        subject,
        TopologicalRelationKind::Adjacent,
        object,
        signal(support),
    )
    .unwrap()
}

fn action_evidence(association: u16, causal_lift: u16) -> ActionConsequenceEvidence {
    ActionConsequenceEvidence::new(
        signal(association),
        signal(association),
        signal(0),
        signal(0),
        signal(0),
        signal(causal_lift),
    )
}

fn consequence(
    event_index: u64,
    input_change: PerceptualChangeHypothesis,
    descriptor: u64,
    association: u16,
    causal_lift: u16,
) -> ActionConsequenceHypothesis {
    ActionConsequenceHypothesis::new(
        ActionObservation::new(
            event_index,
            ActionSource::SelfGenerated,
            CognitiveStructure::atom(descriptor),
        ),
        input_change,
        CognitiveStructure::atom(descriptor.saturating_add(1000)),
        action_evidence(association, causal_lift),
    )
    .unwrap()
}

fn candidates(
    previous_scenes: Vec<SceneInterpretation>,
    current_scenes: Vec<SceneInterpretation>,
    persistence: Vec<PersistenceLinkHypothesis>,
    topology: Vec<TopologicalRelationHypothesis>,
    changes: Vec<PerceptualChangeHypothesis>,
    consequences: Vec<ActionConsequenceHypothesis>,
) -> IntegratedPerceptualWorldCandidates {
    IntegratedPerceptualWorldCandidates::new(
        previous_scenes,
        current_scenes,
        persistence,
        topology,
        changes,
        consequences,
    )
}

fn input(
    previous: PerceptualFrame,
    current: PerceptualFrame,
    world_candidates: IntegratedPerceptualWorldCandidates,
) -> IntegratedPerceptualWorldInput {
    IntegratedPerceptualWorldInput::new(previous, current, world_candidates).unwrap()
}

fn context(
    scene_max_objects: usize,
    scene_max_scenes: usize,
    persistence_total: usize,
    topology_total: usize,
    change_total: usize,
    action_total: usize,
) -> IntegratedPerceptualWorldContext {
    IntegratedPerceptualWorldContext::new(
        PerceptualGroundingPolicy::new(scene_max_objects, scene_max_scenes).unwrap(),
        PersistenceTrackingPolicy::new(4, 4, persistence_total).unwrap(),
        TopologicalRelationPolicy::new(4, topology_total).unwrap(),
        PerceptualChangePolicy::new(4, change_total).unwrap(),
        ActionConsequencePolicy::new(4, 4, action_total).unwrap(),
    )
}

#[test]
fn integrated_input_requires_strictly_forward_frame_order() {
    let first = frame(1, &[1]);

    let second = frame(2, &[10]);

    let empty = || {
        candidates(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    };

    assert!(IntegratedPerceptualWorldInput::new(first.clone(), second.clone(), empty(),).is_some());

    assert_eq!(
        IntegratedPerceptualWorldInput::new(second.clone(), first, empty(),),
        None
    );

    assert_eq!(
        IntegratedPerceptualWorldInput::new(second.clone(), second, empty(),),
        None
    );
}

#[test]
fn persistence_links_require_objects_selected_by_both_scene_interpretations() {
    let previous = frame(1, &[1, 2]);

    let current = frame(2, &[10]);

    let accepted = link(&previous, &[1], &current, &[10], 900);

    let rejected = link(&previous, &[2], &current, &[10], 800);

    let world_input = input(
        previous,
        current,
        candidates(
            vec![scene(vec![object(&[1])], 900)],
            vec![scene(vec![object(&[10])], 900)],
            vec![accepted, rejected],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
    );

    let result = IntegratedPerceptualWorld::evaluate(&world_input, context(4, 2, 4, 4, 4, 4));

    assert_eq!(result.persistence().selected_count(), 1);

    assert_eq!(result.rejected_persistence_dependency_count(), 1);
}

#[test]
fn topology_relations_require_both_endpoint_objects_in_selected_scenes() {
    let previous = frame(1, &[1]);

    let current = frame(2, &[10, 20, 30]);

    let accepted = topology(
        observation(&current, &[10]),
        observation(&current, &[20]),
        900,
    );

    let rejected = topology(
        observation(&current, &[10]),
        observation(&current, &[30]),
        800,
    );

    let world_input = input(
        previous,
        current,
        candidates(
            vec![scene(vec![object(&[1])], 900)],
            vec![scene(vec![object(&[10]), object(&[20])], 900)],
            Vec::new(),
            vec![accepted, rejected],
            Vec::new(),
            Vec::new(),
        ),
    );

    let result = IntegratedPerceptualWorld::evaluate(&world_input, context(4, 2, 4, 4, 4, 4));

    assert_eq!(result.topology().selected_count(), 1);

    assert_eq!(result.rejected_topology_dependency_count(), 1);
}

#[test]
fn changes_require_selected_persistence_transitions_and_cannot_bypass_tracking() {
    let previous = frame(1, &[1, 2]);

    let current = frame(2, &[10, 20]);

    let accepted_link = link(&previous, &[1], &current, &[10], 900);

    let rejected_link = link(&previous, &[2], &current, &[20], 800);

    let accepted_change = change(transition(&accepted_link), 1, 900);

    let rejected_change = change(transition(&rejected_link), 2, 900);

    let world_input = input(
        previous,
        current,
        candidates(
            vec![scene(vec![object(&[1])], 900)],
            vec![scene(vec![object(&[10])], 900)],
            vec![accepted_link, rejected_link],
            Vec::new(),
            vec![accepted_change, rejected_change],
            Vec::new(),
        ),
    );

    let result = IntegratedPerceptualWorld::evaluate(&world_input, context(4, 2, 4, 4, 4, 4));

    assert_eq!(result.persistence().selected_count(), 1);

    assert_eq!(result.changes().selected_count(), 1);

    assert_eq!(result.rejected_change_dependency_count(), 1);
}

#[test]
fn action_consequences_require_selected_change_identity_and_cannot_resurrect_rejected_change() {
    let previous = frame(1, &[1, 2]);

    let current = frame(2, &[10, 20]);

    let accepted_link = link(&previous, &[1], &current, &[10], 900);

    let rejected_link = link(&previous, &[2], &current, &[20], 800);

    let accepted_change = change(transition(&accepted_link), 1, 900);

    let rejected_change = change(transition(&rejected_link), 2, 900);

    let accepted_consequence = consequence(1, accepted_change.clone(), 100, 800, 0);

    let rejected_consequence = consequence(1, rejected_change.clone(), 200, 800, 0);

    let world_input = input(
        previous,
        current,
        candidates(
            vec![scene(vec![object(&[1])], 900)],
            vec![scene(vec![object(&[10])], 900)],
            vec![accepted_link, rejected_link],
            Vec::new(),
            vec![accepted_change, rejected_change],
            vec![accepted_consequence, rejected_consequence],
        ),
    );

    let result = IntegratedPerceptualWorld::evaluate(&world_input, context(4, 2, 4, 4, 4, 4));

    assert_eq!(result.action_consequences().selected_count(), 1);

    assert_eq!(result.rejected_action_consequence_dependency_count(), 1);
}

#[test]
fn winning_scene_grouping_controls_downstream_object_identity_instead_of_raw_frame_membership() {
    let previous = frame(1, &[1, 2]);

    let current = frame(2, &[10, 20]);

    let separate_link = link(&previous, &[1], &current, &[10], 900);

    let world_input = input(
        previous,
        current,
        candidates(
            vec![
                scene(vec![object(&[1, 2])], 900),
                scene(vec![object(&[1]), object(&[2])], 800),
            ],
            vec![
                scene(vec![object(&[10, 20])], 900),
                scene(vec![object(&[10]), object(&[20])], 800),
            ],
            vec![separate_link],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
    );

    let result = IntegratedPerceptualWorld::evaluate(&world_input, context(4, 1, 4, 4, 4, 4));

    assert_eq!(result.previous_scene().selected_count(), 1);

    assert_eq!(result.current_scene().selected_count(), 1);

    assert_eq!(result.persistence().selected_count(), 0);

    assert_eq!(result.rejected_persistence_dependency_count(), 1);
}

#[test]
fn competing_scene_interpretations_can_preserve_alternative_downstream_identity_when_bounds_allow()
{
    let previous = frame(1, &[1, 2]);

    let current = frame(2, &[10, 20]);

    let separate_link = link(&previous, &[1], &current, &[10], 900);

    let world_input = input(
        previous,
        current,
        candidates(
            vec![
                scene(vec![object(&[1, 2])], 900),
                scene(vec![object(&[1]), object(&[2])], 800),
            ],
            vec![
                scene(vec![object(&[10, 20])], 900),
                scene(vec![object(&[10]), object(&[20])], 800),
            ],
            vec![separate_link],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
    );

    let result = IntegratedPerceptualWorld::evaluate(&world_input, context(4, 2, 4, 4, 4, 4));

    assert_eq!(result.previous_scene().selected_count(), 2);

    assert_eq!(result.current_scene().selected_count(), 2);

    assert_eq!(result.persistence().selected_count(), 1);

    assert_eq!(result.rejected_persistence_dependency_count(), 0);
}

#[test]
fn hard_bounds_remain_active_at_every_integrated_frontier() {
    let previous = frame(1, &[1, 2]);

    let current = frame(2, &[10, 20]);

    let link_a = link(&previous, &[1], &current, &[10], 900);

    let link_b = link(&previous, &[2], &current, &[20], 800);

    let change_a = change(transition(&link_a), 1, 900);

    let change_b = change(transition(&link_b), 2, 800);

    let world_input = input(
        previous.clone(),
        current.clone(),
        candidates(
            vec![scene(vec![object(&[1]), object(&[2])], 900)],
            vec![scene(vec![object(&[10]), object(&[20])], 900)],
            vec![link_a.clone(), link_b.clone()],
            vec![
                topology(
                    observation(&current, &[10]),
                    observation(&current, &[20]),
                    900,
                ),
                TopologicalRelationHypothesis::new(
                    observation(&current, &[10]),
                    TopologicalRelationKind::Contact,
                    observation(&current, &[20]),
                    signal(800),
                )
                .unwrap(),
            ],
            vec![change_a.clone(), change_b.clone()],
            vec![
                consequence(1, change_a, 100, 900, 0),
                consequence(1, change_b, 200, 800, 0),
            ],
        ),
    );

    let result = IntegratedPerceptualWorld::evaluate(&world_input, context(4, 1, 1, 1, 1, 1));

    assert_eq!(result.previous_scene().selected_count(), 1);

    assert_eq!(result.current_scene().selected_count(), 1);

    assert_eq!(result.persistence().selected_count(), 1);

    assert_eq!(result.topology().selected_count(), 1);

    assert_eq!(result.changes().selected_count(), 1);

    assert_eq!(result.action_consequences().selected_count(), 1);
}

#[test]
fn association_without_causal_lift_remains_a_hypothesis_but_is_not_promoted_to_fact() {
    let previous = frame(1, &[1]);

    let current = frame(2, &[10]);

    let persistent = link(&previous, &[1], &current, &[10], 900);

    let observed_change = change(transition(&persistent), 1, 900);

    let possible_consequence = consequence(1, observed_change.clone(), 100, 900, 0);

    let world_input = input(
        previous,
        current,
        candidates(
            vec![scene(vec![object(&[1])], 900)],
            vec![scene(vec![object(&[10])], 900)],
            vec![persistent],
            Vec::new(),
            vec![observed_change],
            vec![possible_consequence],
        ),
    );

    let result = IntegratedPerceptualWorld::evaluate(&world_input, context(4, 1, 4, 4, 4, 4));

    assert_eq!(result.action_consequences().selected_count(), 1);

    assert_eq!(
        result.action_consequences().selected()[0]
            .evidence()
            .causal_lift(),
        CognitiveSignal::zero()
    );

    assert!(
        result.action_consequences().selected()[0]
            .evidence()
            .action_change_association()
            > CognitiveSignal::zero()
    );
}

#[test]
fn absent_selected_persistence_prevents_downstream_change_and_action_consequence_resurrection() {
    let previous = frame(1, &[1, 2]);

    let current = frame(2, &[10, 20]);

    let rejected_link = link(&previous, &[2], &current, &[20], 900);

    let rejected_change = change(transition(&rejected_link), 2, 900);

    let rejected_consequence = consequence(1, rejected_change.clone(), 200, 900, 900);

    let world_input = input(
        previous,
        current,
        candidates(
            vec![scene(vec![object(&[1])], 900)],
            vec![scene(vec![object(&[10])], 900)],
            vec![rejected_link],
            Vec::new(),
            vec![rejected_change],
            vec![rejected_consequence],
        ),
    );

    let result = IntegratedPerceptualWorld::evaluate(&world_input, context(4, 1, 4, 4, 4, 4));

    assert_eq!(result.persistence().selected_count(), 0);

    assert_eq!(result.changes().selected_count(), 0);

    assert_eq!(result.action_consequences().selected_count(), 0);

    assert_eq!(result.rejected_change_dependency_count(), 1);

    assert_eq!(result.rejected_action_consequence_dependency_count(), 1);
}

#[test]
fn integrated_perceptual_world_is_deterministic_non_mutating_and_facade_equivalent() {
    let previous = frame(10, &[1, 2]);

    let current = frame(12, &[10, 20]);

    let persistent = link(&previous, &[1], &current, &[10], 900);

    let observed_change = change(transition(&persistent), 1, 900);

    let world_input = input(
        previous,
        current.clone(),
        candidates(
            vec![scene(vec![object(&[1]), object(&[2])], 900)],
            vec![scene(vec![object(&[10]), object(&[20])], 900)],
            vec![persistent],
            vec![topology(
                observation(&current, &[10]),
                observation(&current, &[20]),
                700,
            )],
            vec![observed_change.clone()],
            vec![consequence(10, observed_change, 100, 800, 300)],
        ),
    );

    let before = world_input.clone();

    let world_context = context(4, 2, 4, 4, 4, 4);

    let direct = IntegratedPerceptualWorld::evaluate(&world_input, world_context);

    let facade = CoreKnowledgePerceptualWorld::evaluate(&world_input, world_context);

    let repeated = CoreKnowledgePerceptualWorld::evaluate(&world_input, world_context);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(world_input, before);
}

#[test]
fn complete_dependency_closed_perceptual_chain_survives_end_to_end() {
    let previous = frame(1, &[1, 2]);

    let current = frame(3, &[10, 20]);

    let persistent = link(&previous, &[1], &current, &[10], 900);

    let observed_change = change(transition(&persistent), 500, 900);

    let observed_consequence = consequence(2, observed_change.clone(), 700, 800, 400);

    let world_input = input(
        previous,
        current.clone(),
        candidates(
            vec![scene(vec![object(&[1]), object(&[2])], 900)],
            vec![scene(vec![object(&[10]), object(&[20])], 900)],
            vec![persistent],
            vec![topology(
                observation(&current, &[10]),
                observation(&current, &[20]),
                850,
            )],
            vec![observed_change],
            vec![observed_consequence],
        ),
    );

    let result = CoreKnowledgePerceptualWorld::evaluate(&world_input, context(4, 2, 4, 4, 4, 4));

    assert_eq!(result.previous_scene().selected_count(), 1);

    assert_eq!(result.current_scene().selected_count(), 1);

    assert_eq!(result.persistence().selected_count(), 1);

    assert_eq!(result.topology().selected_count(), 1);

    assert_eq!(result.changes().selected_count(), 1);

    assert_eq!(result.action_consequences().selected_count(), 1);

    assert_eq!(result.rejected_persistence_dependency_count(), 0);

    assert_eq!(result.rejected_topology_dependency_count(), 0);

    assert_eq!(result.rejected_change_dependency_count(), 0);

    assert_eq!(result.rejected_action_consequence_dependency_count(), 0);
}
