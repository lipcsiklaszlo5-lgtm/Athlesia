use std::collections::BTreeMap;

use crate::{
    PartialStructuralState, PredictionEngine, PredictionError, PredictionRule,
    PredictiveStructuralModel,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentCandidate {
    target: usize,
    supporting_rules: Vec<PredictionRule>,
}

impl ExperimentCandidate {
    fn new(target: usize, mut supporting_rules: Vec<PredictionRule>) -> Self {
        supporting_rules.sort();
        supporting_rules.dedup();

        Self {
            target,
            supporting_rules,
        }
    }

    pub const fn target(&self) -> usize {
        self.target
    }

    pub fn supporting_rules(&self) -> &[PredictionRule] {
        &self.supporting_rules
    }

    pub fn information_gain(&self) -> usize {
        self.supporting_rules.len()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExperimentGenerator;

impl ExperimentGenerator {
    pub const fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        model: &PredictiveStructuralModel,
        state: &PartialStructuralState,
    ) -> Result<Vec<ExperimentCandidate>, PredictionError> {
        let predictions = PredictionEngine::new().predict(model, state)?;

        let mut grouped: BTreeMap<usize, Vec<PredictionRule>> = BTreeMap::new();

        for prediction in predictions {
            grouped
                .entry(prediction.target())
                .or_default()
                .push(prediction);
        }

        Ok(grouped
            .into_iter()
            .map(|(target, rules)| ExperimentCandidate::new(target, rules))
            .collect())
    }
}
