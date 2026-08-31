use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_generalization::{
    RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge,
    RecursiveWorldRevisionAbstractionGeneralizationConsensusBuilder,
    RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus,
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

fn consensus_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
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

fn no_common_premise_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    generalized(vec![
        premise_witness(1, 2, 30, 40),
        premise_witness(1, 2, 31, 41),
        premise_witness(3, 4, 32, 42),
        premise_witness(3, 4, 33, 43),
        conclusion_witness(10, 20, 50, 60),
        conclusion_witness(10, 20, 51, 61),
    ])
}

fn consensus_application() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 70], &[10, 80]),
        observation(&[2, 71], &[20, 81]),
    ])
}

#[test]
fn unavailable_projection_blocks_consensus_derivation() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        conflicted_source(),
        consensus_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus::ProjectionUnavailable
    );

    assert!(bridge.consensus().is_none());

    assert!(bridge.projection().is_none());
}

#[test]
fn projected_observations_can_derive_generalized_consensus() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        consensus_source(),
        consensus_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus::ConsensusDerived
    );

    assert!(bridge.is_consensus_derived());

    assert!(bridge.consensus().is_some());
}

#[test]
fn projection_without_common_abstract_premise_has_no_consensus() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        no_common_premise_source(),
        observation_set(vec![observation(&[1], &[10]), observation(&[3], &[20])]),
    );

    assert!(bridge.projection().is_some());

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationConsensusStatus::ConsensusUnavailable
    );

    assert!(bridge.consensus().is_none());
}

#[test]
fn derived_consensus_contains_universal_generalized_premise_class() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        consensus_source(),
        consensus_application(),
    );

    let consensus = bridge.consensus().unwrap();

    assert_eq!(consensus.premise_classes().len(), 1);

    assert_eq!(
        consensus.premise_classes()[0].members(),
        &[unit(1,), unit(2,),]
    );
}

#[test]
fn derived_consensus_contains_universal_generalized_conclusion_class() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        consensus_source(),
        consensus_application(),
    );

    let consensus = bridge.consensus().unwrap();

    assert_eq!(consensus.conclusion_classes().len(), 1);

    assert_eq!(
        consensus.conclusion_classes()[0].members(),
        &[unit(10,), unit(20,),]
    );
}

#[test]
fn derived_consensus_support_matches_application_observation_count() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        consensus_source(),
        consensus_application(),
    );

    let consensus = bridge.consensus().unwrap();

    assert_eq!(consensus.observation_count(), 2);

    assert_eq!(
        consensus.premise_support(&consensus.premise_classes()[0],),
        2
    );

    assert_eq!(
        consensus.conclusion_support(&consensus.conclusion_classes()[0],),
        2
    );
}

#[test]
fn consensus_bridge_preserves_generalized_source_identity() {
    let source = consensus_source();

    let before = source.clone();

    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        source,
        consensus_application(),
    );

    assert_eq!(bridge.generalized_source(), &before);

    assert_eq!(bridge.resolution().source(), &before);
}

#[test]
fn consensus_bridge_preserves_application_provenance() {
    let application = consensus_application();

    let before = application.clone();

    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        consensus_source(),
        application,
    );

    assert_eq!(bridge.application_observations(), &before);

    assert_eq!(bridge.projection().unwrap().source_observations(), &before);

    assert_eq!(bridge.consensus().unwrap().source_observations(), &before);
}

#[test]
fn consensus_bridge_preserves_resolved_vocabulary_identity() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        consensus_source(),
        consensus_application(),
    );

    assert_eq!(
        bridge.consensus().unwrap().vocabulary(),
        bridge.vocabulary().unwrap()
    );

    assert_eq!(bridge.vocabulary().unwrap().classes().len(), 2);
}

#[test]
fn consensus_bridge_preserves_projection_identity() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        consensus_source(),
        consensus_application(),
    );

    assert_eq!(
        bridge.consensus().unwrap().projection(),
        bridge.projection().unwrap()
    );
}

#[test]
fn generalized_consensus_builder_matches_direct_derivation() {
    let source = consensus_source();

    let application = consensus_application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionGeneralizationConsensusBuilder::derive(
            source.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(source, application,)
    );
}

#[test]
fn generalized_consensus_is_canonical_deterministic_and_non_mutating() {
    let source = consensus_source();

    let application = consensus_application();

    let source_before = source.clone();

    let application_before = application.clone();

    let left = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        source.clone(),
        application.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionGeneralizationConsensusBridge::derive(
        generalized(vec![
            conclusion_witness(10, 20, 51, 61),
            premise_witness(1, 2, 31, 41),
            conclusion_witness(10, 20, 50, 60),
            premise_witness(1, 2, 30, 40),
        ]),
        observation_set(vec![
            observation(&[2, 71], &[20, 81]),
            observation(&[1, 70], &[10, 80]),
        ]),
    );

    assert_eq!(left, right);

    assert_eq!(source, source_before);

    assert_eq!(application, application_before);

    assert!(left.is_consensus_derived());
}
