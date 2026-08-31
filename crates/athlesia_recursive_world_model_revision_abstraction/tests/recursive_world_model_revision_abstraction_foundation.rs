use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction::{
    RecursiveWorldRevisionAbstractObservation, RecursiveWorldRevisionAbstractionClass,
    RecursiveWorldRevisionAbstractionProjection, RecursiveWorldRevisionAbstractionProjector,
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

#[test]
fn abstraction_class_requires_two_distinct_members() {
    assert!(RecursiveWorldRevisionAbstractionClass::new(vec![unit(1,),],).is_none());

    assert!(RecursiveWorldRevisionAbstractionClass::new(vec![unit(1,), unit(1,),],).is_none());
}

#[test]
fn abstraction_class_is_canonical() {
    assert_eq!(
        RecursiveWorldRevisionAbstractionClass::new(vec![unit(3,), unit(1,), unit(2,), unit(1,),],)
            .unwrap()
            .members(),
        &[unit(1,), unit(2,), unit(3,),]
    );
}

#[test]
fn abstraction_vocabulary_requires_nonempty_classes() {
    assert!(RecursiveWorldRevisionAbstractionVocabulary::new(Vec::new(),).is_none());
}

#[test]
fn abstraction_vocabulary_rejects_overlapping_classes() {
    assert!(RecursiveWorldRevisionAbstractionVocabulary::new(vec![
        class(&[1, 2],),
        class(&[2, 3],),
    ],)
    .is_none());
}

#[test]
fn abstraction_vocabulary_maps_distinct_concrete_units_to_same_class() {
    let shared = class(&[1, 2]);

    let vocab = vocabulary(vec![shared.clone()]);

    assert_eq!(vocab.class_for(&unit(1,),), Some(&shared,));

    assert_eq!(vocab.class_for(&unit(2,),), Some(&shared,));
}

#[test]
fn abstract_observation_requires_premise_and_conclusion_classes() {
    let shared = class(&[1, 2]);

    assert!(
        RecursiveWorldRevisionAbstractObservation::new(Vec::new(), vec![shared.clone(),],)
            .is_none()
    );

    assert!(RecursiveWorldRevisionAbstractObservation::new(vec![shared,], Vec::new(),).is_none());
}

#[test]
fn projection_maps_different_concrete_premises_to_same_abstract_class() {
    let premise_class = class(&[1, 2]);

    let conclusion_class = class(&[10, 11]);

    let first = observation(&[1], &[10]);

    let second = observation(&[2], &[11]);

    let projection = RecursiveWorldRevisionAbstractionProjection::project(
        vocabulary(vec![premise_class.clone(), conclusion_class.clone()]),
        observation_set(vec![first.clone(), second.clone()]),
    )
    .unwrap();

    assert_eq!(
        projection
            .abstract_observation_for(&first,)
            .unwrap()
            .premise_classes(),
        std::slice::from_ref(&premise_class,)
    );

    assert_eq!(
        projection
            .abstract_observation_for(&second,)
            .unwrap()
            .premise_classes(),
        std::slice::from_ref(&premise_class,)
    );
}

#[test]
fn projection_maps_different_concrete_conclusions_to_same_abstract_class() {
    let premise_class = class(&[1, 2]);

    let conclusion_class = class(&[10, 11]);

    let first = observation(&[1], &[10]);

    let second = observation(&[2], &[11]);

    let projection = RecursiveWorldRevisionAbstractionProjection::project(
        vocabulary(vec![premise_class, conclusion_class.clone()]),
        observation_set(vec![first.clone(), second.clone()]),
    )
    .unwrap();

    assert_eq!(
        projection
            .abstract_observation_for(&first,)
            .unwrap()
            .conclusion_classes(),
        std::slice::from_ref(&conclusion_class,)
    );

    assert_eq!(
        projection
            .abstract_observation_for(&second,)
            .unwrap()
            .conclusion_classes(),
        std::slice::from_ref(&conclusion_class,)
    );
}

#[test]
fn projection_rejects_observation_without_abstract_premise_coverage() {
    let projection = RecursiveWorldRevisionAbstractionProjection::project(
        vocabulary(vec![class(&[10, 11])]),
        observation_set(vec![observation(&[1], &[10]), observation(&[2], &[11])]),
    );

    assert!(projection.is_none());
}

#[test]
fn projection_rejects_observation_without_abstract_conclusion_coverage() {
    let projection = RecursiveWorldRevisionAbstractionProjection::project(
        vocabulary(vec![class(&[1, 2])]),
        observation_set(vec![observation(&[1], &[10]), observation(&[2], &[11])]),
    );

    assert!(projection.is_none());
}

#[test]
fn abstraction_projector_facade_matches_direct_projection() {
    let vocab = vocabulary(vec![class(&[1, 2]), class(&[10, 11])]);

    let observations = observation_set(vec![observation(&[1], &[10]), observation(&[2], &[11])]);

    assert_eq!(
        RecursiveWorldRevisionAbstractionProjector::project(vocab.clone(), observations.clone(),),
        RecursiveWorldRevisionAbstractionProjection::project(vocab, observations,)
    );
}

#[test]
fn abstraction_projection_is_deterministic_and_preserves_provenance() {
    let first = observation(&[1], &[10]);

    let second = observation(&[2], &[11]);

    let vocab = vocabulary(vec![class(&[10, 11]), class(&[1, 2])]);

    let observations = observation_set(vec![second.clone(), first.clone()]);

    let observations_before = observations.clone();

    let left =
        RecursiveWorldRevisionAbstractionProjection::project(vocab.clone(), observations.clone())
            .unwrap();

    let right = RecursiveWorldRevisionAbstractionProjection::project(
        vocabulary(vec![class(&[1, 2]), class(&[10, 11])]),
        observation_set(vec![first.clone(), second.clone()]),
    )
    .unwrap();

    assert_eq!(left, right);

    assert_eq!(observations, observations_before);

    assert_eq!(left.len(), 2);

    assert_eq!(left.source_observations(), &observations_before);

    assert_eq!(left.represented_classes().len(), 2);
}
