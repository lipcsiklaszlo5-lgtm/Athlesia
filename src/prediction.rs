use crate::{PrimitiveSignature, RelationKind, RelationalStructure, StructuralConcept};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PredictionRule {
    reference: usize,
    target: usize,
    kind: RelationKind,
}

impl PredictionRule {
    pub const fn new_equal(reference: usize, target: usize) -> Self {
        assert!(reference < target);

        Self {
            reference,
            target,
            kind: RelationKind::Equal,
        }
    }

    pub const fn reference(self) -> usize {
        self.reference
    }

    pub const fn target(self) -> usize {
        self.target
    }

    pub const fn kind(self) -> RelationKind {
        self.kind
    }

    pub const fn span(self) -> usize {
        self.target - self.reference
    }

    pub const fn signature(self) -> PrimitiveSignature {
        PrimitiveSignature::new(self.kind, self.span())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredictiveStructuralModel {
    concept: StructuralConcept,
    sequence_length: usize,
    rules: Vec<PredictionRule>,
}

impl PredictiveStructuralModel {
    pub fn from_example(
        concept: &StructuralConcept,
        structure: &RelationalStructure,
    ) -> Option<Self> {
        let mut rules = Vec::new();

        for relation in structure.relations() {
            let signature =
                PrimitiveSignature::new(relation.kind(), relation.right() - relation.left());

            if concept.contains(signature) {
                rules.push(PredictionRule::new_equal(relation.left(), relation.right()));
            }
        }

        rules.sort();
        rules.dedup();

        let complete = concept
            .signatures()
            .iter()
            .all(|required| rules.iter().any(|rule| rule.signature() == *required));

        if !complete {
            return None;
        }

        Some(Self {
            concept: concept.clone(),
            sequence_length: structure.length(),
            rules,
        })
    }

    pub fn concept(&self) -> &StructuralConcept {
        &self.concept
    }

    pub const fn sequence_length(&self) -> usize {
        self.sequence_length
    }

    pub fn rules(&self) -> &[PredictionRule] {
        &self.rules
    }

    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialStructuralState {
    observed: Vec<bool>,
}

impl PartialStructuralState {
    pub fn new(length: usize) -> Self {
        Self {
            observed: vec![false; length],
        }
    }

    pub fn from_observed_positions(length: usize, positions: &[usize]) -> Option<Self> {
        let mut state = Self::new(length);

        for &position in positions {
            if position >= length {
                return None;
            }

            state.observed[position] = true;
        }

        Some(state)
    }

    pub fn len(&self) -> usize {
        self.observed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.observed.is_empty()
    }

    pub fn is_observed(&self, position: usize) -> Option<bool> {
        self.observed.get(position).copied()
    }

    pub fn observe(&mut self, position: usize) -> bool {
        match self.observed.get_mut(position) {
            Some(slot) => {
                let changed = !*slot;
                *slot = true;
                changed
            }
            None => false,
        }
    }

    pub fn observed_count(&self) -> usize {
        self.observed.iter().filter(|observed| **observed).count()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredictionError {
    LengthMismatch { expected: usize, actual: usize },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PredictionEngine;

impl PredictionEngine {
    pub const fn new() -> Self {
        Self
    }

    pub fn predict(
        &self,
        model: &PredictiveStructuralModel,
        state: &PartialStructuralState,
    ) -> Result<Vec<PredictionRule>, PredictionError> {
        if model.sequence_length() != state.len() {
            return Err(PredictionError::LengthMismatch {
                expected: model.sequence_length(),
                actual: state.len(),
            });
        }

        let predictions = model
            .rules()
            .iter()
            .copied()
            .filter(|rule| {
                state.is_observed(rule.reference()) == Some(true)
                    && state.is_observed(rule.target()) == Some(false)
            })
            .collect();

        Ok(predictions)
    }
}
