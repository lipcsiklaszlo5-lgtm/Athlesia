use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_counterfactual::{
    RecursiveCounterfactualCandidate, RecursiveCounterfactualInformationRanking,
    RecursiveCounterfactualInformationValue, RecursiveCounterfactualOutcome,
    RecursiveCounterfactualProjection,
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

fn candidate(from: &[usize], to: &[usize], cost: usize) -> RecursiveCounterfactualCandidate {
    RecursiveCounterfactualCandidate::new(
        RecursivePlanningTransition::new(state(from), state(to)).unwrap(),
        cost,
    )
    .unwrap()
}

fn projection(cost: usize, outcome_spans: &[&[usize]]) -> RecursiveCounterfactualProjection {
    RecursiveCounterfactualProjection::new(
        candidate(&[1], &[1, 2], cost),
        outcome_spans
            .iter()
            .map(|spans| RecursiveCounterfactualOutcome::new(state(spans)))
            .collect(),
    )
    .unwrap()
}

#[test]
fn deterministic_projection_has_zero_information_value() {
    let value = RecursiveCounterfactualInformationValue::evaluate(projection(1, &[&[1, 2]]));

    assert_eq!(value.discrimination_capacity(), 0);

    assert!(!value.is_informative());
}

#[test]
fn two_outcomes_have_one_distinguishable_pair() {
    let value =
        RecursiveCounterfactualInformationValue::evaluate(projection(1, &[&[1, 2], &[1, 3]]));

    assert_eq!(value.discrimination_capacity(), 1);
}

#[test]
fn three_outcomes_have_three_distinguishable_pairs() {
    let value = RecursiveCounterfactualInformationValue::evaluate(projection(
        1,
        &[&[1, 2], &[1, 3], &[1, 4]],
    ));

    assert_eq!(value.discrimination_capacity(), 3);
}

#[test]
fn four_outcomes_have_six_distinguishable_pairs() {
    let value = RecursiveCounterfactualInformationValue::evaluate(projection(
        1,
        &[&[1, 2], &[1, 3], &[1, 4], &[1, 5]],
    ));

    assert_eq!(value.discrimination_capacity(), 6);
}

#[test]
fn information_value_preserves_projection_identity() {
    let expected = projection(3, &[&[1, 2], &[1, 3]]);

    let value = RecursiveCounterfactualInformationValue::evaluate(expected.clone());

    assert_eq!(value.projection(), &expected);
}

#[test]
fn information_value_preserves_interaction_cost() {
    let value =
        RecursiveCounterfactualInformationValue::evaluate(projection(7, &[&[1, 2], &[1, 3]]));

    assert_eq!(value.interaction_cost(), 7);
}

#[test]
fn higher_information_per_cost_ranks_first() {
    let weaker =
        RecursiveCounterfactualInformationValue::evaluate(projection(4, &[&[1, 2], &[1, 3]]));

    let stronger =
        RecursiveCounterfactualInformationValue::evaluate(projection(2, &[&[1, 2], &[1, 3]]));

    let ranking = RecursiveCounterfactualInformationRanking::new(vec![weaker, stronger.clone()]);

    assert_eq!(ranking.best(), Some(&stronger));
}

#[test]
fn equal_efficiency_prefers_higher_absolute_information() {
    let low = RecursiveCounterfactualInformationValue::evaluate(projection(1, &[&[1, 2], &[1, 3]]));

    let high = RecursiveCounterfactualInformationValue::evaluate(projection(
        3,
        &[&[1, 2], &[1, 3], &[1, 4]],
    ));

    assert_eq!(low.discrimination_capacity(), 1);

    assert_eq!(high.discrimination_capacity(), 3);

    let ranking = RecursiveCounterfactualInformationRanking::new(vec![low, high.clone()]);

    assert_eq!(ranking.best(), Some(&high));
}

#[test]
fn equal_information_prefers_lower_interaction_cost() {
    let expensive =
        RecursiveCounterfactualInformationValue::evaluate(projection(5, &[&[1, 2], &[1, 3]]));

    let cheap =
        RecursiveCounterfactualInformationValue::evaluate(projection(2, &[&[1, 2], &[1, 3]]));

    let ranking = RecursiveCounterfactualInformationRanking::new(vec![expensive, cheap.clone()]);

    assert_eq!(ranking.best(), Some(&cheap));
}

#[test]
fn ranking_is_deterministic_under_input_order() {
    let first = RecursiveCounterfactualInformationValue::evaluate(
        RecursiveCounterfactualProjection::new(
            candidate(&[1], &[1, 2], 2),
            vec![
                RecursiveCounterfactualOutcome::new(state(&[1, 2])),
                RecursiveCounterfactualOutcome::new(state(&[1, 3])),
            ],
        )
        .unwrap(),
    );

    let second = RecursiveCounterfactualInformationValue::evaluate(
        RecursiveCounterfactualProjection::new(
            candidate(&[1], &[1, 4], 2),
            vec![
                RecursiveCounterfactualOutcome::new(state(&[1, 4])),
                RecursiveCounterfactualOutcome::new(state(&[1, 5])),
            ],
        )
        .unwrap(),
    );

    let left = RecursiveCounterfactualInformationRanking::new(vec![first.clone(), second.clone()]);

    let right = RecursiveCounterfactualInformationRanking::new(vec![second, first]);

    assert_eq!(left, right);
}

#[test]
fn empty_information_ranking_has_no_best_value() {
    let ranking = RecursiveCounterfactualInformationRanking::new(Vec::new());

    assert!(ranking.is_empty());

    assert_eq!(ranking.len(), 0);

    assert!(ranking.best().is_none());
}

#[test]
fn recursive_depth_identity_survives_information_evaluation() {
    let child = RecursiveConcept::new(vec![
        unit(1),
        RecursiveUnit::CrossLevel(
            athlesia_cross_level::CrossLevelConcept::new(vec![
                AbstractionUnit::Structural(structural(2)),
                AbstractionUnit::Hierarchical(
                    athlesia_hierarchy::HierarchicalConcept::new(vec![
                        structural(3),
                        structural(4),
                    ])
                    .unwrap(),
                ),
            ])
            .unwrap(),
        ),
    ])
    .unwrap();

    let deep =
        RecursiveConcept::new(vec![unit(8), RecursiveUnit::Recursive(Box::new(child))]).unwrap();

    let predicted = RecursivePlanningState::new(vec![RecursiveUnit::Recursive(Box::new(deep))]);

    let original_projection = RecursiveCounterfactualProjection::new(
        candidate(&[1], &[1, 2], 1),
        vec![
            RecursiveCounterfactualOutcome::new(predicted.clone()),
            RecursiveCounterfactualOutcome::new(state(&[1, 9])),
        ],
    )
    .unwrap();

    let value = RecursiveCounterfactualInformationValue::evaluate(original_projection);

    assert!(value.projection().contains_state(&predicted,));

    assert_eq!(value.discrimination_capacity(), 1);
}
