use std::collections::BTreeSet;

use crate::{PrimitiveSignature, StructuralHypothesis};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StructuralConcept {
    signatures: Vec<PrimitiveSignature>,
}

impl StructuralConcept {
    pub fn new(mut signatures: Vec<PrimitiveSignature>) -> Self {
        signatures.sort();
        signatures.dedup();

        Self { signatures }
    }

    pub fn signatures(&self) -> &[PrimitiveSignature] {
        &self.signatures
    }

    pub fn complexity(&self) -> usize {
        self.signatures.len()
    }

    pub fn contains(&self, signature: PrimitiveSignature) -> bool {
        self.signatures.binary_search(&signature).is_ok()
    }
}

impl From<&StructuralHypothesis> for StructuralConcept {
    fn from(hypothesis: &StructuralHypothesis) -> Self {
        Self::new(hypothesis.signatures().to_vec())
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

    pub fn contains(&self, concept: &StructuralConcept) -> bool {
        self.concepts.contains(concept)
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
        let concepts = self.consolidate(hypotheses);

        concepts
            .into_iter()
            .filter(|concept| memory.insert(concept.clone()))
            .count()
    }
}
