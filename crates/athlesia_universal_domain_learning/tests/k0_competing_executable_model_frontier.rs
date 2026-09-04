use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    GroundedExecutableModelFrontier, GroundedExecutableModelFrontierPolicy,
    GroundedExecutableWorldModel, GroundedExecutableWorldModelPolicy, GroundedStateSnapshot,
    GroundedStructuralPredictionStatus, GroundedTransformationEpisode,
    GroundedTransitionSchemaHypothesis, TransitionEffectKind, TransitionSchemaPolicy,
    UniversalGroundedExecutableModelFrontier, UniversalTransitionSchemaInduction,
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
        64,
        64,
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
        .expect("target schema must be induced")
}

fn model(schemas: Vec<GroundedTransitionSchemaHypothesis>) -> GroundedExecutableWorldModel {
    GroundedExecutableWorldModel::build(
        &schemas,
        GroundedExecutableWorldModelPolicy::new(32).expect("positive model frontier is valid"),
    )
}

fn frontier(models: Vec<GroundedExecutableWorldModel>) -> GroundedExecutableModelFrontier {
    GroundedExecutableModelFrontier::build(
        &models,
        GroundedExecutableModelFrontierPolicy::new(16)
            .expect("positive competing frontier is valid"),
    )
}

#[test]
fn distinct_models_expose_structural_prediction_disagreement() {
    let left = model(vec![added_schema(10, 11, 2)]);

    let right = model(vec![added_schema(10, 11, 3)]);

    let frontier = frontier(vec![left, right]);

    let result = frontier.evaluate(&state(&[1]), &atom(10));

    assert_eq!(result.model_count(), 2);
    assert_eq!(result.predicted_model_count(), 2);
    assert!(result.has_disagreement());
    assert_eq!(result.disputed_fact_count(), 2);

    let fact_two = result
        .disagreement_for(&atom(2))
        .expect("fact two must be disputed");

    assert_eq!(fact_two.added_model_count(), 1);
    assert_eq!(fact_two.removed_model_count(), 0);
    assert_eq!(fact_two.unknown_model_count(), 1);
    assert_eq!(fact_two.conflict_model_count(), 0);
    assert_eq!(fact_two.disposition_count(), 2);

    let fact_three = result
        .disagreement_for(&atom(3))
        .expect("fact three must be disputed");

    assert_eq!(fact_three.added_model_count(), 1);
    assert_eq!(fact_three.unknown_model_count(), 1);
}

#[test]
fn predictive_silence_is_preserved_as_epistemic_disagreement() {
    let predictive = model(vec![added_schema(10, 11, 2)]);

    let silent_for_action = model(vec![added_schema(20, 21, 3)]);

    let frontier = frontier(vec![predictive, silent_for_action]);

    let result = frontier.evaluate(&state(&[1]), &atom(10));

    assert_eq!(result.predicted_model_count(), 1);
    assert_eq!(result.no_applicable_effect_model_count(), 1);
    assert!(result.has_disagreement());

    let disagreement = result
        .disagreement_for(&atom(2))
        .expect("predicted versus unknown must disagree");

    assert_eq!(disagreement.added_model_count(), 1);
    assert_eq!(disagreement.unknown_model_count(), 1);
}

#[test]
fn different_models_that_make_same_current_prediction_do_not_fake_disagreement() {
    let shared = added_schema(10, 11, 2);

    let left = model(vec![shared.clone(), added_schema(20, 21, 3)]);

    let right = model(vec![shared, added_schema(30, 31, 4)]);

    let frontier = frontier(vec![left, right]);

    let result = frontier.evaluate(&state(&[1]), &atom(10));

    assert_eq!(result.model_count(), 2);
    assert_eq!(result.predicted_model_count(), 2);
    assert!(!result.has_disagreement());
    assert_eq!(result.disputed_fact_count(), 0);

    for prediction in result.model_predictions() {
        assert_eq!(
            prediction.status(),
            GroundedStructuralPredictionStatus::Predicted
        );
        assert_eq!(prediction.additions(), &[atom(2)]);
    }
}

#[test]
fn exact_duplicate_predictive_models_do_not_multiply_authority() {
    let schema = added_schema(10, 11, 2);

    let first = model(vec![schema.clone()]);
    let second = model(vec![schema]);

    let frontier = frontier(vec![first, second]);

    assert_eq!(frontier.model_count(), 1);
    assert_eq!(frontier.admitted_before_frontier(), 1);
    assert!(!frontier.frontier_truncated());
}

#[test]
fn model_frontier_is_bounded_and_input_order_invariant() {
    let first = model(vec![added_schema(10, 11, 2)]);

    let second = model(vec![added_schema(10, 11, 3)]);

    let third = model(vec![added_schema(10, 11, 4)]);

    let policy = GroundedExecutableModelFrontierPolicy::new(2).expect("positive frontier is valid");

    let left = GroundedExecutableModelFrontier::build(
        &[first.clone(), second.clone(), third.clone()],
        policy,
    );

    let right = GroundedExecutableModelFrontier::build(&[third, second, first], policy);

    assert_eq!(left, right);
    assert_eq!(left.model_count(), 2);
    assert_eq!(left.admitted_before_frontier(), 3);
    assert!(left.frontier_truncated());
}

#[test]
fn unrelated_action_with_unanimous_silence_has_no_structural_disagreement() {
    let first = model(vec![added_schema(10, 11, 2)]);

    let second = model(vec![added_schema(20, 21, 3)]);

    let frontier = frontier(vec![first, second]);

    let result = frontier.evaluate(&state(&[1]), &atom(999));

    assert_eq!(result.predicted_model_count(), 0);
    assert_eq!(result.no_applicable_effect_model_count(), 2);
    assert_eq!(result.conflict_model_count(), 0);
    assert!(!result.has_disagreement());
}

#[test]
fn disagreement_is_exact_structure_sensitive() {
    let ordered =
        CognitiveStructure::ordered(vec![atom(8), atom(9)]).expect("ordered structure is nonempty");

    let reversed =
        CognitiveStructure::ordered(vec![atom(9), atom(8)]).expect("ordered structure is nonempty");

    let episodes_left = vec![
        GroundedTransformationEpisode::new(
            state(&[1]),
            GroundedStateSnapshot::new(vec![atom(1), ordered.clone()]).expect("state is nonempty"),
            atom(10),
        ),
        GroundedTransformationEpisode::new(state(&[1]), state(&[1]), atom(11)),
    ];

    let episodes_right = vec![
        GroundedTransformationEpisode::new(
            state(&[1]),
            GroundedStateSnapshot::new(vec![atom(1), reversed.clone()]).expect("state is nonempty"),
            atom(10),
        ),
        GroundedTransformationEpisode::new(state(&[1]), state(&[1]), atom(11)),
    ];

    let left_schema =
        UniversalTransitionSchemaInduction::evaluate(&episodes_left, &[], schema_policy())
            .selected()
            .iter()
            .find(|schema| {
                schema.transformation() == &atom(10)
                    && schema.effect_kind() == TransitionEffectKind::Added
                    && schema.fact() == &ordered
            })
            .cloned()
            .expect("ordered schema must be induced");

    let right_schema =
        UniversalTransitionSchemaInduction::evaluate(&episodes_right, &[], schema_policy())
            .selected()
            .iter()
            .find(|schema| {
                schema.transformation() == &atom(10)
                    && schema.effect_kind() == TransitionEffectKind::Added
                    && schema.fact() == &reversed
            })
            .cloned()
            .expect("reversed schema must be induced");

    let frontier = frontier(vec![model(vec![left_schema]), model(vec![right_schema])]);

    let result = frontier.evaluate(&state(&[1]), &atom(10));

    assert_eq!(result.disputed_fact_count(), 2);
    assert!(result.disagreement_for(&ordered).is_some());
    assert!(result.disagreement_for(&reversed).is_some());
}

#[test]
fn universal_facade_matches_direct_frontier_evaluation() {
    let first = model(vec![added_schema(10, 11, 2)]);

    let second = model(vec![added_schema(10, 11, 3)]);

    let models = vec![first, second];

    let policy = GroundedExecutableModelFrontierPolicy::new(8).expect("positive frontier is valid");

    let direct = GroundedExecutableModelFrontier::build(&models, policy);

    let universal = UniversalGroundedExecutableModelFrontier::build(&models, policy);

    assert_eq!(direct, universal);

    let source = state(&[1]);
    let transformation = atom(10);

    assert_eq!(
        direct.evaluate(&source, &transformation),
        UniversalGroundedExecutableModelFrontier::evaluate(&source, &transformation, &universal,)
    );
}
