use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionClassInducer,
    RecursiveWorldRevisionAbstractionInducedClassSet,
    RecursiveWorldRevisionAbstractionInductionSide,
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

fn discover(
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitnessSet {
    RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::discover(observation_set(observations))
        .unwrap()
}

fn witness(
    first: RecursiveWorldRevisionDiscoveryObservation,
    second: RecursiveWorldRevisionDiscoveryObservation,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(first, second).unwrap()
}

#[test]
fn two_member_premise_clique_induces_class() {
    let induced = RecursiveWorldRevisionAbstractionInducedClassSet::induce(discover(vec![
        observation(&[1, 5], &[10]),
        observation(&[2, 5], &[10]),
    ]))
    .unwrap();

    assert_eq!(induced.len(), 1);

    let class = &induced.classes()[0];

    assert_eq!(
        class.side(),
        RecursiveWorldRevisionAbstractionInductionSide::Premise
    );

    assert_eq!(class.abstraction_class().members(), &[unit(1), unit(2),]);
}

#[test]
fn three_member_complete_premise_clique_induces_single_class() {
    let induced = RecursiveWorldRevisionAbstractionInducedClassSet::induce(discover(vec![
        observation(&[1, 5], &[10]),
        observation(&[2, 5], &[10]),
        observation(&[3, 5], &[10]),
    ]))
    .unwrap();

    assert_eq!(induced.len(), 1);

    assert_eq!(
        induced.classes()[0].abstraction_class().members(),
        &[unit(1), unit(2), unit(3),]
    );

    assert_eq!(induced.classes()[0].witness_count(), 3);
}

#[test]
fn conclusion_clique_induces_conclusion_class() {
    let induced = RecursiveWorldRevisionAbstractionInducedClassSet::induce(discover(vec![
        observation(&[1], &[10, 30]),
        observation(&[1], &[20, 30]),
        observation(&[1], &[25, 30]),
    ]))
    .unwrap();

    assert_eq!(induced.conclusion_classes().len(), 1);

    assert_eq!(
        induced.classes()[0].abstraction_class().members(),
        &[unit(10), unit(20), unit(25),]
    );
}

#[test]
fn same_units_in_different_contexts_remain_separate_classes() {
    let first = witness(observation(&[1, 5], &[10]), observation(&[2, 5], &[10]));

    let second = witness(observation(&[1, 6], &[11]), observation(&[2, 6], &[11]));

    let set =
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(vec![second, first]).unwrap();

    let induced = RecursiveWorldRevisionAbstractionInducedClassSet::induce(set).unwrap();

    assert_eq!(induced.len(), 2);

    assert_eq!(
        induced.classes()[0].abstraction_class(),
        induced.classes()[1].abstraction_class()
    );

    assert_ne!(
        induced.classes()[0].context(),
        induced.classes()[1].context()
    );
}

#[test]
fn transitive_chain_across_different_contexts_does_not_merge() {
    let first = witness(observation(&[1, 5], &[10]), observation(&[2, 5], &[10]));

    let second = witness(observation(&[2, 6], &[11]), observation(&[3, 6], &[11]));

    let induced = RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(vec![first, second]).unwrap(),
    )
    .unwrap();

    assert_eq!(induced.len(), 2);

    assert_eq!(induced.classes()[0].abstraction_class().len(), 2);

    assert_eq!(induced.classes()[1].abstraction_class().len(), 2);
}

#[test]
fn incomplete_same_context_triangle_is_not_induced() {
    let first = witness(observation(&[1, 5], &[10]), observation(&[2, 5], &[10]));

    let second = witness(observation(&[2, 5], &[10]), observation(&[3, 5], &[10]));

    let result = RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(vec![first, second]).unwrap(),
    );

    assert!(result.is_none());
}

#[test]
fn complete_same_context_triangle_requires_all_pairwise_witnesses() {
    let first = observation(&[1, 5], &[10]);

    let second = observation(&[2, 5], &[10]);

    let third = observation(&[3, 5], &[10]);

    let set = RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(vec![
        witness(first.clone(), second.clone()),
        witness(second.clone(), third.clone()),
        witness(first, third),
    ])
    .unwrap();

    let induced = RecursiveWorldRevisionAbstractionInducedClassSet::induce(set).unwrap();

    assert_eq!(induced.len(), 1);

    assert_eq!(induced.classes()[0].abstraction_class().len(), 3);
}

#[test]
fn induced_class_preserves_structural_context() {
    let induced = RecursiveWorldRevisionAbstractionInducedClassSet::induce(discover(vec![
        observation(&[1, 5, 6], &[10, 11]),
        observation(&[2, 5, 6], &[10, 11]),
    ]))
    .unwrap();

    let context = induced.classes()[0].context();

    assert_eq!(
        context.side(),
        RecursiveWorldRevisionAbstractionInductionSide::Premise
    );

    assert_eq!(context.shared_units(), &[unit(5), unit(6),]);

    assert_eq!(context.fixed_opposite_units(), &[unit(10), unit(11),]);
}

#[test]
fn induced_class_preserves_exact_witness_provenance() {
    let witnesses = discover(vec![
        observation(&[1, 5], &[10]),
        observation(&[2, 5], &[10]),
        observation(&[3, 5], &[10]),
    ]);

    let witness_count = witnesses.len();

    let induced = RecursiveWorldRevisionAbstractionInducedClassSet::induce(witnesses).unwrap();

    assert_eq!(induced.classes()[0].witness_count(), witness_count);

    assert_eq!(induced.classes()[0].witnesses().len(), 3);
}

#[test]
fn premise_and_conclusion_contexts_are_never_merged() {
    let premise = witness(observation(&[1, 5], &[10]), observation(&[2, 5], &[10]));

    let conclusion = witness(observation(&[30], &[1, 40]), observation(&[30], &[2, 40]));

    let induced = RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(vec![premise, conclusion])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(induced.len(), 2);

    assert_eq!(induced.premise_classes().len(), 1);

    assert_eq!(induced.conclusion_classes().len(), 1);
}

#[test]
fn class_inducer_facade_matches_direct_induction() {
    let witnesses = discover(vec![
        observation(&[1, 5], &[10]),
        observation(&[2, 5], &[10]),
        observation(&[3, 5], &[10]),
    ]);

    assert_eq!(
        RecursiveWorldRevisionAbstractionClassInducer::induce(witnesses.clone(),),
        RecursiveWorldRevisionAbstractionInducedClassSet::induce(witnesses,)
    );
}

#[test]
fn abstraction_class_induction_is_canonical_and_deterministic() {
    let first = witness(observation(&[1, 5], &[10]), observation(&[2, 5], &[10]));

    let second = witness(observation(&[1, 5], &[10]), observation(&[3, 5], &[10]));

    let third = witness(observation(&[2, 5], &[10]), observation(&[3, 5], &[10]));

    let left = RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(vec![
            third.clone(),
            first.clone(),
            second.clone(),
        ])
        .unwrap(),
    )
    .unwrap();

    let right = RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(vec![second, third, first])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(left, right);

    assert_eq!(
        left.classes()[0].abstraction_class().members(),
        &[unit(1), unit(2), unit(3),]
    );
}
