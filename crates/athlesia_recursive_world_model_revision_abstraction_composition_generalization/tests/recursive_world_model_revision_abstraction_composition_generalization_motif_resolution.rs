use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposition,
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    RecursiveWorldRevisionAbstractionCompositionPathSet,
    RecursiveWorldRevisionAbstractionCompositionPathSupportSet,
    RecursiveWorldRevisionAbstractionCompositionThreshold,
    RecursiveWorldRevisionAbstractionCompositionWitness,
    RecursiveWorldRevisionAbstractionCompositionWitnessSet,
};

use athlesia_recursive_world_model_revision_abstraction_composition_generalization::{
    RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationResolver,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationSource,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold,
    RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet,
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

fn context(
    relations: Vec<(Vec<usize>, Vec<usize>)>,
    context_seed: usize,
) -> RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
    let mut witnesses = Vec::new();

    for (edge_index, (from, to)) in relations.into_iter().enumerate() {
        for support_index in 0..2 {
            witnesses.push(witness(
                &from,
                &to,
                from[support_index % from.len()],
                to[support_index % to.len()],
                context_seed * 10000 + edge_index * 100 + support_index,
            ));
        }
    }

    let composition = RecursiveWorldRevisionAbstractionComposition::compose(
        RecursiveWorldRevisionAbstractionCompositionWitnessSet::new(witnesses).unwrap(),
        RecursiveWorldRevisionAbstractionCompositionThreshold::new(2).unwrap(),
    )
    .unwrap();

    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(composition).unwrap();

    let supports = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths);

    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(supports)
}

fn threshold() -> RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold {
    RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold::new(2).unwrap()
}

fn generalized(
    contexts: Vec<RecursiveWorldRevisionAbstractionCompositionPathSelectionSet>,
) -> RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet {
    let source =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(contexts).unwrap();

    RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(source, threshold())
        .unwrap()
}

fn abc() -> Vec<RecursiveWorldRevisionAbstractionClass> {
    vec![class(&[1, 2]), class(&[10, 20]), class(&[100, 200])]
}

fn abd() -> Vec<RecursiveWorldRevisionAbstractionClass> {
    vec![class(&[1, 2]), class(&[10, 20]), class(&[300, 400])]
}

fn dbc() -> Vec<RecursiveWorldRevisionAbstractionClass> {
    vec![class(&[300, 400]), class(&[10, 20]), class(&[100, 200])]
}

fn adc() -> Vec<RecursiveWorldRevisionAbstractionClass> {
    vec![class(&[1, 2]), class(&[30, 40]), class(&[100, 200])]
}

#[test]
fn identical_motif_identity_is_not_a_conflict() {
    let motif =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(abc()).unwrap();

    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict::between(
            motif.clone(),
            motif,
        )
        .is_none()
    );
}

#[test]
fn same_start_and_middle_with_different_end_conflicts() {
    let conflict =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict::between(
            RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(abc()).unwrap(),
            RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(abd()).unwrap(),
        )
        .unwrap();

    assert_eq!(conflict.shared_positions(), &[0, 1,]);

    assert!(conflict.shares_start_middle());
}

#[test]
fn same_middle_and_end_with_different_start_conflicts() {
    let conflict =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict::between(
            RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(abc()).unwrap(),
            RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(dbc()).unwrap(),
        )
        .unwrap();

    assert_eq!(conflict.shared_positions(), &[1, 2,]);

    assert!(conflict.shares_middle_end());
}

#[test]
fn same_start_and_end_with_different_middle_conflicts() {
    let conflict =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict::between(
            RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(abc()).unwrap(),
            RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(adc()).unwrap(),
        )
        .unwrap();

    assert_eq!(conflict.shared_positions(), &[0, 2,]);

    assert!(conflict.shares_start_end());
}

#[test]
fn shifted_chain_overlap_is_not_a_conflict() {
    let first = RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(vec![
        class(&[1, 2]),
        class(&[10, 20]),
        class(&[100, 200]),
    ])
    .unwrap();

    let second = RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(vec![
        class(&[10, 20]),
        class(&[100, 200]),
        class(&[1000, 2000]),
    ])
    .unwrap();

    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict::between(
            first, second,
        )
        .is_none()
    );
}

#[test]
fn single_class_overlap_is_not_a_conflict() {
    let first =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(abc()).unwrap();

    let second = RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(vec![
        class(&[100, 200]),
        class(&[500, 600]),
        class(&[700, 800]),
    ])
    .unwrap();

    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotifConflict::between(
            first, second,
        )
        .is_none()
    );
}

#[test]
fn prefix_conflict_excludes_both_motifs_without_winner() {
    let relations = vec![
        (vec![1, 2], vec![10, 20]),
        (vec![10, 20], vec![100, 200]),
        (vec![10, 20], vec![300, 400]),
    ];

    let resolution = RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(
        generalized(vec![context(relations.clone(), 1), context(relations, 2)]),
    );

    assert!(resolution.has_conflicts());

    assert!(resolution.conflicted_motif(&abc(),).is_some());

    assert!(resolution.conflicted_motif(&abd(),).is_some());

    assert!(resolution.resolved_motif(&abc(),).is_none());

    assert!(resolution.resolved_motif(&abd(),).is_none());
}

#[test]
fn consistent_shifted_chain_motifs_both_survive() {
    let relations = vec![
        (vec![1, 2], vec![10, 20]),
        (vec![10, 20], vec![100, 200]),
        (vec![100, 200], vec![1000, 2000]),
    ];

    let resolution = RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(
        generalized(vec![context(relations.clone(), 1), context(relations, 2)]),
    );

    assert!(!resolution.has_conflicts());

    assert!(resolution.resolved_motif(&abc(),).is_some());

    assert!(resolution
        .resolved_motif(&[
            class(&[10, 20],),
            class(&[100, 200],),
            class(&[1000, 2000],),
        ],)
        .is_some());
}

#[test]
fn disjoint_motif_survives_while_conflicting_pair_is_excluded() {
    let relations = vec![
        (vec![1, 2], vec![10, 20]),
        (vec![10, 20], vec![100, 200]),
        (vec![10, 20], vec![300, 400]),
        (vec![500, 600], vec![700, 800]),
        (vec![700, 800], vec![900, 901]),
    ];

    let resolution = RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(
        generalized(vec![context(relations.clone(), 1), context(relations, 2)]),
    );

    let disjoint = vec![class(&[500, 600]), class(&[700, 800]), class(&[900, 901])];

    assert!(resolution.conflicted_motif(&abc(),).is_some());

    assert!(resolution.conflicted_motif(&abd(),).is_some());

    assert!(resolution.resolved_motif(&disjoint,).is_some());
}

#[test]
fn endpoint_alternative_conflict_is_detected_across_distinct_contexts() {
    let first_route = vec![(vec![1, 2], vec![10, 20]), (vec![10, 20], vec![100, 200])];

    let second_route = vec![(vec![1, 2], vec![30, 40]), (vec![30, 40], vec![100, 200])];

    let resolution = RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(
        generalized(vec![
            context(first_route.clone(), 1),
            context(first_route, 2),
            context(second_route.clone(), 3),
            context(second_route, 4),
        ]),
    );

    assert!(resolution.conflicted_motif(&abc(),).is_some());

    assert!(resolution.conflicted_motif(&adc(),).is_some());

    assert_eq!(resolution.conflict_len(), 1);
}

#[test]
fn resolver_facade_matches_direct_resolution() {
    let relations = vec![(vec![1, 2], vec![10, 20]), (vec![10, 20], vec![100, 200])];

    let source = generalized(vec![context(relations.clone(), 1), context(relations, 2)]);

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationResolver::resolve(source.clone(),),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(source,)
    );
}

#[test]
fn motif_resolution_is_canonical_deterministic_and_non_mutating() {
    let relations = vec![
        (vec![1, 2], vec![10, 20]),
        (vec![10, 20], vec![100, 200]),
        (vec![10, 20], vec![300, 400]),
        (vec![500, 600], vec![700, 800]),
        (vec![700, 800], vec![900, 901]),
    ];

    let left_source = generalized(vec![
        context(relations.clone(), 1),
        context(relations.clone(), 2),
    ]);

    let right_source = generalized(vec![context(relations.clone(), 2), context(relations, 1)]);

    let before = left_source.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(
        left_source.clone(),
    );

    let right =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(right_source);

    assert_eq!(left, right);

    assert_eq!(left_source, before);

    assert_eq!(left.conflicted_len(), 2);

    assert_eq!(left.conflict_len(), 1);
}
