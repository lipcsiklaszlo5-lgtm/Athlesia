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
    RecursiveWorldRevisionAbstractionCompositionPathSelection,
    RecursiveWorldRevisionAbstractionCompositionPathSelectionSet,
    RecursiveWorldRevisionAbstractionCompositionPathSet,
    RecursiveWorldRevisionAbstractionCompositionPathSupportSet,
    RecursiveWorldRevisionAbstractionCompositionRevisionCycle,
    RecursiveWorldRevisionAbstractionCompositionRevisionCycleStatus,
    RecursiveWorldRevisionAbstractionCompositionRevisionCycler,
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

fn high_budget() -> RecursiveWorldRevisionBudget {
    RecursiveWorldRevisionBudget::new(100).unwrap()
}

fn low_budget() -> RecursiveWorldRevisionBudget {
    RecursiveWorldRevisionBudget::new(1).unwrap()
}

#[test]
fn discovery_unavailable_never_runs_revision_cycle() {
    let target = rule(&[1], &[100]);

    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        selection(),
        application(),
        high_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionRevisionCycleStatus::DiscoveryUnavailable
    );

    assert!(result.cycle_result().is_none());

    assert!(!result.has_revision());
}

#[test]
fn rejected_hypothesis_never_runs_revision_cycle() {
    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![rule(&[8], &[88])]),
        RecursiveWorldEvidenceState::empty(),
        rule(&[9], &[99]),
        selection(),
        application(),
        high_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionRevisionCycleStatus::Rejected
    );

    assert!(result.cycle_result().is_none());
}

#[test]
fn inactive_hypothesis_does_not_run_revision_cycle() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        RecursiveWorldEvidenceState::empty(),
        target,
        selection(),
        application(),
        high_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionRevisionCycleStatus::Inactive
    );

    assert!(result.cycle_result().is_none());

    assert!(!result.has_revision());
}

#[test]
fn active_affordable_composition_revision_revises_world() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        selection(),
        application(),
        high_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionRevisionCycleStatus::Revised
    );

    assert!(result.is_revised());

    assert!(result.has_revision());

    assert!(result.revised_world().is_some());
}

#[test]
fn active_over_budget_composition_revision_is_not_applied() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        selection(),
        application(),
        low_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionCompositionRevisionCycleStatus::ActiveNoRevision
    );

    assert!(result.cycle_result().is_some());

    assert!(!result.has_revision());

    assert!(result.revised_world().is_none());
}

#[test]
fn successful_revision_replaces_exact_target() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target.clone(),
        selection(),
        application(),
        high_budget(),
    );

    let revised = result.revised_world().unwrap();

    assert!(!revised.rules().contains(&target,));

    assert!(revised.rules().contains(&rule(&[1], &[100],),));
}

#[test]
fn successful_revision_preserves_world_rule_count() {
    let target = rule(&[9], &[99]);

    let other = rule(&[8], &[88]);

    let world = RecursiveWorldModel::new(vec![target.clone(), other.clone()]);

    let original_count = world.rules().len();

    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        world,
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        selection(),
        application(),
        high_budget(),
    );

    let revised = result.revised_world().unwrap();

    assert_eq!(revised.rules().len(), original_count);

    assert!(revised.rules().contains(&other,));
}

#[test]
fn revision_cycle_preserves_target_hypothesis_and_replacement_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target.clone(),
        selection(),
        application(),
        high_budget(),
    );

    assert_eq!(result.target(), &target);

    assert_eq!(result.hypothesis().unwrap().target(), &target);

    assert_eq!(result.replacement(), Some(&rule(&[1], &[100],),));
}

#[test]
fn revision_cycle_preserves_realization_and_path_provenance() {
    let target = rule(&[9], &[99]);

    let selected = selection();

    let selected_before = selected.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        selected,
        application(),
        high_budget(),
    );

    assert_eq!(result.selection(), &selected_before);

    assert_eq!(result.path(), selected_before.path());

    assert_eq!(result.minimum_support(), selected_before.minimum_support());

    assert_eq!(
        result.realized_observation(),
        Some(&observation(&[1], &[100],),)
    );
}

#[test]
fn revision_cycle_preserves_application_and_evidence_provenance() {
    let target = rule(&[9], &[99]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let application = application();

    let state_before = state.clone();

    let application_before = application.clone();

    let result = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        RecursiveWorldModel::new(vec![target.clone()]),
        state,
        target,
        selection(),
        application,
        high_budget(),
    );

    assert_eq!(result.evidence_state(), &state_before);

    assert_eq!(
        result.application_observations(),
        application_before.as_slice()
    );
}

#[test]
fn revision_cycler_facade_matches_direct_cycle() {
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
        RecursiveWorldRevisionAbstractionCompositionRevisionCycler::evaluate(
            world.clone(),
            state.clone(),
            target.clone(),
            selected.clone(),
            application.clone(),
            high_budget(),
        ),
        RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
            world,
            state,
            target,
            selected,
            application,
            high_budget(),
        )
    );
}

#[test]
fn composition_revision_cycle_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let world = RecursiveWorldModel::new(vec![target.clone()]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let selected = selection();

    let first = observation(&[1, 700], &[100, 800]);

    let second = observation(&[900], &[901]);

    let world_before = world.clone();

    let state_before = state.clone();

    let selected_before = selected.clone();

    let left = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        world.clone(),
        state.clone(),
        target.clone(),
        selected.clone(),
        vec![first.clone(), second.clone()],
        high_budget(),
    );

    let right = RecursiveWorldRevisionAbstractionCompositionRevisionCycle::evaluate(
        world.clone(),
        state.clone(),
        target.clone(),
        selected.clone(),
        vec![second, first],
        high_budget(),
    );

    assert_eq!(left, right);

    assert_eq!(world, world_before);

    assert_eq!(state, state_before);

    assert_eq!(selected, selected_before);

    assert!(left.is_revised());
}
