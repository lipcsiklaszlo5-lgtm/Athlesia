use crate::StructuralSequence;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RelationKind {
    Equal,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StructuralRelation {
    left: usize,
    right: usize,
    kind: RelationKind,
}

impl StructuralRelation {
    pub const fn new_equal(left: usize, right: usize) -> Self {
        assert!(left < right);
        Self {
            left,
            right,
            kind: RelationKind::Equal,
        }
    }

    pub const fn left(self) -> usize {
        self.left
    }

    pub const fn right(self) -> usize {
        self.right
    }

    pub const fn kind(self) -> RelationKind {
        self.kind
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalStructure {
    length: usize,
    relations: Vec<StructuralRelation>,
}

impl RelationalStructure {
    pub fn from_sequence(sequence: &StructuralSequence) -> Self {
        let roles = sequence.roles();
        let mut relations = Vec::new();

        for left in 0..roles.len() {
            for right in (left + 1)..roles.len() {
                if roles[left] == roles[right] {
                    relations.push(StructuralRelation::new_equal(left, right));
                }
            }
        }

        Self {
            length: sequence.len(),
            relations,
        }
    }

    pub fn length(&self) -> usize {
        self.length
    }

    pub fn relations(&self) -> &[StructuralRelation] {
        &self.relations
    }

    pub fn relation_count(&self) -> usize {
        self.relations.len()
    }

    pub fn has_relation(&self, left: usize, right: usize, kind: RelationKind) -> bool {
        let (canonical_left, canonical_right) = if left < right {
            (left, right)
        } else {
            (right, left)
        };

        self.relations.iter().any(|relation| {
            relation.left() == canonical_left
                && relation.right() == canonical_right
                && relation.kind() == kind
        })
    }
}
