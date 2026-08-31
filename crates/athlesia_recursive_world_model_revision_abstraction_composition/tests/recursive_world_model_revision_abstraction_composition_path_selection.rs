use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposition,
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    RecursiveWorldRevisionAbstractionCompositionPathSelector,
    RecursiveWorldRevisionAbstractionCompositionPathSet,
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

fn composition(
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
        RecursiveWorldRevisionAbstractionCompositionThreshold::new(2).unwrap(),
    )
    .unwrap()
}

fn supports(
    relations: Vec<(Vec<usize>, Vec<usize>, usize, usize)>,
) -> RecursiveWorldRevisionAbstractionCompositionPathSupportSet {
    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(composition(relations))
        .unwrap();

    RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths)
}

fn two_route_supports(
    left_first: usize,
    left_second: usize,
    right_first: usize,
    right_second: usize,
) -> RecursiveWorldRevisionAbstractionCompositionPathSupportSet {
    supports(vec![
        (vec![1, 2], vec![10, 20], left_first, 1),
        (vec![10, 20], vec![100, 200], left_second, 2),
        (vec![1, 2], vec![30, 40], right_first, 3),
        (vec![30, 40], vec![100, 200], right_second, 4),
    ])
}

#[test]
fn higher_minimum_support_wins_same_endpoint_pair() {
    let selections = RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(
        two_route_supports(5, 5, 3, 7),
    );

    let selected = selections
        .selection_for(&class(&[1, 2]), &class(&[100, 200]))
        .unwrap();

    assert_eq!(selected.minimum_support(), 5);

    assert_eq!(selected.path().classes()[1], class(&[10, 20],));
}

#[test]
fn weaker_path_cannot_win_via_strong_single_edge() {
    let selections = RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(
        two_route_supports(4, 4, 9, 2),
    );

    let selected = selections
        .selection_for(&class(&[1, 2]), &class(&[100, 200]))
        .unwrap();

    assert_eq!(selected.minimum_support(), 4);

    assert_eq!(selected.path().classes()[1], class(&[10, 20],));
}

#[test]
fn shorter_path_wins_when_minimum_support_ties() {
    let selections =
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(supports(vec![
            (vec![1, 2], vec![10, 20], 4, 1),
            (vec![10, 20], vec![100, 200], 4, 2),
            (vec![1, 2], vec![30, 40], 4, 3),
            (vec![30, 40], vec![50, 60], 4, 4),
            (vec![50, 60], vec![100, 200], 4, 5),
        ]));

    let selected = selections
        .selection_for(&class(&[1, 2]), &class(&[100, 200]))
        .unwrap();

    assert_eq!(selected.minimum_support(), 4);

    assert_eq!(selected.edge_count(), 2);

    assert_eq!(selected.path().classes()[1], class(&[10, 20],));
}

#[test]
fn stronger_longer_path_beats_weaker_shorter_path() {
    let selections =
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(supports(vec![
            (vec![1, 2], vec![10, 20], 3, 1),
            (vec![10, 20], vec![100, 200], 3, 2),
            (vec![1, 2], vec![30, 40], 5, 3),
            (vec![30, 40], vec![50, 60], 5, 4),
            (vec![50, 60], vec![100, 200], 5, 5),
        ]));

    let selected = selections
        .selection_for(&class(&[1, 2]), &class(&[100, 200]))
        .unwrap();

    assert_eq!(selected.minimum_support(), 5);

    assert_eq!(selected.edge_count(), 3);
}

#[test]
fn stable_path_identity_breaks_exact_rank_tie() {
    let selections = RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(
        two_route_supports(4, 4, 4, 4),
    );

    let selected = selections
        .selection_for(&class(&[1, 2]), &class(&[100, 200]))
        .unwrap();

    assert_eq!(selected.minimum_support(), 4);

    assert_eq!(selected.edge_count(), 2);

    assert_eq!(selected.path().classes()[1], class(&[10, 20],));
}

#[test]
fn different_endpoint_pairs_are_selected_independently() {
    let selections =
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(supports(vec![
            (vec![1, 2], vec![10, 20], 3, 1),
            (vec![10, 20], vec![100, 200], 3, 2),
            (vec![300, 400], vec![500, 600], 5, 3),
            (vec![500, 600], vec![700, 800], 5, 4),
        ]));

    assert!(selections
        .selection_for(&class(&[1, 2],), &class(&[100, 200],),)
        .is_some());

    assert!(selections
        .selection_for(&class(&[300, 400],), &class(&[700, 800],),)
        .is_some());
}

#[test]
fn single_candidate_endpoint_pair_is_preserved() {
    let support_set = supports(vec![
        (vec![1, 2], vec![10, 20], 3, 1),
        (vec![10, 20], vec![100, 200], 4, 2),
    ]);

    let candidate = support_set.supports_from_to(&class(&[1, 2]), &class(&[100, 200]))[0].clone();

    let selections =
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(support_set);

    assert_eq!(
        selections
            .selection_for(&class(&[1, 2],), &class(&[100, 200],),)
            .unwrap()
            .selected(),
        &candidate
    );
}

#[test]
fn selection_preserves_selected_path_support_identity() {
    let support_set = two_route_supports(5, 5, 3, 3);

    let expected = support_set
        .supports_from_to(&class(&[1, 2]), &class(&[100, 200]))
        .into_iter()
        .find(|support| support.minimum_support() == 5)
        .unwrap()
        .clone();

    let selections =
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(support_set);

    assert_eq!(
        selections
            .selection_for(&class(&[1, 2],), &class(&[100, 200],),)
            .unwrap()
            .selected(),
        &expected
    );
}

#[test]
fn selector_does_not_materialize_transitive_edge() {
    let support_set = supports(vec![
        (vec![1, 2], vec![10, 20], 3, 1),
        (vec![10, 20], vec![100, 200], 3, 2),
    ]);

    let selections =
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(support_set);

    let source_composition = selections.source().source().source();

    assert!(source_composition
        .edge(&class(&[1, 2],), &class(&[100, 200],),)
        .is_none());
}

#[test]
fn selection_set_preserves_source_support_set_identity() {
    let support_set = two_route_supports(5, 5, 3, 3);

    let before = support_set.clone();

    let selections =
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(support_set);

    assert_eq!(selections.source(), &before);

    assert!(!selections.is_empty());
}

#[test]
fn path_selector_facade_matches_direct_selection() {
    let support_set = two_route_supports(5, 5, 3, 3);

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionPathSelector::select(support_set.clone(),),
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(support_set,)
    );
}

#[test]
fn path_selection_is_canonical_deterministic_and_non_mutating() {
    let left = two_route_supports(4, 4, 4, 4);

    let right = supports(vec![
        (vec![30, 40], vec![100, 200], 4, 4),
        (vec![1, 2], vec![30, 40], 4, 3),
        (vec![10, 20], vec![100, 200], 4, 2),
        (vec![1, 2], vec![10, 20], 4, 1),
    ]);

    let before = left.clone();

    let left_selection =
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(left.clone());

    let right_selection =
        RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(right);

    assert_eq!(left_selection, right_selection);

    assert_eq!(left, before);

    let selected = left_selection
        .selection_for(&class(&[1, 2]), &class(&[100, 200]))
        .unwrap();

    assert_eq!(selected.path().classes()[1], class(&[10, 20],));
}
