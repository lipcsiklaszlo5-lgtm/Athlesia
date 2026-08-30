use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{
    RecursiveConcept, RecursiveDiscovery, RecursiveMemory, RecursiveObservation, RecursiveUnit,
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

fn recursive_child() -> RecursiveConcept {
    RecursiveConcept::new(vec![base(1), cross(2, &[3, 4])]).unwrap()
}

#[test]
fn observation_canonicalizes_duplicates() {
    let unit = cross(1, &[2, 3]);

    let observation = RecursiveObservation::new(vec![base(4), unit.clone(), unit]);

    assert_eq!(observation.len(), 2);
}

#[test]
fn observation_order_is_canonical() {
    let first = RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]);

    let second = RecursiveObservation::new(vec![cross(2, &[3, 4]), base(1)]);

    assert_eq!(first, second);
}

#[test]
fn empty_observation_is_valid() {
    assert!(RecursiveObservation::new(Vec::new(),).is_empty());
}

#[test]
fn one_observation_is_not_enough_by_default() {
    let observations = vec![RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])])];

    assert!(RecursiveDiscovery::default()
        .discover(&observations,)
        .is_empty());
}

#[test]
fn repeated_base_cross_level_pair_is_discovered() {
    let observations = vec![
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
    ];

    let discovered = RecursiveDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 1);

    assert_eq!(discovered[0].support(), 2);
}

#[test]
fn repeated_cross_level_pair_is_discovered() {
    let observations = vec![
        RecursiveObservation::new(vec![cross(1, &[3, 4]), cross(2, &[3, 4])]),
        RecursiveObservation::new(vec![cross(1, &[3, 4]), cross(2, &[3, 4])]),
    ];

    let discovered = RecursiveDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 1);
}

#[test]
fn base_only_pair_is_not_discovered() {
    let observations = vec![
        RecursiveObservation::new(vec![base(1), base(2)]),
        RecursiveObservation::new(vec![base(1), base(2)]),
    ];

    assert!(RecursiveDiscovery::default()
        .discover(&observations,)
        .is_empty());
}

#[test]
fn repeated_recursive_child_pair_is_discovered() {
    let child = recursive_child();

    let observations = vec![
        RecursiveObservation::new(vec![
            base(5),
            RecursiveUnit::Recursive(Box::new(child.clone())),
        ]),
        RecursiveObservation::new(vec![base(5), RecursiveUnit::Recursive(Box::new(child))]),
    ];

    let discovered = RecursiveDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 1);

    assert_eq!(discovered[0].concept().depth(), 2);
}

#[test]
fn support_counts_canonical_observations_once() {
    let base_unit = base(1);

    let cross_unit = cross(2, &[3, 4]);

    let observations = vec![
        RecursiveObservation::new(vec![
            base_unit.clone(),
            base_unit.clone(),
            cross_unit.clone(),
            cross_unit.clone(),
        ]),
        RecursiveObservation::new(vec![base_unit, cross_unit]),
    ];

    let discovered = RecursiveDiscovery::default().discover(&observations);

    assert_eq!(discovered[0].support(), 2);
}

#[test]
fn minimum_support_is_explicit() {
    assert_eq!(RecursiveDiscovery::new(3).minimum_support(), 3);
}

#[test]
fn stricter_threshold_prunes_candidate() {
    let observations = vec![
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
    ];

    assert!(RecursiveDiscovery::new(3)
        .discover(&observations,)
        .is_empty());
}

#[test]
fn multiple_valid_pairs_are_discovered() {
    let observations = vec![
        RecursiveObservation::new(vec![base(1), base(2), cross(3, &[4, 5])]),
        RecursiveObservation::new(vec![base(1), base(2), cross(3, &[4, 5])]),
    ];

    let discovered = RecursiveDiscovery::default().discover(&observations);

    assert_eq!(discovered.len(), 2);

    assert!(discovered
        .iter()
        .all(|candidate| { candidate.support() == 2 }));
}

#[test]
fn discovery_is_order_independent() {
    let first = vec![
        RecursiveObservation::new(vec![base(1), base(2), cross(3, &[4, 5])]),
        RecursiveObservation::new(vec![base(1), cross(3, &[4, 5])]),
    ];

    let second = vec![
        RecursiveObservation::new(vec![cross(3, &[4, 5]), base(1)]),
        RecursiveObservation::new(vec![cross(3, &[4, 5]), base(2), base(1)]),
    ];

    assert_eq!(
        RecursiveDiscovery::default().discover(&first,),
        RecursiveDiscovery::default().discover(&second,)
    );
}

#[test]
fn discovery_is_deterministic() {
    let observations = vec![
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
    ];

    let discovery = RecursiveDiscovery::default();

    assert_eq!(
        discovery.discover(&observations,),
        discovery.discover(&observations,)
    );
}

#[test]
fn recursive_depth_remains_part_of_identity() {
    let child = recursive_child();

    let level_two = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(child.clone())),
    ])
    .unwrap();

    let observations = vec![
        RecursiveObservation::new(vec![
            base(6),
            RecursiveUnit::Recursive(Box::new(child.clone())),
            RecursiveUnit::Recursive(Box::new(level_two.clone())),
        ]),
        RecursiveObservation::new(vec![
            base(6),
            RecursiveUnit::Recursive(Box::new(child)),
            RecursiveUnit::Recursive(Box::new(level_two)),
        ]),
    ];

    let discovered = RecursiveDiscovery::default().discover(&observations);

    assert!(discovered
        .iter()
        .any(|candidate| { candidate.concept().depth() == 2 }));

    assert!(discovered
        .iter()
        .any(|candidate| { candidate.concept().depth() == 3 }));
}

#[test]
fn consolidation_populates_recursive_memory() {
    let observations = vec![
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
    ];

    let mut memory = RecursiveMemory::new();

    let candidates = RecursiveDiscovery::default().consolidate(&observations, &mut memory);

    assert_eq!(candidates.len(), 1);

    assert_eq!(memory.len(), 1);

    assert!(memory.contains(candidates[0].concept()));
}

#[test]
fn repeated_consolidation_does_not_grow_memory() {
    let observations = vec![
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
        RecursiveObservation::new(vec![base(1), cross(2, &[3, 4])]),
    ];

    let discovery = RecursiveDiscovery::default();

    let mut memory = RecursiveMemory::new();

    discovery.consolidate(&observations, &mut memory);

    discovery.consolidate(&observations, &mut memory);

    assert_eq!(memory.len(), 1);
}
