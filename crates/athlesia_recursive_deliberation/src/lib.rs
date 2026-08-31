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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveDeliberationChoiceKind {
    Act,
    Experiment,
    Counterfactual,
    NoDecision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveDeliberationChoice {
    kind: RecursiveDeliberationChoiceKind,
    bridge: RecursiveDeliberationControlBridge,
    counterfactual: Option<RecursiveCounterfactualInformationValue>,
}

impl RecursiveDeliberationChoice {
    pub const fn kind(&self) -> RecursiveDeliberationChoiceKind {
        self.kind
    }

    pub fn bridge(&self) -> &RecursiveDeliberationControlBridge {
        &self.bridge
    }

    pub fn control(&self) -> &RecursiveControlUncertaintyDecision {
        self.bridge.control()
    }

    pub fn counterfactual(&self) -> Option<&RecursiveCounterfactualInformationValue> {
        self.counterfactual.as_ref()
    }

    pub fn is_action(&self) -> bool {
        self.kind == RecursiveDeliberationChoiceKind::Act
    }

    pub fn is_experiment(&self) -> bool {
        self.kind == RecursiveDeliberationChoiceKind::Experiment
    }

    pub fn is_counterfactual(&self) -> bool {
        self.kind == RecursiveDeliberationChoiceKind::Counterfactual
    }

    pub fn is_no_decision(&self) -> bool {
        self.kind == RecursiveDeliberationChoiceKind::NoDecision
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveDeliberationChoicePolicy;

impl RecursiveDeliberationChoicePolicy {
    pub fn resolve_kind(
        mode: RecursiveDeliberationControlMode,
        has_informative_counterfactual: bool,
    ) -> RecursiveDeliberationChoiceKind {
        match mode {
            RecursiveDeliberationControlMode::Act => RecursiveDeliberationChoiceKind::Act,
            RecursiveDeliberationControlMode::Experiment => {
                RecursiveDeliberationChoiceKind::Experiment
            }
            RecursiveDeliberationControlMode::NoDecision => {
                if has_informative_counterfactual {
                    RecursiveDeliberationChoiceKind::Counterfactual
                } else {
                    RecursiveDeliberationChoiceKind::NoDecision
                }
            }
        }
    }

    pub fn choose(bridge: RecursiveDeliberationControlBridge) -> RecursiveDeliberationChoice {
        let informative = bridge
            .best_counterfactual()
            .is_some_and(RecursiveCounterfactualInformationValue::is_informative);

        let kind = Self::resolve_kind(bridge.mode(), informative);

        let counterfactual = if kind == RecursiveDeliberationChoiceKind::Counterfactual {
            bridge.best_counterfactual().cloned()
        } else {
            None
        };

        RecursiveDeliberationChoice {
            kind,
            bridge,
            counterfactual,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecursiveDeliberationRiskLimit {
    max_interaction_cost: usize,
    max_outcomes: usize,
}

impl RecursiveDeliberationRiskLimit {
    pub fn new(max_interaction_cost: usize, max_outcomes: usize) -> Option<Self> {
        if max_interaction_cost == 0 || max_outcomes == 0 {
            return None;
        }

        Some(Self {
            max_interaction_cost,
            max_outcomes,
        })
    }

    pub const fn max_interaction_cost(&self) -> usize {
        self.max_interaction_cost
    }

    pub const fn max_outcomes(&self) -> usize {
        self.max_outcomes
    }

    pub fn allows(&self, value: &RecursiveCounterfactualInformationValue) -> bool {
        value.interaction_cost() <= self.max_interaction_cost
            && value.projection().outcome_count() <= self.max_outcomes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveDeliberationRiskStatus {
    NotApplicable,
    Eligible,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveDeliberationRiskAssessment {
    choice: RecursiveDeliberationChoice,
    limit: RecursiveDeliberationRiskLimit,
    status: RecursiveDeliberationRiskStatus,
}

impl RecursiveDeliberationRiskAssessment {
    pub fn choice(&self) -> &RecursiveDeliberationChoice {
        &self.choice
    }

    pub const fn limit(&self) -> RecursiveDeliberationRiskLimit {
        self.limit
    }

    pub const fn status(&self) -> RecursiveDeliberationRiskStatus {
        self.status
    }

    pub fn counterfactual(&self) -> Option<&RecursiveCounterfactualInformationValue> {
        self.choice.counterfactual()
    }

    pub fn is_eligible(&self) -> bool {
        self.status == RecursiveDeliberationRiskStatus::Eligible
    }

    pub fn is_rejected(&self) -> bool {
        self.status == RecursiveDeliberationRiskStatus::Rejected
    }

    pub fn is_not_applicable(&self) -> bool {
        self.status == RecursiveDeliberationRiskStatus::NotApplicable
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveDeliberationRiskGate;

impl RecursiveDeliberationRiskGate {
    pub fn assess(
        choice: RecursiveDeliberationChoice,
        limit: RecursiveDeliberationRiskLimit,
    ) -> RecursiveDeliberationRiskAssessment {
        let status = match choice.kind() {
            RecursiveDeliberationChoiceKind::Counterfactual => match choice.counterfactual() {
                Some(value) if limit.allows(value) => RecursiveDeliberationRiskStatus::Eligible,
                Some(_) => RecursiveDeliberationRiskStatus::Rejected,
                None => RecursiveDeliberationRiskStatus::Rejected,
            },
            RecursiveDeliberationChoiceKind::Act
            | RecursiveDeliberationChoiceKind::Experiment
            | RecursiveDeliberationChoiceKind::NoDecision => {
                RecursiveDeliberationRiskStatus::NotApplicable
            }
        };

        RecursiveDeliberationRiskAssessment {
            choice,
            limit,
            status,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RecursiveDeliberationActionKind {
    Act,
    Experiment,
    BoundedAction,
    NoDecision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveDeliberationActionDecision {
    kind: RecursiveDeliberationActionKind,
    assessment: RecursiveDeliberationRiskAssessment,
    counterfactual: Option<RecursiveCounterfactualInformationValue>,
}

impl RecursiveDeliberationActionDecision {
    pub const fn kind(&self) -> RecursiveDeliberationActionKind {
        self.kind
    }

    pub fn assessment(&self) -> &RecursiveDeliberationRiskAssessment {
        &self.assessment
    }

    pub fn choice(&self) -> &RecursiveDeliberationChoice {
        self.assessment.choice()
    }

    pub fn control(&self) -> &RecursiveControlUncertaintyDecision {
        self.choice().control()
    }

    pub fn counterfactual(&self) -> Option<&RecursiveCounterfactualInformationValue> {
        self.counterfactual.as_ref()
    }

    pub fn is_act(&self) -> bool {
        self.kind == RecursiveDeliberationActionKind::Act
    }

    pub fn is_experiment(&self) -> bool {
        self.kind == RecursiveDeliberationActionKind::Experiment
    }

    pub fn is_bounded_action(&self) -> bool {
        self.kind == RecursiveDeliberationActionKind::BoundedAction
    }

    pub fn is_no_decision(&self) -> bool {
        self.kind == RecursiveDeliberationActionKind::NoDecision
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveDeliberationBoundedActionPolicy;

impl RecursiveDeliberationBoundedActionPolicy {
    pub fn resolve_kind(
        choice: RecursiveDeliberationChoiceKind,
        risk: RecursiveDeliberationRiskStatus,
    ) -> RecursiveDeliberationActionKind {
        match choice {
            RecursiveDeliberationChoiceKind::Act => RecursiveDeliberationActionKind::Act,
            RecursiveDeliberationChoiceKind::Experiment => {
                RecursiveDeliberationActionKind::Experiment
            }
            RecursiveDeliberationChoiceKind::Counterfactual => {
                if risk == RecursiveDeliberationRiskStatus::Eligible {
                    RecursiveDeliberationActionKind::BoundedAction
                } else {
                    RecursiveDeliberationActionKind::NoDecision
                }
            }
            RecursiveDeliberationChoiceKind::NoDecision => {
                RecursiveDeliberationActionKind::NoDecision
            }
        }
    }

    pub fn decide(
        assessment: RecursiveDeliberationRiskAssessment,
    ) -> RecursiveDeliberationActionDecision {
        let kind = Self::resolve_kind(assessment.choice().kind(), assessment.status());

        let counterfactual = if kind == RecursiveDeliberationActionKind::BoundedAction {
            assessment.counterfactual().cloned()
        } else {
            None
        };

        RecursiveDeliberationActionDecision {
            kind,
            assessment,
            counterfactual,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveDeliberationActiveCycleResult {
    request: RecursiveDeliberationRequest,
    bridge: RecursiveDeliberationControlBridge,
    choice: RecursiveDeliberationChoice,
    assessment: RecursiveDeliberationRiskAssessment,
    decision: RecursiveDeliberationActionDecision,
}

impl RecursiveDeliberationActiveCycleResult {
    pub fn request(&self) -> &RecursiveDeliberationRequest {
        &self.request
    }

    pub fn bridge(&self) -> &RecursiveDeliberationControlBridge {
        &self.bridge
    }

    pub fn choice(&self) -> &RecursiveDeliberationChoice {
        &self.choice
    }

    pub fn assessment(&self) -> &RecursiveDeliberationRiskAssessment {
        &self.assessment
    }

    pub fn decision(&self) -> &RecursiveDeliberationActionDecision {
        &self.decision
    }

    pub fn final_kind(&self) -> RecursiveDeliberationActionKind {
        self.decision.kind()
    }

    pub fn counterfactual(&self) -> Option<&RecursiveCounterfactualInformationValue> {
        self.decision.counterfactual()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveDeliberationActiveCycle;

impl RecursiveDeliberationActiveCycle {
    pub fn evaluate(
        request: RecursiveDeliberationRequest,
        risk_limit: RecursiveDeliberationRiskLimit,
    ) -> RecursiveDeliberationActiveCycleResult {
        let bridge = RecursiveDeliberationFoundation::bridge(request.clone());

        let choice = RecursiveDeliberationChoicePolicy::choose(bridge.clone());

        let assessment = RecursiveDeliberationRiskGate::assess(choice.clone(), risk_limit);

        let decision = RecursiveDeliberationBoundedActionPolicy::decide(assessment.clone());

        RecursiveDeliberationActiveCycleResult {
            request,
            bridge,
            choice,
            assessment,
            decision,
        }
    }
}
