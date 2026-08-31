use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_induction::{
    RecursiveWorldRevisionInducedStructure, RecursiveWorldRevisionInductionEvidenceScope,
    RecursiveWorldRevisionInductionEvidenceScoper, RecursiveWorldRevisionInductionEvidenceStatus,
    RecursiveWorldRevisionInductionInput, RecursiveWorldRevisionInductionObservationSet,
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
fn noop_induction_scope_is_discovery_unavailable() {
    let target = rule(&[1], &[2]);

    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        induced(
            target,
            vec![observation(&[1, 3], &[2, 4]), observation(&[1, 5], &[2, 6])],
        ),
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionInductionEvidenceStatus::DiscoveryUnavailable
    );

    assert!(scope.is_discovery_unavailable());

    assert!(scope.discovery_scope().is_none());
}

#[test]
fn invalid_induction_scope_is_rejected() {
    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        induced(
            rule(&[9], &[10]),
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionInductionEvidenceStatus::Rejected
    );

    assert!(scope.is_rejected());

    assert!(scope.discovery_scope().is_none());
}

#[test]
fn accepted_induction_without_evidence_is_inactive() {
    let target = rule(&[9], &[10]);

    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        induced(
            target,
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionInductionEvidenceStatus::Inactive
    );

    assert!(scope.is_inactive());

    assert!(scope.inactive_hypothesis().is_some());
}

#[test]
fn confirming_evidence_keeps_induction_inactive() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
        &model,
        &evidence_state,
        induced(
            target,
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
    );

    assert!(scope.is_inactive());

    assert!(scope.active_hypothesis().is_none());
}

#[test]
fn balanced_evidence_keeps_induction_inactive() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 20, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 21, RecursiveWorldEvidenceKind::Violating),
    ]);

    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
        &model,
        &evidence_state,
        induced(
            target,
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
    );

    assert!(scope.is_inactive());

    assert!(scope.pressured_rule().is_none());
}

#[test]
fn negative_pressure_activates_matching_induction() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
        &model,
        &evidence_state,
        induced(
            target.clone(),
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionInductionEvidenceStatus::Active
    );

    assert!(scope.is_active());

    assert_eq!(scope.pressured_rule(), Some(&target,));

    assert!(scope.active_hypothesis().is_some());
}

#[test]
fn pressure_on_other_rule_does_not_activate_induction() {
    let target = rule(&[9], &[10]);

    let other = rule(&[30], &[31]);

    let model = RecursiveWorldModel::new(vec![target.clone(), other.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        other.clone(),
        40,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
        &model,
        &evidence_state,
        induced(
            target,
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
    );

    assert!(scope.is_inactive());

    assert_eq!(scope.pressured_rule(), Some(&other,));

    assert!(scope.active_hypothesis().is_none());
}

#[test]
fn induction_evidence_scope_preserves_support_count() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
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
    );

    assert_eq!(scope.support_count(), 3);
}

#[test]
fn induction_evidence_scope_preserves_source_provenance() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 4], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        induced(target, vec![first.clone(), second.clone()]),
    );

    assert!(scope.source_observations().contains(&first,));

    assert!(scope.source_observations().contains(&second,));
}

#[test]
fn induction_evidence_scope_preserves_m37_scope_identity() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionInductionEvidenceScope::new(
        &model,
        &evidence_state,
        induced(
            target,
            vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
        ),
    );

    assert_eq!(scope.discovery_scope().unwrap().active_count(), 1);

    assert_eq!(scope.discovery_scope().unwrap().inactive_count(), 0);
}

#[test]
fn induction_evidence_scoper_facade_matches_direct_scope() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let structure = induced(
        target,
        vec![observation(&[1, 2], &[3]), observation(&[1, 4], &[3])],
    );

    assert_eq!(
        RecursiveWorldRevisionInductionEvidenceScoper::scope(
            &model,
            &evidence_state,
            structure.clone(),
        ),
        RecursiveWorldRevisionInductionEvidenceScope::new(&model, &evidence_state, structure,)
    );
}

#[test]
fn induction_evidence_scope_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 4], &[3]);

    let structure = induced(target.clone(), vec![second.clone(), first.clone()]);

    let model_before = model.clone();

    let evidence_before = evidence_state.clone();

    let structure_before = structure.clone();

    let left = RecursiveWorldRevisionInductionEvidenceScope::new(
        &model,
        &evidence_state,
        structure.clone(),
    );

    let right = RecursiveWorldRevisionInductionEvidenceScope::new(
        &model,
        &evidence_state,
        induced(target, vec![first, second]),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(evidence_state, evidence_before);

    assert_eq!(structure, structure_before);
}
