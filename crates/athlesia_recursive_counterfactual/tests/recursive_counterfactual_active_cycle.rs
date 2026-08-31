use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_counterfactual::{
    RecursiveCounterfactualActiveCycle, RecursiveCounterfactualBudget,
    RecursiveCounterfactualCandidate, RecursiveCounterfactualOutcome,
    RecursiveCounterfactualProjection, RecursiveCounterfactualProjectionSet,
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

fn projection(
    target: usize,
    cost: usize,
    outcomes: &[&[usize]],
) -> RecursiveCounterfactualProjection {
    let transition = RecursivePlanningTransition::new(state(&[1]), state(&[1, target])).unwrap();

    let candidate = RecursiveCounterfactualCandidate::new(transition, cost).unwrap();

    RecursiveCounterfactualProjection::new(
        candidate,
        outcomes
            .iter()
            .map(|spans| RecursiveCounterfactualOutcome::new(state(spans)))
            .collect(),
    )
    .unwrap()
}

fn standard_budget() -> RecursiveCounterfactualBudget {
    RecursiveCounterfactualBudget::new(5, 4, 6).unwrap()
}

fn beam(width: usize) -> RecursiveCounterfactualSelectionPolicy {
    RecursiveCounterfactualSelectionPolicy::new(width).unwrap()
}

#[test]
fn empty_projection_set_produces_empty_cycle() {
    let projections = RecursiveCounterfactualProjectionSet::new(Vec::new());

    let result =
        RecursiveCounterfactualActiveCycle::evaluate(&projections, standard_budget(), beam(2));

    assert!(result.ranking().is_empty());

    assert!(result.budgeted().is_empty());

    assert!(result.selection().is_empty());

    assert!(result.best().is_none());
}

#[test]
fn active_cycle_evaluates_all_input_projections() {
    let projections = RecursiveCounterfactualProjectionSet::new(vec![
        projection(2, 1, &[&[1, 2], &[1, 3]]),
        projection(4, 2, &[&[1, 4], &[1, 5]]),
        projection(6, 3, &[&[1, 6]]),
    ]);

    let result =
        RecursiveCounterfactualActiveCycle::evaluate(&projections, standard_budget(), beam(3));

    assert_eq!(result.ranking().len(), 3);
}

#[test]
fn active_cycle_applies_information_ranking() {
    let weak = projection(2, 4, &[&[1, 2], &[1, 3]]);

    let strong = projection(5, 1, &[&[1, 5], &[1, 6], &[1, 7]]);

    let projections = RecursiveCounterfactualProjectionSet::new(vec![weak, strong.clone()]);

    let result =
        RecursiveCounterfactualActiveCycle::evaluate(&projections, standard_budget(), beam(2));

    assert_eq!(result.ranking().best().unwrap().projection(), &strong);
}

#[test]
fn active_cycle_applies_hard_budget() {
    let allowed = projection(2, 2, &[&[1, 2], &[1, 3]]);

    let too_expensive = projection(5, 8, &[&[1, 5], &[1, 6], &[1, 7]]);

    let projections =
        RecursiveCounterfactualProjectionSet::new(vec![allowed.clone(), too_expensive]);

    let budget = RecursiveCounterfactualBudget::new(3, 4, 6).unwrap();

    let result = RecursiveCounterfactualActiveCycle::evaluate(&projections, budget, beam(4));

    assert_eq!(result.budgeted().len(), 1);

    assert_eq!(result.budgeted().best().unwrap().projection(), &allowed);
}

#[test]
fn active_cycle_applies_beam_width() {
    let projections = RecursiveCounterfactualProjectionSet::new(vec![
        projection(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]),
        projection(5, 2, &[&[1, 5], &[1, 6]]),
        projection(8, 3, &[&[1, 8], &[1, 9]]),
    ]);

    let result =
        RecursiveCounterfactualActiveCycle::evaluate(&projections, standard_budget(), beam(2));

    assert_eq!(result.selection().len(), 2);

    assert!(result.selection().is_full());
}

#[test]
fn best_value_matches_selected_frontier_head() {
    let projections = RecursiveCounterfactualProjectionSet::new(vec![
        projection(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]),
        projection(5, 2, &[&[1, 5], &[1, 6]]),
    ]);

    let result =
        RecursiveCounterfactualActiveCycle::evaluate(&projections, standard_budget(), beam(2));

    assert_eq!(result.best(), result.selection().best());
}

#[test]
fn restrictive_budget_can_empty_selected_frontier() {
    let projections =
        RecursiveCounterfactualProjectionSet::new(vec![projection(2, 3, &[&[1, 2], &[1, 3]])]);

    let budget = RecursiveCounterfactualBudget::new(2, 1, 0).unwrap();

    let result = RecursiveCounterfactualActiveCycle::evaluate(&projections, budget, beam(2));

    assert_eq!(result.ranking().len(), 1);

    assert!(result.budgeted().is_empty());

    assert!(result.selection().is_empty());
}

#[test]
fn deterministic_projection_remains_zero_information() {
    let deterministic = projection(2, 1, &[&[1, 2]]);

    let projections = RecursiveCounterfactualProjectionSet::new(vec![deterministic]);

    let result =
        RecursiveCounterfactualActiveCycle::evaluate(&projections, standard_budget(), beam(1));

    assert_eq!(
        result.ranking().best().unwrap().discrimination_capacity(),
        0
    );
}

#[test]
fn active_cycle_does_not_mutate_projection_set() {
    let projections = RecursiveCounterfactualProjectionSet::new(vec![
        projection(2, 1, &[&[1, 2], &[1, 3]]),
        projection(5, 2, &[&[1, 5], &[1, 6]]),
    ]);

    let before = projections.clone();

    let _ = RecursiveCounterfactualActiveCycle::evaluate(&projections, standard_budget(), beam(1));

    assert_eq!(projections, before);
}

#[test]
fn active_cycle_is_deterministic() {
    let first = projection(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]);

    let second = projection(5, 2, &[&[1, 5], &[1, 6]]);

    let left = RecursiveCounterfactualProjectionSet::new(vec![first.clone(), second.clone()]);

    let right = RecursiveCounterfactualProjectionSet::new(vec![second, first]);

    let left_result =
        RecursiveCounterfactualActiveCycle::evaluate(&left, standard_budget(), beam(2));

    let right_result =
        RecursiveCounterfactualActiveCycle::evaluate(&right, standard_budget(), beam(2));

    assert_eq!(left_result, right_result);
}

#[test]
fn selected_values_respect_budget_and_beam_simultaneously() {
    let projections = RecursiveCounterfactualProjectionSet::new(vec![
        projection(2, 1, &[&[1, 2], &[1, 3], &[1, 4]]),
        projection(5, 2, &[&[1, 5], &[1, 6]]),
        projection(8, 9, &[&[1, 8], &[1, 9], &[1, 10]]),
    ]);

    let budget = RecursiveCounterfactualBudget::new(3, 4, 6).unwrap();

    let policy = beam(1);

    let result = RecursiveCounterfactualActiveCycle::evaluate(&projections, budget, policy);

    assert_eq!(result.selection().len(), 1);

    for selected in result.selection().selected() {
        assert!(budget.allows(selected,));
    }

    assert_eq!(result.selection().policy().beam_width(), 1);
}

#[test]
fn active_cycle_preserves_candidate_transition_identity() {
    let expected = projection(7, 1, &[&[1, 7], &[1, 8], &[1, 9]]);

    let expected_transition = expected.candidate().transition().clone();

    let projections = RecursiveCounterfactualProjectionSet::new(vec![expected]);

    let result =
        RecursiveCounterfactualActiveCycle::evaluate(&projections, standard_budget(), beam(1));

    assert_eq!(
        result.best().unwrap().projection().candidate().transition(),
        &expected_transition
    );
}
