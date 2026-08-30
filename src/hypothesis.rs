use crate::{RelationKind, StructuralPrimitive};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PrimitiveSignature {
    kind: RelationKind,
    span: usize,
}

impl PrimitiveSignature {
    pub const fn new(kind: RelationKind, span: usize) -> Self {
        Self { kind, span }
    }

    pub const fn kind(self) -> RelationKind {
        self.kind
    }

    pub const fn span(self) -> usize {
        self.span
    }
}

impl From<&StructuralPrimitive> for PrimitiveSignature {
    fn from(primitive: &StructuralPrimitive) -> Self {
        Self::new(primitive.kind(), primitive.span())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralHypothesis {
    signatures: Vec<PrimitiveSignature>,
    sequence_length: usize,
    evidence_count: usize,
    description_cost: usize,
}

impl StructuralHypothesis {
    fn new(
        signatures: Vec<PrimitiveSignature>,
        sequence_length: usize,
        evidence_count: usize,
    ) -> Self {
        let description_cost = signatures.len();

        Self {
            signatures,
            sequence_length,
            evidence_count,
            description_cost,
        }
    }

    pub fn signatures(&self) -> &[PrimitiveSignature] {
        &self.signatures
    }

    pub const fn sequence_length(&self) -> usize {
        self.sequence_length
    }

    pub const fn evidence_count(&self) -> usize {
        self.evidence_count
    }

    pub const fn description_cost(&self) -> usize {
        self.description_cost
    }

    pub fn compression_gain(&self) -> usize {
        self.evidence_count.saturating_sub(self.description_cost)
    }

    pub fn is_compressive(&self) -> bool {
        self.compression_gain() > 0
    }

    pub fn contains(&self, signature: PrimitiveSignature) -> bool {
        self.signatures.binary_search(&signature).is_ok()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HypothesisInducer {
    minimum_gain: usize,
}

impl HypothesisInducer {
    pub const fn new(minimum_gain: usize) -> Self {
        Self { minimum_gain }
    }

    pub const fn minimum_gain(self) -> usize {
        self.minimum_gain
    }

    pub fn induce(&self, primitives: &[StructuralPrimitive]) -> Vec<StructuralHypothesis> {
        let mut hypotheses = Vec::new();

        for primitive in primitives {
            let hypothesis = StructuralHypothesis::new(
                vec![PrimitiveSignature::from(primitive)],
                primitive.sequence_length(),
                primitive.support(),
            );

            if hypothesis.compression_gain() >= self.minimum_gain {
                hypotheses.push(hypothesis);
            }
        }

        if primitives.len() > 1 {
            let sequence_length = primitives[0].sequence_length();

            let same_extent = primitives
                .iter()
                .all(|primitive| primitive.sequence_length() == sequence_length);

            if same_extent {
                let mut signatures: Vec<PrimitiveSignature> =
                    primitives.iter().map(PrimitiveSignature::from).collect();

                signatures.sort();
                signatures.dedup();

                let evidence_count = primitives.iter().map(StructuralPrimitive::support).sum();

                let composite =
                    StructuralHypothesis::new(signatures, sequence_length, evidence_count);

                if composite.compression_gain() >= self.minimum_gain {
                    hypotheses.push(composite);
                }
            }
        }

        hypotheses.sort_by(|left, right| {
            right
                .compression_gain()
                .cmp(&left.compression_gain())
                .then_with(|| left.description_cost().cmp(&right.description_cost()))
                .then_with(|| left.sequence_length().cmp(&right.sequence_length()))
                .then_with(|| left.signatures().cmp(right.signatures()))
        });

        hypotheses
    }
}

impl Default for HypothesisInducer {
    fn default() -> Self {
        Self::new(1)
    }
}
