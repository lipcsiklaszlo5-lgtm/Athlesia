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
    GroundedEpisodicAnalogyPolicy::new(16, s(500)).expect("positive bounded analogy policy")
}

fn state_changing_source() -> GroundedSkillEpisode {
    GroundedSkillEpisode::new(
        a(100),
        a(700),
        vec![
            GroundedSkillStep::new(a(100), a(110), a(210), s(900))
                .expect("valid state-changing prefix"),
            GroundedSkillStep::new(a(210), a(700), a(999), s(900)).expect("valid continuation"),
        ],
        s(900),
    )
    .expect("valid source episode")
}

fn self_loop_source() -> GroundedSkillEpisode {
    GroundedSkillEpisode::new(
        a(100),
        a(700),
        vec![
            GroundedSkillStep::new(a(100), a(110), a(100), s(900)).expect("valid self-loop prefix"),
            GroundedSkillStep::new(a(100), a(700), a(999), s(900)).expect("valid continuation"),
        ],
        s(900),
    )
    .expect("valid source episode")
}

#[test]
fn state_changing_prefix_rejects_self_loops_before_correspondence_voting() {
    let source = state_changing_source();

    let evidence = vec![
        observation(900, 901, 900),
        observation(900, 902, 900),
        observation(900, 910, 1010),
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
    assert_eq!(forward.considered_observation_count(), 1);
    assert_eq!(forward.candidate_count(), 1);
    assert!(!forward.conflicting_evidence());
    assert!(!forward.correspondence_conflict());

    let candidate = &forward.candidates()[0];

    assert_eq!(candidate.required_state(), &a(1010));
    assert_eq!(candidate.action(), &a(20_000));
    assert_eq!(candidate.predicted_outcome(), None);
}

#[test]
fn self_loop_prefix_rejects_state_changes_before_correspondence_voting() {
    let source = self_loop_source();

    let evidence = vec![observation(900, 910, 1010), observation(900, 920, 900)];

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
    assert_eq!(forward.input_observation_count(), 2);
    assert_eq!(forward.considered_observation_count(), 1);
    assert_eq!(forward.candidate_count(), 1);
    assert!(!forward.conflicting_evidence());
    assert!(!forward.correspondence_conflict());

    let candidate = &forward.candidates()[0];

    assert_eq!(candidate.required_state(), &a(900));
    assert_eq!(candidate.action(), &a(20_000));
    assert_eq!(candidate.predicted_outcome(), None);
}
