use athlesia_meta_learning_skill_memory::{
    CompressedSkillRecord, CrossContextSkillGeneralization,
    CrossContextSkillGeneralizationEvidence, CrossContextSkillGeneralizationPolicy,
    GroundedSkillEpisode, GroundedSkillStep, LossControlledSkillCompression,
    RepeatedSkillCandidate, RepeatedSkillCandidateDiscovery, RepeatedSkillCandidatePolicy,
    SkillCompressionBounds, SkillCompressionPolicy, SkillCompressionThresholds,
    SkillMemoryFoundation, SkillMemoryPolicy, SkillRetrievalAndReuse, SkillReuseBounds,
    SkillReusePolicy, SkillReuseThresholds, StructuralSkillAbstractionEvidence,
    StructuralSkillAbstractionInduction, StructuralSkillAbstractionPolicy,
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

fn policy() -> SkillReusePolicy {
    SkillReusePolicy::new(
        SkillReuseBounds::new(32, 32, 16, 16, 32).unwrap(),
        SkillReuseThresholds::new(1, 1, s(500), s(500), s(500)).unwrap(),
    )
}

use athlesia_meta_learning_skill_memory::{
    GroundedSkillCorrespondenceInference, GroundedSkillCorrespondencePolicy,
    SkillExecutionObservation,
};

fn execution_observation(
    state: u64,
    action: u64,
    outcome: u64,
    confidence: u16,
) -> SkillExecutionObservation {
    SkillExecutionObservation::new(a(state), a(action), a(outcome), s(confidence))
        .expect("grounded execution evidence requires positive confidence")
}

fn correspondence_policy() -> GroundedSkillCorrespondencePolicy {
    GroundedSkillCorrespondencePolicy::new(32, 32, 32, s(500)).expect("valid correspondence policy")
}

use athlesia_meta_learning_skill_memory::{
    GroundedPartialSkillTransfer, GroundedSkillReuseRequest,
};

#[test]
fn autonomous_correspondence_enables_next_known_action_without_inventing_future_outcome() {
    let records = vec![record(7, 7, 2)];

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &[execution_observation(900, 910, 1010, 900)],
        correspondence_policy(),
    );

    assert_eq!(correspondence.request_count(), 1);

    let request = &correspondence.requests()[0];

    let full = SkillRetrievalAndReuse::retrieve(&records, request, policy());

    assert_eq!(
        full.plan_count(),
        0,
        "full-plan retrieval must remain conservative while a future outcome is unresolved",
    );

    let partial =
        GroundedPartialSkillTransfer::retrieve_next(&records, request, &a(1010), policy());

    assert_eq!(
        partial.candidate_count(),
        1,
        "the learned skill must still transfer its next fully grounded action",
    );

    let candidate = &partial.candidates()[0];

    assert_eq!(candidate.step_index(), 1);
    assert_eq!(candidate.required_state(), &a(1010));
    assert_eq!(candidate.action(), &a(7));

    assert_eq!(
        candidate.predicted_outcome(),
        None,
        "unseen context-dependent future outcome must remain explicitly unknown",
    );
}

#[test]
fn partial_transfer_generalizes_the_next_action_to_entirely_new_atom_identities() {
    let records = vec![record(7, 7, 2)];

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(1900),
        &a(7),
        &[execution_observation(1900, 1910, 2010, 900)],
        correspondence_policy(),
    );

    assert_eq!(correspondence.request_count(), 1);

    let partial = GroundedPartialSkillTransfer::retrieve_next(
        &records,
        &correspondence.requests()[0],
        &a(2010),
        policy(),
    );

    assert_eq!(partial.candidate_count(), 1);

    let candidate = &partial.candidates()[0];

    assert_eq!(candidate.required_state(), &a(2010));
    assert_eq!(candidate.action(), &a(7));
    assert_eq!(candidate.predicted_outcome(), None);
}

#[test]
fn already_grounded_step_preserves_known_outcome_instead_of_erasing_it() {
    let records = vec![record(7, 7, 2)];

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &[execution_observation(900, 910, 1010, 900)],
        correspondence_policy(),
    );

    let partial = GroundedPartialSkillTransfer::retrieve_next(
        &records,
        &correspondence.requests()[0],
        &a(900),
        policy(),
    );

    assert_eq!(partial.candidate_count(), 1);

    let candidate = &partial.candidates()[0];

    assert_eq!(candidate.step_index(), 0);
    assert_eq!(candidate.required_state(), &a(900));
    assert_eq!(candidate.action(), &a(910));
    assert_eq!(candidate.predicted_outcome(), Some(&a(1010)));
}

#[test]
fn unresolved_required_state_or_action_cannot_be_recovered_by_hidden_guessing() {
    let records = vec![record(7, 7, 2)];

    let request = GroundedSkillReuseRequest::new(a(900), a(7), Vec::new());

    let partial =
        GroundedPartialSkillTransfer::retrieve_next(&records, &request, &a(1010), policy());

    assert_eq!(partial.candidate_count(), 0);
    assert!(partial.abstained());
}

#[test]
fn unrelated_current_state_cannot_trigger_a_learned_action() {
    let records = vec![record(7, 7, 2)];

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &[execution_observation(900, 910, 1010, 900)],
        correspondence_policy(),
    );

    let partial = GroundedPartialSkillTransfer::retrieve_next(
        &records,
        &correspondence.requests()[0],
        &a(9999),
        policy(),
    );

    assert_eq!(partial.candidate_count(), 0);
    assert!(partial.abstained());
}

use athlesia_meta_learning_skill_memory::{GroundedSkillSlotBinding, SkillReuseSlotKind};

#[derive(Clone)]
struct T0bTransferTrace {
    base: u64,
    goal: u64,
    next_action: u64,
    terminal: u64,
}

fn t0b_transfer_candidate(trace: &T0bTransferTrace, support: usize) -> RepeatedSkillCandidate {
    let episodes: Vec<_> = (0..support)
        .map(|_| {
            GroundedSkillEpisode::new(
                a(trace.base),
                a(trace.goal),
                vec![
                    GroundedSkillStep::new(
                        a(trace.base),
                        a(trace.base + 10),
                        a(trace.base + 110),
                        s(1000),
                    )
                    .unwrap(),
                    GroundedSkillStep::new(
                        a(trace.base + 110),
                        a(trace.next_action),
                        a(trace.terminal),
                        s(1000),
                    )
                    .unwrap(),
                ],
                s(1000),
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

fn t0b_transfer_abstraction(
    left_base: u64,
    right_base: u64,
    goal: u64,
    next_action: u64,
    terminal: u64,
    support: usize,
) -> StructuralSkillAbstractionEvidence {
    StructuralSkillAbstractionInduction::induce(
        &[
            t0b_transfer_candidate(
                &T0bTransferTrace {
                    base: left_base,
                    goal,
                    next_action,
                    terminal,
                },
                support,
            ),
            t0b_transfer_candidate(
                &T0bTransferTrace {
                    base: right_base,
                    goal,
                    next_action,
                    terminal,
                },
                support,
            ),
        ],
        StructuralSkillAbstractionPolicy::new(16, 16, 16, 16, 2, s(1), s(1)).unwrap(),
    )
    .abstractions()[0]
        .clone()
}

fn t0b_transfer_record(goal: u64, next_action: u64, support: usize) -> CompressedSkillRecord {
    let generalized = CrossContextSkillGeneralization::generalize(
        &[
            t0b_transfer_abstraction(100, 200, goal, next_action, 70, support),
            t0b_transfer_abstraction(300, 400, goal, next_action, 80, support),
        ],
        CrossContextSkillGeneralizationPolicy::new(16, 16, 16, 16, 1, s(1), s(1)).unwrap(),
    )
    .generalizations()[0]
        .clone();

    LossControlledSkillCompression::compress_all(
        std::slice::from_ref(&generalized),
        SkillCompressionPolicy::new(
            SkillCompressionBounds::new(16, 16, 16, 16).unwrap(),
            SkillCompressionThresholds::new(1, s(1), s(1), 0).unwrap(),
        ),
    )
    .records()[0]
        .clone()
}

fn t0b_request_from_observed_prefix(record: &CompressedSkillRecord) -> GroundedSkillReuseRequest {
    let result = GroundedSkillCorrespondenceInference::infer(
        std::slice::from_ref(record),
        &a(900),
        &a(7),
        &[execution_observation(900, 910, 1010, 900)],
        correspondence_policy(),
    );

    assert_eq!(result.request_count(), 1);

    result.requests()[0].clone()
}

fn t0b_binding(slot_id: usize, value: u64, confidence: u16) -> GroundedSkillSlotBinding {
    GroundedSkillSlotBinding::new(
        SkillReuseSlotKind::Structural,
        slot_id,
        a(value),
        s(confidence),
    )
    .unwrap()
}

#[test]
fn competing_transferred_actions_remain_explicit_and_order_invariant() {
    let records = vec![t0b_transfer_record(7, 70, 4), t0b_transfer_record(7, 80, 3)];

    let request = t0b_request_from_observed_prefix(&records[0]);

    let forward =
        GroundedPartialSkillTransfer::retrieve_next(&records, &request, &a(1010), policy());

    let mut reversed_records = records.clone();
    reversed_records.reverse();

    let reversed = GroundedPartialSkillTransfer::retrieve_next(
        &reversed_records,
        &request,
        &a(1010),
        policy(),
    );

    assert_eq!(
        forward, reversed,
        "competing transferred actions must not depend on skill-memory presentation order",
    );

    assert_eq!(
        forward.candidate_count(),
        2,
        "two evidence-backed incompatible actions must remain two explicit candidates",
    );

    let mut actions: Vec<_> = forward
        .candidates()
        .iter()
        .map(|candidate| candidate.action().clone())
        .collect();

    actions.sort();

    assert_eq!(actions, vec![a(70), a(80)]);

    assert!(
        forward
            .candidates()
            .iter()
            .all(|candidate| candidate.predicted_outcome().is_none()),
        "unknown future outcomes must stay unknown for every competing action",
    );
}

#[test]
fn semantic_duplicate_skill_records_do_not_multiply_partial_candidates() {
    let records = vec![t0b_transfer_record(7, 70, 2), t0b_transfer_record(7, 70, 4)];

    let request = t0b_request_from_observed_prefix(&records[0]);

    let result =
        GroundedPartialSkillTransfer::retrieve_next(&records, &request, &a(1010), policy());

    assert_eq!(result.input_record_count(), 2);
    assert_eq!(result.unique_record_count(), 1);
    assert_eq!(result.candidate_count(), 1);
    assert_eq!(result.candidates()[0].action(), &a(70));
}

#[test]
fn weak_binding_blocks_partial_transfer_before_record_evaluation() {
    let records = vec![t0b_transfer_record(7, 70, 2)];

    let request = GroundedSkillReuseRequest::new(
        a(900),
        a(7),
        vec![t0b_binding(1, 910, 900), t0b_binding(2, 1010, 400)],
    );

    let result =
        GroundedPartialSkillTransfer::retrieve_next(&records, &request, &a(1010), policy());

    assert!(result.binding_threshold_failed());
    assert_eq!(result.evaluation_count(), 0);
    assert!(result.abstained());
}

#[test]
fn conflicting_binding_blocks_partial_transfer_instead_of_overwriting_identity() {
    let records = vec![t0b_transfer_record(7, 70, 2)];

    let request = GroundedSkillReuseRequest::new(
        a(900),
        a(7),
        vec![t0b_binding(2, 1010, 900), t0b_binding(2, 2020, 900)],
    );

    let result =
        GroundedPartialSkillTransfer::retrieve_next(&records, &request, &a(1010), policy());

    assert!(result.binding_conflict());
    assert_eq!(result.evaluation_count(), 0);
    assert!(result.abstained());
}

#[test]
fn record_frontier_is_canonical_and_explicit_when_competing_skills_exceed_capacity() {
    let records = vec![t0b_transfer_record(7, 70, 4), t0b_transfer_record(7, 80, 3)];

    let request = t0b_request_from_observed_prefix(&records[0]);

    let bounded = SkillReusePolicy::new(
        SkillReuseBounds::new(1, 32, 16, 16, 32).unwrap(),
        policy().thresholds(),
    );

    let forward =
        GroundedPartialSkillTransfer::retrieve_next(&records, &request, &a(1010), bounded);

    let mut reversed_records = records.clone();
    reversed_records.reverse();

    let reversed =
        GroundedPartialSkillTransfer::retrieve_next(&reversed_records, &request, &a(1010), bounded);

    assert_eq!(
        forward, reversed,
        "bounded record admission must remain canonical",
    );

    assert_eq!(forward.unique_record_count(), 2);
    assert_eq!(forward.considered_record_count(), 1);
    assert!(forward.record_frontier_truncated());

    assert_eq!(
        forward.candidate_count(),
        1,
        "a bounded candidate may be retained only with explicit incomplete-frontier evidence",
    );
}

#[test]
fn evaluation_frontier_is_explicit_and_order_invariant() {
    let records = vec![t0b_transfer_record(7, 70, 4), t0b_transfer_record(7, 80, 3)];

    let request = t0b_request_from_observed_prefix(&records[0]);

    let bounded = SkillReusePolicy::new(
        SkillReuseBounds::new(32, 1, 16, 16, 32).unwrap(),
        policy().thresholds(),
    );

    let forward =
        GroundedPartialSkillTransfer::retrieve_next(&records, &request, &a(1010), bounded);

    let mut reversed_records = records.clone();
    reversed_records.reverse();

    let reversed =
        GroundedPartialSkillTransfer::retrieve_next(&reversed_records, &request, &a(1010), bounded);

    assert_eq!(forward, reversed);
    assert_eq!(forward.evaluation_count(), 1);
    assert!(forward.evaluation_frontier_truncated());
}

#[test]
fn candidate_frontier_truncation_is_never_hidden_as_complete_transfer_knowledge() {
    let records = vec![t0b_transfer_record(7, 70, 4), t0b_transfer_record(7, 80, 3)];

    let request = t0b_request_from_observed_prefix(&records[0]);

    let bounded = SkillReusePolicy::new(
        SkillReuseBounds::new(32, 32, 16, 16, 1).unwrap(),
        policy().thresholds(),
    );

    let result = GroundedPartialSkillTransfer::retrieve_next(&records, &request, &a(1010), bounded);

    assert_eq!(result.candidates_before_frontier(), 2);
    assert_eq!(result.candidate_count(), 1);

    assert!(
        result.candidate_frontier_truncated(),
        "a singleton retained from a larger action frontier must explicitly report incompleteness",
    );
}
