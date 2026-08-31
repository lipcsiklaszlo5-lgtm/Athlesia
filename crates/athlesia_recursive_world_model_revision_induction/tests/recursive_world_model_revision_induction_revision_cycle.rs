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

use athlesia_recursive_world_model_revision_induction::{
    RecursiveWorldRevisionInducedStructure, RecursiveWorldRevisionInductionCycle,
    RecursiveWorldRevisionInductionCycleStatus, RecursiveWorldRevisionInductionInput,
    RecursiveWorldRevisionInductionObservationSet,
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

fn induced(
    target: RecursiveWorldRule,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionInducedStructure {
    RecursiveWorldRevisionInducedStructure::induce(RecursiveWorldRevisionInductionInput::new(
        target,
        observation_set(observations),
    ))
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
fn noop_induction_cycle_is_discovery_unavailable() {
    let target = rule(&[1], &[2]);

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        induced(
            target,
            vec![observation(&[1, 3], &[2, 4]), observation(&[1, 5], &[2, 6])],
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionInductionCycleStatus::DiscoveryUnavailable
    );

    assert!(result.discovery_cycle().is_none());

    assert!(!result.has_revision());
}

#[test]
fn invalid_induction_cycle_is_rejected() {
    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        induced(
            rule(&[9], &[10]),
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionInductionCycleStatus::Rejected
    );

    assert!(result.discovery_cycle().is_none());

    assert!(!result.has_revision());
}

#[test]
fn induction_without_evidence_is_inactive() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        induced(
            target,
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionInductionCycleStatus::Inactive
    );

    assert!(result.discovery_cycle().is_some());

    assert!(!result.has_revision());
}

#[test]
fn confirming_evidence_blocks_induction_revision() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &state,
        induced(
            target,
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionInductionCycleStatus::Inactive
    );

    assert!(!result.has_revision());
}

#[test]
fn balanced_evidence_blocks_induction_revision() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 20, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 21, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &state,
        induced(
            target,
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionInductionCycleStatus::Inactive
    );

    assert!(result.pressured_rule().is_none());

    assert!(!result.has_revision());
}

#[test]
fn negative_pressure_can_select_induced_revision() {
    let target = rule(&[9], &[10]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &state,
        induced(
            target.clone(),
            vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionInductionCycleStatus::Revised
    );

    assert_eq!(result.pressured_rule(), Some(&target,));

    assert!(result.has_revision());

    assert!(result.revised_world().unwrap().contains(&replacement,));

    assert!(!result.revised_world().unwrap().contains(&target,));
}

#[test]
fn active_induction_over_budget_has_no_revision() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &state,
        induced(
            target,
            vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
        ),
        RecursiveWorldRevisionBudget::new(1).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionInductionCycleStatus::ActiveNoRevision
    );

    assert!(!result.has_revision());

    assert!(result.revised_world().is_none());
}

#[test]
fn pressure_on_other_rule_keeps_induction_inactive() {
    let target = rule(&[9], &[10]);

    let other = rule(&[30], &[31]);

    let model = RecursiveWorldModel::new(vec![target.clone(), other.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        other.clone(),
        40,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &state,
        induced(
            target,
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionInductionCycleStatus::Inactive
    );

    assert_eq!(result.pressured_rule(), Some(&other,));

    assert!(!result.has_revision());
}

#[test]
fn selected_revision_preserves_induced_hypothesis_identity() {
    let target = rule(&[9], &[10]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &state,
        induced(
            target.clone(),
            vec![observation(&[1, 2], &[3, 4]), observation(&[1, 5], &[3, 6])],
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
fn induction_cycle_preserves_support_count() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        induced(
            target,
            vec![
                observation(&[1, 2], &[3]),
                observation(&[1, 4], &[3]),
                observation(&[1, 5], &[3]),
            ],
        ),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.support_count(), 3);
}

#[test]
fn induction_cycle_preserves_source_observation_provenance() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 4], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        induced(target, vec![first.clone(), second.clone()]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(result.source_observations().contains(&first,));

    assert!(result.source_observations().contains(&second,));
}

#[test]
fn induction_cycle_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let first = observation(&[1, 2], &[3, 4]);

    let second = observation(&[1, 5], &[3, 6]);

    let structure = induced(target.clone(), vec![second.clone(), first.clone()]);

    let model_before = model.clone();

    let state_before = state.clone();

    let structure_before = structure.clone();

    let left = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &state,
        structure.clone(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    let right = RecursiveWorldRevisionInductionCycle::evaluate(
        &model,
        &state,
        induced(target, vec![first, second]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(state, state_before);

    assert_eq!(structure, structure_before);
}
