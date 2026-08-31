use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldMinimalRevision, RecursiveWorldModel, RecursiveWorldRevisionActiveCycle,
    RecursiveWorldRevisionBudget, RecursiveWorldRule,
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
fn empty_candidate_set_produces_empty_cycle() {
    let result = RecursiveWorldRevisionActiveCycle::evaluate(
        Vec::new(),
        RecursiveWorldRevisionBudget::new(5).unwrap(),
    );

    assert!(result.ranking().is_empty());

    assert!(result.selection().is_empty());

    assert!(!result.has_revision());
}

#[test]
fn single_affordable_revision_is_selected() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let change = revision(&model, target, rule(&[1], &[3]));

    let result = RecursiveWorldRevisionActiveCycle::evaluate(
        vec![change.clone()],
        RecursiveWorldRevisionBudget::new(3).unwrap(),
    );

    assert!(result.has_revision());

    assert_eq!(result.selected().unwrap().revision(), &change);
}

#[test]
fn over_budget_revision_is_not_selected() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let change = revision(&model, target, rule(&[4, 5], &[6, 7]));

    let result = RecursiveWorldRevisionActiveCycle::evaluate(
        vec![change],
        RecursiveWorldRevisionBudget::new(2).unwrap(),
    );

    assert!(!result.has_revision());

    assert!(result.selected().is_none());
}

#[test]
fn cheapest_revision_is_selected_from_multiple_candidates() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let cheap = revision(&model, target.clone(), rule(&[1], &[3]));

    let expensive = revision(&model, target, rule(&[4, 5], &[6, 7]));

    let result = RecursiveWorldRevisionActiveCycle::evaluate(
        vec![expensive, cheap.clone()],
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.selected().unwrap().revision(), &cheap);
}

#[test]
fn ranking_is_preserved_in_active_cycle_result() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let left = revision(&model, first, rule(&[1], &[3]));

    let right = revision(&model, second, rule(&[5], &[7]));

    let result = RecursiveWorldRevisionActiveCycle::evaluate(
        vec![right, left],
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.ranking().len(), 2);

    assert_eq!(result.selected(), result.ranking().best());
}

#[test]
fn exact_budget_boundary_selects_revision() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let change = revision(&model, target, rule(&[1], &[3]));

    let probe = RecursiveWorldRevisionActiveCycle::evaluate(
        vec![change.clone()],
        RecursiveWorldRevisionBudget::new(100).unwrap(),
    );

    let exact_cost = probe.selected().unwrap().total_cost();

    let result = RecursiveWorldRevisionActiveCycle::evaluate(
        vec![change],
        RecursiveWorldRevisionBudget::new(exact_cost).unwrap(),
    );

    assert!(result.has_revision());
}

#[test]
fn revised_world_matches_selected_revision_after_model() {
    let target = rule(&[1], &[2]);

    let unrelated = rule(&[8], &[9]);

    let model = RecursiveWorldModel::new(vec![target.clone(), unrelated.clone()]);

    let replacement = rule(&[1], &[3]);

    let change = revision(&model, target.clone(), replacement.clone());

    let expected = change.after().clone();

    let result = RecursiveWorldRevisionActiveCycle::evaluate(
        vec![change],
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.revised_world(), Some(&expected,));

    assert!(result.revised_world().unwrap().contains(&replacement,));

    assert!(result.revised_world().unwrap().contains(&unrelated,));

    assert!(!result.revised_world().unwrap().contains(&target,));
}

#[test]
fn no_selection_has_no_revised_world() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let change = revision(&model, target, rule(&[4, 5], &[6, 7]));

    let result = RecursiveWorldRevisionActiveCycle::evaluate(
        vec![change],
        RecursiveWorldRevisionBudget::new(1).unwrap(),
    );

    assert!(result.revised_world().is_none());
}

#[test]
fn duplicate_revision_candidates_are_collapsed_by_ranking() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let change = revision(&model, target, rule(&[1], &[3]));

    let result = RecursiveWorldRevisionActiveCycle::evaluate(
        vec![change.clone(), change],
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.ranking().len(), 1);
}

#[test]
fn active_cycle_is_deterministic_under_candidate_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let left = revision(&model, first, rule(&[1], &[3]));

    let right = revision(&model, second, rule(&[5], &[7]));

    let budget = RecursiveWorldRevisionBudget::new(20).unwrap();

    let first_result =
        RecursiveWorldRevisionActiveCycle::evaluate(vec![left.clone(), right.clone()], budget);

    let second_result = RecursiveWorldRevisionActiveCycle::evaluate(vec![right, left], budget);

    assert_eq!(first_result, second_result);
}

#[test]
fn active_cycle_preserves_selection_budget_identity() {
    let budget = RecursiveWorldRevisionBudget::new(11).unwrap();

    let result = RecursiveWorldRevisionActiveCycle::evaluate(Vec::new(), budget);

    assert_eq!(result.selection().budget(), budget);
}

#[test]
fn active_cycle_does_not_mutate_source_revision_vector() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let source = vec![revision(&model, target, rule(&[1], &[3]))];

    let before = source.clone();

    let _ = RecursiveWorldRevisionActiveCycle::evaluate(
        source.clone(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(source, before);
}
