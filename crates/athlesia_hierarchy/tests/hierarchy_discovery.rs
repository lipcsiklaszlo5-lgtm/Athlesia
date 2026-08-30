use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_hierarchy::{HierarchicalMemory, HierarchyDiscovery, HierarchyObservation};

fn concept(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        6,
    )
}

#[test]
fn observation_canonicalizes_duplicate_concepts() {
    let first = concept(1);

    let observation = HierarchyObservation::new(vec![first.clone(), first, concept(2)]);

    assert_eq!(observation.len(), 2);
}

#[test]
fn observation_order_is_canonical() {
    let first = HierarchyObservation::new(vec![concept(1), concept(2)]);

    let second = HierarchyObservation::new(vec![concept(2), concept(1)]);

    assert_eq!(first, second);
}

#[test]
fn empty_observation_is_valid() {
    let observation = HierarchyObservation::new(Vec::new());

    assert!(observation.is_empty());
}

#[test]
fn one_observation_is_not_enough_by_default() {
    let observations = vec![HierarchyObservation::new(vec![concept(1), concept(2)])];

    let discovered = HierarchyDiscovery::default().discover(&observations);

    assert!(discovered.is_empty());
}

#[test]
fn repeated_cooccurrence_discovers_hierarchy() {
    let observations = vec![
        HierarchyObservation::new(vec![concept(1), concept(2)]),
        HierarchyObservation::new(vec![concept(1), concept(2)]),
    ];

    let discovered = HierarchyDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 1);

    assert_eq!(discovered[0].support(), 2);
}

#[test]
fn unrelated_concept_does_not_enter_pair() {
    let observations = vec![
        HierarchyObservation::new(vec![concept(1), concept(2)]),
        HierarchyObservation::new(vec![concept(1), concept(2)]),
        HierarchyObservation::new(vec![concept(3)]),
    ];

    let discovered = HierarchyDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 1);

    assert!(!discovered[0].concept().contains(&concept(3)));
}

#[test]
fn support_counts_distinct_observations() {
    let observations = vec![
        HierarchyObservation::new(vec![concept(1), concept(1), concept(2)]),
        HierarchyObservation::new(vec![concept(1), concept(2)]),
    ];

    let discovered = HierarchyDiscovery::default().discover(&observations);

    assert_eq!(discovered[0].support(), 2);
}

#[test]
fn minimum_support_is_explicit() {
    let discovery = HierarchyDiscovery::new(3);

    assert_eq!(discovery.minimum_support(), 3);
}

#[test]
fn stricter_support_threshold_prunes_candidate() {
    let observations = vec![
        HierarchyObservation::new(vec![concept(1), concept(2)]),
        HierarchyObservation::new(vec![concept(1), concept(2)]),
    ];

    let discovered = HierarchyDiscovery::new(3).discover(&observations);

    assert!(discovered.is_empty());
}

#[test]
fn multiple_repeated_pairs_can_be_discovered() {
    let observations = vec![
        HierarchyObservation::new(vec![concept(1), concept(2), concept(3)]),
        HierarchyObservation::new(vec![concept(1), concept(2), concept(3)]),
    ];

    let discovered = HierarchyDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 3);

    assert!(discovered
        .iter()
        .all(|candidate| { candidate.support() == 2 }));
}

#[test]
fn discovery_is_order_independent() {
    let first = vec![
        HierarchyObservation::new(vec![concept(1), concept(2), concept(3)]),
        HierarchyObservation::new(vec![concept(1), concept(2)]),
    ];

    let second = vec![
        HierarchyObservation::new(vec![concept(2), concept(1)]),
        HierarchyObservation::new(vec![concept(3), concept(2), concept(1)]),
    ];

    assert_eq!(
        HierarchyDiscovery::default().discover(&first),
        HierarchyDiscovery::default().discover(&second)
    );
}

#[test]
fn discovery_is_deterministic() {
    let observations = vec![
        HierarchyObservation::new(vec![concept(1), concept(2), concept(3)]),
        HierarchyObservation::new(vec![concept(1), concept(2), concept(3)]),
    ];

    let discovery = HierarchyDiscovery::default();

    assert_eq!(
        discovery.discover(&observations),
        discovery.discover(&observations)
    );
}

#[test]
fn consolidation_populates_hierarchical_memory() {
    let observations = vec![
        HierarchyObservation::new(vec![concept(1), concept(2)]),
        HierarchyObservation::new(vec![concept(1), concept(2)]),
    ];

    let mut memory = HierarchicalMemory::new();

    let candidates = HierarchyDiscovery::default().consolidate(&observations, &mut memory);

    assert_eq!(candidates.len(), 1);
    assert_eq!(memory.len(), 1);

    assert!(memory.contains(candidates[0].concept()));
}

#[test]
fn repeated_consolidation_does_not_grow_memory() {
    let observations = vec![
        HierarchyObservation::new(vec![concept(1), concept(2)]),
        HierarchyObservation::new(vec![concept(1), concept(2)]),
    ];

    let mut memory = HierarchicalMemory::new();

    let discovery = HierarchyDiscovery::default();

    discovery.consolidate(&observations, &mut memory);

    discovery.consolidate(&observations, &mut memory);

    assert_eq!(memory.len(), 1);
}

#[test]
fn child_extent_remains_part_of_discovery_identity() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        6,
    );

    let other = concept(2);

    let observations = vec![
        HierarchyObservation::new(vec![short.clone(), other.clone()]),
        HierarchyObservation::new(vec![short, other.clone()]),
        HierarchyObservation::new(vec![long.clone(), other.clone()]),
        HierarchyObservation::new(vec![long, other]),
    ];

    let discovered = HierarchyDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 2);
}
