use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_planning::{
    RecursivePlanningActiveCycle, RecursivePlanningActiveOutcome, RecursivePlanningExecution,
    RecursivePlanningExecutionStatus, RecursivePlanningGoal, RecursivePlanningMemory,
    RecursivePlanningState, RecursivePlanningTransition, RecursiveReplanningOutcome,
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

fn base_execution() -> (
    RecursivePlanningMemory,
    RecursivePlanningExecution,
    RecursivePlanningState,
    RecursivePlanningState,
    RecursivePlanningState,
) {
    let start = state(vec![base(1)]);

    let middle = state(vec![base(1), base(2)]);

    let finish = state(vec![base(1), base(2), base(9)]);

    let target = goal(vec![base(9)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), middle.clone());

    insert(&mut memory, middle.clone(), finish.clone());

    let plan = memory.find_plan(&start, &target).unwrap();

    (
        memory,
        RecursivePlanningExecution::new(plan),
        start,
        middle,
        finish,
    )
}

#[test]
fn matching_observation_preserves_and_advances() {
    let (memory, execution, start, middle, _) = base_execution();

    let result = RecursivePlanningActiveCycle::new().tick(&memory, &execution, start);

    assert_eq!(
        result.reconciliation(),
        RecursiveReplanningOutcome::PlanPreserved
    );

    assert_eq!(
        result.outcome(),
        RecursivePlanningActiveOutcome::PreservedAndAdvanced
    );

    assert_eq!(result.execution().unwrap().current_state(), &middle);
}

#[test]
fn divergent_observation_replans_and_advances() {
    let (mut memory, execution, _, _, _) = base_execution();

    let observed = state(vec![base(5)]);

    let replanned_middle = state(vec![base(5), base(6)]);

    let replanned_finish = state(vec![base(5), base(6), base(9)]);

    insert(&mut memory, observed.clone(), replanned_middle.clone());

    insert(&mut memory, replanned_middle.clone(), replanned_finish);

    let result = RecursivePlanningActiveCycle::new().tick(&memory, &execution, observed);

    assert_eq!(
        result.reconciliation(),
        RecursiveReplanningOutcome::Replanned
    );

    assert_eq!(
        result.outcome(),
        RecursivePlanningActiveOutcome::ReplannedAndAdvanced
    );

    assert_eq!(
        result.execution().unwrap().current_state(),
        &replanned_middle
    );
}

#[test]
fn observation_already_at_goal_stops_without_step() {
    let (memory, execution, _, _, _) = base_execution();

    let observed = state(vec![base(9), base(20)]);

    let result = RecursivePlanningActiveCycle::new().tick(&memory, &execution, observed.clone());

    assert_eq!(
        result.reconciliation(),
        RecursiveReplanningOutcome::GoalReached
    );

    assert_eq!(
        result.outcome(),
        RecursivePlanningActiveOutcome::GoalReached
    );

    assert_eq!(result.execution().unwrap().current_state(), &observed);

    assert_eq!(result.execution().unwrap().accumulated_cost(), 0);
}

#[test]
fn unreachable_observation_returns_no_execution() {
    let (memory, execution, _, _, _) = base_execution();

    let observed = state(vec![base(77)]);

    let result = RecursivePlanningActiveCycle::new().tick(&memory, &execution, observed);

    assert_eq!(
        result.outcome(),
        RecursivePlanningActiveOutcome::Unreachable
    );

    assert!(result.execution().is_none());
}

#[test]
fn final_preserved_step_reports_goal_reached() {
    let (memory, mut execution, _, middle, finish) = base_execution();

    assert_eq!(execution.step(), RecursivePlanningExecutionStatus::Running);

    assert_eq!(execution.current_state(), &middle);

    let result = RecursivePlanningActiveCycle::new().tick(&memory, &execution, middle);

    assert_eq!(
        result.outcome(),
        RecursivePlanningActiveOutcome::GoalReached
    );

    assert_eq!(result.execution().unwrap().current_state(), &finish);
}

#[test]
fn final_replanned_step_reports_goal_reached() {
    let (mut memory, execution, _, _, _) = base_execution();

    let observed = state(vec![base(5)]);

    let finish = state(vec![base(5), base(9)]);

    insert(&mut memory, observed.clone(), finish.clone());

    let result = RecursivePlanningActiveCycle::new().tick(&memory, &execution, observed);

    assert_eq!(
        result.reconciliation(),
        RecursiveReplanningOutcome::Replanned
    );

    assert_eq!(
        result.outcome(),
        RecursivePlanningActiveOutcome::GoalReached
    );

    assert_eq!(result.execution().unwrap().current_state(), &finish);
}

#[test]
fn active_cycle_advances_exactly_one_transition() {
    let (memory, execution, start, _, _) = base_execution();

    let result = RecursivePlanningActiveCycle::new().tick(&memory, &execution, start);

    let next = result.execution().unwrap();

    assert_eq!(next.next_transition_index(), 1);

    assert_eq!(next.accumulated_cost(), 1);
}

#[test]
fn active_cycle_preserves_observed_state_record() {
    let (memory, execution, start, _, _) = base_execution();

    let result = RecursivePlanningActiveCycle::new().tick(&memory, &execution, start.clone());

    assert_eq!(result.observed_state(), &start);
}

#[test]
fn active_cycle_preserves_recursive_depth() {
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

    insert(&mut memory, start.clone(), finish.clone());

    let plan = memory.find_plan(&start, &target).unwrap();

    let execution = RecursivePlanningExecution::new(plan);

    let result = RecursivePlanningActiveCycle::new().tick(&memory, &execution, start);

    let next = result.execution().unwrap();

    assert!(next.current_state().contains(&deep,));

    assert!(!next.current_state().contains(&shallow,));

    assert_eq!(next.current_state(), &finish);
}

#[test]
fn active_cycle_does_not_mutate_inputs() {
    let (memory, execution, start, _, _) = base_execution();

    let memory_before = memory.clone();

    let execution_before = execution.clone();

    let _ = RecursivePlanningActiveCycle::new().tick(&memory, &execution, start);

    assert_eq!(memory, memory_before);

    assert_eq!(execution, execution_before);
}

#[test]
fn active_cycle_is_deterministic() {
    let (memory, execution, start, _, _) = base_execution();

    let left = RecursivePlanningActiveCycle::new().tick(&memory, &execution, start.clone());

    let right = RecursivePlanningActiveCycle::new().tick(&memory, &execution, start);

    assert_eq!(left, right);
}
