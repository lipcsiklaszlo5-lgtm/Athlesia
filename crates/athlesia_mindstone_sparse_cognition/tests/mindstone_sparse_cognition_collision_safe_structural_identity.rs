use athlesia_mindstone_sparse_cognition::{
    CognitiveFingerprint, CognitiveSalience, CognitiveSignal, CognitiveStructure,
    CollisionSafeStructuralCognition, CollisionSafeStructuralIdentity,
    CollisionSafeStructuralObservationStatus, CollisionSafeStructuralState,
    MindstoneCollisionSafeStructuralCognition, MindstoneSignalProfile, StructuralHasher,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn salience(value: u16) -> CognitiveSalience {
    MindstoneSignalProfile::new(
        signal(value),
        signal(value),
        signal(value),
        signal(value),
        signal(value),
    )
    .salience()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn state(capacity: usize) -> CollisionSafeStructuralState {
    CollisionSafeStructuralState::new(capacity).unwrap()
}

#[test]
fn semantic_identity_uses_exact_structure_not_fingerprint_hint() {
    let shared = CognitiveFingerprint::new(42);

    let first = CollisionSafeStructuralIdentity::with_fingerprint_hint(shared, atom(1));

    let second = CollisionSafeStructuralIdentity::with_fingerprint_hint(shared, atom(2));

    assert_eq!(first.fingerprint(), second.fingerprint());

    assert_ne!(first, second);

    assert!(!first.semantically_matches(&second,));
}

#[test]
fn collision_safe_state_requires_positive_hard_capacity() {
    assert_eq!(CollisionSafeStructuralState::new(0,), None);

    let empty = state(3);

    assert_eq!(empty.capacity(), 3);

    assert_eq!(empty.len(), 0);

    assert!(empty.is_empty());
}

#[test]
fn forced_same_fingerprint_distinct_structures_remain_separate_records() {
    let shared = CognitiveFingerprint::new(777);

    let first_structure = atom(10);

    let second_structure = atom(20);

    let first = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        state(4),
        1,
        shared,
        first_structure.clone(),
        salience(300),
    );

    let second = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        first.state_after().clone(),
        2,
        shared,
        second_structure.clone(),
        salience(400),
    );

    assert_eq!(second.state_after().len(), 2);

    assert_eq!(second.state_after().bucket_len(shared,), 2);

    let bucket = second.state_after().records_in_bucket(shared).unwrap();

    assert!(bucket
        .iter()
        .any(|record| { record.structure() == &first_structure },));

    assert!(bucket
        .iter()
        .any(|record| { record.structure() == &second_structure },));
}

#[test]
fn exact_structure_repeat_updates_existing_record_inside_collision_bucket() {
    let shared = CognitiveFingerprint::new(888);

    let structure = atom(30);

    let first = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        state(4),
        1,
        shared,
        structure.clone(),
        salience(200),
    );

    let second = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        first.state_after().clone(),
        2,
        shared,
        structure.clone(),
        salience(600),
    );

    assert_eq!(
        second.status(),
        CollisionSafeStructuralObservationStatus::ExistingStructure
    );

    assert_eq!(second.state_after().len(), 1);

    assert_eq!(second.state_after().bucket_len(shared,), 1);

    let record = second.record().unwrap();

    assert_eq!(record.observation_count(), 2);

    assert_eq!(record.first_seen(), 1);

    assert_eq!(record.last_seen(), 2);
}

#[test]
fn normal_structural_path_computes_fingerprint_and_marks_new_structure_novel() {
    let structure = CognitiveStructure::ordered(vec![atom(1), atom(2)]).unwrap();

    let expected = StructuralHasher::fingerprint(&structure);

    let result =
        CollisionSafeStructuralCognition::observe(state(4), 1, structure.clone(), salience(500));

    assert_eq!(
        result.status(),
        CollisionSafeStructuralObservationStatus::NewStructure
    );

    assert!(result.is_novel());

    assert_eq!(result.novelty(), CognitiveSignal::maximum());

    assert_eq!(result.identity().fingerprint(), expected);

    assert!(result.state_after().contains_structure(&structure,));
}

#[test]
fn repeated_exact_structure_has_zero_novelty() {
    let structure = CognitiveStructure::ordered(vec![atom(4), atom(5)]).unwrap();

    let first =
        CollisionSafeStructuralCognition::observe(state(4), 1, structure.clone(), salience(300));

    let second = CollisionSafeStructuralCognition::observe(
        first.state_after().clone(),
        2,
        structure,
        salience(300),
    );

    assert_eq!(
        second.status(),
        CollisionSafeStructuralObservationStatus::ExistingStructure
    );

    assert!(!second.is_novel());

    assert_eq!(second.novelty(), CognitiveSignal::zero());

    assert_eq!(second.state_after().len(), 1);
}

#[test]
fn canonical_unordered_reordering_is_one_exact_structural_identity() {
    let first_structure = CognitiveStructure::unordered(vec![atom(1), atom(2), atom(3)]).unwrap();

    let second_structure = CognitiveStructure::unordered(vec![atom(3), atom(1), atom(2)]).unwrap();

    assert_eq!(first_structure, second_structure);

    let first =
        CollisionSafeStructuralCognition::observe(state(4), 1, first_structure, salience(300));

    let second = CollisionSafeStructuralCognition::observe(
        first.state_after().clone(),
        2,
        second_structure,
        salience(400),
    );

    assert_eq!(second.state_after().len(), 1);

    assert_eq!(second.record().unwrap().observation_count(), 2);
}

#[test]
fn colliding_structures_keep_independent_streaming_statistics() {
    let shared = CognitiveFingerprint::new(999);

    let first_structure = atom(100);

    let second_structure = atom(200);

    let first = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        state(4),
        1,
        shared,
        first_structure.clone(),
        salience(200),
    );

    let second = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        first.state_after().clone(),
        2,
        shared,
        second_structure.clone(),
        salience(700),
    );

    let third = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        second.state_after().clone(),
        3,
        shared,
        first_structure.clone(),
        salience(400),
    );

    let first_identity =
        CollisionSafeStructuralIdentity::with_fingerprint_hint(shared, first_structure);

    let second_identity =
        CollisionSafeStructuralIdentity::with_fingerprint_hint(shared, second_structure);

    let first_record = third
        .state_after()
        .record_for_identity(&first_identity)
        .unwrap();

    let second_record = third
        .state_after()
        .record_for_identity(&second_identity)
        .unwrap();

    assert_eq!(first_record.observation_count(), 2);

    assert_eq!(second_record.observation_count(), 1);

    assert_ne!(
        first_record.total_salience(),
        second_record.total_salience()
    );

    assert_eq!(third.state_after().total_retained_observations(), 3);
}

#[test]
fn collision_safe_capacity_uses_deterministic_global_recency_eviction() {
    let shared = CognitiveFingerprint::new(55);

    let first_structure = atom(1);

    let second_structure = atom(2);

    let third_structure = atom(3);

    let first = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        state(2),
        1,
        shared,
        first_structure.clone(),
        salience(200),
    );

    let second = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        first.state_after().clone(),
        2,
        shared,
        second_structure.clone(),
        salience(200),
    );

    let refreshed = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        second.state_after().clone(),
        3,
        shared,
        first_structure.clone(),
        salience(300),
    );

    let incoming = CollisionSafeStructuralCognition::observe_with_fingerprint_hint(
        refreshed.state_after().clone(),
        4,
        shared,
        third_structure.clone(),
        salience(400),
    );

    assert_eq!(incoming.state_after().len(), 2);

    let evicted = incoming.evicted().unwrap();

    assert_eq!(evicted.structure(), &second_structure);

    let bucket = incoming.state_after().records_in_bucket(shared).unwrap();

    assert!(bucket
        .iter()
        .any(|record| { record.structure() == &first_structure },));

    assert!(bucket
        .iter()
        .any(|record| { record.structure() == &third_structure },));
}

#[test]
fn non_monotonic_collision_safe_observation_is_rejected_without_mutation() {
    let structure = atom(700);

    let first = CollisionSafeStructuralCognition::observe(state(4), 10, structure, salience(300));

    let before = first.state_after().clone();

    let rejected =
        CollisionSafeStructuralCognition::observe(before.clone(), 10, atom(701), salience(900));

    assert_eq!(
        rejected.status(),
        CollisionSafeStructuralObservationStatus::RejectedOutOfOrder
    );

    assert!(!rejected.accepted());

    assert_eq!(rejected.state_before(), &before);

    assert_eq!(rejected.state_after(), &before);

    assert_eq!(rejected.record(), None);

    assert_eq!(rejected.evicted(), None);
}

#[test]
fn ten_thousand_repeated_observations_remain_one_bounded_exact_record() {
    let structure = CognitiveStructure::ordered(vec![atom(9), atom(8), atom(7)]).unwrap();

    let mut current = state(2);

    for index in 1_u64..=10_000 {
        current = CollisionSafeStructuralCognition::observe(
            current,
            index,
            structure.clone(),
            salience(500),
        )
        .state_after()
        .clone();
    }

    assert_eq!(current.len(), 1);

    assert_eq!(
        current
            .record_for_structure(&structure,)
            .unwrap()
            .observation_count(),
        10_000
    );

    assert_eq!(current.total_retained_observations(), 10_000);

    assert!(current.len() <= current.capacity());
}

#[test]
fn collision_safe_structural_path_is_deterministic_non_mutating_and_facade_equivalent() {
    let structure = CognitiveStructure::ordered(vec![atom(11), atom(22)]).unwrap();

    let structure_before = structure.clone();

    let initial = state(5);

    let initial_before = initial.clone();

    let input_salience = salience(650);

    let direct = CollisionSafeStructuralCognition::observe(
        initial.clone(),
        1,
        structure.clone(),
        input_salience,
    );

    let facade = MindstoneCollisionSafeStructuralCognition::observe(
        initial.clone(),
        1,
        structure.clone(),
        input_salience,
    );

    let repeated = MindstoneCollisionSafeStructuralCognition::observe(
        initial.clone(),
        1,
        structure.clone(),
        input_salience,
    );

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(initial, initial_before);

    assert_eq!(structure, structure_before);

    assert_eq!(facade.state_after().len(), 1);
}
