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
