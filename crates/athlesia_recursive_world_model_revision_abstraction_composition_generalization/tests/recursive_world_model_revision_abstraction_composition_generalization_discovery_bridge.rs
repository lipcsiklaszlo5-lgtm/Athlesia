use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

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
    RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBuilder,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection,
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

fn context(seed: usize) -> RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
    let relations = vec![(vec![1, 2], vec![10, 20]), (vec![10, 20], vec![100, 200])];

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

fn projected() -> RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
    let source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        context(1),
        context(2),
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
        context(10),
    );

    projection.projected_motifs().first().unwrap().clone()
}

fn deterministic_application() -> Vec<RecursiveWorldRevisionDiscoveryObservation> {
    vec![observation(&[1, 700], &[100, 800])]
}

#[test]
fn unavailable_realization_blocks_discovery() {
    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            rule(&[9], &[99]),
            projected(),
            vec![observation(&[900], &[901])],
        );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus::
            RealizationUnavailable
    );

    assert!(result.hypothesis().is_none());
}

#[test]
fn ambiguous_realization_blocks_discovery() {
    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            rule(&[9], &[99]),
            projected(),
            vec![observation(&[1], &[100]), observation(&[2], &[100])],
        );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus::
            RealizationUnavailable
    );

    assert!(result.hypothesis().is_none());
}

#[test]
fn deterministic_realization_discovers_hypothesis() {
    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            rule(&[9], &[99]),
            projected(),
            deterministic_application(),
        );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus::Discovered
    );

    assert!(result.is_discovered());

    assert!(result.hypothesis().is_some());
}

#[test]
fn exact_noop_target_is_discovery_unavailable() {
    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            rule(&[1], &[100]),
            projected(),
            deterministic_application(),
        );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryStatus::
            DiscoveryUnavailable
    );

    assert!(result.hypothesis().is_none());
}

#[test]
fn discovery_preserves_exact_target_identity() {
    let target = rule(&[9], &[99]);

    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            target.clone(),
            projected(),
            deterministic_application(),
        );

    assert_eq!(result.target(), &target);
}

#[test]
fn discovery_preserves_realized_observation_identity() {
    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            rule(&[9], &[99]),
            projected(),
            deterministic_application(),
        );

    assert_eq!(
        result.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );
}

#[test]
fn discovered_replacement_equals_realized_rule() {
    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            rule(&[9], &[99]),
            projected(),
            deterministic_application(),
        );

    assert_eq!(result.replacement(), Some(&rule(&[1], &[100],),));
}

#[test]
fn discovery_preserves_projected_motif_identity() {
    let projected = projected();

    let before = projected.clone();

    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            rule(&[9], &[99]),
            projected,
            deterministic_application(),
        );

    assert_eq!(result.projected_motif(), &before);

    assert_eq!(result.classes(), before.classes());

    assert_eq!(result.support_count(), before.motif().support_count());
}

#[test]
fn discovery_preserves_matching_selection_provenance() {
    let projected = projected();

    let expected = projected.matching_selections().to_vec();

    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            rule(&[9], &[99]),
            projected,
            deterministic_application(),
        );

    assert_eq!(result.matching_selections(), expected.as_slice());
}

#[test]
fn discovery_preserves_application_observation_provenance() {
    let application = deterministic_application();

    let before = application.clone();

    let result =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            rule(&[9], &[99]),
            projected(),
            application,
        );

    assert_eq!(result.application_observations(), before.as_slice());
}

#[test]
fn discovery_builder_facade_matches_direct_discovery() {
    let target = rule(&[9], &[99]);

    let projected = projected();

    let application = deterministic_application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBuilder::discover(
            target.clone(),
            projected.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
            target,
            projected,
            application,
        )
    );
}

#[test]
fn generalized_composition_discovery_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let projected = projected();

    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let target_before = target.clone();

    let projected_before = projected.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
        target.clone(),
        projected.clone(),
        vec![first.clone(), second.clone()],
    );

    let right = RecursiveWorldRevisionAbstractionCompositionGeneralizationDiscoveryBridge::discover(
        target.clone(),
        projected.clone(),
        vec![second, first],
    );

    assert_eq!(left, right);

    assert_eq!(target, target_before);

    assert_eq!(projected, projected_before);

    assert!(left.is_discovered());
}
