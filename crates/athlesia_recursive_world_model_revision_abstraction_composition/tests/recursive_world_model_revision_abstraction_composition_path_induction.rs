use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposition, RecursiveWorldRevisionAbstractionCompositionPath,
    RecursiveWorldRevisionAbstractionCompositionPathInducer,
    RecursiveWorldRevisionAbstractionCompositionPathSet,
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
    premises: &[usize],
    conclusions: &[usize],
) -> RecursiveWorldRevisionAbstractionCompositionWitness {
    RecursiveWorldRevisionAbstractionCompositionWitness::new(
        class(from_members),
        class(to_members),
        observation(premises, conclusions),
    )
    .unwrap()
}

fn threshold() -> RecursiveWorldRevisionAbstractionCompositionThreshold {
    RecursiveWorldRevisionAbstractionCompositionThreshold::new(2).unwrap()
}

fn composition(
    relations: Vec<(Vec<usize>, Vec<usize>, usize)>,
) -> RecursiveWorldRevisionAbstractionComposition {
    let mut witnesses = Vec::new();

    for (from, to, seed) in relations {
        witnesses.push(witness(
            &from,
            &to,
            &[from[0], 1000 + seed],
            &[to[0], 2000 + seed],
        ));

        witnesses.push(witness(
            &from,
            &to,
            &[from[1], 3000 + seed],
            &[to[1], 4000 + seed],
        ));
    }

    RecursiveWorldRevisionAbstractionComposition::compose(
        RecursiveWorldRevisionAbstractionCompositionWitnessSet::new(witnesses).unwrap(),
        threshold(),
    )
    .unwrap()
}

fn edge(
    source:
        &RecursiveWorldRevisionAbstractionComposition,
    from: &[usize],
    to: &[usize],
) -> athlesia_recursive_world_model_revision_abstraction_composition::
RecursiveWorldRevisionAbstractionCompositionEdge{
    source.edge(&class(from), &class(to)).unwrap().clone()
}

#[test]
fn composition_path_requires_at_least_two_edges() {
    let source = composition(vec![(vec![1, 2], vec![10, 20], 1)]);

    assert!(
        RecursiveWorldRevisionAbstractionCompositionPath::new(vec![edge(
            &source,
            &[1, 2],
            &[10, 20],
        ),],)
        .is_none()
    );
}

#[test]
fn composition_path_requires_exact_adjacency() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![30, 40], vec![50, 60], 2),
    ]);

    assert!(RecursiveWorldRevisionAbstractionCompositionPath::new(vec![
        edge(&source, &[1, 2], &[10, 20],),
        edge(&source, &[30, 40], &[50, 60],),
    ],)
    .is_none());
}

#[test]
fn composition_path_rejects_cycle() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![1, 2], 2),
    ]);

    assert!(RecursiveWorldRevisionAbstractionCompositionPath::new(vec![
        edge(&source, &[1, 2], &[10, 20],),
        edge(&source, &[10, 20], &[1, 2],),
    ],)
    .is_none());
}

#[test]
fn exact_two_edge_chain_induces_path() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
    ]);

    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(source).unwrap();

    assert_eq!(paths.len(), 1);

    assert_eq!(paths.paths()[0].edge_count(), 2);
}

#[test]
fn three_edge_chain_materializes_prefix_and_full_path() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
        (vec![100, 200], vec![1000, 2000], 3),
    ]);

    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(source).unwrap();

    assert_eq!(paths.len(), 3);

    assert!(paths.paths().iter().any(|path| {
        path.edge_count() == 3
            && path.start() == &class(&[1, 2])
            && path.end() == &class(&[1000, 2000])
    },));
}

#[test]
fn path_induction_does_not_create_transitive_edge() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
    ]);

    let paths =
        RecursiveWorldRevisionAbstractionCompositionPathSet::induce(source.clone()).unwrap();

    assert!(source
        .edge(&class(&[1, 2],), &class(&[100, 200],),)
        .is_none());

    assert_eq!(
        paths
            .paths_from_to(&class(&[1, 2],), &class(&[100, 200],),)
            .len(),
        1
    );
}

#[test]
fn disconnected_edges_do_not_induce_path() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![30, 40], vec![50, 60], 2),
    ]);

    assert!(RecursiveWorldRevisionAbstractionCompositionPathSet::induce(source,).is_none());
}

#[test]
fn branching_graph_induces_each_explicit_branch_path() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
        (vec![10, 20], vec![300, 400], 3),
    ]);

    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(source).unwrap();

    assert_eq!(paths.len(), 2);

    assert_eq!(
        paths
            .paths_from_to(&class(&[1, 2],), &class(&[100, 200],),)
            .len(),
        1
    );

    assert_eq!(
        paths
            .paths_from_to(&class(&[1, 2],), &class(&[300, 400],),)
            .len(),
        1
    );
}

#[test]
fn path_preserves_exact_edge_and_class_identity() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
    ]);

    let first = edge(&source, &[1, 2], &[10, 20]);

    let second = edge(&source, &[10, 20], &[100, 200]);

    let path =
        RecursiveWorldRevisionAbstractionCompositionPath::new(vec![first.clone(), second.clone()])
            .unwrap();

    assert_eq!(path.edges(), &[first, second,]);

    assert_eq!(
        path.classes(),
        &[class(&[1, 2],), class(&[10, 20],), class(&[100, 200],),]
    );

    assert_eq!(path.class_count(), 3);
}

#[test]
fn path_set_preserves_source_composition_identity() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
    ]);

    let before = source.clone();

    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(source).unwrap();

    assert_eq!(paths.source(), &before);
}

#[test]
fn path_inducer_facade_matches_direct_induction() {
    let source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
    ]);

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionPathInducer::induce(source.clone(),),
        RecursiveWorldRevisionAbstractionCompositionPathSet::induce(source,)
    );
}

#[test]
fn path_induction_is_canonical_deterministic_and_non_mutating() {
    let left_source = composition(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
        (vec![100, 200], vec![1000, 2000], 3),
    ]);

    let right_source = composition(vec![
        (vec![100, 200], vec![1000, 2000], 3),
        (vec![10, 20], vec![100, 200], 2),
        (vec![1, 2], vec![10, 20], 1),
    ]);

    let before = left_source.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(left_source.clone());

    let right = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(right_source);

    assert_eq!(left, right);

    assert_eq!(left_source, before);

    assert_eq!(left.unwrap().len(), 3);
}
