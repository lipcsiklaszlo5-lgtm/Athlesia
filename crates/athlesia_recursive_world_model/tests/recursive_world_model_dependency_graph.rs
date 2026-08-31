use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldDependencyEdge, RecursiveWorldDependencyGraph, RecursiveWorldModel,
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
fn identical_rule_does_not_form_dependency_edge() {
    let source = rule(&[1], &[2]);

    assert!(RecursiveWorldDependencyEdge::new(source.clone(), source,).is_none());
}

#[test]
fn unrelated_rules_do_not_form_dependency_edge() {
    assert!(RecursiveWorldDependencyEdge::new(rule(&[1], &[2],), rule(&[3], &[4],),).is_none());
}

#[test]
fn conclusion_matching_target_premise_forms_dependency() {
    let edge = RecursiveWorldDependencyEdge::new(rule(&[1], &[2]), rule(&[2], &[3]));

    assert!(edge.is_some());
}

#[test]
fn dependency_direction_is_preserved() {
    let source = rule(&[1], &[2]);

    let target = rule(&[2], &[3]);

    let edge = RecursiveWorldDependencyEdge::new(source.clone(), target.clone()).unwrap();

    assert_eq!(edge.source(), &source);

    assert_eq!(edge.target(), &target);

    assert!(RecursiveWorldDependencyEdge::new(target, source,).is_none());
}

#[test]
fn dependency_preserves_shared_unit_identity() {
    let edge = RecursiveWorldDependencyEdge::new(rule(&[1], &[2, 3]), rule(&[2, 3], &[4])).unwrap();

    assert_eq!(edge.shared_units(), vec![unit(2), unit(3),]);
}

#[test]
fn partial_conclusion_match_is_sufficient_for_dependency() {
    let edge = RecursiveWorldDependencyEdge::new(rule(&[1], &[2, 8]), rule(&[2, 7], &[3])).unwrap();

    assert_eq!(edge.shared_units(), vec![unit(2),]);
}

#[test]
fn empty_model_has_empty_dependency_graph() {
    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(Vec::new()));

    assert!(graph.is_empty());

    assert_eq!(graph.len(), 0);
}

#[test]
fn linear_rule_chain_produces_exact_direct_edges() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[3]);

    let third = rule(&[3], &[4]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        first.clone(),
        second.clone(),
        third.clone(),
    ]));

    assert_eq!(graph.len(), 2);

    assert!(graph.contains(&RecursiveWorldDependencyEdge::new(first, second.clone(),).unwrap(),));

    assert!(graph.contains(&RecursiveWorldDependencyEdge::new(second, third,).unwrap(),));
}

#[test]
fn fan_out_dependencies_are_all_preserved() {
    let source = rule(&[1], &[2]);

    let left = rule(&[2], &[3]);

    let right = rule(&[2], &[4]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        source.clone(),
        left.clone(),
        right.clone(),
    ]));

    assert_eq!(graph.outgoing_count(&source,), 2);

    assert_eq!(graph.incoming_count(&left,), 1);

    assert_eq!(graph.incoming_count(&right,), 1);

    assert_eq!(graph.direct_dependents(&source,), vec![left, right,]);
}

#[test]
fn cyclic_dependencies_remain_directionally_explicit() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[1]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        first.clone(),
        second.clone(),
    ]));

    assert_eq!(graph.len(), 2);

    assert!(graph
        .contains(&RecursiveWorldDependencyEdge::new(first.clone(), second.clone(),).unwrap(),));

    assert!(graph.contains(&RecursiveWorldDependencyEdge::new(second, first,).unwrap(),));
}

#[test]
fn dependency_graph_is_deterministic_under_rule_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[3]);

    let third = rule(&[2], &[4]);

    let left = RecursiveWorldModel::new(vec![first.clone(), second.clone(), third.clone()]);

    let right = RecursiveWorldModel::new(vec![third, first, second]);

    assert_eq!(
        RecursiveWorldDependencyGraph::detect(&left,),
        RecursiveWorldDependencyGraph::detect(&right,)
    );
}

#[test]
fn dependency_detection_does_not_mutate_world_model() {
    let model =
        RecursiveWorldModel::new(vec![rule(&[1], &[2]), rule(&[2], &[3]), rule(&[3], &[4])]);

    let before = model.clone();

    let _ = RecursiveWorldDependencyGraph::detect(&model);

    assert_eq!(model, before);
}
