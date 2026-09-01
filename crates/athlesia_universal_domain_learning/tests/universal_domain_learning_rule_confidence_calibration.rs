use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    ContextualTransitionEvidenceThresholds, ContextualTransitionRuleInduction,
    ContextualTransitionRulePolicy, CrossContextGeneralization, CrossContextGeneralizationPolicy,
    CrossContextGeneralizationThresholds, ExceptionRefinement, ExceptionRefinementPolicy,
    ExceptionRefinementThresholds, GroundedCrossContextGeneralizationHypothesis,
    GroundedExceptionRefinementHypothesis, GroundedStateSnapshot, GroundedTransformationEpisode,
    RuleConfidenceCalibration, RuleConfidenceCalibrationPolicy, UniversalRuleConfidenceCalibration,
};

#[derive(Clone, Copy)]
struct CalibrationSpec {
    minimum_support: u64,
    full_support: u64,
    minimum_confidence: u16,
    max_seeds: usize,
    max_exception_checks: usize,
    max_rules: usize,
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

fn calibration_policy(spec: CalibrationSpec) -> RuleConfidenceCalibrationPolicy {
    RuleConfidenceCalibrationPolicy::new(
        spec.minimum_support,
        spec.full_support,
        signal(spec.minimum_confidence),
        spec.max_seeds,
        spec.max_exception_checks,
        spec.max_rules,
    )
    .unwrap()
}

fn default_calibration_policy() -> RuleConfidenceCalibrationPolicy {
    calibration_policy(CalibrationSpec {
        minimum_support: 2,
        full_support: 4,
        minimum_confidence: 1,
        max_seeds: 128,
        max_exception_checks: 128,
        max_rules: 64,
    })
}

fn base_history(transformation: CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
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

fn dual_marker_history(transformation: CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
    vec![
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[1, 6, 7, 9], &[1, 6, 7, 9], transformation.clone()),
        transition(&[1, 6, 7, 9], &[1, 6, 7, 9], transformation),
    ]
}

fn seed_generalizations_from(
    episodes: &[GroundedTransformationEpisode],
) -> Vec<GroundedCrossContextGeneralizationHypothesis> {
    let contextual_thresholds =
        ContextualTransitionEvidenceThresholds::new(2, signal(900), signal(300), signal(300))
            .unwrap();

    let contextual_policy =
        ContextualTransitionRulePolicy::new(2, 256, 8192, 256, contextual_thresholds).unwrap();

    let contextual = ContextualTransitionRuleInduction::induce(episodes, &[], contextual_policy);

    let generalization_thresholds =
        CrossContextGeneralizationThresholds::new(2, 2, signal(600), signal(200)).unwrap();

    let generalization_policy =
        CrossContextGeneralizationPolicy::new(256, 2, 256, 128, generalization_thresholds).unwrap();

    CrossContextGeneralization::generalize(episodes, contextual.selected(), generalization_policy)
        .selected()
        .to_vec()
}

fn exceptions_from(
    episodes: &[GroundedTransformationEpisode],
    seeds: &[GroundedCrossContextGeneralizationHypothesis],
) -> Vec<GroundedExceptionRefinementHypothesis> {
    let thresholds = ExceptionRefinementThresholds::new(2, signal(900), signal(400)).unwrap();

    let policy = ExceptionRefinementPolicy::new(256, 2, 256, 256, 128, thresholds).unwrap();

    ExceptionRefinement::refine(episodes, seeds, policy)
        .selected()
        .to_vec()
}

fn seed_for_effect(
    seeds: &[GroundedCrossContextGeneralizationHypothesis],
    transformation: &CognitiveStructure,
    fact: &CognitiveStructure,
) -> GroundedCrossContextGeneralizationHypothesis {
    seeds
        .iter()
        .find(|seed| seed.transformation() == transformation && seed.effect_fact() == fact)
        .unwrap()
        .clone()
}

#[test]
fn calibration_policy_requires_positive_support_and_hard_bounds() {
    assert_eq!(
        RuleConfidenceCalibrationPolicy::new(0, 4, signal(1), 10, 10, 10,),
        None
    );

    assert_eq!(
        RuleConfidenceCalibrationPolicy::new(4, 2, signal(1), 10, 10, 10,),
        None
    );

    assert_eq!(
        RuleConfidenceCalibrationPolicy::new(1, 4, signal(1), 0, 10, 10,),
        None
    );

    assert_eq!(
        RuleConfidenceCalibrationPolicy::new(1, 4, signal(1), 10, 0, 10,),
        None
    );

    assert!(RuleConfidenceCalibrationPolicy::new(1, 4, signal(1), 10, 10, 10,).is_some());
}

#[test]
fn perfect_small_sample_precision_is_discounted_by_support_adequacy() {
    let transformation = atom(100);

    let training = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&training);

    let seed = seed_for_effect(&seeds, &transformation, &atom(5));

    let evaluation = vec![
        transition(&[1, 2], &[1, 2, 5], transformation.clone()),
        transition(&[1, 3], &[1, 3, 5], transformation),
    ];

    let result = RuleConfidenceCalibration::calibrate(
        &evaluation,
        &[seed],
        &[],
        calibration_policy(CalibrationSpec {
            minimum_support: 1,
            full_support: 10,
            minimum_confidence: 1,
            max_seeds: 16,
            max_exception_checks: 16,
            max_rules: 16,
        }),
    );

    let calibrated = &result.selected()[0];

    assert_eq!(calibrated.raw_precision().value(), 1000);

    assert_eq!(calibrated.effective_precision().value(), 1000);

    assert_eq!(calibrated.support_adequacy().value(), 200);

    assert_eq!(calibrated.calibrated_confidence().value(), 200);
}

#[test]
fn sufficient_support_allows_calibrated_confidence_to_equal_effective_precision() {
    let transformation = atom(100);

    let episodes = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&episodes);

    let seed = seed_for_effect(&seeds, &transformation, &atom(5));

    let result = RuleConfidenceCalibration::calibrate(
        &episodes,
        &[seed],
        &[],
        calibration_policy(CalibrationSpec {
            minimum_support: 2,
            full_support: 4,
            minimum_confidence: 1,
            max_seeds: 16,
            max_exception_checks: 16,
            max_rules: 16,
        }),
    );

    let calibrated = &result.selected()[0];

    assert_eq!(calibrated.support_adequacy().value(), 1000);

    assert_eq!(
        calibrated.calibrated_confidence(),
        calibrated.effective_precision()
    );
}

#[test]
fn learned_exception_abstention_removes_known_failures_without_rewriting_raw_precision() {
    let transformation = atom(100);

    let episodes = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&episodes);

    let exceptions = exceptions_from(&episodes, &seeds);

    assert!(!exceptions.is_empty());

    let seed = seed_for_effect(&seeds, &transformation, &atom(5));

    let result = RuleConfidenceCalibration::calibrate(
        &episodes,
        &[seed],
        &exceptions,
        default_calibration_policy(),
    );

    let calibrated = &result.selected()[0];

    assert_eq!(calibrated.total_opportunity_count(), 6);

    assert_eq!(calibrated.total_success_count(), 4);

    assert_eq!(calibrated.total_failure_count(), 2);

    assert_eq!(calibrated.raw_precision().value(), 666);

    assert_eq!(calibrated.exception_triggered_failure_count(), 2);

    assert_eq!(calibrated.effective_opportunity_count(), 4);

    assert_eq!(calibrated.effective_success_count(), 4);

    assert_eq!(calibrated.effective_failure_count(), 0);

    assert_eq!(calibrated.effective_precision().value(), 1000);

    assert_eq!(calibrated.calibrated_confidence().value(), 1000);

    assert_eq!(calibrated.abstention_rate().value(), 333);
}

#[test]
fn exception_leakage_is_abstained_and_retained_as_explicit_evidence() {
    let transformation = atom(100);

    let training = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&training);

    let exceptions = exceptions_from(&training, &seeds);

    let seed = seed_for_effect(&seeds, &transformation, &atom(5));

    let evaluation = vec![
        transition(&[1, 2], &[1, 2, 5], transformation.clone()),
        transition(&[1, 3], &[1, 3, 5], transformation.clone()),
        transition(&[1, 6], &[1, 5, 6], transformation.clone()),
        transition(&[1, 6], &[1, 6], transformation),
    ];

    let result = RuleConfidenceCalibration::calibrate(
        &evaluation,
        &[seed],
        &exceptions,
        calibration_policy(CalibrationSpec {
            minimum_support: 2,
            full_support: 2,
            minimum_confidence: 1,
            max_seeds: 16,
            max_exception_checks: 16,
            max_rules: 16,
        }),
    );

    let calibrated = &result.selected()[0];

    assert_eq!(calibrated.exception_triggered_opportunity_count(), 2);

    assert_eq!(calibrated.exception_triggered_failure_count(), 1);

    assert_eq!(calibrated.exception_triggered_success_count(), 1);

    assert_eq!(calibrated.effective_opportunity_count(), 2);

    assert_eq!(calibrated.effective_precision().value(), 1000);
}

#[test]
fn unrelated_exception_identity_cannot_change_rule_confidence() {
    let first = atom(100);

    let second = atom(200);

    let first_history = base_history(first.clone());

    let second_history = base_history(second.clone());

    let first_seeds = seed_generalizations_from(&first_history);

    let second_seeds = seed_generalizations_from(&second_history);

    let unrelated_exceptions = exceptions_from(&second_history, &second_seeds);

    let first_seed = seed_for_effect(&first_seeds, &first, &atom(5));

    let without = RuleConfidenceCalibration::calibrate(
        &first_history,
        std::slice::from_ref(&first_seed),
        &[],
        default_calibration_policy(),
    );

    let with_unrelated = RuleConfidenceCalibration::calibrate(
        &first_history,
        &[first_seed],
        &unrelated_exceptions,
        default_calibration_policy(),
    );

    assert_eq!(without, with_unrelated);

    assert_eq!(with_unrelated.total_matching_exception_count(), 0);
}

#[test]
fn exact_transformation_structure_identity_remains_calibration_authority() {
    let first = ordered(&[10, 20]);

    let second = ordered(&[20, 10]);

    let first_history = base_history(first.clone());

    let second_history = base_history(second.clone());

    let first_seeds = seed_generalizations_from(&first_history);

    let second_seeds = seed_generalizations_from(&second_history);

    let first_exceptions = exceptions_from(&first_history, &first_seeds);

    let second_seed = seed_for_effect(&second_seeds, &second, &atom(5));

    let result = RuleConfidenceCalibration::calibrate(
        &second_history,
        &[second_seed],
        &first_exceptions,
        default_calibration_policy(),
    );

    assert_ne!(first, second);

    assert_eq!(result.total_matching_exception_count(), 0);

    assert_eq!(result.selected_count(), 1);
}

#[test]
fn confidence_threshold_rejects_high_precision_rule_when_evidence_volume_is_too_small() {
    let transformation = atom(100);

    let training = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&training);

    let seed = seed_for_effect(&seeds, &transformation, &atom(5));

    let evaluation = vec![
        transition(&[1, 2], &[1, 2, 5], transformation.clone()),
        transition(&[1, 3], &[1, 3, 5], transformation),
    ];

    let result = RuleConfidenceCalibration::calibrate(
        &evaluation,
        &[seed],
        &[],
        calibration_policy(CalibrationSpec {
            minimum_support: 1,
            full_support: 10,
            minimum_confidence: 500,
            max_seeds: 16,
            max_exception_checks: 16,
            max_rules: 16,
        }),
    );

    assert_eq!(result.selected_count(), 0);

    assert_eq!(result.rejected_below_confidence(), 1);
}

#[test]
fn hard_exception_check_budget_is_reported_and_never_exceeded() {
    let transformation = atom(100);

    let episodes = dual_marker_history(transformation.clone());

    let seeds = seed_generalizations_from(&episodes);

    let exceptions = exceptions_from(&episodes, &seeds);

    let seed = seed_for_effect(&seeds, &transformation, &atom(5));

    assert!(exceptions.len() >= 2);

    let result = RuleConfidenceCalibration::calibrate(
        &episodes,
        &[seed],
        &exceptions,
        calibration_policy(CalibrationSpec {
            minimum_support: 2,
            full_support: 4,
            minimum_confidence: 1,
            max_seeds: 16,
            max_exception_checks: 1,
            max_rules: 16,
        }),
    );

    assert!(result.total_matching_exception_count() >= 2);

    assert_eq!(result.total_checked_exception_count(), 1);

    assert!(result.exception_check_budget_exhausted());

    if let Some(calibrated) = result.selected().first() {
        assert_eq!(calibrated.checked_exception_count(), 1);

        assert!(calibrated.exception_check_truncated());
    }
}

#[test]
fn hard_seed_rule_budget_limits_calibration_frontier() {
    let first = atom(100);

    let second = atom(200);

    let mut episodes = base_history(first.clone());

    episodes.extend(base_history(second.clone()));

    let seeds = seed_generalizations_from(&episodes);

    assert!(seeds
        .iter()
        .any(|seed| { seed.transformation() == &first },));

    assert!(seeds
        .iter()
        .any(|seed| { seed.transformation() == &second },));

    let result = RuleConfidenceCalibration::calibrate(
        &episodes,
        &seeds,
        &[],
        calibration_policy(CalibrationSpec {
            minimum_support: 2,
            full_support: 4,
            minimum_confidence: 1,
            max_seeds: 1,
            max_exception_checks: 16,
            max_rules: 16,
        }),
    );

    assert_eq!(result.considered_seed_count(), 1);

    assert!(result.seed_truncated());

    assert!(result.selected_count() <= 1);
}

#[test]
fn hard_calibrated_rule_frontier_is_deterministic_and_input_order_invariant() {
    let first = atom(100);

    let second = atom(200);

    let mut original = base_history(first);

    original.extend(base_history(second));

    let seeds = seed_generalizations_from(&original);

    let exceptions = exceptions_from(&original, &seeds);

    let mut reversed_episodes = original.clone();

    reversed_episodes.reverse();

    let mut reversed_seeds = seeds.clone();

    reversed_seeds.reverse();

    let mut reversed_exceptions = exceptions.clone();

    reversed_exceptions.reverse();

    let policy = calibration_policy(CalibrationSpec {
        minimum_support: 2,
        full_support: 4,
        minimum_confidence: 1,
        max_seeds: 128,
        max_exception_checks: 128,
        max_rules: 1,
    });

    let first_result = RuleConfidenceCalibration::calibrate(&original, &seeds, &exceptions, policy);

    let second_result = RuleConfidenceCalibration::calibrate(
        &reversed_episodes,
        &reversed_seeds,
        &reversed_exceptions,
        policy,
    );

    assert_eq!(first_result, second_result);

    assert_eq!(first_result.selected_count(), 1);

    assert!(first_result.admitted_before_frontier() > first_result.selected_count());

    assert_eq!(
        first_result.selected()[0].calibrated_confidence().value(),
        1000
    );
}

#[test]
fn rule_confidence_calibration_is_deterministic_non_mutating_and_facade_equivalent() {
    let episodes = base_history(atom(100));

    let seeds = seed_generalizations_from(&episodes);

    let exceptions = exceptions_from(&episodes, &seeds);

    let episodes_before = episodes.clone();

    let seeds_before = seeds.clone();

    let exceptions_before = exceptions.clone();

    let policy = default_calibration_policy();

    let direct = RuleConfidenceCalibration::calibrate(&episodes, &seeds, &exceptions, policy);

    let facade =
        UniversalRuleConfidenceCalibration::evaluate(&episodes, &seeds, &exceptions, policy);

    let repeated =
        UniversalRuleConfidenceCalibration::evaluate(&episodes, &seeds, &exceptions, policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, episodes_before);

    assert_eq!(seeds, seeds_before);

    assert_eq!(exceptions, exceptions_before);

    assert_eq!(facade.input_seed_count(), seeds.len());

    assert!(facade.considered_seed_count() <= facade.input_seed_count());

    assert!(facade.total_checked_exception_count() <= facade.total_matching_exception_count());
}
