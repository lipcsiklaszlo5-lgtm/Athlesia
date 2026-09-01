use athlesia_mindstone_sparse_cognition::{
    CognitiveAdmissionClass, CognitiveBudget, CognitiveFingerprint, CognitiveSignal,
    MindstoneNoveltyGate, MindstoneSignalProfile, NoveltyGate, NoveltyMemory, NoveltyStatus,
    SparseCognitionPolicy,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn fingerprint(value: u64) -> CognitiveFingerprint {
    CognitiveFingerprint::new(value)
}

fn memory(capacity: usize) -> NoveltyMemory {
    NoveltyMemory::new(capacity).unwrap()
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
fn cognitive_fingerprint_preserves_exact_identity() {
    let zero = fingerprint(0);

    let maximum = fingerprint(u64::MAX);

    assert_eq!(zero.value(), 0);

    assert_eq!(maximum.value(), u64::MAX);

    assert_ne!(zero, maximum);
}

#[test]
fn novelty_memory_requires_positive_bounded_capacity() {
    assert_eq!(NoveltyMemory::new(0,), None);

    let memory = memory(3);

    assert_eq!(memory.capacity(), 3);

    assert_eq!(memory.len(), 0);

    assert!(memory.is_empty());

    assert!(!memory.is_full());
}

#[test]
fn first_observation_is_novel_and_recorded_with_maximum_novelty() {
    let input = memory(3);

    let result = NoveltyGate::observe(input.clone(), fingerprint(10));

    assert_eq!(result.status(), NoveltyStatus::Novel);

    assert!(result.is_novel());

    assert_eq!(result.novelty_signal(), CognitiveSignal::maximum());

    assert_eq!(result.memory_before(), &input);

    assert!(result.memory_after().contains(fingerprint(10,),));

    assert_eq!(result.memory_after().len(), 1);

    assert_eq!(result.evicted(), None);
}

#[test]
fn exact_duplicate_is_known_zero_novelty_and_does_not_mutate_memory() {
    let first = NoveltyGate::observe(memory(3), fingerprint(10));

    let before = first.memory_after().clone();

    let duplicate = NoveltyGate::observe(before.clone(), fingerprint(10));

    assert_eq!(duplicate.status(), NoveltyStatus::Known);

    assert!(!duplicate.is_novel());

    assert_eq!(duplicate.novelty_signal(), CognitiveSignal::zero());

    assert_eq!(duplicate.memory_before(), &before);

    assert_eq!(duplicate.memory_after(), &before);

    assert_eq!(duplicate.evicted(), None);
}

#[test]
fn distinct_fingerprint_remains_novel() {
    let first = NoveltyGate::observe(memory(3), fingerprint(10));

    let second = NoveltyGate::observe(first.memory_after().clone(), fingerprint(20));

    assert_eq!(second.status(), NoveltyStatus::Novel);

    assert!(second.memory_after().contains(fingerprint(10,),));

    assert!(second.memory_after().contains(fingerprint(20,),));

    assert_eq!(second.memory_after().len(), 2);
}

#[test]
fn novelty_memory_is_hard_bounded_and_evicts_oldest_fingerprint() {
    let first = NoveltyGate::observe(memory(2), fingerprint(10));

    let second = NoveltyGate::observe(first.memory_after().clone(), fingerprint(20));

    let third = NoveltyGate::observe(second.memory_after().clone(), fingerprint(30));

    assert_eq!(third.memory_after().len(), 2);

    assert!(third.memory_after().is_full());

    assert_eq!(third.evicted(), Some(fingerprint(10,),));

    assert!(!third.memory_after().contains(fingerprint(10,),));

    assert!(third.memory_after().contains(fingerprint(20,),));

    assert!(third.memory_after().contains(fingerprint(30,),));
}

#[test]
fn evicted_fingerprint_can_become_novel_again() {
    let first = NoveltyGate::observe(memory(2), fingerprint(10));

    let second = NoveltyGate::observe(first.memory_after().clone(), fingerprint(20));

    let third = NoveltyGate::observe(second.memory_after().clone(), fingerprint(30));

    let returned = NoveltyGate::observe(third.memory_after().clone(), fingerprint(10));

    assert_eq!(returned.status(), NoveltyStatus::Novel);

    assert_eq!(returned.novelty_signal(), CognitiveSignal::maximum());

    assert_eq!(returned.evicted(), Some(fingerprint(20,),));
}

#[test]
fn novel_low_other_signal_event_receives_cheap_update() {
    let result = MindstoneNoveltyGate::evaluate(
        memory(4),
        fingerprint(10),
        profile(0, 0, 0, 0, 0),
        policy(),
        budget(100),
    );

    assert_eq!(result.novelty().status(), NoveltyStatus::Novel);

    assert_eq!(result.profile().novelty(), CognitiveSignal::maximum());

    assert_eq!(
        result.decision().class(),
        CognitiveAdmissionClass::CheapUpdate
    );
}

#[test]
fn repeated_low_signal_event_is_suppressed_to_ignore() {
    let first = MindstoneNoveltyGate::evaluate(
        memory(4),
        fingerprint(10),
        profile(0, 0, 0, 0, 0),
        policy(),
        budget(100),
    );

    let repeated = MindstoneNoveltyGate::evaluate(
        first.novelty().memory_after().clone(),
        fingerprint(10),
        profile(0, 0, 1000, 0, 0),
        policy(),
        budget(100),
    );

    assert_eq!(repeated.novelty().status(), NoveltyStatus::Known);

    assert_eq!(repeated.profile().novelty(), CognitiveSignal::zero());

    assert_eq!(repeated.decision().class(), CognitiveAdmissionClass::Ignore);

    assert_eq!(repeated.decision().requested_units(), 0);
}

#[test]
fn known_event_does_not_suppress_independent_high_epistemic_pressure() {
    let first = NoveltyGate::observe(memory(4), fingerprint(10));

    let result = MindstoneNoveltyGate::evaluate(
        first.memory_after().clone(),
        fingerprint(10),
        profile(1000, 1000, 1000, 1000, 1000),
        policy(),
        budget(100),
    );

    assert_eq!(result.novelty().status(), NoveltyStatus::Known);

    assert_eq!(result.profile().novelty(), CognitiveSignal::zero());

    assert_eq!(
        result.decision().class(),
        CognitiveAdmissionClass::Deliberate
    );

    assert!(result.decision().is_deliberative());
}

#[test]
fn novelty_admission_still_obeys_hard_compute_budget() {
    let result = MindstoneNoveltyGate::evaluate(
        memory(4),
        fingerprint(10),
        profile(1000, 1000, 0, 1000, 1000),
        policy(),
        budget(3),
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
fn novelty_gate_is_deterministic_facade_composable_and_non_mutating() {
    let initial_memory = memory(4);

    let initial_memory_before = initial_memory.clone();

    let input_profile = profile(350, 150, 999, 250, 450);

    let input_profile_before = input_profile;

    let policy = policy();

    let budget = budget(5);

    let first = MindstoneNoveltyGate::evaluate(
        initial_memory.clone(),
        fingerprint(77),
        input_profile,
        policy,
        budget,
    );

    let repeated = MindstoneNoveltyGate::evaluate(
        initial_memory.clone(),
        fingerprint(77),
        input_profile,
        policy,
        budget,
    );

    let direct_novelty = NoveltyGate::observe(initial_memory.clone(), fingerprint(77));

    let direct_profile = input_profile.with_novelty(direct_novelty.novelty_signal());

    let direct_decision = policy.admit(direct_profile, budget);

    assert_eq!(first, repeated);

    assert_eq!(first.novelty(), &direct_novelty);

    assert_eq!(first.profile(), direct_profile);

    assert_eq!(first.decision(), direct_decision);

    assert_eq!(initial_memory, initial_memory_before);

    assert_eq!(input_profile, input_profile_before);
}
