use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_planning::{
    RecursivePlanningGoal, RecursivePlanningMemory, RecursivePlanningSearch,
    RecursivePlanningState, RecursivePlanningTransition,
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
fn already_satisfied_goal_returns_zero_step_plan() {
    let start = state(vec![base(1), base(2)]);

    let target = goal(vec![base(1)]);

    let memory = RecursivePlanningMemory::new();

    let plan = memory.find_plan(&start, &target).unwrap();

    assert_eq!(plan.len(), 0);

    assert!(plan.is_empty());

    assert_eq!(plan.total_cost(), 0);

    assert_eq!(plan.final_state(), &start);
}

#[test]
fn one_step_plan_is_found() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let target = goal(vec![base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish.clone());

    let plan = memory.find_plan(&start, &target).unwrap();

    assert_eq!(plan.len(), 1);

    assert_eq!(plan.total_cost(), 1);

    assert_eq!(plan.final_state(), &finish);
}

#[test]
fn multi_step_plan_is_found() {
    let first = state(vec![base(1)]);

    let second = state(vec![base(1), base(2)]);

    let third = state(vec![base(1), base(2), cross(3, &[4, 5])]);

    let target = goal(vec![cross(3, &[4, 5])]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second);

    insert(&mut memory, state(vec![base(1), base(2)]), third.clone());

    let plan = memory.find_plan(&first, &target).unwrap();

    assert_eq!(plan.len(), 2);

    assert_eq!(plan.total_cost(), 2);

    assert_eq!(plan.final_state(), &third);
}

#[test]
fn unreachable_goal_returns_none() {
    let start = state(vec![base(1)]);

    let target = goal(vec![base(9)]);

    let memory = RecursivePlanningMemory::new();

    assert!(memory.find_plan(&start, &target,).is_none());
}

#[test]
fn lower_total_cost_beats_shorter_path() {
    let start = state(vec![base(1), base(2)]);

    let cheap_middle = state(vec![base(1)]);

    let cheap_finish = state(vec![base(1), base(9)]);

    let expensive_finish = state(vec![base(9), cross(3, &[4, 5])]);

    let target = goal(vec![base(9)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), expensive_finish);

    insert(&mut memory, start.clone(), cheap_middle.clone());

    insert(&mut memory, cheap_middle, cheap_finish.clone());

    let plan = memory.find_plan(&start, &target).unwrap();

    assert_eq!(plan.len(), 2);

    assert_eq!(plan.total_cost(), 2);

    assert_eq!(plan.final_state(), &cheap_finish);
}

#[test]
fn cyclic_graph_terminates_and_finds_goal() {
    let first = state(vec![base(1)]);

    let second = state(vec![base(1), base(2)]);

    let third = state(vec![base(1), base(2), base(3)]);

    let target = goal(vec![base(3)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second.clone());

    insert(&mut memory, second.clone(), first.clone());

    insert(&mut memory, second, third.clone());

    let plan = memory.find_plan(&first, &target).unwrap();

    assert_eq!(plan.len(), 2);

    assert_eq!(plan.final_state(), &third);
}

#[test]
fn goal_can_be_satisfied_by_superset_final_state() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2), base(3)]);

    let target = goal(vec![base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish.clone());

    let plan = memory.find_plan(&start, &target).unwrap();

    assert_eq!(plan.final_state(), &finish);

    assert!(target.is_satisfied_by(plan.final_state(),));
}

#[test]
fn plan_transitions_form_continuous_path() {
    let first = state(vec![base(1)]);

    let second = state(vec![base(1), base(2)]);

    let third = state(vec![base(1), base(2), base(3)]);

    let target = goal(vec![base(3)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second.clone());

    insert(&mut memory, second.clone(), third);

    let plan = memory.find_plan(&first, &target).unwrap();

    assert_eq!(plan.transitions()[0].from(), &first);

    assert_eq!(plan.transitions()[0].to(), &second);

    assert_eq!(plan.transitions()[1].from(), &second);
}

#[test]
fn plan_total_cost_equals_transition_cost_sum() {
    let first = state(vec![base(1), base(2)]);

    let second = state(vec![base(1)]);

    let third = state(vec![base(1), cross(3, &[4, 5])]);

    let target = goal(vec![cross(3, &[4, 5])]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, first.clone(), second.clone());

    insert(&mut memory, second, third);

    let plan = memory.find_plan(&first, &target).unwrap();

    let sum = plan
        .transitions()
        .iter()
        .map(RecursivePlanningTransition::cost)
        .sum::<usize>();

    assert_eq!(plan.total_cost(), sum);
}

#[test]
fn recursive_depth_is_preserved_through_search() {
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

    assert!(plan.final_state().contains(&deep,));

    assert!(!plan.final_state().contains(&shallow,));

    assert_eq!(plan.final_state(), &finish);
}

#[test]
fn equal_cost_paths_are_resolved_deterministically() {
    let start = state(vec![base(1)]);

    let left = state(vec![base(1), base(2)]);

    let right = state(vec![base(1), base(3)]);

    let finish = state(vec![base(1), base(9)]);

    let target = goal(vec![base(9)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), right.clone());

    insert(&mut memory, start.clone(), left.clone());

    insert(&mut memory, left, finish.clone());

    insert(&mut memory, right, finish);

    let first = memory.find_plan(&start, &target).unwrap();

    let second = memory.find_plan(&start, &target).unwrap();

    assert_eq!(first, second);
}

#[test]
fn search_api_and_memory_api_are_equivalent_and_non_mutating() {
    let start = state(vec![base(1)]);

    let finish = state(vec![base(1), base(2)]);

    let target = goal(vec![base(2)]);

    let mut memory = RecursivePlanningMemory::new();

    insert(&mut memory, start.clone(), finish);

    let before = memory.clone();

    let direct = RecursivePlanningSearch::new().find_plan(&memory, &start, &target);

    let convenience = memory.find_plan(&start, &target);

    assert_eq!(direct, convenience);

    assert_eq!(memory, before);
}
