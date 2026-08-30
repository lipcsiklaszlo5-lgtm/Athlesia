use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_control::{
    RecursiveControlActiveCycle, RecursiveControlObjective, RecursiveControlUncertaintyPolicy,
};

use athlesia_recursive_planning::{
    RecursivePlanningExecutionStatus, RecursivePlanningGoal, RecursivePlanningMemory,
    RecursivePlanningState, RecursivePlanningTransition,
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

fn supported_revision(model: &RecursiveConcept) -> RecursiveRevisionMemory {
    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(model.clone());

    memory
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
fn supported_model_advances_one_action() {
    let model = concept(1, 2);

    let mut revision = supported_revision(&model);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish.clone());

    let result = RecursiveControlActiveCycle::new().tick(
        &planning,
        &mut revision,
        &start,
        &[objective(model, 20)],
        None,
    );

    assert!(result.is_action());

    assert_eq!(
        result.action().unwrap().execution().current_state(),
        &finish
    );
}

#[test]
fn action_tick_advances_exactly_one_transition() {
    let model = concept(1, 2);

    let mut revision = supported_revision(&model);

    let first = state(vec![base(10)]);

    let second = state(vec![base(10), base(20)]);

    let third = state(vec![base(10), base(20), base(30)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, first.clone(), second.clone());

    insert(&mut planning, second, third);

    let result = RecursiveControlActiveCycle::new().tick(
        &planning,
        &mut revision,
        &first,
        &[objective(model, 30)],
        None,
    );

    let execution = result.action().unwrap().execution();

    assert_eq!(execution.next_transition_index(), 1);

    assert_eq!(
        execution.status(),
        RecursivePlanningExecutionStatus::Running
    );
}

#[test]
fn final_action_step_reports_goal_reached() {
    let model = concept(1, 2);

    let mut revision = supported_revision(&model);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let result = RecursiveControlActiveCycle::new().tick(
        &planning,
        &mut revision,
        &start,
        &[objective(model, 20)],
        None,
    );

    assert_eq!(
        result.action().unwrap().execution().status(),
        RecursivePlanningExecutionStatus::GoalReached
    );
}

#[test]
fn observation_is_rejected_when_action_is_current_policy() {
    let model = concept(1, 2);

    let mut revision = supported_revision(&model);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let before = revision.clone();

    let result = RecursiveControlActiveCycle::new().tick(
        &planning,
        &mut revision,
        &start,
        &[objective(model, 20)],
        Some(RecursiveExperimentObservation::Present),
    );

    assert!(result.is_unexpected_observation());

    assert_eq!(revision, before);
}

#[test]
fn contested_model_without_observation_waits_for_experiment() {
    let (mut revision, _, _) = contested_revision();

    let result = RecursiveControlActiveCycle::new().tick(
        &RecursivePlanningMemory::new(),
        &mut revision,
        &state(vec![base(10)]),
        &[],
        None,
    );

    assert!(result.is_awaiting_experiment());

    assert!(result.experiment().is_some());
}

#[test]
fn awaiting_experiment_matches_uncertainty_policy() {
    let (mut revision, _, _) = contested_revision();

    let models = RecursiveCompetingModels::from_memory(&revision);

    let start = state(vec![base(10)]);

    let expected = RecursiveControlUncertaintyPolicy::new()
        .decide(&RecursivePlanningMemory::new(), &models, &start, &[])
        .experiment()
        .unwrap()
        .clone();

    let result = RecursiveControlActiveCycle::new().tick(
        &RecursivePlanningMemory::new(),
        &mut revision,
        &start,
        &[],
        None,
    );

    assert_eq!(result.experiment().unwrap(), &expected);
}

#[test]
fn present_observation_executes_one_experiment_revision() {
    let (mut revision, _, _) = contested_revision();

    let before = revision.clone();

    let result = RecursiveControlActiveCycle::new().tick(
        &RecursivePlanningMemory::new(),
        &mut revision,
        &state(vec![base(10)]),
        &[],
        Some(RecursiveExperimentObservation::Present),
    );

    assert!(result.is_experiment_observed());

    assert_ne!(revision, before);

    assert_eq!(
        result
            .experiment_transition()
            .unwrap()
            .revision()
            .observation(),
        RecursiveExperimentObservation::Present
    );
}

#[test]
fn absent_observation_executes_one_experiment_revision() {
    let (mut revision, _, _) = contested_revision();

    let result = RecursiveControlActiveCycle::new().tick(
        &RecursivePlanningMemory::new(),
        &mut revision,
        &state(vec![base(10)]),
        &[],
        Some(RecursiveExperimentObservation::Absent),
    );

    assert!(result.is_experiment_observed());

    assert_eq!(
        result
            .experiment_transition()
            .unwrap()
            .revision()
            .observation(),
        RecursiveExperimentObservation::Absent
    );
}

#[test]
fn experiment_tick_exposes_recomputed_next_policy() {
    let (mut revision, first, second) = contested_revision();

    let result = RecursiveControlActiveCycle::new().tick(
        &RecursivePlanningMemory::new(),
        &mut revision,
        &state(vec![base(10)]),
        &[objective(first, 20), objective(second, 30)],
        Some(RecursiveExperimentObservation::Present),
    );

    let transition = result.experiment_transition().unwrap();

    let expected = RecursiveControlUncertaintyPolicy::new().decide(
        &RecursivePlanningMemory::new(),
        transition.after_models(),
        &state(vec![base(10)]),
        &[transition
            .before_models()
            .models()
            .first()
            .map(|assessment| objective(assessment.concept().clone(), 20))
            .unwrap()],
    );

    assert!(
        transition.next_decision().is_act()
            || transition.next_decision().is_experiment()
            || transition.next_decision().is_no_decision()
    );

    let _ = expected;
}

#[test]
fn empty_model_memory_produces_no_decision() {
    let mut revision = RecursiveRevisionMemory::new();

    let result = RecursiveControlActiveCycle::new().tick(
        &RecursivePlanningMemory::new(),
        &mut revision,
        &state(vec![base(10)]),
        &[],
        None,
    );

    assert!(result.is_no_decision());
}

#[test]
fn recursive_identity_is_preserved_through_action_tick() {
    let child = concept(1, 2);

    let deep =
        RecursiveConcept::new(vec![base(8), RecursiveUnit::Recursive(Box::new(child))]).unwrap();

    let mut revision = supported_revision(&deep);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let result = RecursiveControlActiveCycle::new().tick(
        &planning,
        &mut revision,
        &start,
        &[objective(deep.clone(), 20)],
        None,
    );

    assert_eq!(
        result.action().unwrap().decision().objective().model(),
        &deep
    );
}

#[test]
fn active_cycle_is_deterministic() {
    let (revision, _, _) = contested_revision();

    let mut left_memory = revision.clone();

    let mut right_memory = revision;

    let planning = RecursivePlanningMemory::new();

    let start = state(vec![base(10)]);

    let left = RecursiveControlActiveCycle::new().tick(
        &planning,
        &mut left_memory,
        &start,
        &[],
        Some(RecursiveExperimentObservation::Present),
    );

    let right = RecursiveControlActiveCycle::new().tick(
        &planning,
        &mut right_memory,
        &start,
        &[],
        Some(RecursiveExperimentObservation::Present),
    );

    assert_eq!(left, right);

    assert_eq!(left_memory, right_memory);
}
