use athlesia::{
    ConceptConsolidator, ConceptMemory, Encoder, HypothesisInducer, PrimitiveDiscovery,
    PrimitiveSignature, RecognitionEngine, RelationKind, RelationalStructure, StructuralConcept,
};

fn learn<T>(values: &[T]) -> ConceptMemory
where
    T: Eq + std::hash::Hash,
{
    let encoder = Encoder::new();
    let sequence = encoder.encode(values);

    let relations = RelationalStructure::from_sequence(&sequence);

    let primitives = PrimitiveDiscovery::default().discover(&relations);

    let hypotheses = HypothesisInducer::default().induce(&primitives);

    let mut memory = ConceptMemory::new();

    ConceptConsolidator::new().consolidate_into(&hypotheses, &mut memory);

    memory
}

fn observe<T>(values: &[T]) -> RelationalStructure
where
    T: Eq + std::hash::Hash,
{
    let encoder = Encoder::new();

    let sequence = encoder.encode(values);

    RelationalStructure::from_sequence(&sequence)
}

#[test]
fn trained_concept_is_recognized_on_new_values() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    let novel = observe(&[847, 13, 847, 13, 999]);

    let result = RecognitionEngine::default().recognize(&memory, &novel);

    assert_eq!(result.count(), 1);

    assert_eq!(
        result.recognized()[0].signatures(),
        &[PrimitiveSignature::new(RelationKind::Equal, 2,)]
    );
}

#[test]
fn recognition_does_not_require_training_values() {
    let memory = learn(&[10, 20, 10, 20, 30]);

    let novel = observe(&[-7, 999_999, -7, 999_999, 42]);

    let result = RecognitionEngine::default().recognize(&memory, &novel);

    assert_eq!(result.count(), 1);
}

#[test]
fn wrong_structure_is_rejected() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    let wrong = observe(&[10, 20, 30, 20, 40]);

    let result = RecognitionEngine::default().recognize(&memory, &wrong);

    assert!(result.is_empty());
}

#[test]
fn singleton_evidence_is_not_enough() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    let weak = observe(&[10, 20, 10, 30, 40]);

    let result = RecognitionEngine::default().recognize(&memory, &weak);

    assert!(result.is_empty());
}

#[test]
fn recognition_does_not_mutate_concept_memory() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    let before: Vec<StructuralConcept> = memory.concepts().cloned().collect();

    let novel = observe(&[50, 60, 50, 60, 70]);

    let _result = RecognitionEngine::default().recognize(&memory, &novel);

    let after: Vec<StructuralConcept> = memory.concepts().cloned().collect();

    assert_eq!(before, after);
}

#[test]
fn repeated_recognition_is_deterministic() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    let novel = observe(&[847, 13, 847, 13, 999]);

    let engine = RecognitionEngine::default();

    let first = engine.recognize(&memory, &novel);

    let second = engine.recognize(&memory, &novel);

    assert_eq!(first, second);
}

#[test]
fn recognizer_reports_observed_structure_only() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    let novel = observe(&[847, 13, 847, 13, 999]);

    let result = RecognitionEngine::default().recognize(&memory, &novel);

    assert_eq!(
        result.observed_signatures(),
        &[PrimitiveSignature::new(RelationKind::Equal, 2,)]
    );
}

#[test]
fn recognition_is_independent_of_absolute_position() {
    let memory = learn(&[1, 2, 1, 2, 9]);

    let shifted = observe(&[9, 40, 50, 40, 50]);

    let result = RecognitionEngine::default().recognize(&memory, &shifted);

    assert_eq!(result.count(), 1);
}

#[test]
fn composite_concept_requires_all_signatures() {
    let memory = learn(&[1, 1, 1, 1]);

    let target = StructuralConcept::new(vec![
        PrimitiveSignature::new(RelationKind::Equal, 1),
        PrimitiveSignature::new(RelationKind::Equal, 2),
    ]);

    assert!(memory.contains(&target));

    let incomplete = observe(&[1, 2, 1, 2, 3]);

    let result = RecognitionEngine::default().recognize(&memory, &incomplete);

    assert!(!result.contains(&target));
}

#[test]
fn composite_concept_is_recognized_when_complete() {
    let memory = learn(&[1, 1, 1, 1]);

    let target = StructuralConcept::new(vec![
        PrimitiveSignature::new(RelationKind::Equal, 1),
        PrimitiveSignature::new(RelationKind::Equal, 2),
    ]);

    let novel = observe(&[77, 77, 77, 77]);

    let result = RecognitionEngine::default().recognize(&memory, &novel);

    assert!(result.contains(&target));
}

#[test]
fn empty_memory_recognizes_nothing() {
    let memory = ConceptMemory::new();

    let structure = observe(&[1, 2, 1, 2, 3]);

    let result = RecognitionEngine::default().recognize(&memory, &structure);

    assert!(result.is_empty());
}

#[test]
fn recognizer_has_no_training_operation() {
    let engine = RecognitionEngine::default();

    let memory = ConceptMemory::new();

    let structure = observe(&[1, 2, 1, 2, 3]);

    let result = engine.recognize(&memory, &structure);

    assert!(result.is_empty());
    assert!(memory.is_empty());
}
