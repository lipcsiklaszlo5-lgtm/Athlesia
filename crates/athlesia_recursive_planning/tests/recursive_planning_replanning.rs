use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_planning::{
    RecursivePlanningExecution, RecursivePlanningExecutionStatus, RecursivePlanningGoal,
    RecursivePlanningMemory, RecursivePlanningReplanner, RecursivePlanningState,
    RecursivePlanningTransition, RecursiveReplanningOutcome,
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
) {
    let start = state(vec![base(1)]);

    let middle = state(vec![base(1), base(2)]);

    let finish = state(vec![base(1), base(2), base(9)]);

    let target = goal(vec![base(9)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), middle.clone());

    insert(&mut memory, middle.clone(), finish);

    let plan = memory.find_plan(&start, &target).unwrap();

    (memory, RecursivePlanningExecution::new(plan), start, middle)
}

#[test]
fn matching_observation_preserves_running_plan() {
    let (memory, execution, start, _) = base_execution();

    let result = RecursivePlanningReplanner::new().reconcile(&memory, &execution, start);

    assert_eq!(result.outcome(), RecursiveReplanningOutcome::PlanPreserved);

    assert_eq!(result.execution(), Some(&execution));
}

#[test]
fn divergent_observation_triggers_replanning() {
    let (mut memory, execution, _, _) = base_execution();

    let observed = state(vec![base(5)]);

    let finish = state(vec![base(5), base(9)]);

    insert(&mut memory, observed.clone(), finish);

    let result = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed.clone());

    assert_eq!(result.outcome(), RecursiveReplanningOutcome::Replanned);

    assert_eq!(result.execution().unwrap().current_state(), &observed);
}

#[test]
fn replanned_execution_keeps_original_goal() {
    let (mut memory, execution, _, _) = base_execution();

    let observed = state(vec![base(5)]);

    insert(&mut memory, observed.clone(), state(vec![base(5), base(9)]));

    let original_goal = execution.plan().goal().clone();

    let result = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed);

    assert_eq!(result.execution().unwrap().plan().goal(), &original_goal);
}

#[test]
fn replanning_selects_lowest_cost_route() {
    let (mut memory, execution, _, _) = base_execution();

    let observed = state(vec![base(5), base(6)]);

    let cheap_middle = state(vec![base(5)]);

    let cheap_finish = state(vec![base(5), base(9)]);

    let expensive_finish = state(vec![base(9), cross(10, &[11, 12])]);

    insert(&mut memory, observed.clone(), expensive_finish);

    insert(&mut memory, observed.clone(), cheap_middle.clone());

    insert(&mut memory, cheap_middle, cheap_finish.clone());

    let result = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed);

    let replanned = result.execution().unwrap();

    assert_eq!(replanned.plan().total_cost(), 2);

    assert_eq!(replanned.plan().final_state(), &cheap_finish);
}

#[test]
fn already_satisfied_observation_stops_immediately() {
    let (memory, execution, _, _) = base_execution();

    let observed = state(vec![base(9), base(20)]);

    let result = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed.clone());

    assert_eq!(result.outcome(), RecursiveReplanningOutcome::GoalReached);

    let replanned = result.execution().unwrap();

    assert_eq!(
        replanned.status(),
        RecursivePlanningExecutionStatus::GoalReached
    );

    assert_eq!(replanned.current_state(), &observed);

    assert_eq!(replanned.plan().total_cost(), 0);
}

#[test]
fn unreachable_observation_returns_no_execution() {
    let (memory, execution, _, _) = base_execution();

    let observed = state(vec![base(77)]);

    let result = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed);

    assert_eq!(result.outcome(), RecursiveReplanningOutcome::Unreachable);

    assert!(!result.has_plan());

    assert!(result.execution().is_none());
}

#[test]
fn observed_state_is_preserved_in_result() {
    let (memory, execution, _, _) = base_execution();

    let observed = state(vec![base(77)]);

    let result = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed.clone());

    assert_eq!(result.observed_state(), &observed);
}

#[test]
fn replanned_execution_can_continue_to_goal() {
    let (mut memory, execution, _, _) = base_execution();

    let observed = state(vec![base(5)]);

    let finish = state(vec![base(5), base(9)]);

    insert(&mut memory, observed.clone(), finish.clone());

    let result = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed);

    let mut replanned = result.into_execution().unwrap();

    assert_eq!(
        replanned.run_to_completion(),
        RecursivePlanningExecutionStatus::GoalReached
    );

    assert_eq!(replanned.current_state(), &finish);
}

#[test]
fn recursive_depth_is_preserved_during_replanning() {
    let shallow = recursive(1, 2);

    let child = match &shallow {
        RecursiveUnit::Recursive(concept) => (**concept).clone(),
        _ => unreachable!(),
    };

    let deep = RecursiveUnit::Recursive(Box::new(
        RecursiveConcept::new(vec![base(6), RecursiveUnit::Recursive(Box::new(child))]).unwrap(),
    ));

    let original_start = state(vec![base(1)]);

    let original_finish = state(vec![base(1), deep.clone()]);

    let target = goal(vec![deep.clone()]);

    let observed = state(vec![shallow.clone()]);

    let replanned_finish = state(vec![deep.clone()]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, original_start.clone(), original_finish);

    insert(&mut memory, observed.clone(), replanned_finish);

    let original_plan = memory.find_plan(&original_start, &target).unwrap();

    let execution = RecursivePlanningExecution::new(original_plan);

    let result = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed);

    let replanned = result.execution().unwrap();

    assert!(replanned.plan().final_state().contains(&deep,));

    assert!(!replanned.plan().final_state().contains(&shallow,));
}

#[test]
fn replanning_does_not_mutate_memory_or_original_execution() {
    let (mut memory, execution, _, _) = base_execution();

    let observed = state(vec![base(5)]);

    insert(&mut memory, observed.clone(), state(vec![base(5), base(9)]));

    let memory_before = memory.clone();

    let execution_before = execution.clone();

    let _ = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed);

    assert_eq!(memory, memory_before);

    assert_eq!(execution, execution_before);
}

#[test]
fn replanning_is_deterministic() {
    let (mut memory, execution, _, _) = base_execution();

    let observed = state(vec![base(5)]);

    insert(&mut memory, observed.clone(), state(vec![base(5), base(9)]));

    let left = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed.clone());

    let right = RecursivePlanningReplanner::new().reconcile(&memory, &execution, observed);

    assert_eq!(left, right);
}
