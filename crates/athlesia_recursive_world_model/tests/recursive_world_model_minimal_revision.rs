use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldMinimalRevision, RecursiveWorldModel, RecursiveWorldRule,
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

fn chain_model() -> (
    RecursiveWorldModel,
    RecursiveWorldRule,
    RecursiveWorldRule,
    RecursiveWorldRule,
) {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[3]);

    let third = rule(&[3], &[4]);

    (
        RecursiveWorldModel::new(vec![first.clone(), second.clone(), third.clone()]),
        first,
        second,
        third,
    )
}

#[test]
fn missing_target_is_rejected() {
    let (model, _, _, _) = chain_model();

    assert!(
        RecursiveWorldMinimalRevision::apply(&model, rule(&[8], &[9],), rule(&[8], &[10],),)
            .is_none()
    );
}

#[test]
fn identical_replacement_is_rejected() {
    let (model, first, _, _) = chain_model();

    assert!(RecursiveWorldMinimalRevision::apply(&model, first.clone(), first,).is_none());
}

#[test]
fn existing_replacement_collision_is_rejected() {
    let (model, first, second, _) = chain_model();

    assert!(RecursiveWorldMinimalRevision::apply(&model, first, second,).is_none());
}

#[test]
fn valid_revision_replaces_exact_target() {
    let (model, first, second, third) = chain_model();

    let replacement = rule(&[1], &[5]);

    let revision =
        RecursiveWorldMinimalRevision::apply(&model, first.clone(), replacement.clone()).unwrap();

    assert!(!revision.after().contains(&first,));

    assert!(revision.after().contains(&replacement,));

    assert!(revision.after().contains(&second,));

    assert!(revision.after().contains(&third,));
}

#[test]
fn revision_preserves_before_model_identity() {
    let (model, first, _, _) = chain_model();

    let before = model.clone();

    let revision = RecursiveWorldMinimalRevision::apply(&model, first, rule(&[1], &[5])).unwrap();

    assert_eq!(revision.before(), &before);
}

#[test]
fn revision_preserves_target_and_replacement_identity() {
    let (model, first, _, _) = chain_model();

    let replacement = rule(&[1], &[5]);

    let revision =
        RecursiveWorldMinimalRevision::apply(&model, first.clone(), replacement.clone()).unwrap();

    assert_eq!(revision.target(), &first);

    assert_eq!(revision.replacement(), &replacement);
}

#[test]
fn revision_changes_exactly_one_rule() {
    let (model, first, _, _) = chain_model();

    let revision = RecursiveWorldMinimalRevision::apply(&model, first, rule(&[1], &[5])).unwrap();

    assert_eq!(revision.changed_rule_count(), 1);

    assert_eq!(revision.unaffected_rule_count(), 2);
}

#[test]
fn revision_preserves_world_model_cardinality() {
    let (model, first, _, _) = chain_model();

    let revision = RecursiveWorldMinimalRevision::apply(&model, first, rule(&[1], &[5])).unwrap();

    assert!(revision.preserves_rule_count());

    assert_eq!(revision.before().len(), 3);

    assert_eq!(revision.after().len(), 3);
}

#[test]
fn revision_records_original_dependency_cone() {
    let (model, first, second, third) = chain_model();

    let revision = RecursiveWorldMinimalRevision::apply(&model, first, rule(&[1], &[5])).unwrap();

    assert_eq!(revision.affected_before().len(), 2);

    assert!(revision.affected_before().contains(&second,));

    assert!(revision.affected_before().contains(&third,));
}

#[test]
fn unrelated_rules_remain_exactly_preserved() {
    let target = rule(&[1], &[2]);

    let dependent = rule(&[2], &[3]);

    let unrelated = rule(&[8], &[9]);

    let model =
        RecursiveWorldModel::new(vec![target.clone(), dependent.clone(), unrelated.clone()]);

    let revision = RecursiveWorldMinimalRevision::apply(&model, target, rule(&[1], &[5])).unwrap();

    assert!(revision.after().contains(&unrelated,));

    assert!(revision.after().contains(&dependent,));

    assert!(!revision.affected_before().contains(&unrelated,));
}

#[test]
fn minimal_revision_is_deterministic() {
    let (model, first, _, _) = chain_model();

    let replacement = rule(&[1], &[5]);

    let left =
        RecursiveWorldMinimalRevision::apply(&model, first.clone(), replacement.clone()).unwrap();

    let right = RecursiveWorldMinimalRevision::apply(&model, first, replacement).unwrap();

    assert_eq!(left, right);
}

#[test]
fn minimal_revision_does_not_mutate_source_model() {
    let (model, first, _, _) = chain_model();

    let before = model.clone();

    let _ = RecursiveWorldMinimalRevision::apply(&model, first, rule(&[1], &[5])).unwrap();

    assert_eq!(model, before);
}
