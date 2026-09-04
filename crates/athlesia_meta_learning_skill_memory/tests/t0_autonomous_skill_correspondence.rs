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

#[test]
fn observed_novel_world_prefix_autonomously_infers_correspondence_without_manual_slot_ids() {
    let records = vec![record(7, 7, 2)];

    let evidence = vec![execution_observation(900, 910, 1010, 900)];

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    assert_eq!(
        correspondence.request_count(),
        1,
        "one structurally consistent observed prefix should infer one grounded correspondence",
    );

    let request = &correspondence.requests()[0];

    assert_eq!(request.current_state(), &a(900));
    assert_eq!(request.goal_identity(), &a(7));

    assert!(
        request.bindings().len() >= 3,
        "the observed prefix must autonomously ground its structural roles",
    );

    let reuse = SkillRetrievalAndReuse::retrieve(&records, request, policy());

    assert_eq!(
        reuse.plan_count(),
        0,
        "an unseen future context-dependent outcome must remain unresolved instead of being fabricated",
    );

    assert_eq!(reuse.rejected_unresolved_count(), 1);
    assert!(reuse.abstained());
}

#[test]
fn correspondence_generalizes_to_entirely_new_atom_identities_without_inventing_future_outcomes() {
    let records = vec![record(7, 7, 2)];

    let evidence = vec![execution_observation(1900, 1910, 2010, 900)];

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(1900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    assert_eq!(correspondence.request_count(), 1);

    let request = &correspondence.requests()[0];

    assert_eq!(request.current_state(), &a(1900));
    assert_eq!(request.goal_identity(), &a(7));

    assert!(
        request.bindings().len() >= 3,
        "novel atom identities must be grounded through structural role evidence",
    );

    let reuse = SkillRetrievalAndReuse::retrieve(&records, request, policy());

    assert_eq!(reuse.plan_count(), 0);
    assert_eq!(reuse.rejected_unresolved_count(), 1);
    assert!(reuse.abstained());
}

#[test]
fn absent_new_world_evidence_cannot_fabricate_correspondence() {
    let records = vec![record(7, 7, 2)];

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &[],
        correspondence_policy(),
    );

    assert_eq!(correspondence.request_count(), 0);
    assert!(correspondence.abstained());
}

#[test]
fn incompatible_grounded_prefixes_are_reported_instead_of_arbitrarily_choosing_one() {
    let records = vec![record(7, 7, 2)];

    let evidence = vec![
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 920, 1020, 900),
    ];

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    assert_eq!(
        correspondence.request_count(),
        0,
        "contradictory correspondence evidence must not silently choose a mapping",
    );

    assert!(correspondence.conflicting_evidence());
    assert!(correspondence.abstained());
}

#[test]
fn weak_observation_cannot_ground_a_structural_correspondence() {
    let records = vec![record(7, 7, 2)];

    let evidence = vec![execution_observation(900, 910, 1010, 400)];

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    assert_eq!(correspondence.request_count(), 0);
    assert_eq!(correspondence.rejected_low_confidence_count(), 1);
    assert!(correspondence.abstained());
}

#[test]
fn repeated_consistent_evidence_strengthens_correspondence_without_duplication() {
    let records = vec![record(7, 7, 2)];

    let evidence = vec![
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 850),
        execution_observation(900, 910, 1010, 950),
    ];

    let result = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    assert_eq!(
        result.request_count(),
        1,
        "repeated support for the same structural mapping must not create duplicate correspondences",
    );

    assert!(!result.conflicting_evidence());
    assert_eq!(result.considered_observation_count(), 3);
}

#[test]
fn unrelated_world_evidence_does_not_corrupt_current_correspondence() {
    let records = vec![record(7, 7, 2)];

    let evidence = vec![
        execution_observation(900, 910, 1010, 900),
        execution_observation(5000, 5010, 5110, 1000),
        execution_observation(6000, 6010, 6110, 1000),
    ];

    let result = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    assert_eq!(result.request_count(), 1);
    assert!(!result.conflicting_evidence());
}

#[test]
fn repeated_grounded_consensus_survives_one_contradictory_outlier_and_is_order_invariant() {
    let records = vec![record(7, 7, 2)];

    let mut evidence = vec![
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 920, 1020, 900),
    ];

    let forward = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    evidence.reverse();

    let reversed = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    assert_eq!(
        forward, reversed,
        "noise handling must depend on evidence structure, not presentation order",
    );

    assert_eq!(
        forward.request_count(),
        1,
        "one isolated contradiction must not erase a repeatedly reproduced structural correspondence",
    );

    assert!(
        !forward.conflicting_evidence(),
        "resolved residual noise must not be reported as unresolved mapping conflict",
    );
}

#[test]
fn equally_supported_incompatible_correspondences_remain_unresolved() {
    let records = vec![record(7, 7, 2)];

    let evidence = vec![
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 920, 1020, 900),
        execution_observation(900, 920, 1020, 900),
    ];

    let result = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    assert_eq!(
        result.request_count(),
        0,
        "balanced incompatible evidence does not justify choosing either correspondence",
    );

    assert!(result.conflicting_evidence());
    assert!(result.abstained());
}

#[test]
fn bounded_observation_frontier_preserves_balanced_competing_evidence_independent_of_input_order() {
    let records = vec![record(7, 7, 2)];

    let bounded_policy = GroundedSkillCorrespondencePolicy::new(32, 4, 32, s(500))
        .expect("valid bounded correspondence policy");

    let evidence = vec![
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 920, 1020, 900),
        execution_observation(900, 920, 1020, 900),
        execution_observation(900, 920, 1020, 900),
    ];

    let forward = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        bounded_policy,
    );

    let mut reversed_evidence = evidence.clone();
    reversed_evidence.reverse();

    let reversed = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &reversed_evidence,
        bounded_policy,
    );

    assert_eq!(forward.input_observation_count(), 6);
    assert_eq!(forward.considered_observation_count(), 4);
    assert_eq!(reversed.input_observation_count(), 6);
    assert_eq!(reversed.considered_observation_count(), 4);

    assert_eq!(
        forward, reversed,
        "bounded evidence selection must depend on grounded evidence structure, not presentation order",
    );

    assert_eq!(
        forward.request_count(),
        0,
        "a bounded frontier must not manufacture a correspondence winner from globally balanced competing evidence",
    );

    assert!(
        forward.conflicting_evidence(),
        "balanced reproducible correspondence alternatives must remain explicitly unresolved",
    );

    assert!(forward.abstained());
}

#[test]
fn multiple_skill_records_are_correspondence_order_invariant_when_budget_is_sufficient() {
    let records = vec![record(7, 7, 2), record(7, 8, 2)];

    let evidence = vec![execution_observation(900, 910, 1010, 900)];

    let forward = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    let mut reversed_records = records.clone();
    reversed_records.reverse();

    let reversed = GroundedSkillCorrespondenceInference::infer(
        &reversed_records,
        &a(900),
        &a(7),
        &evidence,
        correspondence_policy(),
    );

    assert_eq!(
        forward, reversed,
        "skill-memory presentation order must not change autonomous correspondence semantics",
    );
}

#[test]
fn record_frontier_cannot_turn_input_order_into_semantic_authority() {
    let records = vec![record(7, 7, 2), record(7, 8, 2)];

    let evidence = vec![execution_observation(900, 910, 1010, 900)];

    let bounded_policy = GroundedSkillCorrespondencePolicy::new(1, 32, 32, s(500))
        .expect("valid record-bounded correspondence policy");

    let forward = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        bounded_policy,
    );

    let mut reversed_records = records.clone();
    reversed_records.reverse();

    let reversed = GroundedSkillCorrespondenceInference::infer(
        &reversed_records,
        &a(900),
        &a(7),
        &evidence,
        bounded_policy,
    );

    assert_eq!(forward.input_record_count(), 2);
    assert_eq!(forward.considered_record_count(), 1);
    assert_eq!(reversed.input_record_count(), 2);
    assert_eq!(reversed.considered_record_count(), 1);

    assert_eq!(
        forward, reversed,
        "bounded record admission must not depend on which skill happened to arrive first",
    );

    assert_eq!(
        forward.request_count(),
        0,
        "when relevant skill alternatives exceed the record frontier, correspondence must abstain rather than crown the first record",
    );

    assert!(forward.abstained());
}

#[test]
fn overflowing_unique_correspondence_frontier_abstains_independent_of_input_order() {
    let records = vec![record(7, 7, 2)];

    let bounded_policy = GroundedSkillCorrespondencePolicy::new(32, 2, 32, s(500))
        .expect("valid evidence-bounded correspondence policy");

    let evidence = vec![
        execution_observation(900, 910, 1010, 900),
        execution_observation(900, 920, 1020, 900),
        execution_observation(900, 930, 1030, 900),
    ];

    let forward = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &evidence,
        bounded_policy,
    );

    let mut reversed_evidence = evidence.clone();
    reversed_evidence.reverse();

    let reversed = GroundedSkillCorrespondenceInference::infer(
        &records,
        &a(900),
        &a(7),
        &reversed_evidence,
        bounded_policy,
    );

    assert_eq!(
        forward, reversed,
        "overflowing correspondence alternatives must not inherit input-order authority",
    );

    assert_eq!(forward.request_count(), 0);
    assert!(forward.conflicting_evidence());
    assert!(forward.abstained());
}
