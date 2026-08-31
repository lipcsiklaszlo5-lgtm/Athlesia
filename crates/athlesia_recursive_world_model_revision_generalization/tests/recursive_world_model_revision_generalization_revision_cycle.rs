use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldModel, RecursiveWorldRevisionBudget, RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_generalization::{
    RecursiveWorldRevisionGeneralizationCycle, RecursiveWorldRevisionGeneralizationCycleStatus,
    RecursiveWorldRevisionGeneralizationInput, RecursiveWorldRevisionGeneralizationThreshold,
    RecursiveWorldRevisionGeneralizedStructure,
};

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

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

fn observation(
    premises: &[usize],
    conclusions: &[usize],
) -> RecursiveWorldRevisionDiscoveryObservation {
    RecursiveWorldRevisionDiscoveryObservation::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn observation_set(
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionInductionObservationSet {
    RecursiveWorldRevisionInductionObservationSet::new(observations).unwrap()
}

fn threshold(
    minimum_support: usize,
    observation_count: usize,
) -> RecursiveWorldRevisionGeneralizationThreshold {
    RecursiveWorldRevisionGeneralizationThreshold::new(minimum_support, observation_count).unwrap()
}

fn generalized(
    target: RecursiveWorldRule,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
    minimum_support: usize,
) -> RecursiveWorldRevisionGeneralizedStructure {
    let set = observation_set(observations);

    let count = set.len();

    RecursiveWorldRevisionGeneralizedStructure::generalize(
        RecursiveWorldRevisionGeneralizationInput::new(
            target,
            set,
            threshold(minimum_support, count),
        )
        .unwrap(),
    )
    .unwrap()
}

fn evidence(
    target: RecursiveWorldRule,
    observation: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(target, unit(observation), kind)
}

#[test]
fn noop_generalization_cycle_is_discovery_unavailable() {
    let target = rule(&[1], &[2]);

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        generalized(
            target,
            vec![
                observation(&[1, 3], &[2, 4]),
                observation(&[1, 5], &[2, 6]),
                observation(&[1, 7], &[2, 8]),
            ],
            3,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionGeneralizationCycleStatus::DiscoveryUnavailable
    );

    assert!(result.discovery_cycle().is_none());

    assert!(!result.has_revision());
}

#[test]
fn invalid_generalization_cycle_is_rejected() {
    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        generalized(
            rule(&[9], &[10]),
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionGeneralizationCycleStatus::Rejected
    );

    assert!(result.discovery_cycle().is_none());

    assert!(!result.has_revision());
}

#[test]
fn generalization_without_evidence_is_inactive() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionGeneralizationCycleStatus::Inactive
    );

    assert!(result.discovery_cycle().is_some());

    assert!(!result.has_revision());
}

#[test]
fn confirming_evidence_blocks_generalized_revision() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &evidence_state,
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionGeneralizationCycleStatus::Inactive
    );

    assert!(!result.has_revision());
}

#[test]
fn balanced_evidence_blocks_generalized_revision() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 20, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 21, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &evidence_state,
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionGeneralizationCycleStatus::Inactive
    );

    assert!(result.pressured_rule().is_none());

    assert!(!result.has_revision());
}

#[test]
fn negative_pressure_can_select_generalized_revision() {
    let target = rule(&[9], &[10]);

    let replacement = rule(&[1, 2], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &evidence_state,
        generalized(
            target.clone(),
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionGeneralizationCycleStatus::Revised
    );

    assert_eq!(result.pressured_rule(), Some(&target,));

    assert!(result.has_revision());

    assert!(result.revised_world().unwrap().contains(&replacement,));

    assert!(!result.revised_world().unwrap().contains(&target,));
}

#[test]
fn active_generalization_over_budget_has_no_revision() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &evidence_state,
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
        RecursiveWorldRevisionBudget::new(1).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionGeneralizationCycleStatus::ActiveNoRevision
    );

    assert!(!result.has_revision());

    assert!(result.revised_world().is_none());
}

#[test]
fn pressure_on_other_rule_keeps_generalization_inactive() {
    let target = rule(&[9], &[10]);

    let other = rule(&[30], &[31]);

    let model = RecursiveWorldModel::new(vec![target.clone(), other.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        other.clone(),
        40,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &evidence_state,
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionGeneralizationCycleStatus::Inactive
    );

    assert_eq!(result.pressured_rule(), Some(&other,));

    assert!(!result.has_revision());
}

#[test]
fn selected_revision_preserves_generalized_hypothesis_identity() {
    let target = rule(&[9], &[10]);

    let replacement = rule(&[1, 2], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &evidence_state,
        generalized(
            target.clone(),
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    let hypothesis = result.selected_hypothesis().unwrap();

    assert_eq!(hypothesis.target(), &target);

    assert_eq!(hypothesis.replacement(), &replacement);

    assert_eq!(
        result.selected_revision().unwrap().replacement(),
        &replacement
    );
}

#[test]
fn generalization_cycle_preserves_threshold_and_support_metadata() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        generalized(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 2, 4], &[3, 5]),
                observation(&[1, 6], &[3]),
            ],
            2,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.threshold().minimum_support(), 2);

    assert_eq!(result.support_count(), 3);

    assert_eq!(result.premise_support(&unit(1,),), 3);

    assert_eq!(result.premise_support(&unit(2,),), 2);

    assert_eq!(result.conclusion_support(&unit(3,),), 3);
}

#[test]
fn generalization_cycle_preserves_source_observation_provenance() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 2, 4], &[3, 5]);

    let third = observation(&[1, 6], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        generalized(
            target,
            vec![third.clone(), first.clone(), second.clone()],
            2,
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(result.source_observations().contains(&first,));

    assert!(result.source_observations().contains(&second,));

    assert!(result.source_observations().contains(&third,));
}

#[test]
fn generalization_cycle_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 2, 4], &[3, 5]);

    let third = observation(&[1, 6], &[3]);

    let structure = generalized(
        target.clone(),
        vec![third.clone(), first.clone(), second.clone()],
        2,
    );

    let model_before = model.clone();

    let evidence_before = evidence_state.clone();

    let structure_before = structure.clone();

    let left = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &evidence_state,
        structure.clone(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    let right = RecursiveWorldRevisionGeneralizationCycle::evaluate(
        &model,
        &evidence_state,
        generalized(target, vec![second, third, first], 2),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(evidence_state, evidence_before);

    assert_eq!(structure, structure_before);
}
