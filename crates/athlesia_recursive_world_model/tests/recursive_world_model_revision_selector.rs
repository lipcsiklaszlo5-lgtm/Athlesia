use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldCostedRevision, RecursiveWorldMinimalRevision, RecursiveWorldModel,
    RecursiveWorldRevisionBudget, RecursiveWorldRevisionRanking, RecursiveWorldRevisionSelector,
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

fn costed(
    model: &RecursiveWorldModel,
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
) -> RecursiveWorldCostedRevision {
    RecursiveWorldCostedRevision::evaluate(
        RecursiveWorldMinimalRevision::apply(model, target, replacement).unwrap(),
    )
}

#[test]
fn zero_revision_budget_is_rejected() {
    assert!(RecursiveWorldRevisionBudget::new(0,).is_none());
}

#[test]
fn positive_revision_budget_preserves_identity() {
    let budget = RecursiveWorldRevisionBudget::new(7).unwrap();

    assert_eq!(budget.max_total_cost(), 7);
}

#[test]
fn exact_budget_boundary_is_allowed() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let revision = costed(&model, target, rule(&[1], &[3]));

    assert_eq!(revision.total_cost(), 3);

    let budget = RecursiveWorldRevisionBudget::new(3).unwrap();

    assert!(budget.allows(&revision,));
}

#[test]
fn over_budget_revision_is_rejected() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let revision = costed(&model, target, rule(&[4, 5], &[6, 7]));

    let budget = RecursiveWorldRevisionBudget::new(3).unwrap();

    assert!(!budget.allows(&revision,));
}

#[test]
fn empty_ranking_produces_empty_selection() {
    let ranking = RecursiveWorldRevisionRanking::new(Vec::new());

    let selection = RecursiveWorldRevisionSelector::select(
        &ranking,
        RecursiveWorldRevisionBudget::new(5).unwrap(),
    );

    assert!(selection.is_empty());

    assert!(!selection.is_selected());
}

#[test]
fn affordable_best_revision_is_selected() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let revision = costed(&model, target, rule(&[1], &[3]));

    let ranking = RecursiveWorldRevisionRanking::new(vec![revision.clone()]);

    let selection = RecursiveWorldRevisionSelector::select(
        &ranking,
        RecursiveWorldRevisionBudget::new(revision.total_cost()).unwrap(),
    );

    assert_eq!(selection.selected(), Some(&revision,));
}

#[test]
fn unaffordable_best_revision_yields_no_selection() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let revision = costed(&model, target, rule(&[1], &[3]));

    assert!(revision.total_cost() > 1);

    let ranking = RecursiveWorldRevisionRanking::new(vec![revision]);

    let selection = RecursiveWorldRevisionSelector::select(
        &ranking,
        RecursiveWorldRevisionBudget::new(1).unwrap(),
    );

    assert!(selection.selected().is_none());
}

#[test]
fn selector_never_skips_best_for_more_expensive_candidate() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let cheaper = costed(&model, target.clone(), rule(&[1], &[3]));

    let expensive = costed(&model, target, rule(&[4, 5], &[6, 7]));

    let ranking = RecursiveWorldRevisionRanking::new(vec![expensive, cheaper.clone()]);

    let selection = RecursiveWorldRevisionSelector::select(
        &ranking,
        RecursiveWorldRevisionBudget::new(cheaper.total_cost()).unwrap(),
    );

    assert_eq!(selection.selected(), Some(&cheaper,));
}

#[test]
fn selector_preserves_ranking_best_identity() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let left = costed(&model, first, rule(&[1], &[3]));

    let right = costed(&model, second, rule(&[5], &[7]));

    let ranking = RecursiveWorldRevisionRanking::new(vec![right, left]);

    let expected = ranking.best().cloned();

    let selection = RecursiveWorldRevisionSelector::select(
        &ranking,
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(selection.selected(), expected.as_ref());
}

#[test]
fn selection_preserves_budget_identity() {
    let ranking = RecursiveWorldRevisionRanking::new(Vec::new());

    let budget = RecursiveWorldRevisionBudget::new(9).unwrap();

    let selection = RecursiveWorldRevisionSelector::select(&ranking, budget);

    assert_eq!(selection.budget(), budget);
}

#[test]
fn selector_is_deterministic() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let ranking =
        RecursiveWorldRevisionRanking::new(vec![costed(&model, target, rule(&[1], &[3]))]);

    let budget = RecursiveWorldRevisionBudget::new(5).unwrap();

    assert_eq!(
        RecursiveWorldRevisionSelector::select(&ranking, budget,),
        RecursiveWorldRevisionSelector::select(&ranking, budget,)
    );
}

#[test]
fn selector_does_not_mutate_ranking() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let ranking =
        RecursiveWorldRevisionRanking::new(vec![costed(&model, target, rule(&[1], &[3]))]);

    let before = ranking.clone();

    let _ = RecursiveWorldRevisionSelector::select(
        &ranking,
        RecursiveWorldRevisionBudget::new(5).unwrap(),
    );

    assert_eq!(ranking, before);
}
