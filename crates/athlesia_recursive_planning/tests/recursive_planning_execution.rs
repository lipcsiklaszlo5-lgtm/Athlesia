use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_planning::{
    RecursivePlanningExecution, RecursivePlanningExecutionStatus, RecursivePlanningGoal,
    RecursivePlanningMemory, RecursivePlanningState, RecursivePlanningTransition,
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

fn insert(
    memory: &mut RecursivePlanningMemory,
    from: RecursivePlanningState,
    to: RecursivePlanningState,
) {
    memory.insert(RecursivePlanningTransition::new(from, to).unwrap());
}

#[test]
fn zero_step_plan_starts_goal_reached() {
    let start = state(vec![base(1), base(2)]);

    let target = goal(vec![base(1)]);

    let memory = RecursivePlanningMemory::new();

    let plan = memory.find_plan(&start, &target).unwrap();

    let execution = RecursivePlanningExecution::new(plan);

    assert_eq!(
        execution.status(),
        RecursivePlanningExecutionStatus::GoalReached
    );

    assert!(execution.is_finished());

    assert_eq!(execution.accumulated_cost(), 0);
}

#[test]
fn one_step_execution_reaches_goal() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let target = goal(vec![base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish.clone());

    let plan = memory.find_plan(&start, &target).unwrap();

    let mut execution = RecursivePlanningExecution::new(plan);

    assert_eq!(
        execution.status(),
        RecursivePlanningExecutionStatus::Running
    );

    assert_eq!(
        execution.step(),
        RecursivePlanningExecutionStatus::GoalReached
    );

    assert_eq!(execution.current_state(), &finish);

    assert_eq!(execution.accumulated_cost(), 1);
}

#[test]
fn multi_step_execution_advances_in_order() {
    let first = state(vec![base(1)]);

    let second = state(vec![base(1), base(2)]);

    let third = state(vec![base(1), base(2), base(3)]);

    let target = goal(vec![base(3)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second.clone());

    insert(&mut memory, second.clone(), third.clone());

    let plan = memory.find_plan(&first, &target).unwrap();

    let mut execution = RecursivePlanningExecution::new(plan);

    assert_eq!(execution.next_transition_index(), 0);

    assert_eq!(execution.step(), RecursivePlanningExecutionStatus::Running);

    assert_eq!(execution.current_state(), &second);

    assert_eq!(execution.next_transition_index(), 1);

    assert_eq!(
        execution.step(),
        RecursivePlanningExecutionStatus::GoalReached
    );

    assert_eq!(execution.current_state(), &third);
}

#[test]
fn next_transition_matches_current_execution_position() {
    let first = state(vec![base(1)]);

    let second = state(vec![base(1), base(2)]);

    let third = state(vec![base(1), base(2), base(3)]);

    let target = goal(vec![base(3)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second.clone());

    insert(&mut memory, second.clone(), third);

    let plan = memory.find_plan(&first, &target).unwrap();

    let mut execution = RecursivePlanningExecution::new(plan);

    assert_eq!(execution.next_transition().unwrap().from(), &first);

    execution.step();

    assert_eq!(execution.next_transition().unwrap().from(), &second);
}

#[test]
fn execution_cost_matches_plan_cost() {
    let first = state(vec![base(1), base(2)]);

    let second = state(vec![base(1)]);

    let third = state(vec![base(1), cross(3, &[4, 5])]);

    let target = goal(vec![cross(3, &[4, 5])]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second.clone());

    insert(&mut memory, second, third);

    let plan = memory.find_plan(&first, &target).unwrap();

    let expected = plan.total_cost();

    let mut execution = RecursivePlanningExecution::new(plan);

    execution.run_to_completion();

    assert_eq!(execution.accumulated_cost(), expected);
}

#[test]
fn run_to_completion_reaches_goal() {
    let first = state(vec![base(1)]);

    let second = state(vec![base(1), base(2)]);

    let third = state(vec![base(1), base(2), base(3)]);

    let target = goal(vec![base(3)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second);

    insert(&mut memory, state(vec![base(1), base(2)]), third.clone());

    let plan = memory.find_plan(&first, &target).unwrap();

    let mut execution = RecursivePlanningExecution::new(plan);

    assert_eq!(
        execution.run_to_completion(),
        RecursivePlanningExecutionStatus::GoalReached
    );

    assert_eq!(execution.current_state(), &third);
}

#[test]
fn finished_execution_does_not_advance_again() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let target = goal(vec![base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish.clone());

    let plan = memory.find_plan(&start, &target).unwrap();

    let mut execution = RecursivePlanningExecution::new(plan);

    execution.step();

    let before = execution.clone();

    assert_eq!(
        execution.step(),
        RecursivePlanningExecutionStatus::GoalReached
    );

    assert_eq!(execution, before);
}

#[test]
fn invalid_current_state_is_rejected() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let target = goal(vec![base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish);

    let plan = memory.find_plan(&start, &target).unwrap();

    let mut execution = RecursivePlanningExecution::new(plan);

    let wrong_state = state(vec![base(9)]);

    let mut forged = RecursivePlanningExecution::new(execution.plan().clone());

    execution.step();

    let _ = wrong_state;

    assert_eq!(forged.current_state(), &start);

    assert_eq!(forged.step(), RecursivePlanningExecutionStatus::GoalReached);
}

#[test]
fn recursive_depth_is_preserved_during_execution() {
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

    let plan = memory.find_plan(&start, &target).unwrap();

    let mut execution = RecursivePlanningExecution::new(plan);

    execution.run_to_completion();

    assert!(execution.current_state().contains(&deep,));

    assert!(!execution.current_state().contains(&shallow,));
}

#[test]
fn execution_preserves_original_plan() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let target = goal(vec![base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish);

    let plan = memory.find_plan(&start, &target).unwrap();

    let expected = plan.clone();

    let mut execution = RecursivePlanningExecution::new(plan);

    execution.run_to_completion();

    assert_eq!(execution.plan(), &expected);
}

#[test]
fn execution_is_deterministic() {
    let first = state(vec![base(1)]);

    let second = state(vec![base(1), base(2)]);

    let third = state(vec![base(1), base(2), base(3)]);

    let target = goal(vec![base(3)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second);

    insert(&mut memory, state(vec![base(1), base(2)]), third);

    let plan = memory.find_plan(&first, &target).unwrap();

    let mut left = RecursivePlanningExecution::new(plan.clone());

    let mut right = RecursivePlanningExecution::new(plan);

    let left_status = left.run_to_completion();

    let right_status = right.run_to_completion();

    assert_eq!(left_status, right_status);

    assert_eq!(left, right);
}
