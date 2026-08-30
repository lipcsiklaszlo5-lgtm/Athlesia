use crate::{
    ExperimentGenerator, ExperimentSelection, ExperimentSelector, PartialStructuralState,
    PredictionEvaluation, PredictionEvaluator, PredictionOutcome, PredictionRule,
    PredictiveStructuralModel, StructuralSequence,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActiveInferenceError {
    StateLengthMismatch { expected: usize, actual: usize },
    ObservationLengthMismatch { expected: usize, actual: usize },
    NoExperimentAvailable,
    ObservationUnavailable { target: usize },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveInferenceTransition {
    selected: ExperimentSelection,
    evaluations: Vec<PredictionEvaluation>,
    before: PartialStructuralState,
    after: PartialStructuralState,
}

impl ActiveInferenceTransition {
    fn new(
        selected: ExperimentSelection,
        evaluations: Vec<PredictionEvaluation>,
        before: PartialStructuralState,
        after: PartialStructuralState,
    ) -> Self {
        Self {
            selected,
            evaluations,
            before,
            after,
        }
    }

    pub fn selected(&self) -> &ExperimentSelection {
        &self.selected
    }

    pub fn evaluations(&self) -> &[PredictionEvaluation] {
        &self.evaluations
    }

    pub fn before(&self) -> &PartialStructuralState {
        &self.before
    }

    pub fn after(&self) -> &PartialStructuralState {
        &self.after
    }

    pub fn confirmed_count(&self) -> usize {
        self.evaluations
            .iter()
            .filter(|evaluation| evaluation.outcome() == PredictionOutcome::Confirmed)
            .count()
    }

    pub fn violated_count(&self) -> usize {
        self.evaluations
            .iter()
            .filter(|evaluation| evaluation.outcome() == PredictionOutcome::Violated)
            .count()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ActiveInferenceEngine;

impl ActiveInferenceEngine {
    pub const fn new() -> Self {
        Self
    }

    pub fn step(
        &self,
        model: &PredictiveStructuralModel,
        state: &PartialStructuralState,
        observation: &StructuralSequence,
    ) -> Result<ActiveInferenceTransition, ActiveInferenceError> {
        if state.len() != model.sequence_length() {
            return Err(ActiveInferenceError::StateLengthMismatch {
                expected: model.sequence_length(),
                actual: state.len(),
            });
        }

        if observation.len() != model.sequence_length() {
            return Err(ActiveInferenceError::ObservationLengthMismatch {
                expected: model.sequence_length(),
                actual: observation.len(),
            });
        }

        let candidates = ExperimentGenerator::new()
            .generate(model, state)
            .map_err(|error| match error {
                crate::PredictionError::LengthMismatch { expected, actual } => {
                    ActiveInferenceError::StateLengthMismatch { expected, actual }
                }
            })?;

        let selected = ExperimentSelector::new()
            .select(&candidates)
            .ok_or(ActiveInferenceError::NoExperimentAvailable)?;

        let target = selected.target();

        if observation.role_at(target).is_none() {
            return Err(ActiveInferenceError::ObservationUnavailable { target });
        }

        let rules: Vec<PredictionRule> = selected.candidate().supporting_rules().to_vec();

        let evaluations = PredictionEvaluator::new().evaluate_all(&rules, observation);

        let before = state.clone();

        let mut after = state.clone();

        if !after.observe(target) {
            return Err(ActiveInferenceError::ObservationUnavailable { target });
        }

        Ok(ActiveInferenceTransition::new(
            selected,
            evaluations,
            before,
            after,
        ))
    }
}
