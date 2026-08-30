use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{
    AbstractionUnit, CrossLevelActiveCycle, CrossLevelCompletionOutcome, CrossLevelConcept,
    CrossLevelMemory, CrossLevelObservation,
};

use athlesia_hierarchy::HierarchicalConcept;

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        6,
    )
}

fn hierarchy(spans: &[usize]) -> HierarchicalConcept {
    HierarchicalConcept::new(spans.iter().copied().map(structural).collect()).unwrap()
}

fn structural_unit(span: usize) -> AbstractionUnit {
    AbstractionUnit::Structural(structural(span))
}

fn hierarchical_unit(spans: &[usize]) -> AbstractionUnit {
    AbstractionUnit::Hierarchical(hierarchy(spans))
}

fn cross_level(structural_span: usize, hierarchy_spans: &[usize]) -> CrossLevelConcept {
    CrossLevelConcept::new(vec![
        structural_unit(structural_span),
        hierarchical_unit(hierarchy_spans),
    ])
    .unwrap()
}

fn memory() -> CrossLevelMemory {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[3, 4]));

    memory.insert(cross_level(2, &[3, 4]));

    memory
}

#[test]
fn cycle_selects_cross_level_target() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.candidate().unit(), &hierarchical_unit(&[3, 4],));

    assert_eq!(transition.candidate().supporting_concepts(), 2);
}

#[test]
fn structural_anchor_can_complete_hierarchical_target() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let prior = CrossLevelObservation::new(vec![structural_unit(1)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3])]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_confirmed());
}

#[test]
fn hierarchical_anchor_can_complete_structural_target() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let prior = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3])]);

    let next = CrossLevelObservation::new(vec![structural_unit(1)]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_confirmed());

    assert_eq!(transition.candidate().unit(), &structural_unit(1));
}

#[test]
fn cycle_confirms_appearing_target() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.outcome(), CrossLevelCompletionOutcome::Confirmed);

    assert!(transition.is_confirmed());

    assert!(!transition.is_violated());
}

#[test]
fn cycle_violates_missing_target() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[5, 6])]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.outcome(), CrossLevelCompletionOutcome::Violated);

    assert!(transition.is_violated());
}

#[test]
fn transition_records_prior_observation() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.prior_observation(), &prior);
}

#[test]
fn transition_records_next_observation() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4]), structural_unit(5)]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(transition.next_observation(), &next);
}

#[test]
fn transition_target_matches_evaluation_target() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert_eq!(
        transition.candidate().unit(),
        transition.evaluation().target()
    );
}

#[test]
fn complete_prior_state_stops_cycle() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let prior = CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]);

    let next = CrossLevelObservation::new(vec![structural_unit(4)]);

    assert!(CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn zero_overlap_stops_cycle() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let prior = CrossLevelObservation::new(vec![structural_unit(4), hierarchical_unit(&[5, 6])]);

    let next = CrossLevelObservation::new(vec![structural_unit(1)]);

    assert!(CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn empty_memory_stops_cycle() {
    let memory = CrossLevelMemory::new();

    let prior = CrossLevelObservation::new(vec![structural_unit(1)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3])]);

    assert!(CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next,)
        .is_none());
}

#[test]
fn extra_next_context_preserves_confirmation() {
    let memory = memory();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![
        hierarchical_unit(&[3, 4]),
        structural_unit(5),
        hierarchical_unit(&[6, 7]),
    ]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_confirmed());
}

#[test]
fn structural_extent_mismatch_produces_violation() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        6,
    );

    let anchor = hierarchical_unit(&[2, 3]);

    let mut memory = CrossLevelMemory::new();

    memory.insert(
        CrossLevelConcept::new(vec![AbstractionUnit::Structural(short), anchor.clone()]).unwrap(),
    );

    let prior = CrossLevelObservation::new(vec![anchor]);

    let next = CrossLevelObservation::new(vec![AbstractionUnit::Structural(long)]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_violated());
}

#[test]
fn hierarchy_identity_mismatch_produces_violation() {
    let mut memory = CrossLevelMemory::new();

    memory.insert(cross_level(1, &[2, 3]));

    let prior = CrossLevelObservation::new(vec![structural_unit(1)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 4])]);

    let transition = CrossLevelActiveCycle::new()
        .step(&memory, &prior, &next)
        .unwrap();

    assert!(transition.is_violated());
}

#[test]
fn active_cycle_is_deterministic_and_non_mutating() {
    let memory = memory();
    let before = memory.clone();

    let prior = CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]);

    let next = CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4])]);

    let cycle = CrossLevelActiveCycle::new();

    let first = cycle.step(&memory, &prior, &next);

    let second = cycle.step(&memory, &prior, &next);

    assert_eq!(first, second);

    assert_eq!(memory, before);
}
