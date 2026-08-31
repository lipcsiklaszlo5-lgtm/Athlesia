use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_abstraction_generalization::{
    RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge,
    RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBuilder,
    RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus,
    RecursiveWorldRevisionAbstractionGeneralizationThreshold,
    RecursiveWorldRevisionAbstractionGeneralizedClassSet,
};

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInducedClassSet,
    RecursiveWorldRevisionAbstractionSubstitutionWitness,
    RecursiveWorldRevisionAbstractionSubstitutionWitnessSet,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(
        StructuralConcept::with_sequence_length(
            vec![PrimitiveSignature::new(RelationKind::Equal, span)],
            8,
        ),
    ))
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

fn observation_set(
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionInductionObservationSet {
    RecursiveWorldRevisionInductionObservationSet::new(observations).unwrap()
}

fn premise_witness(
    first: usize,
    second: usize,
    shared: usize,
    fixed_conclusion: usize,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[first, shared], &[fixed_conclusion]),
        observation(&[second, shared], &[fixed_conclusion]),
    )
    .unwrap()
}

fn conclusion_witness(
    first: usize,
    second: usize,
    shared: usize,
    fixed_premise: usize,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[fixed_premise], &[first, shared]),
        observation(&[fixed_premise], &[second, shared]),
    )
    .unwrap()
}

fn induced(
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
) -> RecursiveWorldRevisionAbstractionInducedClassSet {
    RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(witnesses).unwrap(),
    )
    .unwrap()
}

fn generalized(
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
) -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        induced(witnesses),
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap()
}

fn generalized_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    generalized(vec![
        premise_witness(1, 2, 30, 40),
        premise_witness(1, 2, 31, 41),
        conclusion_witness(10, 20, 50, 60),
        conclusion_witness(10, 20, 51, 61),
    ])
}

fn conflicted_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    generalized(vec![
        premise_witness(1, 2, 30, 40),
        premise_witness(1, 2, 31, 41),
        conclusion_witness(2, 3, 50, 60),
        conclusion_witness(2, 3, 51, 61),
    ])
}

fn deterministic_application() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 70], &[10, 80]),
        observation(&[1, 71], &[10, 81]),
    ])
}

fn ambiguous_application() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 70], &[10, 80]),
        observation(&[2, 71], &[10, 81]),
    ])
}

#[test]
fn unavailable_realization_blocks_discovery() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        rule(&[9], &[99]),
        conflicted_source(),
        deterministic_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus::RealizationUnavailable
    );

    assert!(bridge.hypothesis().is_none());

    assert!(bridge.realized_observation().is_none());
}

#[test]
fn ambiguous_realization_blocks_discovery() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        rule(&[9], &[99]),
        generalized_source(),
        ambiguous_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus::RealizationUnavailable
    );

    assert!(bridge.hypothesis().is_none());

    assert!(bridge.realization().is_ambiguous());
}

#[test]
fn deterministic_realization_discovers_hypothesis() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus::Discovered
    );

    assert!(bridge.is_discovered());

    assert!(bridge.hypothesis().is_some());
}

#[test]
fn deterministic_noop_is_discovery_unavailable() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        rule(&[1], &[10]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationDiscoveryStatus::DiscoveryUnavailable
    );

    assert!(bridge.hypothesis().is_none());

    assert!(bridge.realized_observation().is_some());
}

#[test]
fn discovered_hypothesis_preserves_target_identity() {
    let target = rule(&[9], &[99]);

    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        target.clone(),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(bridge.target(), &target);

    assert_eq!(bridge.hypothesis().unwrap().target(), &target);
}

#[test]
fn discovered_hypothesis_preserves_realized_observation_identity() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        bridge.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );

    assert_eq!(
        bridge.hypothesis().unwrap().observation(),
        bridge.realized_observation().unwrap()
    );
}

#[test]
fn discovered_hypothesis_materializes_expected_replacement() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(bridge.replacement(), Some(&rule(&[1], &[10],),));

    assert_eq!(
        bridge.hypothesis().unwrap().replacement(),
        &rule(&[1], &[10],)
    );
}

#[test]
fn discovery_preserves_generalization_source_provenance() {
    let source = generalized_source();

    let before = source.clone();

    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        rule(&[9], &[99]),
        source,
        deterministic_application(),
    );

    assert_eq!(bridge.generalized_source(), &before);
}

#[test]
fn discovery_preserves_application_provenance() {
    let application = deterministic_application();

    let before = application.clone();

    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        rule(&[9], &[99]),
        generalized_source(),
        application,
    );

    assert_eq!(bridge.application_observations(), &before);

    assert_eq!(bridge.realization().application_observations(), &before);
}

#[test]
fn discovery_preserves_generalized_consensus_and_vocabulary_identity() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        bridge.consensus().unwrap(),
        bridge.realization().consensus().unwrap()
    );

    assert_eq!(
        bridge.vocabulary().unwrap(),
        bridge.realization().vocabulary().unwrap()
    );

    assert_eq!(bridge.vocabulary().unwrap().classes().len(), 2);
}

#[test]
fn discovery_builder_matches_direct_discovery() {
    let target = rule(&[9], &[99]);

    let source = generalized_source();

    let application = deterministic_application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBuilder::discover(
            target.clone(),
            source.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
            target,
            source,
            application,
        )
    );
}

#[test]
fn generalized_discovery_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let source = generalized_source();

    let application = deterministic_application();

    let target_before = target.clone();

    let source_before = source.clone();

    let application_before = application.clone();

    let left = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        target.clone(),
        source.clone(),
        application.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionGeneralizationDiscoveryBridge::discover(
        target.clone(),
        generalized(vec![
            conclusion_witness(10, 20, 51, 61),
            premise_witness(1, 2, 31, 41),
            conclusion_witness(10, 20, 50, 60),
            premise_witness(1, 2, 30, 40),
        ]),
        observation_set(vec![
            observation(&[1, 71], &[10, 81]),
            observation(&[1, 70], &[10, 80]),
        ]),
    );

    assert_eq!(left, right);

    assert_eq!(target, target_before);

    assert_eq!(source, source_before);

    assert_eq!(application, application_before);

    assert!(left.is_discovered());

    assert_eq!(left.replacement(), Some(&rule(&[1], &[10],),));
}
