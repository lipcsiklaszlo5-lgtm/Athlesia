use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    GroundedExecutableModelEvidenceStatus, GroundedExecutableModelFrontier,
    GroundedExecutableModelFrontierPolicy, GroundedExecutableWorldModel,
    GroundedExecutableWorldModelPolicy, GroundedPredictionOutcomeEvidenceEngine,
    GroundedStateSnapshot, GroundedTransformationEpisode, GroundedTransitionSchemaHypothesis,
    TransitionEffectKind, TransitionSchemaPolicy, UniversalGroundedPredictionOutcomeEvidence,
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
        CognitiveSignal::new(1).expect("positive association lift is valid"),
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
        .expect("added schema must be induced")
}

fn removed_schema(
    transformation: u64,
    contrast_transformation: u64,
    fact: u64,
) -> GroundedTransitionSchemaHypothesis {
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
        .expect("removed schema must be induced")
}

fn model(schemas: Vec<GroundedTransitionSchemaHypothesis>) -> GroundedExecutableWorldModel {
    GroundedExecutableWorldModel::build(
        &schemas,
        GroundedExecutableWorldModelPolicy::new(64).expect("positive model frontier is valid"),
    )
}

fn frontier(models: Vec<GroundedExecutableWorldModel>) -> GroundedExecutableModelFrontier {
    GroundedExecutableModelFrontier::build(
        &models,
        GroundedExecutableModelFrontierPolicy::new(32)
            .expect("positive competing frontier is valid"),
    )
}

#[test]
fn predicted_addition_is_supported_when_fact_is_observed_after() {
    let models = frontier(vec![model(vec![added_schema(10, 11, 2)])]);

    let result = GroundedPredictionOutcomeEvidenceEngine::evaluate(
        &state(&[1]),
        &state(&[1, 2]),
        &atom(10),
        &models,
    );

    let assessment = result.assessment_at(0).expect("model exists");

    assert_eq!(
        assessment.status(),
        GroundedExecutableModelEvidenceStatus::Supported
    );
    assert_eq!(assessment.supported_effect_count(), 1);
    assert_eq!(assessment.counterexample_effect_count(), 0);
}

#[test]
fn predicted_addition_is_counterevidenced_when_fact_remains_absent() {
    let models = frontier(vec![model(vec![added_schema(10, 11, 2)])]);

    let result = GroundedPredictionOutcomeEvidenceEngine::evaluate(
        &state(&[1]),
        &state(&[1]),
        &atom(10),
        &models,
    );

    let assessment = result.assessment_at(0).expect("model exists");

    assert_eq!(
        assessment.status(),
        GroundedExecutableModelEvidenceStatus::Counterevidenced
    );
    assert_eq!(assessment.supported_effect_count(), 0);
    assert_eq!(assessment.counterexample_effect_count(), 1);
}

#[test]
fn predicted_removal_uses_existing_m47_after_state_semantics() {
    let models = frontier(vec![model(vec![removed_schema(10, 11, 2)])]);

    let supported = GroundedPredictionOutcomeEvidenceEngine::evaluate(
        &state(&[1, 2]),
        &state(&[1]),
        &atom(10),
        &models,
    );

    let contradicted = GroundedPredictionOutcomeEvidenceEngine::evaluate(
        &state(&[1, 2]),
        &state(&[1, 2]),
        &atom(10),
        &models,
    );

    assert_eq!(
        supported.assessment_at(0).expect("model exists").status(),
        GroundedExecutableModelEvidenceStatus::Supported
    );

    assert_eq!(
        contradicted
            .assessment_at(0)
            .expect("model exists")
            .status(),
        GroundedExecutableModelEvidenceStatus::Counterevidenced
    );
}

#[test]
fn partial_success_and_failure_remain_explicit_mixed_evidence() {
    let models = frontier(vec![model(vec![
        added_schema(10, 11, 2),
        added_schema(10, 11, 3),
    ])]);

    let result = GroundedPredictionOutcomeEvidenceEngine::evaluate(
        &state(&[1]),
        &state(&[1, 2]),
        &atom(10),
        &models,
    );

    let assessment = result.assessment_at(0).expect("model exists");

    assert_eq!(
        assessment.status(),
        GroundedExecutableModelEvidenceStatus::MixedEvidence
    );
    assert_eq!(assessment.supported_effect_count(), 1);
    assert_eq!(assessment.counterexample_effect_count(), 1);
    assert_eq!(assessment.judged_effect_count(), 2);
}

#[test]
fn predictive_silence_remains_unresolved_and_creates_no_fake_evidence() {
    let models = frontier(vec![model(vec![added_schema(20, 21, 2)])]);

    let result = GroundedPredictionOutcomeEvidenceEngine::evaluate(
        &state(&[1]),
        &state(&[1, 99]),
        &atom(10),
        &models,
    );

    let assessment = result.assessment_at(0).expect("model exists");

    assert_eq!(
        assessment.status(),
        GroundedExecutableModelEvidenceStatus::Unresolved
    );
    assert_eq!(assessment.judged_effect_count(), 0);
    assert!(!assessment.has_support());
    assert!(!assessment.has_counterevidence());
}

#[test]
fn one_observation_records_evidence_without_mutating_or_eliminating_models() {
    let supported_model = model(vec![added_schema(10, 11, 2)]);

    let counter_model = model(vec![added_schema(10, 11, 3)]);

    let models = frontier(vec![supported_model.clone(), counter_model.clone()]);

    let original_frontier = models.clone();

    let result = GroundedPredictionOutcomeEvidenceEngine::evaluate(
        &state(&[1]),
        &state(&[1, 2]),
        &atom(10),
        &models,
    );

    assert_eq!(models, original_frontier);
    assert_eq!(result.model_count(), 2);
    assert_eq!(result.supported_model_count(), 1);
    assert_eq!(result.counterevidenced_model_count(), 1);

    assert!(result
        .assessments()
        .iter()
        .any(|assessment| assessment.model() == &supported_model));

    assert!(result
        .assessments()
        .iter()
        .any(|assessment| assessment.model() == &counter_model));
}

#[test]
fn universal_facade_matches_direct_evidence_evaluation() {
    let models = frontier(vec![
        model(vec![added_schema(10, 11, 2)]),
        model(vec![added_schema(10, 11, 3)]),
    ]);

    let before = state(&[1]);
    let after = state(&[1, 2]);
    let transformation = atom(10);

    let direct = GroundedPredictionOutcomeEvidenceEngine::evaluate(
        &before,
        &after,
        &transformation,
        &models,
    );

    let universal = UniversalGroundedPredictionOutcomeEvidence::evaluate(
        &before,
        &after,
        &transformation,
        &models,
    );

    assert_eq!(direct, universal);
}
