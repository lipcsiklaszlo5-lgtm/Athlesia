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

fn source_episode(goal: u64, second_action: u64) -> GroundedSkillEpisode {
    GroundedSkillEpisode::new(
        a(100),
        a(goal),
        vec![
            GroundedSkillStep::new(a(100), a(110), a(210), s(1000))
                .expect("valid first source step"),
            GroundedSkillStep::new(a(210), a(second_action), a(999), s(1000))
                .expect("valid second source step"),
        ],
        s(1000),
    )
    .expect("valid source episode")
}

fn prefix(state: u64, action: u64, outcome: u64, confidence: u16) -> SkillExecutionObservation {
    SkillExecutionObservation::new(a(state), a(action), a(outcome), s(confidence))
        .expect("valid observed prefix")
}

fn policy() -> GroundedEpisodicAnalogyPolicy {
    GroundedEpisodicAnalogyPolicy::new(16, s(500)).expect("valid analogy policy")
}

#[test]
fn one_source_episode_plus_one_novel_prefix_transfers_the_next_relational_action() {
    let source = source_episode(700, 700);

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &[prefix(900, 910, 1010, 900)],
        policy(),
    );

    assert_eq!(result.candidate_count(), 1);

    let candidate = &result.candidates()[0];

    assert_eq!(candidate.source_step_index(), 1);
    assert_eq!(candidate.required_state(), &a(1010));
    assert_eq!(candidate.action(), &a(20_000));

    assert_eq!(
        candidate.predicted_outcome(),
        None,
        "unobserved target terminal outcome must remain explicitly unknown",
    );
}

#[test]
fn target_world_can_use_entirely_disjoint_atom_identities() {
    let source = source_episode(700, 700);

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(90_000),
        &a(80_000),
        &[prefix(90_000, 91_000, 92_000, 900)],
        policy(),
    );

    assert_eq!(result.candidate_count(), 1);
    assert_eq!(result.candidates()[0].required_state(), &a(92_000));
    assert_eq!(result.candidates()[0].action(), &a(80_000));
    assert_eq!(result.candidates()[0].predicted_outcome(), None);
}

#[test]
fn one_episode_does_not_license_an_unrelated_unbound_next_action() {
    let source = source_episode(700, 701);

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &[prefix(900, 910, 1010, 900)],
        policy(),
    );

    assert_eq!(
        result.candidate_count(),
        0,
        "a source action unrelated to a grounded anchor must not be guessed in the target world",
    );
    assert!(result.abstained());
}

#[test]
fn no_target_prefix_means_no_cross_world_correspondence() {
    let source = source_episode(700, 700);

    let result =
        GroundedEpisodicAnalogyTransfer::infer_next(&source, &a(900), &a(20_000), &[], policy());

    assert_eq!(result.candidate_count(), 0);
    assert!(result.abstained());
}

#[test]
fn weak_target_evidence_cannot_trigger_one_shot_transfer() {
    let source = source_episode(700, 700);

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &[prefix(900, 910, 1010, 400)],
        policy(),
    );

    assert_eq!(result.candidate_count(), 0);
    assert!(result.abstained());
    assert_eq!(result.rejected_low_confidence_count(), 1);
}

#[test]
fn incompatible_strong_prefixes_remain_unresolved_instead_of_selecting_by_order() {
    let source = source_episode(700, 700);

    let evidence = vec![prefix(900, 910, 1010, 900), prefix(900, 920, 1020, 900)];

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
    assert_eq!(forward.candidate_count(), 0);
    assert!(!forward.observation_frontier_exceeded());
    assert!(forward.conflicting_evidence());
    assert!(forward.abstained());
}

#[test]
fn repeated_consistent_target_prefix_strengthens_one_candidate_without_duplication() {
    let source = source_episode(700, 700);

    let evidence = vec![
        prefix(900, 910, 1010, 900),
        prefix(900, 910, 1010, 900),
        prefix(900, 910, 1010, 900),
    ];

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &evidence,
        policy(),
    );

    assert_eq!(result.candidate_count(), 1);
    assert!(!result.conflicting_evidence());
    assert_eq!(result.candidates()[0].required_state(), &a(1010));
    assert_eq!(result.candidates()[0].action(), &a(20_000));
}

#[test]
fn dominant_reproducible_prefix_survives_one_strong_contradictory_outlier_independent_of_order() {
    let source = source_episode(700, 700);

    let evidence = vec![
        prefix(900, 910, 1010, 900),
        prefix(900, 910, 1010, 900),
        prefix(900, 910, 1010, 900),
        prefix(900, 920, 1020, 900),
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

    assert_eq!(
        forward, backward,
        "the same evidence multiset must produce the same analogy regardless of presentation order",
    );

    assert_eq!(forward.candidate_count(), 1);
    assert!(!forward.conflicting_evidence());
    assert_eq!(forward.candidates()[0].required_state(), &a(1010));
    assert_eq!(forward.candidates()[0].action(), &a(20_000));
}

#[test]
fn balanced_reproducible_competing_prefixes_remain_unresolved_independent_of_order() {
    let source = source_episode(700, 700);

    let evidence = vec![
        prefix(900, 910, 1010, 900),
        prefix(900, 910, 1010, 900),
        prefix(900, 920, 1020, 900),
        prefix(900, 920, 1020, 900),
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
    assert_eq!(forward.candidate_count(), 0);
    assert!(!forward.observation_frontier_exceeded());
    assert!(forward.conflicting_evidence());
    assert!(forward.abstained());
}

#[test]
fn unrelated_target_observations_do_not_corrupt_the_grounded_prefix() {
    let source = source_episode(700, 700);

    let evidence = vec![
        prefix(7777, 7778, 7779, 900),
        prefix(900, 910, 1010, 900),
        prefix(8888, 8889, 8890, 900),
    ];

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &evidence,
        policy(),
    );

    assert_eq!(result.candidate_count(), 1);
    assert!(!result.conflicting_evidence());
    assert_eq!(result.candidates()[0].required_state(), &a(1010));
    assert_eq!(result.candidates()[0].action(), &a(20_000));
}

#[test]
fn bounded_prefix_frontier_cannot_turn_input_order_into_semantic_authority() {
    let source = source_episode(700, 700);

    let bounded_forward =
        GroundedEpisodicAnalogyPolicy::new(2, s(500)).expect("valid bounded analogy policy");
    let bounded_backward =
        GroundedEpisodicAnalogyPolicy::new(2, s(500)).expect("valid bounded analogy policy");

    let evidence = vec![
        prefix(900, 910, 1010, 900),
        prefix(900, 920, 1020, 900),
        prefix(900, 930, 1030, 900),
    ];

    let forward = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &evidence,
        bounded_forward,
    );

    let mut reversed = evidence;
    reversed.reverse();

    let backward = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &reversed,
        bounded_backward,
    );

    assert_eq!(
        forward, backward,
        "capacity pressure must not make the first observed correspondence authoritative",
    );

    assert_eq!(forward.candidate_count(), 0);
    assert!(forward.observation_frontier_exceeded());
    assert!(!forward.conflicting_evidence());
    assert!(forward.abstained());
}

#[test]
fn one_shot_transfer_uses_relational_correspondence_not_only_the_goal_anchor() {
    let source = source_episode(700, 110);

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &[prefix(900, 910, 1010, 900)],
        policy(),
    );

    assert_eq!(result.candidate_count(), 1);
    assert_eq!(
        result.candidates()[0].action(),
        &a(910),
        "a source next-action identical to the source prefix action must map to the target prefix action",
    );
}

#[test]
fn grounded_relational_future_outcome_is_preserved_when_it_is_already_known() {
    let source = GroundedSkillEpisode::new(
        a(100),
        a(700),
        vec![
            GroundedSkillStep::new(a(100), a(110), a(210), s(1000)).expect("valid source prefix"),
            GroundedSkillStep::new(a(210), a(700), a(210), s(1000))
                .expect("valid source continuation"),
        ],
        s(1000),
    )
    .expect("valid source episode");

    let result = GroundedEpisodicAnalogyTransfer::infer_next(
        &source,
        &a(900),
        &a(20_000),
        &[prefix(900, 910, 1010, 900)],
        policy(),
    );

    assert_eq!(result.candidate_count(), 1);
    assert_eq!(result.candidates()[0].action(), &a(20_000));
    assert_eq!(
        result.candidates()[0].predicted_outcome(),
        Some(&a(1010)),
        "a future term already grounded by the observed correspondence must not be erased into unknown",
    );
}
