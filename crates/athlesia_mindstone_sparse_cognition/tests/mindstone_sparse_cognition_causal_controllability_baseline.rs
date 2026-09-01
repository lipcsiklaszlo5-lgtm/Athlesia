use athlesia_mindstone_sparse_cognition::{
    AdaptiveComputeAllocator, AdaptiveComputeSignals, CausalControllabilityEstimator,
    CausalControllabilityEvidence, CognitiveBudget, CognitiveSignal,
    MindstoneCausalControllability, MindstoneExtendedSignalProfile, MindstoneSignalProfile,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn evidence(
    intervention_successes: u64,
    intervention_attempts: u64,
    passive_successes: u64,
    passive_attempts: u64,
) -> CausalControllabilityEvidence {
    CausalControllabilityEvidence::new(
        intervention_successes,
        intervention_attempts,
        passive_successes,
        passive_attempts,
    )
    .unwrap()
}

fn profile(
    surprise: u16,
    learning_progress: u16,
    information_gain: u16,
    compression_gain: u16,
    raw_controllability: u16,
) -> MindstoneExtendedSignalProfile {
    MindstoneExtendedSignalProfile::new(
        MindstoneSignalProfile::new(
            signal(surprise),
            signal(500),
            signal(0),
            signal(learning_progress),
            signal(information_gain),
        ),
        signal(compression_gain),
        signal(raw_controllability),
    )
}

fn budget(units: u32) -> CognitiveBudget {
    CognitiveBudget::new(units).unwrap()
}

#[test]
fn causal_controllability_evidence_requires_two_valid_nonempty_comparison_groups() {
    assert_eq!(CausalControllabilityEvidence::new(0, 0, 0, 10,), None);

    assert_eq!(CausalControllabilityEvidence::new(0, 10, 0, 0,), None);

    assert_eq!(CausalControllabilityEvidence::new(11, 10, 0, 10,), None);

    assert_eq!(CausalControllabilityEvidence::new(5, 10, 11, 10,), None);

    let valid = evidence(9, 10, 3, 10);

    assert_eq!(valid.intervention_successes(), 9);

    assert_eq!(valid.passive_attempts(), 10);
}

#[test]
fn causal_controllability_is_positive_intervention_lift_over_passive_baseline() {
    let estimate = CausalControllabilityEstimator::estimate(evidence(9, 10, 3, 10));

    assert_eq!(estimate.intervention_rate().value(), 900);

    assert_eq!(estimate.passive_rate().value(), 300);

    assert_eq!(estimate.causal_lift().value(), 600);

    assert!(estimate.has_positive_causal_lift());
}

#[test]
fn equal_intervention_and_passive_success_rates_have_zero_causal_control() {
    let estimate = CausalControllabilityEstimator::estimate(evidence(9, 10, 90, 100));

    assert_eq!(estimate.intervention_rate().value(), 900);

    assert_eq!(estimate.passive_rate().value(), 900);

    assert_eq!(estimate.causal_lift(), CognitiveSignal::zero());

    assert!(!estimate.has_positive_causal_lift());
}

#[test]
fn intervention_underperforming_passive_baseline_clamps_causal_control_to_zero() {
    let estimate = CausalControllabilityEstimator::estimate(evidence(3, 10, 8, 10));

    assert_eq!(estimate.intervention_rate().value(), 300);

    assert_eq!(estimate.passive_rate().value(), 800);

    assert_eq!(estimate.causal_lift(), CognitiveSignal::zero());
}

#[test]
fn zero_passive_success_recovers_intervention_success_rate_as_causal_lift() {
    let estimate = CausalControllabilityEstimator::estimate(evidence(7, 10, 0, 10));

    assert_eq!(estimate.intervention_rate().value(), 700);

    assert_eq!(estimate.passive_rate(), CognitiveSignal::zero());

    assert_eq!(estimate.causal_lift().value(), 700);
}

#[test]
fn proportional_evidence_rescaling_preserves_causal_controllability() {
    let small = CausalControllabilityEstimator::estimate(evidence(9, 10, 3, 10));

    let large = CausalControllabilityEstimator::estimate(evidence(90, 100, 30, 100));

    assert_eq!(small, large);

    assert_eq!(small.causal_lift().value(), 600);
}

#[test]
fn causal_controllability_uses_deterministic_fixed_point_empirical_rates() {
    let estimate = CausalControllabilityEstimator::estimate(evidence(2, 3, 1, 3));

    assert_eq!(estimate.intervention_rate().value(), 666);

    assert_eq!(estimate.passive_rate().value(), 333);

    assert_eq!(estimate.causal_lift().value(), 333);
}

#[test]
fn causal_controllability_is_overflow_safe_for_maximum_u64_evidence() {
    let estimate =
        CausalControllabilityEstimator::estimate(evidence(u64::MAX, u64::MAX, 0, u64::MAX));

    assert_eq!(estimate.intervention_rate(), CognitiveSignal::maximum());

    assert_eq!(estimate.passive_rate(), CognitiveSignal::zero());

    assert_eq!(estimate.causal_lift(), CognitiveSignal::maximum());
}

#[test]
fn cognitive_signal_causal_controllability_convenience_matches_estimator() {
    let direct = CognitiveSignal::causal_controllability(8, 10, 2, 10).unwrap();

    let estimate = CausalControllabilityEstimator::estimate(evidence(8, 10, 2, 10));

    assert_eq!(direct, estimate.causal_lift());

    assert_eq!(direct.value(), 600);

    assert_eq!(CognitiveSignal::causal_controllability(1, 0, 0, 10,), None);
}

#[test]
fn causal_estimate_replaces_raw_profile_controllability_without_changing_other_signals() {
    let base = profile(300, 400, 500, 600, 900);

    let before = base;

    let estimate = CausalControllabilityEstimator::estimate(evidence(9, 10, 3, 10));

    let corrected = base.with_causal_controllability(estimate);

    assert_eq!(corrected.base(), base.base());

    assert_eq!(corrected.compression_gain(), base.compression_gain());

    assert_eq!(corrected.controllability().value(), 600);

    assert_eq!(base.controllability().value(), 900);

    assert_eq!(base, before);
}

#[test]
fn adaptive_compute_uses_causal_lift_instead_of_spurious_raw_intervention_success() {
    let raw_profile = profile(0, 0, 0, 0, 900);

    let raw_signals = AdaptiveComputeSignals::from_profile(raw_profile);

    assert_eq!(raw_signals.goal_pressure().value(), 900);

    let corrected =
        MindstoneCausalControllability::evaluate(raw_profile, evidence(9, 10, 9, 10)).profile();

    assert_eq!(corrected.controllability(), CognitiveSignal::zero());

    let corrected_signals = AdaptiveComputeSignals::from_profile(corrected);

    assert_eq!(corrected_signals.goal_pressure(), CognitiveSignal::zero());

    let allocation = AdaptiveComputeAllocator::allocate(corrected_signals, budget(100), 0).unwrap();

    assert!(allocation.is_idle());

    assert_eq!(allocation.activated_units(), 0);
}

#[test]
fn causal_controllability_facade_is_deterministic_non_mutating_and_composable() {
    let input_profile = profile(200, 300, 400, 500, 950);

    let profile_before = input_profile;

    let input_evidence = evidence(8, 10, 3, 10);

    let evidence_before = input_evidence;

    let direct_estimate = CausalControllabilityEstimator::estimate(input_evidence);

    let first = MindstoneCausalControllability::evaluate(input_profile, input_evidence);

    let repeated = MindstoneCausalControllability::evaluate(input_profile, input_evidence);

    assert_eq!(first, repeated);

    assert_eq!(first.estimate(), direct_estimate);

    assert_eq!(first.profile().controllability().value(), 500);

    assert_eq!(first.evidence(), input_evidence);

    assert_eq!(input_profile, profile_before);

    assert_eq!(input_evidence, evidence_before);

    let adaptive = AdaptiveComputeSignals::from_profile(first.profile());

    assert_eq!(adaptive.controllability().value(), 500);
}
