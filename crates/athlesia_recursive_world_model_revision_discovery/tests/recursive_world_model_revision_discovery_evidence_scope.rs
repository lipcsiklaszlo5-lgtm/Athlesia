use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{RecursiveWorldModel, RecursiveWorldRule};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryEvidenceScope, RecursiveWorldRevisionDiscoveryHypothesis,
    RecursiveWorldRevisionDiscoveryHypothesisSet, RecursiveWorldRevisionDiscoveryObservation,
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

fn hypothesis(
    target: RecursiveWorldRule,
    premises: &[usize],
    conclusions: &[usize],
) -> RecursiveWorldRevisionDiscoveryHypothesis {
    RecursiveWorldRevisionDiscoveryHypothesis::discover(target, observation(premises, conclusions))
        .unwrap()
}

fn evidence(
    source: RecursiveWorldRule,
    observation: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(source, unit(observation), kind)
}

#[test]
fn empty_discovery_evidence_scope_is_empty() {
    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(Vec::new()),
    );

    assert_eq!(scope.active_count(), 0);

    assert_eq!(scope.inactive_count(), 0);

    assert_eq!(scope.rejected_count(), 0);

    assert!(scope.pressured_rule().is_none());
}

#[test]
fn valid_discovery_without_evidence_is_inactive() {
    let target = rule(&[1], &[2]);

    let discovered = hypothesis(target.clone(), &[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target]);

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    assert_eq!(scope.active_count(), 0);

    assert_eq!(scope.inactive_hypotheses(), &[discovered,]);
}

#[test]
fn confirming_evidence_keeps_discovery_inactive() {
    let target = rule(&[1], &[2]);

    let discovered = hypothesis(target.clone(), &[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target,
        9,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    assert!(!scope.has_negative_pressure());

    assert_eq!(scope.inactive_hypotheses(), &[discovered,]);
}

#[test]
fn balanced_evidence_keeps_discovery_inactive() {
    let target = rule(&[1], &[2]);

    let discovered = hypothesis(target.clone(), &[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 9, RecursiveWorldEvidenceKind::Confirming),
        evidence(target, 10, RecursiveWorldEvidenceKind::Violating),
    ]);

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    assert_eq!(scope.active_count(), 0);

    assert_eq!(scope.inactive_hypotheses(), &[discovered,]);

    assert!(scope.pressured_rule().is_none());
}

#[test]
fn negative_pressure_activates_matching_discovery() {
    let target = rule(&[1], &[2]);

    let discovered = hypothesis(target.clone(), &[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    assert_eq!(scope.pressured_rule(), Some(&target,));

    assert_eq!(scope.active_hypotheses(), &[discovered,]);

    assert_eq!(scope.inactive_count(), 0);
}

#[test]
fn accepted_discovery_for_other_target_remains_inactive() {
    let pressured = rule(&[1], &[2]);

    let other = rule(&[5], &[6]);

    let other_hypothesis = hypothesis(other.clone(), &[5], &[7]);

    let model = RecursiveWorldModel::new(vec![pressured.clone(), other]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        pressured,
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![other_hypothesis.clone()]),
    );

    assert_eq!(scope.active_count(), 0);

    assert_eq!(scope.inactive_hypotheses(), &[other_hypothesis,]);
}

#[test]
fn rejected_discovery_never_becomes_active() {
    let target = rule(&[1], &[2]);

    let collision = rule(&[5], &[6]);

    let rejected = hypothesis(target.clone(), &[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), collision]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target,
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![rejected.clone()]),
    );

    assert_eq!(scope.active_count(), 0);

    assert_eq!(scope.inactive_count(), 0);

    assert_eq!(scope.rejected_hypotheses(), &[rejected,]);
}

#[test]
fn multiple_active_discoveries_for_pressured_target_are_preserved() {
    let target = rule(&[1], &[2]);

    let first = hypothesis(target.clone(), &[1], &[3]);

    let second = hypothesis(target.clone(), &[1], &[4]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target,
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![second.clone(), first.clone()]),
    );

    assert_eq!(scope.active_hypotheses(), &[first, second,]);
}

#[test]
fn accepted_partition_cardinality_is_preserved() {
    let pressured = rule(&[1], &[2]);

    let other = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![pressured.clone(), other.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![
            hypothesis(pressured, &[1], &[3]),
            hypothesis(other, &[5], &[7]),
        ]),
    );

    assert_eq!(scope.accepted_count(), scope.validation().accepted_count());

    assert_eq!(scope.active_count() + scope.inactive_count(), 2);
}

#[test]
fn highest_pressure_rule_controls_discovery_scope() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let first_hypothesis = hypothesis(first.clone(), &[1], &[3]);

    let second_hypothesis = hypothesis(second.clone(), &[5], &[7]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(first, 9, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 10, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 11, RecursiveWorldEvidenceKind::Violating),
    ]);

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![
            first_hypothesis.clone(),
            second_hypothesis.clone(),
        ]),
    );

    assert_eq!(scope.pressured_rule(), Some(&second,));

    assert_eq!(scope.active_hypotheses(), &[second_hypothesis,]);

    assert_eq!(scope.inactive_hypotheses(), &[first_hypothesis,]);
}

#[test]
fn discovery_scope_preserves_generation_scope_identity() {
    let target = rule(&[1], &[2]);

    let discovered = hypothesis(target.clone(), &[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target,
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let scope = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    let candidate = scope
        .validation()
        .bridge()
        .candidate_for_hypothesis(&discovered)
        .unwrap();

    assert_eq!(scope.generation_scope().active_candidates(), &[candidate,]);
}

#[test]
fn discovery_evidence_scope_is_deterministic_and_non_mutating() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let first = hypothesis(target.clone(), &[1], &[3]);

    let second = hypothesis(target, &[1], &[4]);

    let hypotheses =
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![second.clone(), first.clone()]);

    let model_before = model.clone();

    let state_before = state.clone();

    let hypotheses_before = hypotheses.clone();

    let left =
        RecursiveWorldRevisionDiscoveryEvidenceScope::new(&model, &state, hypotheses.clone());

    let right = RecursiveWorldRevisionDiscoveryEvidenceScope::new(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![first, second]),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(state, state_before);

    assert_eq!(hypotheses, hypotheses_before);
}
