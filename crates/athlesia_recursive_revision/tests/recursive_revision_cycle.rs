use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_revision::{
    RecursiveExperimentObservation, RecursiveRevisionCycle, RecursiveRevisionMemory,
    RecursiveRevisionStatus,
};

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

fn base(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(structural_unit(span))
}

fn cross(structural_span: usize, hierarchy_spans: &[usize]) -> RecursiveUnit {
    RecursiveUnit::CrossLevel(cross_level(structural_span, hierarchy_spans))
}

fn concept(first: RecursiveUnit, second: RecursiveUnit) -> RecursiveConcept {
    RecursiveConcept::new(vec![first, second]).unwrap()
}

fn competing_pair() -> (RecursiveRevisionMemory, RecursiveConcept, RecursiveConcept) {
    let shared = cross(3, &[4, 5]);

    let first = concept(base(1), shared.clone());

    let second = concept(base(2), shared);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first.clone());

    memory.confirm(second.clone());

    (memory, first, second)
}

#[test]
fn empty_memory_has_no_revision_cycle() {
    let mut memory = RecursiveRevisionMemory::new();

    assert!(RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Present,)
        .is_none());
}

#[test]
fn single_model_has_no_discriminative_cycle() {
    let target = concept(base(1), cross(2, &[3, 4]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(target);

    assert!(RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Present,)
        .is_none());
}

#[test]
fn cycle_records_competing_models_before_update() {
    let (mut memory, _, _) = competing_pair();

    let transition = RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Present)
        .unwrap();

    assert_eq!(transition.before().len(), 2);

    assert_eq!(transition.before().supported_count(), 2);
}

#[test]
fn cycle_selects_discriminative_experiment() {
    let (mut memory, _, _) = competing_pair();

    let transition = RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Present)
        .unwrap();

    assert_eq!(transition.experiment().supporters(), 1);

    assert_eq!(transition.experiment().opponents(), 1);

    assert_eq!(transition.experiment().discrimination_score(), 1);
}

#[test]
fn present_observation_updates_supporter() {
    let (mut memory, first, second) = competing_pair();

    let transition = RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Present)
        .unwrap();

    let target = transition.experiment().target();

    let supporter = if first.contains(target) {
        first
    } else {
        second
    };

    assert_eq!(memory.evidence(&supporter,).unwrap().confirmations(), 2);

    assert_eq!(transition.evidence_updates().len(), 1);
}

#[test]
fn absent_observation_violates_supporter() {
    let (mut memory, first, second) = competing_pair();

    let transition = RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Absent)
        .unwrap();

    let target = transition.experiment().target();

    let supporter = if first.contains(target) {
        first
    } else {
        second
    };

    assert_eq!(memory.evidence(&supporter,).unwrap().violations(), 1);
}

#[test]
fn present_observation_strengthens_supporter_ranking() {
    let (mut memory, first, second) = competing_pair();

    let transition = RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Present)
        .unwrap();

    let target = transition.experiment().target();

    let supporter = if first.contains(target) {
        first
    } else {
        second
    };

    assert_eq!(transition.best_after().unwrap().concept(), &supporter);

    assert_eq!(transition.best_after().unwrap().confirmations(), 2);
}

#[test]
fn absent_observation_can_change_best_model() {
    let (mut memory, first, second) = competing_pair();

    let transition = RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Absent)
        .unwrap();

    let target = transition.experiment().target();

    let opponent = if first.contains(target) {
        second
    } else {
        first
    };

    assert_eq!(transition.best_after().unwrap().concept(), &opponent);

    assert_eq!(
        transition.best_after().unwrap().status(),
        RecursiveRevisionStatus::Supported
    );
}

#[test]
fn cycle_records_observation_value() {
    let (mut memory, _, _) = competing_pair();

    let transition = RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Absent)
        .unwrap();

    assert_eq!(
        transition.observation(),
        RecursiveExperimentObservation::Absent
    );
}

#[test]
fn recursive_target_revision_cycle_preserves_depth() {
    let level_one = concept(base(1), cross(2, &[3, 4]));

    let level_two = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(level_one.clone())),
    ])
    .unwrap();

    let shallow = RecursiveUnit::Recursive(Box::new(level_one));

    let deep = RecursiveUnit::Recursive(Box::new(level_two));

    let shallow_model = concept(base(6), shallow.clone());

    let deep_model = concept(base(6), deep.clone());

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(shallow_model.clone());

    memory.confirm(deep_model.clone());

    let transition = RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Present)
        .unwrap();

    let target = transition.experiment().target();

    assert!(target == &shallow || target == &deep);

    let supporter = if shallow_model.contains(target) {
        shallow_model
    } else {
        deep_model
    };

    assert_eq!(memory.evidence(&supporter,).unwrap().confirmations(), 2);
}

#[test]
fn before_snapshot_is_not_rewritten_after_update() {
    let (mut memory, _, _) = competing_pair();

    let transition = RecursiveRevisionCycle::new()
        .step(&mut memory, RecursiveExperimentObservation::Present)
        .unwrap();

    assert!(transition
        .before()
        .models()
        .iter()
        .all(|model| { model.confirmations() == 1 }));

    assert!(transition
        .after()
        .models()
        .iter()
        .any(|model| { model.confirmations() == 2 }));
}

#[test]
fn revision_cycle_is_deterministic() {
    let (memory, _, _) = competing_pair();

    let mut left = memory.clone();

    let mut right = memory;

    let left_transition =
        RecursiveRevisionCycle::new().step(&mut left, RecursiveExperimentObservation::Absent);

    let right_transition =
        RecursiveRevisionCycle::new().step(&mut right, RecursiveExperimentObservation::Absent);

    assert_eq!(left_transition, right_transition);

    assert_eq!(left, right);
}
