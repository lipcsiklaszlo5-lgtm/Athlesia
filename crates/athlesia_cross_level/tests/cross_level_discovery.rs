use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{
    AbstractionUnit, CrossLevelDiscovery, CrossLevelMemory, CrossLevelObservation,
};

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
fn observation_canonicalizes_duplicate_units() {
    let first = structural_unit(1);

    let observation =
        CrossLevelObservation::new(vec![first.clone(), first, hierarchical_unit(&[2, 3])]);

    assert_eq!(observation.len(), 2);
}

#[test]
fn observation_order_is_canonical() {
    let first = CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]);

    let second = CrossLevelObservation::new(vec![hierarchical_unit(&[2, 3]), structural_unit(1)]);

    assert_eq!(first, second);
}

#[test]
fn empty_observation_is_valid() {
    let observation = CrossLevelObservation::new(Vec::new());

    assert!(observation.is_empty());
}

#[test]
fn one_observation_is_not_enough_by_default() {
    let observations = vec![CrossLevelObservation::new(vec![
        structural_unit(1),
        hierarchical_unit(&[2, 3]),
    ])];

    let discovered = CrossLevelDiscovery::default().discover(&observations);

    assert!(discovered.is_empty());
}

#[test]
fn repeated_mixed_cooccurrence_discovers_concept() {
    let observations = vec![
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]),
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]),
    ];

    let discovered = CrossLevelDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 1);

    assert_eq!(discovered[0].support(), 2);
}

#[test]
fn structural_only_cooccurrence_is_not_cross_level() {
    let observations = vec![
        CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]),
        CrossLevelObservation::new(vec![structural_unit(1), structural_unit(2)]),
    ];

    let discovered = CrossLevelDiscovery::default().discover(&observations);

    assert!(discovered.is_empty());
}

#[test]
fn hierarchical_only_cooccurrence_is_not_cross_level() {
    let observations = vec![
        CrossLevelObservation::new(vec![hierarchical_unit(&[1, 2]), hierarchical_unit(&[2, 3])]),
        CrossLevelObservation::new(vec![hierarchical_unit(&[1, 2]), hierarchical_unit(&[2, 3])]),
    ];

    let discovered = CrossLevelDiscovery::default().discover(&observations);

    assert!(discovered.is_empty());
}

#[test]
fn support_counts_distinct_observations() {
    let structural = structural_unit(1);

    let hierarchical = hierarchical_unit(&[2, 3]);

    let observations = vec![
        CrossLevelObservation::new(vec![
            structural.clone(),
            structural.clone(),
            hierarchical.clone(),
            hierarchical.clone(),
        ]),
        CrossLevelObservation::new(vec![structural, hierarchical]),
    ];

    let discovered = CrossLevelDiscovery::default().discover(&observations);

    assert_eq!(discovered[0].support(), 2);
}

#[test]
fn minimum_support_is_explicit() {
    let discovery = CrossLevelDiscovery::new(3);

    assert_eq!(discovery.minimum_support(), 3);
}

#[test]
fn stricter_support_threshold_prunes_candidate() {
    let observations = vec![
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]),
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]),
    ];

    let discovered = CrossLevelDiscovery::new(3).discover(&observations);

    assert!(discovered.is_empty());
}

#[test]
fn multiple_mixed_pairs_can_be_discovered() {
    let observations = vec![
        CrossLevelObservation::new(vec![
            structural_unit(1),
            structural_unit(2),
            hierarchical_unit(&[3, 4]),
        ]),
        CrossLevelObservation::new(vec![
            structural_unit(1),
            structural_unit(2),
            hierarchical_unit(&[3, 4]),
        ]),
    ];

    let discovered = CrossLevelDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 2);

    assert!(discovered
        .iter()
        .all(|candidate| { candidate.support() == 2 }));
}

#[test]
fn discovery_is_order_independent() {
    let first = vec![
        CrossLevelObservation::new(vec![
            structural_unit(1),
            structural_unit(2),
            hierarchical_unit(&[3, 4]),
        ]),
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[3, 4])]),
    ];

    let second = vec![
        CrossLevelObservation::new(vec![hierarchical_unit(&[3, 4]), structural_unit(1)]),
        CrossLevelObservation::new(vec![
            hierarchical_unit(&[3, 4]),
            structural_unit(2),
            structural_unit(1),
        ]),
    ];

    assert_eq!(
        CrossLevelDiscovery::default().discover(&first),
        CrossLevelDiscovery::default().discover(&second)
    );
}

#[test]
fn discovery_is_deterministic() {
    let observations = vec![
        CrossLevelObservation::new(vec![
            structural_unit(1),
            structural_unit(2),
            hierarchical_unit(&[3, 4]),
        ]),
        CrossLevelObservation::new(vec![
            structural_unit(1),
            structural_unit(2),
            hierarchical_unit(&[3, 4]),
        ]),
    ];

    let discovery = CrossLevelDiscovery::default();

    assert_eq!(
        discovery.discover(&observations,),
        discovery.discover(&observations,)
    );
}

#[test]
fn structural_extent_remains_part_of_discovery_identity() {
    let short = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        4,
    );

    let long = StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, 1)],
        6,
    );

    let hierarchical = hierarchical_unit(&[2, 3]);

    let observations = vec![
        CrossLevelObservation::new(vec![
            AbstractionUnit::Structural(short.clone()),
            hierarchical.clone(),
        ]),
        CrossLevelObservation::new(vec![
            AbstractionUnit::Structural(short),
            hierarchical.clone(),
        ]),
        CrossLevelObservation::new(vec![
            AbstractionUnit::Structural(long.clone()),
            hierarchical.clone(),
        ]),
        CrossLevelObservation::new(vec![AbstractionUnit::Structural(long), hierarchical]),
    ];

    let discovered = CrossLevelDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 2);
}

#[test]
fn consolidation_populates_cross_level_memory() {
    let observations = vec![
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]),
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]),
    ];

    let mut memory = CrossLevelMemory::new();

    let candidates = CrossLevelDiscovery::default().consolidate(&observations, &mut memory);

    assert_eq!(candidates.len(), 1);

    assert_eq!(memory.len(), 1);

    assert!(memory.contains(candidates[0].concept()));
}

#[test]
fn repeated_consolidation_does_not_grow_memory() {
    let observations = vec![
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]),
        CrossLevelObservation::new(vec![structural_unit(1), hierarchical_unit(&[2, 3])]),
    ];

    let discovery = CrossLevelDiscovery::default();

    let mut memory = CrossLevelMemory::new();

    discovery.consolidate(&observations, &mut memory);

    discovery.consolidate(&observations, &mut memory);

    assert_eq!(memory.len(), 1);
}
