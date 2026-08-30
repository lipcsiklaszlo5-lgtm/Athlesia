use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_revision::{CompetingModels, RevisionObservation, RevisionPolicy, RevisionStatus};

fn concept(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        5,
    )
}

#[test]
fn competing_model_set_starts_empty() {
    let models = CompetingModels::default();

    assert!(models.is_empty());
    assert_eq!(models.len(), 0);
    assert!(models.best().is_none());
}

#[test]
fn multiple_models_can_coexist() {
    let mut models = CompetingModels::default();

    models.record(concept(1), RevisionObservation::Confirmed);

    models.record(concept(2), RevisionObservation::Confirmed);

    models.record(concept(3), RevisionObservation::Confirmed);

    assert_eq!(models.len(), 3);
}

#[test]
fn evidence_is_kept_separate_per_model() {
    let mut models = CompetingModels::default();

    let first = concept(1);
    let second = concept(2);

    models.record(first.clone(), RevisionObservation::Confirmed);

    models.record(first.clone(), RevisionObservation::Confirmed);

    models.record(second.clone(), RevisionObservation::Violated);

    assert_eq!(models.assess(&first).unwrap().evidence().confirmations(), 2);

    assert_eq!(models.assess(&second).unwrap().evidence().violations(), 1);
}

#[test]
fn supported_model_ranks_before_contested_model() {
    let mut models = CompetingModels::default();

    let supported = concept(1);
    let contested = concept(2);

    models.record(supported.clone(), RevisionObservation::Confirmed);

    models.record(supported.clone(), RevisionObservation::Confirmed);

    models.record(contested.clone(), RevisionObservation::Confirmed);

    models.record(contested, RevisionObservation::Violated);

    assert_eq!(models.best().unwrap().concept(), &supported);
}

#[test]
fn contested_model_ranks_before_unsupported_model() {
    let mut models = CompetingModels::default();

    let contested = concept(1);
    let unsupported = concept(2);

    models.record(contested.clone(), RevisionObservation::Confirmed);

    models.record(contested.clone(), RevisionObservation::Violated);

    models.record(unsupported, RevisionObservation::Confirmed);

    assert_eq!(models.best().unwrap().concept(), &contested);
}

#[test]
fn weakened_model_is_not_deleted() {
    let mut models = CompetingModels::default();
    let target = concept(1);

    models.record(target.clone(), RevisionObservation::Violated);

    models.record(target.clone(), RevisionObservation::Violated);

    let assessment = models.assess(&target).unwrap();

    assert_eq!(assessment.status(), RevisionStatus::Weakened);

    assert_eq!(models.len(), 1);
}

#[test]
fn stronger_net_support_breaks_same_status_tie() {
    let policy = RevisionPolicy::new(1, 10);
    let mut models = CompetingModels::new(policy);

    let stronger = concept(1);
    let weaker = concept(2);

    models.record(stronger.clone(), RevisionObservation::Confirmed);

    models.record(stronger.clone(), RevisionObservation::Confirmed);

    models.record(weaker.clone(), RevisionObservation::Confirmed);

    let ranked = models.assessments();

    assert_eq!(ranked[0].concept(), &stronger);

    assert_eq!(ranked[1].concept(), &weaker);
}

#[test]
fn evidence_volume_breaks_equal_net_support_tie() {
    let policy = RevisionPolicy::new(10, 10);
    let mut models = CompetingModels::new(policy);

    let richer = concept(1);
    let poorer = concept(2);

    models.record(richer.clone(), RevisionObservation::Confirmed);
    models.record(richer.clone(), RevisionObservation::Confirmed);
    models.record(richer.clone(), RevisionObservation::Violated);

    models.record(poorer.clone(), RevisionObservation::Confirmed);

    let ranked = models.assessments();

    assert_eq!(ranked[0].concept(), &richer);
}

#[test]
fn exact_tie_uses_structural_identity() {
    let mut models = CompetingModels::default();

    let first = concept(1);
    let second = concept(2);

    models.record(second.clone(), RevisionObservation::Confirmed);

    models.record(first.clone(), RevisionObservation::Confirmed);

    let ranked = models.assessments();

    let expected = if first < second { first } else { second };

    assert_eq!(ranked[0].concept(), &expected);
}

#[test]
fn ranking_is_independent_of_insertion_order() {
    let a = concept(1);
    let b = concept(2);
    let c = concept(3);

    let mut first = CompetingModels::default();

    first.record(a.clone(), RevisionObservation::Confirmed);
    first.record(b.clone(), RevisionObservation::Confirmed);
    first.record(c.clone(), RevisionObservation::Violated);

    let mut second = CompetingModels::default();

    second.record(c, RevisionObservation::Violated);
    second.record(b, RevisionObservation::Confirmed);
    second.record(a, RevisionObservation::Confirmed);

    assert_eq!(first.assessments(), second.assessments());
}

#[test]
fn assessment_is_deterministic() {
    let mut models = CompetingModels::default();

    models.record(concept(1), RevisionObservation::Confirmed);

    models.record(concept(2), RevisionObservation::Violated);

    assert_eq!(models.assessments(), models.assessments());
}

#[test]
fn best_model_changes_when_evidence_changes() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    let first = concept(1);
    let second = concept(2);

    models.record(first.clone(), RevisionObservation::Confirmed);

    models.record(second.clone(), RevisionObservation::Confirmed);

    models.record(first.clone(), RevisionObservation::Violated);

    models.record(first, RevisionObservation::Violated);

    assert_eq!(models.best().unwrap().concept(), &second);
}

#[test]
fn ranking_does_not_mutate_models() {
    let mut models = CompetingModels::default();

    models.record(concept(1), RevisionObservation::Confirmed);

    models.record(concept(2), RevisionObservation::Confirmed);

    let before = models.clone();

    let _ = models.assessments();
    let _ = models.best();

    assert_eq!(models, before);
}
