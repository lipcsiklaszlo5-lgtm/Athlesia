use athlesia_meta_learning_skill_memory::{
    GroundedEpisodicAnalogyPolicy, GroundedEpisodicAnalogyTransfer, GroundedSkillEpisode,
    GroundedSkillStep, SkillExecutionObservation,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

fn s(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).expect("positive bounded signal")
}

fn a(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn source_episode() -> GroundedSkillEpisode {
    GroundedSkillEpisode::new(
        a(100),
        a(700),
        vec![
            GroundedSkillStep::new(a(100), a(110), a(210), s(1000)).expect("valid source prefix"),
            GroundedSkillStep::new(a(210), a(700), a(999), s(1000))
                .expect("valid source continuation"),
        ],
        s(1000),
    )
    .expect("valid one-shot source episode")
}

fn prefix(action: u64, outcome: u64, confidence: u16) -> SkillExecutionObservation {
    SkillExecutionObservation::new(a(900), a(action), a(outcome), s(confidence))
        .expect("valid target evidence")
}

fn policy(max_observations: usize) -> GroundedEpisodicAnalogyPolicy {
    GroundedEpisodicAnalogyPolicy::new(max_observations, s(500)).expect("valid analogy policy")
}

#[test]
fn dominant_reproducible_correspondence_survives_one_poisoned_outlier_independent_of_order() {
    let source = source_episode();
    let evidence = vec![
        prefix(910, 1010, 900),
        prefix(910, 1010, 900),
        prefix(910, 1010, 900),
        prefix(920, 1020, 900),
    ];

    let forward = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &evidence,
        policy(16),
    );

    let mut reversed = evidence;
    reversed.reverse();

    let backward = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &reversed,
        policy(16),
    );

    assert_eq!(forward, backward);
    assert_eq!(forward.candidate_count(), 1);
    assert!(!forward.conflicting_evidence());
    assert!(!forward.observation_frontier_exceeded());

    let candidate = &forward.candidates()[0];
    assert_eq!(candidate.required_state(), &a(1010));
    assert_eq!(candidate.action(), &a(20_000));
    assert_eq!(candidate.predicted_outcome(), None);
}

#[test]
fn balanced_reproducible_poisoning_forces_abstention_independent_of_order() {
    let source = source_episode();
    let evidence = vec![
        prefix(910, 1010, 900),
        prefix(910, 1010, 900),
        prefix(920, 1020, 900),
        prefix(920, 1020, 900),
    ];

    let forward = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &evidence,
        policy(16),
    );

    let mut reversed = evidence;
    reversed.reverse();

    let backward = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &reversed,
        policy(16),
    );

    assert_eq!(forward, backward);
    assert_eq!(forward.candidate_count(), 0);
    assert!(forward.conflicting_evidence());
    assert!(!forward.observation_frontier_exceeded());
    assert!(forward.abstained());
}

#[test]
fn low_confidence_poison_is_rejected_without_erasing_strong_grounded_evidence() {
    let source = source_episode();
    let evidence = vec![prefix(910, 1010, 900), prefix(920, 1020, 400)];

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &evidence,
        policy(16),
    );

    assert_eq!(result.candidate_count(), 1);
    assert_eq!(result.rejected_low_confidence_count(), 1);
    assert!(!result.conflicting_evidence());
    assert_eq!(result.candidates()[0].required_state(), &a(1010));
    assert_eq!(result.candidates()[0].action(), &a(20_000));
}

#[test]
fn bounded_evidence_pressure_is_not_misreported_as_semantic_conflict() {
    let source = source_episode();
    let evidence = vec![
        prefix(910, 1010, 900),
        prefix(920, 1020, 900),
        prefix(930, 1030, 900),
    ];

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &evidence,
        policy(2),
    );

    assert_eq!(result.candidate_count(), 0);
    assert!(result.observation_frontier_exceeded());
    assert!(!result.conflicting_evidence());
    assert!(result.abstained());
}

#[test]
fn noisy_correspondence_never_fabricates_the_unobserved_terminal_outcome() {
    let source = source_episode();
    let evidence = vec![
        prefix(910, 1010, 900),
        prefix(910, 1010, 900),
        prefix(910, 1010, 900),
        prefix(920, 1020, 900),
    ];

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &evidence,
        policy(16),
    );

    assert_eq!(result.candidate_count(), 1);
    assert_eq!(
        result.candidates()[0].predicted_outcome(),
        None,
        "poisoned target evidence must not hallucinate the unseen target terminal"
    );
}
