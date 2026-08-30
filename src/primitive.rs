use std::collections::BTreeMap;

use crate::{RelationKind, RelationalStructure};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PrimitiveOccurrence {
    left: usize,
    right: usize,
}

impl PrimitiveOccurrence {
    pub const fn new(left: usize, right: usize) -> Self {
        assert!(left < right);
        Self { left, right }
    }

    pub const fn left(self) -> usize {
        self.left
    }

    pub const fn right(self) -> usize {
        self.right
    }

    pub const fn span(self) -> usize {
        self.right - self.left
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructuralPrimitive {
    kind: RelationKind,
    span: usize,
    sequence_length: usize,
    occurrences: Vec<PrimitiveOccurrence>,
}

impl StructuralPrimitive {
    fn new(
        kind: RelationKind,
        span: usize,
        sequence_length: usize,
        occurrences: Vec<PrimitiveOccurrence>,
    ) -> Self {
        Self {
            kind,
            span,
            sequence_length,
            occurrences,
        }
    }

    pub const fn kind(&self) -> RelationKind {
        self.kind
    }

    pub const fn span(&self) -> usize {
        self.span
    }

    pub const fn sequence_length(&self) -> usize {
        self.sequence_length
    }

    pub fn support(&self) -> usize {
        self.occurrences.len()
    }

    pub fn occurrences(&self) -> &[PrimitiveOccurrence] {
        &self.occurrences
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimitiveDiscovery {
    minimum_support: usize,
}

impl PrimitiveDiscovery {
    pub const fn new(minimum_support: usize) -> Self {
        assert!(minimum_support > 0);
        Self { minimum_support }
    }

    pub const fn minimum_support(self) -> usize {
        self.minimum_support
    }

    pub fn discover(&self, structure: &RelationalStructure) -> Vec<StructuralPrimitive> {
        let mut groups: BTreeMap<(RelationKind, usize), Vec<PrimitiveOccurrence>> = BTreeMap::new();

        for relation in structure.relations() {
            let span = relation.right() - relation.left();

            groups
                .entry((relation.kind(), span))
                .or_default()
                .push(PrimitiveOccurrence::new(relation.left(), relation.right()));
        }

        groups
            .into_iter()
            .filter_map(|((kind, span), occurrences)| {
                if occurrences.len() >= self.minimum_support {
                    Some(StructuralPrimitive::new(
                        kind,
                        span,
                        structure.length(),
                        occurrences,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for PrimitiveDiscovery {
    fn default() -> Self {
        Self::new(2)
    }
}
