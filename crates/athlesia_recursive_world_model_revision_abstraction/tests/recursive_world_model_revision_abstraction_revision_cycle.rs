use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldModel, RecursiveWorldRevisionBudget, RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_abstraction::{
    RecursiveWorldRevisionAbstractionClass, RecursiveWorldRevisionAbstractionConsensus,
    RecursiveWorldRevisionAbstractionCycle, RecursiveWorldRevisionAbstractionCycleStatus,
    RecursiveWorldRevisionAbstractionProjection, RecursiveWorldRevisionAbstractionRealization,
    RecursiveWorldRevisionAbstractionVocabulary,
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
fn ambiguous_abstraction_cycle_is_discovery_unavailable() {
    let target = rule(&[9], &[10]);

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        target,
        ambiguous(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCycleStatus::DiscoveryUnavailable
    );

    assert!(result.discovery_cycle().is_none());

    assert!(!result.has_revision());
}

#[test]
fn deterministic_noop_cycle_is_discovery_unavailable() {
    let target = rule(&[1], &[20]);

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        target,
        deterministic(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCycleStatus::DiscoveryUnavailable
    );

    assert!(result.discovery_cycle().is_none());
}

#[test]
fn invalid_abstraction_cycle_is_rejected() {
    let target = rule(&[9], &[10]);

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        target,
        deterministic(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCycleStatus::Rejected
    );

    assert!(result.discovery_cycle().is_none());

    assert!(!result.has_revision());
}

#[test]
fn abstraction_without_evidence_is_inactive() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        target,
        deterministic(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCycleStatus::Inactive
    );

    assert!(result.discovery_cycle().is_some());

    assert!(!result.has_revision());
}

#[test]
fn confirming_evidence_blocks_abstraction_revision() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        90,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &model,
        &evidence_state,
        target,
        deterministic(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCycleStatus::Inactive
    );

    assert!(!result.has_revision());
}

#[test]
fn negative_pressure_can_select_abstraction_revision() {
    let target = rule(&[9], &[10]);

    let replacement = rule(&[1], &[20]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        90,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &model,
        &evidence_state,
        target.clone(),
        deterministic(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCycleStatus::Revised
    );

    assert!(result.has_revision());

    assert_eq!(result.pressured_rule(), Some(&target,));

    assert_eq!(
        result.selected_revision().unwrap().replacement(),
        &replacement
    );

    assert!(result.revised_world().unwrap().contains(&replacement,));

    assert!(!result.revised_world().unwrap().contains(&target,));
}

#[test]
fn active_abstraction_over_budget_has_no_revision() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        90,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &model,
        &evidence_state,
        target,
        deterministic(),
        RecursiveWorldRevisionBudget::new(1).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCycleStatus::ActiveNoRevision
    );

    assert!(!result.has_revision());

    assert!(result.revised_world().is_none());
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

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &model,
        &evidence_state,
        target,
        deterministic(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCycleStatus::Inactive
    );

    assert_eq!(result.pressured_rule(), Some(&other,));

    assert!(!result.has_revision());
}

#[test]
fn selected_revision_preserves_abstraction_hypothesis_identity() {
    let target = rule(&[9], &[10]);

    let replacement = rule(&[1], &[20]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        90,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &model,
        &evidence_state,
        target.clone(),
        deterministic(),
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
fn abstraction_cycle_preserves_realization_and_provenance() {
    let target = rule(&[9], &[10]);

    let premise = class(&[1, 2]);

    let conclusion = class(&[20, 21]);

    let realized = deterministic();

    let before = realized.clone();

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &RecursiveWorldModel::new(vec![target.clone()]),
        &RecursiveWorldEvidenceState::empty(),
        target.clone(),
        realized,
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.target(), &target);

    assert_eq!(result.realization(), &before);

    assert_eq!(result.observation_count(), 2);

    assert!(result
        .source_observations()
        .contains(&observation(&[1, 50], &[20, 60],),));

    assert_eq!(result.vocabulary().class_for(&unit(1,),), Some(&premise,));

    assert_eq!(
        result.premise_witnesses(&premise,),
        std::slice::from_ref(&unit(1,),)
    );

    assert_eq!(
        result.conclusion_witnesses(&conclusion,),
        std::slice::from_ref(&unit(20,),)
    );
}

#[test]
fn abstraction_cycle_preserves_frozen_m37_cycle_identity() {
    let target = rule(&[9], &[10]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        90,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &model,
        &evidence_state,
        target,
        deterministic(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(result.discovery_cycle().unwrap().has_revision());

    assert_eq!(
        result
            .discovery_cycle()
            .unwrap()
            .selected_hypotheses()
            .len(),
        1
    );
}

#[test]
fn abstraction_cycle_is_deterministic_and_non_mutating() {
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

    let left = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &model,
        &evidence_state,
        target.clone(),
        realized.clone(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    let right = RecursiveWorldRevisionAbstractionCycle::evaluate(
        &model,
        &evidence_state,
        target,
        deterministic(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(evidence_state, evidence_before);

    assert_eq!(realized, realized_before);
}
