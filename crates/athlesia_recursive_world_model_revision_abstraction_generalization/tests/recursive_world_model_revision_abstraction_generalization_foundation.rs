use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_generalization::{
    RecursiveWorldRevisionAbstractionGeneralizationThreshold,
    RecursiveWorldRevisionAbstractionGeneralizedClassSet,
    RecursiveWorldRevisionAbstractionGeneralizer,
};

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInducedClassSet,
    RecursiveWorldRevisionAbstractionInductionSide,
    RecursiveWorldRevisionAbstractionSubstitutionWitness,
    RecursiveWorldRevisionAbstractionSubstitutionWitnessSet,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

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

fn premise_witness(
    first: usize,
    second: usize,
    shared: usize,
    fixed_conclusion: usize,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[first, shared], &[fixed_conclusion]),
        observation(&[second, shared], &[fixed_conclusion]),
    )
    .unwrap()
}

fn conclusion_witness(
    first: usize,
    second: usize,
    shared: usize,
    fixed_premise: usize,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[fixed_premise], &[first, shared]),
        observation(&[fixed_premise], &[second, shared]),
    )
    .unwrap()
}

fn induced(
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
) -> RecursiveWorldRevisionAbstractionInducedClassSet {
    let witness_set =
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(witnesses).unwrap();

    RecursiveWorldRevisionAbstractionInducedClassSet::induce(witness_set).unwrap()
}

fn repeated_pair_source() -> RecursiveWorldRevisionAbstractionInducedClassSet {
    induced(vec![
        premise_witness(1, 2, 3, 5),
        premise_witness(1, 2, 4, 6),
    ])
}

fn repeated_three_member_clique_source() -> RecursiveWorldRevisionAbstractionInducedClassSet {
    induced(vec![
        premise_witness(1, 2, 4, 6),
        premise_witness(1, 3, 4, 6),
        premise_witness(2, 3, 4, 6),
        premise_witness(1, 2, 5, 7),
        premise_witness(1, 3, 5, 7),
        premise_witness(2, 3, 5, 7),
    ])
}

#[test]
fn generalization_threshold_rejects_zero() {
    assert!(RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(0,).is_none());
}

#[test]
fn generalization_threshold_requires_two_contexts() {
    assert!(RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(1,).is_none());

    assert_eq!(
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2,)
            .unwrap()
            .min_context_support(),
        2
    );
}

#[test]
fn repeated_exact_class_generalizes_across_contexts() {
    let generalized = RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        repeated_pair_source(),
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    assert_eq!(generalized.len(), 1);

    assert_eq!(
        generalized.classes()[0].abstraction_class().members(),
        &[unit(1,), unit(2,),]
    );
}

#[test]
fn generalized_pair_support_counts_distinct_contexts() {
    let generalized = RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        repeated_pair_source(),
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    assert_eq!(generalized.classes()[0].pair_supports().len(), 1);

    assert_eq!(
        generalized.classes()[0].pair_supports()[0].support_count(),
        2
    );

    assert_eq!(generalized.classes()[0].minimum_pair_support(), 2);
}

#[test]
fn threshold_above_available_context_support_rejects_class() {
    assert!(
        RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
            repeated_pair_source(),
            RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(3,).unwrap(),
        )
        .is_none()
    );
}

#[test]
fn complete_three_member_support_generalizes_whole_class() {
    let generalized = RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        repeated_three_member_clique_source(),
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    assert_eq!(generalized.len(), 1);

    assert_eq!(
        generalized.classes()[0].abstraction_class().members(),
        &[unit(1,), unit(2,), unit(3,),]
    );

    assert_eq!(generalized.classes()[0].pair_supports().len(), 3);

    assert!(generalized.classes()[0]
        .pair_supports()
        .iter()
        .all(|support| { support.support_count() == 2 },));
}

#[test]
fn incomplete_supported_chain_never_becomes_transitive_class() {
    let source = induced(vec![
        premise_witness(1, 2, 4, 6),
        premise_witness(1, 2, 5, 7),
        premise_witness(2, 3, 6, 4),
        premise_witness(2, 3, 7, 5),
    ]);

    assert!(
        RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
            source,
            RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2,).unwrap(),
        )
        .is_none()
    );
}

#[test]
fn premise_and_conclusion_support_never_merge() {
    let source = induced(vec![
        premise_witness(1, 2, 3, 5),
        premise_witness(1, 2, 4, 6),
        conclusion_witness(1, 2, 3, 5),
        conclusion_witness(1, 2, 4, 6),
    ]);

    let generalized = RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        source,
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    assert_eq!(generalized.len(), 2);

    assert_eq!(generalized.premise_classes().len(), 1);

    assert_eq!(generalized.conclusion_classes().len(), 1);

    assert_eq!(
        generalized.premise_classes()[0].side(),
        RecursiveWorldRevisionAbstractionInductionSide::Premise
    );

    assert_eq!(
        generalized.conclusion_classes()[0].side(),
        RecursiveWorldRevisionAbstractionInductionSide::Conclusion
    );
}

#[test]
fn generalized_class_preserves_exact_context_provenance() {
    let generalized = RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        repeated_pair_source(),
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    assert_eq!(generalized.classes()[0].supporting_contexts().len(), 2);

    assert_eq!(
        generalized.classes()[0].pair_supports()[0].contexts(),
        generalized.classes()[0].supporting_contexts()
    );
}

#[test]
fn generalization_preserves_exact_source_identity() {
    let source = repeated_three_member_clique_source();

    let before = source.clone();

    let generalized = RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        source,
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    assert_eq!(generalized.source(), &before);

    assert_eq!(generalized.threshold().min_context_support(), 2);
}

#[test]
fn generalizer_facade_matches_direct_generalization() {
    let source = repeated_three_member_clique_source();

    let threshold = RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap();

    assert_eq!(
        RecursiveWorldRevisionAbstractionGeneralizer::generalize(source.clone(), threshold,),
        RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(source, threshold,)
    );
}

#[test]
fn abstraction_generalization_is_canonical_deterministic_and_non_mutating() {
    let source = repeated_three_member_clique_source();

    let before = source.clone();

    let threshold = RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap();

    let left =
        RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(source.clone(), threshold)
            .unwrap();

    let right = RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        induced(vec![
            premise_witness(2, 3, 5, 7),
            premise_witness(1, 3, 4, 6),
            premise_witness(1, 2, 5, 7),
            premise_witness(2, 3, 4, 6),
            premise_witness(1, 3, 5, 7),
            premise_witness(1, 2, 4, 6),
        ]),
        threshold,
    )
    .unwrap();

    assert_eq!(left, right);

    assert_eq!(source, before);

    assert_eq!(left.classes()[0].minimum_pair_support(), 2);
}
