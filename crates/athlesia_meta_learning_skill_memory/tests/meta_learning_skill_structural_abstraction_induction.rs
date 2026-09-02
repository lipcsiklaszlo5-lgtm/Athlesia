use athlesia_meta_learning_skill_memory::{
    GroundedSkillEpisode, GroundedSkillStep, RepeatedSkillCandidate,
    RepeatedSkillCandidateDiscovery, RepeatedSkillCandidatePolicy, SkillMemoryFoundation,
    SkillMemoryPolicy, StructuralSkillAbstractionInduction, StructuralSkillAbstractionPolicy,
    StructuralSkillTerm, UniversalStructuralSkillAbstractionInduction,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone)]
struct TraceSpec {
    initial: CognitiveStructure,
    goal: CognitiveStructure,
    a1: CognitiveStructure,
    o1: CognitiveStructure,
    a2: CognitiveStructure,
    o2: CognitiveStructure,
}

struct AbstractionPolicySpec {
    input: usize,
    pairs: usize,
    steps: usize,
    abstractions: usize,
    support: usize,
    success: u16,
    confidence: u16,
}

fn s(v: u16) -> CognitiveSignal {
    CognitiveSignal::new(v).unwrap()
}
fn a(v: u64) -> CognitiveStructure {
    CognitiveStructure::atom(v)
}

fn ordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(values.iter().copied().map(a).collect()).unwrap()
}

fn base() -> TraceSpec {
    TraceSpec {
        initial: a(500),
        goal: a(1),
        a1: a(10),
        o1: a(110),
        a2: a(11),
        o2: a(111),
    }
}

fn step(
    state: CognitiveStructure,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
    confidence: u16,
) -> GroundedSkillStep {
    GroundedSkillStep::new(state, action, outcome, s(confidence)).unwrap()
}

fn episode(spec: &TraceSpec, success: u16, confidence: u16) -> GroundedSkillEpisode {
    GroundedSkillEpisode::new(
        spec.initial.clone(),
        spec.goal.clone(),
        vec![
            step(
                spec.initial.clone(),
                spec.a1.clone(),
                spec.o1.clone(),
                confidence,
            ),
            step(
                spec.o1.clone(),
                spec.a2.clone(),
                spec.o2.clone(),
                confidence,
            ),
        ],
        s(success),
    )
    .unwrap()
}

fn candidate(
    spec: &TraceSpec,
    support: usize,
    success: u16,
    confidence: u16,
) -> RepeatedSkillCandidate {
    let episodes: Vec<_> = (0..support)
        .map(|_| episode(spec, success, confidence))
        .collect();

    let memory = SkillMemoryFoundation::build(
        &episodes,
        SkillMemoryPolicy::new(64, 16, 64, 64, s(1), s(1)).unwrap(),
    );

    let discovered = RepeatedSkillCandidateDiscovery::discover(
        memory.entries(),
        RepeatedSkillCandidatePolicy::new(64, 64, 16, 64, 2, s(1), s(1)).unwrap(),
    );

    discovered.candidates()[0].clone()
}

fn three_step_candidate() -> RepeatedSkillCandidate {
    let make = || {
        GroundedSkillEpisode::new(
            a(500),
            a(1),
            vec![
                step(a(500), a(10), a(110), 1000),
                step(a(110), a(11), a(111), 1000),
                step(a(111), a(12), a(112), 1000),
            ],
            s(1000),
        )
        .unwrap()
    };

    let memory = SkillMemoryFoundation::build(
        &[make(), make()],
        SkillMemoryPolicy::new(64, 16, 64, 64, s(1), s(1)).unwrap(),
    );

    RepeatedSkillCandidateDiscovery::discover(
        memory.entries(),
        RepeatedSkillCandidatePolicy::new(64, 64, 16, 64, 2, s(1), s(1)).unwrap(),
    )
    .candidates()[0]
        .clone()
}

fn policy(spec: AbstractionPolicySpec) -> StructuralSkillAbstractionPolicy {
    StructuralSkillAbstractionPolicy::new(
        spec.input,
        spec.pairs,
        spec.steps,
        spec.abstractions,
        spec.support,
        s(spec.success),
        s(spec.confidence),
    )
    .unwrap()
}

fn default_policy() -> StructuralSkillAbstractionPolicy {
    policy(AbstractionPolicySpec {
        input: 32,
        pairs: 64,
        steps: 16,
        abstractions: 32,
        support: 2,
        success: 500,
        confidence: 500,
    })
}

#[test]
fn abstraction_policy_requires_multiple_candidates_and_positive_bounds() {
    assert_eq!(
        StructuralSkillAbstractionPolicy::new(1, 1, 1, 1, 2, s(1), s(1)),
        None
    );
    assert_eq!(
        StructuralSkillAbstractionPolicy::new(2, 1, 1, 1, 1, s(1), s(1)),
        None
    );
    assert!(StructuralSkillAbstractionPolicy::new(2, 1, 1, 1, 2, s(1), s(1)).is_some());
}

#[test]
fn duplicate_exact_candidate_cannot_fake_multi_source_abstraction() {
    let x = candidate(&base(), 2, 1000, 1000);

    let result = StructuralSkillAbstractionInduction::induce(&[x.clone(), x], default_policy());

    assert_eq!(result.input_candidate_count(), 2);
    assert_eq!(result.unique_candidate_count(), 1);
    assert_eq!(result.abstraction_count(), 0);
}

#[test]
fn two_distinct_repeated_candidates_induce_one_structural_abstraction() {
    let left = candidate(&base(), 3, 900, 900);

    let mut right_spec = base();
    right_spec.initial = a(600);
    right_spec.a1 = a(20);
    right_spec.o1 = a(120);

    let right = candidate(&right_spec, 2, 800, 850);

    let result = StructuralSkillAbstractionInduction::induce(&[left, right], default_policy());

    assert_eq!(result.abstraction_count(), 1);

    let evidence = &result.abstractions()[0];

    assert_eq!(evidence.source_pair_count(), 1);
    assert_eq!(evidence.source_support_sum(), 5);
    assert_eq!(evidence.success_confidence_floor(), s(800));
    assert_eq!(evidence.step_confidence_floor(), s(850));
}

#[test]
fn exact_shared_fields_remain_invariants() {
    let left = candidate(&base(), 2, 1000, 1000);

    let mut spec = base();
    spec.initial = a(600);
    spec.a1 = a(20);
    spec.o1 = a(120);

    let right = candidate(&spec, 2, 1000, 1000);

    let result = StructuralSkillAbstractionInduction::induce(&[left, right], default_policy());

    let abstraction = result.abstractions()[0].abstraction();

    assert_eq!(abstraction.goal_identity().invariant(), Some(&a(1)));

    assert_eq!(abstraction.steps()[1].action().invariant(), Some(&a(11)));

    assert_eq!(
        abstraction.steps()[1].observed_outcome().invariant(),
        Some(&a(111))
    );
}

#[test]
fn differing_values_become_variables_and_preserve_equality_relations() {
    let left = candidate(&base(), 2, 1000, 1000);

    let mut spec = base();
    spec.initial = a(600);

    let right = candidate(&spec, 2, 1000, 1000);

    let result = StructuralSkillAbstractionInduction::induce(&[left, right], default_policy());

    let abstraction = result.abstractions()[0].abstraction();

    let initial_variable = abstraction.initial_state().variable_id().unwrap();
    let required_variable = abstraction.steps()[0]
        .required_state()
        .variable_id()
        .unwrap();

    assert_eq!(initial_variable, required_variable);
    assert_eq!(abstraction.variable_count(), 1);
}

#[test]
fn reordered_opaque_action_is_variable_not_false_invariant() {
    let mut x = base();
    x.a1 = ordered(&[10, 11]);

    let mut y = base();
    y.a1 = ordered(&[11, 10]);

    assert_ne!(x.a1, y.a1);

    let result = StructuralSkillAbstractionInduction::induce(
        &[candidate(&x, 2, 1000, 1000), candidate(&y, 2, 1000, 1000)],
        default_policy(),
    );

    let action = result.abstractions()[0].abstraction().steps()[0].action();

    assert!(matches!(action, StructuralSkillTerm::Variable(_)));
    assert_eq!(action.invariant(), None);
}

#[test]
fn step_count_mismatch_is_not_force_aligned() {
    let two = candidate(&base(), 2, 1000, 1000);
    let three = three_step_candidate();

    let result = StructuralSkillAbstractionInduction::induce(&[two, three], default_policy());

    assert_eq!(result.pair_evaluation_count(), 1);
    assert_eq!(result.rejected_step_mismatch_count(), 1);
    assert!(result.abstained());
}

#[test]
fn insufficient_candidate_support_is_rejected_before_pairing() {
    let x = candidate(&base(), 2, 1000, 1000);

    let mut spec = base();
    spec.initial = a(600);
    let y = candidate(&spec, 3, 1000, 1000);

    let result = StructuralSkillAbstractionInduction::induce(
        &[x, y],
        policy(AbstractionPolicySpec {
            input: 32,
            pairs: 64,
            steps: 16,
            abstractions: 32,
            support: 3,
            success: 500,
            confidence: 500,
        }),
    );

    assert_eq!(result.rejected_support_count(), 1);
    assert_eq!(result.pair_evaluation_count(), 0);
    assert!(result.abstained());
}

#[test]
fn weak_candidate_evidence_is_rejected_before_abstraction() {
    let x = candidate(&base(), 2, 400, 1000);

    let mut spec = base();
    spec.initial = a(600);
    let y = candidate(&spec, 2, 1000, 1000);

    let result = StructuralSkillAbstractionInduction::induce(&[x, y], default_policy());

    assert_eq!(result.rejected_threshold_count(), 1);
    assert_eq!(result.abstraction_count(), 0);
}

#[test]
fn hard_candidate_frontier_selects_stronger_sources_deterministically() {
    let first = candidate(&base(), 4, 1000, 1000);

    let mut b = base();
    b.initial = a(600);
    b.a1 = a(20);
    let second = candidate(&b, 3, 1000, 1000);

    let mut c = base();
    c.initial = a(700);
    c.a1 = a(30);
    let third = candidate(&c, 2, 1000, 1000);

    let result = StructuralSkillAbstractionInduction::induce(
        &[third, second, first],
        policy(AbstractionPolicySpec {
            input: 2,
            pairs: 64,
            steps: 16,
            abstractions: 32,
            support: 2,
            success: 500,
            confidence: 500,
        }),
    );

    assert_eq!(result.unique_candidate_count(), 3);
    assert_eq!(result.considered_candidate_count(), 2);
    assert!(result.candidate_frontier_truncated());
    assert_eq!(result.pair_evaluation_count(), 1);
}

#[test]
fn hard_pair_and_final_abstraction_frontiers_are_enforced() {
    let first = candidate(&base(), 4, 1000, 1000);

    let mut b = base();
    b.initial = a(600);
    b.a1 = a(20);
    let second = candidate(&b, 3, 1000, 1000);

    let mut c = base();
    c.initial = a(700);
    c.goal = a(2);
    c.a2 = a(31);
    let third = candidate(&c, 2, 1000, 1000);

    let pair_limited = StructuralSkillAbstractionInduction::induce(
        &[first.clone(), second.clone(), third.clone()],
        policy(AbstractionPolicySpec {
            input: 32,
            pairs: 1,
            steps: 16,
            abstractions: 32,
            support: 2,
            success: 500,
            confidence: 500,
        }),
    );

    assert_eq!(pair_limited.pair_evaluation_count(), 1);
    assert!(pair_limited.pair_evaluation_truncated());

    let final_limited = StructuralSkillAbstractionInduction::induce(
        &[first, second, third],
        policy(AbstractionPolicySpec {
            input: 32,
            pairs: 64,
            steps: 16,
            abstractions: 1,
            support: 2,
            success: 500,
            confidence: 500,
        }),
    );

    assert!(final_limited.abstractions_before_frontier() >= 2);
    assert!(final_limited.abstraction_frontier_truncated());
    assert_eq!(final_limited.abstraction_count(), 1);
}

#[test]
fn induction_is_order_invariant_non_mutating_and_facade_equivalent() {
    let first = candidate(&base(), 3, 900, 900);

    let mut spec = base();
    spec.initial = a(600);
    spec.a1 = a(20);
    spec.o1 = a(120);

    let second = candidate(&spec, 2, 1000, 1000);

    let candidates = vec![second, first];
    let before = candidates.clone();

    let mut reversed = candidates.clone();
    reversed.reverse();

    let p = default_policy();

    let direct = StructuralSkillAbstractionInduction::induce(&candidates, p);
    let reordered = StructuralSkillAbstractionInduction::induce(&reversed, p);
    let facade = UniversalStructuralSkillAbstractionInduction::evaluate(&candidates, p);
    let repeated = UniversalStructuralSkillAbstractionInduction::evaluate(&candidates, p);

    assert_eq!(direct, reordered);
    assert_eq!(direct, facade);
    assert_eq!(facade, repeated);
    assert_eq!(candidates, before);
}
