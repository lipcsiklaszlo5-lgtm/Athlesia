use athlesia_mindstone_sparse_cognition::{
    CognitiveCandidate, CognitiveFingerprint, CognitiveForgettingPolicy,
    CognitiveMemoryMaintenance, CognitiveMemoryMaintenanceAction, CognitiveMemoryMaintenanceStatus,
    CognitiveMemoryTier, CognitiveSignal, HierarchicalMemoryAdmission, HierarchicalMemoryPolicy,
    HierarchicalMemoryState, MindstoneForgettingColdStorage, MindstoneSignalProfile,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn salience(value: u16) -> athlesia_mindstone_sparse_cognition::CognitiveSalience {
    MindstoneSignalProfile::new(
        signal(value),
        signal(value),
        signal(value),
        signal(value),
        signal(value),
    )
    .salience()
}

fn candidate(id: u64, salience_value: u16, support: u64) -> CognitiveCandidate {
    CognitiveCandidate::new(
        CognitiveFingerprint::new(id),
        salience(salience_value),
        support,
        1,
    )
    .unwrap()
}

fn admission_policy() -> HierarchicalMemoryPolicy {
    HierarchicalMemoryPolicy::new(signal(400), signal(700), 2, 3, 2, 2, 2).unwrap()
}

fn admission_policy_with_capacities(
    active: usize,
    consolidated: usize,
    cold: usize,
) -> HierarchicalMemoryPolicy {
    HierarchicalMemoryPolicy::new(signal(400), signal(700), 2, 3, active, consolidated, cold)
        .unwrap()
}

fn forgetting_policy() -> CognitiveForgettingPolicy {
    CognitiveForgettingPolicy::new(5, 10, 20, 3, signal(950)).unwrap()
}

#[test]
fn forgetting_policy_requires_ordered_positive_temporal_thresholds_and_protection() {
    assert_eq!(
        CognitiveForgettingPolicy::new(0, 10, 20, 3, signal(950,),),
        None
    );

    assert_eq!(
        CognitiveForgettingPolicy::new(5, 5, 20, 3, signal(950,),),
        None
    );

    assert_eq!(
        CognitiveForgettingPolicy::new(5, 10, 10, 3, signal(950,),),
        None
    );

    assert_eq!(
        CognitiveForgettingPolicy::new(5, 10, 20, 0, signal(950,),),
        None
    );

    let policy = forgetting_policy();

    assert_eq!(policy.active_cool_after(), 5);

    assert_eq!(policy.consolidated_cool_after(), 10);

    assert_eq!(policy.cold_forget_after(), 20);
}

#[test]
fn memory_younger_than_threshold_is_retained_in_place() {
    let admitted = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(1, 800, 1),
        admission_policy(),
    );

    let maintained = CognitiveMemoryMaintenance::maintain(
        admitted.state_after().clone(),
        4,
        admission_policy(),
        forgetting_policy(),
    );

    assert_eq!(
        maintained.status(),
        CognitiveMemoryMaintenanceStatus::Maintained
    );

    assert!(maintained.accepted());

    assert!(!maintained.changed());

    assert_eq!(
        maintained
            .state_after()
            .tier_of(CognitiveFingerprint::new(1,),),
        Some(CognitiveMemoryTier::Active,)
    );

    assert_eq!(maintained.state_after().last_event_index(), Some(4,));
}

#[test]
fn stale_unprotected_active_memory_cools_to_consolidated() {
    let admitted = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(1, 800, 1),
        admission_policy(),
    );

    let maintained = CognitiveMemoryMaintenance::maintain(
        admitted.state_after().clone(),
        6,
        admission_policy(),
        forgetting_policy(),
    );

    assert_eq!(maintained.cooled_count(), 1);

    assert_eq!(maintained.forgotten_count(), 0);

    assert_eq!(
        maintained
            .state_after()
            .tier_of(CognitiveFingerprint::new(1,),),
        Some(CognitiveMemoryTier::Consolidated,)
    );

    assert_eq!(
        maintained.actions(),
        &[CognitiveMemoryMaintenanceAction::Cooled {
            fingerprint: CognitiveFingerprint::new(1,),
            from: CognitiveMemoryTier::Active,
            to: CognitiveMemoryTier::Consolidated,
        },]
    );
}

#[test]
fn stale_unprotected_consolidated_memory_cools_to_cold() {
    let admitted = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(2, 500, 3),
        admission_policy(),
    );

    let maintained = CognitiveMemoryMaintenance::maintain(
        admitted.state_after().clone(),
        11,
        admission_policy(),
        forgetting_policy(),
    );

    assert_eq!(
        maintained
            .state_after()
            .tier_of(CognitiveFingerprint::new(2,),),
        Some(CognitiveMemoryTier::Cold,)
    );

    assert_eq!(maintained.cooled_count(), 1);
}

#[test]
fn stale_unprotected_cold_memory_is_explicitly_forgotten() {
    let admitted = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(3, 100, 2),
        admission_policy(),
    );

    let maintained = CognitiveMemoryMaintenance::maintain(
        admitted.state_after().clone(),
        21,
        admission_policy(),
        forgetting_policy(),
    );

    assert_eq!(maintained.forgotten_count(), 1);

    assert_eq!(
        maintained
            .state_after()
            .tier_of(CognitiveFingerprint::new(3,),),
        None
    );

    assert_eq!(maintained.state_after().total_len(), 0);

    assert_eq!(
        maintained.actions(),
        &[CognitiveMemoryMaintenanceAction::Forgotten {
            fingerprint: CognitiveFingerprint::new(3,),
        },]
    );
}

#[test]
fn one_maintenance_cycle_performs_at_most_one_cooling_step_per_record() {
    let admitted = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(4, 800, 1),
        admission_policy(),
    );

    let maintained = CognitiveMemoryMaintenance::maintain(
        admitted.state_after().clone(),
        100,
        admission_policy(),
        forgetting_policy(),
    );

    assert_eq!(
        maintained
            .state_after()
            .tier_of(CognitiveFingerprint::new(4,),),
        Some(CognitiveMemoryTier::Consolidated,)
    );

    assert_eq!(maintained.cooled_count(), 1);

    assert_eq!(maintained.forgotten_count(), 0);
}

#[test]
fn high_salience_memory_is_protected_from_time_only_cooling() {
    let admitted = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(5, 1000, 1),
        admission_policy(),
    );

    let record = admitted.record().unwrap();

    assert!(forgetting_policy().protects(record,));

    let maintained = CognitiveMemoryMaintenance::maintain(
        admitted.state_after().clone(),
        10_000,
        admission_policy(),
        forgetting_policy(),
    );

    assert!(!maintained.changed());

    assert_eq!(
        maintained
            .state_after()
            .tier_of(CognitiveFingerprint::new(5,),),
        Some(CognitiveMemoryTier::Active,)
    );
}

#[test]
fn frequently_readmitted_memory_is_protected_even_at_low_salience() {
    let first = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(6, 100, 2),
        admission_policy(),
    );

    let second = HierarchicalMemoryAdmission::admit(
        first.state_after().clone(),
        2,
        candidate(6, 100, 2),
        admission_policy(),
    );

    let third = HierarchicalMemoryAdmission::admit(
        second.state_after().clone(),
        3,
        candidate(6, 100, 2),
        admission_policy(),
    );

    let record = third.record().unwrap();

    assert_eq!(record.admission_count(), 3);

    assert!(forgetting_policy().protects(record,));

    let maintained = CognitiveMemoryMaintenance::maintain(
        third.state_after().clone(),
        100,
        admission_policy(),
        forgetting_policy(),
    );

    assert!(!maintained.changed());

    assert_eq!(
        maintained
            .state_after()
            .tier_of(CognitiveFingerprint::new(6,),),
        Some(CognitiveMemoryTier::Cold,)
    );
}

#[test]
fn low_use_unprotected_memory_can_progress_active_to_consolidated_to_cold_to_forgotten() {
    let admitted = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(7, 800, 1),
        admission_policy(),
    );

    let cooled_once = CognitiveMemoryMaintenance::maintain(
        admitted.state_after().clone(),
        6,
        admission_policy(),
        forgetting_policy(),
    );

    assert_eq!(
        cooled_once
            .state_after()
            .tier_of(CognitiveFingerprint::new(7,),),
        Some(CognitiveMemoryTier::Consolidated,)
    );

    let cooled_twice = CognitiveMemoryMaintenance::maintain(
        cooled_once.state_after().clone(),
        11,
        admission_policy(),
        forgetting_policy(),
    );

    assert_eq!(
        cooled_twice
            .state_after()
            .tier_of(CognitiveFingerprint::new(7,),),
        Some(CognitiveMemoryTier::Cold,)
    );

    let forgotten = CognitiveMemoryMaintenance::maintain(
        cooled_twice.state_after().clone(),
        21,
        admission_policy(),
        forgetting_policy(),
    );

    assert_eq!(
        forgotten
            .state_after()
            .tier_of(CognitiveFingerprint::new(7,),),
        None
    );

    assert_eq!(forgotten.forgotten_count(), 1);
}

#[test]
fn cooling_into_full_tier_preserves_bound_with_deterministic_capacity_eviction() {
    let memory_policy = admission_policy_with_capacities(2, 1, 2);

    let consolidated = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(10, 500, 3),
        memory_policy,
    );

    let active = HierarchicalMemoryAdmission::admit(
        consolidated.state_after().clone(),
        2,
        candidate(20, 800, 1),
        memory_policy,
    );

    let maintained = CognitiveMemoryMaintenance::maintain(
        active.state_after().clone(),
        7,
        memory_policy,
        forgetting_policy(),
    );

    assert_eq!(maintained.state_after().consolidated_len(), 1);

    assert_eq!(
        maintained
            .state_after()
            .tier_of(CognitiveFingerprint::new(20,),),
        Some(CognitiveMemoryTier::Consolidated,)
    );

    assert_eq!(
        maintained
            .state_after()
            .tier_of(CognitiveFingerprint::new(10,),),
        None
    );

    assert_eq!(maintained.capacity_eviction_count(), 1);

    assert_eq!(maintained.cooled_count(), 1);

    assert_eq!(
        maintained.actions()[0],
        CognitiveMemoryMaintenanceAction::CapacityEvicted {
            fingerprint: CognitiveFingerprint::new(10,),
            tier: CognitiveMemoryTier::Consolidated,
        }
    );

    assert_eq!(
        maintained.actions()[1],
        CognitiveMemoryMaintenanceAction::Cooled {
            fingerprint: CognitiveFingerprint::new(20,),
            from: CognitiveMemoryTier::Active,
            to: CognitiveMemoryTier::Consolidated,
        }
    );
}

#[test]
fn non_monotonic_maintenance_is_rejected_without_memory_mutation() {
    let admitted = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        10,
        candidate(30, 800, 1),
        admission_policy(),
    );

    let before = admitted.state_after().clone();

    let rejected = CognitiveMemoryMaintenance::maintain(
        before.clone(),
        10,
        admission_policy(),
        forgetting_policy(),
    );

    assert_eq!(
        rejected.status(),
        CognitiveMemoryMaintenanceStatus::RejectedOutOfOrder
    );

    assert!(!rejected.accepted());

    assert!(!rejected.changed());

    assert_eq!(rejected.state_before(), &before);

    assert_eq!(rejected.state_after(), &before);
}

#[test]
fn forgetting_cold_storage_is_deterministic_non_mutating_and_facade_equivalent() {
    let first = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(40, 800, 1),
        admission_policy(),
    );

    let second = HierarchicalMemoryAdmission::admit(
        first.state_after().clone(),
        2,
        candidate(50, 100, 2),
        admission_policy(),
    );

    let initial = second.state_after().clone();

    let before = initial.clone();

    let memory_policy = admission_policy();

    let retention_policy = forgetting_policy();

    let direct =
        CognitiveMemoryMaintenance::maintain(initial.clone(), 30, memory_policy, retention_policy);

    let facade = MindstoneForgettingColdStorage::evaluate(
        initial.clone(),
        30,
        memory_policy,
        retention_policy,
    );

    let repeated = MindstoneForgettingColdStorage::evaluate(
        initial.clone(),
        30,
        memory_policy,
        retention_policy,
    );

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(initial, before);

    assert_eq!(facade.cooled_count(), 1);

    assert_eq!(facade.forgotten_count(), 1);

    assert_eq!(facade.state_after().total_len(), 1);

    assert_eq!(
        facade
            .state_after()
            .tier_of(CognitiveFingerprint::new(40,),),
        Some(CognitiveMemoryTier::Consolidated,)
    );

    assert_eq!(
        facade
            .state_after()
            .tier_of(CognitiveFingerprint::new(50,),),
        None
    );
}
