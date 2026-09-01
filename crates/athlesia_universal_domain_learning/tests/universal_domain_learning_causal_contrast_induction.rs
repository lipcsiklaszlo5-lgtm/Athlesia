use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    CalibratedRuleConfidence, CausalContrastInduction, CausalContrastPolicy,
    CausalContrastThresholds, ContextPremiseSet, ContextualTransitionEvidenceThresholds,
    ContextualTransitionRuleInduction, ContextualTransitionRulePolicy, CrossContextGeneralization,
    CrossContextGeneralizationPolicy, CrossContextGeneralizationThresholds, GroundedStateSnapshot,
    GroundedTransformationEpisode, RuleConfidenceCalibration, RuleConfidenceCalibrationPolicy,
    TransitionEffectKind, UniversalCausalContrastInduction,
};

#[derive(Clone, Copy)]
struct ContrastBounds {
    max_seeds: usize,
    max_contrasts_per_seed: usize,
    max_evaluations: usize,
    max_hypotheses: usize,
}

#[derive(Clone, Copy)]
struct ContrastEvidence {
    matched_states: usize,
    target_opportunities: u64,
    contrast_opportunities: u64,
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

fn bounds(
    max_seeds: usize,
    max_contrasts_per_seed: usize,
    max_evaluations: usize,
    max_hypotheses: usize,
) -> ContrastBounds {
    ContrastBounds {
        max_seeds,
        max_contrasts_per_seed,
        max_evaluations,
        max_hypotheses,
    }
}

fn evidence(
    matched_states: usize,
    target_opportunities: u64,
    contrast_opportunities: u64,
    lift: u16,
    confidence: u16,
) -> ContrastEvidence {
    ContrastEvidence {
        matched_states,
        target_opportunities,
        contrast_opportunities,
        lift,
        confidence,
    }
}

fn policy(search: ContrastBounds, evidence: ContrastEvidence) -> CausalContrastPolicy {
    let thresholds = CausalContrastThresholds::new(
        evidence.matched_states,
        evidence.target_opportunities,
        evidence.contrast_opportunities,
        signal(evidence.lift),
        signal(evidence.confidence),
    )
    .unwrap();

    CausalContrastPolicy::new(
        search.max_seeds,
        search.max_contrasts_per_seed,
        search.max_evaluations,
        search.max_hypotheses,
        thresholds,
    )
    .unwrap()
}

fn default_policy() -> CausalContrastPolicy {
    policy(bounds(128, 16, 128, 32), evidence(2, 2, 2, 400, 200))
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

    let calibration = RuleConfidenceCalibration::calibrate(
        &training,
        generalizations.selected(),
        &[],
        calibration_policy,
    );

    let expected_context = ContextPremiseSet::new(vec![atom(1)]).unwrap();

    calibration
        .selected()
        .iter()
        .find(|seed| {
            seed.transformation() == &transformation
                && seed.context() == &expected_context
                && seed.effect_kind() == TransitionEffectKind::Added
                && seed.effect_fact() == &atom(5)
        })
        .unwrap()
        .clone()
}

fn controlled_history(
    target: CognitiveStructure,
    contrast: CognitiveStructure,
    contrast_has_effect: bool,
) -> Vec<GroundedTransformationEpisode> {
    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9], [1, 8, 9]];

    let mut episodes = Vec::new();

    for state in states {
        let mut target_after = state.to_vec();

        target_after.push(5);

        episodes.push(transition(&state, &target_after, target.clone()));

        let contrast_after = if contrast_has_effect {
            let mut after = state.to_vec();

            after.push(5);

            after
        } else {
            state.to_vec()
        };

        episodes.push(transition(&state, &contrast_after, contrast.clone()));
    }

    episodes
}

fn has_contrast(
    result: &athlesia_universal_domain_learning::CausalContrastInductionResult,
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
fn causal_contrast_policy_requires_positive_evidence_and_hard_bounds() {
    assert_eq!(
        CausalContrastThresholds::new(0, 1, 1, signal(100,), signal(100,),),
        None
    );

    assert_eq!(
        CausalContrastThresholds::new(1, 0, 1, signal(100,), signal(100,),),
        None
    );

    assert_eq!(
        CausalContrastThresholds::new(1, 1, 1, signal(0,), signal(100,),),
        None
    );

    let thresholds = CausalContrastThresholds::new(1, 1, 1, signal(100), signal(100)).unwrap();

    assert_eq!(CausalContrastPolicy::new(0, 10, 10, 10, thresholds,), None);

    assert_eq!(CausalContrastPolicy::new(10, 0, 10, 10, thresholds,), None);

    assert!(CausalContrastPolicy::new(10, 10, 10, 10, thresholds,).is_some());
}

#[test]
fn merely_sharing_context_without_exact_pre_state_match_cannot_create_contrast() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = calibrated_seed(target.clone());

    let episodes = vec![
        transition(&[1, 2, 9], &[1, 2, 5, 9], target),
        transition(&[1, 3, 9], &[1, 3, 9], contrast),
    ];

    let result = CausalContrastInduction::induce(
        &episodes,
        std::slice::from_ref(&seed),
        policy(bounds(16, 16, 16, 16), evidence(1, 1, 1, 1, 1)),
    );

    assert_eq!(result.possible_contrast_count(), 0);

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn exact_matched_pre_states_discover_transformation_specific_effect_contrast() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = calibrated_seed(target.clone());

    let episodes = controlled_history(target.clone(), contrast.clone(), false);

    let result =
        CausalContrastInduction::induce(&episodes, std::slice::from_ref(&seed), default_policy());

    assert!(has_contrast(&result, &target, &contrast,));

    let hypothesis = &result.selected()[0];

    assert_eq!(hypothesis.matched_state_count(), 4);

    assert_eq!(hypothesis.target_effect_rate().value(), 1000);

    assert_eq!(hypothesis.contrast_effect_rate().value(), 0);

    assert_eq!(hypothesis.contrast_lift().value(), 1000);

    assert_eq!(
        hypothesis.contrast_confidence().value(),
        seed.calibrated_confidence().value()
    );
}

#[test]
fn effect_common_to_target_and_contrast_has_zero_lift_and_is_rejected() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = calibrated_seed(target.clone());

    let episodes = controlled_history(target, contrast, true);

    let result = CausalContrastInduction::induce(
        &episodes,
        std::slice::from_ref(&seed),
        policy(bounds(16, 16, 16, 16), evidence(2, 2, 2, 1, 1)),
    );

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn target_failures_and_contrast_successes_remain_explicit_counterevidence() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = calibrated_seed(target.clone());

    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9], [1, 8, 9]];

    let mut episodes = Vec::new();

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

        episodes.push(transition(&state, &target_after, target.clone()));

        episodes.push(transition(&state, &contrast_after, contrast.clone()));
    }

    let result = CausalContrastInduction::induce(
        &episodes,
        std::slice::from_ref(&seed),
        policy(bounds(16, 16, 16, 16), evidence(4, 4, 4, 400, 200)),
    );

    let hypothesis = &result.selected()[0];

    assert_eq!(hypothesis.target_success_count(), 3);

    assert_eq!(hypothesis.target_failure_count(), 1);

    assert_eq!(hypothesis.contrast_success_count(), 1);

    assert_eq!(hypothesis.contrast_failure_count(), 3);

    assert_eq!(hypothesis.target_effect_rate().value(), 750);

    assert_eq!(hypothesis.contrast_effect_rate().value(), 250);

    assert_eq!(hypothesis.contrast_lift().value(), 500);
}

#[test]
fn exact_transformation_structure_identity_supports_reordered_control_contrast() {
    let target = ordered(&[10, 20]);

    let contrast = ordered(&[20, 10]);

    let seed = calibrated_seed(target.clone());

    let episodes = controlled_history(target.clone(), contrast.clone(), false);

    let result =
        CausalContrastInduction::induce(&episodes, std::slice::from_ref(&seed), default_policy());

    assert_ne!(target, contrast);

    assert!(has_contrast(&result, &target, &contrast,));
}

#[test]
fn transformation_cannot_be_used_as_its_own_contrast() {
    let target = atom(100);

    let seed = calibrated_seed(target.clone());

    let episodes = controlled_history(target.clone(), target, false);

    let result = CausalContrastInduction::induce(
        &episodes,
        std::slice::from_ref(&seed),
        policy(bounds(16, 16, 16, 16), evidence(1, 1, 1, 1, 1)),
    );

    assert_eq!(result.possible_contrast_count(), 0);

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn multiple_alternative_transformations_remain_distinct_contrast_hypotheses() {
    let target = atom(100);

    let first_contrast = atom(200);

    let second_contrast = atom(300);

    let seed = calibrated_seed(target.clone());

    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9]];

    let mut episodes = Vec::new();

    for state in states {
        let mut target_after = state.to_vec();

        target_after.push(5);

        episodes.push(transition(&state, &target_after, target.clone()));

        episodes.push(transition(&state, &state, first_contrast.clone()));

        episodes.push(transition(&state, &state, second_contrast.clone()));
    }

    let result = CausalContrastInduction::induce(
        &episodes,
        std::slice::from_ref(&seed),
        policy(bounds(16, 16, 32, 16), evidence(2, 2, 2, 400, 200)),
    );

    assert!(has_contrast(&result, &target, &first_contrast,));

    assert!(has_contrast(&result, &target, &second_contrast,));
}

#[test]
fn repeated_observations_of_one_state_cannot_fake_minimum_matched_state_diversity() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = calibrated_seed(target.clone());

    let mut episodes = Vec::new();

    for _ in 0..8 {
        episodes.push(transition(&[1, 2, 9], &[1, 2, 5, 9], target.clone()));

        episodes.push(transition(&[1, 2, 9], &[1, 2, 9], contrast.clone()));
    }

    let result = CausalContrastInduction::induce(
        &episodes,
        std::slice::from_ref(&seed),
        policy(bounds(16, 16, 32, 16), evidence(2, 2, 2, 400, 200)),
    );

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn hard_per_seed_contrast_frontier_prioritizes_more_matched_states() {
    let target = atom(100);

    let broad_contrast = atom(200);

    let narrow_contrast = atom(300);

    let seed = calibrated_seed(target.clone());

    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9]];

    let mut episodes = Vec::new();

    for (index, state) in states.into_iter().enumerate() {
        let mut target_after = state.to_vec();

        target_after.push(5);

        episodes.push(transition(&state, &target_after, target.clone()));

        episodes.push(transition(&state, &state, broad_contrast.clone()));

        if index < 2 {
            episodes.push(transition(&state, &state, narrow_contrast.clone()));
        }
    }

    let result = CausalContrastInduction::induce(
        &episodes,
        std::slice::from_ref(&seed),
        policy(bounds(16, 1, 16, 16), evidence(2, 2, 2, 400, 200)),
    );

    assert_eq!(result.possible_contrast_count(), 2);

    assert_eq!(result.generated_contrast_count(), 1);

    assert!(result.contrast_generation_truncated());

    assert!(has_contrast(&result, &target, &broad_contrast,));

    assert!(!has_contrast(&result, &target, &narrow_contrast,));
}

#[test]
fn hard_evaluation_and_final_frontiers_are_deterministic_and_input_order_invariant() {
    let target = atom(100);

    let first_contrast = atom(200);

    let second_contrast = atom(300);

    let third_contrast = atom(400);

    let seed = calibrated_seed(target.clone());

    let states = [[1, 2, 9], [1, 3, 9], [1, 4, 9]];

    let mut original = Vec::new();

    for state in states {
        let mut target_after = state.to_vec();

        target_after.push(5);

        original.push(transition(&state, &target_after, target.clone()));

        original.push(transition(&state, &state, first_contrast.clone()));

        let mut second_after = state.to_vec();

        if state[1] == 2 {
            second_after.push(5);
        }

        original.push(transition(&state, &second_after, second_contrast.clone()));

        original.push(transition(&state, &state, third_contrast.clone()));
    }

    let mut reversed = original.clone();

    reversed.reverse();

    let induction_policy = policy(bounds(16, 16, 2, 1), evidence(2, 2, 2, 300, 100));

    let first =
        CausalContrastInduction::induce(&original, std::slice::from_ref(&seed), induction_policy);

    let second =
        CausalContrastInduction::induce(&reversed, std::slice::from_ref(&seed), induction_policy);

    assert_eq!(first, second);

    assert_eq!(first.evaluated_candidate_count(), 2);

    assert!(first.evaluation_truncated());

    assert_eq!(first.selected_count(), 1);

    assert!(first.admitted_before_frontier() > first.selected_count());

    assert_eq!(
        first.selected()[0].contrast_transformation(),
        &first_contrast
    );

    assert_eq!(first.selected()[0].contrast_lift().value(), 1000);
}

#[test]
fn causal_contrast_induction_is_deterministic_non_mutating_and_facade_equivalent() {
    let target = atom(100);

    let contrast = atom(200);

    let seed = calibrated_seed(target.clone());

    let episodes = controlled_history(target, contrast, false);

    let seeds = vec![seed];

    let episodes_before = episodes.clone();

    let seeds_before = seeds.clone();

    let induction_policy = default_policy();

    let direct = CausalContrastInduction::induce(&episodes, &seeds, induction_policy);

    let facade = UniversalCausalContrastInduction::evaluate(&episodes, &seeds, induction_policy);

    let repeated = UniversalCausalContrastInduction::evaluate(&episodes, &seeds, induction_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, episodes_before);

    assert_eq!(seeds, seeds_before);

    assert_eq!(facade.input_seed_count(), seeds.len());

    assert!(facade.considered_seed_count() <= facade.input_seed_count());

    assert!(facade.evaluated_candidate_count() <= facade.generated_contrast_count());
}
