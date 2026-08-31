use athlesia_recursive::RecursiveUnit;
use athlesia_recursive_world_model::RecursiveWorldRule;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionDiscoveryObservation {
    premises: Vec<RecursiveUnit>,
    conclusions: Vec<RecursiveUnit>,
}

impl RecursiveWorldRevisionDiscoveryObservation {
    pub fn new(
        mut premises: Vec<RecursiveUnit>,
        mut conclusions: Vec<RecursiveUnit>,
    ) -> Option<Self> {
        if premises.is_empty() || conclusions.is_empty() {
            return None;
        }

        premises.sort();
        premises.dedup();

        conclusions.sort();
        conclusions.dedup();

        Some(Self {
            premises,
            conclusions,
        })
    }

    pub fn premises(&self) -> &[RecursiveUnit] {
        &self.premises
    }

    pub fn conclusions(&self) -> &[RecursiveUnit] {
        &self.conclusions
    }

    pub fn premise_count(&self) -> usize {
        self.premises.len()
    }

    pub fn conclusion_count(&self) -> usize {
        self.conclusions.len()
    }

    pub fn materialize_rule(&self) -> RecursiveWorldRule {
        RecursiveWorldRule::new(self.premises.clone(), self.conclusions.clone())
            .expect("canonical discovery observation must materialize a world rule")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveWorldRevisionDiscoveryHypothesis {
    target: RecursiveWorldRule,
    observation: RecursiveWorldRevisionDiscoveryObservation,
    replacement: RecursiveWorldRule,
}

impl RecursiveWorldRevisionDiscoveryHypothesis {
    pub fn discover(
        target: RecursiveWorldRule,
        observation: RecursiveWorldRevisionDiscoveryObservation,
    ) -> Option<Self> {
        let replacement = observation.materialize_rule();

        if target == replacement {
            return None;
        }

        Some(Self {
            target,
            observation,
            replacement,
        })
    }

    pub fn target(&self) -> &RecursiveWorldRule {
        &self.target
    }

    pub fn observation(&self) -> &RecursiveWorldRevisionDiscoveryObservation {
        &self.observation
    }

    pub fn replacement(&self) -> &RecursiveWorldRule {
        &self.replacement
    }

    pub fn changes_premises(&self) -> bool {
        self.target.premises() != self.replacement.premises()
    }

    pub fn changes_conclusions(&self) -> bool {
        self.target.conclusions() != self.replacement.conclusions()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionDiscoveryHypothesisSet {
    hypotheses: Vec<RecursiveWorldRevisionDiscoveryHypothesis>,
}

impl RecursiveWorldRevisionDiscoveryHypothesisSet {
    pub fn new(mut hypotheses: Vec<RecursiveWorldRevisionDiscoveryHypothesis>) -> Self {
        hypotheses.sort();
        hypotheses.dedup();

        Self { hypotheses }
    }

    pub fn hypotheses(&self) -> &[RecursiveWorldRevisionDiscoveryHypothesis] {
        &self.hypotheses
    }

    pub fn len(&self) -> usize {
        self.hypotheses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hypotheses.is_empty()
    }

    pub fn contains(&self, hypothesis: &RecursiveWorldRevisionDiscoveryHypothesis) -> bool {
        self.hypotheses.binary_search(hypothesis).is_ok()
    }

    pub fn hypotheses_for_target(
        &self,
        target: &RecursiveWorldRule,
    ) -> Vec<RecursiveWorldRevisionDiscoveryHypothesis> {
        self.hypotheses
            .iter()
            .filter(|hypothesis| hypothesis.target() == target)
            .cloned()
            .collect()
    }
}

use athlesia_recursive_world_model_revision_generation::{
    RecursiveWorldRevisionGenerationCandidate, RecursiveWorldRevisionGenerationCandidateSet,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionDiscoveryGenerationBridge {
    hypotheses: RecursiveWorldRevisionDiscoveryHypothesisSet,
    candidates: RecursiveWorldRevisionGenerationCandidateSet,
}

impl RecursiveWorldRevisionDiscoveryGenerationBridge {
    pub fn new(hypotheses: RecursiveWorldRevisionDiscoveryHypothesisSet) -> Self {
        let candidates = RecursiveWorldRevisionGenerationCandidateSet::new(
            hypotheses
                .hypotheses()
                .iter()
                .filter_map(|hypothesis| {
                    let mut basis = hypothesis.observation().premises().to_vec();

                    basis.extend(hypothesis.observation().conclusions().iter().cloned());

                    basis.sort();
                    basis.dedup();

                    RecursiveWorldRevisionGenerationCandidate::new(
                        hypothesis.target().clone(),
                        hypothesis.replacement().clone(),
                        basis,
                    )
                })
                .collect(),
        );

        Self {
            hypotheses,
            candidates,
        }
    }

    pub fn hypotheses(&self) -> &RecursiveWorldRevisionDiscoveryHypothesisSet {
        &self.hypotheses
    }

    pub fn candidates(&self) -> &RecursiveWorldRevisionGenerationCandidateSet {
        &self.candidates
    }

    pub fn hypothesis_count(&self) -> usize {
        self.hypotheses.len()
    }

    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }

    pub fn candidate_for_hypothesis(
        &self,
        hypothesis: &RecursiveWorldRevisionDiscoveryHypothesis,
    ) -> Option<RecursiveWorldRevisionGenerationCandidate> {
        let mut basis = hypothesis.observation().premises().to_vec();

        basis.extend(hypothesis.observation().conclusions().iter().cloned());

        basis.sort();
        basis.dedup();

        RecursiveWorldRevisionGenerationCandidate::new(
            hypothesis.target().clone(),
            hypothesis.replacement().clone(),
            basis,
        )
    }

    pub fn hypotheses_for_candidate(
        &self,
        candidate: &RecursiveWorldRevisionGenerationCandidate,
    ) -> Vec<RecursiveWorldRevisionDiscoveryHypothesis> {
        self.hypotheses
            .hypotheses()
            .iter()
            .filter(|hypothesis| {
                self.candidate_for_hypothesis(hypothesis).as_ref() == Some(candidate)
            })
            .cloned()
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionDiscoveryGenerationBridgeBuilder;

impl RecursiveWorldRevisionDiscoveryGenerationBridgeBuilder {
    pub fn build(
        hypotheses: RecursiveWorldRevisionDiscoveryHypothesisSet,
    ) -> RecursiveWorldRevisionDiscoveryGenerationBridge {
        RecursiveWorldRevisionDiscoveryGenerationBridge::new(hypotheses)
    }
}

use athlesia_recursive_world_model::RecursiveWorldModel;

use athlesia_recursive_world_model_revision_generation::{
    RecursiveWorldRevisionGenerationValidation, RecursiveWorldRevisionGenerationValidator,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveWorldRevisionDiscoveryValidation {
    bridge: RecursiveWorldRevisionDiscoveryGenerationBridge,
    generation_validation: RecursiveWorldRevisionGenerationValidation,
    accepted_hypotheses: Vec<RecursiveWorldRevisionDiscoveryHypothesis>,
    rejected_hypotheses: Vec<RecursiveWorldRevisionDiscoveryHypothesis>,
}

impl RecursiveWorldRevisionDiscoveryValidation {
    pub fn new(
        model: &RecursiveWorldModel,
        hypotheses: RecursiveWorldRevisionDiscoveryHypothesisSet,
    ) -> Self {
        let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(hypotheses);

        let generation_validation =
            RecursiveWorldRevisionGenerationValidator::validate(model, bridge.candidates().clone());

        let accepted_candidates = generation_validation.accepted_candidates();

        let rejected_candidates = generation_validation.rejected_candidates();

        let mut accepted_hypotheses = accepted_candidates
            .iter()
            .flat_map(|candidate| bridge.hypotheses_for_candidate(candidate))
            .collect::<Vec<_>>();

        let mut rejected_hypotheses = rejected_candidates
            .iter()
            .flat_map(|candidate| bridge.hypotheses_for_candidate(candidate))
            .collect::<Vec<_>>();

        accepted_hypotheses.sort();
        accepted_hypotheses.dedup();

        rejected_hypotheses.sort();
        rejected_hypotheses.dedup();

        Self {
            bridge,
            generation_validation,
            accepted_hypotheses,
            rejected_hypotheses,
        }
    }

    pub fn bridge(&self) -> &RecursiveWorldRevisionDiscoveryGenerationBridge {
        &self.bridge
    }

    pub fn generation_validation(&self) -> &RecursiveWorldRevisionGenerationValidation {
        &self.generation_validation
    }

    pub fn accepted_hypotheses(&self) -> &[RecursiveWorldRevisionDiscoveryHypothesis] {
        &self.accepted_hypotheses
    }

    pub fn rejected_hypotheses(&self) -> &[RecursiveWorldRevisionDiscoveryHypothesis] {
        &self.rejected_hypotheses
    }

    pub fn accepted_count(&self) -> usize {
        self.accepted_hypotheses.len()
    }

    pub fn rejected_count(&self) -> usize {
        self.rejected_hypotheses.len()
    }

    pub fn candidate_count(&self) -> usize {
        self.bridge.candidate_count()
    }

    pub fn hypothesis_count(&self) -> usize {
        self.bridge.hypothesis_count()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveWorldRevisionDiscoveryValidator;

impl RecursiveWorldRevisionDiscoveryValidator {
    pub fn validate(
        model: &RecursiveWorldModel,
        hypotheses: RecursiveWorldRevisionDiscoveryHypothesisSet,
    ) -> RecursiveWorldRevisionDiscoveryValidation {
        RecursiveWorldRevisionDiscoveryValidation::new(model, hypotheses)
    }
}
