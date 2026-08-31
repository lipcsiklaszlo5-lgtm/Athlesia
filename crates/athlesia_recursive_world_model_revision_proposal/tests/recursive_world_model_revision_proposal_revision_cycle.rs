use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldModel, RecursiveWorldRevisionBudget, RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_proposal::{
    RecursiveWorldRevisionProposal, RecursiveWorldRevisionProposalCycle,
    RecursiveWorldRevisionProposalSet,
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

fn proposal(
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
) -> RecursiveWorldRevisionProposal {
    RecursiveWorldRevisionProposal::new(target, replacement).unwrap()
}

fn evidence(
    source: RecursiveWorldRule,
    observation: usize,
    kind: RecursiveWorldEvidenceKind,
) -> RecursiveWorldEvidenceRecord {
    RecursiveWorldEvidenceRecord::new(source, unit(observation), kind)
}

#[test]
fn empty_cycle_has_no_revision() {
    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        &RecursiveWorldRevisionProposalSet::new(Vec::new()),
        RecursiveWorldRevisionBudget::new(5).unwrap(),
    );

    assert_eq!(result.validations().accepted_count(), 0);

    assert_eq!(result.validations().rejected_count(), 0);

    assert!(!result.has_revision());
}

#[test]
fn valid_proposal_without_evidence_stays_inactive() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target, rule(&[1], &[3]))]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.validations().accepted_count(), 1);

    assert_eq!(result.active_proposal_count(), 0);

    assert!(!result.has_revision());
}

#[test]
fn confirming_evidence_blocks_revision_cycle() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target, rule(&[1], &[3]))]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.active_proposal_count(), 0);

    assert!(!result.has_revision());
}

#[test]
fn balanced_mixed_evidence_blocks_revision_cycle() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 3, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target, rule(&[1], &[4]))]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(result.pressured_rule().is_none());

    assert!(!result.has_revision());
}

#[test]
fn negative_pressure_valid_proposal_can_revise_world() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(
            target.clone(),
            replacement.clone(),
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.pressured_rule(), Some(&target,));

    assert_eq!(result.active_proposal_count(), 1);

    assert!(result.has_revision());

    assert!(result.revised_world().unwrap().contains(&replacement,));

    assert!(!result.revised_world().unwrap().contains(&target,));
}

#[test]
fn invalid_proposal_is_rejected_before_evidence_scope() {
    let target = rule(&[1], &[2]);

    let collision = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), collision.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target, collision)]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.rejected_proposal_count(), 1);

    assert_eq!(result.active_proposal_count(), 0);

    assert!(!result.has_revision());
}

#[test]
fn highest_pressure_rule_controls_active_proposal_scope() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let second_replacement = rule(&[5], &[7]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(first.clone(), 9, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 10, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 11, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![
            proposal(first, rule(&[1], &[3])),
            proposal(second.clone(), second_replacement.clone()),
        ]),
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.pressured_rule(), Some(&second,));

    assert_eq!(result.active_proposal_count(), 1);

    assert!(result
        .revised_world()
        .unwrap()
        .contains(&second_replacement,));
}

#[test]
fn cheapest_active_proposal_revision_is_selected() {
    let target = rule(&[1], &[2]);

    let cheap = rule(&[1], &[3]);

    let expensive = rule(&[4, 5], &[6, 7]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![
            proposal(target.clone(), expensive),
            proposal(target.clone(), cheap.clone()),
        ]),
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.selected_revision().unwrap().replacement(), &cheap);
}

#[test]
fn over_budget_active_proposal_does_not_revise_world() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(target, rule(&[4, 5], &[6, 7]))]),
        RecursiveWorldRevisionBudget::new(1).unwrap(),
    );

    assert_eq!(result.active_proposal_count(), 1);

    assert!(!result.has_revision());

    assert!(result.revised_world().is_none());
}

#[test]
fn proposal_cycle_preserves_m33_selected_revision_identity() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![proposal(
            target.clone(),
            replacement.clone(),
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    let selected = result.selected_revision().unwrap();

    assert_eq!(
        result.revision_cycle().selected().unwrap().revision(),
        selected
    );

    assert_eq!(selected.target(), &target);

    assert_eq!(selected.replacement(), &replacement);
}

#[test]
fn proposal_cycle_is_deterministic_under_proposal_order() {
    let target = rule(&[1], &[2]);

    let first_replacement = rule(&[1], &[3]);

    let second_replacement = rule(&[1], &[4]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let first = proposal(target.clone(), first_replacement);

    let second = proposal(target, second_replacement);

    let budget = RecursiveWorldRevisionBudget::new(20).unwrap();

    let left = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![first.clone(), second.clone()]),
        budget,
    );

    let right = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &RecursiveWorldRevisionProposalSet::new(vec![second, first]),
        budget,
    );

    assert_eq!(left, right);
}

#[test]
fn proposal_cycle_does_not_mutate_inputs() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let evidence_state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let proposals =
        RecursiveWorldRevisionProposalSet::new(vec![proposal(target, rule(&[1], &[3]))]);

    let model_before = model.clone();

    let evidence_before = evidence_state.clone();

    let proposals_before = proposals.clone();

    let _ = RecursiveWorldRevisionProposalCycle::evaluate(
        &model,
        &evidence_state,
        &proposals,
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(model, model_before);

    assert_eq!(evidence_state, evidence_before);

    assert_eq!(proposals, proposals_before);
}
