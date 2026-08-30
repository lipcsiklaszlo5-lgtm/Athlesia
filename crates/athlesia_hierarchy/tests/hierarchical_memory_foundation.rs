use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_hierarchy::{HierarchicalConcept, HierarchicalMemory};

fn concept(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        6,
    )
}

#[test]
fn hierarchy_requires_multiple_children() {
    assert!(HierarchicalConcept::new(vec![concept(1)]).is_none());
}

#[test]
fn hierarchy_rejects_duplicate_only_children() {
    let child = concept(1);

    assert!(HierarchicalConcept::new(vec![child.clone(), child,]).is_none());
}

#[test]
fn hierarchy_accepts_distinct_children() {
    let hierarchy = HierarchicalConcept::new(vec![concept(1), concept(2)]).unwrap();

    assert_eq!(hierarchy.arity(), 2);
}

#[test]
fn child_order_does_not_change_identity() {
    let first = HierarchicalConcept::new(vec![concept(1), concept(2)]).unwrap();

    let second = HierarchicalConcept::new(vec![concept(2), concept(1)]).unwrap();

    assert_eq!(first, second);
}

#[test]
fn duplicate_children_are_canonicalized() {
    let hierarchy = HierarchicalConcept::new(vec![concept(1), concept(2), concept(1)]).unwrap();

    assert_eq!(hierarchy.arity(), 2);
}

#[test]
fn hierarchy_contains_children() {
    let first = concept(1);
    let second = concept(2);

    let hierarchy = HierarchicalConcept::new(vec![first.clone(), second.clone()]).unwrap();

    assert!(hierarchy.contains(&first));
    assert!(hierarchy.contains(&second));
}

#[test]
fn unrelated_child_is_not_contained() {
    let hierarchy = HierarchicalConcept::new(vec![concept(1), concept(2)]).unwrap();

    assert!(!hierarchy.contains(&concept(3)));
}

#[test]
fn hierarchical_memory_starts_empty() {
    let memory = HierarchicalMemory::new();

    assert!(memory.is_empty());
    assert_eq!(memory.len(), 0);
}

#[test]
fn hierarchical_memory_stores_concept() {
    let mut memory = HierarchicalMemory::new();

    let hierarchy = HierarchicalConcept::new(vec![concept(1), concept(2)]).unwrap();

    assert!(memory.insert(hierarchy.clone()));

    assert!(memory.contains(&hierarchy));

    assert_eq!(memory.len(), 1);
}

#[test]
fn hierarchical_memory_deduplicates_identity() {
    let mut memory = HierarchicalMemory::new();

    let first = HierarchicalConcept::new(vec![concept(1), concept(2)]).unwrap();

    let second = HierarchicalConcept::new(vec![concept(2), concept(1)]).unwrap();

    assert!(memory.insert(first));
    assert!(!memory.insert(second));

    assert_eq!(memory.len(), 1);
}

#[test]
fn hierarchical_memory_iteration_is_deterministic() {
    let mut first = HierarchicalMemory::new();

    let mut second = HierarchicalMemory::new();

    let a = HierarchicalConcept::new(vec![concept(1), concept(2)]).unwrap();

    let b = HierarchicalConcept::new(vec![concept(2), concept(3)]).unwrap();

    first.insert(a.clone());
    first.insert(b.clone());

    second.insert(b);
    second.insert(a);

    let first_items: Vec<_> = first.concepts().cloned().collect();

    let second_items: Vec<_> = second.concepts().cloned().collect();

    assert_eq!(first_items, second_items);
}

#[test]
fn hierarchy_identity_preserves_child_extent() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        6,
    );

    let first = HierarchicalConcept::new(vec![short.clone(), concept(2)]).unwrap();

    let second = HierarchicalConcept::new(vec![long, concept(2)]).unwrap();

    assert_ne!(first, second);

    assert!(first.contains(&short));
}

#[test]
fn hierarchy_contains_structural_concepts_only() {
    let hierarchy = HierarchicalConcept::new(vec![concept(1), concept(2)]).unwrap();

    assert_eq!(hierarchy.children().len(), 2);
}
