use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldModel, RecursiveWorldRevisionBudget, RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryCycle, RecursiveWorldRevisionDiscoveryHypothesis,
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
fn empty_discovery_cycle_has_no_revision() {
    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(Vec::new()),
        RecursiveWorldRevisionBudget::new(5).unwrap(),
    );

    assert!(!result.has_revision());

    assert!(result.selected_hypotheses().is_empty());
}

#[test]
fn discovery_without_evidence_does_not_revise_world() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(target, &[1], &[3])]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.inactive_hypotheses().len(), 1);

    assert!(!result.has_revision());
}

#[test]
fn confirming_evidence_blocks_discovery_revision() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(target, &[1], &[3])]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(!result.has_revision());
}

#[test]
fn balanced_evidence_blocks_discovery_revision() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 9, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 10, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(target, &[1], &[3])]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(result.pressured_rule().is_none());

    assert!(!result.has_revision());
}

#[test]
fn negative_pressure_discovery_can_revise_world() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
            target.clone(),
            &[1],
            &[3],
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.pressured_rule(), Some(&target,));

    assert!(result.has_revision());

    assert!(result.revised_world().unwrap().contains(&replacement,));

    assert!(!result.revised_world().unwrap().contains(&target,));
}

#[test]
fn rejected_discovery_never_reaches_revision_selection() {
    let target = rule(&[1], &[2]);

    let collision = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), collision]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(target, &[5], &[6])]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.rejected_hypotheses().len(), 1);

    assert!(result.active_hypotheses().is_empty());

    assert!(!result.has_revision());
}

#[test]
fn highest_pressure_rule_controls_discovery_revision() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let replacement = rule(&[5], &[7]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(first.clone(), 9, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 10, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 11, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![
            hypothesis(first, &[1], &[3]),
            hypothesis(second.clone(), &[5], &[7]),
        ]),
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.pressured_rule(), Some(&second,));

    assert_eq!(result.active_hypotheses().len(), 1);

    assert!(result.revised_world().unwrap().contains(&replacement,));
}

#[test]
fn cheapest_active_discovery_revision_is_selected() {
    let target = rule(&[1], &[2]);

    let cheap = rule(&[1], &[3]);

    let expensive = rule(&[4, 5], &[6, 7]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![
            hypothesis(target.clone(), &[4, 5], &[6, 7]),
            hypothesis(target, &[1], &[3]),
        ]),
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.selected_revision().unwrap().replacement(), &cheap);

    assert_ne!(
        result.selected_revision().unwrap().replacement(),
        &expensive
    );
}

#[test]
fn over_budget_discovery_revision_is_not_selected() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
            target,
            &[4, 5],
            &[6, 7],
        )]),
        RecursiveWorldRevisionBudget::new(1).unwrap(),
    );

    assert_eq!(result.active_hypotheses().len(), 1);

    assert!(!result.has_revision());

    assert!(result.revised_world().is_none());
}

#[test]
fn selected_discovery_hypothesis_preserves_provenance() {
    let target = rule(&[1], &[2]);

    let selected = hypothesis(target.clone(), &[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target,
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![selected.clone()]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.selected_hypotheses(), vec![selected,]);
}

#[test]
fn selected_revision_preserves_frozen_generation_identity() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
            target.clone(),
            &[1],
            &[3],
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.selected_revision(),
        result.generation_cycle().selected_revision()
    );

    assert_eq!(result.selected_revision().unwrap().target(), &target);

    assert_eq!(
        result.selected_revision().unwrap().replacement(),
        &replacement
    );
}

#[test]
fn discovery_cycle_is_deterministic_and_non_mutating() {
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

    let left = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        hypotheses.clone(),
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    let right = RecursiveWorldRevisionDiscoveryCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![first, second]),
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(left, right);

    assert_eq!(model, model_before);

    assert_eq!(state, state_before);

    assert_eq!(hypotheses, hypotheses_before);
}
