use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_abstraction_generalization::{
    RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope,
    RecursiveWorldRevisionAbstractionGeneralizationEvidenceScopeStatus,
    RecursiveWorldRevisionAbstractionGeneralizationEvidenceScoper,
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

#[test]
fn discovery_unavailable_has_no_evidence_scope() {
    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![rule(&[1], &[10])]),
        RecursiveWorldEvidenceState::empty(),
        rule(&[1], &[10]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationEvidenceScopeStatus::DiscoveryUnavailable
    );

    assert!(result.scope_result().is_none());
}

#[test]
fn rejected_validation_has_no_evidence_scope() {
    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![rule(&[8], &[88])]),
        RecursiveWorldEvidenceState::empty(),
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationEvidenceScopeStatus::Rejected
    );

    assert!(result.scope_result().is_none());
}

#[test]
fn accepted_hypothesis_without_evidence_is_inactive() {
    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![rule(&[9], &[99])]),
        RecursiveWorldEvidenceState::empty(),
        rule(&[9], &[99]),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationEvidenceScopeStatus::Inactive
    );

    assert!(result.is_inactive());

    assert!(result.scope_result().is_some());
}

#[test]
fn confirming_target_evidence_keeps_hypothesis_inactive() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Confirming,
            500,
        )]),
        target,
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationEvidenceScopeStatus::Inactive
    );

    assert!(result.active_hypothesis().is_none());
}

#[test]
fn violating_target_evidence_activates_hypothesis() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target.clone(),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationEvidenceScopeStatus::Active
    );

    assert!(result.is_active());

    assert_eq!(result.pressured_rule(), Some(&target,));
}

#[test]
fn evidence_pressure_on_other_rule_does_not_activate_target() {
    let target = rule(&[9], &[99]);

    let other = rule(&[8], &[88]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![target.clone(), other.clone()]),
        evidence_state(vec![evidence(
            other.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(
        result.status(),
        RecursiveWorldRevisionAbstractionGeneralizationEvidenceScopeStatus::Inactive
    );

    assert!(result.active_hypothesis().is_none());
}

#[test]
fn active_scope_preserves_hypothesis_identity() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(result.active_hypothesis(), result.hypothesis());
}

#[test]
fn evidence_scope_preserves_target_replacement_and_observation() {
    let target = rule(&[9], &[99]);

    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target.clone(),
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(result.target(), &target);

    assert_eq!(result.replacement(), Some(&rule(&[1], &[10],),));

    assert_eq!(
        result.realized_observation(),
        Some(&observation(&[1], &[10],),)
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

    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![target.clone()]),
        state,
        target,
        generalized_source(),
        deterministic_application(),
    );

    assert_eq!(result.evidence_state(), &before);
}

#[test]
fn evidence_scope_preserves_generalization_application_and_abstraction_provenance() {
    let target = rule(&[9], &[99]);

    let source = generalized_source();

    let application = deterministic_application();

    let source_before = source.clone();

    let application_before = application.clone();

    let result = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        model(vec![target.clone()]),
        evidence_state(vec![evidence(
            target.clone(),
            RecursiveWorldEvidenceKind::Violating,
            500,
        )]),
        target,
        source,
        application,
    );

    assert_eq!(result.generalized_source(), &source_before);

    assert_eq!(result.application_observations(), &application_before);

    assert!(result.consensus().is_some());

    assert!(result.vocabulary().is_some());
}

#[test]
fn evidence_scoper_facade_matches_direct_scope() {
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
        RecursiveWorldRevisionAbstractionGeneralizationEvidenceScoper::scope(
            world.clone(),
            state.clone(),
            target.clone(),
            source.clone(),
            application.clone(),
        ),
        RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
            world,
            state,
            target,
            source,
            application,
        )
    );
}

#[test]
fn generalized_evidence_scope_is_canonical_deterministic_and_non_mutating() {
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

    let left = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
        world.clone(),
        state.clone(),
        target.clone(),
        source.clone(),
        application.clone(),
    );

    let right = RecursiveWorldRevisionAbstractionGeneralizationEvidenceScope::scope(
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
    );

    assert_eq!(left, right);

    assert_eq!(world, world_before);

    assert_eq!(state, state_before);

    assert_eq!(source, source_before);

    assert_eq!(application, application_before);

    assert!(left.is_active());
}
