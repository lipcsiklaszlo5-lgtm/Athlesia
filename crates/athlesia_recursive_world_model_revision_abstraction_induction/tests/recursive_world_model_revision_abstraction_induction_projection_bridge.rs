use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInductionProjectionBridge,
    RecursiveWorldRevisionAbstractionInductionProjectionStatus,
    RecursiveWorldRevisionAbstractionInductionProjector,
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

fn fully_projectable_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1], &[10]),
        observation(&[2], &[10]),
        observation(&[1], &[20]),
    ])
}

#[test]
fn no_substitution_evidence_blocks_projection_bridge() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(observation_set(vec![
            observation(&[1], &[10]),
            observation(&[2], &[20]),
        ]));

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionProjectionStatus::SubstitutionUnavailable
    );

    assert!(bridge.witness_set().is_none());

    assert!(bridge.projection().is_none());
}

#[test]
fn premise_only_induction_cannot_satisfy_m40_projection_coverage() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(observation_set(vec![
            observation(&[1], &[10]),
            observation(&[2], &[10]),
        ]));

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionProjectionStatus::ProjectionUnavailable
    );

    assert!(bridge.vocabulary().is_some());

    assert!(bridge.projection().is_none());
}

#[test]
fn premise_and_conclusion_substitutions_enable_projection() {
    let bridge = RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(
        fully_projectable_source(),
    );

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionProjectionStatus::Projected
    );

    assert!(bridge.is_projected());

    assert!(bridge.projection().is_some());
}

#[test]
fn automatic_projection_uses_resolved_induced_vocabulary() {
    let bridge = RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(
        fully_projectable_source(),
    );

    let vocabulary = bridge.vocabulary().unwrap();

    assert_eq!(vocabulary.classes().len(), 2);

    assert!(vocabulary.covers(&unit(1),));

    assert!(vocabulary.covers(&unit(2),));

    assert!(vocabulary.covers(&unit(10),));

    assert!(vocabulary.covers(&unit(20),));

    assert_eq!(bridge.projection().unwrap().vocabulary(), vocabulary);
}

#[test]
fn projection_bridge_preserves_exact_source_observations() {
    let source = fully_projectable_source();

    let before = source.clone();

    let bridge = RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(source);

    assert_eq!(bridge.source_observations(), &before);

    assert_eq!(bridge.projection().unwrap().source_observations(), &before);
}

#[test]
fn projection_bridge_preserves_substitution_witness_provenance() {
    let bridge = RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(
        fully_projectable_source(),
    );

    let witness_set = bridge.witness_set().unwrap();

    assert_eq!(witness_set.len(), 2);

    assert_eq!(witness_set.premise_witnesses().len(), 1);

    assert_eq!(witness_set.conclusion_witnesses().len(), 1);
}

#[test]
fn projection_bridge_preserves_induced_class_provenance() {
    let bridge = RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(
        fully_projectable_source(),
    );

    let induced = bridge.induced_classes().unwrap();

    assert_eq!(induced.len(), 2);

    assert_eq!(induced.premise_classes().len(), 1);

    assert_eq!(induced.conclusion_classes().len(), 1);
}

#[test]
fn projection_bridge_preserves_vocabulary_resolution_identity() {
    let bridge = RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(
        fully_projectable_source(),
    );

    let resolution = bridge.resolution().unwrap();

    assert_eq!(resolution.resolved_count(), 2);

    assert_eq!(resolution.conflicted_count(), 0);

    assert!(!resolution.has_conflicts());
}

#[test]
fn overlapping_induced_classes_make_vocabulary_unavailable() {
    let bridge =
        RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(observation_set(vec![
            observation(&[1, 5], &[10]),
            observation(&[2, 5], &[10]),
            observation(&[2, 6], &[11]),
            observation(&[3, 6], &[11]),
        ]));

    assert_eq!(
        bridge.status(),
        RecursiveWorldRevisionAbstractionInductionProjectionStatus::VocabularyUnavailable
    );

    assert!(bridge.vocabulary().is_none());

    assert!(!bridge.conflicts().is_empty());

    assert!(bridge.projection().is_none());
}

#[test]
fn successful_projection_materializes_all_source_observations() {
    let bridge = RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(
        fully_projectable_source(),
    );

    let projection = bridge.projection().unwrap();

    assert_eq!(projection.len(), 3);

    assert!(!projection.is_empty());

    assert_eq!(projection.projected().len(), 3);
}

#[test]
fn induction_projector_facade_matches_direct_projection() {
    let source = fully_projectable_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionInductionProjector::project(source.clone(),),
        RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(source,)
    );
}

#[test]
fn induction_projection_is_canonical_deterministic_and_non_mutating() {
    let first = observation(&[1], &[10]);

    let second = observation(&[2], &[10]);

    let third = observation(&[1], &[20]);

    let left_source = observation_set(vec![third.clone(), first.clone(), second.clone()]);

    let left_before = left_source.clone();

    let left =
        RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(left_source.clone());

    let right =
        RecursiveWorldRevisionAbstractionInductionProjectionBridge::project(observation_set(vec![
            second, third, first,
        ]));

    assert_eq!(left, right);

    assert_eq!(left_source, left_before);

    assert!(left.is_projected());
}
