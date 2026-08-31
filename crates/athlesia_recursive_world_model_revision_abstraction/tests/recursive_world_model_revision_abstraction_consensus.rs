use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::{
    RecursiveWorldRevisionAbstractionClass, RecursiveWorldRevisionAbstractionConsensus,
    RecursiveWorldRevisionAbstractionConsensusBuilder, RecursiveWorldRevisionAbstractionProjection,
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

fn projection(
    classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionAbstractionProjection {
    RecursiveWorldRevisionAbstractionProjection::project(
        vocabulary(classes),
        observation_set(observations),
    )
    .unwrap()
}

#[test]
fn consensus_recognizes_common_abstract_premise_across_distinct_concrete_units() {
    let premise_class = class(&[1, 2]);

    let conclusion_class = class(&[10, 11]);

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![premise_class.clone(), conclusion_class],
        vec![observation(&[1], &[10]), observation(&[2], &[11])],
    ))
    .unwrap();

    assert_eq!(
        consensus.premise_classes(),
        std::slice::from_ref(&premise_class,)
    );
}

#[test]
fn consensus_recognizes_common_abstract_conclusion_across_distinct_concrete_units() {
    let premise_class = class(&[1, 2]);

    let conclusion_class = class(&[10, 11]);

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![premise_class, conclusion_class.clone()],
        vec![observation(&[1], &[10]), observation(&[2], &[11])],
    ))
    .unwrap();

    assert_eq!(
        consensus.conclusion_classes(),
        std::slice::from_ref(&conclusion_class,)
    );
}

#[test]
fn consensus_drops_nonuniversal_abstract_premise() {
    let common = class(&[1, 2, 3]);

    let optional = class(&[4, 5]);

    let conclusion = class(&[10, 11, 12]);

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![common.clone(), optional.clone(), conclusion],
        vec![
            observation(&[1, 4], &[10]),
            observation(&[2], &[11]),
            observation(&[3], &[12]),
        ],
    ))
    .unwrap();

    assert_eq!(consensus.premise_classes(), std::slice::from_ref(&common,));

    assert_eq!(consensus.premise_support(&optional,), 1);
}

#[test]
fn consensus_drops_nonuniversal_abstract_conclusion() {
    let premise = class(&[1, 2, 3]);

    let common = class(&[10, 11, 12]);

    let optional = class(&[20, 21]);

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![premise, common.clone(), optional.clone()],
        vec![
            observation(&[1], &[10, 20]),
            observation(&[2], &[11]),
            observation(&[3], &[12]),
        ],
    ))
    .unwrap();

    assert_eq!(
        consensus.conclusion_classes(),
        std::slice::from_ref(&common,)
    );

    assert_eq!(consensus.conclusion_support(&optional,), 1);
}

#[test]
fn consensus_requires_common_abstract_premise() {
    let result = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![class(&[1, 2]), class(&[3, 4]), class(&[10, 11])],
        vec![observation(&[1], &[10]), observation(&[3], &[11])],
    ));

    assert!(result.is_none());
}

#[test]
fn consensus_requires_common_abstract_conclusion() {
    let result = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![class(&[1, 2]), class(&[10, 11]), class(&[12, 13])],
        vec![observation(&[1], &[10]), observation(&[2], &[12])],
    ));

    assert!(result.is_none());
}

#[test]
fn consensus_exposes_exact_abstract_class_support_counts() {
    let premise = class(&[1, 2, 3]);

    let optional_premise = class(&[4, 5]);

    let conclusion = class(&[10, 11, 12]);

    let optional_conclusion = class(&[20, 21]);

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![
            premise.clone(),
            optional_premise.clone(),
            conclusion.clone(),
            optional_conclusion.clone(),
        ],
        vec![
            observation(&[1, 4], &[10, 20]),
            observation(&[2], &[11]),
            observation(&[3], &[12]),
        ],
    ))
    .unwrap();

    assert_eq!(consensus.premise_support(&premise,), 3);

    assert_eq!(consensus.premise_support(&optional_premise,), 1);

    assert_eq!(consensus.conclusion_support(&conclusion,), 3);

    assert_eq!(consensus.conclusion_support(&optional_conclusion,), 1);
}

#[test]
fn consensus_observation_count_matches_distinct_source_observations() {
    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![class(&[1, 2, 3]), class(&[10, 11, 12])],
        vec![
            observation(&[1], &[10]),
            observation(&[2], &[11]),
            observation(&[3], &[12]),
        ],
    ))
    .unwrap();

    assert_eq!(consensus.observation_count(), 3);
}

#[test]
fn consensus_preserves_source_observation_provenance() {
    let first = observation(&[1], &[10]);

    let second = observation(&[2], &[11]);

    let third = observation(&[3], &[12]);

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![class(&[1, 2, 3]), class(&[10, 11, 12])],
        vec![third.clone(), first.clone(), second.clone()],
    ))
    .unwrap();

    assert!(consensus.source_observations().contains(&first,));

    assert!(consensus.source_observations().contains(&second,));

    assert!(consensus.source_observations().contains(&third,));
}

#[test]
fn consensus_preserves_abstraction_vocabulary_identity() {
    let premise = class(&[1, 2]);

    let conclusion = class(&[10, 11]);

    let vocab = vocabulary(vec![premise.clone(), conclusion.clone()]);

    let projected = RecursiveWorldRevisionAbstractionProjection::project(
        vocab.clone(),
        observation_set(vec![observation(&[1], &[10]), observation(&[2], &[11])]),
    )
    .unwrap();

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projected).unwrap();

    assert_eq!(consensus.vocabulary(), &vocab);

    assert_eq!(
        consensus.vocabulary().class_for(&unit(1,),),
        Some(&premise,)
    );

    assert_eq!(
        consensus.vocabulary().class_for(&unit(10,),),
        Some(&conclusion,)
    );
}

#[test]
fn abstraction_consensus_builder_matches_direct_derivation() {
    let projected = projection(
        vec![class(&[1, 2]), class(&[10, 11])],
        vec![observation(&[1], &[10]), observation(&[2], &[11])],
    );

    assert_eq!(
        RecursiveWorldRevisionAbstractionConsensusBuilder::derive(projected.clone(),),
        RecursiveWorldRevisionAbstractionConsensus::derive(projected,)
    );
}

#[test]
fn abstraction_consensus_is_deterministic_and_non_mutating() {
    let first = observation(&[1, 4], &[10]);

    let second = observation(&[2], &[11]);

    let third = observation(&[3], &[12]);

    let left_projection = projection(
        vec![class(&[10, 11, 12]), class(&[4, 5]), class(&[1, 2, 3])],
        vec![third.clone(), first.clone(), second.clone()],
    );

    let before = left_projection.clone();

    let left = RecursiveWorldRevisionAbstractionConsensus::derive(left_projection.clone());

    let right = RecursiveWorldRevisionAbstractionConsensus::derive(projection(
        vec![class(&[1, 2, 3]), class(&[4, 5]), class(&[10, 11, 12])],
        vec![second, third, first],
    ));

    assert_eq!(left, right);

    assert_eq!(left_projection, before);
}
