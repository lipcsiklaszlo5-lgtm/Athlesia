use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_induction::{
    RecursiveWorldRevisionInducedStructure, RecursiveWorldRevisionInducer,
    RecursiveWorldRevisionInductionInput, RecursiveWorldRevisionInductionObservationSet,
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

#[test]
fn induction_requires_multiple_distinct_observations() {
    let single = observation(&[1], &[2]);

    assert!(RecursiveWorldRevisionInductionObservationSet::new(vec![single,],).is_none());
}

#[test]
fn duplicate_only_observations_do_not_satisfy_support_requirement() {
    let observed = observation(&[1], &[2]);

    assert!(
        RecursiveWorldRevisionInductionObservationSet::new(vec![observed.clone(), observed,],)
            .is_none()
    );
}

#[test]
fn observation_set_is_canonical_and_deduplicated() {
    let first = observation(&[1], &[2]);

    let second = observation(&[3], &[4]);

    let set = RecursiveWorldRevisionInductionObservationSet::new(vec![
        second.clone(),
        first.clone(),
        second,
    ])
    .unwrap();

    assert_eq!(set.len(), 2);

    assert!(!set.is_empty());

    assert!(set.contains(&first,));
}

#[test]
fn induction_input_preserves_target_identity() {
    let target = rule(&[1], &[2]);

    let input = RecursiveWorldRevisionInductionInput::new(
        target.clone(),
        observation_set(vec![
            observation(&[1, 3], &[4, 5]),
            observation(&[1, 6], &[4, 7]),
        ]),
    );

    assert_eq!(input.target(), &target);
}

#[test]
fn induction_discovers_common_premise_structure() {
    let induced =
        RecursiveWorldRevisionInducedStructure::induce(RecursiveWorldRevisionInductionInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1, 2, 3], &[4]),
                observation(&[1, 3, 5], &[4]),
            ]),
        ))
        .unwrap();

    assert_eq!(induced.induced_premises(), &[unit(1,), unit(3,),]);
}

#[test]
fn induction_discovers_common_conclusion_structure() {
    let induced =
        RecursiveWorldRevisionInducedStructure::induce(RecursiveWorldRevisionInductionInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1], &[2, 3, 4]),
                observation(&[1], &[2, 4, 5]),
            ]),
        ))
        .unwrap();

    assert_eq!(induced.induced_conclusions(), &[unit(2,), unit(4,),]);
}

#[test]
fn induction_requires_nonempty_common_premises() {
    let result =
        RecursiveWorldRevisionInducedStructure::induce(RecursiveWorldRevisionInductionInput::new(
            rule(&[9], &[10]),
            observation_set(vec![observation(&[1], &[4]), observation(&[2], &[4])]),
        ));

    assert!(result.is_none());
}

#[test]
fn induction_requires_nonempty_common_conclusions() {
    let result =
        RecursiveWorldRevisionInducedStructure::induce(RecursiveWorldRevisionInductionInput::new(
            rule(&[9], &[10]),
            observation_set(vec![observation(&[1], &[4]), observation(&[1], &[5])]),
        ));

    assert!(result.is_none());
}

#[test]
fn induced_observation_materializes_exact_common_structure() {
    let induced =
        RecursiveWorldRevisionInducedStructure::induce(RecursiveWorldRevisionInductionInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1, 2], &[3, 4]),
                observation(&[1, 5], &[3, 6]),
            ]),
        ))
        .unwrap();

    assert_eq!(induced.induced_observation(), &observation(&[1], &[3],));
}

#[test]
fn induction_support_count_tracks_distinct_observations() {
    let induced =
        RecursiveWorldRevisionInducedStructure::induce(RecursiveWorldRevisionInductionInput::new(
            rule(&[9], &[10]),
            observation_set(vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 4], &[3]),
                observation(&[1, 5], &[3]),
            ]),
        ))
        .unwrap();

    assert_eq!(induced.support_count(), 3);
}

#[test]
fn inducer_facade_matches_direct_induction() {
    let input = RecursiveWorldRevisionInductionInput::new(
        rule(&[9], &[10]),
        observation_set(vec![
            observation(&[1, 2], &[3, 4]),
            observation(&[1, 5], &[3, 6]),
        ]),
    );

    assert_eq!(
        RecursiveWorldRevisionInducer::induce(input.clone(),),
        RecursiveWorldRevisionInducedStructure::induce(input,)
    );
}

#[test]
fn induction_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3, 4]);

    let second = observation(&[1, 5], &[3, 6]);

    let set = observation_set(vec![second.clone(), first.clone()]);

    let set_before = set.clone();

    let left = RecursiveWorldRevisionInducedStructure::induce(
        RecursiveWorldRevisionInductionInput::new(target.clone(), set.clone()),
    );

    let right = RecursiveWorldRevisionInducedStructure::induce(
        RecursiveWorldRevisionInductionInput::new(target, observation_set(vec![first, second])),
    );

    assert_eq!(left, right);

    assert_eq!(set, set_before);
}
