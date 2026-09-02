use athlesia_meta_learning_skill_memory::{
    CrossContextSkillGeneralization, CrossContextSkillGeneralizationPolicy, GroundedSkillEpisode,
    GroundedSkillStep, RepeatedSkillCandidate, RepeatedSkillCandidateDiscovery,
    RepeatedSkillCandidatePolicy, SkillMemoryFoundation, SkillMemoryPolicy,
    StructuralSkillAbstractionEvidence, StructuralSkillAbstractionInduction,
    StructuralSkillAbstractionPolicy, UniversalCrossContextSkillGeneralization,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone)]
struct Trace {
    initial: CognitiveStructure,
    goal: CognitiveStructure,
    a1: CognitiveStructure,
    o1: CognitiveStructure,
    a2: CognitiveStructure,
    o2: CognitiveStructure,
}

fn s(v: u16) -> CognitiveSignal {
    CognitiveSignal::new(v).unwrap()
}

fn a(v: u64) -> CognitiveStructure {
    CognitiveStructure::atom(v)
}

fn trace(base: u64, goal: u64, tail: u64) -> Trace {
    Trace {
        initial: a(base),
        goal: a(goal),
        a1: a(base + 10),
        o1: a(base + 110),
        a2: a(tail),
        o2: a(tail + 100),
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

fn candidate(t: &Trace, support: usize, success: u16, confidence: u16) -> RepeatedSkillCandidate {
    let eps: Vec<_> = (0..support)
        .map(|_| {
            GroundedSkillEpisode::new(
                t.initial.clone(),
                t.goal.clone(),
                vec![
                    step(t.initial.clone(), t.a1.clone(), t.o1.clone(), confidence),
                    step(t.o1.clone(), t.a2.clone(), t.o2.clone(), confidence),
                ],
                s(success),
            )
            .unwrap()
        })
        .collect();

    let memory = SkillMemoryFoundation::build(
        &eps,
        SkillMemoryPolicy::new(64, 16, 64, 64, s(1), s(1)).unwrap(),
    );

    RepeatedSkillCandidateDiscovery::discover(
        memory.entries(),
        RepeatedSkillCandidatePolicy::new(64, 64, 16, 64, 2, s(1), s(1)).unwrap(),
    )
    .candidates()[0]
        .clone()
}

fn abstraction(
    left: &Trace,
    right: &Trace,
    support: usize,
    success: u16,
    confidence: u16,
) -> StructuralSkillAbstractionEvidence {
    StructuralSkillAbstractionInduction::induce(
        &[
            candidate(left, support, success, confidence),
            candidate(right, support, success, confidence),
        ],
        StructuralSkillAbstractionPolicy::new(16, 16, 16, 16, 2, s(1), s(1)).unwrap(),
    )
    .abstractions()[0]
        .clone()
}

fn compatible_pair() -> (
    StructuralSkillAbstractionEvidence,
    StructuralSkillAbstractionEvidence,
) {
    let x = abstraction(&trace(100, 1, 11), &trace(200, 1, 11), 2, 1000, 1000);

    let y = abstraction(&trace(300, 2, 12), &trace(400, 2, 12), 2, 1000, 1000);

    (x, y)
}

fn policy() -> CrossContextSkillGeneralizationPolicy {
    CrossContextSkillGeneralizationPolicy::new(32, 64, 16, 32, 1, s(500), s(500)).unwrap()
}

#[test]
fn policy_requires_multiple_inputs_and_positive_bounds() {
    assert_eq!(
        CrossContextSkillGeneralizationPolicy::new(1, 1, 1, 1, 1, s(1), s(1)),
        None
    );

    assert!(CrossContextSkillGeneralizationPolicy::new(2, 1, 1, 1, 1, s(1), s(1)).is_some());
}

#[test]
fn duplicate_exact_abstraction_cannot_fake_cross_context_evidence() {
    let (x, _) = compatible_pair();

    let r = CrossContextSkillGeneralization::generalize(&[x.clone(), x], policy());

    assert_eq!(r.unique_abstraction_count(), 1);

    assert!(r.abstained());
}

#[test]
fn compatible_structural_abstractions_form_cross_context_schema() {
    let (x, y) = compatible_pair();

    let r = CrossContextSkillGeneralization::generalize(&[x, y], policy());

    assert_eq!(r.generalization_count(), 1);

    assert_eq!(r.generalizations()[0].schema().step_count(), 2);
}

#[test]
fn invariants_shared_across_contexts_remain_invariants() {
    let x = abstraction(&trace(100, 1, 99), &trace(200, 1, 99), 2, 1000, 1000);

    let mut y_left = trace(300, 1, 99);

    let mut y_right = trace(400, 1, 99);

    y_left.o2 = a(777);
    y_right.o2 = a(777);

    let y = abstraction(&y_left, &y_right, 2, 1000, 1000);

    assert_ne!(x.abstraction(), y.abstraction());

    let r = CrossContextSkillGeneralization::generalize(&[x, y], policy());

    assert_eq!(r.generalization_count(), 1);

    let schema = r.generalizations()[0].schema();

    assert_eq!(schema.goal_identity().invariant(), Some(&a(1)));

    assert_eq!(schema.steps()[1].action().invariant(), Some(&a(99)));
}

#[test]
fn differing_context_invariants_become_context_variables() {
    let x = abstraction(&trace(100, 1, 1), &trace(200, 1, 1), 2, 1000, 1000);

    let y = abstraction(&trace(300, 2, 2), &trace(400, 2, 2), 2, 1000, 1000);

    let r = CrossContextSkillGeneralization::generalize(&[x, y], policy());

    let schema = r.generalizations()[0].schema();

    let goal = schema.goal_identity().context_variable_id().unwrap();

    let action = schema.steps()[1].action().context_variable_id().unwrap();

    assert_eq!(goal, action);

    assert!(schema.context_variable_count() >= 1);
}

#[test]
fn inherited_structural_variable_equality_is_preserved() {
    let (x, y) = compatible_pair();

    let r = CrossContextSkillGeneralization::generalize(&[x, y], policy());

    let schema = r.generalizations()[0].schema();

    let initial = schema.initial_state().structural_variable_id().unwrap();

    let required = schema.steps()[0]
        .required_state()
        .structural_variable_id()
        .unwrap();

    assert_eq!(initial, required);

    assert!(schema.structural_variable_count() >= 1);
}

#[test]
fn variable_invariant_conflict_is_rejected_not_forced() {
    let invariant_goal = abstraction(&trace(100, 1, 11), &trace(200, 1, 11), 2, 1000, 1000);

    let variable_goal = abstraction(&trace(300, 2, 12), &trace(400, 3, 12), 2, 1000, 1000);

    let r = CrossContextSkillGeneralization::generalize(&[invariant_goal, variable_goal], policy());

    assert_eq!(r.incompatible_structure_count(), 1);

    assert!(r.abstained());
}

#[test]
fn incompatible_step_counts_are_rejected() {
    let two = abstraction(&trace(100, 1, 11), &trace(200, 1, 11), 2, 1000, 1000);

    let make = |base: u64| {
        GroundedSkillEpisode::new(
            a(base),
            a(1),
            vec![
                step(a(base), a(base + 10), a(base + 110), 1000),
                step(a(base + 110), a(base + 11), a(base + 111), 1000),
                step(a(base + 111), a(base + 12), a(base + 112), 1000),
            ],
            s(1000),
        )
        .unwrap()
    };

    let make_candidate = |base: u64| {
        let memory = SkillMemoryFoundation::build(
            &[make(base), make(base)],
            SkillMemoryPolicy::new(16, 16, 16, 16, s(1), s(1)).unwrap(),
        );

        RepeatedSkillCandidateDiscovery::discover(
            memory.entries(),
            RepeatedSkillCandidatePolicy::new(16, 16, 16, 16, 2, s(1), s(1)).unwrap(),
        )
        .candidates()[0]
            .clone()
    };

    let three = StructuralSkillAbstractionInduction::induce(
        &[make_candidate(500), make_candidate(600)],
        StructuralSkillAbstractionPolicy::new(16, 16, 16, 16, 2, s(1), s(1)).unwrap(),
    )
    .abstractions()[0]
        .clone();

    let r = CrossContextSkillGeneralization::generalize(&[two, three], policy());

    assert_eq!(r.incompatible_structure_count(), 1);

    assert!(r.abstained());
}

#[test]
fn weak_abstraction_evidence_is_rejected_before_pairing() {
    let weak = abstraction(&trace(100, 1, 11), &trace(200, 1, 11), 2, 400, 1000);

    let strong = abstraction(&trace(300, 2, 12), &trace(400, 2, 12), 2, 1000, 1000);

    let r = CrossContextSkillGeneralization::generalize(&[weak, strong], policy());

    assert_eq!(r.rejected_threshold_count(), 1);

    assert_eq!(r.pair_evaluation_count(), 0);
}

#[test]
fn hard_input_frontier_prefers_stronger_abstraction_evidence() {
    let strong = abstraction(&trace(100, 1, 11), &trace(200, 1, 11), 4, 1000, 1000);

    let medium = abstraction(&trace(300, 2, 12), &trace(400, 2, 12), 3, 1000, 1000);

    let weak = abstraction(&trace(500, 3, 13), &trace(600, 3, 13), 2, 1000, 1000);

    let r = CrossContextSkillGeneralization::generalize(
        &[weak, medium, strong],
        CrossContextSkillGeneralizationPolicy::new(2, 64, 16, 32, 1, s(500), s(500)).unwrap(),
    );

    assert_eq!(r.unique_abstraction_count(), 3);

    assert_eq!(r.considered_abstraction_count(), 2);

    assert!(r.abstraction_frontier_truncated());

    assert_eq!(r.pair_evaluation_count(), 1);
}

#[test]
fn hard_pair_and_output_frontiers_are_enforced() {
    let a1 = abstraction(&trace(100, 1, 11), &trace(200, 1, 11), 4, 1000, 1000);

    let a2 = abstraction(&trace(300, 2, 12), &trace(400, 2, 12), 3, 1000, 1000);

    let a3 = abstraction(&trace(500, 1, 13), &trace(600, 1, 13), 2, 1000, 1000);

    let pair = CrossContextSkillGeneralization::generalize(
        &[a1.clone(), a2.clone(), a3.clone()],
        CrossContextSkillGeneralizationPolicy::new(32, 1, 16, 32, 1, s(500), s(500)).unwrap(),
    );

    assert_eq!(pair.pair_evaluation_count(), 1);

    assert!(pair.pair_evaluation_truncated());

    let output = CrossContextSkillGeneralization::generalize(
        &[a1, a2, a3],
        CrossContextSkillGeneralizationPolicy::new(32, 64, 16, 1, 1, s(500), s(500)).unwrap(),
    );

    assert!(output.generalizations_before_frontier() >= 2);

    assert!(output.generalization_frontier_truncated());

    assert_eq!(output.generalization_count(), 1);
}

#[test]
fn generalization_is_order_invariant_and_facade_equivalent() {
    let (x, y) = compatible_pair();

    let items = vec![y, x];

    let before = items.clone();

    let mut reversed = items.clone();

    reversed.reverse();

    let p = policy();

    let direct = CrossContextSkillGeneralization::generalize(&items, p);

    let reorder = CrossContextSkillGeneralization::generalize(&reversed, p);

    let facade = UniversalCrossContextSkillGeneralization::evaluate(&items, p);

    let again = UniversalCrossContextSkillGeneralization::evaluate(&items, p);

    assert_eq!(direct, reorder);

    assert_eq!(direct, facade);

    assert_eq!(facade, again);

    assert_eq!(items, before);
}
