use athlesia_core_knowledge_perceptual_grounding::{
    ActionConsequenceCompetition, ActionConsequenceEvidence, ActionConsequenceHypothesis,
    ActionConsequencePolicy, ActionObservation, ActionSource, ChangeEvidence,
    CoreKnowledgeActionConsequences, ObjectHypothesis, ObjectObservation,
    ObjectTransitionObservation, ObjecthoodEvidence, PerceptualChangeHypothesis,
    PerceptualChangeKind, PerceptualElement, PerceptualElementHandle, PerceptualFrame,
    PersistenceEvidence, PersistenceLinkHypothesis,
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
        signal(600),
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
    observed_transition: ObjectTransitionObservation,
    kind: PerceptualChangeKind,
    descriptor: u64,
    support: u16,
) -> PerceptualChangeHypothesis {
    PerceptualChangeHypothesis::new(
        observed_transition,
        kind,
        None,
        CognitiveStructure::atom(descriptor),
        change_evidence(support),
    )
    .unwrap()
}

fn action(event_index: u64, source: ActionSource, descriptor: u64) -> ActionObservation {
    ActionObservation::new(event_index, source, CognitiveStructure::atom(descriptor))
}

fn evidence(
    temporal: u16,
    association: u16,
    repeatability: u16,
    counterfactual: u16,
    specificity: u16,
    causal_lift: u16,
) -> ActionConsequenceEvidence {
    ActionConsequenceEvidence::new(
        signal(temporal),
        signal(association),
        signal(repeatability),
        signal(counterfactual),
        signal(specificity),
        signal(causal_lift),
    )
}

fn consequence(
    observed_action: ActionObservation,
    observed_change: PerceptualChangeHypothesis,
    descriptor: u64,
    support: ActionConsequenceEvidence,
) -> ActionConsequenceHypothesis {
    ActionConsequenceHypothesis::new(
        observed_action,
        observed_change,
        CognitiveStructure::atom(descriptor),
        support,
    )
    .unwrap()
}

fn policy(per_action: usize, per_change: usize, total: usize) -> ActionConsequencePolicy {
    ActionConsequencePolicy::new(per_action, per_change, total).unwrap()
}

#[test]
fn action_observation_preserves_event_source_and_opaque_descriptor() {
    let observed = action(7, ActionSource::SelfGenerated, 900);

    assert_eq!(observed.event_index(), 7);

    assert_eq!(observed.source(), ActionSource::SelfGenerated);

    assert_eq!(observed.descriptor(), &CognitiveStructure::atom(900,));
}

#[test]
fn consequence_requires_action_inside_transition_window_and_positive_evidence() {
    let first = frame(10, &[1]);

    let third = frame(13, &[10]);

    let observed_change = change(
        transition(&first, &[1], &third, &[10]),
        PerceptualChangeKind::StateTransition,
        1,
        500,
    );

    assert_eq!(
        ActionConsequenceHypothesis::new(
            action(9, ActionSource::SelfGenerated, 1,),
            observed_change.clone(),
            CognitiveStructure::atom(100,),
            evidence(500, 0, 0, 0, 0, 0,),
        ),
        None
    );

    assert_eq!(
        ActionConsequenceHypothesis::new(
            action(13, ActionSource::SelfGenerated, 1,),
            observed_change.clone(),
            CognitiveStructure::atom(100,),
            evidence(500, 0, 0, 0, 0, 0,),
        ),
        None
    );

    assert_eq!(
        ActionConsequenceHypothesis::new(
            action(11, ActionSource::SelfGenerated, 1,),
            observed_change,
            CognitiveStructure::atom(100,),
            evidence(0, 0, 0, 0, 0, 0,),
        ),
        None
    );
}

#[test]
fn no_single_action_consequence_evidence_axis_is_mandatory() {
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

        assert!(support.consequence_score() > CognitiveSignal::zero());
    }
}

#[test]
fn self_generated_and_externally_observed_actions_are_both_representable() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10, 20]);

    let first_change = change(
        transition(&first, &[1], &second, &[10]),
        PerceptualChangeKind::StateTransition,
        10,
        500,
    );

    let second_change = change(
        transition(&first, &[2], &second, &[20]),
        PerceptualChangeKind::Motion,
        20,
        500,
    );

    let support = evidence(700, 500, 0, 0, 0, 0);

    let result = ActionConsequenceCompetition::select(
        &[
            consequence(
                action(1, ActionSource::SelfGenerated, 100),
                first_change,
                1,
                support,
            ),
            consequence(
                action(1, ActionSource::ObservedExternal, 200),
                second_change,
                2,
                support,
            ),
        ],
        policy(4, 4, 8),
    );

    assert_eq!(result.selected_count(), 2);

    assert!(
        result
            .selected()
            .iter()
            .any(|candidate| { candidate.action().source() == ActionSource::SelfGenerated },)
    );

    assert!(
        result
            .selected()
            .iter()
            .any(|candidate| { candidate.action().source() == ActionSource::ObservedExternal },)
    );
}

#[test]
fn multiple_consequence_descriptors_can_coexist_for_same_action_and_change() {
    let first = frame(1, &[1]);

    let second = frame(2, &[10]);

    let observed_change = change(
        transition(&first, &[1], &second, &[10]),
        PerceptualChangeKind::StateTransition,
        1,
        600,
    );

    let observed_action = action(1, ActionSource::SelfGenerated, 500);

    let support = evidence(700, 700, 0, 0, 0, 0);

    let result = ActionConsequenceCompetition::select(
        &[
            consequence(
                observed_action.clone(),
                observed_change.clone(),
                100,
                support,
            ),
            consequence(observed_action, observed_change, 101, support),
        ],
        policy(4, 4, 8),
    );

    assert_eq!(result.canonical_hypothesis_count(), 2);

    assert_eq!(result.selected_count(), 2);
}

#[test]
fn exact_duplicate_consequence_keeps_only_best_evidence_variant() {
    let first = frame(1, &[1]);

    let second = frame(2, &[10]);

    let observed_change = change(
        transition(&first, &[1], &second, &[10]),
        PerceptualChangeKind::Motion,
        1,
        500,
    );

    let observed_action = action(1, ActionSource::SelfGenerated, 500);

    let weak = consequence(
        observed_action.clone(),
        observed_change.clone(),
        100,
        evidence(200, 0, 0, 0, 0, 0),
    );

    let strong = consequence(
        observed_action,
        observed_change,
        100,
        evidence(900, 900, 0, 0, 0, 0),
    );

    let result = ActionConsequenceCompetition::select(&[weak, strong], policy(4, 4, 8));

    assert_eq!(result.input_hypothesis_count(), 2);

    assert_eq!(result.canonical_hypothesis_count(), 1);

    assert_eq!(result.duplicate_hypothesis_count(), 1);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].evidence().peak_support().value(), 900);
}

#[test]
fn one_action_can_have_multiple_competing_perceptual_consequences() {
    let first = frame(1, &[1, 2]);

    let second = frame(2, &[10, 20]);

    let observed_action = action(1, ActionSource::SelfGenerated, 700);

    let result = ActionConsequenceCompetition::select(
        &[
            consequence(
                observed_action.clone(),
                change(
                    transition(&first, &[1], &second, &[10]),
                    PerceptualChangeKind::StateTransition,
                    10,
                    600,
                ),
                1,
                evidence(700, 500, 0, 0, 0, 0),
            ),
            consequence(
                observed_action,
                change(
                    transition(&first, &[2], &second, &[20]),
                    PerceptualChangeKind::Motion,
                    20,
                    600,
                ),
                2,
                evidence(600, 600, 0, 0, 0, 0),
            ),
        ],
        policy(2, 4, 8),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.dropped_by_action_bound_count(), 0);
}

#[test]
fn one_change_can_have_multiple_competing_action_explanations() {
    let first = frame(1, &[1]);

    let second = frame(3, &[10]);

    let observed_change = change(
        transition(&first, &[1], &second, &[10]),
        PerceptualChangeKind::StateTransition,
        10,
        600,
    );

    let result = ActionConsequenceCompetition::select(
        &[
            consequence(
                action(1, ActionSource::SelfGenerated, 100),
                observed_change.clone(),
                1,
                evidence(700, 500, 0, 0, 0, 0),
            ),
            consequence(
                action(2, ActionSource::ObservedExternal, 200),
                observed_change,
                2,
                evidence(600, 600, 0, 0, 0, 0),
            ),
        ],
        policy(4, 2, 8),
    );

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.dropped_by_change_bound_count(), 0);
}

#[test]
fn hard_per_action_bound_retains_only_best_consequences_for_an_action() {
    let first = frame(1, &[1, 2, 3]);

    let second = frame(2, &[10, 20, 30]);

    let observed_action = action(1, ActionSource::SelfGenerated, 500);

    let candidates = [(1, 10, 900), (2, 20, 700), (3, 30, 500)]
        .into_iter()
        .map(|(previous, current, support)| {
            consequence(
                observed_action.clone(),
                change(
                    transition(&first, &[previous], &second, &[current]),
                    PerceptualChangeKind::StateTransition,
                    previous,
                    500,
                ),
                previous,
                evidence(support, support, 0, 0, 0, 0),
            )
        })
        .collect::<Vec<_>>();

    let result = ActionConsequenceCompetition::select(&candidates, policy(2, 4, 8));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.selected()[0].evidence().peak_support().value(), 900);

    assert_eq!(result.selected()[1].evidence().peak_support().value(), 700);

    assert_eq!(result.dropped_by_action_bound_count(), 1);
}

#[test]
fn hard_per_change_bound_retains_only_best_action_explanations() {
    let first = frame(1, &[1]);

    let fourth = frame(4, &[10]);

    let observed_change = change(
        transition(&first, &[1], &fourth, &[10]),
        PerceptualChangeKind::Motion,
        10,
        500,
    );

    let candidates = vec![
        consequence(
            action(1, ActionSource::SelfGenerated, 1),
            observed_change.clone(),
            100,
            evidence(900, 900, 0, 0, 0, 0),
        ),
        consequence(
            action(2, ActionSource::ObservedExternal, 2),
            observed_change.clone(),
            101,
            evidence(700, 700, 0, 0, 0, 0),
        ),
        consequence(
            action(3, ActionSource::SelfGenerated, 3),
            observed_change,
            102,
            evidence(500, 500, 0, 0, 0, 0),
        ),
    ];

    let result = ActionConsequenceCompetition::select(&candidates, policy(4, 2, 8));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.dropped_by_change_bound_count(), 1);

    assert_eq!(result.selected()[0].evidence().peak_support().value(), 900);

    assert_eq!(result.selected()[1].evidence().peak_support().value(), 700);
}

#[test]
fn hard_global_consequence_frontier_retains_only_highest_scoring_hypotheses() {
    let first = frame(1, &[1, 2, 3, 4]);

    let second = frame(2, &[11, 12, 13, 14]);

    let candidates = [(1, 11, 300), (2, 12, 900), (3, 13, 700), (4, 14, 500)]
        .into_iter()
        .map(|(previous, current, support)| {
            consequence(
                action(1, ActionSource::SelfGenerated, previous),
                change(
                    transition(&first, &[previous], &second, &[current]),
                    PerceptualChangeKind::StateTransition,
                    previous,
                    500,
                ),
                previous,
                evidence(support, support, 0, 0, 0, 0),
            )
        })
        .collect::<Vec<_>>();

    let result = ActionConsequenceCompetition::select(&candidates, policy(4, 4, 2));

    assert_eq!(result.selected_count(), 2);

    assert_eq!(result.selected()[0].evidence().peak_support().value(), 900);

    assert_eq!(result.selected()[1].evidence().peak_support().value(), 700);

    assert_eq!(result.dropped_by_global_bound_count(), 2);
}

#[test]
fn action_consequence_competition_is_deterministic_non_mutating_and_facade_equivalent() {
    let first = frame(10, &[1, 2]);

    let second = frame(12, &[10, 20]);

    let candidates = vec![
        consequence(
            action(10, ActionSource::SelfGenerated, 100),
            change(
                transition(&first, &[1], &second, &[10]),
                PerceptualChangeKind::StateTransition,
                1,
                700,
            ),
            1000,
            evidence(800, 700, 500, 400, 300, 200),
        ),
        consequence(
            action(11, ActionSource::ObservedExternal, 200),
            change(
                transition(&first, &[2], &second, &[20]),
                PerceptualChangeKind::Motion,
                2,
                700,
            ),
            2000,
            evidence(700, 800, 500, 400, 300, 200),
        ),
    ];

    let before = candidates.clone();

    let consequence_policy = policy(4, 4, 8);

    let direct = ActionConsequenceCompetition::select(&candidates, consequence_policy);

    let facade = CoreKnowledgeActionConsequences::evaluate(&candidates, consequence_policy);

    let repeated = CoreKnowledgeActionConsequences::evaluate(&candidates, consequence_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(candidates, before);

    assert_eq!(facade.selected_count(), 2);
}
