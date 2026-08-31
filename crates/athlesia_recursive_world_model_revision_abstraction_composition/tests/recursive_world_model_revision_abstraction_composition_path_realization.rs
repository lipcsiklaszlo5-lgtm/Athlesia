use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposition,
    RecursiveWorldRevisionAbstractionCompositionPathRealization,
    RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus,
    RecursiveWorldRevisionAbstractionCompositionPathRealizer,
    RecursiveWorldRevisionAbstractionCompositionPathSelection,
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
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

fn composition() -> RecursiveWorldRevisionAbstractionComposition {
    let mut witnesses = Vec::new();

    for index in 0..3 {
        witnesses.push(witness(
            &[1, 2],
            &[10, 20],
            if index % 2 == 0 { 1 } else { 2 },
            if index % 2 == 0 { 10 } else { 20 },
            100 + index,
        ));

        witnesses.push(witness(
            &[10, 20],
            &[100, 200],
            if index % 2 == 0 { 10 } else { 20 },
            if index % 2 == 0 { 100 } else { 200 },
            200 + index,
        ));
    }

    RecursiveWorldRevisionAbstractionComposition::compose(
        RecursiveWorldRevisionAbstractionCompositionWitnessSet::new(witnesses).unwrap(),
        RecursiveWorldRevisionAbstractionCompositionThreshold::new(2).unwrap(),
    )
    .unwrap()
}

fn selection() -> RecursiveWorldRevisionAbstractionCompositionPathSelection {
    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(composition()).unwrap();

    let supports = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths);

    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(supports)
        .selection_for(&class(&[1, 2]), &class(&[100, 200]))
        .unwrap()
        .clone()
}

#[test]
fn realization_without_start_witness_is_unavailable() {
    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selection(),
        vec![observation(&[900], &[100])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::Unavailable
    );

    assert!(result.realized_observation().is_none());
}

#[test]
fn realization_without_end_witness_is_unavailable() {
    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selection(),
        vec![observation(&[1], &[900])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::Unavailable
    );

    assert!(result.realized_observation().is_none());
}

#[test]
fn unique_endpoint_witnesses_realize_deterministically() {
    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selection(),
        vec![
            observation(&[1, 700], &[100, 800]),
            observation(&[1, 701], &[100, 801]),
        ],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::Deterministic
    );

    assert!(result.is_deterministic());

    assert_eq!(
        result.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );
}

#[test]
fn multiple_start_witnesses_are_ambiguous() {
    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selection(),
        vec![observation(&[1], &[100]), observation(&[2], &[100])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::Ambiguous
    );

    assert!(result.is_ambiguous());

    assert!(result.realized_observation().is_none());
}

#[test]
fn multiple_end_witnesses_are_ambiguous() {
    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selection(),
        vec![observation(&[1], &[100]), observation(&[1], &[200])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionPathRealizationStatus::Ambiguous
    );

    assert!(result.realized_observation().is_none());
}

#[test]
fn uncovered_application_noise_is_ignored() {
    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selection(),
        vec![
            observation(&[1, 700, 701], &[100, 800, 801]),
            observation(&[900], &[901]),
        ],
    );

    assert_eq!(result.premise_witnesses(), &[unit(1,),]);

    assert_eq!(result.conclusion_witnesses(), &[unit(100,),]);

    assert!(result.is_deterministic());
}

#[test]
fn duplicate_application_observations_do_not_create_ambiguity() {
    let application = observation(&[1, 700], &[100, 800]);

    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selection(),
        vec![application.clone(), application],
    );

    assert_eq!(result.application_observations().len(), 1);

    assert!(result.is_deterministic());
}

#[test]
fn realization_preserves_selected_path_identity() {
    let selected = selection();

    let before = selected.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selected,
        vec![observation(&[1], &[100])],
    );

    assert_eq!(result.selection(), &before);

    assert_eq!(result.path(), before.path());

    assert_eq!(result.minimum_support(), before.minimum_support());
}

#[test]
fn realization_preserves_endpoint_class_identity() {
    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selection(),
        vec![observation(&[1], &[100])],
    );

    assert_eq!(result.from(), &class(&[1, 2],));

    assert_eq!(result.to(), &class(&[100, 200],));
}

#[test]
fn realization_preserves_canonical_application_observations() {
    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let result = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selection(),
        vec![second.clone(), first.clone()],
    );

    let mut expected = vec![first, second];

    expected.sort();
    expected.dedup();

    assert_eq!(result.application_observations(), expected.as_slice());
}

#[test]
fn path_realizer_facade_matches_direct_realization() {
    let selected = selection();

    let application = vec![observation(&[1], &[100])];

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionPathRealizer::realize(
            selected.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(selected, application,)
    );
}

#[test]
fn path_realization_is_canonical_deterministic_and_non_mutating() {
    let selected = selection();

    let selected_before = selected.clone();

    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let left = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selected.clone(),
        vec![first.clone(), second.clone()],
    );

    let right = RecursiveWorldRevisionAbstractionCompositionPathRealization::realize(
        selected.clone(),
        vec![second, first],
    );

    assert_eq!(left, right);

    assert_eq!(selected, selected_before);

    assert!(left.is_deterministic());

    assert_eq!(
        left.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );
}
