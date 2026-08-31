use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

use athlesia_recursive_world_model_revision_generalization::{
    RecursiveWorldRevisionGeneralizationEvidenceScope,
    RecursiveWorldRevisionGeneralizationEvidenceScoper,
    RecursiveWorldRevisionGeneralizationEvidenceStatus, RecursiveWorldRevisionGeneralizationInput,
    RecursiveWorldRevisionGeneralizationThreshold, RecursiveWorldRevisionGeneralizedStructure,
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
fn noop_generalization_scope_is_discovery_unavailable() {
    let target = rule(&[1], &[2]);

    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
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
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionGeneralizationEvidenceStatus::DiscoveryUnavailable
    );

    assert!(scope.is_discovery_unavailable());

    assert!(scope.discovery_scope().is_none());
}

#[test]
fn invalid_generalization_scope_is_rejected() {
    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
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
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionGeneralizationEvidenceStatus::Rejected
    );

    assert!(scope.is_rejected());

    assert!(scope.discovery_scope().is_none());
}

#[test]
fn accepted_generalization_without_evidence_is_inactive() {
    let target = rule(&[9], &[10]);

    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
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
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionGeneralizationEvidenceStatus::Inactive
    );

    assert!(scope.is_inactive());

    assert!(scope.inactive_hypothesis().is_some());
}

#[test]
fn confirming_evidence_keeps_generalization_inactive() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
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
    );

    assert!(scope.is_inactive());

    assert!(scope.active_hypothesis().is_none());
}

#[test]
fn balanced_evidence_keeps_generalization_inactive() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 20, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 21, RecursiveWorldEvidenceKind::Violating),
    ]);

    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
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
    );

    assert!(scope.is_inactive());

    assert!(scope.pressured_rule().is_none());
}

#[test]
fn negative_pressure_activates_matching_generalization() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
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
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionGeneralizationEvidenceStatus::Active
    );

    assert!(scope.is_active());

    assert_eq!(scope.pressured_rule(), Some(&target,));

    assert!(scope.active_hypothesis().is_some());
}

#[test]
fn pressure_on_other_rule_does_not_activate_generalization() {
    let target = rule(&[9], &[10]);

    let other = rule(&[30], &[31]);

    let model = RecursiveWorldModel::new(vec![target.clone(), other.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        other.clone(),
        40,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
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
    );

    assert!(scope.is_inactive());

    assert_eq!(scope.pressured_rule(), Some(&other,));

    assert!(scope.active_hypothesis().is_none());
}

#[test]
fn evidence_scope_preserves_threshold_support_and_provenance() {
    let target = rule(&[9], &[10]);

    let first = observation(&[1, 2], &[3]);

    let second = observation(&[1, 2, 4], &[3, 5]);

    let third = observation(&[1, 6], &[3]);

    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        generalized(
            target,
            vec![third.clone(), first.clone(), second.clone()],
            2,
        ),
    );

    assert_eq!(scope.threshold().minimum_support(), 2);

    assert_eq!(scope.support_count(), 3);

    assert!(scope.source_observations().contains(&first,));

    assert!(scope.source_observations().contains(&second,));

    assert!(scope.source_observations().contains(&third,));
}

#[test]
fn evidence_scope_preserves_exact_unit_support_counts() {
    let target = rule(&[9], &[10]);

    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
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
    );

    assert_eq!(scope.premise_support(&unit(1,),), 3);

    assert_eq!(scope.premise_support(&unit(2,),), 2);

    assert_eq!(scope.conclusion_support(&unit(3,),), 3);
}

#[test]
fn evidence_scope_preserves_frozen_m37_scope_identity() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
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
    );

    assert_eq!(scope.discovery_scope().unwrap().active_count(), 1);

    assert_eq!(scope.discovery_scope().unwrap().inactive_count(), 0);
}

#[test]
fn evidence_scoper_facade_matches_direct_scope() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        20,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let structure = generalized(
        target,
        vec![
            observation(&[1, 2], &[3]),
            observation(&[1, 2, 4], &[3, 5]),
            observation(&[1, 6], &[3]),
        ],
        2,
    );

    assert_eq!(
        RecursiveWorldRevisionGeneralizationEvidenceScoper::scope(
            &model,
            &evidence_state,
            structure.clone(),
        ),
        RecursiveWorldRevisionGeneralizationEvidenceScope::new(&model, &evidence_state, structure,)
    );
}

#[test]
fn generalization_evidence_scope_is_deterministic_and_non_mutating() {
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

    let left = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
        &model,
        &evidence_state,
        structure.clone(),
    );

    let right = RecursiveWorldRevisionGeneralizationEvidenceScope::new(
        &model,
        &evidence_state,
        generalized(target, vec![second, third, first], 2),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(evidence_state, evidence_before);

    assert_eq!(structure, structure_before);
}
