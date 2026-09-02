use athlesia_meta_learning_skill_memory::{
    CompressedSkillTerm, CrossContextSkillGeneralization, CrossContextSkillGeneralizationEvidence,
    CrossContextSkillGeneralizationPolicy, GroundedSkillEpisode, GroundedSkillStep,
    LossControlledSkillCompression, RepeatedSkillCandidate, RepeatedSkillCandidateDiscovery,
    RepeatedSkillCandidatePolicy, SkillCompressionBounds, SkillCompressionPolicy,
    SkillCompressionThresholds, SkillMemoryFoundation, SkillMemoryPolicy,
    StructuralSkillAbstractionEvidence, StructuralSkillAbstractionInduction,
    StructuralSkillAbstractionPolicy, UniversalLossControlledSkillCompression,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone)]
struct Trace {
    base: u64,
    fixed: u64,
    terminal: u64,
}

#[derive(Clone)]
struct AbstractionSpec {
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

fn candidate(
    spec: &Trace,
    support: usize,
    success: u16,
    confidence: u16,
) -> RepeatedSkillCandidate {
    let episodes: Vec<_> = (0..support)
        .map(|_| {
            GroundedSkillEpisode::new(
                a(spec.base),
                a(spec.fixed),
                vec![
                    step(
                        a(spec.base),
                        a(spec.base + 10),
                        a(spec.base + 110),
                        confidence,
                    ),
                    step(
                        a(spec.base + 110),
                        a(spec.fixed),
                        a(spec.terminal),
                        confidence,
                    ),
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

fn abstraction(spec: &AbstractionSpec) -> StructuralSkillAbstractionEvidence {
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

fn generalization(
    left: AbstractionSpec,
    right: AbstractionSpec,
) -> CrossContextSkillGeneralizationEvidence {
    CrossContextSkillGeneralization::generalize(
        &[abstraction(&left), abstraction(&right)],
        CrossContextSkillGeneralizationPolicy::new(16, 16, 16, 16, 1, s(1), s(1)).unwrap(),
    )
    .generalizations()[0]
        .clone()
}

fn gain_generalization(
    fixed: u64,
    terminal_a: u64,
    terminal_b: u64,
) -> CrossContextSkillGeneralizationEvidence {
    generalization(
        AbstractionSpec {
            left: 100,
            right: 200,
            fixed,
            terminal: terminal_a,
            support: 2,
            success: 1000,
            confidence: 1000,
        },
        AbstractionSpec {
            left: 300,
            right: 400,
            fixed,
            terminal: terminal_b,
            support: 2,
            success: 1000,
            confidence: 1000,
        },
    )
}

fn no_gain_generalization() -> CrossContextSkillGeneralizationEvidence {
    generalization(
        AbstractionSpec {
            left: 100,
            right: 200,
            fixed: 7,
            terminal: 70,
            support: 2,
            success: 1000,
            confidence: 1000,
        },
        AbstractionSpec {
            left: 300,
            right: 400,
            fixed: 8,
            terminal: 80,
            support: 2,
            success: 1000,
            confidence: 1000,
        },
    )
}

fn policy(gain: usize) -> SkillCompressionPolicy {
    SkillCompressionPolicy::new(
        SkillCompressionBounds::new(32, 32, 16, 32).unwrap(),
        SkillCompressionThresholds::new(1, s(500), s(500), gain).unwrap(),
    )
}

#[test]
fn compression_policy_requires_positive_bounds_and_evidence_thresholds() {
    assert_eq!(SkillCompressionBounds::new(0, 1, 1, 1), None);

    assert_eq!(SkillCompressionThresholds::new(0, s(1), s(1), 0), None);

    assert!(SkillCompressionBounds::new(1, 1, 1, 1).is_some());

    assert!(SkillCompressionThresholds::new(1, s(1), s(1), 0).is_some());
}

#[test]
fn repeated_exact_invariant_is_dictionary_compressed_losslessly() {
    let g = gain_generalization(7, 70, 80);

    let r = LossControlledSkillCompression::compress_all(std::slice::from_ref(&g), policy(1));

    assert_eq!(r.record_count(), 1);

    let record = &r.records()[0];

    assert!(record.compression_gain() >= 1);

    assert!(record.semantically_matches(g.schema()));
}

#[test]
fn zero_gain_schema_is_rejected_when_gain_is_required() {
    let g = no_gain_generalization();

    let r = LossControlledSkillCompression::compress_all(std::slice::from_ref(&g), policy(1));

    assert_eq!(r.rejected_gain_count(), 1);

    assert!(r.abstained());
}

#[test]
fn exact_invariant_dictionary_preserves_opaque_structure_identity() {
    let g = gain_generalization(7, 70, 80);

    let r = LossControlledSkillCompression::compress_all(std::slice::from_ref(&g), policy(1));

    let record = &r.records()[0];

    assert_eq!(record.invariant_dictionary(), &[a(7)]);

    assert!(matches!(
        record.goal_identity(),
        CompressedSkillTerm::InvariantRef(0)
    ));
}

#[test]
fn structural_slot_equality_topology_survives_compression() {
    let g = gain_generalization(7, 70, 80);

    let r = LossControlledSkillCompression::compress_all(std::slice::from_ref(&g), policy(1));

    let record = &r.records()[0];

    assert_eq!(record.initial_state(), record.steps()[0].required_state());

    assert!(matches!(
        record.initial_state(),
        CompressedSkillTerm::StructuralSlot(_)
    ));
}

#[test]
fn context_slot_equality_topology_survives_compression() {
    let g = no_gain_generalization();

    let r = LossControlledSkillCompression::compress_all(std::slice::from_ref(&g), policy(0));

    let record = &r.records()[0];

    assert_eq!(record.goal_identity(), record.steps()[1].action());

    assert!(matches!(
        record.goal_identity(),
        CompressedSkillTerm::ContextSlot(_)
    ));
}

#[test]
fn ordered_step_semantics_round_trip_exactly() {
    let g = gain_generalization(7, 70, 80);

    let r = LossControlledSkillCompression::compress_all(std::slice::from_ref(&g), policy(1));

    let record = &r.records()[0];

    assert_eq!(record.step_count(), g.schema().step_count());

    assert!(record.semantically_matches(g.schema()));
}

#[test]
fn weak_generalization_evidence_is_rejected_before_compression() {
    let weak = generalization(
        AbstractionSpec {
            left: 100,
            right: 200,
            fixed: 7,
            terminal: 70,
            support: 2,
            success: 400,
            confidence: 1000,
        },
        AbstractionSpec {
            left: 300,
            right: 400,
            fixed: 7,
            terminal: 80,
            support: 2,
            success: 400,
            confidence: 1000,
        },
    );

    let r = LossControlledSkillCompression::compress_all(std::slice::from_ref(&weak), policy(0));

    assert_eq!(r.rejected_threshold_count(), 1);

    assert!(r.abstained());
}

#[test]
fn source_pair_support_threshold_is_enforced() {
    let g = gain_generalization(7, 70, 80);

    let p = SkillCompressionPolicy::new(
        SkillCompressionBounds::new(32, 32, 16, 32).unwrap(),
        SkillCompressionThresholds::new(2, s(1), s(1), 0).unwrap(),
    );

    let r = LossControlledSkillCompression::compress_all(std::slice::from_ref(&g), p);

    assert_eq!(r.rejected_support_count(), 1);

    assert!(r.abstained());
}

#[test]
fn distinct_semantic_schemas_remain_distinct_compressed_records() {
    let first = gain_generalization(7, 70, 80);

    let second = gain_generalization(9, 90, 100);

    let r = LossControlledSkillCompression::compress_all(&[first, second], policy(1));

    assert_eq!(r.record_count(), 2);

    assert_ne!(
        r.records()[0].invariant_dictionary(),
        r.records()[1].invariant_dictionary()
    );
}

#[test]
fn hard_input_evaluation_and_record_frontiers_are_enforced() {
    let first = gain_generalization(7, 70, 80);

    let second = gain_generalization(9, 90, 100);

    let third = gain_generalization(11, 110, 120);

    let input = LossControlledSkillCompression::compress_all(
        &[first.clone(), second.clone(), third.clone()],
        SkillCompressionPolicy::new(
            SkillCompressionBounds::new(1, 32, 16, 32).unwrap(),
            SkillCompressionThresholds::new(1, s(1), s(1), 1).unwrap(),
        ),
    );

    assert_eq!(input.unique_generalization_count(), 3);

    assert_eq!(input.considered_generalization_count(), 1);

    assert!(input.input_frontier_truncated());

    let eval = LossControlledSkillCompression::compress_all(
        &[first.clone(), second.clone(), third.clone()],
        SkillCompressionPolicy::new(
            SkillCompressionBounds::new(32, 1, 16, 32).unwrap(),
            SkillCompressionThresholds::new(1, s(1), s(1), 1).unwrap(),
        ),
    );

    assert_eq!(eval.evaluation_count(), 1);

    assert!(eval.evaluation_frontier_truncated());

    let output = LossControlledSkillCompression::compress_all(
        &[first, second, third],
        SkillCompressionPolicy::new(
            SkillCompressionBounds::new(32, 32, 16, 1).unwrap(),
            SkillCompressionThresholds::new(1, s(1), s(1), 1).unwrap(),
        ),
    );

    assert_eq!(output.records_before_frontier(), 3);

    assert!(output.record_frontier_truncated());

    assert_eq!(output.record_count(), 1);
}

#[test]
fn compression_is_order_invariant_non_mutating_and_facade_equivalent() {
    let first = gain_generalization(7, 70, 80);

    let second = gain_generalization(9, 90, 100);

    let items = vec![second, first];

    let before = items.clone();

    let mut reversed = items.clone();

    reversed.reverse();

    let p = policy(1);

    let direct = LossControlledSkillCompression::compress_all(&items, p);

    let reordered = LossControlledSkillCompression::compress_all(&reversed, p);

    let facade = UniversalLossControlledSkillCompression::evaluate(&items, p);

    let repeated = UniversalLossControlledSkillCompression::evaluate(&items, p);

    assert_eq!(direct, reordered);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(items, before);
}
