use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInductionConsensusBridge,
    RecursiveWorldRevisionAbstractionInductionConsensusBuilder,
    RecursiveWorldRevisionAbstractionInductionConsensusStatus,
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

fn consensus_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1], &[10]),
        observation(&[2], &[10]),
        observation(&[1], &[20]),
        observation(&[2], &[20]),
    ])
}

#[test]
fn unavailable_projection_blocks_consensus() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(observation_set(vec![
            observation(&[1], &[10]),
            observation(&[2], &[20]),
        ]));

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionConsensusStatus::ProjectionUnavailable
    );

    assert!(bridge.consensus().is_none());
}

#[test]
fn automatically_induced_projection_derives_consensus() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(consensus_source());

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionConsensusStatus::ConsensusDerived
    );

    assert!(bridge.is_consensus_derived());

    assert!(bridge.consensus().is_some());
}

#[test]
fn induced_consensus_contains_abstract_premise_class() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(consensus_source());

    let consensus = bridge.consensus().unwrap();

    assert_eq!(consensus.premise_classes().len(), 1);

    let premise_class = &consensus.premise_classes()[0];

    assert_eq!(premise_class.members(), &[unit(1), unit(2),]);
}

#[test]
fn induced_consensus_contains_abstract_conclusion_class() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(consensus_source());

    let consensus = bridge.consensus().unwrap();

    assert_eq!(consensus.conclusion_classes().len(), 1);

    let conclusion_class = &consensus.conclusion_classes()[0];

    assert_eq!(conclusion_class.members(), &[unit(10), unit(20),]);
}

#[test]
fn induced_consensus_support_equals_observation_count() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(consensus_source());

    let consensus = bridge.consensus().unwrap();

    assert_eq!(consensus.observation_count(), 4);

    assert_eq!(
        consensus.premise_support(&consensus.premise_classes()[0],),
        4
    );

    assert_eq!(
        consensus.conclusion_support(&consensus.conclusion_classes()[0],),
        4
    );
}

#[test]
fn consensus_bridge_preserves_source_observations() {
    let source = consensus_source();

    let before = source.clone();

    let bridge = RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(source);

    assert_eq!(bridge.source_observations(), &before);

    assert_eq!(bridge.consensus().unwrap().source_observations(), &before);
}

#[test]
fn consensus_bridge_preserves_witness_provenance() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(consensus_source());

    let witnesses = bridge.witness_set().unwrap();

    assert_eq!(witnesses.len(), 4);

    assert_eq!(witnesses.premise_witnesses().len(), 2);

    assert_eq!(witnesses.conclusion_witnesses().len(), 2);
}

#[test]
fn consensus_bridge_preserves_induced_classes_and_resolution() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(consensus_source());

    assert_eq!(bridge.induced_classes().unwrap().len(), 4);

    assert_eq!(bridge.resolution().unwrap().resolved_count(), 2);

    assert_eq!(bridge.resolution().unwrap().conflicted_count(), 0);
}

#[test]
fn consensus_bridge_preserves_vocabulary_and_projection_identity() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(consensus_source());

    let consensus = bridge.consensus().unwrap();

    assert_eq!(bridge.vocabulary().unwrap(), consensus.vocabulary());

    assert_eq!(bridge.projection().unwrap(), consensus.projection());
}

#[test]
fn vocabulary_conflict_prevents_consensus_derivation() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(observation_set(vec![
            observation(&[1, 5], &[10]),
            observation(&[2, 5], &[10]),
            observation(&[2, 6], &[11]),
            observation(&[3, 6], &[11]),
        ]));

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionConsensusStatus::ProjectionUnavailable
    );

    assert!(!bridge.conflicts().is_empty());

    assert!(bridge.consensus().is_none());
}

#[test]
fn consensus_builder_facade_matches_direct_derivation() {
    let source = consensus_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionInductionConsensusBuilder::derive(source.clone(),),
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(source,)
    );
}

#[test]
fn induced_consensus_is_canonical_deterministic_and_non_mutating() {
    let first = observation(&[1], &[10]);

    let second = observation(&[2], &[10]);

    let third = observation(&[1], &[20]);

    let fourth = observation(&[2], &[20]);

    let source = observation_set(vec![
        fourth.clone(),
        first.clone(),
        third.clone(),
        second.clone(),
    ]);

    let before = source.clone();

    let left = RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(source.clone());

    let right =
        RecursiveWorldRevisionAbstractionInductionConsensusBridge::derive(observation_set(vec![
            second, third, first, fourth,
        ]));

    assert_eq!(left, right);

    assert_eq!(source, before);

    assert!(left.is_consensus_derived());
}
