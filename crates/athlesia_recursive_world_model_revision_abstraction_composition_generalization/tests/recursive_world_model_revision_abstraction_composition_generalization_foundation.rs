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
    RecursiveWorldRevisionAbstractionCompositionGeneralizationSource,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold,
    RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet,
    RecursiveWorldRevisionAbstractionCompositionGeneralizer,
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
    relations: Vec<(Vec<usize>, Vec<usize>, usize)>,
) -> RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
    let mut witnesses = Vec::new();

    for (from, to, seed) in relations {
        for index in 0..2 {
            witnesses.push(witness(
                &from,
                &to,
                from[index % from.len()],
                to[index % to.len()],
                seed * 100 + index,
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

fn shared_context(
    unique_base: usize,
) -> RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
    context(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
        (
            vec![unique_base, unique_base + 1],
            vec![unique_base + 10, unique_base + 11],
            unique_base,
        ),
        (
            vec![unique_base + 10, unique_base + 11],
            vec![unique_base + 20, unique_base + 21],
            unique_base + 1,
        ),
    ])
}

fn motif_classes() -> Vec<RecursiveWorldRevisionAbstractionClass> {
    vec![class(&[1, 2]), class(&[10, 20]), class(&[100, 200])]
}

fn threshold(value: usize) -> RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold {
    RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold::new(value).unwrap()
}

#[test]
fn generalization_threshold_rejects_values_below_two() {
    assert!(RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold::new(0,).is_none());

    assert!(RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold::new(1,).is_none());

    assert_eq!(threshold(2,).min_context_support(), 2);
}

#[test]
fn generalization_source_requires_two_distinct_contexts() {
    let first = shared_context(300);

    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::
            new(
                vec![
                    first.clone(),
                ],
            )
            .is_none()
    );

    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
            first.clone(),
            first,
        ],)
        .is_none()
    );
}

#[test]
fn generalization_motif_requires_exactly_three_classes() {
    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(vec![
            class(&[1, 2],),
            class(&[10, 20],),
        ],)
        .is_none()
    );

    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(vec![
            class(&[1, 2],),
            class(&[10, 20],),
            class(&[100, 200],),
            class(&[1000, 2000],),
        ],)
        .is_none()
    );
}

#[test]
fn generalization_motif_rejects_repeated_class_identity() {
    let repeated = class(&[1, 2]);

    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(vec![
            repeated.clone(),
            class(&[10, 20],),
            repeated,
        ],)
        .is_none()
    );
}

#[test]
fn repeated_exact_motif_generalizes_across_two_contexts() {
    let source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        shared_context(300),
        shared_context(500),
    ])
    .unwrap();

    let generalized = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        source,
        threshold(2),
    )
    .unwrap();

    let motif = generalized.motif(&motif_classes()).unwrap();

    assert_eq!(motif.support_count(), 2);

    assert_eq!(motif.motif().edge_count(), 2);
}

#[test]
fn motif_seen_in_only_one_context_does_not_generalize() {
    let first = context(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
    ]);

    let second = context(vec![
        (vec![300, 301], vec![310, 311], 3),
        (vec![310, 311], vec![320, 321], 4),
    ]);

    let source =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![first, second])
            .unwrap();

    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
            source,
            threshold(2,),
        )
        .is_none()
    );
}

#[test]
fn repeated_occurrence_inside_one_context_counts_once() {
    let long = context(vec![
        (vec![1, 2], vec![10, 20], 1),
        (vec![10, 20], vec![100, 200], 2),
        (vec![100, 200], vec![1000, 2000], 3),
    ]);

    let other = shared_context(500);

    let source =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![long, other])
            .unwrap();

    let generalized = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        source,
        threshold(2),
    )
    .unwrap();

    assert_eq!(
        generalized
            .motif(&motif_classes(),)
            .unwrap()
            .support_count(),
        2
    );
}

#[test]
fn threshold_three_requires_three_distinct_contexts() {
    let two_contexts = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        shared_context(300),
        shared_context(500),
    ])
    .unwrap();

    assert!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
            two_contexts,
            threshold(3,),
        )
        .is_none()
    );

    let three_contexts =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
            shared_context(300),
            shared_context(500),
            shared_context(700),
        ])
        .unwrap();

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
            three_contexts,
            threshold(3,),
        )
        .unwrap()
        .motif(&motif_classes(),)
        .unwrap()
        .support_count(),
        3
    );
}

#[test]
fn motif_order_and_direction_are_exact() {
    let forward =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(motif_classes())
            .unwrap();

    let reverse = RecursiveWorldRevisionAbstractionCompositionGeneralizationMotif::new(vec![
        class(&[100, 200]),
        class(&[10, 20]),
        class(&[1, 2]),
    ])
    .unwrap();

    assert_ne!(forward, reverse);

    assert_eq!(forward.start(), &class(&[1, 2],));

    assert_eq!(forward.middle(), &class(&[10, 20],));

    assert_eq!(forward.end(), &class(&[100, 200],));
}

#[test]
fn generalized_motif_preserves_exact_context_provenance() {
    let first = shared_context(300);

    let second = shared_context(500);

    let source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        second.clone(),
        first.clone(),
    ])
    .unwrap();

    let source_before = source.clone();

    let generalized = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        source,
        threshold(2),
    )
    .unwrap();

    let motif = generalized.motif(&motif_classes()).unwrap();

    assert_eq!(motif.supporting_contexts().len(), 2);

    for context in motif.supporting_contexts() {
        assert!(source_before.contexts().contains(context,));
    }
}

#[test]
fn generalizer_facade_matches_direct_generalization() {
    let source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        shared_context(300),
        shared_context(500),
    ])
    .unwrap();

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizer::generalize(
            source.clone(),
            threshold(2,),
        ),
        RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
            source,
            threshold(2,),
        )
    );
}

#[test]
fn composition_generalization_is_canonical_deterministic_and_non_mutating() {
    let first = shared_context(300);

    let second = shared_context(500);

    let left_source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        first.clone(),
        second.clone(),
    ])
    .unwrap();

    let right_source =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![second, first])
            .unwrap();

    let before = left_source.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        left_source.clone(),
        threshold(2),
    );

    let right = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        right_source,
        threshold(2),
    );

    assert_eq!(left, right);

    assert_eq!(left_source, before);

    let generalized = left.unwrap();

    assert!(!generalized.is_empty());

    assert_eq!(
        generalized
            .motif(&motif_classes(),)
            .unwrap()
            .support_count(),
        2
    );
}
