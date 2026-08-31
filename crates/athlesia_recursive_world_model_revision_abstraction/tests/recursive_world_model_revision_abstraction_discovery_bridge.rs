use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_abstraction::{
    RecursiveWorldRevisionAbstractionClass, RecursiveWorldRevisionAbstractionConsensus,
    RecursiveWorldRevisionAbstractionDiscoveryBridge,
    RecursiveWorldRevisionAbstractionDiscoveryBridgeBuilder,
    RecursiveWorldRevisionAbstractionProjection, RecursiveWorldRevisionAbstractionRealization,
    RecursiveWorldRevisionAbstractionVocabulary,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

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

fn class(members: &[usize]) -> RecursiveWorldRevisionAbstractionClass {
    RecursiveWorldRevisionAbstractionClass::new(members.iter().copied().map(unit).collect())
        .unwrap()
}

fn vocabulary(
    classes: Vec<RecursiveWorldRevisionAbstractionClass>,
) -> RecursiveWorldRevisionAbstractionVocabulary {
    RecursiveWorldRevisionAbstractionVocabulary::new(classes).unwrap()
}

fn realization(
    classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionAbstractionRealization {
    let projection = RecursiveWorldRevisionAbstractionProjection::project(
        vocabulary(classes),
        observation_set(observations),
    )
    .unwrap();

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection).unwrap();

    RecursiveWorldRevisionAbstractionRealization::realize(consensus)
}

#[test]
fn discovery_bridge_rejects_ambiguous_realization() {
    let result = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        rule(&[9], &[10]),
        realization(
            vec![class(&[1, 2]), class(&[10, 11])],
            vec![observation(&[1], &[10]), observation(&[2], &[11])],
        ),
    );

    assert!(result.is_none());
}

#[test]
fn discovery_bridge_rejects_deterministic_noop() {
    let target = rule(&[1], &[10]);

    let result = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        target,
        realization(
            vec![class(&[1, 2]), class(&[10, 11])],
            vec![
                observation(&[1, 50], &[10, 60]),
                observation(&[1, 51], &[10, 61]),
            ],
        ),
    );

    assert!(result.is_none());
}

#[test]
fn deterministic_realization_creates_discovery_hypothesis() {
    let bridge = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        rule(&[9], &[10]),
        realization(
            vec![class(&[1, 2]), class(&[20, 21])],
            vec![
                observation(&[1, 50], &[20, 60]),
                observation(&[1, 51], &[20, 61]),
            ],
        ),
    )
    .unwrap();

    assert_eq!(
        bridge.hypothesis().observation(),
        &observation(&[1], &[20],)
    );
}

#[test]
fn discovery_bridge_preserves_target_identity() {
    let target = rule(&[9], &[10]);

    let bridge = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        target.clone(),
        realization(
            vec![class(&[1, 2]), class(&[20, 21])],
            vec![
                observation(&[1, 50], &[20, 60]),
                observation(&[1, 51], &[20, 61]),
            ],
        ),
    )
    .unwrap();

    assert_eq!(bridge.target(), &target);

    assert_eq!(bridge.hypothesis().target(), &target);
}

#[test]
fn discovery_bridge_preserves_realized_observation_identity() {
    let realized = realization(
        vec![class(&[1, 2]), class(&[20, 21])],
        vec![
            observation(&[1, 50], &[20, 60]),
            observation(&[1, 51], &[20, 61]),
        ],
    );

    let expected = realized.realized_observation().unwrap().clone();

    let bridge =
        RecursiveWorldRevisionAbstractionDiscoveryBridge::new(rule(&[9], &[10]), realized).unwrap();

    assert_eq!(bridge.realized_observation(), &expected);

    assert_eq!(bridge.hypothesis().observation(), &expected);
}

#[test]
fn discovery_bridge_materializes_replacement_from_realization() {
    let bridge = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        rule(&[9], &[10]),
        realization(
            vec![class(&[1, 2]), class(&[20, 21])],
            vec![
                observation(&[1, 50], &[20, 60]),
                observation(&[1, 51], &[20, 61]),
            ],
        ),
    )
    .unwrap();

    assert_eq!(bridge.replacement(), &rule(&[1], &[20],));
}

#[test]
fn discovery_bridge_preserves_realization_identity() {
    let realized = realization(
        vec![class(&[1, 2]), class(&[20, 21])],
        vec![
            observation(&[1, 50], &[20, 60]),
            observation(&[1, 51], &[20, 61]),
        ],
    );

    let before = realized.clone();

    let bridge =
        RecursiveWorldRevisionAbstractionDiscoveryBridge::new(rule(&[9], &[10]), realized).unwrap();

    assert_eq!(bridge.realization(), &before);
}

#[test]
fn discovery_bridge_preserves_source_observation_count_and_provenance() {
    let first = observation(&[1, 50], &[20, 60]);

    let second = observation(&[1, 51], &[20, 61]);

    let bridge = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        rule(&[9], &[10]),
        realization(
            vec![class(&[1, 2]), class(&[20, 21])],
            vec![second.clone(), first.clone()],
        ),
    )
    .unwrap();

    assert_eq!(bridge.observation_count(), 2);

    assert!(bridge.source_observations().contains(&first,));

    assert!(bridge.source_observations().contains(&second,));
}

#[test]
fn discovery_bridge_preserves_vocabulary_identity() {
    let premise = class(&[1, 2]);

    let conclusion = class(&[20, 21]);

    let bridge = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        rule(&[9], &[10]),
        realization(
            vec![premise.clone(), conclusion.clone()],
            vec![
                observation(&[1, 50], &[20, 60]),
                observation(&[1, 51], &[20, 61]),
            ],
        ),
    )
    .unwrap();

    assert_eq!(bridge.vocabulary().class_for(&unit(1,),), Some(&premise,));

    assert_eq!(
        bridge.vocabulary().class_for(&unit(20,),),
        Some(&conclusion,)
    );
}

#[test]
fn discovery_bridge_preserves_unique_witness_provenance() {
    let premise = class(&[1, 2]);

    let conclusion = class(&[20, 21]);

    let bridge = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        rule(&[9], &[10]),
        realization(
            vec![premise.clone(), conclusion.clone()],
            vec![
                observation(&[1, 50], &[20, 60]),
                observation(&[1, 51], &[20, 61]),
            ],
        ),
    )
    .unwrap();

    assert_eq!(
        bridge.premise_witnesses(&premise,),
        std::slice::from_ref(&unit(1,),)
    );

    assert_eq!(
        bridge.conclusion_witnesses(&conclusion,),
        std::slice::from_ref(&unit(20,),)
    );
}

#[test]
fn discovery_bridge_builder_matches_direct_construction() {
    let realized = realization(
        vec![class(&[1, 2]), class(&[20, 21])],
        vec![
            observation(&[1, 50], &[20, 60]),
            observation(&[1, 51], &[20, 61]),
        ],
    );

    let target = rule(&[9], &[10]);

    assert_eq!(
        RecursiveWorldRevisionAbstractionDiscoveryBridgeBuilder::build(
            target.clone(),
            realized.clone(),
        ),
        RecursiveWorldRevisionAbstractionDiscoveryBridge::new(target, realized,)
    );
}

#[test]
fn abstraction_discovery_bridge_is_deterministic_and_non_mutating() {
    let first = observation(&[1, 50], &[20, 60]);

    let second = observation(&[1, 51], &[20, 61]);

    let target = rule(&[9], &[10]);

    let left_realization = realization(
        vec![class(&[20, 21]), class(&[1, 2])],
        vec![second.clone(), first.clone()],
    );

    let before = left_realization.clone();

    let left = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        target.clone(),
        left_realization.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionDiscoveryBridge::new(
        target,
        realization(vec![class(&[1, 2]), class(&[20, 21])], vec![first, second]),
    );

    assert_eq!(left, right);

    assert_eq!(left_realization, before);
}
