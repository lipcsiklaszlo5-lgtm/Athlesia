use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInductionSide,
    RecursiveWorldRevisionAbstractionSubstitutionDiscoverer,
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

#[test]
fn identical_observations_do_not_create_substitution_witness() {
    let first = observation(&[1, 2], &[10]);

    assert!(
        RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(first.clone(), first,)
            .is_none()
    );
}

#[test]
fn one_premise_substitution_with_fixed_conclusion_is_discovered() {
    let witness = RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[1, 5], &[10]),
        observation(&[2, 5], &[10]),
    )
    .unwrap();

    assert_eq!(
        witness.side(),
        RecursiveWorldRevisionAbstractionInductionSide::Premise
    );

    assert_eq!(witness.first_unit(), &unit(1));

    assert_eq!(witness.second_unit(), &unit(2));
}

#[test]
fn one_conclusion_substitution_with_fixed_premise_is_discovered() {
    let witness = RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[1], &[10, 30]),
        observation(&[1], &[20, 30]),
    )
    .unwrap();

    assert_eq!(
        witness.side(),
        RecursiveWorldRevisionAbstractionInductionSide::Conclusion
    );

    assert_eq!(witness.first_unit(), &unit(10));

    assert_eq!(witness.second_unit(), &unit(20));
}

#[test]
fn changing_both_sides_does_not_create_witness() {
    assert!(
        RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
            observation(&[1], &[10],),
            observation(&[2], &[20],),
        )
        .is_none()
    );
}

#[test]
fn multiple_variable_changes_do_not_create_witness() {
    assert!(
        RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
            observation(&[1, 2], &[10],),
            observation(&[3, 4], &[10],),
        )
        .is_none()
    );
}

#[test]
fn premise_witness_preserves_shared_context() {
    let witness = RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[1, 5, 6], &[10, 11]),
        observation(&[2, 5, 6], &[10, 11]),
    )
    .unwrap();

    assert_eq!(witness.shared_units(), &[unit(5), unit(6),]);

    assert_eq!(witness.fixed_opposite_units(), &[unit(10), unit(11),]);
}

#[test]
fn conclusion_witness_preserves_shared_context() {
    let witness = RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[1, 2], &[10, 30]),
        observation(&[1, 2], &[20, 30]),
    )
    .unwrap();

    assert_eq!(witness.shared_units(), std::slice::from_ref(&unit(30),));

    assert_eq!(witness.fixed_opposite_units(), &[unit(1), unit(2),]);
}

#[test]
fn substitution_witness_materializes_two_member_abstraction_class() {
    let witness = RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[1], &[10]),
        observation(&[2], &[10]),
    )
    .unwrap();

    assert_eq!(witness.abstraction_class().members(), &[unit(1), unit(2),]);
}

#[test]
fn witness_set_discovers_all_pairwise_supported_substitutions() {
    let witnesses =
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::discover(observation_set(vec![
            observation(&[1], &[10]),
            observation(&[2], &[10]),
            observation(&[3], &[10]),
        ]))
        .unwrap();

    assert_eq!(witnesses.len(), 3);

    assert_eq!(witnesses.premise_witnesses().len(), 3);
}

#[test]
fn witness_set_returns_none_without_substitution_evidence() {
    assert!(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::discover(observation_set(vec![
            observation(&[1], &[10],),
            observation(&[2], &[20],),
        ],),)
        .is_none()
    );
}

#[test]
fn substitution_discoverer_facade_matches_direct_discovery() {
    let observations = observation_set(vec![observation(&[1], &[10]), observation(&[2], &[10])]);

    assert_eq!(
        RecursiveWorldRevisionAbstractionSubstitutionDiscoverer::discover(observations.clone(),),
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::discover(observations,)
    );
}

#[test]
fn abstraction_substitution_discovery_is_canonical_and_deterministic() {
    let first = observation(&[1, 5], &[10]);

    let second = observation(&[2, 5], &[10]);

    let third = observation(&[3, 5], &[10]);

    let left =
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::discover(observation_set(vec![
            third.clone(),
            first.clone(),
            second.clone(),
        ]))
        .unwrap();

    let right =
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::discover(observation_set(vec![
            second, third, first,
        ]))
        .unwrap();

    assert_eq!(left, right);

    assert_eq!(left.len(), 3);
}
