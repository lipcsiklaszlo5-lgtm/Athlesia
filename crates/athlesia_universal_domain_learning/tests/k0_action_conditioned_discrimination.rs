use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    GroundedActionDiscriminationEngine, GroundedActionDiscriminationPolicy,
    GroundedExecutableModelFrontier, GroundedExecutableModelFrontierPolicy,
    GroundedExecutableWorldModel, GroundedExecutableWorldModelPolicy, GroundedStateSnapshot,
    GroundedTransformationEpisode, GroundedTransitionSchemaHypothesis, TransitionEffectKind,
    TransitionSchemaPolicy, UniversalGroundedActionDiscrimination,
    UniversalTransitionSchemaInduction,
};

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn state(facts: &[u64]) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts.iter().copied().map(atom).collect())
        .expect("test state must be nonempty")
}

fn schema_policy() -> TransitionSchemaPolicy {
    TransitionSchemaPolicy::new(
        1,
        CognitiveSignal::maximum(),
        CognitiveSignal::new(1).expect("positive lift is valid"),
        128,
        128,
    )
    .expect("schema policy must be valid")
}

fn added_schema(
    transformation: u64,
    contrast_transformation: u64,
    fact: u64,
) -> GroundedTransitionSchemaHypothesis {
    let episodes = vec![
        GroundedTransformationEpisode::new(state(&[1]), state(&[1, fact]), atom(transformation)),
        GroundedTransformationEpisode::new(state(&[1]), state(&[1]), atom(contrast_transformation)),
    ];

    UniversalTransitionSchemaInduction::evaluate(&episodes, &[], schema_policy())
        .selected()
        .iter()
        .find(|schema| {
            schema.transformation() == &atom(transformation)
                && schema.effect_kind() == TransitionEffectKind::Added
                && schema.fact() == &atom(fact)
        })
        .cloned()
        .expect("target added schema must be induced")
}

fn model(schemas: Vec<GroundedTransitionSchemaHypothesis>) -> GroundedExecutableWorldModel {
    GroundedExecutableWorldModel::build(
        &schemas,
        GroundedExecutableWorldModelPolicy::new(64)
            .expect("positive world-model frontier is valid"),
    )
}

fn competing_frontier(
    models: Vec<GroundedExecutableWorldModel>,
) -> GroundedExecutableModelFrontier {
    GroundedExecutableModelFrontier::build(
        &models,
        GroundedExecutableModelFrontierPolicy::new(32)
            .expect("positive competing frontier is valid"),
    )
}

fn discriminative_models() -> GroundedExecutableModelFrontier {
    let model_one = model(vec![
        added_schema(10, 90, 2),
        added_schema(20, 91, 3),
        added_schema(30, 92, 101),
    ]);

    let model_two = model(vec![
        added_schema(10, 90, 2),
        added_schema(20, 91, 3),
        added_schema(31, 93, 102),
    ]);

    let model_three = model(vec![added_schema(20, 91, 3), added_schema(32, 94, 103)]);

    let model_four = model(vec![added_schema(33, 95, 104)]);

    competing_frontier(vec![model_one, model_two, model_three, model_four])
}

#[test]
fn balanced_model_split_beats_more_lopsided_split() {
    let models = discriminative_models();

    let result = GroundedActionDiscriminationEngine::evaluate(
        &state(&[1]),
        &[atom(10), atom(20)],
        &models,
        GroundedActionDiscriminationPolicy::new(8).expect("positive action bound is valid"),
    );

    let action_ten = result
        .evaluation_for(&atom(10))
        .expect("action ten must be evaluated");

    let action_twenty = result
        .evaluation_for(&atom(20))
        .expect("action twenty must be evaluated");

    assert_eq!(action_ten.pairwise_separation_score(), 4);
    assert_eq!(action_twenty.pairwise_separation_score(), 3);

    assert_eq!(
        result
            .best_informative_action()
            .expect("an informative action must exist")
            .transformation(),
        &atom(10)
    );
}

#[test]
fn candidate_action_order_and_exact_duplicates_do_not_change_result() {
    let models = discriminative_models();

    let policy =
        GroundedActionDiscriminationPolicy::new(8).expect("positive action bound is valid");

    let left = GroundedActionDiscriminationEngine::evaluate(
        &state(&[1]),
        &[atom(20), atom(10), atom(20), atom(10)],
        &models,
        policy,
    );

    let right = GroundedActionDiscriminationEngine::evaluate(
        &state(&[1]),
        &[atom(10), atom(20)],
        &models,
        policy,
    );

    assert_eq!(left, right);
    assert_eq!(left.admitted_action_count(), 2);
    assert_eq!(left.evaluated_action_count(), 2);
}

#[test]
fn action_evaluation_frontier_is_hard_bounded_and_deterministic() {
    let models = discriminative_models();

    let policy =
        GroundedActionDiscriminationPolicy::new(2).expect("positive action bound is valid");

    let left = GroundedActionDiscriminationEngine::evaluate(
        &state(&[1]),
        &[atom(30), atom(10), atom(20)],
        &models,
        policy,
    );

    let right = GroundedActionDiscriminationEngine::evaluate(
        &state(&[1]),
        &[atom(20), atom(30), atom(10)],
        &models,
        policy,
    );

    assert_eq!(left, right);
    assert_eq!(left.admitted_action_count(), 3);
    assert_eq!(left.evaluated_action_count(), 2);
    assert!(left.action_evaluation_truncated());

    assert!(left.evaluation_for(&atom(10)).is_some());
    assert!(left.evaluation_for(&atom(20)).is_some());
    assert!(left.evaluation_for(&atom(30)).is_none());
}

#[test]
fn unanimous_ignorance_does_not_create_fake_informative_action() {
    let models = discriminative_models();

    let result = GroundedActionDiscriminationEngine::evaluate(
        &state(&[1]),
        &[atom(999)],
        &models,
        GroundedActionDiscriminationPolicy::new(4).expect("positive action bound is valid"),
    );

    let evaluation = result
        .evaluation_for(&atom(999))
        .expect("candidate action must still be evaluated");

    assert_eq!(evaluation.pairwise_separation_score(), 0);
    assert!(!evaluation.is_informative());
    assert!(result.best_informative_action().is_none());
}

#[test]
fn exact_structural_action_identity_remains_authoritative() {
    let ordered =
        CognitiveStructure::ordered(vec![atom(7), atom(8)]).expect("ordered action is nonempty");

    let reversed =
        CognitiveStructure::ordered(vec![atom(8), atom(7)]).expect("reversed action is nonempty");

    let ordered_contrast = CognitiveStructure::ordered(vec![atom(70), atom(80)])
        .expect("ordered contrast is nonempty");

    let reversed_contrast = CognitiveStructure::ordered(vec![atom(80), atom(70)])
        .expect("reversed contrast is nonempty");

    let ordered_episodes = vec![
        GroundedTransformationEpisode::new(state(&[1]), state(&[1, 2]), ordered.clone()),
        GroundedTransformationEpisode::new(state(&[1]), state(&[1]), ordered_contrast),
    ];

    let reversed_episodes = vec![
        GroundedTransformationEpisode::new(state(&[1]), state(&[1, 3]), reversed.clone()),
        GroundedTransformationEpisode::new(state(&[1]), state(&[1]), reversed_contrast),
    ];

    let ordered_schema =
        UniversalTransitionSchemaInduction::evaluate(&ordered_episodes, &[], schema_policy())
            .selected()
            .iter()
            .find(|schema| {
                schema.transformation() == &ordered
                    && schema.effect_kind() == TransitionEffectKind::Added
                    && schema.fact() == &atom(2)
            })
            .cloned()
            .expect("ordered schema must be induced");

    let reversed_schema =
        UniversalTransitionSchemaInduction::evaluate(&reversed_episodes, &[], schema_policy())
            .selected()
            .iter()
            .find(|schema| {
                schema.transformation() == &reversed
                    && schema.effect_kind() == TransitionEffectKind::Added
                    && schema.fact() == &atom(3)
            })
            .cloned()
            .expect("reversed schema must be induced");

    let models = competing_frontier(vec![
        model(vec![ordered_schema]),
        model(vec![reversed_schema]),
    ]);

    let result = GroundedActionDiscriminationEngine::evaluate(
        &state(&[1]),
        &[reversed.clone(), ordered.clone()],
        &models,
        GroundedActionDiscriminationPolicy::new(4).expect("positive action bound is valid"),
    );

    assert!(result.evaluation_for(&ordered).is_some());
    assert!(result.evaluation_for(&reversed).is_some());
    assert_ne!(ordered, reversed);
}

#[test]
fn universal_facade_matches_direct_discrimination() {
    let models = discriminative_models();

    let state = state(&[1]);
    let actions = vec![atom(20), atom(10)];

    let policy =
        GroundedActionDiscriminationPolicy::new(8).expect("positive action bound is valid");

    let direct = GroundedActionDiscriminationEngine::evaluate(&state, &actions, &models, policy);

    let universal =
        UniversalGroundedActionDiscrimination::evaluate(&state, &actions, &models, policy);

    assert_eq!(direct, universal);
}
