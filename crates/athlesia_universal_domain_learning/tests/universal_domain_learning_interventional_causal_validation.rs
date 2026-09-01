use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    CalibratedRuleConfidence, CausalContrastInduction, CausalContrastPolicy,
    CausalContrastThresholds, ContextualTransitionEvidenceThresholds,
    ContextualTransitionRuleInduction, ContextualTransitionRulePolicy, CrossContextGeneralization,
    CrossContextGeneralizationPolicy, CrossContextGeneralizationThresholds,
    GroundedCausalContrastHypothesis, GroundedStateSnapshot, GroundedTransformationEpisode,
    InterventionEvidenceKind, InterventionalCausalThresholds, InterventionalCausalValidation,
    InterventionalCausalValidationPolicy, InterventionalTransformationEpisode,
    RuleConfidenceCalibration, RuleConfidenceCalibrationPolicy, TransitionEffectKind,
    UniversalInterventionalCausalValidation,
};

#[derive(Clone, Copy)]
struct ValidationBounds {
    max_seeds: usize,
    max_evaluations: usize,
    max_validated: usize,
    full_support: u64,
}

#[derive(Clone, Copy)]
struct ValidationEvidence {
    matched_states: usize,
    target_interventions: u64,
    contrast_interventions: u64,
    lift: u16,
    confidence: u16,
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(values.iter().copied().map(atom).collect()).unwrap()
}

fn snapshot(facts: &[u64]) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts.iter().copied().map(atom).collect()).unwrap()
}

fn transition(
    before: &[u64],
    after: &[u64],
    transformation: CognitiveStructure,
) -> GroundedTransformationEpisode {
    GroundedTransformationEpisode::new(snapshot(before), snapshot(after), transformation)
}

fn controlled(
    before: &[u64],
    after: &[u64],
    transformation: CognitiveStructure,
) -> InterventionalTransformationEpisode {
    InterventionalTransformationEpisode::controlled(transition(before, after, transformation))
}

fn observed(
    before: &[u64],
    after: &[u64],
    transformation: CognitiveStructure,
) -> InterventionalTransformationEpisode {
    InterventionalTransformationEpisode::observed(transition(before, after, transformation))
}

fn validation_policy(
    bounds: ValidationBounds,
    evidence: ValidationEvidence,
) -> InterventionalCausalValidationPolicy {
    let thresholds = InterventionalCausalThresholds::new(
        evidence.matched_states,
        evidence.target_interventions,
        evidence.contrast_interventions,
        signal(evidence.lift),
        signal(evidence.confidence),
    )
    .unwrap();

    InterventionalCausalValidationPolicy::new(
        bounds.max_seeds,
        bounds.max_evaluations,
        bounds.max_validated,
        bounds.full_support,
        thresholds,
    )
    .unwrap()
}

fn default_policy() -> InterventionalCausalValidationPolicy {
    validation_policy(
        ValidationBounds {
            max_seeds: 64,
            max_evaluations: 64,
            max_validated: 32,
            full_support: 4,
        },
        ValidationEvidence {
            matched_states: 2,
            target_interventions: 2,
            contrast_interventions: 2,
            lift: 400,
            confidence: 100,
        },
    )
}

fn training_history(transformation: CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
    vec![
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[1, 6, 9], &[1, 6, 9], transformation.clone()),
        transition(&[1, 6, 9], &[1, 6, 9], transformation),
    ]
}

fn calibrated_seed(transformation: CognitiveStructure) -> CalibratedRuleConfidence {
    let training = training_history(transformation.clone());

    let contextual_thresholds =
        ContextualTransitionEvidenceThresholds::new(2, signal(900), signal(300), signal(300))
            .unwrap();

    let contextual_policy =
        ContextualTransitionRulePolicy::new(2, 256, 8192, 256, contextual_thresholds).unwrap();

    let contextual = ContextualTransitionRuleInduction::induce(&training, &[], contextual_policy);

    let generalization_thresholds =
        CrossContextGeneralizationThresholds::new(2, 2, signal(600), signal(200)).unwrap();

    let generalization_policy =
        CrossContextGeneralizationPolicy::new(256, 2, 256, 128, generalization_thresholds).unwrap();

    let generalizations = CrossContextGeneralization::generalize(
        &training,
        contextual.selected(),
        generalization_policy,
    );

    let calibration_policy =
        RuleConfidenceCalibrationPolicy::new(2, 4, signal(1), 128, 128, 64).unwrap();

    RuleConfidenceCalibration::calibrate(
        &training,
        generalizations.selected(),
        &[],
        calibration_policy,
    )
    .selected()
    .iter()
    .find(|seed| {
        seed.transformation() == &transformation
            && seed.effect_kind() == TransitionEffectKind::Added
            && seed.effect_fact() == &atom(5)
    })
    .unwrap()
    .clone()
}

fn contrast_training(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
) -> Vec<GroundedTransformationEpisode> {
    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9], [1, 8, 9]];

    let mut episodes = Vec::new();

    for state in states {
        let mut target_after = state.to_vec();

        target_after.push(5);

        episodes.push(transition(&state, &target_after, target.clone()));

        episodes.push(transition(&state, &state, contrast.clone()));
    }

    episodes
}

fn contrast_seed(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
) -> GroundedCausalContrastHypothesis {
    let calibrated = calibrated_seed(target.clone());

    let episodes = contrast_training(target, contrast);

    let thresholds = CausalContrastThresholds::new(2, 2, 2, signal(400), signal(100)).unwrap();

    let policy = CausalContrastPolicy::new(16, 16, 32, 16, thresholds).unwrap();

    CausalContrastInduction::induce(&episodes, std::slice::from_ref(&calibrated), policy).selected()
        [0]
    .clone()
}

fn clean_controlled_evidence(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
) -> Vec<InterventionalTransformationEpisode> {
    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9], [1, 8, 9]];

    let mut evidence = Vec::new();

    for state in states {
        let mut target_after = state.to_vec();

        target_after.push(5);

        evidence.push(controlled(&state, &target_after, target.clone()));

        evidence.push(controlled(&state, &state, contrast.clone()));
    }

    evidence
}

fn has_validation(
    result: &athlesia_universal_domain_learning::InterventionalCausalValidationResult,
    target: &CognitiveStructure,
    contrast: &CognitiveStructure,
) -> bool {
    result.selected().iter().any(|hypothesis| {
        hypothesis.transformation() == target
            && hypothesis.contrast_transformation() == contrast
            && hypothesis.effect_kind() == TransitionEffectKind::Added
            && hypothesis.effect_fact() == &atom(5)
    })
}

#[test]
fn interventional_policy_requires_positive_evidence_support_and_hard_bounds() {
    assert_eq!(
        InterventionalCausalThresholds::new(0, 1, 1, signal(100,), signal(100,),),
        None
    );

    assert_eq!(
        InterventionalCausalThresholds::new(1, 0, 1, signal(100,), signal(100,),),
        None
    );

    assert_eq!(
        InterventionalCausalThresholds::new(1, 1, 1, signal(0,), signal(100,),),
        None
    );

    let thresholds =
        InterventionalCausalThresholds::new(1, 1, 1, signal(100), signal(100)).unwrap();

    assert_eq!(
        InterventionalCausalValidationPolicy::new(0, 1, 1, 1, thresholds,),
        None
    );

    assert_eq!(
        InterventionalCausalValidationPolicy::new(1, 1, 1, 0, thresholds,),
        None
    );

    assert!(InterventionalCausalValidationPolicy::new(1, 1, 1, 1, thresholds,).is_some());
}

#[test]
fn passive_observation_alone_cannot_validate_causal_contrast() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let raw = clean_controlled_evidence(target, contrast);

    let passive = raw
        .into_iter()
        .map(|item| {
            InterventionalTransformationEpisode::new(
                item.episode().clone(),
                InterventionEvidenceKind::PassiveObservation,
            )
        })
        .collect::<Vec<_>>();

    let result = InterventionalCausalValidation::validate(
        &passive,
        std::slice::from_ref(&seed),
        default_policy(),
    );

    assert_eq!(result.selected_count(), 0);

    assert_eq!(result.rejected_without_matched_interventions(), 1);
}

#[test]
fn controlled_target_with_passive_contrast_does_not_form_intervention_pair() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let states = [[1, 2, 9], [1, 3, 9]];

    let mut evidence = Vec::new();

    for state in states {
        let mut target_after = state.to_vec();

        target_after.push(5);

        evidence.push(controlled(&state, &target_after, target.clone()));

        evidence.push(observed(&state, &state, contrast.clone()));
    }

    let result = InterventionalCausalValidation::validate(
        &evidence,
        std::slice::from_ref(&seed),
        default_policy(),
    );

    assert_eq!(result.selected_count(), 0);

    assert_eq!(result.rejected_without_matched_interventions(), 1);
}

#[test]
fn matched_controlled_assignments_validate_transformation_specific_effect() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let evidence = clean_controlled_evidence(target.clone(), contrast.clone());

    let result = InterventionalCausalValidation::validate(
        &evidence,
        std::slice::from_ref(&seed),
        default_policy(),
    );

    assert!(has_validation(&result, &target, &contrast,));

    let hypothesis = &result.selected()[0];

    assert_eq!(hypothesis.matched_intervention_state_count(), 4);

    assert_eq!(hypothesis.target_intervention_success_count(), 4);

    assert_eq!(hypothesis.contrast_intervention_success_count(), 0);

    assert_eq!(hypothesis.target_intervention_rate().value(), 1000);

    assert_eq!(hypothesis.contrast_intervention_rate().value(), 0);

    assert_eq!(hypothesis.interventional_lift().value(), 1000);

    assert_eq!(hypothesis.intervention_support_adequacy().value(), 1000);

    assert_eq!(
        hypothesis.validated_causal_confidence(),
        hypothesis.source_contrast_confidence()
    );
}

#[test]
fn intervention_effect_shared_by_target_and_control_has_zero_lift_and_is_rejected() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let states = [[1, 2, 9], [1, 3, 9]];

    let mut evidence = Vec::new();

    for state in states {
        let mut after = state.to_vec();

        after.push(5);

        evidence.push(controlled(&state, &after, target.clone()));

        evidence.push(controlled(&state, &after, contrast.clone()));
    }

    let result = InterventionalCausalValidation::validate(
        &evidence,
        std::slice::from_ref(&seed),
        validation_policy(
            ValidationBounds {
                max_seeds: 16,
                max_evaluations: 16,
                max_validated: 16,
                full_support: 2,
            },
            ValidationEvidence {
                matched_states: 2,
                target_interventions: 2,
                contrast_interventions: 2,
                lift: 1,
                confidence: 1,
            },
        ),
    );

    assert_eq!(result.selected_count(), 0);

    assert_eq!(result.rejected_below_interventional_threshold(), 1);
}

#[test]
fn intervention_failures_and_control_successes_remain_explicit_counterevidence() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9], [1, 8, 9]];

    let mut evidence = Vec::new();

    for (index, state) in states.into_iter().enumerate() {
        let target_after = if index == 3 {
            state.to_vec()
        } else {
            let mut after = state.to_vec();

            after.push(5);

            after
        };

        let contrast_after = if index == 0 {
            let mut after = state.to_vec();

            after.push(5);

            after
        } else {
            state.to_vec()
        };

        evidence.push(controlled(&state, &target_after, target.clone()));

        evidence.push(controlled(&state, &contrast_after, contrast.clone()));
    }

    let result = InterventionalCausalValidation::validate(
        &evidence,
        std::slice::from_ref(&seed),
        validation_policy(
            ValidationBounds {
                max_seeds: 16,
                max_evaluations: 16,
                max_validated: 16,
                full_support: 4,
            },
            ValidationEvidence {
                matched_states: 4,
                target_interventions: 4,
                contrast_interventions: 4,
                lift: 400,
                confidence: 100,
            },
        ),
    );

    let hypothesis = &result.selected()[0];

    assert_eq!(hypothesis.target_intervention_success_count(), 3);

    assert_eq!(hypothesis.target_intervention_failure_count(), 1);

    assert_eq!(hypothesis.contrast_intervention_success_count(), 1);

    assert_eq!(hypothesis.contrast_intervention_failure_count(), 3);

    assert_eq!(hypothesis.target_intervention_rate().value(), 750);

    assert_eq!(hypothesis.contrast_intervention_rate().value(), 250);

    assert_eq!(hypothesis.interventional_lift().value(), 500);
}

#[test]
fn repeated_intervention_on_one_state_cannot_fake_matched_state_diversity() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let mut evidence = Vec::new();

    for _ in 0..8 {
        evidence.push(controlled(&[1, 2, 9], &[1, 2, 5, 9], target.clone()));

        evidence.push(controlled(&[1, 2, 9], &[1, 2, 9], contrast.clone()));
    }

    let result = InterventionalCausalValidation::validate(
        &evidence,
        std::slice::from_ref(&seed),
        default_policy(),
    );

    assert_eq!(result.selected_count(), 0);

    assert_eq!(result.rejected_without_matched_interventions(), 1);
}

#[test]
fn passive_corroboration_and_counterevidence_are_retained_without_changing_causal_score() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let controlled_only = clean_controlled_evidence(target.clone(), contrast.clone());

    let base_result = InterventionalCausalValidation::validate(
        &controlled_only,
        std::slice::from_ref(&seed),
        default_policy(),
    );

    let mut enriched = controlled_only.clone();

    enriched.push(observed(&[1, 11, 9], &[1, 5, 11, 9], target.clone()));

    enriched.push(observed(&[1, 12, 9], &[1, 12, 9], contrast.clone()));

    enriched.push(observed(&[1, 13, 9], &[1, 13, 9], target.clone()));

    enriched.push(observed(&[1, 14, 9], &[1, 5, 14, 9], contrast));

    let enriched_result = InterventionalCausalValidation::validate(
        &enriched,
        std::slice::from_ref(&seed),
        default_policy(),
    );

    let base = &base_result.selected()[0];

    let enriched_hypothesis = &enriched_result.selected()[0];

    assert_eq!(
        enriched_hypothesis.validated_causal_confidence(),
        base.validated_causal_confidence()
    );

    assert_eq!(
        enriched_hypothesis.interventional_lift(),
        base.interventional_lift()
    );

    assert_eq!(enriched_hypothesis.passive_corroborating_count(), 2);

    assert_eq!(enriched_hypothesis.passive_counterevidence_count(), 2);
}

#[test]
fn small_balanced_intervention_sample_is_discounted_by_support_adequacy() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let evidence = vec![
        controlled(&[1, 2, 9], &[1, 2, 5, 9], target),
        controlled(&[1, 2, 9], &[1, 2, 9], contrast),
    ];

    let result = InterventionalCausalValidation::validate(
        &evidence,
        std::slice::from_ref(&seed),
        validation_policy(
            ValidationBounds {
                max_seeds: 16,
                max_evaluations: 16,
                max_validated: 16,
                full_support: 4,
            },
            ValidationEvidence {
                matched_states: 1,
                target_interventions: 1,
                contrast_interventions: 1,
                lift: 1,
                confidence: 1,
            },
        ),
    );

    let hypothesis = &result.selected()[0];

    assert_eq!(hypothesis.balanced_intervention_support(), 1);

    assert_eq!(hypothesis.intervention_support_adequacy().value(), 250);

    let expected = (u32::from(seed.contrast_confidence().value()) * 250) / 1000;

    assert_eq!(
        u32::from(hypothesis.validated_causal_confidence().value(),),
        expected
    );
}

#[test]
fn exact_transformation_structure_identity_remains_interventional_authority() {
    let target = ordered(&[10, 20]);

    let contrast = ordered(&[20, 10]);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let evidence = clean_controlled_evidence(target.clone(), contrast.clone());

    let result = InterventionalCausalValidation::validate(
        &evidence,
        std::slice::from_ref(&seed),
        default_policy(),
    );

    assert_ne!(target, contrast);

    assert!(has_validation(&result, &target, &contrast,));
}

#[test]
fn hard_seed_evaluation_and_final_frontiers_are_enforced_deterministically() {
    let target_a = atom(100);

    let contrast_a = atom(200);

    let target_b = atom(300);

    let contrast_b = atom(400);

    let seed_a = contrast_seed(target_a.clone(), contrast_a.clone());

    let seed_b = contrast_seed(target_b.clone(), contrast_b.clone());

    let mut evidence = clean_controlled_evidence(target_a, contrast_a);

    evidence.extend(clean_controlled_evidence(target_b, contrast_b));

    let seeds = vec![seed_a, seed_b];

    let seed_limited = InterventionalCausalValidation::validate(
        &evidence,
        &seeds,
        validation_policy(
            ValidationBounds {
                max_seeds: 1,
                max_evaluations: 1,
                max_validated: 1,
                full_support: 4,
            },
            ValidationEvidence {
                matched_states: 2,
                target_interventions: 2,
                contrast_interventions: 2,
                lift: 400,
                confidence: 100,
            },
        ),
    );

    assert_eq!(seed_limited.considered_seed_count(), 1);

    assert!(seed_limited.seed_truncated());

    let evaluation_limited = InterventionalCausalValidation::validate(
        &evidence,
        &seeds,
        validation_policy(
            ValidationBounds {
                max_seeds: 2,
                max_evaluations: 1,
                max_validated: 1,
                full_support: 4,
            },
            ValidationEvidence {
                matched_states: 2,
                target_interventions: 2,
                contrast_interventions: 2,
                lift: 400,
                confidence: 100,
            },
        ),
    );

    assert_eq!(evaluation_limited.evaluated_seed_count(), 1);

    assert!(evaluation_limited.evaluation_truncated());

    let final_limited = InterventionalCausalValidation::validate(
        &evidence,
        &seeds,
        validation_policy(
            ValidationBounds {
                max_seeds: 2,
                max_evaluations: 2,
                max_validated: 1,
                full_support: 4,
            },
            ValidationEvidence {
                matched_states: 2,
                target_interventions: 2,
                contrast_interventions: 2,
                lift: 400,
                confidence: 100,
            },
        ),
    );

    assert_eq!(final_limited.selected_count(), 1);

    assert_eq!(final_limited.admitted_before_frontier(), 2);
}

#[test]
fn interventional_validation_is_deterministic_non_mutating_and_facade_equivalent() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = contrast_seed(target.clone(), contrast.clone());

    let evidence = clean_controlled_evidence(target, contrast);

    let seeds = vec![seed];

    let evidence_before = evidence.clone();

    let seeds_before = seeds.clone();

    let policy = default_policy();

    let direct = InterventionalCausalValidation::validate(&evidence, &seeds, policy);

    let facade = UniversalInterventionalCausalValidation::evaluate(&evidence, &seeds, policy);

    let repeated = UniversalInterventionalCausalValidation::evaluate(&evidence, &seeds, policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(evidence, evidence_before);

    assert_eq!(seeds, seeds_before);

    assert_eq!(facade.input_seed_count(), seeds.len());

    assert!(facade.evaluated_seed_count() <= facade.considered_seed_count());
}
