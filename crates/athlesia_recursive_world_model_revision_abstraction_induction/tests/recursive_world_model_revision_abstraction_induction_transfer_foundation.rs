use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionTransfer, RecursiveWorldRevisionAbstractionTransferEngine,
    RecursiveWorldRevisionAbstractionTransferStatus,
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
fn unavailable_induction_consensus_blocks_transfer() {
    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
        observation_set(vec![observation(&[1], &[10]), observation(&[2], &[20])]),
        deterministic_transfer_source(),
    );

    assert_eq!(
        transfer.status(),
        RecursiveWorldRevisionAbstractionTransferStatus::ConsensusUnavailable
    );

    assert!(transfer.consensus().is_none());

    assert!(transfer.realized_observation().is_none());
}

#[test]
fn separate_transfer_evidence_can_realize_learned_abstraction() {
    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        transfer.status(),
        RecursiveWorldRevisionAbstractionTransferStatus::Deterministic
    );

    assert!(transfer.is_deterministic());

    assert!(!transfer.is_ambiguous());
}

#[test]
fn deterministic_transfer_materializes_concrete_observation() {
    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        transfer.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );
}

#[test]
fn transfer_ignores_uncovered_concrete_noise() {
    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        transfer.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );

    assert!(transfer
        .vocabulary()
        .unwrap()
        .class_for(&unit(50),)
        .is_none());
}

#[test]
fn transfer_premise_witnesses_come_only_from_transfer_evidence() {
    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
        induction_source(),
        deterministic_transfer_source(),
    );

    let consensus = transfer.consensus().unwrap();

    let premise_class = &consensus.premise_classes()[0];

    assert_eq!(premise_class.members(), &[unit(1), unit(2),]);

    assert_eq!(
        transfer.premise_witnesses(premise_class,),
        std::slice::from_ref(&unit(1),)
    );
}

#[test]
fn transfer_conclusion_witnesses_come_only_from_transfer_evidence() {
    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
        induction_source(),
        deterministic_transfer_source(),
    );

    let consensus = transfer.consensus().unwrap();

    let conclusion_class = &consensus.conclusion_classes()[0];

    assert_eq!(conclusion_class.members(), &[unit(10), unit(20),]);

    assert_eq!(
        transfer.conclusion_witnesses(conclusion_class,),
        std::slice::from_ref(&unit(10),)
    );
}

#[test]
fn ambiguous_transfer_premise_blocks_concrete_realization() {
    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
        induction_source(),
        observation_set(vec![
            observation(&[1], &[10, 60]),
            observation(&[2], &[10, 61]),
        ]),
    );

    assert_eq!(
        transfer.status(),
        RecursiveWorldRevisionAbstractionTransferStatus::Ambiguous
    );

    assert!(transfer.realized_observation().is_none());
}

#[test]
fn ambiguous_transfer_conclusion_blocks_concrete_realization() {
    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
        induction_source(),
        observation_set(vec![
            observation(&[1, 50], &[10]),
            observation(&[1, 51], &[20]),
        ]),
    );

    assert_eq!(
        transfer.status(),
        RecursiveWorldRevisionAbstractionTransferStatus::Ambiguous
    );

    assert!(transfer.realized_observation().is_none());
}

#[test]
fn missing_transfer_witness_blocks_concrete_realization() {
    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(
        induction_source(),
        observation_set(vec![observation(&[50], &[10]), observation(&[51], &[10])]),
    );

    assert_eq!(
        transfer.status(),
        RecursiveWorldRevisionAbstractionTransferStatus::Ambiguous
    );

    let consensus = transfer.consensus().unwrap();

    assert!(transfer
        .premise_witnesses(&consensus.premise_classes()[0],)
        .is_empty());

    assert!(transfer.realized_observation().is_none());
}

#[test]
fn transfer_preserves_learning_and_application_provenance_separately() {
    let induction = induction_source();

    let application = deterministic_transfer_source();

    let induction_before = induction.clone();

    let application_before = application.clone();

    let transfer = RecursiveWorldRevisionAbstractionTransfer::transfer(induction, application);

    assert_eq!(transfer.induction_observations(), &induction_before);

    assert_eq!(transfer.transfer_observations(), &application_before);

    assert_ne!(
        transfer.induction_observations(),
        transfer.transfer_observations()
    );
}

#[test]
fn transfer_engine_facade_matches_direct_transfer() {
    let induction = induction_source();

    let application = deterministic_transfer_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionTransferEngine::transfer(
            induction.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionTransfer::transfer(induction, application,)
    );
}

#[test]
fn abstraction_transfer_is_canonical_deterministic_and_non_mutating() {
    let induction = induction_source();

    let application = deterministic_transfer_source();

    let induction_before = induction.clone();

    let application_before = application.clone();

    let left =
        RecursiveWorldRevisionAbstractionTransfer::transfer(induction.clone(), application.clone());

    let right = RecursiveWorldRevisionAbstractionTransfer::transfer(
        observation_set(vec![
            observation(&[2], &[20]),
            observation(&[1], &[10]),
            observation(&[2], &[10]),
            observation(&[1], &[20]),
        ]),
        observation_set(vec![
            observation(&[1, 51], &[10, 61]),
            observation(&[1, 50], &[10, 60]),
        ]),
    );

    assert_eq!(left, right);

    assert_eq!(induction, induction_before);

    assert_eq!(application, application_before);

    assert!(left.is_deterministic());
}
