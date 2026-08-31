use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldMinimalRevision, RecursiveWorldModel, RecursiveWorldRevisionBudget,
    RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceRevisionCycle,
    RecursiveWorldEvidenceState,
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

fn record(
    source: RecursiveWorldRule,
    observation: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(source, unit(observation), kind)
}

fn revision(
    model: &RecursiveWorldModel,
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
) -> RecursiveWorldMinimalRevision {
    RecursiveWorldMinimalRevision::apply(model, target, replacement).unwrap()
}

#[test]
fn empty_inputs_produce_no_revision() {
    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &RecursiveWorldEvidenceState::empty(),
        Vec::new(),
        Vec::new(),
        RecursiveWorldRevisionBudget::new(5).unwrap(),
    );

    assert!(result.ranking().is_empty());

    assert!(result.bridge().is_empty());

    assert!(!result.has_revision());
}

#[test]
fn no_evidence_produces_no_revision() {
    let source = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![source.clone()]);

    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &RecursiveWorldEvidenceState::empty(),
        vec![source.clone()],
        vec![revision(&model, source, rule(&[1], &[3]))],
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(result.pressured_rule().is_none());

    assert!(!result.has_revision());
}

#[test]
fn confirming_evidence_blocks_revision() {
    let source = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![source.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        vec![source.clone()],
        vec![revision(&model, source, rule(&[1], &[3]))],
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(!result.has_revision());
}

#[test]
fn balanced_mixed_evidence_blocks_revision() {
    let source = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![source.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(source.clone(), 3, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        vec![source.clone()],
        vec![revision(&model, source, rule(&[1], &[4]))],
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(!result.has_revision());
}

#[test]
fn negative_evidence_pressure_selects_matching_affordable_revision() {
    let source = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![source.clone()]);

    let change = revision(&model, source.clone(), replacement.clone());

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        vec![source.clone()],
        vec![change.clone()],
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(result.has_revision());

    assert_eq!(result.pressured_rule(), Some(&source,));

    assert_eq!(result.selected_revision(), Some(&change,));

    assert!(result.revised_world().unwrap().contains(&replacement,));
}

#[test]
fn unrelated_revision_is_filtered_before_m33_cycle() {
    let pressured = rule(&[1], &[2]);

    let unrelated = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![pressured.clone(), unrelated.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        model.rules().to_vec(),
        vec![revision(&model, unrelated, rule(&[5], &[7]))],
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(result.bridge().has_negative_pressure());

    assert!(result.bridge().is_empty());

    assert!(!result.has_revision());
}

#[test]
fn cheapest_matching_revision_is_selected() {
    let source = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![source.clone()]);

    let cheap = revision(&model, source.clone(), rule(&[1], &[3]));

    let expensive = revision(&model, source.clone(), rule(&[4, 5], &[6, 7]));

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        vec![source],
        vec![expensive, cheap.clone()],
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.selected_revision(), Some(&cheap,));
}

#[test]
fn over_budget_matching_revision_is_not_selected() {
    let source = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![source.clone()]);

    let change = revision(&model, source.clone(), rule(&[4, 5], &[6, 7]));

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        vec![source],
        vec![change],
        RecursiveWorldRevisionBudget::new(1).unwrap(),
    );

    assert!(result.bridge().has_negative_pressure());

    assert!(!result.has_revision());

    assert!(result.revised_world().is_none());
}

#[test]
fn highest_evidence_pressure_rule_controls_revision_scope() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let first_revision = revision(&model, first.clone(), rule(&[1], &[3]));

    let second_revision = revision(&model, second.clone(), rule(&[5], &[7]));

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(first.clone(), 9, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 10, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 11, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        model.rules().to_vec(),
        vec![first_revision, second_revision.clone()],
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.pressured_rule(), Some(&second,));

    assert_eq!(result.selected_revision(), Some(&second_revision,));
}

#[test]
fn cycle_is_deterministic_under_rule_and_revision_order() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let first_revision = revision(&model, first.clone(), rule(&[1], &[3]));

    let second_revision = revision(&model, second.clone(), rule(&[5], &[7]));

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(first.clone(), 9, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 10, RecursiveWorldEvidenceKind::Violating),
        record(second.clone(), 11, RecursiveWorldEvidenceKind::Violating),
    ]);

    let budget = RecursiveWorldRevisionBudget::new(20).unwrap();

    let left = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        vec![first.clone(), second.clone()],
        vec![first_revision.clone(), second_revision.clone()],
        budget,
    );

    let right = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        vec![second, first],
        vec![second_revision, first_revision],
        budget,
    );

    assert_eq!(left, right);
}

#[test]
fn cycle_preserves_frozen_m33_revision_result_identity() {
    let source = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![source.clone()]);

    let change = revision(&model, source.clone(), rule(&[1], &[3]));

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        vec![source],
        vec![change.clone()],
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.revision_cycle().selected().unwrap().revision(),
        &change
    );

    assert_eq!(result.selected_revision(), Some(&change,));
}

#[test]
fn cycle_does_not_mutate_evidence_state_or_source_vectors() {
    let source = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![source.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let rules = vec![source.clone()];

    let revisions = vec![revision(&model, source, rule(&[1], &[3]))];

    let state_before = state.clone();

    let rules_before = rules.clone();

    let revisions_before = revisions.clone();

    let _ = RecursiveWorldEvidenceRevisionCycle::evaluate(
        &state,
        rules.clone(),
        revisions.clone(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(state, state_before);

    assert_eq!(rules, rules_before);

    assert_eq!(revisions, revisions_before);
}
