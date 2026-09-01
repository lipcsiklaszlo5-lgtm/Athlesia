use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

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
    RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScopeStatus,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScoper,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjectedMotif,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationProjection,
    RecursiveWorldRevisionAbstractionCompositionGeneralizationResolution,
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

#[test]
fn discovery_unavailable_scope_propagates() {
    let target = rule(&[1], &[100]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScopeStatus::
            DiscoveryUnavailable
    );

    assert!(result.active_hypothesis().is_none());
}

#[test]
fn rejected_validation_scope_propagates() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![rule(&[8], &[88])]),
        RecursiveWorldEvidenceState::empty(),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScopeStatus::Rejected
    );
}

#[test]
fn accepted_without_evidence_is_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScopeStatus::Inactive
    );

    assert!(result.active_hypothesis().is_none());
}

#[test]
fn confirming_exact_target_evidence_is_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Confirming,
            500,
        )]),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScopeStatus::Inactive
    );
}

#[test]
fn violating_exact_target_evidence_activates_scope() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScopeStatus::Active
    );

    assert!(result.is_active());

    assert!(result.active_hypothesis().is_some());
}

#[test]
fn violating_other_target_evidence_is_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            rule(&[8], &[88]),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        projected(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScopeStatus::Inactive
    );

    assert!(result.active_hypothesis().is_none());
}

#[test]
fn active_hypothesis_identity_matches_validated_hypothesis() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        projected(),
        application(),
    );

    assert_eq!(result.active_hypothesis(), result.hypothesis());
}

#[test]
fn evidence_scope_preserves_target_replacement_and_observation_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target.clone(),
        projected(),
        application(),
    );

    assert_eq!(result.target(), &target);

    assert_eq!(result.replacement(), Some(&rule(&[1], &[100],),));

    assert_eq!(
        result.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );
}

#[test]
fn evidence_scope_preserves_evidence_state_provenance() {
    let target = rule(&[9], &[99]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let before = state.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        state,
        target,
        projected(),
        application(),
    );

    assert_eq!(result.evidence_state(), &before);
}

#[test]
fn evidence_scope_preserves_projection_and_application_provenance() {
    let target = rule(&[9], &[99]);

    let projected = projected();

    let projected_before = projected.clone();

    let application = application();

    let application_before = application.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        projected,
        application,
    );

    assert_eq!(result.projected_motif(), &projected_before);

    assert_eq!(
        result.application_observations(),
        application_before.as_slice()
    );

    assert_eq!(
        result.support_count(),
        projected_before.motif().support_count()
    );

    assert_eq!(
        result.matching_selections(),
        projected_before.matching_selections()
    );
}

#[test]
fn evidence_scoper_facade_matches_direct_scope() {
    let target = rule(&[9], &[99]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let projected = projected();

    let application = application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScoper::scope(
            model.clone(),
            state.clone(),
            target.clone(),
            projected.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
            model,
            state,
            target,
            projected,
            application,
        )
    );
}

#[test]
fn generalized_composition_evidence_scope_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = evidence_state(vec![
        evidence(target.clone(), RecursiveWorldEvidenceKind::Violating, 500),
        evidence(target.clone(), RecursiveWorldEvidenceKind::Confirming, 501),
    ]);

    let projected = projected();

    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let model_before = model.clone();

    let state_before = state.clone();

    let projected_before = projected.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        model.clone(),
        state.clone(),
        target.clone(),
        projected.clone(),
        vec![first.clone(), second.clone()],
    );

    let right = RecursiveWorldRevisionAbstractionCompositionGeneralizationEvidenceScope::scope(
        model.clone(),
        state.clone(),
        target,
        projected.clone(),
        vec![second, first],
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(state, state_before);

    assert_eq!(projected, projected_before);

    assert!(left.is_active());
}
