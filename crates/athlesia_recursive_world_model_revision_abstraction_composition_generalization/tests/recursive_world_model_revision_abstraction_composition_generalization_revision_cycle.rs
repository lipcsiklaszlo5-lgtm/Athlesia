use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldModel, RecursiveWorldRevisionBudget, RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_abstraction::RecursiveWorldRevisionAbstractionClass;

use athlesia_recursive_world_model_revision_abstraction_composition::{
    RecursiveWorldRevisionAbstractionComposition,
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    RecursiveWorldRevisionAbstractionCompositionPathSet,
    RecursiveWorldRevisionAbstractionCompositionPathSupportSet,
    RecursiveWorldRevisionAbstractionCompositionThreshold,
    RecursiveWorldRevisionAbstractionCompositionWitness,
    RecursiveWorldRevisionAbstractionCompositionWitnessSet,
};

use athlesia_recursive_world_model_revision_abstraction_composition_generalization::{
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycleStatus,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycler,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationSource,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold,
    RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet,
};

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(
        StructuralConcept::with_sequence_length(
            vec![PrimitiveSignature::new(RelationKind::Equal, span)],
            8,
        ),
    ))
}

fn class(members: &[usize]) -> RecursiveWorldRevisionAbstractionClass {
    RecursiveWorldRevisionAbstractionClass::new(members.iter().copied().map(unit).collect())
        .unwrap()
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

fn witness(
    from_members: &[usize],
    to_members: &[usize],
    premise_member: usize,
    conclusion_member: usize,
    noise: usize,
) -> RecursiveWorldRevisionAbstractionCompositionWitness {
    RecursiveWorldRevisionAbstractionCompositionWitness::new(
        class(from_members),
        class(to_members),
        observation(
            &[premise_member, 1000 + noise],
            &[conclusion_member, 2000 + noise],
        ),
    )
    .unwrap()
}

fn context(seed: usize) -> RecursiveWorldRevisionAbstractionCompositionPathSelectionSet {
    let relations = vec![(vec![1, 2], vec![10, 20]), (vec![10, 20], vec![100, 200])];

    let mut witnesses = Vec::new();

    for (edge_index, (from, to)) in relations.into_iter().enumerate() {
        for support_index in 0..2 {
            witnesses.push(witness(
                &from,
                &to,
                from[support_index % from.len()],
                to[support_index % to.len()],
                seed * 10000 + edge_index * 100 + support_index,
            ));
        }
    }

    let composition = RecursiveWorldRevisionAbstractionComposition::compose(
        RecursiveWorldRevisionAbstractionCompositionWitnessSet::new(witnesses).unwrap(),
        RecursiveWorldRevisionAbstractionCompositionThreshold::new(2).unwrap(),
    )
    .unwrap();

    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(composition).unwrap();

    let supports = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths);

    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(supports)
}

fn projected() -> RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif {
    let source = RecursiveWorldRevisionAbstractionCompositionGeneralizationSource::new(vec![
        context(1),
        context(2),
    ])
    .unwrap();

    let generalized = RecursiveWorldRevisionAbstractionCompositionGeneralizedMotifSet::generalize(
        source,
        RecursiveWorldRevisionAbstractionCompositionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap();

    let resolution =
        RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution::resolve(generalized);

    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection::project(
        resolution,
        context(10),
    )
    .projected_motifs()
    .first()
    .unwrap()
    .clone()
}

fn application() -> Vec<RecursiveWorldRevisionDiscoveryObservation> {
    vec![observation(&[1, 700], &[100, 800])]
}

fn evidence(
    target: RecursiveWorldRule,
    kind: RecursiveWorldEvidenceKind,
    id: usize,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(target, unit(id), kind)
}

fn evidence_state(records: Vec<RecursiveWorldEvidenceRecord>) -> RecursiveWorldEvidenceState {
    let mut state = RecursiveWorldEvidenceState::empty();

    for record in records {
        state = state.accumulate(record);
    }

    state
}

fn large_budget() -> RecursiveWorldRevisionBudget {
    RecursiveWorldRevisionBudget::new(100).unwrap()
}

fn tiny_budget() -> RecursiveWorldRevisionBudget {
    RecursiveWorldRevisionBudget::new(1).unwrap()
}

#[test]
fn discovery_unavailable_cycle_propagates_without_discovery_execution() {
    let target = rule(&[1], &[100]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        projected(),
        application(),
        large_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycleStatus::
            DiscoveryUnavailable
    );

    assert!(result.discovery_cycle().is_none());
}

#[test]
fn rejected_cycle_propagates_without_discovery_execution() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![rule(&[8], &[88])]),
        RecursiveWorldEvidenceState::empty(),
        target,
        projected(),
        application(),
        large_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycleStatus::Rejected
    );

    assert!(result.discovery_cycle().is_none());
}

#[test]
fn empty_evidence_cycle_is_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        projected(),
        application(),
        large_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycleStatus::Inactive
    );

    assert!(result.discovery_cycle().is_none());
}

#[test]
fn confirming_only_cycle_is_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Confirming,
            500,
        )]),
        target,
        projected(),
        application(),
        large_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycleStatus::Inactive
    );
}

#[test]
fn violating_other_target_cycle_is_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            rule(&[8], &[88]),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        projected(),
        application(),
        large_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycleStatus::Inactive
    );
}

#[test]
fn exact_violation_with_sufficient_budget_revises() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        projected(),
        application(),
        large_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycleStatus::Revised
    );

    assert!(result.has_revision());

    assert!(result.discovery_cycle().unwrap().has_revision());
}

#[test]
fn exact_violation_with_tiny_budget_is_active_without_revision() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        projected(),
        application(),
        tiny_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycleStatus::
            ActiveNoRevision
    );

    assert!(!result.has_revision());

    assert!(result.discovery_cycle().is_some());
}

#[test]
fn revised_cycle_preserves_expected_replacement_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target.clone(),
        projected(),
        application(),
        large_budget(),
    );

    assert_eq!(result.target(), &target);

    assert_eq!(result.replacement(), Some(&rule(&[1], &[100],),));

    assert!(result.discovery_cycle().unwrap().has_revision());
}

#[test]
fn revision_cycle_preserves_original_world_model() {
    let target = rule(&[9], &[99]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let before = model.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        model.clone(),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        projected(),
        application(),
        large_budget(),
    );

    assert_eq!(model, before);

    assert_eq!(result.model(), &before);
}

#[test]
fn revision_cycle_preserves_evidence_budget_projection_and_application_provenance() {
    let target = rule(&[9], &[99]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let state_before = state.clone();

    let projection = projected();

    let projection_before = projection.clone();

    let observations = application();

    let observations_before = observations.clone();

    let budget = large_budget();

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        state,
        target,
        projection,
        observations,
        budget,
    );

    assert_eq!(result.evidence_state(), &state_before);

    assert_eq!(result.budget(), budget);

    assert_eq!(result.projected_motif(), &projection_before);

    assert_eq!(
        result.application_observations(),
        observations_before.as_slice()
    );

    assert!(result.active_hypothesis().is_some());
}

#[test]
fn revision_cycler_facade_matches_direct_cycle() {
    let target = rule(&[9], &[99]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let projection = projected();

    let observations = application();

    let budget = large_budget();

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycler::evaluate(
            model.clone(),
            state.clone(),
            target.clone(),
            projection.clone(),
            observations.clone(),
            budget,
        ),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
            model,
            state,
            target,
            projection,
            observations,
            budget,
        )
    );
}

#[test]
fn generalized_revision_cycle_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = evidence_state(vec![
        evidence(target.clone(), RecursiveWorldEvidenceKind::Violating, 500),
        evidence(target.clone(), RecursiveWorldEvidenceKind::Confirming, 501),
    ]);

    let projection = projected();

    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let model_before = model.clone();

    let state_before = state.clone();

    let projection_before = projection.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        model.clone(),
        state.clone(),
        target.clone(),
        projection.clone(),
        vec![first.clone(), second.clone()],
        large_budget(),
    );

    let right = RecursiveWorldRevisionAbstractionCompositionGeneralizationRevisionCycle::evaluate(
        model.clone(),
        state.clone(),
        target,
        projection.clone(),
        vec![second, first],
        large_budget(),
    );

    assert_eq!(left, right);

    assert_eq!(left.status(), right.status());

    assert_eq!(left.has_revision(), right.has_revision());

    assert_eq!(
        left.discovery_cycle().is_some(),
        right.discovery_cycle().is_some()
    );

    assert!(left.active_hypothesis().is_some());

    assert_eq!(model, model_before);

    assert_eq!(state, state_before);

    assert_eq!(projection, projection_before);
}
