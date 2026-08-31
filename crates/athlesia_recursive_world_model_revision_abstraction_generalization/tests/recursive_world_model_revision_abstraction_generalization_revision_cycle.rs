use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldModel, RecursiveWorldRevisionBudget, RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_abstraction_generalization::{
    RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle,
    RecursiveWorldRevisionAbstractionGeneralizationRevisionCycleStatus,
    RecursiveWorldRevisionAbstractionGeneralizationRevisionCycler,
    RecursiveWorldRevisionAbstractionGeneralizationThreshold,
    RecursiveWorldRevisionAbstractionGeneralizedClassSet,
};

use athlesia_recursive_world_model_revision_abstraction_induction::{
    RecursiveWorldRevisionAbstractionInducedClassSet,
    RecursiveWorldRevisionAbstractionSubstitutionWitness,
    RecursiveWorldRevisionAbstractionSubstitutionWitnessSet,
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

fn premise_witness(
    first: usize,
    second: usize,
    shared: usize,
    fixed_conclusion: usize,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[first, shared], &[fixed_conclusion]),
        observation(&[second, shared], &[fixed_conclusion]),
    )
    .unwrap()
}

fn conclusion_witness(
    first: usize,
    second: usize,
    shared: usize,
    fixed_premise: usize,
) -> RecursiveWorldRevisionAbstractionSubstitutionWitness {
    RecursiveWorldRevisionAbstractionSubstitutionWitness::discover(
        observation(&[fixed_premise], &[first, shared]),
        observation(&[fixed_premise], &[second, shared]),
    )
    .unwrap()
}

fn induced(
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
) -> RecursiveWorldRevisionAbstractionInducedClassSet {
    RecursiveWorldRevisionAbstractionInducedClassSet::induce(
        RecursiveWorldRevisionAbstractionSubstitutionWitnessSet::new(witnesses).unwrap(),
    )
    .unwrap()
}

fn generalized(
    witnesses: Vec<RecursiveWorldRevisionAbstractionSubstitutionWitness>,
) -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    RecursiveWorldRevisionAbstractionGeneralizedClassSet::generalize(
        induced(witnesses),
        RecursiveWorldRevisionAbstractionGeneralizationThreshold::new(2).unwrap(),
    )
    .unwrap()
}

fn generalized_source() -> RecursiveWorldRevisionAbstractionGeneralizedClassSet {
    generalized(vec![
        premise_witness(1, 2, 30, 40),
        premise_witness(1, 2, 31, 41),
        conclusion_witness(10, 20, 50, 60),
        conclusion_witness(10, 20, 51, 61),
    ])
}

fn deterministic_application() -> RecursiveWorldRevisionInductionObservationSet {
    observation_set(vec![
        observation(&[1, 70], &[10, 80]),
        observation(&[1, 71], &[10, 81]),
    ])
}

fn evidence(
    target: RecursiveWorldRule,
    kind: RecursiveWorldEvidenceKind,
    observation_id: usize,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(target, unit(observation_id), kind)
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
    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        model(vec![rule(&[1], &[10])]),
        RecursiveWorldEvidenceState::empty(),
        rule(&[1], &[10]),
        generalized_source(),
        deterministic_application(),
        high_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationRevisionCycleStatus::DiscoveryUnavailable
    );

    assert!(result.cycle_result().is_none());

    assert!(!result.has_revision());
}

#[test]
fn rejected_generalized_hypothesis_never_runs_revision_cycle() {
    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        model(vec![rule(&[8], &[88])]),
        RecursiveWorldEvidenceState::empty(),
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
        high_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationRevisionCycleStatus::Rejected
    );

    assert!(result.cycle_result().is_none());
}

#[test]
fn accepted_but_unpressured_hypothesis_is_inactive() {
    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        model(vec![rule(&[9], &[99])]),
        RecursiveWorldEvidenceState::empty(),
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
        high_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationRevisionCycleStatus::Inactive
    );

    assert!(result.cycle_result().is_some());

    assert!(!result.has_revision());
}

#[test]
fn active_affordable_generalized_revision_revises_world() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        generalized_source(),
        deterministic_application(),
        high_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationRevisionCycleStatus::Revised
    );

    assert!(result.is_revised());

    assert!(result.has_revision());

    assert!(result.revised_world().is_some());
}

#[test]
fn active_over_budget_generalized_revision_is_not_applied() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        generalized_source(),
        deterministic_application(),
        low_budget(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationRevisionCycleStatus::ActiveNoRevision
    );

    assert!(!result.has_revision());

    assert!(result.revised_world().is_none());
}

#[test]
fn successful_revision_replaces_exact_target() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target.clone(),
        generalized_source(),
        deterministic_application(),
        high_budget(),
    );

    let revised = result.revised_world().unwrap();

    assert!(!revised.rules().contains(&target,));

    assert!(revised.rules().contains(&rule(&[1], &[10],),));
}

#[test]
fn successful_revision_preserves_world_rule_count() {
    let target = rule(&[9], &[99]);

    let other = rule(&[8], &[88]);

    let original = model(vec![target.clone(), other.clone()]);

    let original_count = original.rules().len();

    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        original,
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        generalized_source(),
        deterministic_application(),
        high_budget(),
    );

    let revised = result.revised_world().unwrap();

    assert_eq!(revised.rules().len(), original_count);

    assert!(revised.rules().contains(&other,));
}

#[test]
fn revision_cycle_preserves_target_and_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target.clone(),
        generalized_source(),
        deterministic_application(),
        high_budget(),
    );

    assert_eq!(result.target(), &target);

    assert_eq!(
        result.cycle_result().unwrap().selected_hypotheses(),
        &[result.hypothesis().unwrap().clone(),]
    );
}

#[test]
fn revision_cycle_preserves_replacement_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        generalized_source(),
        deterministic_application(),
        high_budget(),
    );

    assert_eq!(result.replacement(), Some(&rule(&[1], &[10],),));

    assert!(result.has_revision());
}

#[test]
fn revision_cycle_preserves_generalization_application_and_evidence_provenance() {
    let target = rule(&[9], &[99]);

    let source = generalized_source();

    let application = deterministic_application();

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let source_before = source.clone();

    let application_before = application.clone();

    let state_before = state.clone();

    let result = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        model(vec![target.clone()]),
        state,
        target,
        source,
        application,
        high_budget(),
    );

    assert_eq!(result.generalized_source(), &source_before);

    assert_eq!(result.application_observations(), &application_before);

    assert_eq!(result.evidence_state(), &state_before);
}

#[test]
fn revision_cycler_facade_matches_direct_cycle() {
    let target = rule(&[9], &[99]);

    let world = model(vec![target.clone()]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let source = generalized_source();

    let application = deterministic_application();

    assert_eq!(
        RecursiveWorldRevisionAbstractionGeneralizationRevisionCycler::evaluate(
            world.clone(),
            state.clone(),
            target.clone(),
            source.clone(),
            application.clone(),
            high_budget(),
        ),
        RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
            world,
            state,
            target,
            source,
            application,
            high_budget(),
        )
    );
}

#[test]
fn generalized_revision_cycle_is_canonical_deterministic_and_non_mutating() {
    let target = rule(&[9], &[99]);

    let world = model(vec![target.clone()]);

    let state = evidence_state(vec![evidence(
        target.clone(),
        RecursiveWorldEvidenceKind::Violating,
        500,
    )]);

    let source = generalized_source();

    let application = deterministic_application();

    let world_before = world.clone();

    let state_before = state.clone();

    let source_before = source.clone();

    let application_before = application.clone();

    let left = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        world.clone(),
        state.clone(),
        target.clone(),
        source.clone(),
        application.clone(),
        high_budget(),
    );

    let right = RecursiveWorldRevisionAbstractionGeneralizationRevisionCycle::evaluate(
        world.clone(),
        state.clone(),
        target.clone(),
        generalized(vec![
            conclusion_witness(10, 20, 51, 61),
            premise_witness(1, 2, 31, 41),
            conclusion_witness(10, 20, 50, 60),
            premise_witness(1, 2, 30, 40),
        ]),
        observation_set(vec![
            observation(&[1, 71], &[10, 81]),
            observation(&[1, 70], &[10, 80]),
        ]),
        high_budget(),
    );

    assert_eq!(left, right);

    assert_eq!(world, world_before);

    assert_eq!(state, state_before);

    assert_eq!(source, source_before);

    assert_eq!(application, application_before);

    assert!(left.is_revised());
}
