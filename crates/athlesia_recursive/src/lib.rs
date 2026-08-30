use std::collections::BTreeSet;

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveUnit {
    Base(AbstractionUnit),
    CrossLevel(CrossLevelConcept),
    Recursive(Box<RecursiveConcept>),
}

impl RecursiveUnit {
    pub fn is_base(&self) -> bool {
        matches!(self, Self::Base(_))
    }

    pub fn is_cross_level(&self) -> bool {
        matches!(self, Self::CrossLevel(_))
    }

    pub fn is_recursive(&self) -> bool {
        matches!(self, Self::Recursive(_))
    }

    pub fn is_higher_order(&self) -> bool {
        self.is_cross_level() || self.is_recursive()
    }

    pub fn base(&self) -> Option<&AbstractionUnit> {
        match self {
            Self::Base(unit) => Some(unit),
            Self::CrossLevel(_) | Self::Recursive(_) => None,
        }
    }

    pub fn cross_level(&self) -> Option<&CrossLevelConcept> {
        match self {
            Self::CrossLevel(concept) => Some(concept),
            Self::Base(_) | Self::Recursive(_) => None,
        }
    }

    pub fn recursive(&self) -> Option<&RecursiveConcept> {
        match self {
            Self::Recursive(concept) => Some(concept),
            Self::Base(_) | Self::CrossLevel(_) => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveConcept {
    units: Vec<RecursiveUnit>,
}

impl RecursiveConcept {
    pub fn new(units: Vec<RecursiveUnit>) -> Option<Self> {
        let unique: BTreeSet<RecursiveUnit> = units.into_iter().collect();

        if unique.len() < 2 {
            return None;
        }

        let has_higher_order = unique.iter().any(RecursiveUnit::is_higher_order);

        if !has_higher_order {
            return None;
        }

        Some(Self {
            units: unique.into_iter().collect(),
        })
    }

    pub fn units(&self) -> &[RecursiveUnit] {
        &self.units
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    pub fn contains(&self, unit: &RecursiveUnit) -> bool {
        self.units.binary_search(unit).is_ok()
    }

    pub fn base_count(&self) -> usize {
        self.units.iter().filter(|unit| unit.is_base()).count()
    }

    pub fn cross_level_count(&self) -> usize {
        self.units
            .iter()
            .filter(|unit| unit.is_cross_level())
            .count()
    }

    pub fn recursive_count(&self) -> usize {
        self.units.iter().filter(|unit| unit.is_recursive()).count()
    }

    pub fn depth(&self) -> usize {
        1 + self
            .units
            .iter()
            .filter_map(|unit| unit.recursive())
            .map(RecursiveConcept::depth)
            .max()
            .unwrap_or(0)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveMemory {
    concepts: BTreeSet<RecursiveConcept>,
}

impl RecursiveMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.concepts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.concepts.is_empty()
    }

    pub fn insert(&mut self, concept: RecursiveConcept) -> bool {
        self.concepts.insert(concept)
    }

    pub fn contains(&self, concept: &RecursiveConcept) -> bool {
        self.concepts.contains(concept)
    }

    pub fn concepts(&self) -> impl Iterator<Item = &RecursiveConcept> {
        self.concepts.iter()
    }
}
