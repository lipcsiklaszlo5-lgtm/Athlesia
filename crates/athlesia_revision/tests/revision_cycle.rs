use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_revision::{
    CompetingModels, RevisionCycleEngine, RevisionObservation, RevisionPolicy, RevisionStatus,
    StructuralObservationResult,
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

fn models() -> CompetingModels {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    models.record(concept(&[1]), RevisionObservation::Confirmed);

    models.record(concept(&[2]), RevisionObservation::Confirmed);

    models
}

#[test]
fn cycle_selects_discriminative_experiment() {
    let mut models = models();

    let transition = RevisionCycleEngine::new()
        .step(&mut models, StructuralObservationResult::Present)
        .unwrap();

    assert!(transition.experiment().discrimination_gain() > 0);
}

#[test]
fn cycle_preserves_observation() {
    let mut models = models();

    let transition = RevisionCycleEngine::new()
        .step(&mut models, StructuralObservationResult::Absent)
        .unwrap();

    assert_eq!(
        transition.observation(),
        StructuralObservationResult::Absent
    );
}

#[test]
fn cycle_records_evidence_updates() {
    let mut models = models();

    let transition = RevisionCycleEngine::new()
        .step(&mut models, StructuralObservationResult::Present)
        .unwrap();

    assert!(!transition.updates().is_empty());

    assert!(transition
        .updates()
        .iter()
        .all(|update| { update.after().total() == update.before().total() + 1 }));
}

#[test]
fn cycle_records_ranking_before_and_after() {
    let mut models = models();

    let transition = RevisionCycleEngine::new()
        .step(&mut models, StructuralObservationResult::Absent)
        .unwrap();

    assert_eq!(transition.ranking_before().len(), 2);

    assert_eq!(transition.ranking_after().len(), 2);
}

#[test]
fn absent_observation_can_change_preferred_model() {
    let mut models = models();

    let transition = RevisionCycleEngine::new()
        .step(&mut models, StructuralObservationResult::Absent)
        .unwrap();

    assert!(transition.preference_changed());

    assert_ne!(transition.best_before(), transition.best_after());
}

#[test]
fn present_observation_can_preserve_preferred_model() {
    let mut models = models();

    let transition = RevisionCycleEngine::new()
        .step(&mut models, StructuralObservationResult::Present)
        .unwrap();

    assert!(!transition.preference_changed());

    assert_eq!(transition.best_before(), transition.best_after());
}

#[test]
fn repeated_cycles_can_weaken_model() {
    let mut models = models();

    let target = concept(&[1]);

    let engine = RevisionCycleEngine::new();

    let first = engine
        .step(&mut models, StructuralObservationResult::Absent)
        .unwrap();

    let tested = first.updates().first().unwrap().concept().clone();

    engine.step(&mut models, StructuralObservationResult::Absent);

    engine.step(&mut models, StructuralObservationResult::Absent);

    assert_eq!(
        models.assess(&tested).unwrap().status(),
        RevisionStatus::Weakened
    );

    assert!(models.assess(&target).is_some());
}

#[test]
fn weakened_model_remains_in_memory() {
    let mut models = models();
    let engine = RevisionCycleEngine::new();

    let transition = engine
        .step(&mut models, StructuralObservationResult::Absent)
        .unwrap();

    let tested = transition.updates().first().unwrap().concept().clone();

    engine.step(&mut models, StructuralObservationResult::Absent);

    engine.step(&mut models, StructuralObservationResult::Absent);

    assert_eq!(models.len(), 2);

    assert_eq!(
        models.assess(&tested).unwrap().status(),
        RevisionStatus::Weakened
    );
}

#[test]
fn cycle_stops_when_no_discrimination_remains() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    models.record(concept(&[1]), RevisionObservation::Confirmed);

    let before = models.clone();

    let result = RevisionCycleEngine::new().step(&mut models, StructuralObservationResult::Present);

    assert!(result.is_none());
    assert_eq!(models, before);
}

#[test]
fn cycle_is_deterministic() {
    let mut first = models();
    let mut second = models();

    let first_transition =
        RevisionCycleEngine::new().step(&mut first, StructuralObservationResult::Absent);

    let second_transition =
        RevisionCycleEngine::new().step(&mut second, StructuralObservationResult::Absent);

    assert_eq!(first_transition, second_transition);

    assert_eq!(first, second);
}

#[test]
fn transition_contains_structural_state_only() {
    let mut models = models();

    let transition = RevisionCycleEngine::new()
        .step(&mut models, StructuralObservationResult::Present)
        .unwrap();

    assert!(transition.experiment().supporting_models() > 0);

    assert!(transition.best_after().is_some());
}
