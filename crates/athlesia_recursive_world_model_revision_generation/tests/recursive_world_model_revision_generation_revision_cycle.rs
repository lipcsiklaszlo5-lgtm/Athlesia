use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldModel, RecursiveWorldRevisionBudget, RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceState,
};

use athlesia_recursive_world_model_revision_generation::{
    RecursiveWorldRevisionGenerationCandidate, RecursiveWorldRevisionGenerationCandidateSet,
    RecursiveWorldRevisionGenerationCycle,
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

fn candidate(
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
    basis: &[usize],
) -> RecursiveWorldRevisionGenerationCandidate {
    RecursiveWorldRevisionGenerationCandidate::new(
        target,
        replacement,
        basis.iter().copied().map(unit).collect(),
    )
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
fn empty_generation_cycle_has_no_revision() {
    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &RecursiveWorldModel::new(Vec::new()),
        &RecursiveWorldEvidenceState::empty(),
        RecursiveWorldRevisionGenerationCandidateSet::new(Vec::new()),
        RecursiveWorldRevisionBudget::new(5).unwrap(),
    );

    assert!(!result.has_revision());

    assert!(result.selected_generation_candidates().is_empty());
}

#[test]
fn valid_candidate_without_evidence_does_not_revise() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &RecursiveWorldEvidenceState::empty(),
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target,
            rule(&[1], &[3]),
            &[9],
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.inactive_candidates().len(), 1);

    assert!(!result.has_revision());
}

#[test]
fn confirming_evidence_blocks_generation_revision() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target,
            rule(&[1], &[3]),
            &[9],
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(!result.has_revision());
}

#[test]
fn balanced_evidence_blocks_generation_revision() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(target.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        evidence(target.clone(), 3, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target,
            rule(&[1], &[4]),
            &[9],
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert!(result.pressured_rule().is_none());

    assert!(!result.has_revision());
}

#[test]
fn negative_pressure_candidate_can_revise_world() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target.clone(),
            replacement.clone(),
            &[10],
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.pressured_rule(), Some(&target,));

    assert!(result.has_revision());

    assert!(result.revised_world().unwrap().contains(&replacement,));

    assert!(!result.revised_world().unwrap().contains(&target,));
}

#[test]
fn rejected_generation_candidate_never_reaches_revision_cycle() {
    let target = rule(&[1], &[2]);

    let collision = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![target.clone(), collision.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target,
            collision,
            &[10],
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(result.rejected_candidates().len(), 1);

    assert!(result.active_candidates().is_empty());

    assert!(!result.has_revision());
}

#[test]
fn highest_pressure_rule_controls_generation_revision_scope() {
    let first = rule(&[1], &[2]);

    let second = rule(&[5], &[6]);

    let second_replacement = rule(&[5], &[7]);

    let model = RecursiveWorldModel::new(vec![first.clone(), second.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        evidence(first.clone(), 9, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 10, RecursiveWorldEvidenceKind::Violating),
        evidence(second.clone(), 11, RecursiveWorldEvidenceKind::Violating),
    ]);

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![
            candidate(first, rule(&[1], &[3]), &[12]),
            candidate(second.clone(), second_replacement.clone(), &[13]),
        ]),
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.pressured_rule(), Some(&second,));

    assert_eq!(result.active_candidates().len(), 1);

    assert!(result
        .revised_world()
        .unwrap()
        .contains(&second_replacement,));
}

#[test]
fn cheapest_active_generated_revision_is_selected() {
    let target = rule(&[1], &[2]);

    let cheap = rule(&[1], &[3]);

    let expensive = rule(&[4, 5], &[6, 7]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![
            candidate(target.clone(), expensive, &[10]),
            candidate(target, cheap.clone(), &[11]),
        ]),
        RecursiveWorldRevisionBudget::new(20).unwrap(),
    );

    assert_eq!(result.selected_revision().unwrap().replacement(), &cheap);
}

#[test]
fn over_budget_generated_revision_is_not_selected() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target,
            rule(&[4, 5], &[6, 7]),
            &[10],
        )]),
        RecursiveWorldRevisionBudget::new(1).unwrap(),
    );

    assert_eq!(result.active_candidates().len(), 1);

    assert!(!result.has_revision());

    assert!(result.revised_world().is_none());
}

#[test]
fn selected_generation_provenance_preserves_multiple_basis_paths() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let first = candidate(target.clone(), replacement.clone(), &[10]);

    let second = candidate(target.clone(), replacement.clone(), &[11]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target,
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![second.clone(), first.clone()]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.selected_generation_candidates(),
        vec![first, second,]
    );
}

#[test]
fn selected_revision_preserves_frozen_m33_identity() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let result = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
            target.clone(),
            replacement.clone(),
            &[10],
        )]),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(
        result.revision_cycle().selected().unwrap().revision(),
        result.selected_revision().unwrap()
    );

    assert_eq!(result.selected_revision().unwrap().target(), &target);

    assert_eq!(
        result.selected_revision().unwrap().replacement(),
        &replacement
    );
}

#[test]
fn generation_cycle_does_not_mutate_inputs() {
    let target = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![target.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(evidence(
        target.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let candidates = RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
        target,
        rule(&[1], &[3]),
        &[10],
    )]);

    let model_before = model.clone();

    let state_before = state.clone();

    let candidates_before = candidates.clone();

    let _ = RecursiveWorldRevisionGenerationCycle::evaluate(
        &model,
        &state,
        candidates.clone(),
        RecursiveWorldRevisionBudget::new(10).unwrap(),
    );

    assert_eq!(model, model_before);

    assert_eq!(state, state_before);

    assert_eq!(candidates, candidates_before);
}
