use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_planning::{
    RecursivePlanningMemory, RecursivePlanningState, RecursivePlanningSuccessorGenerator,
    RecursivePlanningTransition,
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

fn recursive(base_span: usize, cross_span: usize) -> RecursiveUnit {
    let concept = RecursiveConcept::new(vec![
        base(base_span),
        cross(cross_span, &[cross_span + 1, cross_span + 2]),
    ])
    .unwrap();

    RecursiveUnit::Recursive(Box::new(concept))
}

fn state(units: Vec<RecursiveUnit>) -> RecursivePlanningState {
    RecursivePlanningState::new(units)
}

#[test]
fn empty_memory_has_no_successors() {
    let memory = RecursivePlanningMemory::new();

    let current = state(vec![base(1)]);

    assert!(memory.successors(&current,).is_empty());
}

#[test]
fn unrelated_transitions_are_not_successors() {
    let current = state(vec![base(1)]);

    let unrelated_from = state(vec![base(5)]);

    let unrelated_to = state(vec![base(5), cross(6, &[7, 8])]);

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(RecursivePlanningTransition::new(unrelated_from, unrelated_to).unwrap());

    assert!(memory.successors(&current,).is_empty());
}

#[test]
fn exact_source_state_generates_successor() {
    let current = state(vec![base(1)]);

    let target = state(vec![base(1), cross(2, &[3, 4])]);

    let transition = RecursivePlanningTransition::new(current.clone(), target.clone()).unwrap();

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(transition.clone());

    let successors = memory.successors(&current);

    assert_eq!(successors.len(), 1);

    assert_eq!(successors[0].state(), &target);

    assert_eq!(successors[0].transition(), &transition);
}

#[test]
fn multiple_transitions_generate_multiple_successors() {
    let current = state(vec![base(1)]);

    let first = state(vec![base(1), cross(2, &[3, 4])]);

    let second = state(vec![base(1), recursive(5, 6)]);

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(RecursivePlanningTransition::new(current.clone(), first).unwrap());

    memory.insert(RecursivePlanningTransition::new(current.clone(), second).unwrap());

    assert_eq!(memory.successors(&current,).len(), 2);
}

#[test]
fn successor_preserves_transition_cost() {
    let current = state(vec![base(1), base(2)]);

    let target = state(vec![base(1), cross(3, &[4, 5])]);

    let transition = RecursivePlanningTransition::new(current.clone(), target).unwrap();

    assert_eq!(transition.cost(), 2);

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(transition);

    let successors = memory.successors(&current);

    assert_eq!(successors[0].cost(), 2);
}

#[test]
fn cheaper_successor_ranks_first() {
    let current = state(vec![base(1), base(2)]);

    let cheap = state(vec![base(1)]);

    let expensive = state(vec![cross(3, &[4, 5])]);

    let cheap_transition =
        RecursivePlanningTransition::new(current.clone(), cheap.clone()).unwrap();

    let expensive_transition =
        RecursivePlanningTransition::new(current.clone(), expensive).unwrap();

    assert!(cheap_transition.cost() < expensive_transition.cost());

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(expensive_transition);

    memory.insert(cheap_transition);

    let successors = memory.successors(&current);

    assert_eq!(successors[0].state(), &cheap);
}

#[test]
fn equal_cost_successors_use_state_identity() {
    let current = state(vec![base(1)]);

    let first = state(vec![base(1), base(2)]);

    let second = state(vec![base(1), base(3)]);

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(RecursivePlanningTransition::new(current.clone(), second.clone()).unwrap());

    memory.insert(RecursivePlanningTransition::new(current.clone(), first.clone()).unwrap());

    let successors = memory.successors(&current);

    let expected = if first < second { first } else { second };

    assert_eq!(successors[0].state(), &expected);
}

#[test]
fn exact_state_identity_is_required() {
    let current = state(vec![base(1)]);

    let superset = state(vec![base(1), base(2)]);

    let target = state(vec![base(1), base(2), cross(3, &[4, 5])]);

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(RecursivePlanningTransition::new(superset, target).unwrap());

    assert!(memory.successors(&current,).is_empty());
}

#[test]
fn recursive_successor_preserves_depth_identity() {
    let shallow = recursive(1, 2);

    let child = match &shallow {
        RecursiveUnit::Recursive(concept) => (**concept).clone(),
        _ => unreachable!(),
    };

    let deep_concept =
        RecursiveConcept::new(vec![base(5), RecursiveUnit::Recursive(Box::new(child))]).unwrap();

    let deep = RecursiveUnit::Recursive(Box::new(deep_concept));

    let current = state(vec![shallow.clone()]);

    let target = state(vec![deep.clone()]);

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(RecursivePlanningTransition::new(current.clone(), target.clone()).unwrap());

    let successors = memory.successors(&current);

    assert_eq!(successors.len(), 1);

    assert!(successors[0].state().contains(&deep,));

    assert!(!successors[0].state().contains(&shallow,));
}

#[test]
fn generator_and_memory_api_are_equivalent() {
    let current = state(vec![base(1)]);

    let target = state(vec![base(1), cross(2, &[3, 4])]);

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(RecursivePlanningTransition::new(current.clone(), target).unwrap());

    let direct = RecursivePlanningSuccessorGenerator::new().generate(&memory, &current);

    let convenience = memory.successors(&current);

    assert_eq!(direct, convenience);
}

#[test]
fn successor_generation_is_deterministic() {
    let current = state(vec![base(1)]);

    let first = state(vec![base(1), base(2)]);

    let second = state(vec![base(1), cross(3, &[4, 5])]);

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(RecursivePlanningTransition::new(current.clone(), second).unwrap());

    memory.insert(RecursivePlanningTransition::new(current.clone(), first).unwrap());

    let left = memory.successors(&current);

    let right = memory.successors(&current);

    assert_eq!(left, right);
}

#[test]
fn successor_generation_does_not_mutate_memory() {
    let current = state(vec![base(1)]);

    let target = state(vec![base(1), cross(2, &[3, 4])]);

    let mut memory = RecursivePlanningMemory::new();

    memory.insert(RecursivePlanningTransition::new(current.clone(), target).unwrap());

    let before = memory.clone();

    let _ = memory.successors(&current);

    assert_eq!(memory, before);
}
