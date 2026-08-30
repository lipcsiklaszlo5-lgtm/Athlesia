use std::collections::BTreeSet;

use athlesia::StructuralConcept;
use athlesia_hierarchy::HierarchicalConcept;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum AbstractionUnit {
    Structural(StructuralConcept),
    Hierarchical(HierarchicalConcept),
}

impl AbstractionUnit {
    pub fn is_structural(&self) -> bool {
        matches!(self, Self::Structural(_))
    }

    pub fn is_hierarchical(&self) -> bool {
        matches!(self, Self::Hierarchical(_))
    }

    pub fn structural(&self) -> Option<&StructuralConcept> {
        match self {
            Self::Structural(concept) => Some(concept),
            Self::Hierarchical(_) => None,
        }
    }

    pub fn hierarchical(&self) -> Option<&HierarchicalConcept> {
        match self {
            Self::Structural(_) => None,
            Self::Hierarchical(concept) => Some(concept),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct CrossLevelConcept {
    units: Vec<AbstractionUnit>,
}

impl CrossLevelConcept {
    pub fn new(units: Vec<AbstractionUnit>) -> Option<Self> {
        let unique: BTreeSet<AbstractionUnit> = units.into_iter().collect();

        let has_structural = unique.iter().any(AbstractionUnit::is_structural);

        let has_hierarchical = unique.iter().any(AbstractionUnit::is_hierarchical);

        if !has_structural || !has_hierarchical {
            return None;
        }

        Some(Self {
            units: unique.into_iter().collect(),
        })
    }

    pub fn units(&self) -> &[AbstractionUnit] {
        &self.units
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    pub fn structural_count(&self) -> usize {
        self.units
            .iter()
            .filter(|unit| unit.is_structural())
            .count()
    }

    pub fn hierarchical_count(&self) -> usize {
        self.units
            .iter()
            .filter(|unit| unit.is_hierarchical())
            .count()
    }

    pub fn contains(&self, unit: &AbstractionUnit) -> bool {
        self.units.binary_search(unit).is_ok()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CrossLevelMemory {
    concepts: BTreeSet<CrossLevelConcept>,
}

impl CrossLevelMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    pub fn insert(&mut self, concept: CrossLevelConcept) -> bool {
        self.concepts.insert(concept)
    }

    pub fn contains(&self, concept: &CrossLevelConcept) -> bool {
        self.concepts.contains(concept)
    }

    pub fn concepts(&self) -> impl Iterator<Item = &CrossLevelConcept> {
        self.concepts.iter()
    }
}
