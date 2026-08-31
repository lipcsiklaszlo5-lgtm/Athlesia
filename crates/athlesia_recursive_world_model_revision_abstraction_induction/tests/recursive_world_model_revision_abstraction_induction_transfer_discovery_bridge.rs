use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionTransferDiscoveryBridge,
    RecursiveWorldRevisionAbstractionTransferDiscoveryBuilder,
    RecursiveWorldRevisionAbstractionTransferDiscoveryStatus,
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

fn induction_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1], &[10]),
        observation(&[2], &[10]),
        observation(&[1], &[20]),
        observation(&[2], &[20]),
    ])
}

fn deterministic_transfer_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 50], &[10, 60]),
        observation(&[1, 51], &[10, 61]),
    ])
}

#[test]
fn ambiguous_transfer_blocks_discovery() {
    let target = rule(&[9], &[99]);

    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        target,
        induction_source(),
        observation_set(vec![observation(&[1], &[10]), observation(&[2], &[10])]),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionTransferDiscoveryStatus::TransferUnavailable
    );

    assert!(bridge.hypothesis().is_none());
}

#[test]
fn deterministic_transfer_discovers_revision_hypothesis() {
    let target = rule(&[9], &[99]);

    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionTransferDiscoveryStatus::Discovered
    );

    assert!(bridge.is_discovered());

    assert!(bridge.hypothesis().is_some());
}

#[test]
fn discovered_hypothesis_preserves_target_identity() {
    let target = rule(&[9], &[99]);

    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        target.clone(),
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(bridge.target(), &target);

    assert_eq!(bridge.hypothesis().unwrap().target(), &target);
}

#[test]
fn discovered_hypothesis_uses_transfer_realized_observation() {
    let target = rule(&[9], &[99]);

    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        bridge.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );

    assert_eq!(
        bridge.hypothesis().unwrap().observation(),
        &observation(&[1], &[10],)
    );
}

#[test]
fn discovered_replacement_matches_realized_transfer_rule() {
    let target = rule(&[9], &[99]);

    let expected = rule(&[1], &[10]);

    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(bridge.replacement(), Some(&expected,));

    assert_eq!(bridge.hypothesis().unwrap().replacement(), &expected);
}

#[test]
fn transfer_noop_is_discovery_unavailable() {
    let target = rule(&[1], &[10]);

    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionTransferDiscoveryStatus::DiscoveryUnavailable
    );

    assert!(bridge.hypothesis().is_none());

    assert!(bridge.replacement().is_none());
}

#[test]
fn discovery_bridge_preserves_induction_provenance() {
    let induction = induction_source();

    let before = induction.clone();

    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        rule(&[9], &[99]),
        induction,
        deterministic_transfer_source(),
    );

    assert_eq!(bridge.induction_observations(), &before);
}

#[test]
fn discovery_bridge_preserves_transfer_provenance() {
    let transfer = deterministic_transfer_source();

    let before = transfer.clone();

    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        rule(&[9], &[99]),
        induction_source(),
        transfer,
    );

    assert_eq!(bridge.transfer_observations(), &before);
}

#[test]
fn discovery_bridge_preserves_learned_consensus_identity() {
    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        rule(&[9], &[99]),
        induction_source(),
        deterministic_transfer_source(),
    );

    let consensus = bridge.consensus().unwrap();

    assert_eq!(consensus.observation_count(), 4);

    assert_eq!(
        consensus.premise_classes()[0].members(),
        &[unit(1), unit(2),]
    );

    assert_eq!(
        consensus.conclusion_classes()[0].members(),
        &[unit(10), unit(20),]
    );
}

#[test]
fn discovery_bridge_preserves_learned_vocabulary_identity() {
    let bridge = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        rule(&[9], &[99]),
        induction_source(),
        deterministic_transfer_source(),
    );

    let vocabulary = bridge.vocabulary().unwrap();

    assert_eq!(vocabulary.classes().len(), 2);

    assert!(vocabulary.covers(&unit(1),));

    assert!(vocabulary.covers(&unit(10),));
}

#[test]
fn transfer_discovery_builder_matches_direct_discovery() {
    let target = rule(&[9], &[99]);

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionTransferDiscoveryBuilder::discover(
            target.clone(),
            induction.clone(),
            transfer.clone(),
        ),
        RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
            target, induction, transfer,
        )
    );
}

#[test]
fn transfer_discovery_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    let induction_before = induction.clone();

    let transfer_before = transfer.clone();

    let left = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        target.clone(),
        induction.clone(),
        transfer.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionTransferDiscoveryBridge::discover(
        target,
        observation_set(vec![
            observation(&[2], &[20]),
            observation(&[1], &[10]),
            observation(&[1], &[20]),
            observation(&[2], &[10]),
        ]),
        observation_set(vec![
            observation(&[1, 51], &[10, 61]),
            observation(&[1, 50], &[10, 60]),
        ]),
    );

    assert_eq!(left, right);

    assert_eq!(induction, induction_before);

    assert_eq!(transfer, transfer_before);

    assert!(left.is_discovered());
}
