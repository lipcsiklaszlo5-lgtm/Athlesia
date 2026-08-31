use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposer, RecursiveWorldRevisionAbstractionComposition,
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

fn source(
    witnesses: Vec<RecursiveWorldRevisionAbstractionCompositionWitness>,
) -> RecursiveWorldRevisionAbstractionCompositionWitnessSet {
    RecursiveWorldRevisionAbstractionCompositionWitnessSet::new(witnesses).unwrap()
}

fn threshold(support: usize) -> RecursiveWorldRevisionAbstractionCompositionThreshold {
    RecursiveWorldRevisionAbstractionCompositionThreshold::new(support).unwrap()
}

#[test]
fn composition_threshold_rejects_zero() {
    assert!(RecursiveWorldRevisionAbstractionCompositionThreshold::new(0,).is_none());
}

#[test]
fn composition_threshold_requires_at_least_two_observations() {
    assert!(RecursiveWorldRevisionAbstractionCompositionThreshold::new(1,).is_none());

    assert_eq!(threshold(2,).min_observation_support(), 2);
}

#[test]
fn composition_witness_requires_premise_class_coverage() {
    assert!(RecursiveWorldRevisionAbstractionCompositionWitness::new(
        class(&[1, 2],),
        class(&[10, 20],),
        observation(&[3], &[10],),
    )
    .is_none());
}

#[test]
fn composition_witness_requires_conclusion_class_coverage() {
    assert!(RecursiveWorldRevisionAbstractionCompositionWitness::new(
        class(&[1, 2],),
        class(&[10, 20],),
        observation(&[1], &[30],),
    )
    .is_none());
}

#[test]
fn composition_witness_rejects_self_loop() {
    let abstraction = class(&[1, 2]);

    assert!(RecursiveWorldRevisionAbstractionCompositionWitness::new(
        abstraction.clone(),
        abstraction,
        observation(&[1], &[2],),
    )
    .is_none());
}

#[test]
fn repeated_exact_relation_reaches_support_threshold() {
    let composition = RecursiveWorldRevisionAbstractionComposition::compose(
        source(vec![
            witness(&[1, 2], &[10, 20], &[1, 70], &[10, 80]),
            witness(&[1, 2], &[10, 20], &[2, 71], &[20, 81]),
        ]),
        threshold(2),
    )
    .unwrap();

    assert_eq!(composition.len(), 1);

    assert_eq!(composition.edges()[0].support_count(), 2);
}

#[test]
fn insufficient_support_does_not_materialize_edge() {
    let result = RecursiveWorldRevisionAbstractionComposition::compose(
        source(vec![witness(&[1, 2], &[10, 20], &[1], &[10])]),
        threshold(2),
    );

    assert!(result.is_none());
}

#[test]
fn composition_is_directional() {
    let composition = RecursiveWorldRevisionAbstractionComposition::compose(
        source(vec![
            witness(&[1, 2], &[10, 20], &[1], &[10]),
            witness(&[1, 2], &[10, 20], &[2], &[20]),
            witness(&[10, 20], &[1, 2], &[10], &[1]),
            witness(&[10, 20], &[1, 2], &[20], &[2]),
        ]),
        threshold(2),
    )
    .unwrap();

    assert_eq!(composition.len(), 2);

    assert!(composition
        .edge(&class(&[1, 2],), &class(&[10, 20],),)
        .is_some());

    assert!(composition
        .edge(&class(&[10, 20],), &class(&[1, 2],),)
        .is_some());
}

#[test]
fn duplicate_observation_does_not_inflate_support() {
    let repeated = witness(&[1, 2], &[10, 20], &[1], &[10]);

    let result = RecursiveWorldRevisionAbstractionComposition::compose(
        source(vec![repeated.clone(), repeated]),
        threshold(2),
    );

    assert!(result.is_none());
}

#[test]
fn composition_preserves_exact_observation_provenance() {
    let first = observation(&[1, 70], &[10, 80]);

    let second = observation(&[2, 71], &[20, 81]);

    let composition = RecursiveWorldRevisionAbstractionComposition::compose(
        source(vec![
            RecursiveWorldRevisionAbstractionCompositionWitness::new(
                class(&[1, 2]),
                class(&[10, 20]),
                first.clone(),
            )
            .unwrap(),
            RecursiveWorldRevisionAbstractionCompositionWitness::new(
                class(&[1, 2]),
                class(&[10, 20]),
                second.clone(),
            )
            .unwrap(),
        ]),
        threshold(2),
    )
    .unwrap();

    assert_eq!(
        composition.edges()[0].supporting_observations(),
        &[first, second,]
    );
}

#[test]
fn composer_facade_matches_direct_composition() {
    let witnesses = source(vec![
        witness(&[1, 2], &[10, 20], &[1], &[10]),
        witness(&[1, 2], &[10, 20], &[2], &[20]),
    ]);

    assert_eq!(
        RecursiveWorldRevisionAbstractionComposer::compose(witnesses.clone(), threshold(2,),),
        RecursiveWorldRevisionAbstractionComposition::compose(witnesses, threshold(2,),)
    );
}

#[test]
fn composition_is_canonical_deterministic_and_non_mutating() {
    let first = witness(&[1, 2], &[10, 20], &[1, 70], &[10, 80]);

    let second = witness(&[1, 2], &[10, 20], &[2, 71], &[20, 81]);

    let witnesses = source(vec![first.clone(), second.clone()]);

    let before = witnesses.clone();

    let left =
        RecursiveWorldRevisionAbstractionComposition::compose(witnesses.clone(), threshold(2));

    let right = RecursiveWorldRevisionAbstractionComposition::compose(
        source(vec![second, first]),
        threshold(2),
    );

    assert_eq!(left, right);

    assert_eq!(witnesses, before);

    assert_eq!(left.unwrap().edges()[0].support_count(), 2);
}
