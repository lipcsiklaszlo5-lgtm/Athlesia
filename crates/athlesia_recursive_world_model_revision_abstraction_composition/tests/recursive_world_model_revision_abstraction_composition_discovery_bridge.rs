use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposition,
    RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge,
    RecursiveWorldRevisionAbstractionCompositionDiscoveryBuilder,
    RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus,
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

#[test]
fn unavailable_realization_blocks_discovery() {
    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        rule(&[9], &[99]),
        selection(),
        vec![observation(&[900], &[901])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus::RealizationUnavailable
    );

    assert!(result.hypothesis().is_none());
}

#[test]
fn ambiguous_realization_blocks_discovery() {
    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        rule(&[9], &[99]),
        selection(),
        vec![observation(&[1], &[100]), observation(&[2], &[100])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus::RealizationUnavailable
    );

    assert!(result.hypothesis().is_none());
}

#[test]
fn deterministic_realization_discovers_hypothesis() {
    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        rule(&[9], &[99]),
        selection(),
        vec![observation(&[1], &[100])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus::Discovered
    );

    assert!(result.is_discovered());

    assert!(result.hypothesis().is_some());
}

#[test]
fn noop_replacement_is_discovery_unavailable() {
    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        rule(&[1], &[100]),
        selection(),
        vec![observation(&[1], &[100])],
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionDiscoveryStatus::DiscoveryUnavailable
    );

    assert!(result.hypothesis().is_none());
}

#[test]
fn discovery_preserves_exact_target_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        target.clone(),
        selection(),
        vec![observation(&[1], &[100])],
    );

    assert_eq!(result.target(), &target);
}

#[test]
fn discovery_preserves_realized_observation_identity() {
    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        rule(&[9], &[99]),
        selection(),
        vec![observation(&[1, 700], &[100, 800])],
    );

    assert_eq!(
        result.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );
}

#[test]
fn discovery_replacement_matches_realized_rule() {
    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        rule(&[9], &[99]),
        selection(),
        vec![observation(&[1], &[100])],
    );

    assert_eq!(result.replacement(), Some(&rule(&[1], &[100],),));
}

#[test]
fn discovery_preserves_selected_path_identity() {
    let selected = selection();

    let before = selected.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        rule(&[9], &[99]),
        selected,
        vec![observation(&[1], &[100])],
    );

    assert_eq!(result.selection(), &before);

    assert_eq!(result.path(), before.path());

    assert_eq!(result.minimum_support(), before.minimum_support());
}

#[test]
fn discovery_preserves_application_observation_provenance() {
    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        rule(&[9], &[99]),
        selection(),
        vec![second.clone(), first.clone()],
    );

    let mut expected = vec![first, second];

    expected.sort();
    expected.dedup();

    assert_eq!(result.application_observations(), expected.as_slice());
}

#[test]
fn discovered_hypothesis_preserves_target_and_replacement() {
    let target = rule(&[9], &[99]);

    let replacement = rule(&[1], &[100]);

    let result = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        target.clone(),
        selection(),
        vec![observation(&[1], &[100])],
    );

    let hypothesis = result.hypothesis().unwrap();

    assert_eq!(hypothesis.target(), &target);

    assert_eq!(hypothesis.replacement(), &replacement);
}

#[test]
fn discovery_builder_facade_matches_direct_discovery() {
    let target = rule(&[9], &[99]);

    let selected = selection();

    let application = vec![observation(&[1], &[100])];

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionDiscoveryBuilder::discover(
            target.clone(),
            selected.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
            target,
            selected,
            application,
        )
    );
}

#[test]
fn composition_discovery_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let selected = selection();

    let target_before = target.clone();

    let selected_before = selected.clone();

    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let left = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        target.clone(),
        selected.clone(),
        vec![first.clone(), second.clone()],
    );

    let right = RecursiveWorldRevisionAbstractionCompositionDiscoveryBridge::discover(
        target.clone(),
        selected.clone(),
        vec![second, first],
    );

    assert_eq!(left, right);

    assert_eq!(target, target_before);

    assert_eq!(selected, selected_before);

    assert!(left.is_discovered());
}
