use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_generalization::{
    RecursiveWorldRevisionAbstractionGeneralizationResolution,
    RecursiveWorldRevisionAbstractionGeneralizationResolver,
    RecursiveWorldRevisionAbstractionGeneralizationThreshold,
    RecursiveWorldRevisionAbstractionGeneralizedClassSet,
};

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInducedClassSet,
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
    RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(witnesses).unwrap(),
    )
    .unwrap()
}

fn generalized(
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
) -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        induced(witnesses),
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap()
}

fn identical_cross_side_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    generalized(vec![
        premise_witness(1, 2, 10, 20),
        premise_witness(1, 2, 11, 21),
        conclusion_witness(1, 2, 30, 40),
        conclusion_witness(1, 2, 31, 41),
    ])
}

fn overlapping_cross_side_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    generalized(vec![
        premise_witness(1, 2, 10, 20),
        premise_witness(1, 2, 11, 21),
        conclusion_witness(2, 3, 30, 40),
        conclusion_witness(2, 3, 31, 41),
    ])
}

fn disjoint_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    generalized(vec![
        premise_witness(1, 2, 10, 20),
        premise_witness(1, 2, 11, 21),
        conclusion_witness(3, 4, 30, 40),
        conclusion_witness(3, 4, 31, 41),
    ])
}

#[test]
fn identical_generalized_class_identities_are_merged() {
    let resolution = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(
        identical_cross_side_source(),
    );

    assert_eq!(resolution.resolved_count(), 1);

    assert_eq!(resolution.conflicted_count(), 0);

    assert_eq!(
        resolution.resolved_classes()[0]
            .abstraction_class()
            .members(),
        &[unit(1,), unit(2,),]
    );
}

#[test]
fn identical_class_merge_preserves_all_generalized_sources() {
    let resolution = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(
        identical_cross_side_source(),
    );

    assert_eq!(resolution.resolved_classes()[0].source_count(), 2);

    assert_eq!(resolution.resolved_classes()[0].sources().len(), 2);
}

#[test]
fn identical_class_merge_preserves_context_provenance() {
    let resolution = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(
        identical_cross_side_source(),
    );

    assert_eq!(
        resolution.resolved_classes()[0].supporting_contexts().len(),
        4
    );
}

#[test]
fn overlapping_distinct_classes_are_both_conflicted() {
    let resolution = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(
        overlapping_cross_side_source(),
    );

    assert_eq!(resolution.resolved_count(), 0);

    assert_eq!(resolution.conflicted_count(), 2);

    assert!(resolution.has_conflicts());
}

#[test]
fn overlap_conflict_exposes_exact_shared_units() {
    let resolution = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(
        overlapping_cross_side_source(),
    );

    assert_eq!(resolution.conflicts().len(), 1);

    assert_eq!(resolution.conflicts()[0].overlap(), &[unit(2,),]);
}

#[test]
fn conflicts_never_choose_arbitrary_winner() {
    let resolution = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(
        overlapping_cross_side_source(),
    );

    assert!(resolution.resolved_classes().is_empty());

    assert!(resolution.vocabulary().is_none());
}

#[test]
fn independent_disjoint_classes_survive_resolution() {
    let resolution =
        RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(disjoint_source());

    assert_eq!(resolution.resolved_count(), 2);

    assert_eq!(resolution.conflicted_count(), 0);

    assert!(!resolution.has_conflicts());
}

#[test]
fn resolved_disjoint_classes_materialize_m40_vocabulary() {
    let resolution =
        RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(disjoint_source());

    let vocabulary = resolution.vocabulary().unwrap();

    assert_eq!(vocabulary.classes().len(), 2);

    assert!(vocabulary.covers(&unit(1,),));

    assert!(vocabulary.covers(&unit(4,),));
}

#[test]
fn identical_merged_class_materializes_single_vocabulary_class() {
    let resolution = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(
        identical_cross_side_source(),
    );

    let vocabulary = resolution.vocabulary().unwrap();

    assert_eq!(vocabulary.classes().len(), 1);

    assert_eq!(vocabulary.classes()[0].members(), &[unit(1,), unit(2,),]);
}

#[test]
fn resolution_preserves_exact_generalization_source() {
    let source = disjoint_source();

    let before = source.clone();

    let resolution = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(source);

    assert_eq!(resolution.source(), &before);
}

#[test]
fn resolver_facade_matches_direct_resolution() {
    let source = disjoint_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionGeneralizationResolver::resolve(source.clone(),),
        RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(source,)
    );
}

#[test]
fn resolution_is_canonical_deterministic_and_non_mutating() {
    let source = disjoint_source();

    let before = source.clone();

    let left = RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(source.clone());

    let right =
        RecursiveWorldRevisionAbstractionGeneralizationResolution::resolve(generalized(vec![
            conclusion_witness(3, 4, 31, 41),
            premise_witness(1, 2, 11, 21),
            conclusion_witness(3, 4, 30, 40),
            premise_witness(1, 2, 10, 20),
        ]));

    assert_eq!(left, right);

    assert_eq!(source, before);

    assert_eq!(left.resolved_count(), 2);

    assert!(left.vocabulary().is_some());
}
