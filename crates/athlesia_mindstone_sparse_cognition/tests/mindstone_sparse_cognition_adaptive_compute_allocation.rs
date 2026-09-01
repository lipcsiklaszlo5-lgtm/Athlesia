use athlesia_mindstone_sparse_cognition::{
    AdaptiveComputeAllocator, AdaptiveComputeSignals, CognitiveBudget, CognitiveSignal,
    MindstoneAdaptiveComputeAllocation, MindstoneExtendedSignalProfile, MindstoneSignalProfile,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn budget(units: u32) -> CognitiveBudget {
    CognitiveBudget::new(units).unwrap()
}

fn signals(
    surprise: u16,
    learning_progress: u16,
    information_gain: u16,
    controllability: u16,
) -> AdaptiveComputeSignals {
    AdaptiveComputeSignals::new(
        signal(surprise),
        signal(learning_progress),
        signal(information_gain),
        signal(controllability),
    )
}

fn profile(
    surprise: u16,
    learning_progress: u16,
    information_gain: u16,
    controllability: u16,
) -> MindstoneExtendedSignalProfile {
    MindstoneExtendedSignalProfile::new(
        MindstoneSignalProfile::new(
            signal(surprise),
            signal(500),
            signal(0),
            signal(learning_progress),
            signal(information_gain),
        ),
        signal(0),
        signal(controllability),
    )
}

#[test]
fn adaptive_compute_signals_preserve_four_domain_neutral_pressure_axes() {
    let extracted = AdaptiveComputeSignals::from_profile(profile(100, 300, 700, 900));

    assert_eq!(extracted.surprise().value(), 100);

    assert_eq!(extracted.learning_progress().value(), 300);

    assert_eq!(extracted.information_gain().value(), 700);

    assert_eq!(extracted.controllability().value(), 900);

    assert_eq!(extracted.goal_pressure().value(), 900);

    assert_eq!(extracted.hypothesis_pressure().value(), 300);

    assert_eq!(extracted.overall_pressure().value(), 900);
}

#[test]
fn reservation_above_hard_cap_is_rejected() {
    assert_eq!(
        AdaptiveComputeAllocator::allocate(signals(1000, 1000, 1000, 1000,), budget(10,), 11,),
        None
    );

    assert!(
        AdaptiveComputeAllocator::allocate(signals(1000, 1000, 1000, 1000,), budget(10,), 10,)
            .is_some()
    );
}

#[test]
fn zero_epistemic_pressure_leaves_all_unreserved_compute_unused() {
    let allocation =
        AdaptiveComputeAllocator::allocate(signals(0, 0, 0, 0), budget(100), 20).unwrap();

    assert_eq!(allocation.available_units(), 80);

    assert_eq!(allocation.activated_units(), 0);

    assert_eq!(allocation.goal_units(), 0);

    assert_eq!(allocation.hypothesis_units(), 0);

    assert_eq!(allocation.unused_units(), 80);

    assert!(allocation.is_idle());
}

#[test]
fn full_goal_pressure_assigns_all_active_compute_to_goal_frontier() {
    let allocation =
        AdaptiveComputeAllocator::allocate(signals(0, 0, 1000, 0), budget(100), 10).unwrap();

    assert_eq!(allocation.available_units(), 90);

    assert_eq!(allocation.activated_units(), 90);

    assert_eq!(allocation.goal_units(), 90);

    assert_eq!(allocation.hypothesis_units(), 0);

    assert_eq!(allocation.unused_units(), 0);
}

#[test]
fn full_hypothesis_pressure_assigns_all_active_compute_to_search() {
    let allocation =
        AdaptiveComputeAllocator::allocate(signals(1000, 0, 0, 0), budget(100), 10).unwrap();

    assert_eq!(allocation.activated_units(), 90);

    assert_eq!(allocation.goal_units(), 0);

    assert_eq!(allocation.hypothesis_units(), 90);
}

#[test]
fn intermediate_pressure_scales_total_active_compute_below_hard_cap() {
    let allocation =
        AdaptiveComputeAllocator::allocate(signals(500, 0, 0, 0), budget(100), 20).unwrap();

    assert_eq!(allocation.available_units(), 80);

    assert_eq!(allocation.overall_pressure().value(), 500);

    assert_eq!(allocation.activated_units(), 40);

    assert_eq!(allocation.hypothesis_units(), 40);

    assert_eq!(allocation.unused_units(), 40);
}

#[test]
fn mixed_pressure_splits_active_compute_proportionally_and_conserves_units() {
    let allocation =
        AdaptiveComputeAllocator::allocate(signals(250, 0, 750, 0), budget(100), 0).unwrap();

    assert_eq!(allocation.overall_pressure().value(), 750);

    assert_eq!(allocation.activated_units(), 75);

    assert_eq!(allocation.goal_units(), 56);

    assert_eq!(allocation.hypothesis_units(), 19);

    assert_eq!(
        allocation.goal_units() + allocation.hypothesis_units(),
        allocation.activated_units()
    );

    assert_eq!(allocation.total_accounted_units(), 100);
}

#[test]
fn stronger_expected_information_gain_shifts_compute_toward_goals() {
    let low_gain =
        AdaptiveComputeAllocator::allocate(signals(500, 0, 100, 0), budget(100), 0).unwrap();

    let high_gain =
        AdaptiveComputeAllocator::allocate(signals(500, 0, 900, 0), budget(100), 0).unwrap();

    assert!(high_gain.goal_units() > low_gain.goal_units());

    assert!(high_gain.goal_pressure() > low_gain.goal_pressure());
}

#[test]
fn stronger_surprise_or_learning_progress_shifts_compute_toward_hypothesis_search() {
    let low_search_pressure =
        AdaptiveComputeAllocator::allocate(signals(100, 100, 500, 0), budget(100), 0).unwrap();

    let high_search_pressure =
        AdaptiveComputeAllocator::allocate(signals(900, 800, 500, 0), budget(100), 0).unwrap();

    assert!(high_search_pressure.hypothesis_units() > low_search_pressure.hypothesis_units());

    assert_eq!(high_search_pressure.hypothesis_pressure().value(), 900);
}

#[test]
fn reserved_admission_compute_reduces_available_pool_without_exceeding_hard_cap() {
    let allocation =
        AdaptiveComputeAllocator::allocate(signals(1000, 1000, 1000, 1000), budget(64), 16)
            .unwrap();

    assert_eq!(allocation.hard_cap_units(), 64);

    assert_eq!(allocation.reserved_units(), 16);

    assert_eq!(allocation.available_units(), 48);

    assert_eq!(allocation.activated_units(), 48);

    assert_eq!(allocation.total_accounted_units(), 64);

    assert!(
        allocation.goal_units() + allocation.hypothesis_units() <= allocation.available_units()
    );
}

#[test]
fn adaptive_allocation_is_overflow_safe_at_maximum_u32_budget() {
    let allocation =
        AdaptiveComputeAllocator::allocate(signals(1000, 1000, 1000, 1000), budget(u32::MAX), 1)
            .unwrap();

    assert_eq!(allocation.hard_cap_units(), u32::MAX);

    assert_eq!(allocation.available_units(), u32::MAX - 1);

    assert_eq!(allocation.activated_units(), u32::MAX - 1);

    assert_eq!(allocation.total_accounted_units(), u32::MAX);

    assert!(allocation.goal_units() <= allocation.available_units());

    assert!(allocation.hypothesis_units() <= allocation.available_units());
}

#[test]
fn adaptive_compute_allocation_is_deterministic_non_mutating_and_facade_equivalent() {
    let input_profile = profile(700, 400, 800, 600);

    let profile_before = input_profile;

    let hard_budget = budget(100);

    let direct = AdaptiveComputeAllocator::allocate(
        AdaptiveComputeSignals::from_profile(input_profile),
        hard_budget,
        20,
    )
    .unwrap();

    let facade =
        MindstoneAdaptiveComputeAllocation::evaluate(input_profile, hard_budget, 20).unwrap();

    let repeated =
        MindstoneAdaptiveComputeAllocation::evaluate(input_profile, hard_budget, 20).unwrap();

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(input_profile, profile_before);

    assert_eq!(facade.total_accounted_units(), hard_budget.units());

    assert!(
        facade.goal_units() + facade.hypothesis_units() + facade.reserved_units()
            <= hard_budget.units()
    );
}
