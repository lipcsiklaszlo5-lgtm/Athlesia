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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveObservation {
    units: Vec<RecursiveUnit>,
}

impl RecursiveObservation {
    pub fn new(units: Vec<RecursiveUnit>) -> Self {
        let unique: BTreeSet<RecursiveUnit> = units.into_iter().collect();

        Self {
            units: unique.into_iter().collect(),
        }
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveCandidate {
    concept: RecursiveConcept,
    support: usize,
}

impl RecursiveCandidate {
    pub fn concept(&self) -> &RecursiveConcept {
        &self.concept
    }

    pub const fn support(&self) -> usize {
        self.support
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecursiveDiscovery {
    minimum_support: usize,
}

impl RecursiveDiscovery {
    pub const fn new(minimum_support: usize) -> Self {
        Self { minimum_support }
    }

    pub const fn minimum_support(self) -> usize {
        self.minimum_support
    }

    pub fn discover(&self, observations: &[RecursiveObservation]) -> Vec<RecursiveCandidate> {
        use std::collections::BTreeMap;

        let mut support: BTreeMap<RecursiveConcept, usize> = BTreeMap::new();

        for observation in observations {
            let units = observation.units();

            for left_index in 0..units.len() {
                for right_index in (left_index + 1)..units.len() {
                    let candidate = RecursiveConcept::new(vec![
                        units[left_index].clone(),
                        units[right_index].clone(),
                    ]);

                    if let Some(candidate) = candidate {
                        *support.entry(candidate).or_insert(0) += 1;
                    }
                }
            }
        }

        support
            .into_iter()
            .filter_map(|(concept, count)| {
                if count >= self.minimum_support {
                    Some(RecursiveCandidate {
                        concept,
                        support: count,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn consolidate(
        &self,
        observations: &[RecursiveObservation],
        memory: &mut RecursiveMemory,
    ) -> Vec<RecursiveCandidate> {
        let candidates = self.discover(observations);

        for candidate in &candidates {
            memory.insert(candidate.concept().clone());
        }

        candidates
    }
}

impl Default for RecursiveDiscovery {
    fn default() -> Self {
        Self::new(2)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveMatch {
    concept: RecursiveConcept,
    matched_units: usize,
    observation_size: usize,
}

impl RecursiveMatch {
    pub fn concept(&self) -> &RecursiveConcept {
        &self.concept
    }

    pub const fn matched_units(&self) -> usize {
        self.matched_units
    }

    pub const fn observation_size(&self) -> usize {
        self.observation_size
    }

    pub fn is_exact_context(&self) -> bool {
        self.matched_units == self.observation_size
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveRecognizer;

impl RecursiveRecognizer {
    pub const fn new() -> Self {
        Self
    }

    pub fn recognizes(
        &self,
        concept: &RecursiveConcept,
        observation: &RecursiveObservation,
    ) -> bool {
        concept
            .units()
            .iter()
            .all(|unit| observation.units().binary_search(unit).is_ok())
    }

    pub fn recognize(
        &self,
        concept: &RecursiveConcept,
        observation: &RecursiveObservation,
    ) -> Option<RecursiveMatch> {
        if !self.recognizes(concept, observation) {
            return None;
        }

        Some(RecursiveMatch {
            concept: concept.clone(),
            matched_units: concept.len(),
            observation_size: observation.len(),
        })
    }

    pub fn recognize_memory(
        &self,
        memory: &RecursiveMemory,
        observation: &RecursiveObservation,
    ) -> Vec<RecursiveMatch> {
        memory
            .concepts()
            .filter_map(|concept| self.recognize(concept, observation))
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursivePrediction {
    concept: RecursiveConcept,
    observed_units: usize,
    missing_units: Vec<RecursiveUnit>,
}

impl RecursivePrediction {
    pub fn concept(&self) -> &RecursiveConcept {
        &self.concept
    }

    pub const fn observed_units(&self) -> usize {
        self.observed_units
    }

    pub fn missing_units(&self) -> &[RecursiveUnit] {
        &self.missing_units
    }

    pub fn missing_count(&self) -> usize {
        self.missing_units.len()
    }

    pub fn is_single_step_completion(&self) -> bool {
        self.missing_units.len() == 1
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursivePredictor;

impl RecursivePredictor {
    pub const fn new() -> Self {
        Self
    }

    pub fn predict(
        &self,
        concept: &RecursiveConcept,
        observation: &RecursiveObservation,
    ) -> Option<RecursivePrediction> {
        let observed_units = concept
            .units()
            .iter()
            .filter(|unit| observation.units().binary_search(unit).is_ok())
            .count();

        if observed_units == 0 || observed_units == concept.len() {
            return None;
        }

        let missing_units = concept
            .units()
            .iter()
            .filter(|unit| observation.units().binary_search(unit).is_err())
            .cloned()
            .collect();

        Some(RecursivePrediction {
            concept: concept.clone(),
            observed_units,
            missing_units,
        })
    }

    pub fn predict_memory(
        &self,
        memory: &RecursiveMemory,
        observation: &RecursiveObservation,
    ) -> Vec<RecursivePrediction> {
        let mut predictions: Vec<RecursivePrediction> = memory
            .concepts()
            .filter_map(|concept| self.predict(concept, observation))
            .collect();

        predictions.sort_by(|left, right| {
            left.missing_count()
                .cmp(&right.missing_count())
                .then_with(|| right.observed_units().cmp(&left.observed_units()))
                .then_with(|| left.concept().cmp(right.concept()))
        });

        predictions
    }

    pub fn best_prediction(
        &self,
        memory: &RecursiveMemory,
        observation: &RecursiveObservation,
    ) -> Option<RecursivePrediction> {
        self.predict_memory(memory, observation).into_iter().next()
    }
}
