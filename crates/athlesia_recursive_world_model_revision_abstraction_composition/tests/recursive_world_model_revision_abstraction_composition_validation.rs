use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposition,
    RecursiveWorldRevisionAbstractionCompositionPathSelection,
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    RecursiveWorldRevisionAbstractionCompositionPathSet,
    RecursiveWorldRevisionAbstractionCompositionPathSupportSet,
    RecursiveWorldRevisionAbstractionCompositionThreshold,
    RecursiveWorldRevisionAbstractionCompositionValidation,
    RecursiveWorldRevisionAbstractionCompositionValidationStatus,
    RecursiveWorldRevisionAbstractionCompositionValidator,
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

fn rule(premises: &[usize], conclusions: &[usize]) -> RecursiveWorldRule {
    RecursiveWorldRule::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
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

fn selection() -> RecursiveWorldRevisionAbstractionCompositionPathSelection {
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

    let composition = RecursiveWorldRevisionAbstractionComposition::compose(
        RecursiveWorldRevisionAbstractionCompositionWitnessSet::new(witnesses).unwrap(),
        RecursiveWorldRevisionAbstractionCompositionThreshold::new(2).unwrap(),
    )
    .unwrap();

    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(composition).unwrap();

    let supports = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths);

    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(supports)
        .selection_for(&class(&[1, 2]), &class(&[100, 200]))
        .unwrap()
        .clone()
}

fn deterministic_application() -> Vec<RecursiveWorldRevisionDiscoveryObservation> {
    vec![observation(&[1, 700], &[100, 800])]
}

#[test]
fn unavailable_discovery_is_validation_unavailable() {
    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        RecursiveWorldModel::new(vec![rule(&[9], &[99])]),
        rule(&[9], &[99]),
        selection(),
        vec![observation(&[900], &[901])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionValidationStatus::DiscoveryUnavailable
    );

    assert!(result.validation_result().is_none());
}

#[test]
fn noop_discovery_is_validation_unavailable() {
    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        RecursiveWorldModel::new(vec![rule(&[1], &[100])]),
        rule(&[1], &[100]),
        selection(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionValidationStatus::DiscoveryUnavailable
    );

    assert!(result.hypothesis().is_none());
}

#[test]
fn missing_target_revision_is_rejected() {
    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        RecursiveWorldModel::new(vec![rule(&[8], &[88])]),
        rule(&[9], &[99]),
        selection(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionValidationStatus::Rejected
    );

    assert!(result.is_rejected());

    assert!(result.rejected_hypothesis().is_some());
}

#[test]
fn replacement_collision_is_rejected() {
    let target = rule(&[9], &[99]);

    let replacement = rule(&[1], &[100]);

    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        RecursiveWorldModel::new(vec![target.clone(), replacement]),
        target,
        selection(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionValidationStatus::Rejected
    );

    assert!(result.is_rejected());
}

#[test]
fn valid_composed_revision_is_accepted() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target,
        selection(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionValidationStatus::Accepted
    );

    assert!(result.is_accepted());

    assert!(result.accepted_hypothesis().is_some());
}

#[test]
fn accepted_validation_preserves_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target,
        selection(),
        deterministic_application(),
    );

    assert_eq!(result.accepted_hypothesis(), result.hypothesis());
}

#[test]
fn rejected_validation_preserves_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        RecursiveWorldModel::new(vec![rule(&[8], &[88])]),
        target,
        selection(),
        deterministic_application(),
    );

    assert_eq!(result.rejected_hypothesis(), result.hypothesis());
}

#[test]
fn validation_preserves_target_and_replacement_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target.clone(),
        selection(),
        deterministic_application(),
    );

    assert_eq!(result.target(), &target);

    assert_eq!(result.replacement(), Some(&rule(&[1], &[100],),));
}

#[test]
fn validation_preserves_realization_and_path_provenance() {
    let selected = selection();

    let before = selected.clone();

    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        RecursiveWorldModel::new(vec![target.clone()]),
        target,
        selected,
        deterministic_application(),
    );

    assert_eq!(result.selection(), &before);

    assert_eq!(result.path(), before.path());

    assert_eq!(result.minimum_support(), before.minimum_support());

    assert_eq!(
        result.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );
}

#[test]
fn validation_preserves_model_and_application_provenance() {
    let target = rule(&[9], &[99]);

    let world = RecursiveWorldModel::new(vec![target.clone()]);

    let application = deterministic_application();

    let world_before = world.clone();

    let application_before = application.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        world,
        target,
        selection(),
        application,
    );

    assert_eq!(result.model(), &world_before);

    assert_eq!(
        result.application_observations(),
        application_before.as_slice()
    );
}

#[test]
fn composition_validator_facade_matches_direct_validation() {
    let target = rule(&[9], &[99]);

    let world = RecursiveWorldModel::new(vec![target.clone()]);

    let selected = selection();

    let application = deterministic_application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionValidator::validate(
            world.clone(),
            target.clone(),
            selected.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionCompositionValidation::validate(
            world,
            target,
            selected,
            application,
        )
    );
}

#[test]
fn composition_validation_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let world = RecursiveWorldModel::new(vec![target.clone()]);

    let selected = selection();

    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let world_before = world.clone();

    let target_before = target.clone();

    let selected_before = selected.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        world.clone(),
        target.clone(),
        selected.clone(),
        vec![first.clone(), second.clone()],
    );

    let right = RecursiveWorldRevisionAbstractionCompositionValidation::validate(
        world.clone(),
        target.clone(),
        selected.clone(),
        vec![second, first],
    );

    assert_eq!(left, right);

    assert_eq!(world, world_before);

    assert_eq!(target, target_before);

    assert_eq!(selected, selected_before);

    assert!(left.is_accepted());
}
