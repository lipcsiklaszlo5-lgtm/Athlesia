use std::collections::BTreeSet;

use crate::{PrimitiveSignature, StructuralHypothesis};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StructuralConcept {
    signatures: Vec<PrimitiveSignature>,
    sequence_length: Option<usize>,
}

impl StructuralConcept {
    pub fn new(mut signatures: Vec<PrimitiveSignature>) -> Self {
        signatures.sort();
        signatures.dedup();

        Self {
            signatures,
            sequence_length: None,
        }
    }

    pub fn with_sequence_length(
        mut signatures: Vec<PrimitiveSignature>,
        sequence_length: usize,
    ) -> Self {
        signatures.sort();
        signatures.dedup();

        Self {
            signatures,
            sequence_length: Some(sequence_length),
        }
    }

    pub fn signatures(&self) -> &[PrimitiveSignature] {
        &self.signatures
    }

    pub const fn sequence_length(&self) -> Option<usize> {
        self.sequence_length
    }

    pub fn complexity(&self) -> usize {
        self.signatures.len()
    }

    pub fn contains(&self, signature: PrimitiveSignature) -> bool {
        self.signatures.binary_search(&signature).is_ok()
    }

    pub fn matches_query(&self, query: &StructuralConcept) -> bool {
        if self.signatures != query.signatures {
            return false;
        }

        match query.sequence_length {
            Some(expected) => self.sequence_length == Some(expected),
            None => true,
        }
    }
}

impl From<&StructuralHypothesis> for StructuralConcept {
    fn from(hypothesis: &StructuralHypothesis) -> Self {
        Self::with_sequence_length(
            hypothesis.signatures().to_vec(),
            hypothesis.sequence_length(),
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConceptMemory {
    concepts: BTreeSet<StructuralConcept>,
}

impl ConceptMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, concept: StructuralConcept) -> bool {
        self.concepts.insert(concept)
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    pub fn contains(&self, query: &StructuralConcept) -> bool {
        self.concepts
            .iter()
            .any(|stored| stored.matches_query(query))
    }

    pub fn concepts(&self) -> impl Iterator<Item = &StructuralConcept> {
        self.concepts.iter()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConceptConsolidator;

impl ConceptConsolidator {
    pub const fn new() -> Self {
        Self
    }

    pub fn consolidate(&self, hypotheses: &[StructuralHypothesis]) -> Vec<StructuralConcept> {
        let mut concepts = BTreeSet::new();

        for hypothesis in hypotheses {
            if hypothesis.is_compressive() {
                concepts.insert(StructuralConcept::from(hypothesis));
            }
        }

        concepts.into_iter().collect()
    }

    pub fn consolidate_into(
        &self,
        hypotheses: &[StructuralHypothesis],
        memory: &mut ConceptMemory,
    ) -> usize {
        self.consolidate(hypotheses)
            .into_iter()
            .filter(|concept| memory.insert(concept.clone()))
            .count()
    }
}
