use athlesia_mindstone_sparse_cognition::{
    CognitiveAdmissionClass, CognitiveBudget, CognitiveSignal, MindstoneSignalProfile,
    MindstoneSparseCognition, SparseCognitionPolicy, COGNITIVE_SIGNAL_SCALE,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
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
fn cognitive_signal_is_bounded_and_canonical() {
    assert_eq!(CognitiveSignal::new(0,).unwrap().value(), 0);

    assert_eq!(
        CognitiveSignal::new(COGNITIVE_SIGNAL_SCALE,)
            .unwrap()
            .value(),
        COGNITIVE_SIGNAL_SCALE
    );

    assert_eq!(CognitiveSignal::new(COGNITIVE_SIGNAL_SCALE + 1,), None);
}

#[test]
fn learning_progress_rewards_uncertainty_reduction_only() {
    assert_eq!(
        CognitiveSignal::learning_progress(signal(800,), signal(300,),).value(),
        500
    );

    assert_eq!(
        CognitiveSignal::learning_progress(signal(300,), signal(800,),).value(),
        0
    );

    assert_eq!(
        CognitiveSignal::learning_progress(signal(500,), signal(500,),).value(),
        0
    );
}

#[test]
fn zero_epistemic_signal_has_zero_salience() {
    assert_eq!(profile(0, 0, 0, 0, 0,).salience().value(), 0);
}

#[test]
fn isolated_strong_signal_survives_mean_dilution() {
    let value = profile(1000, 0, 0, 0, 0).salience().value();

    assert_eq!(value, 428);

    assert!(value > 200);
}

#[test]
fn salience_is_domain_neutral_across_epistemic_signal_positions() {
    let first = profile(900, 100, 300, 500, 700);

    let second = profile(300, 700, 100, 900, 500);

    assert_eq!(first.salience(), second.salience());
}

#[test]
fn sparse_policy_rejects_invalid_thresholds_and_compute_shape() {
    assert_eq!(
        SparseCognitionPolicy::new(signal(600,), signal(600,), 2, 8,),
        None
    );

    assert_eq!(
        SparseCognitionPolicy::new(signal(700,), signal(600,), 2, 8,),
        None
    );

    assert_eq!(
        SparseCognitionPolicy::new(signal(200,), signal(600,), 0, 8,),
        None
    );

    assert_eq!(
        SparseCognitionPolicy::new(signal(200,), signal(600,), 8, 2,),
        None
    );
}

#[test]
fn predictable_low_salience_input_is_ignored_without_compute_request() {
    let decision = policy().admit(profile(100, 100, 100, 100, 100), budget(100));

    assert_eq!(decision.class(), CognitiveAdmissionClass::Ignore);

    assert_eq!(decision.requested_units(), 0);

    assert_eq!(decision.granted_units(), 0);

    assert!(!decision.is_admitted());
}

#[test]
fn medium_salience_input_receives_only_cheap_update_budget() {
    let decision = policy().admit(profile(400, 400, 400, 400, 400), budget(100));

    assert_eq!(decision.class(), CognitiveAdmissionClass::CheapUpdate);

    assert_eq!(decision.requested_units(), 2);

    assert_eq!(decision.granted_units(), 2);

    assert!(decision.is_admitted());

    assert!(!decision.is_deliberative());
}

#[test]
fn high_salience_input_receives_deliberative_compute_request() {
    let decision = policy().admit(profile(800, 800, 800, 800, 800), budget(100));

    assert_eq!(decision.class(), CognitiveAdmissionClass::Deliberate);

    assert_eq!(decision.requested_units(), 8);

    assert_eq!(decision.granted_units(), 8);

    assert!(decision.is_deliberative());
}

#[test]
fn hard_compute_budget_caps_deliberation_without_changing_salience() {
    let input = profile(800, 800, 800, 800, 800);

    let unrestricted = policy().admit(input, budget(100));

    let constrained = policy().admit(input, budget(3));

    assert_eq!(constrained.class(), CognitiveAdmissionClass::Deliberate);

    assert_eq!(constrained.salience(), unrestricted.salience());

    assert_eq!(constrained.requested_units(), 8);

    assert_eq!(constrained.granted_units(), 3);

    assert!(constrained.is_budget_limited());
}

#[test]
fn cognitive_budget_rejects_zero_and_preserves_positive_capacity() {
    assert_eq!(CognitiveBudget::new(0,), None);

    assert_eq!(CognitiveBudget::new(17,).unwrap().units(), 17);
}

#[test]
fn mindstone_foundation_is_deterministic_non_mutating_and_facade_equivalent() {
    let input = profile(810, 620, 740, 330, 900);

    let input_before = input;

    let policy = policy();

    let policy_before = policy;

    let budget = budget(5);

    let budget_before = budget;

    let direct = policy.admit(input, budget);

    let facade = MindstoneSparseCognition::evaluate(input, policy, budget);

    let repeated = MindstoneSparseCognition::evaluate(input, policy, budget);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(input, input_before);

    assert_eq!(policy, policy_before);

    assert_eq!(budget, budget_before);
}
