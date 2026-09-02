use athlesia_meta_learning_skill_memory::{
    CompressedSkillRecord, CrossContextSkillGeneralization,
    CrossContextSkillGeneralizationEvidence, CrossContextSkillGeneralizationPolicy,
    GroundedSkillEpisode, GroundedSkillReuseRequest, GroundedSkillSlotBinding, GroundedSkillStep,
    LossControlledSkillCompression, RepeatedSkillCandidate, RepeatedSkillCandidateDiscovery,
    RepeatedSkillCandidatePolicy, SkillCompressionBounds, SkillCompressionPolicy,
    SkillCompressionThresholds, SkillMemoryFoundation, SkillMemoryPolicy, SkillRetrievalAndReuse,
    SkillReuseBounds, SkillReusePolicy, SkillReuseSlotKind, SkillReuseThresholds,
    StructuralSkillAbstractionEvidence, StructuralSkillAbstractionInduction,
    StructuralSkillAbstractionPolicy, UniversalSkillRetrievalAndReuse,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone)]
struct Trace {
    base: u64,
    fixed: u64,
    terminal: u64,
}

#[derive(Clone)]
struct Abs {
    left: u64,
    right: u64,
    fixed: u64,
    terminal: u64,
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

fn step(
    state: CognitiveStructure,
    action: CognitiveStructure,
    outcome: CognitiveStructure,
    confidence: u16,
) -> GroundedSkillStep {
    GroundedSkillStep::new(state, action, outcome, s(confidence)).unwrap()
}

fn candidate(t: &Trace, support: usize, success: u16, confidence: u16) -> RepeatedSkillCandidate {
    let episodes: Vec<_> = (0..support)
        .map(|_| {
            GroundedSkillEpisode::new(
                a(t.base),
                a(t.fixed),
                vec![
                    step(a(t.base), a(t.base + 10), a(t.base + 110), confidence),
                    step(a(t.base + 110), a(t.fixed), a(t.terminal), confidence),
                ],
                s(success),
            )
            .unwrap()
        })
        .collect();

    let memory = SkillMemoryFoundation::build(
        &episodes,
        SkillMemoryPolicy::new(64, 16, 64, 64, s(1), s(1)).unwrap(),
    );

    RepeatedSkillCandidateDiscovery::discover(
        memory.entries(),
        RepeatedSkillCandidatePolicy::new(64, 64, 16, 64, 2, s(1), s(1)).unwrap(),
    )
    .candidates()[0]
        .clone()
}

fn abstraction(spec: &Abs) -> StructuralSkillAbstractionEvidence {
    StructuralSkillAbstractionInduction::induce(
        &[
            candidate(
                &Trace {
                    base: spec.left,
                    fixed: spec.fixed,
                    terminal: spec.terminal,
                },
                spec.support,
                spec.success,
                spec.confidence,
            ),
            candidate(
                &Trace {
                    base: spec.right,
                    fixed: spec.fixed,
                    terminal: spec.terminal,
                },
                spec.support,
                spec.success,
                spec.confidence,
            ),
        ],
        StructuralSkillAbstractionPolicy::new(16, 16, 16, 16, 2, s(1), s(1)).unwrap(),
    )
    .abstractions()[0]
        .clone()
}

fn generalization(left: Abs, right: Abs) -> CrossContextSkillGeneralizationEvidence {
    CrossContextSkillGeneralization::generalize(
        &[abstraction(&left), abstraction(&right)],
        CrossContextSkillGeneralizationPolicy::new(16, 16, 16, 16, 1, s(1), s(1)).unwrap(),
    )
    .generalizations()[0]
        .clone()
}

fn record(fixed_left: u64, fixed_right: u64, support: usize) -> CompressedSkillRecord {
    let g = generalization(
        Abs {
            left: 100,
            right: 200,
            fixed: fixed_left,
            terminal: 70,
            support,
            success: 1000,
            confidence: 1000,
        },
        Abs {
            left: 300,
            right: 400,
            fixed: fixed_right,
            terminal: 80,
            support,
            success: 1000,
            confidence: 1000,
        },
    );

    LossControlledSkillCompression::compress_all(
        std::slice::from_ref(&g),
        SkillCompressionPolicy::new(
            SkillCompressionBounds::new(16, 16, 16, 16).unwrap(),
            SkillCompressionThresholds::new(1, s(1), s(1), 0).unwrap(),
        ),
    )
    .records()[0]
        .clone()
}

fn binding(
    kind: SkillReuseSlotKind,
    id: usize,
    value: u64,
    confidence: u16,
) -> GroundedSkillSlotBinding {
    GroundedSkillSlotBinding::new(kind, id, a(value), s(confidence)).unwrap()
}

fn request(goal: u64) -> GroundedSkillReuseRequest {
    GroundedSkillReuseRequest::new(
        a(900),
        a(goal),
        vec![
            binding(SkillReuseSlotKind::Structural, 1, 910, 1000),
            binding(SkillReuseSlotKind::Structural, 2, 1010, 1000),
            binding(SkillReuseSlotKind::Context, 0, goal, 1000),
            binding(SkillReuseSlotKind::Context, 1, 999, 1000),
        ],
    )
}

fn policy() -> SkillReusePolicy {
    SkillReusePolicy::new(
        SkillReuseBounds::new(32, 32, 16, 16, 32).unwrap(),
        SkillReuseThresholds::new(1, 1, s(500), s(500), s(500)).unwrap(),
    )
}

#[test]
fn reuse_policy_and_binding_require_positive_bounds_and_confidence() {
    assert_eq!(SkillReuseBounds::new(0, 1, 1, 1, 1), None);

    assert_eq!(SkillReuseThresholds::new(1, 1, s(1), s(1), s(0)), None);

    assert_eq!(
        GroundedSkillSlotBinding::new(SkillReuseSlotKind::Structural, 0, a(1), s(0)),
        None
    );

    assert!(SkillReuseBounds::new(1, 1, 1, 1, 1).is_some());
}

#[test]
fn exact_invariant_goal_record_is_retrieved_and_grounded() {
    let r = SkillRetrievalAndReuse::retrieve(&[record(7, 7, 2)], &request(7), policy());

    assert_eq!(r.plan_count(), 1);

    let plan = &r.plans()[0];

    assert_eq!(plan.initial_state(), &a(900));
    assert_eq!(plan.goal_identity(), &a(7));
}

#[test]
fn initial_structural_slot_is_inferred_from_current_state_anchor() {
    let r = SkillRetrievalAndReuse::retrieve(&[record(7, 7, 2)], &request(7), policy());

    let plan = &r.plans()[0];

    assert_eq!(plan.first_step().unwrap().required_state(), &a(900));
}

#[test]
fn context_goal_slot_is_inferred_from_exact_goal_anchor() {
    let r = SkillRetrievalAndReuse::retrieve(&[record(7, 8, 2)], &request(42), policy());

    assert_eq!(r.plan_count(), 1);

    assert_eq!(r.plans()[0].steps()[1].action(), &a(42));
}

#[test]
fn conflicting_grounded_binding_cannot_override_anchor_identity() {
    let req = GroundedSkillReuseRequest::new(
        a(900),
        a(7),
        vec![
            binding(SkillReuseSlotKind::Structural, 0, 901, 1000),
            binding(SkillReuseSlotKind::Structural, 1, 910, 1000),
            binding(SkillReuseSlotKind::Structural, 2, 1010, 1000),
            binding(SkillReuseSlotKind::Context, 0, 7, 1000),
        ],
    );

    let r = SkillRetrievalAndReuse::retrieve(&[record(7, 7, 2)], &req, policy());

    assert_eq!(r.rejected_anchor_mismatch_count(), 1);

    assert!(r.abstained());
}

#[test]
fn weak_binding_evidence_blocks_reuse_before_record_evaluation() {
    let req = GroundedSkillReuseRequest::new(
        a(900),
        a(7),
        vec![binding(SkillReuseSlotKind::Structural, 1, 910, 400)],
    );

    let r = SkillRetrievalAndReuse::retrieve(&[record(7, 7, 2)], &req, policy());

    assert!(r.binding_threshold_failed());
    assert_eq!(r.evaluation_count(), 0);
    assert!(r.abstained());
}

#[test]
fn unresolved_slots_cause_abstention_instead_of_hidden_guessing() {
    let req = GroundedSkillReuseRequest::new(a(900), a(7), Vec::new());

    let r = SkillRetrievalAndReuse::retrieve(&[record(7, 7, 2)], &req, policy());

    assert_eq!(r.rejected_unresolved_count(), 1);
    assert!(r.abstained());
}

#[test]
fn exact_invariant_goal_mismatch_rejects_skill_record() {
    let r = SkillRetrievalAndReuse::retrieve(&[record(7, 7, 2)], &request(8), policy());

    assert_eq!(r.rejected_anchor_mismatch_count(), 1);

    assert!(r.abstained());
}

#[test]
fn grounded_reuse_preserves_exact_step_order_and_state_continuity() {
    let r = SkillRetrievalAndReuse::retrieve(&[record(7, 7, 2)], &request(7), policy());

    let steps = r.plans()[0].steps();

    assert_eq!(steps.len(), 2);

    assert_eq!(steps[0].required_state(), &a(900));
    assert_eq!(steps[0].action(), &a(910));
    assert_eq!(steps[0].predicted_outcome(), &a(1010));

    assert_eq!(steps[1].required_state(), &a(1010));
    assert_eq!(steps[1].action(), &a(7));
    assert_eq!(steps[1].predicted_outcome(), &a(7));
}

#[test]
fn semantically_duplicate_records_keep_stronger_provenance() {
    let weak = record(7, 7, 2);
    let strong = record(7, 7, 4);

    let r = SkillRetrievalAndReuse::retrieve(&[weak, strong], &request(7), policy());

    assert_eq!(r.input_record_count(), 2);
    assert_eq!(r.unique_record_count(), 1);
    assert_eq!(r.plan_count(), 1);

    assert!(r.plans()[0].source_record().source_support_sum() >= 16);
}

#[test]
fn hard_record_evaluation_binding_and_plan_frontiers_are_enforced() {
    let invariant = record(7, 7, 4);
    let contextual = record(7, 8, 3);
    let irrelevant = record(9, 9, 2);

    let input_policy = SkillReusePolicy::new(
        SkillReuseBounds::new(1, 32, 16, 16, 32).unwrap(),
        policy().thresholds(),
    );

    let input = SkillRetrievalAndReuse::retrieve(
        &[irrelevant.clone(), contextual.clone(), invariant.clone()],
        &request(7),
        input_policy,
    );

    assert_eq!(input.unique_record_count(), 3);
    assert_eq!(input.considered_record_count(), 1);
    assert!(input.record_frontier_truncated());

    let eval_policy = SkillReusePolicy::new(
        SkillReuseBounds::new(32, 1, 16, 16, 32).unwrap(),
        policy().thresholds(),
    );

    let eval = SkillRetrievalAndReuse::retrieve(
        &[invariant.clone(), contextual.clone(), irrelevant],
        &request(7),
        eval_policy,
    );

    assert_eq!(eval.evaluation_count(), 1);
    assert!(eval.evaluation_frontier_truncated());

    let plan_policy = SkillReusePolicy::new(
        SkillReuseBounds::new(32, 32, 16, 16, 1).unwrap(),
        policy().thresholds(),
    );

    let selected =
        SkillRetrievalAndReuse::retrieve(&[invariant, contextual], &request(7), plan_policy);

    assert_eq!(selected.plans_before_frontier(), 2);
    assert!(selected.plan_frontier_truncated());
    assert_eq!(selected.plan_count(), 1);

    let binding_policy = SkillReusePolicy::new(
        SkillReuseBounds::new(32, 32, 1, 16, 32).unwrap(),
        policy().thresholds(),
    );

    let bound = SkillRetrievalAndReuse::retrieve(&[record(7, 7, 2)], &request(7), binding_policy);

    assert!(bound.binding_frontier_exceeded());
    assert!(bound.abstained());
}

#[test]
fn retrieval_is_order_invariant_non_mutating_and_facade_equivalent() {
    let records = vec![record(7, 8, 3), record(7, 7, 4)];

    let before = records.clone();

    let mut reversed = records.clone();
    reversed.reverse();

    let req = request(7);
    let p = policy();

    let direct = SkillRetrievalAndReuse::retrieve(&records, &req, p);

    let reordered = SkillRetrievalAndReuse::retrieve(&reversed, &req, p);

    let facade = UniversalSkillRetrievalAndReuse::evaluate(&records, &req, p);

    let repeated = UniversalSkillRetrievalAndReuse::evaluate(&records, &req, p);

    assert_eq!(direct, reordered);
    assert_eq!(direct, facade);
    assert_eq!(facade, repeated);
    assert_eq!(records, before);
}
