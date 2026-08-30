use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_control::{
    RecursiveControlExperimentLoop, RecursiveControlObjective, RecursiveControlUncertaintyPolicy,
};

use athlesia_recursive_planning::{
    RecursivePlanningGoal, RecursivePlanningMemory, RecursivePlanningState,
    RecursivePlanningTransition,
};

use athlesia_recursive_revision::{
    RecursiveCompetingModels, RecursiveExperimentObservation, RecursiveRevisionMemory,
};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
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

fn concept(base_span: usize, cross_span: usize) -> RecursiveConcept {
    RecursiveConcept::new(vec![
        base(base_span),
        cross(cross_span, &[cross_span + 1, cross_span + 2]),
    ])
    .unwrap()
}

fn state(units: Vec<RecursiveUnit>) -> RecursivePlanningState {
    RecursivePlanningState::new(units)
}

fn goal(units: Vec<RecursiveUnit>) -> RecursivePlanningGoal {
    RecursivePlanningGoal::new(units).unwrap()
}

fn objective(model: RecursiveConcept, target_span: usize) -> RecursiveControlObjective {
    RecursiveControlObjective::new(model, goal(vec![base(target_span)]))
}

fn insert(
    memory: &mut RecursivePlanningMemory,
    from: RecursivePlanningState,
    to: RecursivePlanningState,
) {
    memory.insert(RecursivePlanningTransition::new(from, to).unwrap());
}

fn contested_revision() -> (RecursiveRevisionMemory, RecursiveConcept, RecursiveConcept) {
    let first = concept(1, 2);

    let second = concept(1, 5);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first.clone());

    memory.violate(first.clone());

    memory.confirm(second.clone());

    memory.violate(second.clone());

    (memory, first, second)
}

#[test]
fn experiment_loop_requires_experiment_decision() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(model.clone());

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let result = RecursiveControlExperimentLoop::new().observe(
        &planning,
        &mut revision,
        &start,
        &[objective(model, 20)],
        RecursiveExperimentObservation::Present,
    );

    assert!(result.is_none());
}

#[test]
fn present_observation_is_recorded() {
    let (mut revision, _, _) = contested_revision();

    let result = RecursiveControlExperimentLoop::new()
        .observe(
            &RecursivePlanningMemory::new(),
            &mut revision,
            &state(vec![base(10)]),
            &[],
            RecursiveExperimentObservation::Present,
        )
        .unwrap();

    assert_eq!(
        result.revision().observation(),
        RecursiveExperimentObservation::Present
    );
}

#[test]
fn absent_observation_is_recorded() {
    let (mut revision, _, _) = contested_revision();

    let result = RecursiveControlExperimentLoop::new()
        .observe(
            &RecursivePlanningMemory::new(),
            &mut revision,
            &state(vec![base(10)]),
            &[],
            RecursiveExperimentObservation::Absent,
        )
        .unwrap();

    assert_eq!(
        result.revision().observation(),
        RecursiveExperimentObservation::Absent
    );
}

#[test]
fn control_selected_experiment_matches_revision_cycle() {
    let (mut revision, _, _) = contested_revision();

    let models = RecursiveCompetingModels::from_memory(&revision);

    let start = state(vec![base(10)]);

    let before = RecursiveControlUncertaintyPolicy::new().decide(
        &RecursivePlanningMemory::new(),
        &models,
        &start,
        &[],
    );

    let expected = before.experiment().unwrap().clone();

    let result = RecursiveControlExperimentLoop::new()
        .observe(
            &RecursivePlanningMemory::new(),
            &mut revision,
            &start,
            &[],
            RecursiveExperimentObservation::Present,
        )
        .unwrap();

    assert_eq!(result.revision().experiment(), &expected);
}

#[test]
fn before_model_snapshot_matches_revision_cycle() {
    let (mut revision, _, _) = contested_revision();

    let expected = RecursiveCompetingModels::from_memory(&revision);

    let result = RecursiveControlExperimentLoop::new()
        .observe(
            &RecursivePlanningMemory::new(),
            &mut revision,
            &state(vec![base(10)]),
            &[],
            RecursiveExperimentObservation::Present,
        )
        .unwrap();

    assert_eq!(result.before_models(), &expected);

    assert_eq!(result.revision().before(), &expected);
}

#[test]
fn after_model_snapshot_matches_revision_cycle() {
    let (mut revision, _, _) = contested_revision();

    let result = RecursiveControlExperimentLoop::new()
        .observe(
            &RecursivePlanningMemory::new(),
            &mut revision,
            &state(vec![base(10)]),
            &[],
            RecursiveExperimentObservation::Present,
        )
        .unwrap();

    assert_eq!(result.after_models(), result.revision().after());

    assert_eq!(
        result.after_models(),
        &RecursiveCompetingModels::from_memory(&revision,)
    );
}

#[test]
fn observation_mutates_revision_memory() {
    let (mut revision, _, _) = contested_revision();

    let before = revision.clone();

    let _ = RecursiveControlExperimentLoop::new()
        .observe(
            &RecursivePlanningMemory::new(),
            &mut revision,
            &state(vec![base(10)]),
            &[],
            RecursiveExperimentObservation::Present,
        )
        .unwrap();

    assert_ne!(revision, before);
}

#[test]
fn next_policy_is_recomputed_from_after_models() {
    let (mut revision, first, second) = contested_revision();

    let start = state(vec![base(10)]);

    let objectives = vec![objective(first, 20), objective(second, 30)];

    let planning = RecursivePlanningMemory::new();

    let result = RecursiveControlExperimentLoop::new()
        .observe(
            &planning,
            &mut revision,
            &start,
            &objectives,
            RecursiveExperimentObservation::Present,
        )
        .unwrap();

    let expected = RecursiveControlUncertaintyPolicy::new().decide(
        &planning,
        result.after_models(),
        &start,
        &objectives,
    );

    assert_eq!(result.next_decision(), &expected);
}

#[test]
fn experiment_changes_competing_model_snapshot() {
    let (mut revision, _, _) = contested_revision();

    let before = RecursiveCompetingModels::from_memory(&revision);

    let result = RecursiveControlExperimentLoop::new()
        .observe(
            &RecursivePlanningMemory::new(),
            &mut revision,
            &state(vec![base(10)]),
            &[],
            RecursiveExperimentObservation::Present,
        )
        .unwrap();

    assert_ne!(result.after_models(), &before);
}

#[test]
fn recursive_depth_identity_survives_experiment_loop() {
    let child = concept(1, 2);

    let deep =
        RecursiveConcept::new(vec![base(8), RecursiveUnit::Recursive(Box::new(child))]).unwrap();

    let other = concept(8, 20);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(deep.clone());

    revision.confirm(deep.clone());

    revision.violate(deep.clone());

    revision.confirm(other.clone());

    revision.violate(other);

    let before = RecursiveCompetingModels::from_memory(&revision);

    assert_eq!(before.best().unwrap().concept(), &deep);

    let result = RecursiveControlExperimentLoop::new()
        .observe(
            &RecursivePlanningMemory::new(),
            &mut revision,
            &state(vec![base(10)]),
            &[],
            RecursiveExperimentObservation::Present,
        )
        .unwrap();

    assert_eq!(result.before_models().best().unwrap().concept(), &deep);
}

#[test]
fn planning_memory_and_start_are_not_mutated() {
    let (mut revision, _, _) = contested_revision();

    let planning = RecursivePlanningMemory::new();

    let start = state(vec![base(10)]);

    let planning_before = planning.clone();

    let start_before = start.clone();

    let _ = RecursiveControlExperimentLoop::new()
        .observe(
            &planning,
            &mut revision,
            &start,
            &[],
            RecursiveExperimentObservation::Present,
        )
        .unwrap();

    assert_eq!(planning, planning_before);

    assert_eq!(start, start_before);
}

#[test]
fn experiment_loop_is_deterministic() {
    let (revision, _, _) = contested_revision();

    let mut left_memory = revision.clone();

    let mut right_memory = revision;

    let planning = RecursivePlanningMemory::new();

    let start = state(vec![base(10)]);

    let left = RecursiveControlExperimentLoop::new().observe(
        &planning,
        &mut left_memory,
        &start,
        &[],
        RecursiveExperimentObservation::Present,
    );

    let right = RecursiveControlExperimentLoop::new().observe(
        &planning,
        &mut right_memory,
        &start,
        &[],
        RecursiveExperimentObservation::Present,
    );

    assert_eq!(left, right);

    assert_eq!(left_memory, right_memory);
}
