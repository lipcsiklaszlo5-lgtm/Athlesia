use std::collections::BTreeMap;

use athlesia_recursive::RecursiveUnit;
use athlesia_recursive_world_model::RecursiveWorldRule;
use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;
use athlesia_recursive_world_model_revision_induction::RecursiveWorldRevisionInductionObservationSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionGeneralizationThreshold {
    minimum_support: usize,
}

impl RecursiveWorldRevisionGeneralizationThreshold {
    pub fn new(minimum_support: usize, observation_count: usize) -> Option<Self> {
        if minimum_support < 2 || minimum_support > observation_count {
            return None;
        }

        Some(Self { minimum_support })
    }

    pub fn minimum_support(&self) -> usize {
        self.minimum_support
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizationInput {
    target: RecursiveWorldRule,
    observations: RecursiveWorldRevisionInductionObservationSet,
    threshold: RecursiveWorldRevisionGeneralizationThreshold,
}

impl RecursiveWorldRevisionGeneralizationInput {
    pub fn new(
        target: RecursiveWorldRule,
        observations: RecursiveWorldRevisionInductionObservationSet,
        threshold: RecursiveWorldRevisionGeneralizationThreshold,
    ) -> Option<Self> {
        if threshold.minimum_support() > observations.len() {
            return None;
        }

        Some(Self {
            target,
            observations,
            threshold,
        })
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.observations
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionGeneralizationThreshold {
        self.threshold
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizedStructure {
    target: RecursiveWorldRule,
    observations: RecursiveWorldRevisionInductionObservationSet,
    threshold: RecursiveWorldRevisionGeneralizationThreshold,
    generalized_observation: RecursiveWorldRevisionDiscoveryObservation,
    premise_support: BTreeMap<RecursiveUnit, usize>,
    conclusion_support: BTreeMap<RecursiveUnit, usize>,
}

impl RecursiveWorldRevisionGeneralizedStructure {
    pub fn generalize(input: RecursiveWorldRevisionGeneralizationInput) -> Option<Self> {
        let mut premise_support = BTreeMap::<RecursiveUnit, usize>::new();

        let mut conclusion_support = BTreeMap::<RecursiveUnit, usize>::new();

        for observation in input.observations().observations() {
            for unit in observation.premises() {
                let count = premise_support.entry(unit.clone()).or_insert(0);

                *count = count.saturating_add(1);
            }

            for unit in observation.conclusions() {
                let count = conclusion_support.entry(unit.clone()).or_insert(0);

                *count = count.saturating_add(1);
            }
        }

        let minimum_support = input.threshold().minimum_support();

        let generalized_premises = premise_support
            .iter()
            .filter_map(|(unit, support)| {
                if *support >= minimum_support {
                    Some(unit.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let generalized_conclusions = conclusion_support
            .iter()
            .filter_map(|(unit, support)| {
                if *support >= minimum_support {
                    Some(unit.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let generalized_observation = RecursiveWorldRevisionDiscoveryObservation::new(
            generalized_premises,
            generalized_conclusions,
        )?;

        Some(Self {
            target: input.target().clone(),
            observations: input.observations().clone(),
            threshold: input.threshold(),
            generalized_observation,
            premise_support,
            conclusion_support,
        })
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.observations
    }

    pub fn threshold(&self) -> RecursiveWorldRevisionGeneralizationThreshold {
        self.threshold
    }

    pub fn generalized_observation(&self) -> &RecursiveWorldRevisionDiscoveryObservation {
        &self.generalized_observation
    }

    pub fn generalized_premises(&self) -> &[RecursiveUnit] {
        self.generalized_observation.premises()
    }

    pub fn generalized_conclusions(&self) -> &[RecursiveUnit] {
        self.generalized_observation.conclusions()
    }

    pub fn premise_support(&self, unit: &RecursiveUnit) -> usize {
        self.premise_support.get(unit).copied().unwrap_or(0)
    }

    pub fn conclusion_support(&self, unit: &RecursiveUnit) -> usize {
        self.conclusion_support.get(unit).copied().unwrap_or(0)
    }

    pub fn support_count(&self) -> usize {
        self.observations.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionGeneralizer;

impl RecursiveWorldRevisionGeneralizer {
    pub fn generalize(
        input: RecursiveWorldRevisionGeneralizationInput,
    ) -> Option<RecursiveWorldRevisionGeneralizedStructure> {
        RecursiveWorldRevisionGeneralizedStructure::generalize(input)
    }
}
