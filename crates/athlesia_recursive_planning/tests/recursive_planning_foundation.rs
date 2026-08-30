use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_planning::{
    RecursivePlanningGoal, RecursivePlanningMemory, RecursivePlanningState,
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

fn recursive_unit() -> RecursiveUnit {
    let concept = RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap();

    RecursiveUnit::Recursive(Box::new(concept))
}

#[test]
fn planning_state_canonicalizes_order() {
    let left = RecursivePlanningState::new(vec![cross(2, &[3, 4]), base(1)]);

    let right = RecursivePlanningState::new(vec![base(1), cross(2, &[3, 4])]);

    assert_eq!(left, right);
}

#[test]
fn planning_state_deduplicates_units() {
    let state = RecursivePlanningState::new(vec![base(1), base(1), cross(2, &[3, 4])]);

    assert_eq!(state.len(), 2);

    assert!(!state.is_empty());
}

#[test]
fn planning_state_preserves_recursive_unit_identity() {
    let recursive = recursive_unit();

    let state = RecursivePlanningState::new(vec![recursive.clone()]);

    assert!(state.contains(&recursive));
}

#[test]
fn empty_goal_is_rejected() {
    assert!(RecursivePlanningGoal::new(Vec::new(),).is_none());
}

#[test]
fn goal_canonicalizes_duplicate_requirements() {
    let goal = RecursivePlanningGoal::new(vec![base(1), base(1), cross(2, &[3, 4])]).unwrap();

    assert_eq!(goal.len(), 2);

    assert!(!goal.is_empty());
}

#[test]
fn goal_is_satisfied_by_superset_state() {
    let goal = RecursivePlanningGoal::new(vec![base(1)]).unwrap();

    let state = RecursivePlanningState::new(vec![base(1), cross(2, &[3, 4])]);

    assert!(goal.is_satisfied_by(&state,));
}

#[test]
fn goal_reports_missing_units() {
    let goal = RecursivePlanningGoal::new(vec![base(1), cross(2, &[3, 4])]).unwrap();

    let state = RecursivePlanningState::new(vec![base(1)]);

    assert_eq!(goal.missing_units(&state,), vec![cross(2, &[3, 4],),]);
}

#[test]
fn identical_states_do_not_form_transition() {
    let state = RecursivePlanningState::new(vec![base(1)]);

    assert!(RecursivePlanningTransition::new(state.clone(), state,).is_none());
}

#[test]
fn transition_reports_added_units() {
    let from = RecursivePlanningState::new(vec![base(1)]);

    let target = cross(2, &[3, 4]);

    let to = RecursivePlanningState::new(vec![base(1), target.clone()]);

    let transition = RecursivePlanningTransition::new(from, to).unwrap();

    assert_eq!(transition.added(), &[target,]);

    assert!(transition.removed().is_empty());

    assert!(transition.is_pure_addition());
}

#[test]
fn transition_reports_removed_units() {
    let removed = cross(2, &[3, 4]);

    let from = RecursivePlanningState::new(vec![base(1), removed.clone()]);

    let to = RecursivePlanningState::new(vec![base(1)]);

    let transition = RecursivePlanningTransition::new(from, to).unwrap();

    assert_eq!(transition.removed(), &[removed,]);

    assert!(transition.added().is_empty());

    assert!(transition.is_pure_removal());
}

#[test]
fn transition_cost_counts_structural_changes() {
    let from = RecursivePlanningState::new(vec![base(1), base(2)]);

    let to = RecursivePlanningState::new(vec![base(1), cross(3, &[4, 5])]);

    let transition = RecursivePlanningTransition::new(from, to).unwrap();

    assert_eq!(transition.cost(), 2);
}

#[test]
fn recursive_depth_is_preserved_in_transition_identity() {
    let level_one = recursive_unit();

    let nested = match &level_one {
        RecursiveUnit::Recursive(concept) => RecursiveConcept::new(vec![
            base(5),
            RecursiveUnit::Recursive(Box::new((**concept).clone())),
        ])
        .unwrap(),
        _ => unreachable!(),
    };

    let level_two = RecursiveUnit::Recursive(Box::new(nested));

    let from = RecursivePlanningState::new(vec![level_one.clone()]);

    let to = RecursivePlanningState::new(vec![level_two.clone()]);

    let transition = RecursivePlanningTransition::new(from, to).unwrap();

    assert_eq!(transition.removed(), &[level_one,]);

    assert_eq!(transition.added(), &[level_two,]);
}

#[test]
fn planning_memory_deduplicates_transitions() {
    let transition = RecursivePlanningTransition::new(
        RecursivePlanningState::new(vec![base(1)]),
        RecursivePlanningState::new(vec![base(1), cross(2, &[3, 4])]),
    )
    .unwrap();

    let mut memory = RecursivePlanningMemory::new();

    assert!(memory.insert(transition.clone(),));

    assert!(!memory.insert(transition.clone(),));

    assert_eq!(memory.len(), 1);

    assert!(memory.contains(&transition));
}

#[test]
fn planning_memory_iteration_is_deterministic() {
    let first = RecursivePlanningTransition::new(
        RecursivePlanningState::new(vec![base(1)]),
        RecursivePlanningState::new(vec![base(1), cross(2, &[3, 4])]),
    )
    .unwrap();

    let second = RecursivePlanningTransition::new(
        RecursivePlanningState::new(vec![base(5)]),
        RecursivePlanningState::new(vec![base(5), cross(6, &[7, 8])]),
    )
    .unwrap();

    let mut left = RecursivePlanningMemory::new();

    left.insert(first.clone());

    left.insert(second.clone());

    let mut right = RecursivePlanningMemory::new();

    right.insert(second);

    right.insert(first);

    let left_items = left.transitions().cloned().collect::<Vec<_>>();

    let right_items = right.transitions().cloned().collect::<Vec<_>>();

    assert_eq!(left_items, right_items);
}
