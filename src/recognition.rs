use std::collections::BTreeSet;

use crate::{
    ConceptMemory, PrimitiveDiscovery, PrimitiveSignature, RelationalStructure, StructuralConcept,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecognitionResult {
    recognized: Vec<StructuralConcept>,
    observed_signatures: Vec<PrimitiveSignature>,
}

impl RecognitionResult {
    fn new(
        recognized: Vec<StructuralConcept>,
        observed_signatures: Vec<PrimitiveSignature>,
    ) -> Self {
        Self {
            recognized,
            observed_signatures,
        }
    }

    pub fn recognized(&self) -> &[StructuralConcept] {
        &self.recognized
    }

    pub fn observed_signatures(&self) -> &[PrimitiveSignature] {
        &self.observed_signatures
    }

    pub fn count(&self) -> usize {
        self.recognized.len()
    }

    pub fn is_empty(&self) -> bool {
        self.recognized.is_empty()
    }

    pub fn contains(&self, query: &StructuralConcept) -> bool {
        self.recognized
            .iter()
            .any(|recognized| recognized.matches_query(query))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecognitionEngine {
    primitive_discovery: PrimitiveDiscovery,
}

impl RecognitionEngine {
    pub const fn new(minimum_support: usize) -> Self {
        Self {
            primitive_discovery: PrimitiveDiscovery::new(minimum_support),
        }
    }

    pub fn recognize(
        &self,
        memory: &ConceptMemory,
        structure: &RelationalStructure,
    ) -> RecognitionResult {
        let primitives = self.primitive_discovery.discover(structure);

        let observed: BTreeSet<PrimitiveSignature> =
            primitives.iter().map(PrimitiveSignature::from).collect();

        let observed_signatures: Vec<PrimitiveSignature> = observed.iter().copied().collect();

        let mut recognized: Vec<StructuralConcept> = memory
            .concepts()
            .filter(|concept| {
                let length_matches = match concept.sequence_length() {
                    Some(expected) => expected == structure.length(),
                    None => true,
                };

                length_matches
                    && concept
                        .signatures()
                        .iter()
                        .all(|signature| observed.contains(signature))
            })
            .cloned()
            .collect();

        recognized.sort();

        RecognitionResult::new(recognized, observed_signatures)
    }
}

impl Default for RecognitionEngine {
    fn default() -> Self {
        Self::new(2)
    }
}
