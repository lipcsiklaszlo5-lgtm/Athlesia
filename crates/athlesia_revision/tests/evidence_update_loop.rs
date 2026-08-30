use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_revision::{
    CompetingModels, DiscriminativeExperimentSelector, EvidenceUpdateLoop, RevisionObservation,
    RevisionPolicy, RevisionStatus, StructuralObservationResult,
};

fn concept(spans: &[usize]) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        spans
            .iter()
            .copied()
            .map(|span| PrimitiveSignature::new(RelationKind::Equal, span))
            .collect(),
        6,
    )
}

fn seeded_models() -> CompetingModels {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    models.record(concept(&[1]), RevisionObservation::Confirmed);

    models.record(concept(&[2]), RevisionObservation::Confirmed);

    models
}

#[test]
fn present_observation_confirms_supporting_model() {
    let mut models = seeded_models();

    let experiment = DiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    let updates = EvidenceUpdateLoop::new().apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Present,
    );

    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].before().confirmations(), 1);
    assert_eq!(updates[0].after().confirmations(), 2);
    assert_eq!(updates[0].after().violations(), 0);
}

#[test]
fn absent_observation_violates_supporting_model() {
    let mut models = seeded_models();

    let experiment = DiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    let updates = EvidenceUpdateLoop::new().apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].before().violations(), 0);
    assert_eq!(updates[0].after().violations(), 1);
}

#[test]
fn non_supporting_model_is_not_falsely_confirmed() {
    let mut models = seeded_models();

    let experiment = DiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    let signature = experiment.signature();

    let untouched = models
        .assessments()
        .into_iter()
        .find(|assessment| !assessment.concept().contains(signature))
        .unwrap()
        .concept()
        .clone();

    let before = models.assess(&untouched).unwrap().evidence();

    EvidenceUpdateLoop::new().apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Present,
    );

    let after = models.assess(&untouched).unwrap().evidence();

    assert_eq!(before, after);
}

#[test]
fn non_supporting_model_is_not_falsely_violated() {
    let mut models = seeded_models();

    let experiment = DiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    let signature = experiment.signature();

    let untouched = models
        .assessments()
        .into_iter()
        .find(|assessment| !assessment.concept().contains(signature))
        .unwrap()
        .concept()
        .clone();

    let before = models.assess(&untouched).unwrap().evidence();

    EvidenceUpdateLoop::new().apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    let after = models.assess(&untouched).unwrap().evidence();

    assert_eq!(before, after);
}

#[test]
fn absent_evidence_can_change_model_preference() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    let first = concept(&[1]);
    let second = concept(&[2]);

    models.record(first.clone(), RevisionObservation::Confirmed);

    models.record(second.clone(), RevisionObservation::Confirmed);

    let initial_best = models.best().unwrap().concept().clone();

    let experiment = DiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    EvidenceUpdateLoop::new().apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    let new_best = models.best().unwrap().concept().clone();

    assert_ne!(initial_best, new_best);
}

#[test]
fn repeated_absence_can_weaken_model() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    let first = concept(&[1]);
    let second = concept(&[2]);

    models.record(first.clone(), RevisionObservation::Confirmed);

    models.record(second, RevisionObservation::Confirmed);

    let signature = PrimitiveSignature::new(RelationKind::Equal, 1);

    let experiment = DiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|candidate| candidate.signature() == signature)
        .unwrap();

    let engine = EvidenceUpdateLoop::new();

    engine.apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    assert_eq!(
        models.assess(&first).unwrap().status(),
        RevisionStatus::Contested
    );

    engine.apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    assert_eq!(
        models.assess(&first).unwrap().status(),
        RevisionStatus::Contested
    );

    engine.apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    assert_eq!(
        models.assess(&first).unwrap().status(),
        RevisionStatus::Weakened
    );
}

#[test]
fn weakened_models_are_not_updated_again() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    let first = concept(&[1]);
    let second = concept(&[2]);

    models.record(first.clone(), RevisionObservation::Confirmed);

    models.record(second, RevisionObservation::Confirmed);

    let signature = PrimitiveSignature::new(RelationKind::Equal, 1);

    let experiment = DiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|candidate| candidate.signature() == signature)
        .unwrap();

    let engine = EvidenceUpdateLoop::new();

    engine.apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    engine.apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    engine.apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    assert_eq!(
        models.assess(&first).unwrap().status(),
        RevisionStatus::Weakened
    );

    let before = models.assess(&first).unwrap().evidence();

    let updates = engine.apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Absent,
    );

    let after = models.assess(&first).unwrap().evidence();

    assert!(updates.is_empty());
    assert_eq!(before, after);
}

#[test]
fn update_records_before_and_after_state() {
    let mut models = seeded_models();

    let experiment = DiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    let updates = EvidenceUpdateLoop::new().apply(
        &mut models,
        &experiment,
        StructuralObservationResult::Present,
    );

    assert_eq!(updates[0].before().total() + 1, updates[0].after().total());

    assert_eq!(
        updates[0].observation(),
        StructuralObservationResult::Present
    );
}

#[test]
fn select_and_apply_executes_complete_loop() {
    let mut models = seeded_models();

    let result = EvidenceUpdateLoop::new()
        .select_and_apply(&mut models, StructuralObservationResult::Absent);

    assert!(result.is_some());

    let (experiment, updates) = result.unwrap();

    assert!(experiment.discrimination_gain() > 0);
    assert!(!updates.is_empty());
}

#[test]
fn no_discriminative_experiment_means_no_update() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    models.record(concept(&[1]), RevisionObservation::Confirmed);

    let before = models.clone();

    let result = EvidenceUpdateLoop::new()
        .select_and_apply(&mut models, StructuralObservationResult::Present);

    assert!(result.is_none());
    assert_eq!(models, before);
}

#[test]
fn update_loop_is_deterministic() {
    let mut first = seeded_models();
    let mut second = seeded_models();

    let first_result =
        EvidenceUpdateLoop::new().select_and_apply(&mut first, StructuralObservationResult::Absent);

    let second_result = EvidenceUpdateLoop::new()
        .select_and_apply(&mut second, StructuralObservationResult::Absent);

    assert_eq!(first_result, second_result);
    assert_eq!(first, second);
}

#[test]
fn update_contains_structural_information_only() {
    let mut models = seeded_models();

    let result = EvidenceUpdateLoop::new()
        .select_and_apply(&mut models, StructuralObservationResult::Present)
        .unwrap();

    let (experiment, updates) = result;

    assert!(experiment.supporting_models() > 0);

    assert!(updates
        .iter()
        .all(|update| { update.after().total() > update.before().total() }));
}
