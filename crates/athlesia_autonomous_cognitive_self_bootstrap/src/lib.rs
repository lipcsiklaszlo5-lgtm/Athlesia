use athlesia_mindstone_sparse_cognition::CognitiveStructure;
use std::cmp::Ordering;

pub const BOOTSTRAP_SIGNAL_MAX: u16 = 1000;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BootstrapSignal(u16);

impl BootstrapSignal {
    pub fn new(value: u16) -> Option<Self> {
        if value <= BOOTSTRAP_SIGNAL_MAX {
            Some(Self(value))
        } else {
            None
        }
    }

    pub const fn zero() -> Self {
        Self(0)
    }

    pub const fn maximum() -> Self {
        Self(BOOTSTRAP_SIGNAL_MAX)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelfBootstrapBounds {
    max_action_affordances: usize,
    max_hypotheses: usize,
    max_candidate_frontier: usize,
}

impl SelfBootstrapBounds {
    pub fn new(
        max_action_affordances: usize,
        max_hypotheses: usize,
        max_candidate_frontier: usize,
    ) -> Option<Self> {
        if max_action_affordances == 0 || max_hypotheses == 0 || max_candidate_frontier == 0 {
            return None;
        }

        Some(Self {
            max_action_affordances,
            max_hypotheses,
            max_candidate_frontier,
        })
    }

    pub const fn max_action_affordances(self) -> usize {
        self.max_action_affordances
    }

    pub const fn max_hypotheses(self) -> usize {
        self.max_hypotheses
    }

    pub const fn max_candidate_frontier(self) -> usize {
        self.max_candidate_frontier
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelfBootstrapThresholds {
    min_confidence: BootstrapSignal,
    min_information_gain: BootstrapSignal,
    min_controllability: BootstrapSignal,
    max_execution_cost: BootstrapSignal,
}

impl SelfBootstrapThresholds {
    pub const fn new(
        min_confidence: BootstrapSignal,
        min_information_gain: BootstrapSignal,
        min_controllability: BootstrapSignal,
        max_execution_cost: BootstrapSignal,
    ) -> Self {
        Self {
            min_confidence,
            min_information_gain,
            min_controllability,
            max_execution_cost,
        }
    }

    pub const fn min_confidence(self) -> BootstrapSignal {
        self.min_confidence
    }

    pub const fn min_information_gain(self) -> BootstrapSignal {
        self.min_information_gain
    }

    pub const fn min_controllability(self) -> BootstrapSignal {
        self.min_controllability
    }

    pub const fn max_execution_cost(self) -> BootstrapSignal {
        self.max_execution_cost
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelfBootstrapPolicy {
    bounds: SelfBootstrapBounds,
    thresholds: SelfBootstrapThresholds,
}

impl SelfBootstrapPolicy {
    pub const fn new(bounds: SelfBootstrapBounds, thresholds: SelfBootstrapThresholds) -> Self {
        Self { bounds, thresholds }
    }

    pub const fn bounds(self) -> SelfBootstrapBounds {
        self.bounds
    }

    pub const fn thresholds(self) -> SelfBootstrapThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutcomeHypothesis {
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
    confidence: BootstrapSignal,
    information_gain: BootstrapSignal,
    controllability: BootstrapSignal,
    execution_cost: BootstrapSignal,
}

impl OutcomeHypothesis {
    pub fn new(
        source_state: CognitiveStructure,
        action: CognitiveStructure,
        predicted_outcome: CognitiveStructure,
        confidence: BootstrapSignal,
        information_gain: BootstrapSignal,
        controllability: BootstrapSignal,
        execution_cost: BootstrapSignal,
    ) -> Self {
        Self {
            source_state,
            action,
            predicted_outcome,
            confidence,
            information_gain,
            controllability,
            execution_cost,
        }
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        &self.predicted_outcome
    }

    pub const fn confidence(&self) -> BootstrapSignal {
        self.confidence
    }

    pub const fn information_gain(&self) -> BootstrapSignal {
        self.information_gain
    }

    pub const fn controllability(&self) -> BootstrapSignal {
        self.controllability
    }

    pub const fn execution_cost(&self) -> BootstrapSignal {
        self.execution_cost
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BootstrapFeedback {
    #[default]
    Unspecified,
    Progress(BootstrapSignal),
    Regression(BootstrapSignal),
    TerminalSuccess,
    TerminalFailure,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfBootstrapInput {
    current_state: CognitiveStructure,
    previous_state: Option<CognitiveStructure>,
    action_affordances: Vec<CognitiveStructure>,
    hypotheses: Vec<OutcomeHypothesis>,
    feedback: BootstrapFeedback,
}

impl SelfBootstrapInput {
    pub fn new(
        current_state: CognitiveStructure,
        previous_state: Option<CognitiveStructure>,
        action_affordances: Vec<CognitiveStructure>,
        hypotheses: Vec<OutcomeHypothesis>,
        feedback: BootstrapFeedback,
    ) -> Self {
        Self {
            current_state,
            previous_state,
            action_affordances,
            hypotheses,
            feedback,
        }
    }

    pub fn current_state(&self) -> &CognitiveStructure {
        &self.current_state
    }

    pub fn previous_state(&self) -> Option<&CognitiveStructure> {
        self.previous_state.as_ref()
    }

    pub fn action_affordances(&self) -> &[CognitiveStructure] {
        &self.action_affordances
    }

    pub fn hypotheses(&self) -> &[OutcomeHypothesis] {
        &self.hypotheses
    }

    pub const fn feedback(&self) -> BootstrapFeedback {
        self.feedback
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BootstrapObjectiveKind {
    Complete,
    ModelExpansion,
    HypothesisDiscrimination,
    ProgressContinuation,
    RecoveryExploration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapObjective {
    kind: BootstrapObjectiveKind,
    source_state: CognitiveStructure,
    target_state: Option<CognitiveStructure>,
}

impl BootstrapObjective {
    fn new(
        kind: BootstrapObjectiveKind,
        source_state: CognitiveStructure,
        target_state: Option<CognitiveStructure>,
    ) -> Self {
        Self {
            kind,
            source_state,
            target_state,
        }
    }

    pub const fn kind(&self) -> BootstrapObjectiveKind {
        self.kind
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn target_state(&self) -> Option<&CognitiveStructure> {
        self.target_state.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootstrapCandidate {
    input_index: usize,
    hypothesis: OutcomeHypothesis,
}

impl BootstrapCandidate {
    fn new(input_index: usize, hypothesis: OutcomeHypothesis) -> Self {
        Self {
            input_index,
            hypothesis,
        }
    }

    pub const fn input_index(&self) -> usize {
        self.input_index
    }

    pub fn hypothesis(&self) -> &OutcomeHypothesis {
        &self.hypothesis
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelfBootstrapStatus {
    Complete,
    Selected,
    ModelExpansionRequired,
    Blocked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfBootstrapResult {
    status: SelfBootstrapStatus,
    objective: BootstrapObjective,
    action_affordances: Vec<CognitiveStructure>,
    candidate_frontier: Vec<BootstrapCandidate>,
    selected: Option<BootstrapCandidate>,
    rejected_source_state_count: usize,
    rejected_unauthorized_action_count: usize,
    rejected_threshold_count: usize,
    duplicate_affordance_count: usize,
    duplicate_hypothesis_count: usize,
    frontier_truncated: bool,
}

impl SelfBootstrapResult {
    pub const fn status(&self) -> SelfBootstrapStatus {
        self.status
    }

    pub fn objective(&self) -> &BootstrapObjective {
        &self.objective
    }

    pub fn action_affordances(&self) -> &[CognitiveStructure] {
        &self.action_affordances
    }

    pub fn candidate_frontier(&self) -> &[BootstrapCandidate] {
        &self.candidate_frontier
    }

    pub fn selected(&self) -> Option<&BootstrapCandidate> {
        self.selected.as_ref()
    }

    pub const fn rejected_source_state_count(&self) -> usize {
        self.rejected_source_state_count
    }

    pub const fn rejected_unauthorized_action_count(&self) -> usize {
        self.rejected_unauthorized_action_count
    }

    pub const fn rejected_threshold_count(&self) -> usize {
        self.rejected_threshold_count
    }

    pub const fn duplicate_affordance_count(&self) -> usize {
        self.duplicate_affordance_count
    }

    pub const fn duplicate_hypothesis_count(&self) -> usize {
        self.duplicate_hypothesis_count
    }

    pub const fn frontier_truncated(&self) -> bool {
        self.frontier_truncated
    }

    pub const fn abstained(&self) -> bool {
        !matches!(self.status, SelfBootstrapStatus::Selected)
    }

    pub const fn requires_model_expansion(&self) -> bool {
        matches!(self.status, SelfBootstrapStatus::ModelExpansionRequired)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelfBootstrapError {
    ActionAffordanceFrontierExceeded,
    HypothesisFrontierExceeded,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousCognitiveSelfBootstrapFoundation;

impl AutonomousCognitiveSelfBootstrapFoundation {
    fn deduplicate_affordances(
        affordances: &[CognitiveStructure],
    ) -> (Vec<CognitiveStructure>, usize) {
        let mut canonical = Vec::with_capacity(affordances.len());

        let mut duplicate_count = 0usize;

        for affordance in affordances {
            if canonical.contains(affordance) {
                duplicate_count += 1;
                continue;
            }

            canonical.push(affordance.clone());
        }

        (canonical, duplicate_count)
    }

    fn candidate_same_semantics(left: &BootstrapCandidate, right: &BootstrapCandidate) -> bool {
        left.hypothesis() == right.hypothesis()
    }

    fn candidate_order(left: &BootstrapCandidate, right: &BootstrapCandidate) -> Ordering {
        right
            .hypothesis()
            .information_gain()
            .cmp(&left.hypothesis().information_gain())
            .then_with(|| {
                right
                    .hypothesis()
                    .controllability()
                    .cmp(&left.hypothesis().controllability())
            })
            .then_with(|| {
                right
                    .hypothesis()
                    .confidence()
                    .cmp(&left.hypothesis().confidence())
            })
            .then_with(|| {
                left.hypothesis()
                    .execution_cost()
                    .cmp(&right.hypothesis().execution_cost())
            })
            .then_with(|| left.input_index().cmp(&right.input_index()))
    }

    fn objective_kind(feedback: BootstrapFeedback) -> BootstrapObjectiveKind {
        match feedback {
            BootstrapFeedback::Progress(_) => BootstrapObjectiveKind::ProgressContinuation,
            BootstrapFeedback::Regression(_) | BootstrapFeedback::TerminalFailure => {
                BootstrapObjectiveKind::RecoveryExploration
            }
            BootstrapFeedback::Unspecified => BootstrapObjectiveKind::HypothesisDiscrimination,
            BootstrapFeedback::TerminalSuccess => BootstrapObjectiveKind::Complete,
        }
    }

    fn meets_thresholds(
        hypothesis: &OutcomeHypothesis,
        thresholds: SelfBootstrapThresholds,
    ) -> bool {
        hypothesis.confidence() >= thresholds.min_confidence()
            && hypothesis.information_gain() >= thresholds.min_information_gain()
            && hypothesis.controllability() >= thresholds.min_controllability()
            && hypothesis.execution_cost() <= thresholds.max_execution_cost()
    }

    pub fn evaluate(
        input: &SelfBootstrapInput,
        policy: SelfBootstrapPolicy,
    ) -> Result<SelfBootstrapResult, SelfBootstrapError> {
        if input.action_affordances().len() > policy.bounds().max_action_affordances() {
            return Err(SelfBootstrapError::ActionAffordanceFrontierExceeded);
        }

        if input.hypotheses().len() > policy.bounds().max_hypotheses() {
            return Err(SelfBootstrapError::HypothesisFrontierExceeded);
        }

        let (action_affordances, duplicate_affordance_count) =
            Self::deduplicate_affordances(input.action_affordances());

        if matches!(input.feedback(), BootstrapFeedback::TerminalSuccess) {
            return Ok(SelfBootstrapResult {
                status: SelfBootstrapStatus::Complete,
                objective: BootstrapObjective::new(
                    BootstrapObjectiveKind::Complete,
                    input.current_state().clone(),
                    Some(input.current_state().clone()),
                ),
                action_affordances,
                candidate_frontier: Vec::new(),
                selected: None,
                rejected_source_state_count: 0,
                rejected_unauthorized_action_count: 0,
                rejected_threshold_count: 0,
                duplicate_affordance_count,
                duplicate_hypothesis_count: 0,
                frontier_truncated: false,
            });
        }

        let mut rejected_source_state_count = 0usize;
        let mut rejected_unauthorized_action_count = 0usize;
        let mut rejected_threshold_count = 0usize;
        let mut duplicate_hypothesis_count = 0usize;

        let mut frontier = Vec::new();

        for (input_index, hypothesis) in input.hypotheses().iter().enumerate() {
            if hypothesis.source_state() != input.current_state() {
                rejected_source_state_count += 1;
                continue;
            }

            if !action_affordances.contains(hypothesis.action()) {
                rejected_unauthorized_action_count += 1;
                continue;
            }

            if !Self::meets_thresholds(hypothesis, policy.thresholds()) {
                rejected_threshold_count += 1;
                continue;
            }

            let candidate = BootstrapCandidate::new(input_index, hypothesis.clone());

            if frontier
                .iter()
                .any(|existing| Self::candidate_same_semantics(existing, &candidate))
            {
                duplicate_hypothesis_count += 1;
                continue;
            }

            frontier.push(candidate);
        }

        frontier.sort_by(Self::candidate_order);

        let frontier_truncated = frontier.len() > policy.bounds().max_candidate_frontier();

        if frontier_truncated {
            frontier.truncate(policy.bounds().max_candidate_frontier());
        }

        let selected = frontier.first().cloned();

        if let Some(selected_candidate) = selected {
            let objective_kind = Self::objective_kind(input.feedback());

            let objective = BootstrapObjective::new(
                objective_kind,
                input.current_state().clone(),
                Some(selected_candidate.hypothesis().predicted_outcome().clone()),
            );

            return Ok(SelfBootstrapResult {
                status: SelfBootstrapStatus::Selected,
                objective,
                action_affordances,
                candidate_frontier: frontier,
                selected: Some(selected_candidate),
                rejected_source_state_count,
                rejected_unauthorized_action_count,
                rejected_threshold_count,
                duplicate_affordance_count,
                duplicate_hypothesis_count,
                frontier_truncated,
            });
        }

        let status = if action_affordances.is_empty() {
            SelfBootstrapStatus::Blocked
        } else {
            SelfBootstrapStatus::ModelExpansionRequired
        };

        Ok(SelfBootstrapResult {
            status,
            objective: BootstrapObjective::new(
                BootstrapObjectiveKind::ModelExpansion,
                input.current_state().clone(),
                None,
            ),
            action_affordances,
            candidate_frontier: frontier,
            selected: None,
            rejected_source_state_count,
            rejected_unauthorized_action_count,
            rejected_threshold_count,
            duplicate_affordance_count,
            duplicate_hypothesis_count,
            frontier_truncated,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousCognitiveSelfBootstrap;

impl UniversalAutonomousCognitiveSelfBootstrap {
    pub fn evaluate(
        input: &SelfBootstrapInput,
        policy: SelfBootstrapPolicy,
    ) -> Result<SelfBootstrapResult, SelfBootstrapError> {
        AutonomousCognitiveSelfBootstrapFoundation::evaluate(input, policy)
    }
}
