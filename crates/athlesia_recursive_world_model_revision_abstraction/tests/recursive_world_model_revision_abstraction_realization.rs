use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::{
    RecursiveWorldRevisionAbstractionClass, RecursiveWorldRevisionAbstractionConsensus,
    RecursiveWorldRevisionAbstractionProjection, RecursiveWorldRevisionAbstractionRealization,
    RecursiveWorldRevisionAbstractionRealizationStatus, RecursiveWorldRevisionAbstractionRealizer,
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

fn consensus(
    classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionAbstractionConsensus {
    let projection = RecursiveWorldRevisionAbstractionProjection::project(
        vocabulary(classes),
        observation_set(observations),
    )
    .unwrap();

    RecursiveWorldRevisionAbstractionConsensus::derive(projection).unwrap()
}

#[test]
fn realization_marks_cross_concrete_premise_as_ambiguous() {
    let premise = class(&[1, 2]);

    let conclusion = class(&[10, 11]);

    let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus(
        vec![premise.clone(), conclusion],
        vec![observation(&[1], &[10]), observation(&[2], &[10])],
    ));

    assert_eq!(
        realization.status(),
        RecursiveWorldRevisionAbstractionRealizationStatus::Ambiguous
    );

    assert_eq!(
        realization.premise_witnesses(&premise,),
        &[unit(1,), unit(2,),]
    );
}

#[test]
fn realization_marks_cross_concrete_conclusion_as_ambiguous() {
    let premise = class(&[1, 2]);

    let conclusion = class(&[10, 11]);

    let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus(
        vec![premise, conclusion.clone()],
        vec![observation(&[1], &[10]), observation(&[1], &[11])],
    ));

    assert!(realization.is_ambiguous());

    assert_eq!(
        realization.conclusion_witnesses(&conclusion,),
        &[unit(10,), unit(11,),]
    );
}

#[test]
fn realization_is_deterministic_with_unique_premise_witness() {
    let premise = class(&[1, 2]);

    let conclusion = class(&[10, 11]);

    let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus(
        vec![premise.clone(), conclusion],
        vec![observation(&[1, 50], &[10]), observation(&[1, 51], &[10])],
    ));

    assert_eq!(
        realization.premise_witnesses(&premise,),
        std::slice::from_ref(&unit(1,),)
    );

    assert!(realization.is_deterministic());
}

#[test]
fn realization_is_deterministic_with_unique_conclusion_witness() {
    let premise = class(&[1, 2]);

    let conclusion = class(&[10, 11]);

    let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus(
        vec![premise, conclusion.clone()],
        vec![observation(&[1], &[10, 50]), observation(&[1], &[10, 51])],
    ));

    assert_eq!(
        realization.conclusion_witnesses(&conclusion,),
        std::slice::from_ref(&unit(10,),)
    );

    assert!(realization.is_deterministic());
}

#[test]
fn deterministic_realization_materializes_concrete_observation() {
    let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus(
        vec![class(&[1, 2]), class(&[10, 11])],
        vec![
            observation(&[1, 50], &[10, 60]),
            observation(&[1, 51], &[10, 61]),
        ],
    ));

    assert_eq!(
        realization.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );
}

#[test]
fn ambiguous_realization_does_not_materialize_observation() {
    let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus(
        vec![class(&[1, 2]), class(&[10, 11])],
        vec![observation(&[1], &[10]), observation(&[2], &[11])],
    ));

    assert!(realization.realized_observation().is_none());

    assert!(realization.is_ambiguous());
}

#[test]
fn realization_tracks_each_consensus_class_independently() {
    let first_premise = class(&[1, 2]);

    let second_premise = class(&[3, 4]);

    let conclusion = class(&[10, 11]);

    let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus(
        vec![first_premise.clone(), second_premise.clone(), conclusion],
        vec![
            observation(&[1, 3, 50], &[10]),
            observation(&[1, 3, 51], &[10]),
        ],
    ));

    assert_eq!(
        realization.premise_witnesses(&first_premise,),
        std::slice::from_ref(&unit(1,),)
    );

    assert_eq!(
        realization.premise_witnesses(&second_premise,),
        std::slice::from_ref(&unit(3,),)
    );

    assert_eq!(
        realization.realized_observation(),
        Some(&observation(&[1, 3], &[10],),)
    );
}

#[test]
fn realization_ignores_uncovered_concrete_noise() {
    let realization = RecursiveWorldRevisionAbstractionRealization::realize(consensus(
        vec![class(&[1, 2]), class(&[10, 11])],
        vec![
            observation(&[1, 90], &[10, 92]),
            observation(&[1, 91], &[10, 93]),
        ],
    ));

    assert_eq!(
        realization.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );
}

#[test]
fn realization_preserves_consensus_and_source_provenance() {
    let first = observation(&[1, 50], &[10]);

    let second = observation(&[1, 51], &[10]);

    let derived = consensus(
        vec![class(&[1, 2]), class(&[10, 11])],
        vec![second.clone(), first.clone()],
    );

    let before = derived.clone();

    let realization = RecursiveWorldRevisionAbstractionRealization::realize(derived);

    assert_eq!(realization.consensus(), &before);

    assert_eq!(realization.observation_count(), 2);

    assert!(realization.source_observations().contains(&first,));

    assert!(realization.source_observations().contains(&second,));
}

#[test]
fn realization_preserves_vocabulary_identity() {
    let premise = class(&[1, 2]);

    let conclusion = class(&[10, 11]);

    let derived = consensus(
        vec![premise.clone(), conclusion.clone()],
        vec![observation(&[1, 50], &[10]), observation(&[1, 51], &[10])],
    );

    let realization = RecursiveWorldRevisionAbstractionRealization::realize(derived);

    assert_eq!(
        realization.vocabulary().class_for(&unit(1,),),
        Some(&premise,)
    );

    assert_eq!(
        realization.vocabulary().class_for(&unit(10,),),
        Some(&conclusion,)
    );
}

#[test]
fn abstraction_realizer_facade_matches_direct_realization() {
    let derived = consensus(
        vec![class(&[1, 2]), class(&[10, 11])],
        vec![observation(&[1, 50], &[10]), observation(&[1, 51], &[10])],
    );

    assert_eq!(
        RecursiveWorldRevisionAbstractionRealizer::realize(derived.clone(),),
        RecursiveWorldRevisionAbstractionRealization::realize(derived,)
    );
}

#[test]
fn abstraction_realization_is_deterministic_and_non_mutating() {
    let first = observation(&[1, 50], &[10, 60]);

    let second = observation(&[1, 51], &[10, 61]);

    let left_consensus = consensus(
        vec![class(&[10, 11]), class(&[1, 2])],
        vec![second.clone(), first.clone()],
    );

    let before = left_consensus.clone();

    let left = RecursiveWorldRevisionAbstractionRealization::realize(left_consensus.clone());

    let right = RecursiveWorldRevisionAbstractionRealization::realize(consensus(
        vec![class(&[1, 2]), class(&[10, 11])],
        vec![first, second],
    ));

    assert_eq!(left, right);

    assert_eq!(left_consensus, before);
}
