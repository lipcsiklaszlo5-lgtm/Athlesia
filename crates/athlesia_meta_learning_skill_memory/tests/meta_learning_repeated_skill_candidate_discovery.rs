use athlesia_meta_learning_skill_memory::{
    GroundedSkillEpisode, GroundedSkillStep, RepeatedSkillCandidateDiscovery,
    RepeatedSkillCandidatePolicy, SkillMemoryEntry, SkillMemoryFoundation, SkillMemoryPolicy,
    UniversalRepeatedSkillCandidateDiscovery,
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

struct PolicySpec {
    input: usize,
    eval: usize,
    steps: usize,
    candidates: usize,
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

fn ordered(v: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(v.iter().copied().map(a).collect()).unwrap()
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

fn mem_policy() -> SkillMemoryPolicy {
    SkillMemoryPolicy::new(128, 32, 128, 128, s(1), s(1)).unwrap()
}

fn entry(spec: &TraceSpec, support: usize, success: u16, confidence: u16) -> SkillMemoryEntry {
    let eps: Vec<_> = (0..support)
        .map(|_| episode(spec, success, confidence))
        .collect();
    SkillMemoryFoundation::build(&eps, mem_policy()).entries()[0].clone()
}

fn three_step() -> SkillMemoryEntry {
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

    SkillMemoryFoundation::build(&[make(), make()], mem_policy()).entries()[0].clone()
}

fn pol(p: PolicySpec) -> RepeatedSkillCandidatePolicy {
    RepeatedSkillCandidatePolicy::new(
        p.input,
        p.eval,
        p.steps,
        p.candidates,
        p.support,
        s(p.success),
        s(p.confidence),
    )
    .unwrap()
}

fn default_policy() -> RepeatedSkillCandidatePolicy {
    pol(PolicySpec {
        input: 32,
        eval: 32,
        steps: 16,
        candidates: 32,
        support: 2,
        success: 500,
        confidence: 500,
    })
}

#[test]
fn policy_requires_positive_bounds_and_real_repetition() {
    assert_eq!(
        RepeatedSkillCandidatePolicy::new(0, 1, 1, 1, 2, s(1), s(1)),
        None
    );
    assert_eq!(
        RepeatedSkillCandidatePolicy::new(1, 1, 1, 1, 1, s(1), s(1)),
        None
    );
    assert!(RepeatedSkillCandidatePolicy::new(1, 1, 1, 1, 2, s(1), s(1)).is_some());
}

#[test]
fn one_success_remains_evidence_not_candidate() {
    let e = entry(&base(), 1, 1000, 1000);
    let r = RepeatedSkillCandidateDiscovery::discover(std::slice::from_ref(&e), default_policy());
    assert_eq!(r.rejected_support_count(), 1);
    assert!(r.abstained());
}

#[test]
fn repeated_exact_trace_becomes_candidate() {
    let e = entry(&base(), 2, 900, 850);
    let r = RepeatedSkillCandidateDiscovery::discover(std::slice::from_ref(&e), default_policy());
    assert_eq!(r.candidate_count(), 1);
    assert_eq!(r.candidates()[0].support_count(), 2);
    assert_eq!(r.candidates()[0].conservative_evidence_floor(), s(850));
}

#[test]
fn configurable_support_threshold_is_enforced() {
    let e = entry(&base(), 2, 1000, 1000);
    let r = RepeatedSkillCandidateDiscovery::discover(
        std::slice::from_ref(&e),
        pol(PolicySpec {
            input: 32,
            eval: 32,
            steps: 16,
            candidates: 32,
            support: 3,
            success: 500,
            confidence: 500,
        }),
    );
    assert_eq!(r.rejected_support_count(), 1);
    assert!(r.abstained());
}

#[test]
fn weak_success_floor_blocks_promotion() {
    let e = entry(&base(), 2, 400, 1000);
    let r = RepeatedSkillCandidateDiscovery::discover(std::slice::from_ref(&e), default_policy());
    assert_eq!(r.rejected_threshold_count(), 1);
    assert!(r.abstained());
}

#[test]
fn weak_step_floor_blocks_promotion() {
    let e = entry(&base(), 2, 1000, 400);
    let r = RepeatedSkillCandidateDiscovery::discover(std::slice::from_ref(&e), default_policy());
    assert_eq!(r.rejected_threshold_count(), 1);
    assert!(r.abstained());
}

#[test]
fn overlong_repeated_trace_hits_step_frontier() {
    let e = three_step();
    let r = RepeatedSkillCandidateDiscovery::discover(
        std::slice::from_ref(&e),
        pol(PolicySpec {
            input: 32,
            eval: 32,
            steps: 2,
            candidates: 32,
            support: 2,
            success: 500,
            confidence: 500,
        }),
    );
    assert_eq!(r.rejected_step_bound_count(), 1);
    assert!(r.abstained());
}

#[test]
fn distinct_exact_traces_remain_distinct_candidates() {
    let first = entry(&base(), 2, 1000, 1000);
    let mut other = base();
    other.initial = a(600);
    other.a1 = a(20);
    other.o1 = a(120);
    other.a2 = a(21);
    other.o2 = a(121);
    let second = entry(&other, 2, 1000, 1000);

    let r = RepeatedSkillCandidateDiscovery::discover(&[first, second], default_policy());
    assert_eq!(r.candidate_count(), 2);
}

#[test]
fn reordered_opaque_action_remains_distinct() {
    let mut x = base();
    x.a1 = ordered(&[10, 11]);

    let mut y = base();
    y.a1 = ordered(&[11, 10]);

    assert_ne!(x.a1, y.a1);

    let r = RepeatedSkillCandidateDiscovery::discover(
        &[entry(&x, 2, 1000, 1000), entry(&y, 2, 1000, 1000)],
        default_policy(),
    );

    assert_eq!(r.candidate_count(), 2);
    assert_ne!(r.candidates()[0].trace(), r.candidates()[1].trace());
}

#[test]
fn repetition_support_ranks_before_higher_confidence() {
    let repeated = entry(&base(), 3, 800, 800);

    let mut other = base();
    other.initial = a(600);
    other.a1 = a(20);
    other.o1 = a(120);
    other.a2 = a(21);
    other.o2 = a(121);

    let confident = entry(&other, 2, 1000, 1000);

    let r = RepeatedSkillCandidateDiscovery::discover(
        &[confident, repeated],
        pol(PolicySpec {
            input: 32,
            eval: 32,
            steps: 16,
            candidates: 1,
            support: 2,
            success: 500,
            confidence: 500,
        }),
    );

    assert_eq!(r.candidates_before_frontier(), 2);
    assert_eq!(r.candidates()[0].support_count(), 3);
}

#[test]
fn hard_input_evaluation_and_candidate_frontiers_hold() {
    let first = entry(&base(), 4, 1000, 1000);

    let mut b = base();
    b.initial = a(600);
    b.a1 = a(20);
    b.o1 = a(120);
    b.a2 = a(21);
    b.o2 = a(121);
    let second = entry(&b, 3, 1000, 1000);

    let mut c = base();
    c.initial = a(700);
    c.a1 = a(30);
    c.o1 = a(130);
    c.a2 = a(31);
    c.o2 = a(131);
    let third = entry(&c, 2, 1000, 1000);

    let input = RepeatedSkillCandidateDiscovery::discover(
        &[third.clone(), second.clone(), first.clone()],
        pol(PolicySpec {
            input: 1,
            eval: 32,
            steps: 16,
            candidates: 32,
            support: 2,
            success: 500,
            confidence: 500,
        }),
    );
    assert_eq!(input.unique_entry_count(), 3);
    assert_eq!(input.considered_entry_count(), 1);
    assert!(input.entry_frontier_truncated());

    let eval = RepeatedSkillCandidateDiscovery::discover(
        &[first.clone(), second.clone(), third.clone()],
        pol(PolicySpec {
            input: 32,
            eval: 1,
            steps: 16,
            candidates: 32,
            support: 2,
            success: 500,
            confidence: 500,
        }),
    );
    assert_eq!(eval.entry_evaluation_count(), 1);
    assert!(eval.entry_evaluation_truncated());

    let final_gate = RepeatedSkillCandidateDiscovery::discover(
        &[first, second, third],
        pol(PolicySpec {
            input: 32,
            eval: 32,
            steps: 16,
            candidates: 1,
            support: 2,
            success: 500,
            confidence: 500,
        }),
    );
    assert_eq!(final_gate.candidates_before_frontier(), 3);
    assert!(final_gate.candidate_frontier_truncated());
    assert_eq!(final_gate.candidate_count(), 1);
}

#[test]
fn discovery_is_order_invariant_non_mutating_and_facade_equivalent() {
    let first = entry(&base(), 3, 900, 900);

    let mut other = base();
    other.initial = a(600);
    other.a1 = a(20);
    other.o1 = a(120);
    other.a2 = a(21);
    other.o2 = a(121);

    let second = entry(&other, 2, 1000, 1000);

    let entries = vec![second, first];
    let before = entries.clone();

    let mut reversed = entries.clone();
    reversed.reverse();

    let p = default_policy();

    let direct = RepeatedSkillCandidateDiscovery::discover(&entries, p);
    let rev = RepeatedSkillCandidateDiscovery::discover(&reversed, p);
    let facade = UniversalRepeatedSkillCandidateDiscovery::evaluate(&entries, p);
    let again = UniversalRepeatedSkillCandidateDiscovery::evaluate(&entries, p);

    assert_eq!(direct, rev);
    assert_eq!(direct, facade);
    assert_eq!(facade, again);
    assert_eq!(entries, before);
}
