pub mod concept;
pub mod encoder;
pub mod hypothesis;
pub mod primitive;
pub mod recognition;
pub mod relation;
pub mod role;
pub mod structure;

pub use concept::{ConceptConsolidator, ConceptMemory, StructuralConcept};
pub use encoder::Encoder;
pub use hypothesis::{HypothesisInducer, PrimitiveSignature, StructuralHypothesis};
pub use primitive::{PrimitiveDiscovery, PrimitiveOccurrence, StructuralPrimitive};
pub use recognition::{RecognitionEngine, RecognitionResult};
pub use relation::{RelationKind, RelationalStructure, StructuralRelation};
pub use role::Role;
pub use structure::StructuralSequence;

pub fn architecture_name() -> &'static str {
    "Athlesia"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn architecture_name_is_stable() {
        assert_eq!(architecture_name(), "Athlesia");
    }

    #[test]
    fn structural_sequence_can_be_constructed() {
        let sequence = StructuralSequence::new(vec![
            Role::new(0),
            Role::new(1),
            Role::new(0),
            Role::new(1),
            Role::new(2),
        ]);

        assert_eq!(sequence.len(), 5);
        assert_eq!(sequence.roles()[0], Role::new(0));
        assert_eq!(sequence.roles()[2], Role::new(0));
    }
}
