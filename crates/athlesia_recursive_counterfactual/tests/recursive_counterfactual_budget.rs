use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_counterfactual::{
    RecursiveCounterfactualBudget, RecursiveCounterfactualCandidate,
    RecursiveCounterfactualInformationRanking, RecursiveCounterfactualInformationValue,
    RecursiveCounterfactualOutcome, RecursiveCounterfactualProjection,
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

fn value(cost: usize, outcome_spans: &[&[usize]]) -> RecursiveCounterfactualInformationValue {
    let transition = RecursivePlanningTransition::new(state(&[1]), state(&[1, 2])).unwrap();

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

#[test]
fn zero_interaction_budget_is_rejected() {
    assert!(RecursiveCounterfactualBudget::new(0, 4, 6,).is_none());
}

#[test]
fn zero_outcome_budget_is_rejected() {
    assert!(RecursiveCounterfactualBudget::new(4, 0, 6,).is_none());
}

#[test]
fn zero_information_capacity_budget_is_valid() {
    let budget = RecursiveCounterfactualBudget::new(4, 1, 0).unwrap();

    assert_eq!(budget.max_discrimination_capacity(), 0);
}

#[test]
fn budget_preserves_exact_limits() {
    let budget = RecursiveCounterfactualBudget::new(7, 5, 10).unwrap();

    assert_eq!(budget.max_interaction_cost(), 7);

    assert_eq!(budget.max_outcomes(), 5);

    assert_eq!(budget.max_discrimination_capacity(), 10);
}

#[test]
fn value_exactly_on_all_limits_is_allowed() {
    let candidate_value = value(3, &[&[1, 2], &[1, 3], &[1, 4]]);

    assert_eq!(candidate_value.discrimination_capacity(), 3);

    let budget = RecursiveCounterfactualBudget::new(3, 3, 3).unwrap();

    assert!(budget.allows(&candidate_value,));
}

#[test]
fn interaction_cost_over_budget_is_rejected() {
    let candidate_value = value(4, &[&[1, 2], &[1, 3]]);

    let budget = RecursiveCounterfactualBudget::new(3, 4, 10).unwrap();

    assert!(!budget.allows(&candidate_value,));
}

#[test]
fn outcome_count_over_budget_is_rejected() {
    let candidate_value = value(1, &[&[1, 2], &[1, 3], &[1, 4], &[1, 5]]);

    let budget = RecursiveCounterfactualBudget::new(5, 3, 10).unwrap();

    assert!(!budget.allows(&candidate_value,));
}

#[test]
fn discrimination_capacity_over_budget_is_rejected() {
    let candidate_value = value(1, &[&[1, 2], &[1, 3], &[1, 4]]);

    assert_eq!(candidate_value.discrimination_capacity(), 3);

    let budget = RecursiveCounterfactualBudget::new(5, 5, 2).unwrap();

    assert!(!budget.allows(&candidate_value,));
}

#[test]
fn budget_filter_preserves_information_ranking_order() {
    let best_allowed = value(1, &[&[1, 2], &[1, 3]]);

    let rejected = value(8, &[&[1, 2], &[1, 3], &[1, 4], &[1, 5]]);

    let weaker_allowed = value(3, &[&[1, 2], &[1, 3]]);

    let ranking = RecursiveCounterfactualInformationRanking::new(vec![
        weaker_allowed.clone(),
        rejected,
        best_allowed.clone(),
    ]);

    let budget = RecursiveCounterfactualBudget::new(4, 3, 3).unwrap();

    let filtered = budget.apply(&ranking);

    assert_eq!(filtered.len(), 2);

    assert_eq!(filtered.best(), Some(&best_allowed));

    assert_eq!(filtered.values()[1], weaker_allowed);
}

#[test]
fn restrictive_budget_can_produce_empty_ranking() {
    let ranking =
        RecursiveCounterfactualInformationRanking::new(vec![value(3, &[&[1, 2], &[1, 3]])]);

    let budget = RecursiveCounterfactualBudget::new(2, 1, 0).unwrap();

    let filtered = budget.apply(&ranking);

    assert!(filtered.is_empty());

    assert!(filtered.best().is_none());
}

#[test]
fn applying_budget_does_not_mutate_source_ranking() {
    let ranking = RecursiveCounterfactualInformationRanking::new(vec![
        value(1, &[&[1, 2], &[1, 3], &[1, 4]]),
        value(5, &[&[1, 2], &[1, 3]]),
    ]);

    let before = ranking.clone();

    let budget = RecursiveCounterfactualBudget::new(2, 3, 3).unwrap();

    let _ = budget.apply(&ranking);

    assert_eq!(ranking, before);
}

#[test]
fn budget_application_is_deterministic() {
    let first = value(1, &[&[1, 2], &[1, 3], &[1, 4]]);

    let second = value(2, &[&[1, 2], &[1, 3]]);

    let left = RecursiveCounterfactualInformationRanking::new(vec![first.clone(), second.clone()]);

    let right = RecursiveCounterfactualInformationRanking::new(vec![second, first]);

    let budget = RecursiveCounterfactualBudget::new(3, 3, 3).unwrap();

    assert_eq!(budget.apply(&left,), budget.apply(&right,));
}
