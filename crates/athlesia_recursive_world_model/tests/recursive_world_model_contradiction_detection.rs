use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldContradictionCandidate, RecursiveWorldContradictionSet, RecursiveWorldModel,
    RecursiveWorldRule,
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

#[test]
fn identical_rule_is_not_a_contradiction_candidate() {
    let source = rule(&[1], &[2]);

    assert!(RecursiveWorldContradictionCandidate::new(source.clone(), source,).is_none());
}

#[test]
fn different_premises_are_not_competing() {
    assert!(
        RecursiveWorldContradictionCandidate::new(rule(&[1], &[3],), rule(&[2], &[4],),).is_none()
    );
}

#[test]
fn same_premises_and_same_conclusions_are_not_competing() {
    assert!(RecursiveWorldContradictionCandidate::new(
        rule(&[1, 2], &[3, 4],),
        rule(&[2, 1], &[4, 3],),
    )
    .is_none());
}

#[test]
fn same_premises_and_different_conclusions_form_candidate() {
    let candidate = RecursiveWorldContradictionCandidate::new(rule(&[1], &[2]), rule(&[1], &[3]));

    assert!(candidate.is_some());
}

#[test]
fn contradiction_candidate_preserves_premise_identity() {
    let candidate =
        RecursiveWorldContradictionCandidate::new(rule(&[1, 2], &[3]), rule(&[2, 1], &[4]))
            .unwrap();

    assert_eq!(candidate.premises(), &[unit(1), unit(2),]);
}

#[test]
fn contradiction_pair_identity_is_canonical() {
    let first = rule(&[1], &[2]);

    let second = rule(&[1], &[3]);

    let left = RecursiveWorldContradictionCandidate::new(first.clone(), second.clone()).unwrap();

    let right = RecursiveWorldContradictionCandidate::new(second, first).unwrap();

    assert_eq!(left, right);
}

#[test]
fn disjoint_conclusions_are_detected() {
    let candidate =
        RecursiveWorldContradictionCandidate::new(rule(&[1], &[2]), rule(&[1], &[3])).unwrap();

    assert!(candidate.is_disjoint());

    assert!(!candidate.shares_conclusion());
}

#[test]
fn overlapping_conclusions_are_distinguished_from_disjoint() {
    let candidate =
        RecursiveWorldContradictionCandidate::new(rule(&[1], &[2, 3]), rule(&[1], &[3, 4]))
            .unwrap();

    assert!(candidate.shares_conclusion());

    assert!(!candidate.is_disjoint());
}

#[test]
fn empty_model_has_no_contradiction_candidates() {
    let model = RecursiveWorldModel::new(Vec::new());

    let detected = RecursiveWorldContradictionSet::detect(&model);

    assert!(detected.is_empty());

    assert_eq!(detected.len(), 0);
}

#[test]
fn detector_finds_all_same_premise_divergences() {
    let model = RecursiveWorldModel::new(vec![
        rule(&[1], &[2]),
        rule(&[1], &[3]),
        rule(&[1], &[4]),
        rule(&[8], &[9]),
    ]);

    let detected = RecursiveWorldContradictionSet::detect(&model);

    assert_eq!(detected.len(), 3);

    assert_eq!(detected.disjoint_count(), 3);
}

#[test]
fn detector_is_deterministic_under_rule_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[1], &[3]);

    let third = rule(&[1], &[4]);

    let left = RecursiveWorldModel::new(vec![first.clone(), second.clone(), third.clone()]);

    let right = RecursiveWorldModel::new(vec![third, first, second]);

    assert_eq!(
        RecursiveWorldContradictionSet::detect(&left,),
        RecursiveWorldContradictionSet::detect(&right,)
    );
}

#[test]
fn contradiction_detection_does_not_mutate_world_model() {
    let model = RecursiveWorldModel::new(vec![rule(&[1], &[2]), rule(&[1], &[3])]);

    let before = model.clone();

    let _ = RecursiveWorldContradictionSet::detect(&model);

    assert_eq!(model, before);
}
