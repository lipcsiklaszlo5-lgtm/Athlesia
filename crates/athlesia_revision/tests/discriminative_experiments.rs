use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_revision::{
    CompetingModels, DiscriminativeExperimentSelector, RevisionObservation, RevisionPolicy,
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

fn confirmed(models: &mut CompetingModels, target: StructuralConcept) {
    models.record(target, RevisionObservation::Confirmed);
}

#[test]
fn no_models_produce_no_experiment() {
    let models = CompetingModels::default();

    assert!(DiscriminativeExperimentSelector::new()
        .select(&models)
        .is_none());
}

#[test]
fn one_model_produces_no_discriminative_experiment() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1]));

    assert!(DiscriminativeExperimentSelector::new()
        .select(&models)
        .is_none());
}

#[test]
fn identical_models_have_no_discriminating_signature() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1, 2]));

    let same_identity = concept(&[2, 1]);

    confirmed(&mut models, same_identity);

    assert_eq!(models.len(), 1);

    assert!(DiscriminativeExperimentSelector::new()
        .select(&models)
        .is_none());
}

#[test]
fn differing_models_generate_candidate() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1]));

    confirmed(&mut models, concept(&[2]));

    let candidates = DiscriminativeExperimentSelector::new().generate(&models);

    assert_eq!(candidates.len(), 2);

    assert_eq!(candidates[0].discrimination_gain(), 1);
}

#[test]
fn candidate_counts_support_and_opposition() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1]));

    confirmed(&mut models, concept(&[1, 2]));

    confirmed(&mut models, concept(&[2]));

    let signature = PrimitiveSignature::new(RelationKind::Equal, 1);

    let candidate = DiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|candidate| candidate.signature() == signature)
        .unwrap();

    assert_eq!(candidate.supporting_models(), 2);

    assert_eq!(candidate.opposing_models(), 1);

    assert_eq!(candidate.discrimination_gain(), 2);
}

#[test]
fn balanced_partition_has_high_information_value() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1]));

    confirmed(&mut models, concept(&[1, 3]));

    confirmed(&mut models, concept(&[2]));

    confirmed(&mut models, concept(&[2, 3]));

    let candidates = DiscriminativeExperimentSelector::new().generate(&models);

    let best = candidates.first().unwrap();

    assert_eq!(best.discrimination_gain(), 4);

    assert_eq!(best.supporting_models(), 2);

    assert_eq!(best.opposing_models(), 2);
}

#[test]
fn selector_prefers_largest_model_partition() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1]));

    confirmed(&mut models, concept(&[1, 3]));

    confirmed(&mut models, concept(&[2]));

    confirmed(&mut models, concept(&[2, 3]));

    confirmed(&mut models, concept(&[3]));

    let selected = DiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    assert_eq!(selected.discrimination_gain(), 6);
}

#[test]
fn universally_shared_signature_is_not_candidate() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1, 2]));

    confirmed(&mut models, concept(&[1, 3]));

    let shared = PrimitiveSignature::new(RelationKind::Equal, 1);

    let candidates = DiscriminativeExperimentSelector::new().generate(&models);

    assert!(candidates
        .iter()
        .all(|candidate| { candidate.signature() != shared }));
}

#[test]
fn weakened_model_is_excluded_from_discrimination() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    let first = concept(&[1]);
    let second = concept(&[2]);
    let weakened = concept(&[3]);

    confirmed(&mut models, first);

    confirmed(&mut models, second);

    models.record(weakened.clone(), RevisionObservation::Violated);

    models.record(weakened, RevisionObservation::Violated);

    let signature_three = PrimitiveSignature::new(RelationKind::Equal, 3);

    let candidates = DiscriminativeExperimentSelector::new().generate(&models);

    assert!(candidates
        .iter()
        .all(|candidate| { candidate.signature() != signature_three }));
}

#[test]
fn candidate_generation_is_deterministic() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1]));

    confirmed(&mut models, concept(&[2]));

    confirmed(&mut models, concept(&[3]));

    let selector = DiscriminativeExperimentSelector::new();

    assert_eq!(selector.generate(&models), selector.generate(&models));
}

#[test]
fn insertion_order_does_not_change_candidates() {
    let policy = RevisionPolicy::new(1, 2);

    let a = concept(&[1]);
    let b = concept(&[2]);
    let c = concept(&[3]);

    let mut first = CompetingModels::new(policy);

    confirmed(&mut first, a.clone());
    confirmed(&mut first, b.clone());
    confirmed(&mut first, c.clone());

    let mut second = CompetingModels::new(policy);

    confirmed(&mut second, c);
    confirmed(&mut second, a);
    confirmed(&mut second, b);

    let selector = DiscriminativeExperimentSelector::new();

    assert_eq!(selector.generate(&first), selector.generate(&second));
}

#[test]
fn tie_break_uses_structural_signature_order() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1]));

    confirmed(&mut models, concept(&[2]));

    let selected = DiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    assert_eq!(
        selected.signature(),
        PrimitiveSignature::new(RelationKind::Equal, 1,)
    );
}

#[test]
fn selection_does_not_mutate_competing_models() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1]));

    confirmed(&mut models, concept(&[2]));

    let before = models.clone();

    let selector = DiscriminativeExperimentSelector::new();

    let _ = selector.generate(&models);
    let _ = selector.select(&models);

    assert_eq!(models, before);
}

#[test]
fn experiment_contains_structural_information_only() {
    let mut models = CompetingModels::new(RevisionPolicy::new(1, 2));

    confirmed(&mut models, concept(&[1]));

    confirmed(&mut models, concept(&[2]));

    let selected = DiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    assert!(selected.supporting_models() > 0);

    assert!(selected.opposing_models() > 0);
}
