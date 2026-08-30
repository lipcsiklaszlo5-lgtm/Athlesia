use athlesia::{
    ConceptConsolidator, ConceptMemory, Encoder, HypothesisInducer, PrimitiveDiscovery,
    PrimitiveSignature, RecognitionEngine, RelationKind, RelationalStructure, StructuralConcept,
};

fn learn(values: &[i32]) -> ConceptMemory {
    let encoder = Encoder::new();
    let sequence = encoder.encode(values);

    let relations = RelationalStructure::from_sequence(&sequence);

    let primitives = PrimitiveDiscovery::default().discover(&relations);

    let hypotheses = HypothesisInducer::default().induce(&primitives);

    let mut memory = ConceptMemory::new();

    ConceptConsolidator::new().consolidate_into(&hypotheses, &mut memory);

    memory
}

fn recognize(memory: &ConceptMemory, values: &[i32]) -> usize {
    let encoder = Encoder::new();
    let sequence = encoder.encode(values);

    let structure = RelationalStructure::from_sequence(&sequence);

    RecognitionEngine::default()
        .recognize(memory, &structure)
        .count()
}

#[test]
fn learned_concept_retains_sequence_length() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    let concept = memory.concepts().next().unwrap();

    assert_eq!(concept.sequence_length(), Some(5));
}

#[test]
fn exact_length_positive_transfer_still_works() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    assert_eq!(recognize(&memory, &[847, 13, 847, 13, 999,],), 1);
}

#[test]
fn shorter_matching_pattern_is_rejected() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    assert_eq!(recognize(&memory, &[10, 20, 10, 20,],), 0);
}

#[test]
fn longer_matching_pattern_is_rejected() {
    let memory = learn(&[1, 2, 1, 2, 3]);

    assert_eq!(recognize(&memory, &[10, 20, 10, 20, 30, 40,],), 0);
}

#[test]
fn same_relations_at_different_lengths_are_distinct_concepts() {
    let short = learn(&[1, 2, 1, 2]);

    let long = learn(&[1, 2, 1, 2, 3]);

    let short_concept = short.concepts().next().unwrap();

    let long_concept = long.concepts().next().unwrap();

    assert_ne!(short_concept, long_concept);

    assert_eq!(short_concept.sequence_length(), Some(4));

    assert_eq!(long_concept.sequence_length(), Some(5));
}

#[test]
fn lengthless_query_matches_exact_stored_concept() {
    let memory = learn(&[1, 1, 1, 1]);

    let query = StructuralConcept::new(vec![
        PrimitiveSignature::new(RelationKind::Equal, 1),
        PrimitiveSignature::new(RelationKind::Equal, 2),
    ]);

    assert!(memory.contains(&query));
}

#[test]
fn explicit_wrong_length_query_is_rejected() {
    let memory = learn(&[1, 1, 1, 1]);

    let query = StructuralConcept::with_sequence_length(
        vec![
            PrimitiveSignature::new(RelationKind::Equal, 1),
            PrimitiveSignature::new(RelationKind::Equal, 2),
        ],
        5,
    );

    assert!(!memory.contains(&query));
}
