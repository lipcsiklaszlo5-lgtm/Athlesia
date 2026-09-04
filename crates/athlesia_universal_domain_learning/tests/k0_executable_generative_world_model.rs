use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    GroundedExecutableWorldModel, GroundedExecutableWorldModelPolicy, GroundedStateSnapshot,
    GroundedStructuralPredictionStatus, GroundedTransformationEpisode, TransitionEffectKind,
    TransitionSchemaPolicy, UniversalGroundedExecutableWorldModel,
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
        CognitiveSignal::new(1000).expect("maximum precision is valid"),
        CognitiveSignal::new(1).expect("positive association lift is valid"),
        64,
        64,
    )
    .expect("test schema policy is valid")
}

fn added_schema(
    transformation: u64,
    contrast_transformation: u64,
    fact: u64,
) -> athlesia_universal_domain_learning::GroundedTransitionSchemaHypothesis {
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

fn removed_schema(
    transformation: u64,
    contrast_transformation: u64,
    fact: u64,
) -> athlesia_universal_domain_learning::GroundedTransitionSchemaHypothesis {
    let episodes = vec![
        GroundedTransformationEpisode::new(state(&[1, fact]), state(&[1]), atom(transformation)),
        GroundedTransformationEpisode::new(
            state(&[1, fact]),
            state(&[1, fact]),
            atom(contrast_transformation),
        ),
    ];

    UniversalTransitionSchemaInduction::evaluate(&episodes, &[], schema_policy())
        .selected()
        .iter()
        .find(|schema| {
            schema.transformation() == &atom(transformation)
                && schema.effect_kind() == TransitionEffectKind::Removed
                && schema.fact() == &atom(fact)
        })
        .cloned()
        .expect("target removed schema must be induced")
}

fn model(
    schemas: Vec<athlesia_universal_domain_learning::GroundedTransitionSchemaHypothesis>,
) -> GroundedExecutableWorldModel {
    GroundedExecutableWorldModel::build(
        &schemas,
        GroundedExecutableWorldModelPolicy::new(16).expect("positive frontier is valid"),
    )
}

#[test]
fn learned_added_effect_is_executable_as_partial_prediction() {
    let learned = added_schema(10, 11, 2);
    let model = model(vec![learned]);

    let prediction = model.predict(&state(&[1]), &atom(10));

    assert_eq!(
        prediction.status(),
        GroundedStructuralPredictionStatus::Predicted
    );
    assert_eq!(prediction.additions(), &[atom(2)]);
    assert!(prediction.removals().is_empty());
    assert!(prediction.predicts_addition(&atom(2)));
    assert_eq!(prediction.applicable_schema_count(), 1);
}

#[test]
fn learned_removed_effect_is_executable_as_partial_prediction() {
    let learned = removed_schema(10, 11, 2);
    let model = model(vec![learned]);

    let prediction = model.predict(&state(&[1, 2]), &atom(10));

    assert_eq!(
        prediction.status(),
        GroundedStructuralPredictionStatus::Predicted
    );
    assert_eq!(prediction.removals(), &[atom(2)]);
    assert!(prediction.additions().is_empty());
    assert!(prediction.predicts_removal(&atom(2)));
}

#[test]
fn exact_action_identity_is_required() {
    let learned = added_schema(10, 11, 2);
    let model = model(vec![learned]);

    let prediction = model.predict(&state(&[1]), &atom(999));

    assert_eq!(
        prediction.status(),
        GroundedStructuralPredictionStatus::NoApplicableEffect
    );
    assert!(prediction.additions().is_empty());
    assert!(prediction.removals().is_empty());
}

#[test]
fn addition_requires_real_source_state_opportunity() {
    let learned = added_schema(10, 11, 2);
    let model = model(vec![learned]);

    let prediction = model.predict(&state(&[1, 2]), &atom(10));

    assert_eq!(
        prediction.status(),
        GroundedStructuralPredictionStatus::NoApplicableEffect
    );
}

#[test]
fn removal_requires_real_source_state_opportunity() {
    let learned = removed_schema(10, 11, 2);
    let model = model(vec![learned]);

    let prediction = model.predict(&state(&[1]), &atom(10));

    assert_eq!(
        prediction.status(),
        GroundedStructuralPredictionStatus::NoApplicableEffect
    );
}

#[test]
fn add_and_remove_schemas_for_same_fact_form_state_conditioned_toggle_not_false_conflict() {
    let add = added_schema(10, 11, 2);
    let remove = removed_schema(10, 11, 2);
    let model = model(vec![add, remove]);

    let absent_prediction = model.predict(&state(&[1]), &atom(10));
    let present_prediction = model.predict(&state(&[1, 2]), &atom(10));

    assert_eq!(
        absent_prediction.status(),
        GroundedStructuralPredictionStatus::Predicted
    );
    assert_eq!(absent_prediction.additions(), &[atom(2)]);
    assert!(absent_prediction.removals().is_empty());

    assert_eq!(
        present_prediction.status(),
        GroundedStructuralPredictionStatus::Predicted
    );
    assert_eq!(present_prediction.removals(), &[atom(2)]);
    assert!(present_prediction.additions().is_empty());
}

#[test]
fn multiple_independent_learned_effects_compose_into_one_partial_delta() {
    let first = added_schema(10, 11, 2);
    let second = added_schema(10, 11, 3);
    let model = model(vec![second, first]);

    let prediction = model.predict(&state(&[1]), &atom(10));

    assert_eq!(
        prediction.status(),
        GroundedStructuralPredictionStatus::Predicted
    );
    assert_eq!(prediction.additions(), &[atom(2), atom(3)]);
    assert_eq!(prediction.applicable_schema_count(), 2);
}

#[test]
fn frontier_is_bounded_and_deterministic() {
    let a = added_schema(10, 11, 2);
    let b = added_schema(20, 21, 3);

    let left = GroundedExecutableWorldModel::build(
        &[a.clone(), b.clone()],
        GroundedExecutableWorldModelPolicy::new(1).expect("positive frontier is valid"),
    );

    let right = GroundedExecutableWorldModel::build(
        &[b, a],
        GroundedExecutableWorldModelPolicy::new(1).expect("positive frontier is valid"),
    );

    assert_eq!(left, right);
    assert_eq!(left.schema_count(), 1);
    assert_eq!(left.admitted_before_frontier(), 2);
    assert!(left.frontier_truncated());
}

#[test]
fn universal_facade_matches_direct_execution() {
    let learned = added_schema(10, 11, 2);

    let policy = GroundedExecutableWorldModelPolicy::new(16).expect("positive frontier is valid");

    let direct_model = GroundedExecutableWorldModel::build(std::slice::from_ref(&learned), policy);

    let universal_model = UniversalGroundedExecutableWorldModel::build(&[learned], policy);

    assert_eq!(direct_model, universal_model);

    let source = state(&[1]);
    let action = atom(10);

    assert_eq!(
        direct_model.predict(&source, &action),
        UniversalGroundedExecutableWorldModel::predict(&source, &action, &universal_model,)
    );
}
