use athlesia_mindstone_sparse_cognition::CognitiveStructure;
use athlesia_universal_domain_learning::{
    GroundedRepresentationInsufficiencyDetection, GroundedRepresentationInsufficiencyPolicy,
    GroundedStateSnapshot, GroundedTransformationEpisode, TransitionEffectKind,
};

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(left: u64, right: u64) -> CognitiveStructure {
    CognitiveStructure::ordered(vec![atom(left), atom(right)])
        .expect("ordered structure must be nonempty")
}

fn state(values: &[u64]) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(values.iter().copied().map(atom).collect())
        .expect("state must be nonempty")
}

fn structural_state(facts: Vec<CognitiveStructure>) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts).expect("state must be nonempty")
}

fn episode(action: u64, before: &[u64], after: &[u64]) -> GroundedTransformationEpisode {
    GroundedTransformationEpisode::new(state(before), state(after), atom(action))
}

fn policy(max_evidence: usize, max_conflicts: usize) -> GroundedRepresentationInsufficiencyPolicy {
    GroundedRepresentationInsufficiencyPolicy::new(max_evidence, max_conflicts)
        .expect("policy bounds must be positive")
}

#[test]
fn exact_same_observation_and_action_with_incompatible_effect_detects_insufficiency() {
    let evidence = vec![episode(10, &[1], &[1, 900]), episode(10, &[1], &[1])];

    let result = GroundedRepresentationInsufficiencyDetection::detect(&evidence, policy(8, 8));

    assert!(result.representation_insufficient());
    assert_eq!(result.conflict_count(), 1);

    let conflict = &result.conflicts()[0];

    assert_eq!(conflict.source_state(), &state(&[1]));
    assert_eq!(conflict.transformation(), &atom(10));
    assert_eq!(conflict.effect_kind(), TransitionEffectKind::Added);
    assert_eq!(conflict.effect_fact(), &atom(900));
    assert_eq!(conflict.occurrence_episode_count(), 1);
    assert_eq!(conflict.nonoccurrence_episode_count(), 1);
}

#[test]
fn visible_distinctions_actions_and_missing_opportunity_do_not_false_alias() {
    let different_source = vec![episode(10, &[1], &[1, 900]), episode(10, &[1, 2], &[1, 2])];

    let different_action = vec![episode(10, &[1], &[1, 900]), episode(11, &[1], &[1])];

    let no_addition_opportunity = vec![
        episode(10, &[1, 900], &[1, 900]),
        episode(10, &[1, 900], &[1, 900]),
    ];

    for evidence in [different_source, different_action, no_addition_opportunity] {
        let result = GroundedRepresentationInsufficiencyDetection::detect(&evidence, policy(8, 8));

        assert!(
            !result.representation_insufficient(),
            "observable distinctions must not be collapsed into hidden-state evidence",
        );
    }
}

#[test]
fn ordered_structural_identity_remains_authoritative() {
    let left = ordered(1, 2);
    let right = ordered(2, 1);

    assert_ne!(left, right);

    let evidence = vec![
        GroundedTransformationEpisode::new(
            structural_state(vec![atom(50), left.clone()]),
            structural_state(vec![atom(50), left, atom(900)]),
            atom(10),
        ),
        GroundedTransformationEpisode::new(
            structural_state(vec![atom(50), right.clone()]),
            structural_state(vec![atom(50), right]),
            atom(10),
        ),
    ];

    let result = GroundedRepresentationInsufficiencyDetection::detect(&evidence, policy(8, 8));

    assert!(
        !result.representation_insufficient(),
        "distinct ordered observations must remain distinct observable states",
    );
}

#[test]
fn detection_is_invariant_to_evidence_input_order() {
    let evidence = vec![
        episode(10, &[1], &[1]),
        episode(10, &[1], &[1, 900]),
        episode(20, &[2], &[2]),
        episode(30, &[3], &[3, 700]),
    ];

    let forward = GroundedRepresentationInsufficiencyDetection::detect(&evidence, policy(8, 8));

    let mut reversed = evidence.clone();
    reversed.reverse();

    let backward = GroundedRepresentationInsufficiencyDetection::detect(&reversed, policy(8, 8));

    assert_eq!(forward, backward);
    assert!(forward.representation_insufficient());
}

#[test]
fn hard_evidence_budget_preserves_observed_contrast() {
    let evidence = vec![
        episode(10, &[1], &[1]),
        episode(10, &[1], &[1]),
        episode(10, &[1], &[1]),
        episode(10, &[1], &[1, 900]),
    ];

    let result = GroundedRepresentationInsufficiencyDetection::detect(&evidence, policy(2, 4));

    assert_eq!(
        result.considered_evidence_count(),
        2,
        "hard evidence budget must remain real",
    );

    assert!(
        result.representation_insufficient(),
        "bounded evidence selection must preserve a witnessed outcome contrast",
    );

    assert_eq!(result.conflict_count(), 1);

    let conflict = &result.conflicts()[0];

    assert_eq!(conflict.effect_kind(), TransitionEffectKind::Added);
    assert_eq!(conflict.effect_fact(), &atom(900));
    assert_eq!(conflict.occurrence_episode_count(), 1);
    assert_eq!(conflict.nonoccurrence_episode_count(), 1);
}

#[test]
fn removed_effect_conflict_is_first_class_representation_insufficiency() {
    let evidence = vec![
        episode(10, &[1, 900], &[1]),
        episode(10, &[1, 900], &[1, 900]),
    ];

    let result = GroundedRepresentationInsufficiencyDetection::detect(&evidence, policy(8, 8));

    assert!(result.representation_insufficient());
    assert_eq!(result.conflict_count(), 1);

    let conflict = &result.conflicts()[0];

    assert_eq!(conflict.source_state(), &state(&[1, 900]));
    assert_eq!(conflict.transformation(), &atom(10));
    assert_eq!(conflict.effect_kind(), TransitionEffectKind::Removed);
    assert_eq!(conflict.effect_fact(), &atom(900));
    assert_eq!(conflict.occurrence_episode_count(), 1);
    assert_eq!(conflict.nonoccurrence_episode_count(), 1);
}

#[test]
fn conflict_frontier_is_hard_bounded_deterministic_and_order_invariant() {
    let evidence = vec![
        episode(10, &[1], &[1]),
        episode(10, &[1], &[1, 700]),
        episode(10, &[1], &[1, 900]),
    ];

    let forward = GroundedRepresentationInsufficiencyDetection::detect(&evidence, policy(8, 1));

    let mut reversed = evidence.clone();
    reversed.reverse();

    let backward = GroundedRepresentationInsufficiencyDetection::detect(&reversed, policy(8, 1));

    assert_eq!(
        forward, backward,
        "bounded conflict admission must not depend on evidence input order",
    );

    assert!(forward.representation_insufficient());
    assert_eq!(
        forward.conflict_count(),
        1,
        "max_conflicts must be a real hard frontier",
    );
    assert!(
        forward.frontier_truncated(),
        "result must expose that additional grounded conflicts existed",
    );

    let admitted = &forward.conflicts()[0];

    assert_eq!(admitted.effect_kind(), TransitionEffectKind::Added);
    assert_eq!(
        admitted.effect_fact(),
        &atom(700),
        "exact structural ordering must deterministically break equal-priority ties",
    );
}
