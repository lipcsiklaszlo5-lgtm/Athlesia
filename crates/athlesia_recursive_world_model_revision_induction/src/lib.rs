use athlesia_recursive::RecursiveUnit;
use athlesia_recursive_world_model::RecursiveWorldRule;
use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryObservation;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionInductionObservationSet {
    observations: Vec<RecursiveWorldRevisionDiscoveryObservation>,
}

impl RecursiveWorldRevisionInductionObservationSet {
    pub fn new(mut observations: Vec<RecursiveWorldRevisionDiscoveryObservation>) -> Option<Self> {
        observations.sort();
        observations.dedup();

        if observations.len() < 2 {
            return None;
        }

        Some(Self { observations })
    }

    pub fn observations(&self) -> &[RecursiveWorldRevisionDiscoveryObservation] {
        &self.observations
    }

    pub fn len(&self) -> usize {
        self.observations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    pub fn contains(&self, observation: &RecursiveWorldRevisionDiscoveryObservation) -> bool {
        self.observations.binary_search(observation).is_ok()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionInductionInput {
    target: RecursiveWorldRule,
    observations: RecursiveWorldRevisionInductionObservationSet,
}

impl RecursiveWorldRevisionInductionInput {
    pub fn new(
        target: RecursiveWorldRule,
        observations: RecursiveWorldRevisionInductionObservationSet,
    ) -> Self {
        Self {
            target,
            observations,
        }
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.observations
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionInducedStructure {
    target: RecursiveWorldRule,
    observations: RecursiveWorldRevisionInductionObservationSet,
    induced_observation: RecursiveWorldRevisionDiscoveryObservation,
}

impl RecursiveWorldRevisionInducedStructure {
    pub fn induce(input: RecursiveWorldRevisionInductionInput) -> Option<Self> {
        let observations = input.observations().observations();

        let first = observations
            .first()
            .expect("induction observation set must contain at least two observations");

        let mut common_premises = first.premises().to_vec();

        let mut common_conclusions = first.conclusions().to_vec();

        for observation in observations.iter().skip(1) {
            common_premises.retain(|unit| observation.premises().binary_search(unit).is_ok());

            common_conclusions.retain(|unit| observation.conclusions().binary_search(unit).is_ok());
        }

        let induced_observation =
            RecursiveWorldRevisionDiscoveryObservation::new(common_premises, common_conclusions)?;

        Some(Self {
            target: input.target().clone(),
            observations: input.observations().clone(),
            induced_observation,
        })
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        &self.observations
    }

    pub fn induced_observation(&self) -> &RecursiveWorldRevisionDiscoveryObservation {
        &self.induced_observation
    }

    pub fn support_count(&self) -> usize {
        self.observations.len()
    }

    pub fn induced_premises(&self) -> &[RecursiveUnit] {
        self.induced_observation.premises()
    }

    pub fn induced_conclusions(&self) -> &[RecursiveUnit] {
        self.induced_observation.conclusions()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionInducer;

impl RecursiveWorldRevisionInducer {
    pub fn induce(
        input: RecursiveWorldRevisionInductionInput,
    ) -> Option<RecursiveWorldRevisionInducedStructure> {
        RecursiveWorldRevisionInducedStructure::induce(input)
    }
}

use athlesia_recursive_world_model_revision_discovery::RecursiveWorldRevisionDiscoveryHypothesis;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionInductionDiscoveryBridge {
    induced: RecursiveWorldRevisionInducedStructure,
    hypothesis: RecursiveWorldRevisionDiscoveryHypothesis,
}

impl RecursiveWorldRevisionInductionDiscoveryBridge {
    pub fn new(induced: RecursiveWorldRevisionInducedStructure) -> Option<Self> {
        let hypothesis = RecursiveWorldRevisionDiscoveryHypothesis::discover(
            induced.target().clone(),
            induced.induced_observation().clone(),
        )?;

        Some(Self {
            induced,
            hypothesis,
        })
    }

    pub fn induced(&self) -> &RecursiveWorldRevisionInducedStructure {
        &self.induced
    }

    pub fn hypothesis(&self) -> &RecursiveWorldRevisionDiscoveryHypothesis {
        &self.hypothesis
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        self.hypothesis.target()
    }

    pub fn replacement(&self) -> &RecursiveWorldRule {
        self.hypothesis.replacement()
    }

    pub fn support_count(&self) -> usize {
        self.induced.support_count()
    }

    pub fn source_observations(&self) -> &RecursiveWorldRevisionInductionObservationSet {
        self.induced.observations()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionInductionDiscoveryBridgeBuilder;

impl RecursiveWorldRevisionInductionDiscoveryBridgeBuilder {
    pub fn build(
        induced: RecursiveWorldRevisionInducedStructure,
    ) -> Option<RecursiveWorldRevisionInductionDiscoveryBridge> {
        RecursiveWorldRevisionInductionDiscoveryBridge::new(induced)
    }
}
