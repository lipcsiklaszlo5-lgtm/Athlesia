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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossLevelObservation {
    units: Vec<AbstractionUnit>,
}

impl CrossLevelObservation {
    pub fn new(units: Vec<AbstractionUnit>) -> Self {
        let unique: BTreeSet<AbstractionUnit> = units.into_iter().collect();

        Self {
            units: unique.into_iter().collect(),
        }
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossLevelCandidate {
    concept: CrossLevelConcept,
    support: usize,
}

impl CrossLevelCandidate {
    pub fn concept(&self) -> &CrossLevelConcept {
        &self.concept
    }

    pub const fn support(&self) -> usize {
        self.support
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CrossLevelDiscovery {
    minimum_support: usize,
}

impl CrossLevelDiscovery {
    pub const fn new(minimum_support: usize) -> Self {
        Self { minimum_support }
    }

    pub const fn minimum_support(self) -> usize {
        self.minimum_support
    }

    pub fn discover(&self, observations: &[CrossLevelObservation]) -> Vec<CrossLevelCandidate> {
        use std::collections::BTreeMap;

        let mut support: BTreeMap<CrossLevelConcept, usize> = BTreeMap::new();

        for observation in observations {
            let structural_units: Vec<AbstractionUnit> = observation
                .units()
                .iter()
                .filter(|unit| unit.is_structural())
                .cloned()
                .collect();

            let hierarchical_units: Vec<AbstractionUnit> = observation
                .units()
                .iter()
                .filter(|unit| unit.is_hierarchical())
                .cloned()
                .collect();

            for structural in &structural_units {
                for hierarchical in &hierarchical_units {
                    let candidate =
                        CrossLevelConcept::new(vec![structural.clone(), hierarchical.clone()]);

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
                    Some(CrossLevelCandidate {
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
        observations: &[CrossLevelObservation],
        memory: &mut CrossLevelMemory,
    ) -> Vec<CrossLevelCandidate> {
        let candidates = self.discover(observations);

        for candidate in &candidates {
            memory.insert(candidate.concept().clone());
        }

        candidates
    }
}

impl Default for CrossLevelDiscovery {
    fn default() -> Self {
        Self::new(2)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossLevelMatch {
    concept: CrossLevelConcept,
    matched_units: usize,
    observation_size: usize,
}

impl CrossLevelMatch {
    pub fn concept(&self) -> &CrossLevelConcept {
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
pub struct CrossLevelRecognizer;

impl CrossLevelRecognizer {
    pub const fn new() -> Self {
        Self
    }

    pub fn recognizes(
        &self,
        concept: &CrossLevelConcept,
        observation: &CrossLevelObservation,
    ) -> bool {
        concept
            .units()
            .iter()
            .all(|unit| observation.units().binary_search(unit).is_ok())
    }

    pub fn recognize(
        &self,
        concept: &CrossLevelConcept,
        observation: &CrossLevelObservation,
    ) -> Option<CrossLevelMatch> {
        if !self.recognizes(concept, observation) {
            return None;
        }

        Some(CrossLevelMatch {
            concept: concept.clone(),
            matched_units: concept.len(),
            observation_size: observation.len(),
        })
    }

    pub fn recognize_memory(
        &self,
        memory: &CrossLevelMemory,
        observation: &CrossLevelObservation,
    ) -> Vec<CrossLevelMatch> {
        memory
            .concepts()
            .filter_map(|concept| self.recognize(concept, observation))
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossLevelPrediction {
    concept: CrossLevelConcept,
    observed_units: usize,
    missing_units: Vec<AbstractionUnit>,
}

impl CrossLevelPrediction {
    pub fn concept(&self) -> &CrossLevelConcept {
        &self.concept
    }

    pub const fn observed_units(&self) -> usize {
        self.observed_units
    }

    pub fn missing_units(&self) -> &[AbstractionUnit] {
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
pub struct CrossLevelPredictor;

impl CrossLevelPredictor {
    pub const fn new() -> Self {
        Self
    }

    pub fn predict(
        &self,
        concept: &CrossLevelConcept,
        observation: &CrossLevelObservation,
    ) -> Option<CrossLevelPrediction> {
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

        Some(CrossLevelPrediction {
            concept: concept.clone(),
            observed_units,
            missing_units,
        })
    }

    pub fn predict_memory(
        &self,
        memory: &CrossLevelMemory,
        observation: &CrossLevelObservation,
    ) -> Vec<CrossLevelPrediction> {
        let mut predictions: Vec<CrossLevelPrediction> = memory
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
        memory: &CrossLevelMemory,
        observation: &CrossLevelObservation,
    ) -> Option<CrossLevelPrediction> {
        self.predict_memory(memory, observation).into_iter().next()
    }
}
