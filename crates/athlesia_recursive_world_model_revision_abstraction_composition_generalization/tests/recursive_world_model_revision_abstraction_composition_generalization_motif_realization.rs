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
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizer,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution,
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
    seed: usize,
) -> RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
    let mut witnesses = Vec::new();

    for (edge_index, (from, to)) in relations.into_iter().enumerate() {
        for support_index in 0..2 {
            witnesses.push(witness(
                &from,
                &to,
                from[support_index % from.len()],
                to[support_index % to.len()],
                seed * 10000 + edge_index * 100 + support_index,
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

fn relations() -> Vec<(Vec<usize>, Vec<usize>)> {
    vec![(vec![1, 2], vec![10, 20]), (vec![10, 20], vec![100, 200])]
}

fn projected() -> RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
    let source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        context(relations(), 1),
        context(relations(), 2),
    ])
    .unwrap();

    let generalized = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        source,
        RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    let resolution =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(generalized);

    let projection = RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        resolution,
        context(relations(), 10),
    );

    projection.projected_motifs().first().unwrap().clone()
}

#[test]
fn missing_start_witness_is_unavailable() {
    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected(),
            vec![observation(&[500], &[100])],
        );

    assert_eq!(
        realization.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::Unavailable
    );

    assert!(realization.realized_observation().is_none());
}

#[test]
fn missing_end_witness_is_unavailable() {
    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected(),
            vec![observation(&[1], &[500])],
        );

    assert_eq!(
        realization.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::Unavailable
    );

    assert!(realization.realized_observation().is_none());
}

#[test]
fn unique_endpoint_witnesses_realize_deterministically() {
    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected(),
            vec![observation(&[1], &[100])],
        );

    assert_eq!(
        realization.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::Deterministic
    );

    assert!(realization.is_deterministic());

    assert_eq!(
        realization.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );
}

#[test]
fn multiple_start_witnesses_are_ambiguous() {
    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected(),
            vec![observation(&[1], &[100]), observation(&[2], &[100])],
        );

    assert_eq!(
        realization.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::Ambiguous
    );

    assert!(realization.is_ambiguous());

    assert!(realization.realized_observation().is_none());
}

#[test]
fn multiple_end_witnesses_are_ambiguous() {
    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected(),
            vec![observation(&[1], &[100]), observation(&[1], &[200])],
        );

    assert_eq!(
        realization.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::Ambiguous
    );

    assert!(realization.is_ambiguous());
}

#[test]
fn uncovered_application_noise_is_ignored() {
    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected(),
            vec![
                observation(&[1, 700], &[100, 800]),
                observation(&[900], &[901]),
            ],
        );

    assert_eq!(
        realization.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizationStatus::Deterministic
    );

    assert_eq!(realization.premise_witnesses(), &[unit(1,),]);

    assert_eq!(realization.conclusion_witnesses(), &[unit(100,),]);
}

#[test]
fn duplicate_observations_do_not_create_false_ambiguity() {
    let exact = observation(&[1], &[100]);

    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected(),
            vec![exact.clone(), exact],
        );

    assert_eq!(realization.application_observations().len(), 1);

    assert!(realization.is_deterministic());
}

#[test]
fn middle_class_does_not_require_arbitrary_concrete_witness() {
    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected(),
            vec![observation(&[1], &[100])],
        );

    assert_eq!(realization.middle_class(), &class(&[10, 20],));

    assert!(realization.is_deterministic());
}

#[test]
fn realization_preserves_projected_motif_and_support() {
    let projected = projected();

    let before = projected.clone();

    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected,
            vec![observation(&[1], &[100])],
        );

    assert_eq!(realization.projected_motif(), &before);

    assert_eq!(realization.support_count(), before.motif().support_count());

    assert_eq!(
        realization.matching_selections(),
        before.matching_selections()
    );
}

#[test]
fn realization_preserves_endpoint_class_identity() {
    let realization =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected(),
            vec![observation(&[1], &[100])],
        );

    assert_eq!(realization.start_class(), &class(&[1, 2],));

    assert_eq!(realization.middle_class(), &class(&[10, 20],));

    assert_eq!(realization.end_class(), &class(&[100, 200],));
}

#[test]
fn realizer_facade_matches_direct_realization() {
    let projected = projected();

    let application = vec![observation(&[1], &[100])];

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealizer::realize(
            projected.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
            projected,
            application,
        )
    );
}

#[test]
fn motif_realization_is_canonical_deterministic_and_non_mutating() {
    let projected = projected();

    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let projected_before = projected.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
        projected.clone(),
        vec![first.clone(), second.clone()],
    );

    let right = RecursiveWorldRevisionAbstractionCompositionGeneralizationRealization::realize(
        projected.clone(),
        vec![second, first],
    );

    assert_eq!(left, right);

    assert_eq!(projected, projected_before);

    assert!(left.is_deterministic());

    assert_eq!(
        left.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );
}
