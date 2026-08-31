use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposition,
    RecursiveWorldRevisionAbstractionCompositionPathSet,
    RecursiveWorldRevisionAbstractionCompositionPathSupport,
    RecursiveWorldRevisionAbstractionCompositionPathSupportDeriver,
    RecursiveWorldRevisionAbstractionCompositionPathSupportSet,
    RecursiveWorldRevisionAbstractionCompositionThreshold,
    RecursiveWorldRevisionAbstractionCompositionWitness,
    RecursiveWorldRevisionAbstractionCompositionWitnessSet,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(
        StructuralConcept::with_sequence_length(
            vec![PrimitiveSignature::new(RelationKind::Equal, span)],
            8,
        ),
    ))
}

fn class(members: &[usize]) -> RecursiveWorldRevisionAbstractionClass {
    RecursiveWorldRevisionAbstractionClass::new(members.iter().copied().map(unit).collect())
        .unwrap()
}

fn observation(
    premises: &[usize],
    conclusions: &[usize],
) -> RecursiveWorldRevisionDiscoveryObservation {
    RecursiveWorldRevisionDiscoveryObservation::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn witness(
    from_members: &[usize],
    to_members: &[usize],
    premise_member: usize,
    conclusion_member: usize,
    noise: usize,
) -> RecursiveWorldRevisionAbstractionCompositionWitness {
    RecursiveWorldRevisionAbstractionCompositionWitness::new(
        class(from_members),
        class(to_members),
        observation(
            &[premise_member, 1000 + noise],
            &[conclusion_member, 2000 + noise],
        ),
    )
    .unwrap()
}

fn threshold() -> RecursiveWorldRevisionAbstractionCompositionThreshold {
    RecursiveWorldRevisionAbstractionCompositionThreshold::new(2).unwrap()
}

fn composition_with_supports(
    relations: Vec<(Vec<usize>, Vec<usize>, usize, usize)>,
) -> RecursiveWorldRevisionAbstractionComposition {
    let mut witnesses = Vec::new();

    for (from, to, support, seed) in relations {
        for index in 0..support {
            witnesses.push(witness(
                &from,
                &to,
                from[index % from.len()],
                to[index % to.len()],
                seed * 100 + index,
            ));
        }
    }

    RecursiveWorldRevisionAbstractionComposition::compose(
        RecursiveWorldRevisionAbstractionCompositionWitnessSet::new(witnesses).unwrap(),
        threshold(),
    )
    .unwrap()
}

fn path_set(
    relations: Vec<(Vec<usize>, Vec<usize>, usize, usize)>,
) -> RecursiveWorldRevisionAbstractionCompositionPathSet {
    RecursiveWorldRevisionAbstractionCompositionPathSet::induce(composition_with_supports(
        relations,
    ))
    .unwrap()
}

fn two_edge_paths(
    first_support: usize,
    second_support: usize,
) -> RecursiveWorldRevisionAbstractionCompositionPathSet {
    path_set(vec![
        (vec![1, 2], vec![10, 20], first_support, 1),
        (vec![10, 20], vec![100, 200], second_support, 2),
    ])
}

#[test]
fn path_support_preserves_every_edge_support() {
    let paths = two_edge_paths(3, 4);

    let support =
        RecursiveWorldRevisionAbstractionCompositionPathSupport::derive(paths.paths()[0].clone());

    assert_eq!(support.edge_supports()[0].support_count(), 3);

    assert_eq!(support.edge_supports()[1].support_count(), 4);
}

#[test]
fn path_support_is_minimum_edge_support() {
    let paths = two_edge_paths(3, 5);

    let support =
        RecursiveWorldRevisionAbstractionCompositionPathSupport::derive(paths.paths()[0].clone());

    assert_eq!(support.minimum_support(), 3);
}

#[test]
fn stronger_edge_cannot_hide_weaker_edge() {
    let paths = two_edge_paths(2, 9);

    let support =
        RecursiveWorldRevisionAbstractionCompositionPathSupport::derive(paths.paths()[0].clone());

    assert_eq!(support.minimum_support(), 2);

    assert_ne!(support.minimum_support(), 9);
}

#[test]
fn path_support_does_not_average_edge_supports() {
    let paths = two_edge_paths(2, 6);

    let support =
        RecursiveWorldRevisionAbstractionCompositionPathSupport::derive(paths.paths()[0].clone());

    assert_eq!(support.minimum_support(), 2);

    assert_ne!(support.minimum_support(), 4);
}

#[test]
fn path_support_preserves_exact_observation_provenance_per_edge() {
    let paths = two_edge_paths(2, 3);

    let path = paths.paths()[0].clone();

    let support = RecursiveWorldRevisionAbstractionCompositionPathSupport::derive(path.clone());

    for edge in path.edges() {
        let edge_support = support.support_for_edge(edge).unwrap();

        assert_eq!(
            edge_support.supporting_observations(),
            edge.supporting_observations()
        );

        assert_eq!(edge_support.support_count(), edge.support_count());
    }
}

#[test]
fn path_support_preserves_exact_path_identity() {
    let paths = two_edge_paths(2, 3);

    let path = paths.paths()[0].clone();

    let support = RecursiveWorldRevisionAbstractionCompositionPathSupport::derive(path.clone());

    assert_eq!(support.path(), &path);

    assert_eq!(support.edge_count(), path.edge_count());
}

#[test]
fn longer_path_uses_global_bottleneck() {
    let paths = path_set(vec![
        (vec![1, 2], vec![10, 20], 5, 1),
        (vec![10, 20], vec![100, 200], 3, 2),
        (vec![100, 200], vec![1000, 2000], 7, 3),
    ]);

    let full_path = paths
        .paths()
        .iter()
        .find(|path| path.edge_count() == 3)
        .unwrap()
        .clone();

    let support = RecursiveWorldRevisionAbstractionCompositionPathSupport::derive(full_path);

    assert_eq!(support.minimum_support(), 3);
}

#[test]
fn path_support_set_contains_one_support_per_induced_path() {
    let paths = path_set(vec![
        (vec![1, 2], vec![10, 20], 2, 1),
        (vec![10, 20], vec![100, 200], 2, 2),
        (vec![100, 200], vec![1000, 2000], 2, 3),
    ]);

    let path_count = paths.len();

    let supports = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths);

    assert_eq!(supports.len(), path_count);

    assert!(!supports.is_empty());
}

#[test]
fn path_support_lookup_preserves_endpoint_identity() {
    let paths = two_edge_paths(2, 3);

    let supports = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths);

    let matches = supports.supports_from_to(&class(&[1, 2]), &class(&[100, 200]));

    assert_eq!(matches.len(), 1);

    assert_eq!(matches[0].start(), &class(&[1, 2],));

    assert_eq!(matches[0].end(), &class(&[100, 200],));
}

#[test]
fn path_support_set_preserves_source_path_set_identity() {
    let paths = two_edge_paths(2, 3);

    let before = paths.clone();

    let supports = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths);

    assert_eq!(supports.source(), &before);

    for path in before.paths() {
        assert!(supports.support_for_path(path,).is_some());
    }
}

#[test]
fn path_support_deriver_facade_matches_direct_derivation() {
    let paths = two_edge_paths(3, 4);

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionPathSupportDeriver::derive(paths.clone(),),
        RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths,)
    );
}

#[test]
fn path_support_is_canonical_deterministic_and_non_mutating() {
    let left_paths = path_set(vec![
        (vec![1, 2], vec![10, 20], 4, 1),
        (vec![10, 20], vec![100, 200], 3, 2),
    ]);

    let right_paths = path_set(vec![
        (vec![10, 20], vec![100, 200], 3, 2),
        (vec![1, 2], vec![10, 20], 4, 1),
    ]);

    let before = left_paths.clone();

    let left =
        RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(left_paths.clone());

    let right = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(right_paths);

    assert_eq!(left, right);

    assert_eq!(left_paths, before);

    assert_eq!(left.supports()[0].minimum_support(), 3);
}
