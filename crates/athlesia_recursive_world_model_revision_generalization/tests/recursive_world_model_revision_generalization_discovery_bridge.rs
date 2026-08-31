use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_generalization::{
    RecursiveWorldRevisionGeneralizationDiscoveryBridge,
    RecursiveWorldRevisionGeneralizationDiscoveryBridgeBuilder,
    RecursiveWorldRevisionGeneralizationInput, RecursiveWorldRevisionGeneralizationThreshold,
    RecursiveWorldRevisionGeneralizedStructure,
};

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

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

fn threshold(
    minimum_support: usize,
    observation_count: usize,
) -> RecursiveWorldRevisionGeneralizationThreshold {
    RecursiveWorldRevisionGeneralizationThreshold::new(minimum_support, observation_count).unwrap()
}

fn generalized(
    target: RecursiveWorldRule,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    minimum_support: usize,
) -> RecursiveWorldRevisionGeneralizedStructure {
    let set = observation_set(observations);

    let count = set.len();

    RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            target,
            set,
            threshold(minimum_support, count),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn bridge_rejects_generalization_that_rediscovers_target_exactly() {
    let target = rule(&[1], &[2]);

    let structure = generalized(
        target.clone(),
        vec![
            observation(&[1, 3], &[2, 4]),
            observation(&[1, 5], &[2, 6]),
            observation(&[1, 7], &[2, 8]),
        ],
        3,
    );

    assert_eq!(
        structure.generalized_observation(),
        &observation(&[1], &[2],)
    );

    assert!(RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(structure,).is_none());
}

#[test]
fn generalized_structure_materializes_discovery_hypothesis() {
    let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized(
        rule(&[9], &[10]),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3, 5]),
            observation(&[1, 6], &[3]),
        ],
        2,
    ))
    .unwrap();

    assert_eq!(
        bridge.hypothesis().observation(),
        &observation(&[1, 2], &[3],)
    );
}

#[test]
fn bridge_preserves_target_identity() {
    let target = rule(&[9], &[10]);

    let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized(
        target.clone(),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3]),
            observation(&[1, 5], &[3]),
        ],
        2,
    ))
    .unwrap();

    assert_eq!(bridge.target(), &target);

    assert_eq!(bridge.hypothesis().target(), &target);
}

#[test]
fn bridge_preserves_generalized_observation_identity() {
    let structure = generalized(
        rule(&[9], &[10]),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3, 5]),
            observation(&[1, 6], &[3]),
        ],
        2,
    );

    let expected = structure.generalized_observation().clone();

    let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(structure).unwrap();

    assert_eq!(bridge.hypothesis().observation(), &expected);
}

#[test]
fn bridge_materializes_replacement_from_generalized_structure() {
    let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized(
        rule(&[9], &[10]),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3, 5]),
            observation(&[1, 6], &[3]),
        ],
        2,
    ))
    .unwrap();

    assert_eq!(bridge.replacement(), &rule(&[1, 2], &[3],));
}

#[test]
fn bridge_preserves_threshold_identity() {
    let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized(
        rule(&[9], &[10]),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3, 5]),
            observation(&[1, 6], &[3]),
        ],
        2,
    ))
    .unwrap();

    assert_eq!(bridge.threshold().minimum_support(), 2);
}

#[test]
fn bridge_preserves_distinct_observation_support_count() {
    let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized(
        rule(&[9], &[10]),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3, 5]),
            observation(&[1, 6], &[3]),
        ],
        2,
    ))
    .unwrap();

    assert_eq!(bridge.support_count(), 3);
}

#[test]
fn bridge_preserves_source_observation_provenance() {
    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 2, 4], &[3, 5]);

    let third = observation(&[1, 6], &[3]);

    let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized(
        rule(&[9], &[10]),
        vec![third.clone(), first.clone(), second.clone()],
        2,
    ))
    .unwrap();

    assert!(bridge.source_observations().contains(&first,));

    assert!(bridge.source_observations().contains(&second,));

    assert!(bridge.source_observations().contains(&third,));
}

#[test]
fn bridge_preserves_premise_support_counts() {
    let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized(
        rule(&[9], &[10]),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3]),
            observation(&[1, 5], &[3]),
        ],
        2,
    ))
    .unwrap();

    assert_eq!(bridge.premise_support(&unit(1,),), 3);

    assert_eq!(bridge.premise_support(&unit(2,),), 2);
}

#[test]
fn bridge_preserves_conclusion_support_counts() {
    let bridge = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized(
        rule(&[9], &[10]),
        vec![
            observation(&[1], &[2, 3]),
            observation(&[1, 4], &[2, 3, 5]),
            observation(&[1, 6], &[2, 7]),
        ],
        2,
    ))
    .unwrap();

    assert_eq!(bridge.conclusion_support(&unit(2,),), 3);

    assert_eq!(bridge.conclusion_support(&unit(3,),), 2);
}

#[test]
fn bridge_builder_matches_direct_construction() {
    let structure = generalized(
        rule(&[9], &[10]),
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3, 5]),
            observation(&[1, 6], &[3]),
        ],
        2,
    );

    assert_eq!(
        RecursiveWorldRevisionGeneralizationDiscoveryBridgeBuilder::build(structure.clone(),),
        RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(structure,)
    );
}

#[test]
fn generalization_discovery_bridge_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 2, 4], &[3, 5]);

    let third = observation(&[1, 6], &[3]);

    let left_structure = generalized(
        target.clone(),
        vec![third.clone(), first.clone(), second.clone()],
        2,
    );

    let before = left_structure.clone();

    let left = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(left_structure.clone());

    let right = RecursiveWorldRevisionGeneralizationDiscoveryBridge::new(generalized(
        target,
        vec![second, third, first],
        2,
    ));

    assert_eq!(left, right);

    assert_eq!(left_structure, before);
}
