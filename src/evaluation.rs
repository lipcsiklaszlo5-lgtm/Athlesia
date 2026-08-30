use crate::{PredictionRule, RelationKind, StructuralSequence};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PredictionOutcome {
    Confirmed,
    Violated,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PredictionEvaluation {
    rule: PredictionRule,
    outcome: PredictionOutcome,
}

impl PredictionEvaluation {
    const fn new(rule: PredictionRule, outcome: PredictionOutcome) -> Self {
        Self { rule, outcome }
    }

    pub const fn rule(self) -> PredictionRule {
        self.rule
    }

    pub const fn outcome(self) -> PredictionOutcome {
        self.outcome
    }

    pub const fn is_confirmed(self) -> bool {
        matches!(self.outcome, PredictionOutcome::Confirmed)
    }

    pub const fn is_violated(self) -> bool {
        matches!(self.outcome, PredictionOutcome::Violated)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PredictionEvaluator;

impl PredictionEvaluator {
    pub const fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &self,
        rule: PredictionRule,
        observation: &StructuralSequence,
    ) -> Option<PredictionEvaluation> {
        let reference = observation.role_at(rule.reference())?;

        let target = observation.role_at(rule.target())?;

        let outcome = match rule.kind() {
            RelationKind::Equal => {
                if reference == target {
                    PredictionOutcome::Confirmed
                } else {
                    PredictionOutcome::Violated
                }
            }
        };

        Some(PredictionEvaluation::new(rule, outcome))
    }

    pub fn evaluate_all(
        &self,
        rules: &[PredictionRule],
        observation: &StructuralSequence,
    ) -> Vec<PredictionEvaluation> {
        rules
            .iter()
            .copied()
            .filter_map(|rule| self.evaluate(rule, observation))
            .collect()
    }
}
