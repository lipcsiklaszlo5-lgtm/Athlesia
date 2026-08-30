use std::collections::BTreeSet;

use athlesia::StructuralConcept;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct HierarchicalConcept {
    children: Vec<StructuralConcept>,
}

impl HierarchicalConcept {
    pub fn new(children: Vec<StructuralConcept>) -> Option<Self> {
        let unique: BTreeSet<StructuralConcept> = children.into_iter().collect();

        if unique.len() < 2 {
            return None;
        }

        Some(Self {
            children: unique.into_iter().collect(),
        })
    }

    pub fn children(&self) -> &[StructuralConcept] {
        &self.children
    }

    pub fn arity(&self) -> usize {
        self.children.len()
    }

    pub fn contains(&self, concept: &StructuralConcept) -> bool {
        self.children.binary_search(concept).is_ok()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HierarchicalMemory {
    concepts: BTreeSet<HierarchicalConcept>,
}

impl HierarchicalMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    pub fn insert(&mut self, concept: HierarchicalConcept) -> bool {
        self.concepts.insert(concept)
    }

    pub fn contains(&self, concept: &HierarchicalConcept) -> bool {
        self.concepts.contains(concept)
    }

    pub fn concepts(&self) -> impl Iterator<Item = &HierarchicalConcept> {
        self.concepts.iter()
    }
}
