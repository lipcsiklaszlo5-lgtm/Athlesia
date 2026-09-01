use athlesia_mindstone_sparse_cognition::{
    CognitiveFingerprint, CognitiveSignal, CognitiveStructure, EpistemicSelfClass,
    EpistemicSelfModel, EpistemicSelfPolicy, EpistemicSelfState, EpistemicSelfUpdateStatus,
    MindstoneEpistemicSelfModel, MindstoneExtendedSignalProfile, MindstoneSignalProfile,
    StructuralHasher,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn policy() -> EpistemicSelfPolicy {
    EpistemicSelfPolicy::new(signal(200), signal(300), signal(400), signal(600), 2).unwrap()
}

fn state(capacity: usize) -> EpistemicSelfState {
    EpistemicSelfState::new(capacity).unwrap()
}

fn profile(
    uncertainty: u16,
    learning_progress: u16,
    compression_gain: u16,
    controllability: u16,
) -> MindstoneExtendedSignalProfile {
    let base = MindstoneSignalProfile::new(
        signal(100),
        signal(uncertainty),
        signal(100),
        signal(learning_progress),
        signal(100),
    );

    MindstoneExtendedSignalProfile::new(base, signal(compression_gain), signal(controllability))
}

#[test]
fn epistemic_policy_and_state_require_nonvacuous_bounds() {
    assert_eq!(
        EpistemicSelfPolicy::new(signal(200,), signal(0,), signal(400,), signal(600,), 2,),
        None
    );

    assert_eq!(
        EpistemicSelfPolicy::new(signal(200,), signal(300,), signal(0,), signal(600,), 2,),
        None
    );

    assert_eq!(
        EpistemicSelfPolicy::new(signal(200,), signal(300,), signal(400,), signal(0,), 2,),
        None
    );

    assert_eq!(
        EpistemicSelfPolicy::new(signal(200,), signal(300,), signal(400,), signal(600,), 0,),
        None
    );

    assert_eq!(EpistemicSelfState::new(0,), None);

    let empty = state(3);

    assert_eq!(empty.capacity(), 3);

    assert!(empty.is_empty());
}

#[test]
fn first_observation_creates_bounded_self_record() {
    let result = EpistemicSelfModel::observe(
        state(4),
        1,
        CognitiveFingerprint::new(10),
        profile(900, 0, 100, 100),
        policy(),
    );

    assert!(result.accepted());

    assert_eq!(result.status(), EpistemicSelfUpdateStatus::Updated);

    assert_eq!(result.state_after().len(), 1);

    let record = result.record().unwrap();

    assert_eq!(record.fingerprint(), CognitiveFingerprint::new(10,));

    assert_eq!(record.observation_count(), 1);

    assert_eq!(record.first_updated_at(), 1);

    assert_eq!(record.last_updated_at(), 1);
}

#[test]
fn repeated_identity_updates_latest_epistemic_state_without_growth() {
    let first = EpistemicSelfModel::observe(
        state(4),
        1,
        CognitiveFingerprint::new(10),
        profile(900, 100, 100, 100),
        policy(),
    );

    let second = EpistemicSelfModel::observe(
        first.state_after().clone(),
        2,
        CognitiveFingerprint::new(10),
        profile(500, 400, 700, 800),
        policy(),
    );

    assert_eq!(second.state_after().len(), 1);

    let record = second.record().unwrap();

    assert_eq!(record.observation_count(), 2);

    assert_eq!(record.first_updated_at(), 1);

    assert_eq!(record.last_updated_at(), 2);

    assert_eq!(record.uncertainty().value(), 500);

    assert_eq!(record.learning_progress().value(), 400);

    assert_eq!(record.compression_gain().value(), 700);

    assert_eq!(record.controllability().value(), 800);
}

#[test]
fn unresolved_high_uncertainty_without_progress_is_classified_uncertain() {
    let result = EpistemicSelfModel::observe(
        state(4),
        1,
        CognitiveFingerprint::new(1),
        profile(900, 100, 100, 100),
        policy(),
    );

    let assessment = result.assessment().unwrap();

    assert_eq!(assessment.class(), EpistemicSelfClass::Uncertain);

    assert!(assessment.is_uncertain());

    assert!(!assessment.is_learning());

    assert!(!assessment.is_stable());
}

#[test]
fn uncertainty_with_measurable_learning_progress_is_classified_learning() {
    let result = EpistemicSelfModel::observe(
        state(4),
        1,
        CognitiveFingerprint::new(2),
        profile(800, 500, 100, 100),
        policy(),
    );

    let assessment = result.assessment().unwrap();

    assert_eq!(assessment.class(), EpistemicSelfClass::Learning);

    assert!(assessment.is_learning());
}

#[test]
fn low_uncertainty_becomes_stable_only_after_sufficient_observation() {
    let first = EpistemicSelfModel::observe(
        state(4),
        1,
        CognitiveFingerprint::new(3),
        profile(100, 0, 100, 100),
        policy(),
    );

    assert_eq!(
        first.assessment().unwrap().class(),
        EpistemicSelfClass::Uncertain
    );

    let second = EpistemicSelfModel::observe(
        first.state_after().clone(),
        2,
        CognitiveFingerprint::new(3),
        profile(100, 0, 100, 100),
        policy(),
    );

    let assessment = second.assessment().unwrap();

    assert_eq!(assessment.class(), EpistemicSelfClass::Stable);

    assert!(assessment.is_stable());
}

#[test]
fn compression_and_control_are_independent_supported_self_axes() {
    let first = EpistemicSelfModel::observe(
        state(4),
        1,
        CognitiveFingerprint::new(10),
        profile(500, 0, 900, 100),
        policy(),
    );

    let compression_supported = EpistemicSelfModel::observe(
        first.state_after().clone(),
        2,
        CognitiveFingerprint::new(10),
        profile(500, 0, 900, 100),
        policy(),
    );

    let compression_assessment = compression_supported.assessment().unwrap();

    assert!(compression_assessment.is_compressible());

    assert!(!compression_assessment.is_controllable());

    let control_first = EpistemicSelfModel::observe(
        compression_supported.state_after().clone(),
        3,
        CognitiveFingerprint::new(20),
        profile(500, 0, 100, 900),
        policy(),
    );

    let control_second = EpistemicSelfModel::observe(
        control_first.state_after().clone(),
        4,
        CognitiveFingerprint::new(20),
        profile(500, 0, 100, 900),
        policy(),
    );

    let control_assessment = control_second.assessment().unwrap();

    assert!(!control_assessment.is_compressible());

    assert!(control_assessment.is_controllable());
}

#[test]
fn compression_and_control_claims_require_minimum_observation_support() {
    let result = EpistemicSelfModel::observe(
        state(4),
        1,
        CognitiveFingerprint::new(30),
        profile(500, 0, 1000, 1000),
        policy(),
    );

    let assessment = result.assessment().unwrap();

    assert!(!assessment.is_compressible());

    assert!(!assessment.is_controllable());

    assert_eq!(result.record().unwrap().observation_count(), 1);
}

#[test]
fn epistemic_self_state_is_hard_bounded_with_recency_eviction() {
    let first = EpistemicSelfModel::observe(
        state(2),
        1,
        CognitiveFingerprint::new(1),
        profile(500, 0, 100, 100),
        policy(),
    );

    let second = EpistemicSelfModel::observe(
        first.state_after().clone(),
        2,
        CognitiveFingerprint::new(2),
        profile(500, 0, 100, 100),
        policy(),
    );

    let refreshed = EpistemicSelfModel::observe(
        second.state_after().clone(),
        3,
        CognitiveFingerprint::new(1),
        profile(400, 100, 200, 200),
        policy(),
    );

    let incoming = EpistemicSelfModel::observe(
        refreshed.state_after().clone(),
        4,
        CognitiveFingerprint::new(3),
        profile(500, 0, 100, 100),
        policy(),
    );

    assert_eq!(incoming.state_after().len(), 2);

    assert_eq!(incoming.evicted(), Some(CognitiveFingerprint::new(2,),));

    assert!(incoming
        .state_after()
        .contains(CognitiveFingerprint::new(1,),));

    assert!(incoming
        .state_after()
        .contains(CognitiveFingerprint::new(3,),));
}

#[test]
fn non_monotonic_epistemic_update_is_rejected_without_mutation() {
    let first = EpistemicSelfModel::observe(
        state(4),
        10,
        CognitiveFingerprint::new(1),
        profile(500, 0, 100, 100),
        policy(),
    );

    let before = first.state_after().clone();

    let rejected = EpistemicSelfModel::observe(
        before.clone(),
        10,
        CognitiveFingerprint::new(2),
        profile(100, 1000, 1000, 1000),
        policy(),
    );

    assert_eq!(
        rejected.status(),
        EpistemicSelfUpdateStatus::RejectedOutOfOrder
    );

    assert!(!rejected.accepted());

    assert_eq!(rejected.state_before(), &before);

    assert_eq!(rejected.state_after(), &before);

    assert_eq!(rejected.record(), None);

    assert_eq!(rejected.assessment(), None);
}

#[test]
fn canonical_structures_update_one_shared_epistemic_identity() {
    let first_structure = CognitiveStructure::unordered(vec![
        CognitiveStructure::atom(1),
        CognitiveStructure::atom(2),
        CognitiveStructure::atom(3),
    ])
    .unwrap();

    let second_structure = CognitiveStructure::unordered(vec![
        CognitiveStructure::atom(3),
        CognitiveStructure::atom(1),
        CognitiveStructure::atom(2),
    ])
    .unwrap();

    let first = MindstoneEpistemicSelfModel::observe_structure(
        state(4),
        1,
        &first_structure,
        profile(700, 400, 500, 500),
        policy(),
    );

    let second = MindstoneEpistemicSelfModel::observe_structure(
        first.state_after().clone(),
        2,
        &second_structure,
        profile(300, 500, 600, 700),
        policy(),
    );

    assert_eq!(
        StructuralHasher::fingerprint(&first_structure,),
        StructuralHasher::fingerprint(&second_structure,)
    );

    assert_eq!(second.state_after().len(), 1);

    assert_eq!(second.record().unwrap().observation_count(), 2);
}

#[test]
fn epistemic_self_model_is_deterministic_non_mutating_and_facade_equivalent() {
    let structure = CognitiveStructure::ordered(vec![
        CognitiveStructure::atom(7),
        CognitiveStructure::atom(9),
    ])
    .unwrap();

    let structure_before = structure.clone();

    let initial = state(5);

    let initial_before = initial.clone();

    let input_profile = profile(350, 450, 650, 750);

    let profile_before = input_profile;

    let self_policy = policy();

    let fingerprint = StructuralHasher::fingerprint(&structure);

    let direct =
        EpistemicSelfModel::observe(initial.clone(), 1, fingerprint, input_profile, self_policy);

    let facade = MindstoneEpistemicSelfModel::observe_structure(
        initial.clone(),
        1,
        &structure,
        input_profile,
        self_policy,
    );

    let repeated = MindstoneEpistemicSelfModel::observe_structure(
        initial.clone(),
        1,
        &structure,
        input_profile,
        self_policy,
    );

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(initial, initial_before);

    assert_eq!(structure, structure_before);

    assert_eq!(input_profile, profile_before);

    assert_eq!(facade.fingerprint(), fingerprint);
}
