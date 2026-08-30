use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept, CrossLevelMemory};

use athlesia_hierarchy::HierarchicalConcept;

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

#[test]
fn unit_preserves_structural_level() {
    let unit = structural_unit(1);

    assert!(unit.is_structural());
    assert!(!unit.is_hierarchical());

    assert_eq!(unit.structural(), Some(&structural(1)));

    assert!(unit.hierarchical().is_none());
}

#[test]
fn unit_preserves_hierarchical_level() {
    let hierarchy = hierarchy(&[1, 2]);

    let unit = AbstractionUnit::Hierarchical(hierarchy.clone());

    assert!(unit.is_hierarchical());
    assert!(!unit.is_structural());

    assert_eq!(unit.hierarchical(), Some(&hierarchy));

    assert!(unit.structural().is_none());
}

#[test]
fn structural_only_concept_is_rejected() {
    assert!(CrossLevelConcept::new(vec![structural_unit(1), structural_unit(2),],).is_none());
}

#[test]
fn hierarchical_only_concept_is_rejected() {
    assert!(CrossLevelConcept::new(vec![
        hierarchical_unit(&[1, 2],),
        hierarchical_unit(&[2, 3],),
    ],)
    .is_none());
}

#[test]
fn mixed_levels_form_cross_level_concept() {
    let concept =
        CrossLevelConcept::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]).unwrap();

    assert_eq!(concept.len(), 2);
    assert_eq!(concept.structural_count(), 1);
    assert_eq!(concept.hierarchical_count(), 1);
}

#[test]
fn unit_order_does_not_change_identity() {
    let first =
        CrossLevelConcept::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]).unwrap();

    let second =
        CrossLevelConcept::new(vec![hierarchical_unit(&[2, 3]), structural_unit(1)]).unwrap();

    assert_eq!(first, second);
}

#[test]
fn duplicate_units_are_canonicalized() {
    let structural = structural_unit(1);

    let hierarchical = hierarchical_unit(&[2, 3]);

    let concept = CrossLevelConcept::new(vec![
        structural.clone(),
        hierarchical.clone(),
        structural,
        hierarchical,
    ])
    .unwrap();

    assert_eq!(concept.len(), 2);
}

#[test]
fn cross_level_identity_preserves_structural_extent() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        6,
    );

    let hierarchical = hierarchical_unit(&[2, 3]);

    let first = CrossLevelConcept::new(vec![
        AbstractionUnit::Structural(short),
        hierarchical.clone(),
    ])
    .unwrap();

    let second =
        CrossLevelConcept::new(vec![AbstractionUnit::Structural(long), hierarchical]).unwrap();

    assert_ne!(first, second);
}

#[test]
fn cross_level_identity_preserves_hierarchy_identity() {
    let structural = structural_unit(1);

    let first =
        CrossLevelConcept::new(vec![structural.clone(), hierarchical_unit(&[2, 3])]).unwrap();

    let second = CrossLevelConcept::new(vec![structural, hierarchical_unit(&[2, 4])]).unwrap();

    assert_ne!(first, second);
}

#[test]
fn memory_starts_empty() {
    let memory = CrossLevelMemory::new();

    assert!(memory.is_empty());
    assert_eq!(memory.len(), 0);
}

#[test]
fn memory_stores_cross_level_concept() {
    let mut memory = CrossLevelMemory::new();

    let concept =
        CrossLevelConcept::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]).unwrap();

    assert!(memory.insert(concept.clone()));

    assert!(memory.contains(&concept));

    assert_eq!(memory.len(), 1);
}

#[test]
fn memory_deduplicates_canonical_identity() {
    let mut memory = CrossLevelMemory::new();

    let first =
        CrossLevelConcept::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]).unwrap();

    let second =
        CrossLevelConcept::new(vec![hierarchical_unit(&[2, 3]), structural_unit(1)]).unwrap();

    assert!(memory.insert(first));
    assert!(!memory.insert(second));

    assert_eq!(memory.len(), 1);
}

#[test]
fn memory_iteration_is_deterministic() {
    let first_concept =
        CrossLevelConcept::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]).unwrap();

    let second_concept =
        CrossLevelConcept::new(vec![structural_unit(2), hierarchical_unit(&[3, 4])]).unwrap();

    let mut first = CrossLevelMemory::new();

    first.insert(first_concept.clone());

    first.insert(second_concept.clone());

    let mut second = CrossLevelMemory::new();

    second.insert(second_concept);

    second.insert(first_concept);

    let first_items: Vec<_> = first.concepts().cloned().collect();

    let second_items: Vec<_> = second.concepts().cloned().collect();

    assert_eq!(first_items, second_items);
}
