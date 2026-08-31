use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_induction::{
    RecursiveWorldRevisionInducedStructure, RecursiveWorldRevisionInductionDiscoveryBridge,
    RecursiveWorldRevisionInductionDiscoveryBridgeBuilder, RecursiveWorldRevisionInductionInput,
    RecursiveWorldRevisionInductionObservationSet,
};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
    )
}

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(structural(span)))
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

fn induced(
    target: RecursiveWorldRule,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionInducedStructure {
    RecursiveWorldRevisionInducedStructure::induce(RecursiveWorldRevisionInductionInput::new(
        target,
        observation_set(observations),
    ))
    .unwrap()
}

#[test]
fn bridge_rejects_induction_that_rediscovers_target_exactly() {
    let target = rule(&[1], &[2]);

    let structure = induced(
        target.clone(),
        vec![observation(&[1, 3], &[2, 4]), observation(&[1, 5], &[2, 6])],
    );

    assert_eq!(structure.induced_observation(), &observation(&[1], &[2],));

    assert!(RecursiveWorldRevisionInductionDiscoveryBridge::new(structure,).is_none());
}

#[test]
fn induced_structure_materializes_discovery_hypothesis() {
    let target = rule(&[9], &[10]);

    let bridge = RecursiveWorldRevisionInductionDiscoveryBridge::new(induced(
        target,
        vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
    ))
    .unwrap();

    assert_eq!(bridge.hypothesis().observation(), &observation(&[1], &[3],));
}

#[test]
fn bridge_preserves_induction_target_identity() {
    let target = rule(&[9], &[10]);

    let bridge = RecursiveWorldRevisionInductionDiscoveryBridge::new(induced(
        target.clone(),
        vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
    ))
    .unwrap();

    assert_eq!(bridge.target(), &target);

    assert_eq!(bridge.hypothesis().target(), &target);
}

#[test]
fn bridge_preserves_induced_observation_identity() {
    let structure = induced(
        rule(&[9], &[10]),
        vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
    );

    let expected = structure.induced_observation().clone();

    let bridge = RecursiveWorldRevisionInductionDiscoveryBridge::new(structure).unwrap();

    assert_eq!(bridge.hypothesis().observation(), &expected);
}

#[test]
fn bridge_materializes_replacement_from_induced_common_structure() {
    let bridge = RecursiveWorldRevisionInductionDiscoveryBridge::new(induced(
        rule(&[9], &[10]),
        vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
    ))
    .unwrap();

    assert_eq!(bridge.replacement(), &rule(&[1], &[3],));
}

#[test]
fn bridge_preserves_support_count() {
    let bridge = RecursiveWorldRevisionInductionDiscoveryBridge::new(induced(
        rule(&[9], &[10]),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 4], &[3]),
            observation(&[1, 5], &[3]),
        ],
    ))
    .unwrap();

    assert_eq!(bridge.support_count(), 3);
}

#[test]
fn bridge_preserves_source_observation_provenance() {
    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 4], &[3]);

    let bridge = RecursiveWorldRevisionInductionDiscoveryBridge::new(induced(
        rule(&[9], &[10]),
        vec![first.clone(), second.clone()],
    ))
    .unwrap();

    assert!(bridge.source_observations().contains(&first,));

    assert!(bridge.source_observations().contains(&second,));
}

#[test]
fn bridge_preserves_common_premise_structure() {
    let bridge = RecursiveWorldRevisionInductionDiscoveryBridge::new(induced(
        rule(&[9], &[10]),
        vec![observation(&[1, 2, 3], &[4]), observation(&[1, 3, 5], &[4])],
    ))
    .unwrap();

    assert_eq!(
        bridge.hypothesis().observation().premises(),
        &[unit(1,), unit(3,),]
    );
}

#[test]
fn bridge_preserves_common_conclusion_structure() {
    let bridge = RecursiveWorldRevisionInductionDiscoveryBridge::new(induced(
        rule(&[9], &[10]),
        vec![observation(&[1], &[2, 3, 4]), observation(&[1], &[2, 4, 5])],
    ))
    .unwrap();

    assert_eq!(
        bridge.hypothesis().observation().conclusions(),
        &[unit(2,), unit(4,),]
    );
}

#[test]
fn distinct_source_observations_remain_distinct_provenance() {
    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 4], &[3]);

    let third = observation(&[1, 5], &[3]);

    let bridge = RecursiveWorldRevisionInductionDiscoveryBridge::new(induced(
        rule(&[9], &[10]),
        vec![third, first, second],
    ))
    .unwrap();

    assert_eq!(bridge.source_observations().len(), 3);
}

#[test]
fn bridge_builder_matches_direct_construction() {
    let structure = induced(
        rule(&[9], &[10]),
        vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
    );

    assert_eq!(
        RecursiveWorldRevisionInductionDiscoveryBridgeBuilder::build(structure.clone(),),
        RecursiveWorldRevisionInductionDiscoveryBridge::new(structure,)
    );
}

#[test]
fn induction_discovery_bridge_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3, 4]);

    let second = observation(&[1, 5], &[3, 6]);

    let left_structure = induced(target.clone(), vec![second.clone(), first.clone()]);

    let before = left_structure.clone();

    let left = RecursiveWorldRevisionInductionDiscoveryBridge::new(left_structure.clone());

    let right =
        RecursiveWorldRevisionInductionDiscoveryBridge::new(induced(target, vec![first, second]));

    assert_eq!(left, right);

    assert_eq!(left_structure, before);
}
