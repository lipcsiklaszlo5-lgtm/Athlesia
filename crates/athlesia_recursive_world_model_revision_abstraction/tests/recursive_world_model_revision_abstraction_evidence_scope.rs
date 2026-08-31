use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_abstraction::{
    RecursiveWorldRevisionAbstractionClass, RecursiveWorldRevisionAbstractionConsensus,
    RecursiveWorldRevisionAbstractionEvidenceScope,
    RecursiveWorldRevisionAbstractionEvidenceScoper,
    RecursiveWorldRevisionAbstractionEvidenceStatus, RecursiveWorldRevisionAbstractionProjection,
    RecursiveWorldRevisionAbstractionRealization, RecursiveWorldRevisionAbstractionVocabulary,
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

fn class(members: &[usize]) -> RecursiveWorldRevisionAbstractionClass {
    RecursiveWorldRevisionAbstractionClass::new(members.iter().copied().map(unit).collect())
        .unwrap()
}

fn realization(
    classes: Vec<RecursiveWorldRevisionAbstractionClass>,
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
) -> RecursiveWorldRevisionAbstractionRealization {
    let vocabulary = RecursiveWorldRevisionAbstractionVocabulary::new(classes).unwrap();

    let observations = RecursiveWorldRevisionInductionObservationSet::new(observations).unwrap();

    let projection =
        RecursiveWorldRevisionAbstractionProjection::project(vocabulary, observations).unwrap();

    let consensus = RecursiveWorldRevisionAbstractionConsensus::derive(projection).unwrap();

    RecursiveWorldRevisionAbstractionRealization::realize(consensus)
}

fn deterministic() -> RecursiveWorldRevisionAbstractionRealization {
    realization(
        vec![class(&[1, 2]), class(&[20, 21])],
        vec![
            observation(&[1, 50], &[20, 60]),
            observation(&[1, 51], &[20, 61]),
        ],
    )
}

fn ambiguous() -> RecursiveWorldRevisionAbstractionRealization {
    realization(
        vec![class(&[1, 2]), class(&[20, 21])],
        vec![observation(&[1], &[20]), observation(&[2], &[21])],
    )
}

fn evidence(
    target: RecursiveWorldRule,
    observation: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(target, unit(observation), kind)
}

#[test]
fn ambiguous_abstraction_scope_is_discovery_unavailable() {
    let target = rule(&[9], &[10]);

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        target,
        ambiguous(),
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionAbstractionEvidenceStatus::DiscoveryUnavailable
    );

    assert!(scope.is_discovery_unavailable());

    assert!(scope.discovery_scope().is_none());
}

#[test]
fn deterministic_noop_scope_is_discovery_unavailable() {
    let target = rule(&[1], &[20]);

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        target,
        deterministic(),
    );

    assert!(scope.is_discovery_unavailable());

    assert!(scope.active_hypothesis().is_none());
}

#[test]
fn invalid_abstraction_scope_is_rejected() {
    let target = rule(&[9], &[10]);

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        target,
        deterministic(),
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionAbstractionEvidenceStatus::Rejected
    );

    assert!(scope.is_rejected());

    assert!(scope.discovery_scope().is_none());
}

#[test]
fn accepted_abstraction_without_evidence_is_inactive() {
    let target = rule(&[9], &[10]);

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        target,
        deterministic(),
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionAbstractionEvidenceStatus::Inactive
    );

    assert!(scope.is_inactive());

    assert!(scope.inactive_hypothesis().is_some());
}

#[test]
fn confirming_evidence_keeps_abstraction_inactive() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        90,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &model,
        &evidence_state,
        target,
        deterministic(),
    );

    assert!(scope.is_inactive());

    assert!(scope.active_hypothesis().is_none());
}

#[test]
fn balanced_evidence_keeps_abstraction_inactive() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 90, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 91, RecursiveWorldEvidenceKind::Violating),
    ]);

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &model,
        &evidence_state,
        target,
        deterministic(),
    );

    assert!(scope.is_inactive());

    assert!(scope.pressured_rule().is_none());
}

#[test]
fn negative_pressure_activates_matching_abstraction() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        90,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &model,
        &evidence_state,
        target.clone(),
        deterministic(),
    );

    assert_eq!(
        scope.status(),
        RecursiveWorldRevisionAbstractionEvidenceStatus::Active
    );

    assert!(scope.is_active());

    assert_eq!(scope.pressured_rule(), Some(&target,));

    assert!(scope.active_hypothesis().is_some());
}

#[test]
fn pressure_on_other_rule_keeps_abstraction_inactive() {
    let target = rule(&[9], &[10]);

    let other = rule(&[30], &[31]);

    let model = RecursiveWorldModel::new(vec![target.clone(), other.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        other.clone(),
        90,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &model,
        &evidence_state,
        target,
        deterministic(),
    );

    assert!(scope.is_inactive());

    assert_eq!(scope.pressured_rule(), Some(&other,));

    assert!(scope.active_hypothesis().is_none());
}

#[test]
fn evidence_scope_preserves_target_and_realization_identity() {
    let target = rule(&[9], &[10]);

    let realized = deterministic();

    let before = realized.clone();

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        target.clone(),
        realized,
    );

    assert_eq!(scope.target(), &target);

    assert_eq!(scope.realization(), &before);
}

#[test]
fn evidence_scope_preserves_source_vocabulary_and_witness_provenance() {
    let target = rule(&[9], &[10]);

    let premise = class(&[1, 2]);

    let conclusion = class(&[20, 21]);

    let scope = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        target,
        deterministic(),
    );

    assert_eq!(scope.observation_count(), 2);

    assert!(scope
        .source_observations()
        .contains(&observation(&[1, 50], &[20, 60],),));

    assert_eq!(scope.vocabulary().class_for(&unit(1,),), Some(&premise,));

    assert_eq!(
        scope.premise_witnesses(&premise,),
        std::slice::from_ref(&unit(1,),)
    );

    assert_eq!(
        scope.conclusion_witnesses(&conclusion,),
        std::slice::from_ref(&unit(20,),)
    );
}

#[test]
fn abstraction_evidence_scoper_facade_matches_direct_scope() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        90,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let realized = deterministic();

    assert_eq!(
        RecursiveWorldRevisionAbstractionEvidenceScoper::scope(
            &model,
            &evidence_state,
            target.clone(),
            realized.clone(),
        ),
        RecursiveWorldRevisionAbstractionEvidenceScope::new(
            &model,
            &evidence_state,
            target,
            realized,
        )
    );
}

#[test]
fn abstraction_evidence_scope_is_deterministic_and_non_mutating() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        90,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let realized = deterministic();

    let model_before = model.clone();

    let evidence_before = evidence_state.clone();

    let realized_before = realized.clone();

    let left = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &model,
        &evidence_state,
        target.clone(),
        realized.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionEvidenceScope::new(
        &model,
        &evidence_state,
        target,
        deterministic(),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(evidence_state, evidence_before);

    assert_eq!(realized, realized_before);
}
