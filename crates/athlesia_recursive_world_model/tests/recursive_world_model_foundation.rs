use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

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
fn empty_premises_are_rejected() {
    assert!(RecursiveWorldRule::new(Vec::new(), vec![unit(2),],).is_none());
}

#[test]
fn empty_conclusions_are_rejected() {
    assert!(RecursiveWorldRule::new(vec![unit(1),], Vec::new(),).is_none());
}

#[test]
fn rule_preserves_premise_identity() {
    let first = unit(1);

    let second = unit(2);

    let world_rule =
        RecursiveWorldRule::new(vec![second.clone(), first.clone()], vec![unit(3)]).unwrap();

    assert!(world_rule.contains_premise(&first,));

    assert!(world_rule.contains_premise(&second,));
}

#[test]
fn rule_preserves_conclusion_identity() {
    let conclusion = unit(4);

    let world_rule = RecursiveWorldRule::new(vec![unit(1)], vec![conclusion.clone()]).unwrap();

    assert!(world_rule.contains_conclusion(&conclusion,));
}

#[test]
fn duplicate_premises_are_deduplicated() {
    let world_rule =
        RecursiveWorldRule::new(vec![unit(1), unit(1), unit(2)], vec![unit(3)]).unwrap();

    assert_eq!(world_rule.premise_count(), 2);
}

#[test]
fn duplicate_conclusions_are_deduplicated() {
    let world_rule =
        RecursiveWorldRule::new(vec![unit(1)], vec![unit(2), unit(2), unit(3)]).unwrap();

    assert_eq!(world_rule.conclusion_count(), 2);
}

#[test]
fn rule_identity_is_deterministic_under_input_order() {
    let left = RecursiveWorldRule::new(vec![unit(2), unit(1)], vec![unit(4), unit(3)]).unwrap();

    let right = RecursiveWorldRule::new(vec![unit(1), unit(2)], vec![unit(3), unit(4)]).unwrap();

    assert_eq!(left, right);
}

#[test]
fn empty_world_model_is_empty() {
    let model = RecursiveWorldModel::new(Vec::new());

    assert!(model.is_empty());

    assert_eq!(model.len(), 0);
}

#[test]
fn world_model_preserves_distinct_rules() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[3]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    assert_eq!(model.len(), 2);

    assert!(model.contains(&first,));

    assert!(model.contains(&second,));
}

#[test]
fn duplicate_rules_are_deduplicated() {
    let world_rule = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![world_rule.clone(), world_rule]);

    assert_eq!(model.len(), 1);
}

#[test]
fn world_model_is_deterministic_under_rule_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[3]);

    let left = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let right = RecursiveWorldModel::new(vec![second, first]);

    assert_eq!(left, right);
}

#[test]
fn world_model_construction_does_not_mutate_source_rules() {
    let source = vec![rule(&[1], &[2]), rule(&[2], &[3])];

    let before = source.clone();

    let _ = RecursiveWorldModel::new(source.clone());

    assert_eq!(source, before);
}
