use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_generalization::{
    RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge,
    RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus,
    RecursiveWorldRevisionAbstractionGeneralizationProjector,
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

fn resolved_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
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

fn application_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 70], &[10, 80]),
        observation(&[2, 71], &[20, 81]),
    ])
}

#[test]
fn conflicted_generalized_vocabulary_blocks_projection() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        conflicted_source(),
        application_source(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus::VocabularyUnavailable
    );

    assert!(bridge.projection().is_none());

    assert!(bridge.vocabulary().is_none());

    assert!(!bridge.conflicts().is_empty());
}

#[test]
fn resolved_generalized_vocabulary_projects_application_observations() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        resolved_source(),
        application_source(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus::Projected
    );

    assert!(bridge.is_projected());

    assert!(bridge.projection().is_some());
}

#[test]
fn missing_abstract_premise_coverage_blocks_projection() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        resolved_source(),
        observation_set(vec![observation(&[99], &[10]), observation(&[1], &[20])]),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus::ProjectionUnavailable
    );

    assert!(bridge.projection().is_none());

    assert!(bridge.vocabulary().is_some());
}

#[test]
fn missing_abstract_conclusion_coverage_blocks_projection() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        resolved_source(),
        observation_set(vec![observation(&[1], &[99]), observation(&[2], &[10])]),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionGeneralizationProjectionStatus::ProjectionUnavailable
    );

    assert!(bridge.projection().is_none());
}

#[test]
fn projection_bridge_preserves_generalized_source_identity() {
    let source = resolved_source();

    let before = source.clone();

    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        source,
        application_source(),
    );

    assert_eq!(bridge.generalized_source(), &before);

    assert_eq!(bridge.resolution().source(), &before);
}

#[test]
fn projection_bridge_preserves_application_observation_provenance() {
    let application = application_source();

    let before = application.clone();

    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        resolved_source(),
        application,
    );

    assert_eq!(bridge.application_observations(), &before);

    assert_eq!(bridge.projection().unwrap().source_observations(), &before);
}

#[test]
fn projection_bridge_preserves_resolved_vocabulary_identity() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        resolved_source(),
        application_source(),
    );

    let vocabulary = bridge.vocabulary().unwrap();

    assert_eq!(vocabulary.classes().len(), 2);

    assert!(vocabulary.covers(&unit(1,),));

    assert!(vocabulary.covers(&unit(10,),));

    assert_eq!(bridge.projection().unwrap().vocabulary(), vocabulary);
}

#[test]
fn projected_application_observation_count_is_preserved() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        resolved_source(),
        application_source(),
    );

    assert_eq!(bridge.projection().unwrap().len(), 2);

    assert!(!bridge.projection().unwrap().is_empty());
}

#[test]
fn different_concrete_premises_share_generalized_abstract_class() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        resolved_source(),
        application_source(),
    );

    let projection = bridge.projection().unwrap();

    let first = projection
        .abstract_observation_for(&observation(&[1, 70], &[10, 80]))
        .unwrap();

    let second = projection
        .abstract_observation_for(&observation(&[2, 71], &[20, 81]))
        .unwrap();

    assert_eq!(first.premise_classes(), second.premise_classes());

    assert_eq!(first.premise_classes().len(), 1);
}

#[test]
fn different_concrete_conclusions_share_generalized_abstract_class() {
    let bridge = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        resolved_source(),
        application_source(),
    );

    let projection = bridge.projection().unwrap();

    let first = projection
        .abstract_observation_for(&observation(&[1, 70], &[10, 80]))
        .unwrap();

    let second = projection
        .abstract_observation_for(&observation(&[2, 71], &[20, 81]))
        .unwrap();

    assert_eq!(first.conclusion_classes(), second.conclusion_classes());

    assert_eq!(first.conclusion_classes().len(), 1);
}

#[test]
fn generalized_projector_facade_matches_direct_projection() {
    let source = resolved_source();

    let application = application_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionGeneralizationProjector::project(
            source.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
            source,
            application,
        )
    );
}

#[test]
fn generalized_projection_is_canonical_deterministic_and_non_mutating() {
    let source = resolved_source();

    let application = application_source();

    let source_before = source.clone();

    let application_before = application.clone();

    let left = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
        source.clone(),
        application.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionGeneralizationProjectionBridge::project(
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

    assert!(left.is_projected());
}
