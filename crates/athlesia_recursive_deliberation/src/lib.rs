use athlesia_recursive_control::RecursiveControlUncertaintyDecision;

use athlesia_recursive_counterfactual::{
    RecursiveCounterfactualInformationValue, RecursiveCounterfactualSelection,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveDeliberationRequest {
    control: RecursiveControlUncertaintyDecision,
    counterfactual: RecursiveCounterfactualSelection,
}

impl RecursiveDeliberationRequest {
    pub fn new(
        control: RecursiveControlUncertaintyDecision,
        counterfactual: RecursiveCounterfactualSelection,
    ) -> Self {
        Self {
            control,
            counterfactual,
        }
    }

    pub fn control(&self) -> &RecursiveControlUncertaintyDecision {
        &self.control
    }

    pub fn counterfactual(&self) -> &RecursiveCounterfactualSelection {
        &self.counterfactual
    }

    pub fn counterfactual_len(&self) -> usize {
        self.counterfactual.len()
    }

    pub fn has_counterfactual_frontier(&self) -> bool {
        !self.counterfactual.is_empty()
    }

    pub fn best_counterfactual(&self) -> Option<&RecursiveCounterfactualInformationValue> {
        self.counterfactual.best()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveDeliberationFoundation;

impl RecursiveDeliberationFoundation {
    pub fn prepare(
        control: RecursiveControlUncertaintyDecision,
        counterfactual: RecursiveCounterfactualSelection,
    ) -> RecursiveDeliberationRequest {
        RecursiveDeliberationRequest::new(control, counterfactual)
    }
}
