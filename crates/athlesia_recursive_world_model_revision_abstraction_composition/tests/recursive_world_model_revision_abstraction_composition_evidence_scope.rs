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
    RecursiveWorldRevisionAbstractionCompositionEvidenceScope,
    RecursiveWorldRevisionAbstractionCompositionEvidenceScopeStatus,
    RecursiveWorldRevisionAbstractionCompositionEvidenceScoper,
    RecursiveWorldRevisionAbstractionCompositionPathSelection,
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    RecursiveWorldRevisionAbstractionCompositionPathSet,
    RecursiveWorldRevisionAbstractionCompositionPathSupportSet,
    RecursiveWorldRevisionAbstractionCompositionThreshold,
    RecursiveWorldRevisionAbstractionCompositionWitness,
    RecursiveWorldRevisionAbstractionCompositionWitnessSet,
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

fn selection() -> RecursiveWorldRevisionAbstractionCompositionPathSelection {
    let mut witnesses = Vec::new();

    for index in 0..3 {
        witnesses.push(witness(
            &[1, 2],
            &[10, 20],
            if index % 2 == 0 { 1 } else { 2 },
            if index % 2 == 0 { 10 } else { 20 },
            100 + index,
        ));

        witnesses.push(witness(
            &[10, 20],
            &[100, 200],
            if index % 2 == 0 { 10 } else { 20 },
            if index % 2 == 0 { 100 } else { 200 },
            200 + index,
        ));
    }

    let composition = RecursiveWorldRevisionAbstractionComposition::compose(
        RecursiveWorldRevisionAbstractionCompositionWitnessSet::new(witnesses).unwrap(),
        RecursiveWorldRevisionAbstractionCompositionThreshold::new(2).unwrap(),
    )
    .unwrap();

    let paths = RecursiveWorldRevisionAbstractionCompositionPathSet::induce(composition).unwrap();

    let supports = RecursiveWorldRevisionAbstractionCompositionPathSupportSet::derive(paths);

    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet::select(supports)
        .selection_for(&class(&[1, 2]), &class(&[100, 200]))
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
fn discovery_unavailable_has_no_evidence_scope() {
    let target = rule(&[1], &[100]);

    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        selection(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionEvidenceScopeStatus::DiscoveryUnavailable
    );

    assert!(result.scope_result().is_none());
}

#[test]
fn rejected_validation_has_no_evidence_scope() {
    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![rule(&[8], &[88])]),
        RecursiveWorldEvidenceState::empty(),
        rule(&[9], &[99]),
        selection(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionEvidenceScopeStatus::Rejected
    );

    assert!(result.scope_result().is_none());
}

#[test]
fn accepted_hypothesis_without_evidence_is_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        selection(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionEvidenceScopeStatus::Inactive
    );

    assert!(result.is_inactive());

    assert!(result.active_hypothesis().is_none());
}

#[test]
fn confirming_target_evidence_keeps_hypothesis_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Confirming,
            500,
        )]),
        target,
        selection(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionEvidenceScopeStatus::Inactive
    );

    assert!(!result.is_active());
}

#[test]
fn violating_target_evidence_activates_hypothesis() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target.clone(),
        selection(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionEvidenceScopeStatus::Active
    );

    assert!(result.is_active());

    assert_eq!(result.pressured_rule(), Some(&target,));

    assert!(result.active_hypothesis().is_some());
}

#[test]
fn evidence_pressure_on_other_rule_does_not_activate_target() {
    let target = rule(&[9], &[99]);

    let other = rule(&[8], &[88]);

    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone(), other.clone()]),
        evidence_state(vec![evidence(
            other,
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        selection(),
        application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionEvidenceScopeStatus::Inactive
    );

    assert!(result.active_hypothesis().is_none());
}

#[test]
fn active_scope_preserves_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        selection(),
        application(),
    );

    assert_eq!(result.active_hypothesis(), result.hypothesis());
}

#[test]
fn evidence_scope_preserves_target_replacement_and_realized_observation() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target.clone(),
        selection(),
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
fn evidence_scope_preserves_exact_evidence_state() {
    let target = rule(&[9], &[99]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let before = state.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        state,
        target,
        selection(),
        application(),
    );

    assert_eq!(result.evidence_state(), &before);
}

#[test]
fn evidence_scope_preserves_path_and_application_provenance() {
    let target = rule(&[9], &[99]);

    let selected = selection();

    let selected_before = selected.clone();

    let application = application();

    let application_before = application.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        selected,
        application,
    );

    assert_eq!(result.selection(), &selected_before);

    assert_eq!(result.path(), selected_before.path());

    assert_eq!(result.minimum_support(), selected_before.minimum_support());

    assert_eq!(
        result.application_observations(),
        application_before.as_slice()
    );
}

#[test]
fn evidence_scoper_facade_matches_direct_scope() {
    let target = rule(&[9], &[99]);

    let world = RecursiveWorldModel::new(vec![target.clone()]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let selected = selection();

    let application = application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionCompositionEvidenceScoper::scope(
            world.clone(),
            state.clone(),
            target.clone(),
            selected.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
            world,
            state,
            target,
            selected,
            application,
        )
    );
}

#[test]
fn composition_evidence_scope_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let world = RecursiveWorldModel::new(vec![target.clone()]);

    let selected = selection();

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let application = vec![
        observation(&[1, 700], &[100, 800]),
        observation(&[900], &[901]),
    ];

    let world_before = world.clone();

    let selected_before = selected.clone();

    let state_before = state.clone();

    let application_before = application.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        world.clone(),
        state.clone(),
        target.clone(),
        selected.clone(),
        application.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionCompositionEvidenceScope::scope(
        world.clone(),
        state.clone(),
        target.clone(),
        selected.clone(),
        vec![application[1].clone(), application[0].clone()],
    );

    assert_eq!(left, right);

    assert_eq!(world, world_before);

    assert_eq!(selected, selected_before);

    assert_eq!(state, state_before);

    assert_eq!(application, application_before);

    assert!(left.is_active());
}
