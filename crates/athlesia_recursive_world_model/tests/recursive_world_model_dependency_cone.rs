use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldDependencyCone, RecursiveWorldDependencyGraph, RecursiveWorldModel,
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
fn isolated_rule_has_empty_dependency_cone() {
    let root = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![root.clone()]);

    let graph = RecursiveWorldDependencyGraph::detect(&model);

    let cone = RecursiveWorldDependencyCone::compute(&graph, root);

    assert!(cone.is_empty());

    assert_eq!(cone.len(), 0);
}

#[test]
fn cone_preserves_root_identity() {
    let root = rule(&[1], &[2]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        root.clone(),
        rule(&[2], &[3]),
    ]));

    let cone = RecursiveWorldDependencyCone::compute(&graph, root.clone());

    assert_eq!(cone.root(), &root);
}

#[test]
fn direct_dependent_is_in_cone() {
    let root = rule(&[1], &[2]);

    let dependent = rule(&[2], &[3]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        root.clone(),
        dependent.clone(),
    ]));

    let cone = RecursiveWorldDependencyCone::compute(&graph, root);

    assert!(cone.contains(&dependent,));
}

#[test]
fn transitive_dependent_is_in_cone() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[3]);

    let third = rule(&[3], &[4]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        first.clone(),
        second.clone(),
        third.clone(),
    ]));

    let cone = RecursiveWorldDependencyCone::compute(&graph, first);

    assert_eq!(cone.len(), 2);

    assert!(cone.contains(&second,));

    assert!(cone.contains(&third,));
}

#[test]
fn root_is_excluded_from_its_own_cone() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[1]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        first.clone(),
        second,
    ]));

    let cone = RecursiveWorldDependencyCone::compute(&graph, first);

    assert!(!cone.includes_root());
}

#[test]
fn cycles_do_not_cause_infinite_expansion() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[3]);

    let third = rule(&[3], &[1]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        first.clone(),
        second.clone(),
        third.clone(),
    ]));

    let cone = RecursiveWorldDependencyCone::compute(&graph, first);

    assert_eq!(cone.len(), 2);

    assert!(cone.contains(&second,));

    assert!(cone.contains(&third,));
}

#[test]
fn fan_out_branches_are_all_included() {
    let root = rule(&[1], &[2]);

    let left = rule(&[2], &[3]);

    let right = rule(&[2], &[4]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        root.clone(),
        left.clone(),
        right.clone(),
    ]));

    let cone = RecursiveWorldDependencyCone::compute(&graph, root);

    assert_eq!(cone.len(), 2);

    assert!(cone.contains(&left,));

    assert!(cone.contains(&right,));
}

#[test]
fn converging_branches_are_deduplicated() {
    let root = rule(&[1], &[2]);

    let left = rule(&[2], &[3]);

    let right = rule(&[2], &[4]);

    let merge = rule(&[3, 4], &[5]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        root.clone(),
        left.clone(),
        right.clone(),
        merge.clone(),
    ]));

    let cone = RecursiveWorldDependencyCone::compute(&graph, root);

    assert_eq!(cone.len(), 3);

    assert!(cone.contains(&left,));

    assert!(cone.contains(&right,));

    assert!(cone.contains(&merge,));
}

#[test]
fn unrelated_branch_is_excluded() {
    let root = rule(&[1], &[2]);

    let dependent = rule(&[2], &[3]);

    let unrelated = rule(&[8], &[9]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        root.clone(),
        dependent.clone(),
        unrelated.clone(),
    ]));

    let cone = RecursiveWorldDependencyCone::compute(&graph, root);

    assert!(cone.contains(&dependent,));

    assert!(!cone.contains(&unrelated,));
}

#[test]
fn affected_rules_are_canonicalized() {
    let root = rule(&[1], &[2]);

    let left = rule(&[2], &[4]);

    let right = rule(&[2], &[3]);

    let graph = RecursiveWorldDependencyGraph::detect(&RecursiveWorldModel::new(vec![
        left.clone(),
        root.clone(),
        right.clone(),
    ]));

    let cone = RecursiveWorldDependencyCone::compute(&graph, root);

    let mut expected = vec![left, right];

    expected.sort();

    assert_eq!(cone.affected(), expected.as_slice());
}

#[test]
fn cone_is_deterministic_under_model_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[2], &[3]);

    let third = rule(&[3], &[4]);

    let left_model = RecursiveWorldModel::new(vec![first.clone(), second.clone(), third.clone()]);

    let right_model = RecursiveWorldModel::new(vec![third, first.clone(), second]);

    let left_graph = RecursiveWorldDependencyGraph::detect(&left_model);

    let right_graph = RecursiveWorldDependencyGraph::detect(&right_model);

    assert_eq!(
        RecursiveWorldDependencyCone::compute(&left_graph, first.clone(),),
        RecursiveWorldDependencyCone::compute(&right_graph, first,)
    );
}

#[test]
fn cone_computation_does_not_mutate_graph() {
    let root = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![root.clone(), rule(&[2], &[3]), rule(&[3], &[4])]);

    let graph = RecursiveWorldDependencyGraph::detect(&model);

    let before = graph.clone();

    let _ = RecursiveWorldDependencyCone::compute(&graph, root);

    assert_eq!(graph, before);
}
