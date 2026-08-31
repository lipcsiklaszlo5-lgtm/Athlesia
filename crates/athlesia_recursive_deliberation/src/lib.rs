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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveDeliberationControlMode {
    Act,
    Experiment,
    NoDecision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveDeliberationControlBridge {
    request: RecursiveDeliberationRequest,
    mode: RecursiveDeliberationControlMode,
}

impl RecursiveDeliberationControlBridge {
    pub fn new(request: RecursiveDeliberationRequest) -> Self {
        let mode = Self::classify(request.control());

        Self { request, mode }
    }

    pub fn classify(
        control: &RecursiveControlUncertaintyDecision,
    ) -> RecursiveDeliberationControlMode {
        match control {
            RecursiveControlUncertaintyDecision::Act(_) => RecursiveDeliberationControlMode::Act,
            RecursiveControlUncertaintyDecision::Experiment(_) => {
                RecursiveDeliberationControlMode::Experiment
            }
            RecursiveControlUncertaintyDecision::NoDecision => {
                RecursiveDeliberationControlMode::NoDecision
            }
        }
    }

    pub fn request(&self) -> &RecursiveDeliberationRequest {
        &self.request
    }

    pub const fn mode(&self) -> RecursiveDeliberationControlMode {
        self.mode
    }

    pub fn control(&self) -> &RecursiveControlUncertaintyDecision {
        self.request.control()
    }

    pub fn counterfactual(&self) -> &RecursiveCounterfactualSelection {
        self.request.counterfactual()
    }

    pub fn best_counterfactual(&self) -> Option<&RecursiveCounterfactualInformationValue> {
        self.request.best_counterfactual()
    }

    pub fn has_action(&self) -> bool {
        self.mode == RecursiveDeliberationControlMode::Act
    }

    pub fn has_experiment(&self) -> bool {
        self.mode == RecursiveDeliberationControlMode::Experiment
    }

    pub fn is_undecided(&self) -> bool {
        self.mode == RecursiveDeliberationControlMode::NoDecision
    }
}

impl RecursiveDeliberationFoundation {
    pub fn bridge(request: RecursiveDeliberationRequest) -> RecursiveDeliberationControlBridge {
        RecursiveDeliberationControlBridge::new(request)
    }
}
