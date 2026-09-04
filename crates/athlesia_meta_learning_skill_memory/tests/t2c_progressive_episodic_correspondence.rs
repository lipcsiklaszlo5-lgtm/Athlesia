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

fn observation(state: u64, action: u64, outcome: u64) -> SkillExecutionObservation {
    SkillExecutionObservation::new(a(state), a(action), a(outcome), s(900))
        .expect("positive grounded observation")
}

fn policy() -> GroundedEpisodicAnalogyPolicy {
    GroundedEpisodicAnalogyPolicy::new(32, s(500)).expect("positive bounded analogy policy")
}

fn three_step_source() -> GroundedSkillEpisode {
    GroundedSkillEpisode::new(
        a(100),
        a(700),
        vec![
            GroundedSkillStep::new(a(100), a(110), a(210), s(900))
                .expect("valid first source transition"),
            GroundedSkillStep::new(a(210), a(700), a(310), s(900))
                .expect("valid second source transition"),
            GroundedSkillStep::new(a(310), a(700), a(999), s(900))
                .expect("valid third source transition"),
        ],
        s(900),
    )
    .expect("valid three-step source episode")
}

#[test]
fn two_grounded_target_transitions_advance_to_third_source_step_independent_of_input_order() {
    let source = three_step_source();

    let evidence = vec![
        observation(900, 901, 900),
        observation(900, 910, 1010),
        observation(1010, 20_000, 1110),
    ];

    let forward = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &evidence,
        policy(),
    );

    let mut reversed = evidence;
    reversed.reverse();

    let backward = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &reversed,
        policy(),
    );

    assert_eq!(forward, backward);
    assert_eq!(forward.input_observation_count(), 3);
    assert_eq!(forward.considered_observation_count(), 2);
    assert_eq!(forward.candidate_count(), 1);
    assert!(!forward.conflicting_evidence());
    assert!(!forward.correspondence_conflict());

    let candidate = &forward.candidates()[0];

    assert_eq!(candidate.source_step_index(), 2);
    assert_eq!(candidate.required_state(), &a(1110));
    assert_eq!(candidate.action(), &a(20_000));
    assert_eq!(candidate.predicted_outcome(), None);
}

#[test]
fn disconnected_future_observation_cannot_fake_progressive_depth() {
    let source = three_step_source();

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &[observation(900, 910, 1010), observation(5555, 20_000, 6666)],
        policy(),
    );

    assert_eq!(result.candidate_count(), 1);

    let candidate = &result.candidates()[0];

    assert_eq!(candidate.source_step_index(), 1);
    assert_eq!(candidate.required_state(), &a(1010));
    assert_eq!(candidate.action(), &a(20_000));
    assert_eq!(candidate.predicted_outcome(), None);
}

#[test]
fn balanced_conflict_at_second_depth_forces_abstention() {
    let source = three_step_source();

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &[
            observation(900, 910, 1010),
            observation(1010, 20_000, 1110),
            observation(1010, 20_001, 1120),
        ],
        policy(),
    );

    assert_eq!(result.candidate_count(), 0);
    assert!(result.conflicting_evidence());
    assert!(!result.correspondence_conflict());
    assert!(result.abstained());
}

#[test]
fn repeated_self_loop_evidence_cannot_be_reused_as_multiple_source_steps_without_provenance() {
    let source = GroundedSkillEpisode::new(
        a(100),
        a(700),
        vec![
            GroundedSkillStep::new(a(100), a(110), a(100), s(900))
                .expect("valid self-loop source prefix"),
            GroundedSkillStep::new(a(100), a(700), a(210), s(900))
                .expect("valid state-changing continuation"),
            GroundedSkillStep::new(a(210), a(700), a(999), s(900))
                .expect("valid final continuation"),
        ],
        s(900),
    )
    .expect("valid self-loop source episode");

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &[observation(900, 910, 900), observation(900, 910, 900)],
        policy(),
    );

    assert_eq!(result.candidate_count(), 1);

    let candidate = &result.candidates()[0];

    assert_eq!(candidate.source_step_index(), 1);
    assert_eq!(candidate.required_state(), &a(900));
    assert_eq!(candidate.action(), &a(20_000));
    assert_eq!(candidate.predicted_outcome(), None);
}
