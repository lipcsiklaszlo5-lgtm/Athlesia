use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_counterfactual::{
    RecursiveCounterfactualBudget, RecursiveCounterfactualCandidate,
    RecursiveCounterfactualInformationRanking, RecursiveCounterfactualInformationValue,
    RecursiveCounterfactualOutcome, RecursiveCounterfactualProjection,
    RecursiveCounterfactualSelectionPolicy,
};

use athlesia_recursive_planning::{RecursivePlanningState, RecursivePlanningTransition};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
    )
}

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(structural(span)))
}

fn state(spans: &[usize]) -> RecursivePlanningState {
    RecursivePlanningState::new(spans.iter().copied().map(unit).collect())
}

fn value(
    transition_target: usize,
    cost: usize,
    outcome_spans: &[&[usize]],
) -> RecursiveCounterfactualInformationValue {
    let transition =
        RecursivePlanningTransition::new(state(&[1]), state(&[1, transition_target])).unwrap();

    let candidate = RecursiveCounterfactualCandidate::new(transition, cost).unwrap();

    let projection = RecursiveCounterfactualProjection::new(
        candidate,
        outcome_spans
            .iter()
            .map(|spans| RecursiveCounterfactualOutcome::new(state(spans)))
            .collect(),
    )
    .unwrap();

    RecursiveCounterfactualInformationValue::evaluate(projection)
}

fn budgeted_ranking(
    values: Vec<RecursiveCounterfactualInformationValue>,
) -> athlesia_recursive_counterfactual::RecursiveCounterfactualBudgetedRanking {
    let ranking = RecursiveCounterfactualInformationRanking::new(values);

    let budget = RecursiveCounterfactualBudget::new(10, 5, 10).unwrap();

    budget.apply(&ranking)
}

#[test]
fn zero_beam_width_is_rejected() {
    assert!(RecursiveCounterfactualSelectionPolicy::new(0,).is_none());
}

#[test]
fn positive_beam_width_is_accepted() {
    let policy = RecursiveCounterfactualSelectionPolicy::new(3).unwrap();

    assert_eq!(policy.beam_width(), 3);
}

#[test]
fn empty_ranking_produces_empty_selection() {
    let ranking = budgeted_ranking(Vec::new());

    let policy = RecursiveCounterfactualSelectionPolicy::new(3).unwrap();

    let selection = policy.select(&ranking);

    assert!(selection.is_empty());

    assert_eq!(selection.len(), 0);

    assert!(selection.best().is_none());
}

#[test]
fn beam_width_one_keeps_only_best_value() {
    let best = value(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]);

    let weaker = value(5, 4, &[&[1, 5], &[1, 6]]);

    let ranking = budgeted_ranking(vec![weaker, best.clone()]);

    let policy = RecursiveCounterfactualSelectionPolicy::new(1).unwrap();

    let selection = policy.select(&ranking);

    assert_eq!(selection.len(), 1);

    assert_eq!(selection.best(), Some(&best));
}

#[test]
fn beam_width_caps_frontier_size() {
    let first = value(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]);

    let second = value(5, 2, &[&[1, 5], &[1, 6], &[1, 7]]);

    let third = value(8, 3, &[&[1, 8], &[1, 9]]);

    let ranking = budgeted_ranking(vec![third, first, second]);

    let policy = RecursiveCounterfactualSelectionPolicy::new(2).unwrap();

    let selection = policy.select(&ranking);

    assert_eq!(selection.len(), 2);

    assert!(selection.is_full());
}

#[test]
fn selection_preserves_budgeted_ranking_order() {
    let first = value(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]);

    let second = value(5, 2, &[&[1, 5], &[1, 6]]);

    let third = value(8, 5, &[&[1, 8], &[1, 9]]);

    let ranking = budgeted_ranking(vec![third, second, first]);

    let expected = ranking.values().iter().take(2).cloned().collect::<Vec<_>>();

    let policy = RecursiveCounterfactualSelectionPolicy::new(2).unwrap();

    let selection = policy.select(&ranking);

    assert_eq!(selection.selected(), expected.as_slice());
}

#[test]
fn beam_larger_than_frontier_keeps_all_values() {
    let first = value(2, 1, &[&[1, 2], &[1, 3]]);

    let second = value(5, 2, &[&[1, 5], &[1, 6]]);

    let ranking = budgeted_ranking(vec![first, second]);

    let policy = RecursiveCounterfactualSelectionPolicy::new(5).unwrap();

    let selection = policy.select(&ranking);

    assert_eq!(selection.len(), 2);

    assert!(!selection.is_full());
}

#[test]
fn selection_preserves_policy_identity() {
    let ranking = budgeted_ranking(vec![value(2, 1, &[&[1, 2], &[1, 3]])]);

    let policy = RecursiveCounterfactualSelectionPolicy::new(4).unwrap();

    let selection = policy.select(&ranking);

    assert_eq!(selection.policy(), policy);
}

#[test]
fn selection_preserves_budget_identity() {
    let ranking = budgeted_ranking(vec![value(2, 1, &[&[1, 2], &[1, 3]])]);

    let expected = ranking.budget();

    let policy = RecursiveCounterfactualSelectionPolicy::new(2).unwrap();

    let selection = policy.select(&ranking);

    assert_eq!(selection.budget(), expected);
}

#[test]
fn selection_does_not_mutate_budgeted_ranking() {
    let ranking = budgeted_ranking(vec![
        value(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]),
        value(5, 2, &[&[1, 5], &[1, 6]]),
    ]);

    let before = ranking.clone();

    let policy = RecursiveCounterfactualSelectionPolicy::new(1).unwrap();

    let _ = policy.select(&ranking);

    assert_eq!(ranking, before);
}

#[test]
fn selection_is_deterministic_from_equivalent_rankings() {
    let first = value(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]);

    let second = value(5, 2, &[&[1, 5], &[1, 6]]);

    let left = budgeted_ranking(vec![first.clone(), second.clone()]);

    let right = budgeted_ranking(vec![second, first]);

    let policy = RecursiveCounterfactualSelectionPolicy::new(2).unwrap();

    assert_eq!(policy.select(&left,), policy.select(&right,));
}

#[test]
fn selected_values_remain_within_local_budget() {
    let ranking = budgeted_ranking(vec![
        value(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]),
        value(5, 2, &[&[1, 5], &[1, 6]]),
    ]);

    let policy = RecursiveCounterfactualSelectionPolicy::new(2).unwrap();

    let selection = policy.select(&ranking);

    for selected in selection.selected() {
        assert!(selection.budget().allows(selected,));
    }
}
