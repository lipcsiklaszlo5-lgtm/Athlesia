use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInducedClassSet,
    RecursiveWorldRevisionAbstractionSubstitutionWitness,
    RecursiveWorldRevisionAbstractionSubstitutionWitnessSet,
    RecursiveWorldRevisionAbstractionVocabularyResolution,
    RecursiveWorldRevisionAbstractionVocabularyResolver,
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

fn witness(
    first: RecursiveWorldRevisionDiscoveryObservation,
    second: RecursiveWorldRevisionDiscoveryObservation,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(first, second).unwrap()
}

fn induced_from(
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
) -> RecursiveWorldRevisionAbstractionInducedClassSet {
    RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(witnesses).unwrap(),
    )
    .unwrap()
}

fn pair_in_context(
    first: usize,
    second: usize,
    shared: usize,
    conclusion: usize,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    witness(
        observation(&[first, shared], &[conclusion]),
        observation(&[second, shared], &[conclusion]),
    )
}

#[test]
fn single_induced_class_resolves_into_vocabulary() {
    let resolution =
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
            pair_in_context(1, 2, 5, 10),
        ]));

    assert_eq!(resolution.resolved_count(), 1);

    assert_eq!(resolution.conflicted_count(), 0);

    assert!(resolution.vocabulary().is_some());

    assert!(resolution.vocabulary().unwrap().covers(&unit(1),));
}

#[test]
fn disjoint_induced_classes_both_enter_vocabulary() {
    let resolution =
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
            pair_in_context(1, 2, 5, 10),
            pair_in_context(3, 4, 6, 11),
        ]));

    assert_eq!(resolution.resolved_count(), 2);

    assert_eq!(resolution.vocabulary().unwrap().classes().len(), 2);
}

#[test]
fn identical_classes_from_different_contexts_merge_identity() {
    let resolution =
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
            pair_in_context(1, 2, 5, 10),
            pair_in_context(1, 2, 6, 11),
        ]));

    assert_eq!(resolution.resolved_count(), 1);

    assert_eq!(
        resolution.resolved_classes()[0]
            .abstraction_class()
            .members(),
        &[unit(1), unit(2),]
    );

    assert_eq!(resolution.resolved_classes()[0].context_count(), 2);
}

#[test]
fn identical_class_merge_preserves_all_witness_provenance() {
    let resolution =
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
            pair_in_context(1, 2, 5, 10),
            pair_in_context(1, 2, 6, 11),
        ]));

    assert_eq!(resolution.resolved_classes()[0].witness_count(), 2);
}

#[test]
fn partially_overlapping_classes_are_conflicted() {
    let resolution =
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
            pair_in_context(1, 2, 5, 10),
            pair_in_context(2, 3, 6, 11),
        ]));

    assert!(resolution.has_conflicts());

    assert_eq!(resolution.conflicted_count(), 2);

    assert_eq!(resolution.resolved_count(), 0);

    assert!(resolution.vocabulary().is_none());
}

#[test]
fn conflict_records_exact_overlap() {
    let resolution =
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
            pair_in_context(1, 2, 5, 10),
            pair_in_context(2, 3, 6, 11),
        ]));

    assert_eq!(resolution.conflicts().len(), 1);

    assert_eq!(
        resolution.conflicts()[0].overlap(),
        std::slice::from_ref(&unit(2),)
    );
}

#[test]
fn resolver_never_selects_one_side_of_overlap_conflict() {
    let resolution =
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
            pair_in_context(1, 2, 5, 10),
            pair_in_context(2, 3, 6, 11),
        ]));

    assert!(resolution.resolved_classes().is_empty());

    assert!(resolution.vocabulary().is_none());
}

#[test]
fn independent_class_survives_unrelated_overlap_conflict() {
    let resolution =
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
            pair_in_context(1, 2, 5, 10),
            pair_in_context(2, 3, 6, 11),
            pair_in_context(7, 8, 9, 12),
        ]));

    assert_eq!(resolution.conflicted_count(), 2);

    assert_eq!(resolution.resolved_count(), 1);

    let vocabulary = resolution.vocabulary().unwrap();

    assert!(vocabulary.covers(&unit(7),));

    assert!(vocabulary.covers(&unit(8),));

    assert!(!vocabulary.covers(&unit(1),));

    assert!(!vocabulary.covers(&unit(2),));
}

#[test]
fn resolved_vocabulary_is_m40_non_overlapping_by_construction() {
    let resolution =
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
            pair_in_context(1, 2, 5, 10),
            pair_in_context(3, 4, 6, 11),
            pair_in_context(7, 8, 9, 12),
        ]));

    let vocabulary = resolution.vocabulary().unwrap();

    assert_eq!(vocabulary.classes().len(), 3);

    assert_eq!(
        vocabulary.class_for(&unit(1),),
        Some(&vocabulary.classes()[0],)
    );

    assert_ne!(
        vocabulary.class_for(&unit(1),),
        vocabulary.class_for(&unit(3),)
    );
}

#[test]
fn vocabulary_resolution_preserves_source_induction() {
    let source = induced_from(vec![
        pair_in_context(1, 2, 5, 10),
        pair_in_context(3, 4, 6, 11),
    ]);

    let before = source.clone();

    let resolution = RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(source);

    assert_eq!(resolution.source(), &before);
}

#[test]
fn vocabulary_resolver_facade_matches_direct_resolution() {
    let source = induced_from(vec![
        pair_in_context(1, 2, 5, 10),
        pair_in_context(3, 4, 6, 11),
    ]);

    assert_eq!(
        RecursiveWorldRevisionAbstractionVocabularyResolver::resolve(source.clone(),),
        RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(source,)
    );
}

#[test]
fn vocabulary_resolution_is_canonical_and_deterministic() {
    let first = pair_in_context(1, 2, 5, 10);

    let second = pair_in_context(1, 2, 6, 11);

    let third = pair_in_context(7, 8, 9, 12);

    let left = RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
        third.clone(),
        first.clone(),
        second.clone(),
    ]));

    let right = RecursiveWorldRevisionAbstractionVocabularyResolution::resolve(induced_from(vec![
        second, third, first,
    ]));

    assert_eq!(left, right);

    assert_eq!(left.resolved_count(), 2);

    assert!(!left.has_conflicts());
}
