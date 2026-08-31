use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionTransferEvidenceScope,
    RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus,
    RecursiveWorldRevisionAbstractionTransferEvidenceScoper,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

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

fn violating_target_evidence(target: RecursiveWorldRule) -> RecursiveWorldEvidenceState {
    RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target,
        700,
        RecursiveWorldEvidenceKind::Violating,
    ))
}

fn confirming_target_evidence(target: RecursiveWorldRule) -> RecursiveWorldEvidenceState {
    RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target,
        700,
        RecursiveWorldEvidenceKind::Confirming,
    ))
}

#[test]
fn unavailable_discovery_never_reaches_evidence_scope() {
    let target = rule(&[1], &[10]);

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![target.clone()]),
        violating_target_evidence(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        scoped.status(),
        RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::DiscoveryUnavailable
    );

    assert!(scoped.scope_result().is_none());
}

#[test]
fn rejected_validation_never_reaches_evidence_scope() {
    let target = rule(&[9], &[99]);

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![rule(&[30], &[40])]),
        violating_target_evidence(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        scoped.status(),
        RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Rejected
    );

    assert!(scoped.scope_result().is_none());
}

#[test]
fn accepted_transfer_without_evidence_is_inactive() {
    let target = rule(&[9], &[99]);

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        scoped.status(),
        RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Inactive
    );

    assert!(scoped.is_inactive());

    assert!(!scoped.is_active());
}

#[test]
fn confirming_target_evidence_keeps_transfer_inactive() {
    let target = rule(&[9], &[99]);

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![target.clone()]),
        confirming_target_evidence(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        scoped.status(),
        RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Inactive
    );

    assert!(scoped.active_hypothesis().is_none());
}

#[test]
fn violating_target_evidence_activates_transfer_hypothesis() {
    let target = rule(&[9], &[99]);

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![target.clone()]),
        violating_target_evidence(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        scoped.status(),
        RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Active
    );

    assert!(scoped.is_active());

    assert!(scoped.active_hypothesis().is_some());
}

#[test]
fn evidence_pressure_on_other_rule_does_not_activate_target() {
    let target = rule(&[9], &[99]);

    let other = rule(&[30], &[40]);

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![target.clone(), other.clone()]),
        violating_target_evidence(other),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        scoped.status(),
        RecursiveWorldRevisionAbstractionTransferEvidenceScopeStatus::Inactive
    );

    assert!(scoped.active_hypothesis().is_none());
}

#[test]
fn active_scope_preserves_exact_target_pressure_identity() {
    let target = rule(&[9], &[99]);

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![target.clone()]),
        violating_target_evidence(target.clone()),
        target.clone(),
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(scoped.pressured_rule(), Some(&target,));

    assert_eq!(scoped.target(), &target);
}

#[test]
fn active_scope_preserves_accepted_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![target.clone()]),
        violating_target_evidence(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(scoped.active_hypothesis(), scoped.hypothesis());

    assert_eq!(
        scoped.active_hypothesis().unwrap().replacement(),
        &rule(&[1], &[10],)
    );
}

#[test]
fn evidence_scope_preserves_transfer_realization_identity() {
    let target = rule(&[9], &[99]);

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![target.clone()]),
        violating_target_evidence(target.clone()),
        target,
        induction_source(),
        deterministic_transfer_source(),
    );

    assert_eq!(
        scoped.realized_observation(),
        Some(&observation(&[1], &[10],),)
    );

    assert_eq!(scoped.replacement(), Some(&rule(&[1], &[10],),));
}

#[test]
fn evidence_scope_preserves_learning_and_application_provenance() {
    let target = rule(&[9], &[99]);

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    let induction_before = induction.clone();

    let transfer_before = transfer.clone();

    let scoped = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![target.clone()]),
        violating_target_evidence(target.clone()),
        target,
        induction,
        transfer,
    );

    assert_eq!(scoped.induction_observations(), &induction_before);

    assert_eq!(scoped.transfer_observations(), &transfer_before);

    assert_eq!(scoped.consensus().unwrap().observation_count(), 4);

    assert_eq!(scoped.vocabulary().unwrap().classes().len(), 2);
}

#[test]
fn transfer_evidence_scoper_facade_matches_direct_scope() {
    let target = rule(&[9], &[99]);

    let world = model(vec![target.clone()]);

    let evidence_state = violating_target_evidence(target.clone());

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    assert_eq!(
        RecursiveWorldRevisionAbstractionTransferEvidenceScoper::scope(
            world.clone(),
            evidence_state.clone(),
            target.clone(),
            induction.clone(),
            transfer.clone(),
        ),
        RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
            world,
            evidence_state,
            target,
            induction,
            transfer,
        )
    );
}

#[test]
fn transfer_evidence_scope_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let world = model(vec![target.clone(), rule(&[30], &[40])]);

    let evidence_state = violating_target_evidence(target.clone());

    let induction = induction_source();

    let transfer = deterministic_transfer_source();

    let world_before = world.clone();

    let evidence_before = evidence_state.clone();

    let induction_before = induction.clone();

    let transfer_before = transfer.clone();

    let left = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        world.clone(),
        evidence_state.clone(),
        target.clone(),
        induction.clone(),
        transfer.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionTransferEvidenceScope::scope(
        model(vec![rule(&[30], &[40]), target.clone()]),
        RecursiveWorldEvidenceState::empty().accumulate(evidence(
            target.clone(),
            700,
            RecursiveWorldEvidenceKind::Violating,
        )),
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
    );

    assert_eq!(left, right);

    assert_eq!(world, world_before);

    assert_eq!(evidence_state, evidence_before);

    assert_eq!(induction, induction_before);

    assert_eq!(transfer, transfer_before);

    assert!(left.is_active());
}
