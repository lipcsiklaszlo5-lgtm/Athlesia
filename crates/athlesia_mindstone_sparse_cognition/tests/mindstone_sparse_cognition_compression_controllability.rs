use athlesia_mindstone_sparse_cognition::{
    CognitiveAdmissionClass, CognitiveBudget, CognitiveSignal, MindstoneCompressionControllability,
    MindstoneExtendedSignalProfile, MindstoneSignalProfile, SparseCognitionPolicy,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn profile(value: u16) -> MindstoneSignalProfile {
    MindstoneSignalProfile::new(
        signal(value),
        signal(value),
        signal(value),
        signal(value),
        signal(value),
    )
}

fn policy() -> SparseCognitionPolicy {
    SparseCognitionPolicy::new(signal(200), signal(600), 2, 8).unwrap()
}

fn budget(units: u32) -> CognitiveBudget {
    CognitiveBudget::new(units).unwrap()
}

#[test]
fn compression_gain_requires_nonzero_original_size() {
    assert_eq!(CognitiveSignal::compression_gain(0, 0,), None);

    assert_eq!(CognitiveSignal::compression_gain(0, 100,), None);
}

#[test]
fn compression_gain_is_zero_when_representation_does_not_shrink() {
    assert_eq!(
        CognitiveSignal::compression_gain(100, 100,).unwrap(),
        CognitiveSignal::zero()
    );

    assert_eq!(
        CognitiveSignal::compression_gain(100, 120,).unwrap(),
        CognitiveSignal::zero()
    );
}

#[test]
fn compression_gain_measures_fractional_reduction_on_fixed_scale() {
    assert_eq!(
        CognitiveSignal::compression_gain(100, 75,).unwrap().value(),
        250
    );

    assert_eq!(
        CognitiveSignal::compression_gain(100, 50,).unwrap().value(),
        500
    );

    assert_eq!(
        CognitiveSignal::compression_gain(100, 0,).unwrap(),
        CognitiveSignal::maximum()
    );
}

#[test]
fn controllability_requires_valid_nonempty_intervention_evidence() {
    assert_eq!(CognitiveSignal::controllability(0, 0,), None);

    assert_eq!(CognitiveSignal::controllability(4, 3,), None);
}

#[test]
fn controllability_maps_empirical_success_ratio_to_fixed_scale() {
    assert_eq!(CognitiveSignal::controllability(0, 4,).unwrap().value(), 0);

    assert_eq!(
        CognitiveSignal::controllability(3, 4,).unwrap().value(),
        750
    );

    assert_eq!(
        CognitiveSignal::controllability(4, 4,).unwrap().value(),
        1000
    );
}

#[test]
fn zero_meta_signals_preserve_existing_base_salience_exactly() {
    let base = profile(400);

    let extended =
        MindstoneExtendedSignalProfile::new(base, CognitiveSignal::zero(), CognitiveSignal::zero());

    assert_eq!(extended.salience(), base.salience());

    assert_eq!(extended.meta_salience().value(), 0);
}

#[test]
fn compression_and_controllability_are_preserved_as_distinct_signals() {
    let extended = MindstoneExtendedSignalProfile::new(profile(100), signal(250), signal(750));

    assert_eq!(extended.compression_gain().value(), 250);

    assert_eq!(extended.controllability().value(), 750);

    assert_eq!(extended.base(), profile(100,));
}

#[test]
fn meta_salience_is_domain_neutral_across_signal_positions() {
    let first = MindstoneExtendedSignalProfile::new(profile(0), signal(250), signal(750));

    let second = MindstoneExtendedSignalProfile::new(profile(0), signal(750), signal(250));

    assert_eq!(first.meta_salience(), second.meta_salience());

    assert_eq!(first.salience(), second.salience());
}

#[test]
fn strong_compression_or_control_signal_can_raise_cognitive_priority() {
    let base = profile(100);

    assert_eq!(policy().classify(base,), CognitiveAdmissionClass::Ignore);

    let extended = MindstoneExtendedSignalProfile::new(
        base,
        CognitiveSignal::maximum(),
        CognitiveSignal::zero(),
    );

    assert_eq!(
        policy().classify_extended(extended,),
        CognitiveAdmissionClass::Deliberate
    );

    assert!(extended.salience() > base.salience());
}

#[test]
fn complete_meta_signal_can_raise_extended_salience_to_maximum() {
    let extended = MindstoneExtendedSignalProfile::new(
        profile(100),
        CognitiveSignal::maximum(),
        CognitiveSignal::maximum(),
    );

    assert_eq!(extended.meta_salience().value(), 1000);

    assert_eq!(extended.salience().value(), 1000);
}

#[test]
fn extended_admission_obeys_existing_hard_compute_budget() {
    let extended = MindstoneExtendedSignalProfile::new(
        profile(100),
        CognitiveSignal::maximum(),
        CognitiveSignal::maximum(),
    );

    let decision = policy().admit_extended(extended, budget(3));

    assert_eq!(decision.class(), CognitiveAdmissionClass::Deliberate);

    assert_eq!(decision.requested_units(), 8);

    assert_eq!(decision.granted_units(), 3);

    assert!(decision.is_budget_limited());
}

#[test]
fn compression_controllability_facade_is_deterministic_non_mutating_and_composable() {
    let base = profile(100);

    let base_before = base;

    let search_policy = policy();

    let compute_budget = budget(5);

    let first = MindstoneCompressionControllability::evaluate(
        base,
        100,
        50,
        3,
        4,
        search_policy,
        compute_budget,
    )
    .unwrap();

    let repeated = MindstoneCompressionControllability::evaluate(
        base,
        100,
        50,
        3,
        4,
        search_policy,
        compute_budget,
    )
    .unwrap();

    let compression = CognitiveSignal::compression_gain(100, 50).unwrap();

    let controllability = CognitiveSignal::controllability(3, 4).unwrap();

    let extended = MindstoneExtendedSignalProfile::new(base, compression, controllability);

    let direct = search_policy.admit_extended(extended, compute_budget);

    assert_eq!(first, repeated);

    assert_eq!(first.base_profile(), base);

    assert_eq!(first.compression_gain(), compression);

    assert_eq!(first.controllability(), controllability);

    assert_eq!(first.extended_profile(), extended);

    assert_eq!(first.decision(), direct);

    assert_eq!(base, base_before);

    assert!(first.decision().granted_units() <= compute_budget.units());
}
