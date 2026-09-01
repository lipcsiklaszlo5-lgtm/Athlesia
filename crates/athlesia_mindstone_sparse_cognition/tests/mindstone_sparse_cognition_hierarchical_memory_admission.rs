use athlesia_mindstone_sparse_cognition::{
    CognitiveCandidate, CognitiveFingerprint, CognitiveMemoryTier, CognitiveSignal,
    HierarchicalMemoryAdmission, HierarchicalMemoryAdmissionClass,
    HierarchicalMemoryAdmissionStatus, HierarchicalMemoryPolicy, HierarchicalMemoryState,
    MindstoneHierarchicalMemoryAdmission, MindstoneSignalProfile,
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

fn policy_with_capacities(
    active_capacity: usize,
    consolidated_capacity: usize,
    cold_capacity: usize,
) -> HierarchicalMemoryPolicy {
    HierarchicalMemoryPolicy::new(
        signal(400),
        signal(700),
        2,
        3,
        active_capacity,
        consolidated_capacity,
        cold_capacity,
    )
    .unwrap()
}

fn policy() -> HierarchicalMemoryPolicy {
    policy_with_capacities(2, 2, 2)
}

#[test]
fn hierarchical_policy_rejects_invalid_threshold_support_and_capacity_shape() {
    assert_eq!(
        HierarchicalMemoryPolicy::new(signal(700,), signal(700,), 2, 3, 1, 1, 1,),
        None
    );

    assert_eq!(
        HierarchicalMemoryPolicy::new(signal(800,), signal(700,), 2, 3, 1, 1, 1,),
        None
    );

    assert_eq!(
        HierarchicalMemoryPolicy::new(signal(400,), signal(700,), 0, 3, 1, 1, 1,),
        None
    );

    assert_eq!(
        HierarchicalMemoryPolicy::new(signal(400,), signal(700,), 4, 3, 1, 1, 1,),
        None
    );

    assert_eq!(
        HierarchicalMemoryPolicy::new(signal(400,), signal(700,), 2, 3, 0, 1, 1,),
        None
    );

    assert_eq!(policy().total_capacity(), 6);
}

#[test]
fn weak_singleton_candidate_is_discarded_without_retained_memory() {
    let result = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(1, 100, 1),
        policy(),
    );

    assert_eq!(result.class(), HierarchicalMemoryAdmissionClass::Discard);

    assert_eq!(
        result.status(),
        HierarchicalMemoryAdmissionStatus::Discarded
    );

    assert!(result.accepted());

    assert!(!result.retained());

    assert_eq!(result.state_after().total_len(), 0);

    assert_eq!(result.state_after().last_event_index(), Some(1,));
}

#[test]
fn repeated_low_salience_candidate_enters_cold_memory() {
    let result = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(1, 100, 2),
        policy(),
    );

    assert_eq!(result.status(), HierarchicalMemoryAdmissionStatus::Cold);

    assert_eq!(
        result.state_after().tier_of(CognitiveFingerprint::new(1,),),
        Some(CognitiveMemoryTier::Cold,)
    );

    assert_eq!(result.state_after().cold_len(), 1);
}

#[test]
fn supported_medium_salience_candidate_enters_consolidated_memory() {
    let result = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(2, 500, 3),
        policy(),
    );

    assert_eq!(
        result.status(),
        HierarchicalMemoryAdmissionStatus::Consolidated
    );

    assert_eq!(
        result.state_after().tier_of(CognitiveFingerprint::new(2,),),
        Some(CognitiveMemoryTier::Consolidated,)
    );
}

#[test]
fn high_salience_candidate_enters_active_memory_even_with_low_support() {
    let result = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(3, 800, 1),
        policy(),
    );

    assert_eq!(result.status(), HierarchicalMemoryAdmissionStatus::Active);

    assert_eq!(
        result.state_after().tier_of(CognitiveFingerprint::new(3,),),
        Some(CognitiveMemoryTier::Active,)
    );

    assert_eq!(result.state_after().active_len(), 1);
}

#[test]
fn same_identity_can_promote_from_cold_to_active_without_duplication() {
    let cold = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(10, 100, 2),
        policy(),
    );

    let promoted = HierarchicalMemoryAdmission::admit(
        cold.state_after().clone(),
        2,
        candidate(10, 900, 5),
        policy(),
    );

    assert_eq!(promoted.previous_tier(), Some(CognitiveMemoryTier::Cold,));

    assert_eq!(promoted.status(), HierarchicalMemoryAdmissionStatus::Active);

    assert_eq!(promoted.state_after().total_len(), 1);

    assert_eq!(promoted.state_after().cold_len(), 0);

    assert_eq!(promoted.state_after().active_len(), 1);

    let record = promoted.record().unwrap();

    assert_eq!(record.first_admitted_at(), 1);

    assert_eq!(record.last_admitted_at(), 2);

    assert_eq!(record.admission_count(), 2);
}

#[test]
fn same_identity_can_demote_from_active_to_cold_without_duplication() {
    let active = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(11, 900, 4),
        policy(),
    );

    let demoted = HierarchicalMemoryAdmission::admit(
        active.state_after().clone(),
        2,
        candidate(11, 100, 2),
        policy(),
    );

    assert_eq!(demoted.previous_tier(), Some(CognitiveMemoryTier::Active,));

    assert_eq!(demoted.status(), HierarchicalMemoryAdmissionStatus::Cold);

    assert_eq!(demoted.state_after().total_len(), 1);

    assert_eq!(demoted.state_after().active_len(), 0);

    assert_eq!(demoted.state_after().cold_len(), 1);
}

#[test]
fn tier_capacity_overflow_evicts_least_recently_admitted_record() {
    let memory_policy = policy_with_capacities(2, 2, 2);

    let first = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(1, 800, 1),
        memory_policy,
    );

    let second = HierarchicalMemoryAdmission::admit(
        first.state_after().clone(),
        2,
        candidate(2, 800, 1),
        memory_policy,
    );

    let third = HierarchicalMemoryAdmission::admit(
        second.state_after().clone(),
        3,
        candidate(3, 800, 1),
        memory_policy,
    );

    let eviction = third.eviction().unwrap();

    assert_eq!(eviction.fingerprint(), CognitiveFingerprint::new(1,));

    assert_eq!(eviction.tier(), CognitiveMemoryTier::Active);

    assert_eq!(third.state_after().active_len(), 2);

    assert!(!third.state_after().contains(CognitiveFingerprint::new(1,),));
}

#[test]
fn memory_tier_capacities_are_independent_and_total_state_is_bounded() {
    let memory_policy = policy_with_capacities(1, 1, 1);

    let cold = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(1, 100, 2),
        memory_policy,
    );

    let consolidated = HierarchicalMemoryAdmission::admit(
        cold.state_after().clone(),
        2,
        candidate(2, 500, 3),
        memory_policy,
    );

    let active = HierarchicalMemoryAdmission::admit(
        consolidated.state_after().clone(),
        3,
        candidate(3, 800, 1),
        memory_policy,
    );

    let replacement_active = HierarchicalMemoryAdmission::admit(
        active.state_after().clone(),
        4,
        candidate(4, 900, 1),
        memory_policy,
    );

    assert_eq!(replacement_active.state_after().cold_len(), 1);

    assert_eq!(replacement_active.state_after().consolidated_len(), 1);

    assert_eq!(replacement_active.state_after().active_len(), 1);

    assert_eq!(replacement_active.state_after().total_len(), 3);

    assert!(replacement_active.state_after().total_len() <= memory_policy.total_capacity());

    assert_eq!(
        replacement_active.eviction().unwrap().fingerprint(),
        CognitiveFingerprint::new(3,)
    );
}

#[test]
fn non_monotonic_memory_event_is_rejected_without_mutation() {
    let first = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        10,
        candidate(1, 800, 1),
        policy(),
    );

    let before = first.state_after().clone();

    let rejected =
        HierarchicalMemoryAdmission::admit(before.clone(), 10, candidate(2, 900, 1), policy());

    assert_eq!(
        rejected.status(),
        HierarchicalMemoryAdmissionStatus::RejectedOutOfOrder
    );

    assert!(!rejected.accepted());

    assert_eq!(rejected.state_before(), &before);

    assert_eq!(rejected.state_after(), &before);

    assert_eq!(rejected.eviction(), None);

    assert_eq!(rejected.record(), None);
}

#[test]
fn refreshing_existing_record_updates_recency_and_protects_it_from_eviction() {
    let memory_policy = policy_with_capacities(2, 2, 2);

    let first = HierarchicalMemoryAdmission::admit(
        HierarchicalMemoryState::empty(),
        1,
        candidate(1, 800, 1),
        memory_policy,
    );

    let second = HierarchicalMemoryAdmission::admit(
        first.state_after().clone(),
        2,
        candidate(2, 800, 1),
        memory_policy,
    );

    let refreshed = HierarchicalMemoryAdmission::admit(
        second.state_after().clone(),
        3,
        candidate(1, 850, 2),
        memory_policy,
    );

    let incoming = HierarchicalMemoryAdmission::admit(
        refreshed.state_after().clone(),
        4,
        candidate(3, 900, 1),
        memory_policy,
    );

    assert_eq!(
        incoming.eviction().unwrap().fingerprint(),
        CognitiveFingerprint::new(2,)
    );

    assert!(incoming
        .state_after()
        .contains(CognitiveFingerprint::new(1,),));

    let refreshed_record = incoming
        .state_after()
        .record(CognitiveFingerprint::new(1))
        .unwrap();

    assert_eq!(refreshed_record.admission_count(), 2);

    assert_eq!(refreshed_record.last_admitted_at(), 3);
}

#[test]
fn hierarchical_memory_is_deterministic_non_mutating_and_facade_equivalent() {
    let initial = HierarchicalMemoryState::empty();

    let initial_before = initial.clone();

    let input_candidate = candidate(77, 800, 4);

    let input_before = input_candidate;

    let memory_policy = policy();

    let direct =
        HierarchicalMemoryAdmission::admit(initial.clone(), 1, input_candidate, memory_policy);

    let facade = MindstoneHierarchicalMemoryAdmission::evaluate(
        initial.clone(),
        1,
        input_candidate,
        memory_policy,
    );

    let repeated = MindstoneHierarchicalMemoryAdmission::evaluate(
        initial.clone(),
        1,
        input_candidate,
        memory_policy,
    );

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(initial, initial_before);

    assert_eq!(input_candidate, input_before);

    assert_eq!(facade.status(), HierarchicalMemoryAdmissionStatus::Active);
}
