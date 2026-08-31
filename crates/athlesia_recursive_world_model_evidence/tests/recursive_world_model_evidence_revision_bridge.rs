use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::{
    RecursiveWorldMinimalRevision, RecursiveWorldModel, RecursiveWorldRule,
};

use athlesia_recursive_world_model_evidence::{
    RecursiveWorldEvidenceAssessor, RecursiveWorldEvidenceKind, RecursiveWorldEvidenceRanking,
    RecursiveWorldEvidenceRecord, RecursiveWorldEvidenceRevisionBridge,
    RecursiveWorldEvidenceRevisionBridgeBuilder, RecursiveWorldEvidenceState,
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

fn ranking_for(
    state: &RecursiveWorldEvidenceState,
    rules: Vec<RecursiveWorldRule>,
) -> RecursiveWorldEvidenceRanking {
    RecursiveWorldEvidenceRanking::new(RecursiveWorldEvidenceAssessor::assess_many(state, rules))
}

#[test]
fn empty_ranking_produces_empty_bridge() {
    let bridge = RecursiveWorldEvidenceRevisionBridge::new(
        &RecursiveWorldEvidenceRanking::new(Vec::new()),
        Vec::new(),
    );

    assert!(bridge.pressure().is_none());

    assert!(bridge.is_empty());

    assert!(!bridge.has_negative_pressure());
}

#[test]
fn no_evidence_does_not_create_revision_pressure() {
    let source = rule(&[1], &[2]);

    let ranking = ranking_for(&RecursiveWorldEvidenceState::empty(), vec![source]);

    let bridge = RecursiveWorldEvidenceRevisionBridge::new(&ranking, Vec::new());

    assert!(bridge.pressure().is_none());
}

#[test]
fn confirming_only_evidence_does_not_create_revision_pressure() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        source.clone(),
        2,
        RecursiveWorldEvidenceKind::Confirming,
    ));

    let ranking = ranking_for(&state, vec![source]);

    let bridge = RecursiveWorldEvidenceRevisionBridge::new(&ranking, Vec::new());

    assert!(bridge.pressure().is_none());
}

#[test]
fn balanced_mixed_evidence_does_not_create_revision_pressure() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(source.clone(), 3, RecursiveWorldEvidenceKind::Violating),
    ]);

    let ranking = ranking_for(&state, vec![source]);

    let bridge = RecursiveWorldEvidenceRevisionBridge::new(&ranking, Vec::new());

    assert!(bridge.pressure().is_none());

    assert!(!bridge.has_negative_pressure());
}

#[test]
fn negative_balance_creates_revision_pressure() {
    let source = rule(&[1], &[2]);

    let state = RecursiveWorldEvidenceState::empty().accumulate_many(vec![
        record(source.clone(), 2, RecursiveWorldEvidenceKind::Confirming),
        record(source.clone(), 3, RecursiveWorldEvidenceKind::Violating),
        record(source.clone(), 4, RecursiveWorldEvidenceKind::Violating),
    ]);

    let ranking = ranking_for(&state, vec![source.clone()]);

    let bridge = RecursiveWorldEvidenceRevisionBridge::new(&ranking, Vec::new());

    assert!(bridge.has_negative_pressure());

    assert_eq!(bridge.pressured_rule(), Some(&source,));
}

#[test]
fn bridge_keeps_only_revisions_for_pressured_rule() {
    let pressured = rule(&[1], &[2]);

    let unrelated = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![pressured.clone(), unrelated.clone()]);

    let pressured_revision = revision(&model, pressured.clone(), rule(&[1], &[3]));

    let unrelated_revision = revision(&model, unrelated, rule(&[5], &[7]));

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let ranking = ranking_for(&state, model.rules().to_vec());

    let bridge = RecursiveWorldEvidenceRevisionBridge::new(
        &ranking,
        vec![unrelated_revision, pressured_revision.clone()],
    );

    assert_eq!(bridge.len(), 1);

    assert_eq!(bridge.candidates(), &[pressured_revision,]);
}

#[test]
fn negative_pressure_without_matching_revision_has_empty_candidates() {
    let pressured = rule(&[1], &[2]);

    let unrelated = rule(&[5], &[6]);

    let model = RecursiveWorldModel::new(vec![pressured.clone(), unrelated.clone()]);

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let ranking = ranking_for(&state, model.rules().to_vec());

    let bridge = RecursiveWorldEvidenceRevisionBridge::new(
        &ranking,
        vec![revision(&model, unrelated, rule(&[5], &[7]))],
    );

    assert!(bridge.has_negative_pressure());

    assert!(bridge.is_empty());
}

#[test]
fn multiple_matching_revisions_are_preserved() {
    let pressured = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![pressured.clone()]);

    let first = revision(&model, pressured.clone(), rule(&[1], &[3]));

    let second = revision(&model, pressured.clone(), rule(&[1], &[4]));

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let ranking = ranking_for(&state, vec![pressured]);

    let bridge =
        RecursiveWorldEvidenceRevisionBridge::new(&ranking, vec![second.clone(), first.clone()]);

    assert_eq!(bridge.len(), 2);

    assert!(bridge.candidates().contains(&first,));

    assert!(bridge.candidates().contains(&second,));
}

#[test]
fn duplicate_matching_revisions_are_deduplicated() {
    let pressured = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![pressured.clone()]);

    let change = revision(&model, pressured.clone(), rule(&[1], &[3]));

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let ranking = ranking_for(&state, vec![pressured]);

    let bridge = RecursiveWorldEvidenceRevisionBridge::new(&ranking, vec![change.clone(), change]);

    assert_eq!(bridge.len(), 1);
}

#[test]
fn bridge_builder_matches_direct_construction() {
    let pressured = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![pressured.clone()]);

    let change = revision(&model, pressured.clone(), rule(&[1], &[3]));

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let ranking = ranking_for(&state, vec![pressured]);

    assert_eq!(
        RecursiveWorldEvidenceRevisionBridgeBuilder::build(&ranking, vec![change.clone(),],),
        RecursiveWorldEvidenceRevisionBridge::new(&ranking, vec![change,],)
    );
}

#[test]
fn bridge_is_deterministic_under_revision_order() {
    let pressured = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![pressured.clone()]);

    let first = revision(&model, pressured.clone(), rule(&[1], &[3]));

    let second = revision(&model, pressured.clone(), rule(&[1], &[4]));

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let ranking = ranking_for(&state, vec![pressured]);

    assert_eq!(
        RecursiveWorldEvidenceRevisionBridge::new(&ranking, vec![first.clone(), second.clone(),],),
        RecursiveWorldEvidenceRevisionBridge::new(&ranking, vec![second, first,],)
    );
}

#[test]
fn bridge_does_not_mutate_ranking_or_revision_vector() {
    let pressured = rule(&[1], &[2]);

    let model = RecursiveWorldModel::new(vec![pressured.clone()]);

    let change = revision(&model, pressured.clone(), rule(&[1], &[3]));

    let state = RecursiveWorldEvidenceState::empty().accumulate(record(
        pressured.clone(),
        9,
        RecursiveWorldEvidenceKind::Violating,
    ));

    let ranking = ranking_for(&state, vec![pressured]);

    let ranking_before = ranking.clone();

    let revisions = vec![change];

    let revisions_before = revisions.clone();

    let _ = RecursiveWorldEvidenceRevisionBridge::new(&ranking, revisions.clone());

    assert_eq!(ranking, ranking_before);

    assert_eq!(revisions, revisions_before);
}
