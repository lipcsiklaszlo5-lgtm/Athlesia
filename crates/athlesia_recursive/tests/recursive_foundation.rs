use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveMemory, RecursiveUnit};

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

#[test]
fn base_unit_preserves_identity() {
    let source = structural_unit(1);

    let unit = RecursiveUnit::Base(source.clone());

    assert!(unit.is_base());

    assert!(!unit.is_cross_level());

    assert!(!unit.is_recursive());

    assert_eq!(unit.base(), Some(&source));
}

#[test]
fn cross_level_unit_preserves_identity() {
    let source = cross_level(1, &[2, 3]);

    let unit = RecursiveUnit::CrossLevel(source.clone());

    assert!(unit.is_cross_level());

    assert!(unit.is_higher_order());

    assert_eq!(unit.cross_level(), Some(&source));
}

#[test]
fn base_only_concept_is_rejected() {
    assert!(RecursiveConcept::new(vec![base(1), base(2),],).is_none());
}

#[test]
fn single_higher_order_unit_is_rejected() {
    assert!(RecursiveConcept::new(vec![cross(1, &[2, 3],),],).is_none());
}

#[test]
fn duplicate_only_input_is_rejected() {
    let unit = cross(1, &[2, 3]);

    assert!(RecursiveConcept::new(vec![unit.clone(), unit,],).is_none());
}

#[test]
fn base_and_cross_level_form_recursive_concept() {
    let concept = RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap();

    assert_eq!(concept.len(), 2);

    assert_eq!(concept.base_count(), 1);

    assert_eq!(concept.cross_level_count(), 1);

    assert_eq!(concept.recursive_count(), 0);
}

#[test]
fn multiple_cross_level_units_form_recursive_concept() {
    let concept = RecursiveConcept::new(vec![cross(1, &[3, 4]), cross(2, &[3, 4])]).unwrap();

    assert_eq!(concept.cross_level_count(), 2);
}

#[test]
fn recursive_concept_can_be_child_of_recursive_concept() {
    let child = RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap();

    let parent = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(child.clone())),
    ])
    .unwrap();

    assert_eq!(parent.recursive_count(), 1);

    assert_eq!(parent.depth(), 2);

    assert_eq!(parent.units()[1].recursive(), Some(&child));
}

#[test]
fn deeper_recursive_depth_is_reported() {
    let level_one = RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap();

    let level_two =
        RecursiveConcept::new(vec![base(5), RecursiveUnit::Recursive(Box::new(level_one))])
            .unwrap();

    let level_three =
        RecursiveConcept::new(vec![base(6), RecursiveUnit::Recursive(Box::new(level_two))])
            .unwrap();

    assert_eq!(level_three.depth(), 3);
}

#[test]
fn child_order_does_not_change_identity() {
    let first = RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap();

    let second = RecursiveConcept::new(vec![cross(2, &[3, 4]), base(1)]).unwrap();

    assert_eq!(first, second);
}

#[test]
fn duplicate_units_are_canonicalized() {
    let base_unit = base(1);

    let cross_unit = cross(2, &[3, 4]);

    let concept = RecursiveConcept::new(vec![
        base_unit.clone(),
        cross_unit.clone(),
        base_unit,
        cross_unit,
    ])
    .unwrap();

    assert_eq!(concept.len(), 2);
}

#[test]
fn recursive_identity_preserves_cross_level_identity() {
    let first = RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap();

    let second = RecursiveConcept::new(vec![base(1), cross(2, &[3, 5])]).unwrap();

    assert_ne!(first, second);
}

#[test]
fn memory_stores_and_deduplicates_recursive_identity() {
    let concept = RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap();

    let mut memory = RecursiveMemory::new();

    assert!(memory.insert(concept.clone(),));

    assert!(!memory.insert(concept.clone(),));

    assert!(memory.contains(&concept));

    assert_eq!(memory.len(), 1);
}

#[test]
fn memory_iteration_is_deterministic() {
    let first = RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap();

    let second = RecursiveConcept::new(vec![base(5), cross(6, &[7, 8])]).unwrap();

    let mut left = RecursiveMemory::new();

    left.insert(first.clone());

    left.insert(second.clone());

    let mut right = RecursiveMemory::new();

    right.insert(second);

    right.insert(first);

    let left_items: Vec<_> = left.concepts().cloned().collect();

    let right_items: Vec<_> = right.concepts().cloned().collect();

    assert_eq!(left_items, right_items);
}
