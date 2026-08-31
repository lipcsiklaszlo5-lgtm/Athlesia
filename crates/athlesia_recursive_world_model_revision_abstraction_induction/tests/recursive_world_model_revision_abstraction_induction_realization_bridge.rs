use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionRealizationStatus;

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInductionRealizationBridge,
    RecursiveWorldRevisionAbstractionInductionRealizationStatus,
    RecursiveWorldRevisionAbstractionInductionRealizer,
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

fn full_grid_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1], &[10]),
        observation(&[2], &[10]),
        observation(&[1], &[20]),
        observation(&[2], &[20]),
    ])
}

#[test]
fn unavailable_consensus_blocks_realization() {
    let bridge = RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(
        observation_set(vec![observation(&[1], &[10]), observation(&[2], &[20])]),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionRealizationStatus::ConsensusUnavailable
    );

    assert!(bridge.realization().is_none());

    assert!(bridge.realized_observation().is_none());
}

#[test]
fn automatically_induced_consensus_is_ambiguous_on_its_own_evidence() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(full_grid_source());

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionRealizationStatus::Ambiguous
    );

    assert!(bridge.is_ambiguous());

    assert!(!bridge.is_deterministic());
}

#[test]
fn ambiguous_induced_realization_never_materializes_concrete_observation() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(full_grid_source());

    assert!(bridge.realized_observation().is_none());

    assert_eq!(
        bridge.realization().unwrap().status(),
        RecursiveWorldRevisionAbstractionRealizationStatus::Ambiguous
    );
}

#[test]
fn induced_premise_class_preserves_all_concrete_witnesses() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(full_grid_source());

    let consensus = bridge.consensus().unwrap();

    let premise_class = &consensus.premise_classes()[0];

    assert_eq!(premise_class.members(), &[unit(1), unit(2),]);

    assert_eq!(
        bridge.premise_witnesses(premise_class,),
        &[unit(1), unit(2),]
    );
}

#[test]
fn induced_conclusion_class_preserves_all_concrete_witnesses() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(full_grid_source());

    let consensus = bridge.consensus().unwrap();

    let conclusion_class = &consensus.conclusion_classes()[0];

    assert_eq!(conclusion_class.members(), &[unit(10), unit(20),]);

    assert_eq!(
        bridge.conclusion_witnesses(conclusion_class,),
        &[unit(10), unit(20),]
    );
}

#[test]
fn realization_preserves_induced_consensus_identity() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(full_grid_source());

    assert_eq!(
        bridge.realization().unwrap().consensus(),
        bridge.consensus().unwrap()
    );
}

#[test]
fn realization_preserves_source_observation_identity() {
    let source = full_grid_source();

    let before = source.clone();

    let bridge = RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(source);

    assert_eq!(bridge.source_observations(), &before);

    assert_eq!(bridge.realization().unwrap().source_observations(), &before);
}

#[test]
fn realization_preserves_induced_vocabulary_identity() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(full_grid_source());

    assert_eq!(
        bridge.realization().unwrap().vocabulary(),
        bridge.vocabulary().unwrap()
    );
}

#[test]
fn realization_preserves_consensus_support_identity() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(full_grid_source());

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
fn vocabulary_conflict_never_reaches_realization() {
    let bridge = RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(
        observation_set(vec![
            observation(&[1, 5], &[10]),
            observation(&[2, 5], &[10]),
            observation(&[2, 6], &[11]),
            observation(&[3, 6], &[11]),
        ]),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionRealizationStatus::ConsensusUnavailable
    );

    assert!(!bridge.conflicts().is_empty());

    assert!(bridge.realization().is_none());
}

#[test]
fn induction_realizer_facade_matches_direct_realization() {
    let source = full_grid_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionInductionRealizer::realize(source.clone(),),
        RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(source,)
    );
}

#[test]
fn induced_realization_is_canonical_deterministic_and_non_mutating() {
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

    let left = RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(source.clone());

    let right = RecursiveWorldRevisionAbstractionInductionRealizationBridge::realize(
        observation_set(vec![second, third, first, fourth]),
    );

    assert_eq!(left, right);

    assert_eq!(source, before);

    assert!(left.is_ambiguous());

    assert!(left.realized_observation().is_none());
}
