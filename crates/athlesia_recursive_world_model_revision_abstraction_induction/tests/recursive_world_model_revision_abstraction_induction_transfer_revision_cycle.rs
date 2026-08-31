use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldModel, RecursiveWorldRevisionBudget, RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionTransferCycle,
    RecursiveWorldRevisionAbstractionTransferCycleResult,
    RecursiveWorldRevisionAbstractionTransferCycleStatus,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

fn budget_high() -> RecursiveWorldRevisionBudget {
    RecursiveWorldRevisionBudget::new(100).unwrap()
}

fn budget_low() -> RecursiveWorldRevisionBudget {
    RecursiveWorldRevisionBudget::new(1).unwrap()
}

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(
        StructuralConcept::with_sequence_length(
            vec![PrimitiveSignature::new(RelationKind::Equal, span)],
            8,
        ),
    ))
}

fn rule(premises: &[usize], conclusions: &[usize]) -> RecursiveWorldRule {
    RecursiveWorldRule::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn model(rules: Vec<RecursiveWorldRule>) -> RecursiveWorldModel {
    RecursiveWorldModel::new(rules)
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

fn induction_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1], &[10]),
        observation(&[2], &[10]),
        observation(&[1], &[20]),
        observation(&[2], &[20]),
    ])
}

fn deterministic_transfer_source() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 50], &[10, 60]),
        observation(&[1, 51], &[10, 61]),
    ])
}

fn evidence(
    target: RecursiveWorldRule,
    observed: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(target, unit(observed), kind)
}

fn violating(target: RecursiveWorldRule) -> RecursiveWorldEvidenceState {
    RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target,
        700,
        RecursiveWorldEvidenceKind::Violating,
    ))
}

#[test]
fn unavailable_discovery_never_enters_revision_cycle() {
    let target = rule(&[1], &[10]);

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![target.clone()]),
        violating(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
        budget_high(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionTransferCycleStatus::DiscoveryUnavailable
    );

    assert!(result.cycle().is_none());

    assert!(!result.has_revision());
}

#[test]
fn rejected_validation_never_enters_revision_cycle() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![rule(&[30], &[40])]),
        violating(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
        budget_high(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionTransferCycleStatus::Rejected
    );

    assert!(result.cycle().is_none());
}

#[test]
fn accepted_but_unpressured_transfer_is_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        induction_source(),
        deterministic_transfer_source(),
        budget_high(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionTransferCycleStatus::Inactive
    );

    assert!(result.cycle().is_some());

    assert!(!result.has_revision());
}

#[test]
fn active_affordable_transfer_revises_world_model() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![target.clone()]),
        violating(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
        budget_high(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionTransferCycleStatus::Revised
    );

    assert!(result.has_revision());

    assert!(result.cycle().unwrap().has_revision());
}

#[test]
fn low_budget_blocks_active_revision() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![target.clone()]),
        violating(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
        budget_low(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionTransferCycleStatus::ActiveNoRevision
    );

    assert!(!result.has_revision());
}

#[test]
fn revised_cycle_preserves_target_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![target.clone()]),
        violating(target.clone()),
        target.clone(),
        induction_source(),
        deterministic_transfer_source(),
        budget_high(),
    );

    assert_eq!(result.target(), &target);

    assert_eq!(result.hypothesis().unwrap().target(), &target);
}

#[test]
fn revised_cycle_preserves_transfer_realization_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![target.clone()]),
        violating(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
        budget_high(),
    );

    assert_eq!(
        result.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );

    assert_eq!(
        result.hypothesis().unwrap().replacement(),
        &rule(&[1], &[10],)
    );
}

#[test]
fn revised_cycle_selects_transfer_hypothesis() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![target.clone()]),
        violating(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
        budget_high(),
    );

    assert_eq!(result.cycle().unwrap().selected_hypotheses().len(), 1);

    assert_eq!(
        &result.cycle().unwrap().selected_hypotheses()[0],
        result.hypothesis().unwrap()
    );
}

#[test]
fn cycle_preserves_learning_and_application_provenance() {
    let target = rule(&[9], &[99]);

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    let induction_before = induction.clone();

    let transfer_before = transfer.clone();

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![target.clone()]),
        violating(target.clone()),
        target,
        induction,
        transfer,
        budget_high(),
    );

    assert_eq!(result.induction_observations(), &induction_before);

    assert_eq!(result.transfer_observations(), &transfer_before);
}

#[test]
fn cycle_preserves_learned_abstraction_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![target.clone()]),
        violating(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
        budget_high(),
    );

    assert_eq!(result.consensus().unwrap().observation_count(), 4);

    assert_eq!(result.vocabulary().unwrap().classes().len(), 2);
}

#[test]
fn transfer_cycle_facade_matches_direct_evaluation() {
    let target = rule(&[9], &[99]);

    let world = model(vec![target.clone()]);

    let evidence_state = violating(target.clone());

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionTransferCycle::evaluate(
            world.clone(),
            evidence_state.clone(),
            target.clone(),
            induction.clone(),
            transfer.clone(),
            budget_high(),
        ),
        RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
            world,
            evidence_state,
            target,
            induction,
            transfer,
            budget_high(),
        )
    );
}

#[test]
fn transfer_revision_cycle_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let world = model(vec![target.clone(), rule(&[30], &[40])]);

    let evidence_state = violating(target.clone());

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    let world_before = world.clone();

    let evidence_before = evidence_state.clone();

    let induction_before = induction.clone();

    let transfer_before = transfer.clone();

    let left = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        world.clone(),
        evidence_state.clone(),
        target.clone(),
        induction.clone(),
        transfer.clone(),
        budget_high(),
    );

    let right = RecursiveWorldRevisionAbstractionTransferCycleResult::evaluate(
        model(vec![rule(&[30], &[40]), target.clone()]),
        violating(target.clone()),
        target,
        observation_set(vec![
            observation(&[2], &[20]),
            observation(&[1], &[20]),
            observation(&[2], &[10]),
            observation(&[1], &[10]),
        ]),
        observation_set(vec![
            observation(&[1, 51], &[10, 61]),
            observation(&[1, 50], &[10, 60]),
        ]),
        budget_high(),
    );

    assert_eq!(left, right);

    assert_eq!(world, world_before);

    assert_eq!(evidence_state, evidence_before);

    assert_eq!(induction, induction_before);

    assert_eq!(transfer, transfer_before);

    assert_eq!(
        left.status(),
        RecursiveWorldRevisionAbstractionTransferCycleStatus::Revised
    );

    assert!(left.has_revision());
}
