use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_generation::{
    RecursiveWorldRevisionGenerationCandidate, RecursiveWorldRevisionGenerationCandidateSet,
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

fn candidate(
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
    basis: &[usize],
) -> RecursiveWorldRevisionGenerationCandidate {
    RecursiveWorldRevisionGenerationCandidate::new(
        target,
        replacement,
        basis.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

#[test]
fn empty_basis_candidate_is_rejected() {
    assert!(RecursiveWorldRevisionGenerationCandidate::new(
        rule(&[1], &[2],),
        rule(&[1], &[3],),
        Vec::new(),
    )
    .is_none());
}

#[test]
fn exact_noop_candidate_is_rejected() {
    let source = rule(&[1], &[2]);

    assert!(RecursiveWorldRevisionGenerationCandidate::new(
        source.clone(),
        source,
        vec![unit(9,),],
    )
    .is_none());
}

#[test]
fn distinct_candidate_with_basis_is_accepted() {
    assert!(RecursiveWorldRevisionGenerationCandidate::new(
        rule(&[1], &[2],),
        rule(&[1], &[3],),
        vec![unit(9,),],
    )
    .is_some());
}

#[test]
fn candidate_preserves_target_identity() {
    let target = rule(&[1], &[2]);

    let generated = candidate(target.clone(), rule(&[1], &[3]), &[9]);

    assert_eq!(generated.target(), &target);
}

#[test]
fn candidate_preserves_replacement_identity() {
    let replacement = rule(&[1], &[3]);

    let generated = candidate(rule(&[1], &[2]), replacement.clone(), &[9]);

    assert_eq!(generated.replacement(), &replacement);
}

#[test]
fn basis_is_canonicalized_and_deduplicated() {
    let generated = RecursiveWorldRevisionGenerationCandidate::new(
        rule(&[1], &[2]),
        rule(&[1], &[3]),
        vec![unit(11), unit(9), unit(11), unit(10), unit(9)],
    )
    .unwrap();

    assert_eq!(generated.basis_count(), 3);

    assert_eq!(generated.basis(), &[unit(9,), unit(10,), unit(11,),]);

    assert!(generated.contains_basis_unit(&unit(10,),));
}

#[test]
fn distinct_basis_preserves_distinct_candidates() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let first = candidate(target.clone(), replacement.clone(), &[9]);

    let second = candidate(target, replacement, &[10]);

    let set =
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![first.clone(), second.clone()]);

    assert_eq!(set.len(), 2);

    assert!(set.contains(&first,));

    assert!(set.contains(&second,));
}

#[test]
fn exact_duplicate_candidates_are_deduplicated() {
    let generated = candidate(rule(&[1], &[2]), rule(&[1], &[3]), &[9, 10]);

    let set = RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated.clone(), generated]);

    assert_eq!(set.len(), 1);
}

#[test]
fn target_scoped_query_is_exact() {
    let first_target = rule(&[1], &[2]);

    let second_target = rule(&[5], &[6]);

    let first = candidate(first_target.clone(), rule(&[1], &[3]), &[9]);

    let second = candidate(second_target.clone(), rule(&[5], &[7]), &[10]);

    let set = RecursiveWorldRevisionGenerationCandidateSet::new(vec![second, first.clone()]);

    assert_eq!(set.candidates_for_target(&first_target,), vec![first,]);

    assert!(set.candidates_for_target(&rule(&[8], &[9],),).is_empty());
}

#[test]
fn replacement_scoped_query_is_exact() {
    let first_replacement = rule(&[1], &[3]);

    let second_replacement = rule(&[5], &[7]);

    let first = candidate(rule(&[1], &[2]), first_replacement.clone(), &[9]);

    let second = candidate(rule(&[5], &[6]), second_replacement.clone(), &[10]);

    let set = RecursiveWorldRevisionGenerationCandidateSet::new(vec![second, first.clone()]);

    assert_eq!(
        set.candidates_for_replacement(&first_replacement,),
        vec![first,]
    );

    assert!(set
        .candidates_for_replacement(&rule(&[8], &[9],),)
        .is_empty());
}

#[test]
fn candidate_set_is_deterministic_under_input_order() {
    let first = candidate(rule(&[1], &[2]), rule(&[1], &[3]), &[9]);

    let second = candidate(rule(&[5], &[6]), rule(&[5], &[7]), &[10]);

    assert_eq!(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![first.clone(), second.clone(),],),
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![second, first,],)
    );
}

#[test]
fn candidate_set_construction_does_not_mutate_source_vector() {
    let source = vec![
        candidate(rule(&[1], &[2]), rule(&[1], &[3]), &[9]),
        candidate(rule(&[5], &[6]), rule(&[5], &[7]), &[10]),
    ];

    let before = source.clone();

    let _ = RecursiveWorldRevisionGenerationCandidateSet::new(source.clone());

    assert_eq!(source, before);
}
