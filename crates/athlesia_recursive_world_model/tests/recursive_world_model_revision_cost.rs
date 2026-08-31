use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldCostedRevision, RecursiveWorldMinimalRevision, RecursiveWorldModel,
    RecursiveWorldRevisionCost, RecursiveWorldRule,
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

fn revision(
    model: &RecursiveWorldModel,
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
) -> RecursiveWorldMinimalRevision {
    RecursiveWorldMinimalRevision::apply(model, target, replacement).unwrap()
}

#[test]
fn minimal_revision_has_single_rule_change_cost() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let change = revision(&model, target, rule(&[1], &[3]));

    let cost = RecursiveWorldRevisionCost::evaluate(&change);

    assert_eq!(cost.rule_change_cost(), 1);
}

#[test]
fn isolated_revision_has_zero_dependency_impact_cost() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone(), rule(&[8], &[9])]);

    let cost = RecursiveWorldRevisionCost::evaluate(&revision(&model, target, rule(&[1], &[3])));

    assert_eq!(cost.dependency_impact_cost(), 0);
}

#[test]
fn dependency_impact_equals_original_dependency_cone_size() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[3]);

    let third = rule(&[3], &[4]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second, third]);

    let cost = RecursiveWorldRevisionCost::evaluate(&revision(&model, first, rule(&[1], &[5])));

    assert_eq!(cost.dependency_impact_cost(), 2);
}

#[test]
fn unchanged_premises_have_zero_premise_delta() {
    let target = rule(&[1, 2], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let cost = RecursiveWorldRevisionCost::evaluate(&revision(&model, target, rule(&[2, 1], &[4])));

    assert_eq!(cost.premise_delta_cost(), 0);
}

#[test]
fn replacing_one_premise_has_symmetric_delta_two() {
    let target = rule(&[1, 2], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let cost = RecursiveWorldRevisionCost::evaluate(&revision(&model, target, rule(&[1, 4], &[3])));

    assert_eq!(cost.premise_delta_cost(), 2);
}

#[test]
fn adding_one_premise_has_delta_one() {
    let target = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let cost = RecursiveWorldRevisionCost::evaluate(&revision(&model, target, rule(&[1, 2], &[3])));

    assert_eq!(cost.premise_delta_cost(), 1);
}

#[test]
fn replacing_one_conclusion_has_symmetric_delta_two() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let cost = RecursiveWorldRevisionCost::evaluate(&revision(&model, target, rule(&[1], &[3])));

    assert_eq!(cost.conclusion_delta_cost(), 2);
}

#[test]
fn structural_delta_combines_premise_and_conclusion_delta() {
    let target = rule(&[1, 2], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let cost = RecursiveWorldRevisionCost::evaluate(&revision(&model, target, rule(&[1, 4], &[5])));

    assert_eq!(cost.premise_delta_cost(), 2);

    assert_eq!(cost.conclusion_delta_cost(), 2);

    assert_eq!(cost.structural_delta_cost(), 4);
}

#[test]
fn total_cost_is_exact_component_sum() {
    let first = rule(&[1], &[2]);

    let dependent = rule(&[2], &[7]);

    let model = RecursiveWorldModel::new(vec![first.clone(), dependent]);

    let cost = RecursiveWorldRevisionCost::evaluate(&revision(&model, first, rule(&[1], &[3])));

    assert_eq!(cost.rule_change_cost(), 1);

    assert_eq!(cost.dependency_impact_cost(), 1);

    assert_eq!(cost.structural_delta_cost(), 2);

    assert_eq!(cost.total(), 4);
}

#[test]
fn costed_revision_preserves_revision_identity() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let source = revision(&model, target, rule(&[1], &[3]));

    let expected = source.clone();

    let costed = RecursiveWorldCostedRevision::evaluate(source);

    assert_eq!(costed.revision(), &expected);
}

#[test]
fn revision_cost_is_deterministic() {
    let target = rule(&[1, 2], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone(), rule(&[3], &[4])]);

    let change = revision(&model, target, rule(&[1, 5], &[6]));

    assert_eq!(
        RecursiveWorldRevisionCost::evaluate(&change,),
        RecursiveWorldRevisionCost::evaluate(&change,)
    );
}

#[test]
fn revision_cost_evaluation_does_not_mutate_revision() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone(), rule(&[2], &[3])]);

    let change = revision(&model, target, rule(&[1], &[4]));

    let before = change.clone();

    let costed = RecursiveWorldCostedRevision::evaluate(change.clone());

    assert_eq!(change, before);

    assert_eq!(costed.total_cost(), costed.cost().total());
}
