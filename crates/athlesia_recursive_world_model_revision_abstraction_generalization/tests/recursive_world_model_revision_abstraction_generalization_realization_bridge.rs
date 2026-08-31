use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_generalization::{
    RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge,
    RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus,
    RecursiveWorldRevisionAbstractionGeneralizationRealizer,
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

fn ambiguous_premise_application() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 70], &[10, 80]),
        observation(&[2, 71], &[10, 81]),
    ])
}

fn ambiguous_conclusion_application() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 70], &[10, 80]),
        observation(&[1, 71], &[20, 81]),
    ])
}

#[test]
fn unavailable_consensus_blocks_realization() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        conflicted_source(),
        deterministic_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus::ConsensusUnavailable
    );

    assert!(bridge.realization().is_none());

    assert!(bridge.realized_observation().is_none());
}

#[test]
fn unique_application_witnesses_realize_deterministically() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus::Deterministic
    );

    assert!(bridge.is_deterministic());

    assert!(!bridge.is_ambiguous());
}

#[test]
fn multiple_premise_witnesses_make_realization_ambiguous() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        generalized_source(),
        ambiguous_premise_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus::Ambiguous
    );

    assert!(bridge.is_ambiguous());

    assert!(bridge.realized_observation().is_none());
}

#[test]
fn multiple_conclusion_witnesses_make_realization_ambiguous() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        generalized_source(),
        ambiguous_conclusion_application(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationRealizationStatus::Ambiguous
    );

    assert!(bridge.is_ambiguous());

    assert!(bridge.realized_observation().is_none());
}

#[test]
fn deterministic_realization_materializes_application_observation() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        bridge.realized_observation(),
        Some(&observation(&[1,], &[10,],),)
    );
}

#[test]
fn deterministic_realization_tracks_exact_premise_witness() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        generalized_source(),
        deterministic_application(),
    );

    let consensus = bridge.consensus().unwrap();

    let premise_class = &consensus.premise_classes()[0];

    assert_eq!(bridge.premise_witnesses(premise_class,), &[unit(1,),]);
}

#[test]
fn deterministic_realization_tracks_exact_conclusion_witness() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        generalized_source(),
        deterministic_application(),
    );

    let consensus = bridge.consensus().unwrap();

    let conclusion_class = &consensus.conclusion_classes()[0];

    assert_eq!(
        bridge.conclusion_witnesses(conclusion_class,),
        &[unit(10,),]
    );
}

#[test]
fn ambiguous_realization_preserves_all_premise_witnesses() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        generalized_source(),
        ambiguous_premise_application(),
    );

    let consensus = bridge.consensus().unwrap();

    let premise_class = &consensus.premise_classes()[0];

    assert_eq!(
        bridge.premise_witnesses(premise_class,),
        &[unit(1,), unit(2,),]
    );
}

#[test]
fn realization_preserves_generalized_and_application_provenance() {
    let source = generalized_source();

    let application = deterministic_application();

    let source_before = source.clone();

    let application_before = application.clone();

    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        source,
        application,
    );

    assert_eq!(bridge.generalized_source(), &source_before);

    assert_eq!(bridge.application_observations(), &application_before);

    assert_eq!(
        bridge.consensus().unwrap().source_observations(),
        &application_before
    );
}

#[test]
fn realization_preserves_generalized_vocabulary_identity() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        bridge.realization().unwrap().vocabulary(),
        bridge.vocabulary().unwrap()
    );

    assert_eq!(bridge.vocabulary().unwrap().classes().len(), 2);
}

#[test]
fn generalized_realizer_facade_matches_direct_realization() {
    let source = generalized_source();

    let application = deterministic_application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionGeneralizationRealizer::realize(
            source.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
            source,
            application,
        )
    );
}

#[test]
fn generalized_realization_is_canonical_deterministic_and_non_mutating() {
    let source = generalized_source();

    let application = deterministic_application();

    let source_before = source.clone();

    let application_before = application.clone();

    let left = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
        source.clone(),
        application.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionGeneralizationRealizationBridge::realize(
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

    assert_eq!(source, source_before);

    assert_eq!(application, application_before);

    assert!(left.is_deterministic());

    assert_eq!(
        left.realized_observation(),
        Some(&observation(&[1,], &[10,],),)
    );
}
