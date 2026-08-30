use athlesia::{
    ConceptConsolidator, ConceptMemory, Encoder, HypothesisInducer, PrimitiveDiscovery,
    PrimitiveSignature, RelationKind, RelationalStructure, StructuralConcept, StructuralHypothesis,
};

fn induce<T>(values: &[T]) -> Vec<StructuralHypothesis>
where
    T: Eq + std::hash::Hash,
{
    let encoder = Encoder::new();

    let sequence = encoder.encode(values);

    let relations = RelationalStructure::from_sequence(&sequence);

    let primitives = PrimitiveDiscovery::default().discover(&relations);

    HypothesisInducer::default().induce(&primitives)
}

fn concepts<T>(values: &[T]) -> Vec<StructuralConcept>
where
    T: Eq + std::hash::Hash,
{
    let hypotheses = induce(values);

    ConceptConsolidator::new().consolidate(&hypotheses)
}

#[test]
fn hypothesis_becomes_structural_concept() {
    let result = concepts(&[1, 2, 1, 2, 3]);

    assert_eq!(result.len(), 1);

    assert_eq!(
        result[0].signatures(),
        &[PrimitiveSignature::new(RelationKind::Equal, 2,)]
    );
}

#[test]
fn concept_excludes_hypothesis_evidence_count() {
    let hypotheses = induce(&[1, 2, 1, 2, 3]);

    let concepts = ConceptConsolidator::new().consolidate(&hypotheses);

    assert_eq!(concepts[0].signatures(), hypotheses[0].signatures());

    assert_eq!(concepts[0].complexity(), 1);
}

#[test]
fn concept_identity_is_value_invariant() {
    let first = concepts(&[1, 2, 1, 2, 3]);

    let second = concepts(&[847, 13, 847, 13, 999]);

    assert_eq!(first, second);
}

#[test]
fn concept_identity_is_position_independent() {
    let first = concepts(&[1, 2, 1, 2, 9]);

    let second = concepts(&[9, 1, 2, 1, 2]);

    assert_eq!(first, second);
}

#[test]
fn concept_signature_is_canonicalized() {
    let concept = StructuralConcept::new(vec![
        PrimitiveSignature::new(RelationKind::Equal, 2),
        PrimitiveSignature::new(RelationKind::Equal, 1),
        PrimitiveSignature::new(RelationKind::Equal, 2),
    ]);

    assert_eq!(
        concept.signatures(),
        &[
            PrimitiveSignature::new(RelationKind::Equal, 1,),
            PrimitiveSignature::new(RelationKind::Equal, 2,),
        ]
    );
}

#[test]
fn duplicate_hypotheses_do_not_duplicate_concepts() {
    let hypotheses = induce(&[1, 2, 1, 2, 3]);

    let duplicated = vec![
        hypotheses[0].clone(),
        hypotheses[0].clone(),
        hypotheses[0].clone(),
    ];

    let result = ConceptConsolidator::new().consolidate(&duplicated);

    assert_eq!(result.len(), 1);
}

#[test]
fn concept_memory_deduplicates_identity() {
    let concept = StructuralConcept::new(vec![PrimitiveSignature::new(RelationKind::Equal, 2)]);

    let mut memory = ConceptMemory::new();

    assert!(memory.insert(concept.clone()));

    assert!(!memory.insert(concept.clone()));

    assert_eq!(memory.len(), 1);
    assert!(memory.contains(&concept));
}

#[test]
fn repeated_training_does_not_grow_memory() {
    let hypotheses = induce(&[1, 2, 1, 2, 3]);

    let consolidator = ConceptConsolidator::new();

    let mut memory = ConceptMemory::new();

    let first_added = consolidator.consolidate_into(&hypotheses, &mut memory);

    let second_added = consolidator.consolidate_into(&hypotheses, &mut memory);

    assert_eq!(first_added, 1);
    assert_eq!(second_added, 0);
    assert_eq!(memory.len(), 1);
}

#[test]
fn equivalent_observations_share_one_concept() {
    let first = induce(&[1, 2, 1, 2, 3]);

    let second = induce(&[847, 13, 847, 13, 999]);

    let consolidator = ConceptConsolidator::new();

    let mut memory = ConceptMemory::new();

    assert_eq!(consolidator.consolidate_into(&first, &mut memory,), 1);

    assert_eq!(consolidator.consolidate_into(&second, &mut memory,), 0);

    assert_eq!(memory.len(), 1);
}

#[test]
fn concept_memory_has_deterministic_iteration_order() {
    let first = StructuralConcept::new(vec![PrimitiveSignature::new(RelationKind::Equal, 2)]);

    let second = StructuralConcept::new(vec![PrimitiveSignature::new(RelationKind::Equal, 1)]);

    let mut memory = ConceptMemory::new();

    memory.insert(first.clone());
    memory.insert(second.clone());

    let stored: Vec<StructuralConcept> = memory.concepts().cloned().collect();

    assert_eq!(stored, vec![second, first]);
}

#[test]
fn composite_hypothesis_becomes_composite_concept() {
    let result = concepts(&[1, 1, 1, 1]);

    assert_eq!(result.len(), 3);

    assert!(result.iter().any(|concept| {
        concept.complexity() == 2
            && concept.contains(PrimitiveSignature::new(RelationKind::Equal, 1))
            && concept.contains(PrimitiveSignature::new(RelationKind::Equal, 2))
    }));
}

#[test]
fn empty_hypothesis_set_creates_no_concepts() {
    let result = ConceptConsolidator::new().consolidate(&[]);

    assert!(result.is_empty());
}

#[test]
fn empty_concept_memory_is_valid() {
    let memory = ConceptMemory::new();

    assert!(memory.is_empty());
    assert_eq!(memory.len(), 0);
}

#[test]
fn concept_contains_structural_description_only() {
    let result = concepts(&[847, 13, 847, 13, 999]);

    assert_eq!(result.len(), 1);

    let concept = &result[0];

    assert_eq!(concept.complexity(), 1);

    assert!(concept.contains(PrimitiveSignature::new(RelationKind::Equal, 2,)));
}
