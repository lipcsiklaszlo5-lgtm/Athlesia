use athlesia_mindstone_sparse_cognition::{
    CognitiveAdmissionClass, CognitiveBudget, CognitiveSignal, EpistemicOutcomePrediction,
    ExpectedInformationGain, MindstoneExpectedInformationGain, MindstoneSignalProfile,
    SparseCognitionPolicy,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn outcome(weight: u32, uncertainty: u16) -> EpistemicOutcomePrediction {
    EpistemicOutcomePrediction::new(weight, signal(uncertainty)).unwrap()
}

fn profile(
    surprise: u16,
    uncertainty: u16,
    novelty: u16,
    learning_progress: u16,
    information_gain: u16,
) -> MindstoneSignalProfile {
    MindstoneSignalProfile::new(
        signal(surprise),
        signal(uncertainty),
        signal(novelty),
        signal(learning_progress),
        signal(information_gain),
    )
}

fn policy() -> SparseCognitionPolicy {
    SparseCognitionPolicy::new(signal(200), signal(600), 2, 8).unwrap()
}

fn budget(units: u32) -> CognitiveBudget {
    CognitiveBudget::new(units).unwrap()
}

#[test]
fn epistemic_outcome_requires_positive_weight() {
    assert_eq!(EpistemicOutcomePrediction::new(0, signal(500),), None);

    let valid = outcome(3, 500);

    assert_eq!(valid.weight(), 3);

    assert_eq!(valid.resulting_uncertainty().value(), 500);
}

#[test]
fn expected_information_gain_requires_at_least_one_predicted_outcome() {
    assert_eq!(ExpectedInformationGain::estimate(signal(800), &[],), None);
}

#[test]
fn unchanged_expected_uncertainty_has_zero_information_gain() {
    let estimate =
        ExpectedInformationGain::estimate(signal(700), &[outcome(1, 700), outcome(3, 700)])
            .unwrap();

    assert_eq!(estimate.expected_uncertainty().value(), 700);

    assert_eq!(estimate.information_gain(), CognitiveSignal::zero());

    assert!(!estimate.predicts_learning());
}

#[test]
fn deterministic_uncertainty_reduction_has_exact_information_gain() {
    let estimate = ExpectedInformationGain::estimate(signal(900), &[outcome(1, 300)]).unwrap();

    assert_eq!(estimate.current_uncertainty().value(), 900);

    assert_eq!(estimate.expected_uncertainty().value(), 300);

    assert_eq!(estimate.information_gain().value(), 600);

    assert!(estimate.predicts_learning());
}

#[test]
fn weighted_outcomes_compute_expected_uncertainty_before_gain() {
    let estimate =
        ExpectedInformationGain::estimate(signal(800), &[outcome(1, 200), outcome(3, 600)])
            .unwrap();

    assert_eq!(estimate.total_weight(), 4);

    assert_eq!(estimate.outcome_count(), 2);

    assert_eq!(estimate.expected_uncertainty().value(), 500);

    assert_eq!(estimate.information_gain().value(), 300);
}

#[test]
fn expected_uncertainty_increase_never_becomes_negative_information_gain() {
    let estimate =
        ExpectedInformationGain::estimate(signal(300), &[outcome(1, 700), outcome(1, 900)])
            .unwrap();

    assert_eq!(estimate.expected_uncertainty().value(), 800);

    assert_eq!(estimate.information_gain(), CognitiveSignal::zero());
}

#[test]
fn proportional_weight_rescaling_preserves_expected_information_gain() {
    let small = ExpectedInformationGain::estimate(signal(900), &[outcome(1, 200), outcome(3, 600)])
        .unwrap();

    let large =
        ExpectedInformationGain::estimate(signal(900), &[outcome(10, 200), outcome(30, 600)])
            .unwrap();

    assert_eq!(small.expected_uncertainty(), large.expected_uncertainty());

    assert_eq!(small.information_gain(), large.information_gain());
}

#[test]
fn expected_uncertainty_rounding_is_conservative_against_gain_overstatement() {
    let estimate =
        ExpectedInformationGain::estimate(signal(500), &[outcome(1, 0), outcome(2, 1)]).unwrap();

    assert_eq!(estimate.expected_uncertainty().value(), 1);

    assert_eq!(estimate.information_gain().value(), 499);
}

#[test]
fn derived_information_gain_replaces_stale_profile_value_without_mutating_source() {
    let base = profile(100, 600, 100, 100, 1000);

    let before = base;

    let updated = base.with_information_gain(signal(250));

    let expected = profile(100, 600, 100, 100, 250);

    assert_eq!(updated, expected);

    assert_eq!(updated.expected_information_gain_signal().value(), 250);

    assert_eq!(base, before);
}

#[test]
fn expected_information_gain_can_raise_low_pressure_event_into_sparse_admission() {
    let base = profile(0, 400, 0, 0, 0);

    let baseline_class = policy().classify(base);

    let result =
        MindstoneExpectedInformationGain::evaluate(base, &[outcome(1, 0)], policy(), budget(100))
            .unwrap();

    assert_eq!(baseline_class, CognitiveAdmissionClass::Ignore);

    assert_eq!(result.estimate().information_gain().value(), 400);

    assert_eq!(
        result.profile().expected_information_gain_signal().value(),
        400
    );

    assert_eq!(
        result.decision().class(),
        CognitiveAdmissionClass::CheapUpdate
    );
}

#[test]
fn expected_information_gain_admission_preserves_hard_compute_budget() {
    let base = profile(1000, 1000, 1000, 1000, 0);

    let result =
        MindstoneExpectedInformationGain::evaluate(base, &[outcome(1, 0)], policy(), budget(3))
            .unwrap();

    assert_eq!(
        result.profile().expected_information_gain_signal().value(),
        1000
    );

    assert_eq!(
        result.decision().class(),
        CognitiveAdmissionClass::Deliberate
    );

    assert_eq!(result.decision().requested_units(), 8);

    assert_eq!(result.decision().granted_units(), 3);

    assert!(result.decision().is_budget_limited());
}

#[test]
fn expected_information_gain_is_deterministic_non_mutating_and_facade_composable() {
    let base = profile(300, 800, 200, 400, 900);

    let base_before = base;

    let outcomes = vec![outcome(2, 200), outcome(3, 500), outcome(5, 700)];

    let outcomes_before = outcomes.clone();

    let estimate = ExpectedInformationGain::estimate(base.uncertainty(), &outcomes).unwrap();

    let direct_profile = base.with_information_gain(estimate.information_gain());

    let direct_decision = policy().admit(direct_profile, budget(5));

    let first =
        MindstoneExpectedInformationGain::evaluate(base, &outcomes, policy(), budget(5)).unwrap();

    let repeated =
        MindstoneExpectedInformationGain::evaluate(base, &outcomes, policy(), budget(5)).unwrap();

    assert_eq!(first, repeated);

    assert_eq!(first.estimate(), &estimate);

    assert_eq!(first.profile(), direct_profile);

    assert_eq!(first.decision(), direct_decision);

    assert_eq!(base, base_before);

    assert_eq!(outcomes, outcomes_before);
}
