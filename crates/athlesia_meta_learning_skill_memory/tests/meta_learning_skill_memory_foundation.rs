use athlesia_meta_learning_skill_memory::{
    GroundedSkillEpisode, GroundedSkillStep, SkillMemoryFoundation, SkillMemoryPolicy,
    UniversalSkillMemoryFoundation,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

struct TwoStepEpisodeSpec {
    initial_state: CognitiveStructure,
    goal_identity: CognitiveStructure,
    first_action: CognitiveStructure,
    first_outcome: CognitiveStructure,
    second_action: CognitiveStructure,
    second_outcome: CognitiveStructure,
    success_confidence: u16,
    first_step_confidence: u16,
    second_step_confidence: u16,
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(values.iter().copied().map(atom).collect()).unwrap()
}

fn step(
    required_state: CognitiveStructure,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
    confidence: u16,
) -> GroundedSkillStep {
    GroundedSkillStep::new(required_state, action, outcome, signal(confidence)).unwrap()
}

fn two_step_episode(spec: TwoStepEpisodeSpec) -> GroundedSkillEpisode {
    let TwoStepEpisodeSpec {
        initial_state,
        goal_identity,
        first_action,
        first_outcome,
        second_action,
        second_outcome,
        success_confidence,
        first_step_confidence,
        second_step_confidence,
    } = spec;

    GroundedSkillEpisode::new(
        initial_state.clone(),
        goal_identity,
        vec![
            step(
                initial_state,
                first_action,
                first_outcome.clone(),
                first_step_confidence,
            ),
            step(
                first_outcome,
                second_action,
                second_outcome,
                second_step_confidence,
            ),
        ],
        signal(success_confidence),
    )
    .unwrap()
}

fn three_step_episode() -> GroundedSkillEpisode {
    GroundedSkillEpisode::new(
        atom(500),
        atom(1),
        vec![
            step(atom(500), atom(10), atom(110), 1000),
            step(atom(110), atom(11), atom(111), 1000),
            step(atom(111), atom(12), atom(112), 1000),
        ],
        signal(1000),
    )
    .unwrap()
}

fn standard_episode() -> GroundedSkillEpisode {
    two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(500),
        goal_identity: atom(1),
        first_action: atom(10),
        first_outcome: atom(110),
        second_action: atom(11),
        second_outcome: atom(111),
        success_confidence: 1000,
        first_step_confidence: 1000,
        second_step_confidence: 1000,
    })
}

fn policy(
    max_input: usize,
    max_steps: usize,
    max_evaluations: usize,
    max_entries: usize,
    minimum_success: u16,
    minimum_step: u16,
) -> SkillMemoryPolicy {
    SkillMemoryPolicy::new(
        max_input,
        max_steps,
        max_evaluations,
        max_entries,
        signal(minimum_success),
        signal(minimum_step),
    )
    .unwrap()
}

fn default_policy() -> SkillMemoryPolicy {
    policy(32, 16, 32, 32, 500, 500)
}

#[test]
fn foundation_policy_and_grounded_steps_require_positive_bounded_evidence() {
    assert_eq!(
        SkillMemoryPolicy::new(0, 1, 1, 1, signal(1,), signal(1,),),
        None
    );

    assert_eq!(
        SkillMemoryPolicy::new(1, 1, 1, 1, signal(0,), signal(1,),),
        None
    );

    assert_eq!(
        GroundedSkillStep::new(atom(1,), atom(2,), atom(3,), signal(0,),),
        None
    );

    assert!(SkillMemoryPolicy::new(1, 1, 1, 1, signal(1,), signal(1,),).is_some());
}

#[test]
fn grounded_episode_requires_nonempty_exact_state_continuous_sequence() {
    assert_eq!(
        GroundedSkillEpisode::new(atom(500,), atom(1,), Vec::new(), signal(1000,),),
        None
    );

    let wrong_first = vec![step(atom(999), atom(10), atom(110), 1000)];

    assert_eq!(
        GroundedSkillEpisode::new(atom(500,), atom(1,), wrong_first, signal(1000,),),
        None
    );

    let broken_chain = vec![
        step(atom(500), atom(10), atom(110), 1000),
        step(atom(777), atom(11), atom(111), 1000),
    ];

    assert_eq!(
        GroundedSkillEpisode::new(atom(500,), atom(1,), broken_chain, signal(1000,),),
        None
    );

    assert!(GroundedSkillEpisode::new(
        atom(500,),
        atom(1,),
        vec![
            step(atom(500,), atom(10,), atom(110,), 1000,),
            step(atom(110,), atom(11,), atom(111,), 1000,),
        ],
        signal(1000,),
    )
    .is_some());
}

#[test]
fn one_grounded_successful_episode_creates_one_exact_memory_entry() {
    let episode = standard_episode();

    let result = SkillMemoryFoundation::build(std::slice::from_ref(&episode), default_policy());

    assert_eq!(result.entry_count(), 1);

    assert_eq!(result.admitted_episode_count(), 1);

    let entry = &result.entries()[0];

    assert_eq!(entry.support_count(), 1);

    assert_eq!(entry.trace().initial_state(), &atom(500,));

    assert_eq!(entry.trace().goal_identity(), &atom(1,));

    assert_eq!(entry.trace().step_count(), 2);
}

#[test]
fn repeated_exact_trace_aggregates_support_and_conservative_confidence_floors() {
    let strong = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(500),
        goal_identity: atom(1),
        first_action: atom(10),
        first_outcome: atom(110),
        second_action: atom(11),
        second_outcome: atom(111),
        success_confidence: 950,
        first_step_confidence: 900,
        second_step_confidence: 850,
    });

    let weaker_repeat = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(500),
        goal_identity: atom(1),
        first_action: atom(10),
        first_outcome: atom(110),
        second_action: atom(11),
        second_outcome: atom(111),
        success_confidence: 800,
        first_step_confidence: 700,
        second_step_confidence: 750,
    });

    let result = SkillMemoryFoundation::build(&[strong, weaker_repeat], default_policy());

    assert_eq!(result.entry_count(), 1);

    let entry = &result.entries()[0];

    assert_eq!(entry.support_count(), 2);

    assert_eq!(entry.success_confidence_floor(), signal(800,));

    assert_eq!(entry.step_confidence_floor(), signal(700,));
}

#[test]
fn different_action_sequences_remain_distinct_skill_evidence_traces() {
    let first = standard_episode();

    let second = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(500),
        goal_identity: atom(1),
        first_action: atom(20),
        first_outcome: atom(120),
        second_action: atom(21),
        second_outcome: atom(121),
        success_confidence: 1000,
        first_step_confidence: 1000,
        second_step_confidence: 1000,
    });

    let result = SkillMemoryFoundation::build(&[first, second], default_policy());

    assert_eq!(result.entry_count(), 2);

    assert_eq!(result.entries_before_memory_frontier(), 2);
}

#[test]
fn reordered_opaque_action_structure_cannot_impersonate_exact_skill_trace() {
    let exact_action = ordered(&[10, 11]);

    let reordered_action = ordered(&[11, 10]);

    assert_ne!(exact_action, reordered_action);

    let first = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(500),
        goal_identity: atom(1),
        first_action: exact_action,
        first_outcome: atom(110),
        second_action: atom(12),
        second_outcome: atom(112),
        success_confidence: 1000,
        first_step_confidence: 1000,
        second_step_confidence: 1000,
    });

    let second = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(500),
        goal_identity: atom(1),
        first_action: reordered_action,
        first_outcome: atom(110),
        second_action: atom(12),
        second_outcome: atom(112),
        success_confidence: 1000,
        first_step_confidence: 1000,
        second_step_confidence: 1000,
    });

    let result = SkillMemoryFoundation::build(&[first, second], default_policy());

    assert_eq!(result.entry_count(), 2);
}

#[test]
fn context_and_goal_identity_are_semantic_parts_of_exact_skill_evidence() {
    let base = standard_episode();

    let different_context = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(600),
        goal_identity: atom(1),
        first_action: atom(10),
        first_outcome: atom(110),
        second_action: atom(11),
        second_outcome: atom(111),
        success_confidence: 1000,
        first_step_confidence: 1000,
        second_step_confidence: 1000,
    });

    let different_goal = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(500),
        goal_identity: atom(2),
        first_action: atom(10),
        first_outcome: atom(110),
        second_action: atom(11),
        second_outcome: atom(111),
        success_confidence: 1000,
        first_step_confidence: 1000,
        second_step_confidence: 1000,
    });

    let result =
        SkillMemoryFoundation::build(&[base, different_context, different_goal], default_policy());

    assert_eq!(result.entry_count(), 3);
}

#[test]
fn low_success_confidence_is_rejected_before_entering_skill_memory() {
    let weak = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(500),
        goal_identity: atom(1),
        first_action: atom(10),
        first_outcome: atom(110),
        second_action: atom(11),
        second_outcome: atom(111),
        success_confidence: 499,
        first_step_confidence: 1000,
        second_step_confidence: 1000,
    });

    let result = SkillMemoryFoundation::build(std::slice::from_ref(&weak), default_policy());

    assert_eq!(result.rejected_threshold_count(), 1);

    assert!(result.abstained());
}

#[test]
fn weak_step_evidence_rejects_episode_even_when_terminal_success_is_high() {
    let weak_step = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(500),
        goal_identity: atom(1),
        first_action: atom(10),
        first_outcome: atom(110),
        second_action: atom(11),
        second_outcome: atom(111),
        success_confidence: 1000,
        first_step_confidence: 1000,
        second_step_confidence: 499,
    });

    let result = SkillMemoryFoundation::build(std::slice::from_ref(&weak_step), default_policy());

    assert_eq!(result.rejected_threshold_count(), 1);

    assert_eq!(result.entry_count(), 0);
}

#[test]
fn overlong_episode_is_rejected_by_hard_step_frontier() {
    let episode = three_step_episode();

    assert_eq!(episode.step_count(), 3);

    let result = SkillMemoryFoundation::build(
        std::slice::from_ref(&episode),
        policy(32, 2, 32, 32, 500, 500),
    );

    assert_eq!(result.rejected_step_bound_count(), 1);

    assert_eq!(result.entry_count(), 0);
}

#[test]
fn hard_episode_evaluation_and_memory_frontiers_are_deterministic() {
    let strongest = standard_episode();

    let medium = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(600),
        goal_identity: atom(1),
        first_action: atom(20),
        first_outcome: atom(120),
        second_action: atom(21),
        second_outcome: atom(121),
        success_confidence: 900,
        first_step_confidence: 900,
        second_step_confidence: 900,
    });

    let weakest = two_step_episode(TwoStepEpisodeSpec {
        initial_state: atom(700),
        goal_identity: atom(1),
        first_action: atom(30),
        first_outcome: atom(130),
        second_action: atom(31),
        second_outcome: atom(131),
        success_confidence: 800,
        first_step_confidence: 800,
        second_step_confidence: 800,
    });

    let episode_limited = SkillMemoryFoundation::build(
        &[weakest.clone(), strongest.clone(), medium.clone()],
        policy(1, 16, 32, 32, 500, 500),
    );

    assert_eq!(episode_limited.input_episode_count(), 3);

    assert_eq!(episode_limited.considered_episode_count(), 1);

    assert!(episode_limited.episode_frontier_truncated());

    assert_eq!(
        episode_limited.entries()[0].trace().initial_state(),
        &atom(500,)
    );

    let evaluation_limited = SkillMemoryFoundation::build(
        &[strongest.clone(), medium.clone(), weakest.clone()],
        policy(32, 16, 1, 32, 500, 500),
    );

    assert_eq!(evaluation_limited.episode_evaluation_count(), 1);

    assert!(evaluation_limited.episode_evaluation_truncated());

    assert_eq!(evaluation_limited.entry_count(), 1);

    let memory_limited = SkillMemoryFoundation::build(
        &[weakest, medium, strongest],
        policy(32, 16, 32, 1, 500, 500),
    );

    assert_eq!(memory_limited.entries_before_memory_frontier(), 3);

    assert!(memory_limited.memory_frontier_truncated());

    assert_eq!(memory_limited.entry_count(), 1);

    assert_eq!(
        memory_limited.entries()[0].trace().initial_state(),
        &atom(500,)
    );
}

#[test]
fn foundation_is_order_invariant_non_mutating_and_facade_equivalent() {
    let episodes = vec![
        standard_episode(),
        two_step_episode(TwoStepEpisodeSpec {
            initial_state: atom(600),
            goal_identity: atom(1),
            first_action: atom(20),
            first_outcome: atom(120),
            second_action: atom(21),
            second_outcome: atom(121),
            success_confidence: 900,
            first_step_confidence: 900,
            second_step_confidence: 900,
        }),
        standard_episode(),
    ];

    let episodes_before = episodes.clone();

    let mut reversed = episodes.clone();

    reversed.reverse();

    let memory_policy = default_policy();

    let direct = SkillMemoryFoundation::build(&episodes, memory_policy);

    let reordered = SkillMemoryFoundation::build(&reversed, memory_policy);

    let facade = UniversalSkillMemoryFoundation::evaluate(&episodes, memory_policy);

    let repeated = UniversalSkillMemoryFoundation::evaluate(&episodes, memory_policy);

    assert_eq!(direct, reordered);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, episodes_before);

    assert_eq!(direct.entries()[0].support_count(), 2);
}
