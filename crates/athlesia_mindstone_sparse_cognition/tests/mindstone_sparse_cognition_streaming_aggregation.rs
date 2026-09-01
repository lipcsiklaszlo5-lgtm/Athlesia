use athlesia_mindstone_sparse_cognition::{
    CognitiveSignal, CognitiveStructure, MindstoneSignalProfile, MindstoneStreamingAggregator,
    StreamingAggregationState, StreamingAggregationStatus, StreamingAggregator, StructuralHasher,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn unordered(children: Vec<CognitiveStructure>) -> CognitiveStructure {
    CognitiveStructure::unordered(children).unwrap()
}

fn state(capacity: usize) -> StreamingAggregationState {
    StreamingAggregationState::new(capacity).unwrap()
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

#[test]
fn streaming_state_requires_positive_capacity() {
    assert_eq!(StreamingAggregationState::new(0,), None);

    let empty = state(3);

    assert_eq!(empty.capacity(), 3);

    assert_eq!(empty.len(), 0);

    assert!(empty.is_empty());

    assert_eq!(empty.last_event_index(), None);
}

#[test]
fn first_observation_creates_one_sufficient_statistic() {
    let structure = atom(10);

    let fingerprint = StructuralHasher::fingerprint(&structure);

    let result = StreamingAggregator::observe(state(4), 1, fingerprint, profile(400).salience());

    assert_eq!(result.status(), StreamingAggregationStatus::Inserted);

    assert!(result.accepted());

    assert_eq!(result.state_after().len(), 1);

    let aggregate = result.aggregate().unwrap();

    assert_eq!(aggregate.fingerprint(), fingerprint);

    assert_eq!(aggregate.observation_count(), 1);

    assert_eq!(aggregate.first_seen(), 1);

    assert_eq!(aggregate.last_seen(), 1);
}

#[test]
fn repeated_observation_updates_existing_statistic_without_state_growth() {
    let fingerprint = StructuralHasher::fingerprint(&atom(10));

    let first = StreamingAggregator::observe(state(4), 1, fingerprint, profile(300).salience());

    let second = StreamingAggregator::observe(
        first.state_after().clone(),
        2,
        fingerprint,
        profile(500).salience(),
    );

    assert_eq!(second.status(), StreamingAggregationStatus::Updated);

    assert_eq!(second.state_after().len(), 1);

    let aggregate = second.aggregate().unwrap();

    assert_eq!(aggregate.observation_count(), 2);

    assert_eq!(aggregate.first_seen(), 1);

    assert_eq!(aggregate.last_seen(), 2);
}

#[test]
fn many_repeated_events_are_compressed_into_constant_distinct_state() {
    let fingerprint = StructuralHasher::fingerprint(&atom(99));

    let mut current = state(4);

    for event_index in 1..=10_000_u64 {
        current = StreamingAggregator::observe(
            current,
            event_index,
            fingerprint,
            profile(250).salience(),
        )
        .state_after()
        .clone();
    }

    assert_eq!(current.len(), 1);

    assert_eq!(current.total_retained_observations(), 10_000);

    assert_eq!(
        current.aggregate(fingerprint,).unwrap().observation_count(),
        10_000
    );
}

#[test]
fn aggregate_preserves_total_mean_and_peak_salience() {
    let fingerprint = StructuralHasher::fingerprint(&atom(5));

    let first = StreamingAggregator::observe(state(3), 1, fingerprint, profile(100).salience());

    let second = StreamingAggregator::observe(
        first.state_after().clone(),
        2,
        fingerprint,
        profile(500).salience(),
    );

    let third = StreamingAggregator::observe(
        second.state_after().clone(),
        3,
        fingerprint,
        profile(300).salience(),
    );

    let aggregate = third.aggregate().unwrap();

    assert_eq!(aggregate.total_salience(), 900);

    assert_eq!(aggregate.mean_salience(), 300);

    assert_eq!(aggregate.peak_salience().value(), 500);
}

#[test]
fn distinct_structures_are_retained_until_capacity_is_reached() {
    let first = MindstoneStreamingAggregator::observe(state(3), 1, atom(1), profile(100));

    let second = MindstoneStreamingAggregator::observe(
        first.aggregation().state_after().clone(),
        2,
        atom(2),
        profile(200),
    );

    let third = MindstoneStreamingAggregator::observe(
        second.aggregation().state_after().clone(),
        3,
        atom(3),
        profile(300),
    );

    assert_eq!(third.aggregation().state_after().len(), 3);

    assert!(third.aggregation().state_after().is_full());

    assert_eq!(third.aggregation().evicted(), None);
}

#[test]
fn capacity_overflow_evicts_least_recently_seen_statistic() {
    let first = MindstoneStreamingAggregator::observe(state(2), 1, atom(1), profile(100));

    let second = MindstoneStreamingAggregator::observe(
        first.aggregation().state_after().clone(),
        2,
        atom(2),
        profile(100),
    );

    let fingerprint_one = first.fingerprint();

    let fingerprint_two = second.fingerprint();

    let third = MindstoneStreamingAggregator::observe(
        second.aggregation().state_after().clone(),
        3,
        atom(3),
        profile(100),
    );

    assert_eq!(third.aggregation().evicted(), Some(fingerprint_one,));

    assert!(!third.aggregation().state_after().contains(fingerprint_one,));

    assert!(third.aggregation().state_after().contains(fingerprint_two,));

    assert_eq!(third.aggregation().state_after().len(), 2);
}

#[test]
fn recent_update_protects_statistic_from_lru_eviction() {
    let first = MindstoneStreamingAggregator::observe(state(2), 1, atom(1), profile(100));

    let second = MindstoneStreamingAggregator::observe(
        first.aggregation().state_after().clone(),
        2,
        atom(2),
        profile(100),
    );

    let fingerprint_one = first.fingerprint();

    let fingerprint_two = second.fingerprint();

    let refreshed = StreamingAggregator::observe(
        second.aggregation().state_after().clone(),
        3,
        fingerprint_one,
        profile(100).salience(),
    );

    let incoming = MindstoneStreamingAggregator::observe(
        refreshed.state_after().clone(),
        4,
        atom(3),
        profile(100),
    );

    assert_eq!(incoming.aggregation().evicted(), Some(fingerprint_two,));

    assert!(incoming
        .aggregation()
        .state_after()
        .contains(fingerprint_one,));
}

#[test]
fn evicted_structure_reenters_with_fresh_statistics() {
    let first = MindstoneStreamingAggregator::observe(state(1), 1, atom(1), profile(100));

    let second = MindstoneStreamingAggregator::observe(
        first.aggregation().state_after().clone(),
        2,
        atom(2),
        profile(100),
    );

    let returned = MindstoneStreamingAggregator::observe(
        second.aggregation().state_after().clone(),
        3,
        atom(1),
        profile(700),
    );

    let aggregate = returned.aggregation().aggregate().unwrap();

    assert_eq!(aggregate.observation_count(), 1);

    assert_eq!(aggregate.first_seen(), 3);

    assert_eq!(aggregate.last_seen(), 3);

    assert_eq!(aggregate.mean_salience(), 700);
}

#[test]
fn non_monotonic_event_index_is_rejected_without_mutation() {
    let first = MindstoneStreamingAggregator::observe(state(3), 10, atom(1), profile(400));

    let before = first.aggregation().state_after().clone();

    let rejected = MindstoneStreamingAggregator::observe(before.clone(), 10, atom(2), profile(900));

    assert_eq!(
        rejected.aggregation().status(),
        StreamingAggregationStatus::RejectedOutOfOrder
    );

    assert!(!rejected.aggregation().accepted());

    assert_eq!(rejected.aggregation().state_before(), &before);

    assert_eq!(rejected.aggregation().state_after(), &before);

    assert_eq!(rejected.aggregation().aggregate(), None);
}

#[test]
fn canonical_unordered_structures_stream_into_same_statistic() {
    let first_structure = unordered(vec![atom(1), atom(2), atom(3)]);

    let second_structure = unordered(vec![atom(3), atom(1), atom(2)]);

    let first = MindstoneStreamingAggregator::observe(state(4), 1, first_structure, profile(200));

    let second = MindstoneStreamingAggregator::observe(
        first.aggregation().state_after().clone(),
        2,
        second_structure,
        profile(400),
    );

    assert_eq!(first.fingerprint(), second.fingerprint());

    assert_eq!(second.aggregation().state_after().len(), 1);

    assert_eq!(
        second
            .aggregation()
            .aggregate()
            .unwrap()
            .observation_count(),
        2
    );
}

#[test]
fn streaming_facade_is_deterministic_non_mutating_and_matches_direct_path() {
    let structure = unordered(vec![atom(9), atom(7), atom(8)]);

    let structure_before = structure.clone();

    let initial_state = state(5);

    let state_before = initial_state.clone();

    let input_profile = profile(620);

    let profile_before = input_profile;

    let fingerprint = StructuralHasher::fingerprint(&structure);

    let direct = StreamingAggregator::observe(
        initial_state.clone(),
        1,
        fingerprint,
        input_profile.salience(),
    );

    let facade = MindstoneStreamingAggregator::observe(
        initial_state.clone(),
        1,
        structure.clone(),
        input_profile,
    );

    let repeated = MindstoneStreamingAggregator::observe(
        initial_state.clone(),
        1,
        structure.clone(),
        input_profile,
    );

    assert_eq!(facade.aggregation(), &direct);

    assert_eq!(facade, repeated);

    assert_eq!(facade.fingerprint(), fingerprint);

    assert_eq!(structure, structure_before);

    assert_eq!(initial_state, state_before);

    assert_eq!(input_profile, profile_before);
}
