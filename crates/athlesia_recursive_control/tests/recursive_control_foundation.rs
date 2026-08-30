use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_planning::{
    RecursivePlanningExecutionStatus, RecursivePlanningGoal, RecursivePlanningMemory,
    RecursivePlanningState, RecursivePlanningTransition,
};

use athlesia_recursive_revision::{RecursiveCompetingModels, RecursiveRevisionMemory};

use athlesia_recursive_control::{RecursiveControlPlanner, RecursiveControlRequest};

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

fn recursive(base_span: usize, cross_span: usize) -> RecursiveUnit {
    RecursiveUnit::Recursive(Box::new(
        RecursiveConcept::new(vec![
            base(base_span),
            cross(cross_span, &[cross_span + 1, cross_span + 2]),
        ])
        .unwrap(),
    ))
}

fn state(units: Vec<RecursiveUnit>) -> RecursivePlanningState {
    RecursivePlanningState::new(units)
}

fn goal(units: Vec<RecursiveUnit>) -> RecursivePlanningGoal {
    RecursivePlanningGoal::new(units).unwrap()
}

fn models() -> RecursiveCompetingModels {
    RecursiveCompetingModels::from_memory(&RecursiveRevisionMemory::new())
}

fn insert(
    memory: &mut RecursivePlanningMemory,
    from: RecursivePlanningState,
    to: RecursivePlanningState,
) {
    memory.insert(RecursivePlanningTransition::new(from, to).unwrap());
}

#[test]
fn control_request_preserves_competing_models() {
    let competing = models();

    let request =
        RecursiveControlRequest::new(competing.clone(), state(vec![base(1)]), goal(vec![base(2)]));

    assert_eq!(request.models(), &competing);
}

#[test]
fn control_request_preserves_start_state() {
    let start = state(vec![base(1)]);

    let request = RecursiveControlRequest::new(models(), start.clone(), goal(vec![base(2)]));

    assert_eq!(request.start(), &start);
}

#[test]
fn control_request_preserves_goal() {
    let target = goal(vec![base(2)]);

    let request = RecursiveControlRequest::new(models(), state(vec![base(1)]), target.clone());

    assert_eq!(request.goal(), &target);
}

#[test]
fn reachable_request_produces_control_decision() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let target = goal(vec![base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish);

    let decision = RecursiveControlPlanner::new().prepare(
        &memory,
        RecursiveControlRequest::new(models(), start, target),
    );

    assert!(decision.is_some());
}

#[test]
fn unreachable_request_produces_no_decision() {
    let start = state(vec![base(1)]);

    let target = goal(vec![base(99)]);

    let memory = RecursivePlanningMemory::new();

    assert!(RecursiveControlPlanner::new()
        .prepare(
            &memory,
            RecursiveControlRequest::new(models(), start, target,),
        )
        .is_none());
}

#[test]
fn control_decision_preserves_request() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let target = goal(vec![base(2)]);

    let request = RecursiveControlRequest::new(models(), start.clone(), target);

    let expected = request.clone();

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start, finish);

    let decision = RecursiveControlPlanner::new()
        .prepare(&memory, request)
        .unwrap();

    assert_eq!(decision.request(), &expected);
}

#[test]
fn control_plan_starts_from_request_state() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish);

    let decision = RecursiveControlPlanner::new()
        .prepare(
            &memory,
            RecursiveControlRequest::new(models(), start.clone(), goal(vec![base(2)])),
        )
        .unwrap();

    assert_eq!(decision.plan().start(), &start);
}

#[test]
fn control_execution_starts_at_request_state() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish);

    let decision = RecursiveControlPlanner::new()
        .prepare(
            &memory,
            RecursiveControlRequest::new(models(), start.clone(), goal(vec![base(2)])),
        )
        .unwrap();

    assert_eq!(decision.execution().current_state(), &start);

    assert_eq!(
        decision.execution().status(),
        RecursivePlanningExecutionStatus::Running
    );
}

#[test]
fn already_satisfied_control_request_is_goal_reached() {
    let start = state(vec![base(1), base(2)]);

    let memory = RecursivePlanningMemory::new();

    let decision = RecursiveControlPlanner::new()
        .prepare(
            &memory,
            RecursiveControlRequest::new(models(), start.clone(), goal(vec![base(1)])),
        )
        .unwrap();

    assert_eq!(decision.plan().total_cost(), 0);

    assert_eq!(
        decision.execution().status(),
        RecursivePlanningExecutionStatus::GoalReached
    );

    assert_eq!(decision.execution().current_state(), &start);
}

#[test]
fn multi_step_control_plan_is_preserved() {
    let first = state(vec![base(1)]);

    let second = state(vec![base(1), base(2)]);

    let third = state(vec![base(1), base(2), base(3)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second.clone());

    insert(&mut memory, second, third);

    let decision = RecursiveControlPlanner::new()
        .prepare(
            &memory,
            RecursiveControlRequest::new(models(), first, goal(vec![base(3)])),
        )
        .unwrap();

    assert_eq!(decision.plan().len(), 2);

    assert_eq!(decision.plan().total_cost(), 2);
}

#[test]
fn recursive_depth_is_preserved_in_control_decision() {
    let shallow = recursive(1, 2);

    let child = match &shallow {
        RecursiveUnit::Recursive(concept) => (**concept).clone(),
        _ => unreachable!(),
    };

    let deep = RecursiveUnit::Recursive(Box::new(
        RecursiveConcept::new(vec![base(6), RecursiveUnit::Recursive(Box::new(child))]).unwrap(),
    ));

    let start = state(vec![shallow.clone()]);

    let finish = state(vec![deep.clone()]);

    let target = goal(vec![deep.clone()]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish);

    let decision = RecursiveControlPlanner::new()
        .prepare(
            &memory,
            RecursiveControlRequest::new(models(), start, target),
        )
        .unwrap();

    assert!(decision.plan().final_state().contains(&deep,));

    assert!(!decision.plan().final_state().contains(&shallow,));
}

#[test]
fn control_planning_is_deterministic_and_non_mutating() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let target = goal(vec![base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish);

    let memory_before = memory.clone();

    let request = RecursiveControlRequest::new(models(), start, target);

    let left = RecursiveControlPlanner::new().prepare(&memory, request.clone());

    let right = RecursiveControlPlanner::new().prepare(&memory, request);

    assert_eq!(left, right);

    assert_eq!(memory, memory_before);
}
