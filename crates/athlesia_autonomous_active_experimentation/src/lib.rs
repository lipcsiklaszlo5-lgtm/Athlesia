use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExperimentEvidence {
    prediction_uncertainty: CognitiveSignal,
    expected_information_gain: CognitiveSignal,
    controllability: CognitiveSignal,
    grounding_confidence: CognitiveSignal,
    execution_cost: CognitiveSignal,
}

impl ExperimentEvidence {
    pub fn new(
        prediction_uncertainty: CognitiveSignal,
        expected_information_gain: CognitiveSignal,
        controllability: CognitiveSignal,
        grounding_confidence: CognitiveSignal,
        execution_cost: CognitiveSignal,
    ) -> Option<Self> {
        if prediction_uncertainty == CognitiveSignal::zero()
            || expected_information_gain == CognitiveSignal::zero()
            || controllability == CognitiveSignal::zero()
            || grounding_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            prediction_uncertainty,
            expected_information_gain,
            controllability,
            grounding_confidence,
            execution_cost,
        })
    }

    pub fn prediction_uncertainty(self) -> CognitiveSignal {
        self.prediction_uncertainty
    }

    pub fn expected_information_gain(self) -> CognitiveSignal {
        self.expected_information_gain
    }

    pub fn controllability(self) -> CognitiveSignal {
        self.controllability
    }

    pub fn grounding_confidence(self) -> CognitiveSignal {
        self.grounding_confidence
    }

    pub fn execution_cost(self) -> CognitiveSignal {
        self.execution_cost
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutonomousExperimentProposal {
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
    evidence: ExperimentEvidence,
}

impl AutonomousExperimentProposal {
    pub fn new(
        source_state: CognitiveStructure,
        action: CognitiveStructure,
        predicted_outcome: CognitiveStructure,
        evidence: ExperimentEvidence,
    ) -> Self {
        Self {
            source_state,
            action,
            predicted_outcome,
            evidence,
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

    pub fn evidence(&self) -> ExperimentEvidence {
        self.evidence
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveExperimentBounds {
    max_input_proposals: usize,
    max_evaluations: usize,
    max_selected_experiments: usize,
}

impl ActiveExperimentBounds {
    pub fn new(
        max_input_proposals: usize,
        max_evaluations: usize,
        max_selected_experiments: usize,
    ) -> Option<Self> {
        if max_input_proposals == 0 || max_evaluations == 0 || max_selected_experiments == 0 {
            return None;
        }

        Some(Self {
            max_input_proposals,
            max_evaluations,
            max_selected_experiments,
        })
    }

    pub fn max_input_proposals(self) -> usize {
        self.max_input_proposals
    }

    pub fn max_evaluations(self) -> usize {
        self.max_evaluations
    }

    pub fn max_selected_experiments(self) -> usize {
        self.max_selected_experiments
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveExperimentThresholds {
    minimum_information_gain: CognitiveSignal,
    minimum_controllability: CognitiveSignal,
    minimum_grounding_confidence: CognitiveSignal,
    maximum_execution_cost: CognitiveSignal,
}

impl ActiveExperimentThresholds {
    pub fn new(
        minimum_information_gain: CognitiveSignal,
        minimum_controllability: CognitiveSignal,
        minimum_grounding_confidence: CognitiveSignal,
        maximum_execution_cost: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_information_gain == CognitiveSignal::zero()
            || minimum_controllability == CognitiveSignal::zero()
            || minimum_grounding_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_information_gain,
            minimum_controllability,
            minimum_grounding_confidence,
            maximum_execution_cost,
        })
    }

    pub fn minimum_information_gain(self) -> CognitiveSignal {
        self.minimum_information_gain
    }

    pub fn minimum_controllability(self) -> CognitiveSignal {
        self.minimum_controllability
    }

    pub fn minimum_grounding_confidence(self) -> CognitiveSignal {
        self.minimum_grounding_confidence
    }

    pub fn maximum_execution_cost(self) -> CognitiveSignal {
        self.maximum_execution_cost
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveExperimentPolicy {
    bounds: ActiveExperimentBounds,
    thresholds: ActiveExperimentThresholds,
}

impl ActiveExperimentPolicy {
    pub fn new(bounds: ActiveExperimentBounds, thresholds: ActiveExperimentThresholds) -> Self {
        Self { bounds, thresholds }
    }

    pub fn bounds(self) -> ActiveExperimentBounds {
        self.bounds
    }

    pub fn thresholds(self) -> ActiveExperimentThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedActiveExperiment {
    proposal: AutonomousExperimentProposal,
}

impl SelectedActiveExperiment {
    pub fn proposal(&self) -> &AutonomousExperimentProposal {
        &self.proposal
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveExperimentSelectionResult {
    input_proposal_count: usize,
    unique_proposal_count: usize,
    considered_proposal_count: usize,
    input_frontier_truncated: bool,
    evaluation_count: usize,
    evaluation_frontier_truncated: bool,
    rejected_information_gain_count: usize,
    rejected_controllability_count: usize,
    rejected_grounding_count: usize,
    rejected_cost_count: usize,
    selected_before_frontier: usize,
    selection_frontier_truncated: bool,
    selected: Vec<SelectedActiveExperiment>,
}

impl ActiveExperimentSelectionResult {
    pub fn input_proposal_count(&self) -> usize {
        self.input_proposal_count
    }

    pub fn unique_proposal_count(&self) -> usize {
        self.unique_proposal_count
    }

    pub fn considered_proposal_count(&self) -> usize {
        self.considered_proposal_count
    }

    pub fn input_frontier_truncated(&self) -> bool {
        self.input_frontier_truncated
    }

    pub fn evaluation_count(&self) -> usize {
        self.evaluation_count
    }

    pub fn evaluation_frontier_truncated(&self) -> bool {
        self.evaluation_frontier_truncated
    }

    pub fn rejected_information_gain_count(&self) -> usize {
        self.rejected_information_gain_count
    }

    pub fn rejected_controllability_count(&self) -> usize {
        self.rejected_controllability_count
    }

    pub fn rejected_grounding_count(&self) -> usize {
        self.rejected_grounding_count
    }

    pub fn rejected_cost_count(&self) -> usize {
        self.rejected_cost_count
    }

    pub fn selected_before_frontier(&self) -> usize {
        self.selected_before_frontier
    }

    pub fn selection_frontier_truncated(&self) -> bool {
        self.selection_frontier_truncated
    }

    pub fn selected(&self) -> &[SelectedActiveExperiment] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn abstained(&self) -> bool {
        self.selected.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousActiveExperimentationFoundation;

impl AutonomousActiveExperimentationFoundation {
    fn same_semantics(
        left: &AutonomousExperimentProposal,
        right: &AutonomousExperimentProposal,
    ) -> bool {
        left.source_state() == right.source_state()
            && left.action() == right.action()
            && left.predicted_outcome() == right.predicted_outcome()
    }

    fn proposal_order(
        left: &AutonomousExperimentProposal,
        right: &AutonomousExperimentProposal,
    ) -> std::cmp::Ordering {
        let le = left.evidence();
        let re = right.evidence();

        re.expected_information_gain()
            .value()
            .cmp(&le.expected_information_gain().value())
            .then_with(|| {
                re.prediction_uncertainty()
                    .value()
                    .cmp(&le.prediction_uncertainty().value())
            })
            .then_with(|| {
                re.controllability()
                    .value()
                    .cmp(&le.controllability().value())
            })
            .then_with(|| {
                re.grounding_confidence()
                    .value()
                    .cmp(&le.grounding_confidence().value())
            })
            .then_with(|| {
                le.execution_cost()
                    .value()
                    .cmp(&re.execution_cost().value())
            })
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    pub fn select(
        proposals: &[AutonomousExperimentProposal],
        policy: ActiveExperimentPolicy,
    ) -> ActiveExperimentSelectionResult {
        let bounds = policy.bounds();
        let thresholds = policy.thresholds();

        let input_proposal_count = proposals.len();

        let mut ranked = proposals.to_vec();
        ranked.sort_by(Self::proposal_order);

        let mut unique = Vec::new();

        for proposal in ranked {
            if !unique
                .iter()
                .any(|existing| Self::same_semantics(existing, &proposal))
            {
                unique.push(proposal);
            }
        }

        let unique_proposal_count = unique.len();

        unique.truncate(bounds.max_input_proposals());

        let considered_proposal_count = unique.len();

        let mut evaluation_count = 0;
        let mut evaluation_frontier_truncated = false;

        let mut rejected_information_gain_count = 0;
        let mut rejected_controllability_count = 0;
        let mut rejected_grounding_count = 0;
        let mut rejected_cost_count = 0;

        let mut selected = Vec::new();

        for proposal in unique {
            if evaluation_count >= bounds.max_evaluations() {
                evaluation_frontier_truncated = true;
                break;
            }

            evaluation_count += 1;

            let evidence = proposal.evidence();

            if evidence.expected_information_gain().value()
                < thresholds.minimum_information_gain().value()
            {
                rejected_information_gain_count += 1;
                continue;
            }

            if evidence.controllability().value() < thresholds.minimum_controllability().value() {
                rejected_controllability_count += 1;
                continue;
            }

            if evidence.grounding_confidence().value()
                < thresholds.minimum_grounding_confidence().value()
            {
                rejected_grounding_count += 1;
                continue;
            }

            if evidence.execution_cost().value() > thresholds.maximum_execution_cost().value() {
                rejected_cost_count += 1;
                continue;
            }

            selected.push(SelectedActiveExperiment { proposal });
        }

        selected.sort_by(|left, right| Self::proposal_order(left.proposal(), right.proposal()));

        let selected_before_frontier = selected.len();

        selected.truncate(bounds.max_selected_experiments());

        ActiveExperimentSelectionResult {
            input_proposal_count,
            unique_proposal_count,
            considered_proposal_count,
            input_frontier_truncated: unique_proposal_count > considered_proposal_count,
            evaluation_count,
            evaluation_frontier_truncated,
            rejected_information_gain_count,
            rejected_controllability_count,
            rejected_grounding_count,
            rejected_cost_count,
            selected_before_frontier,
            selection_frontier_truncated: selected_before_frontier > selected.len(),
            selected,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousActiveExperimentationFoundation;

impl UniversalAutonomousActiveExperimentationFoundation {
    pub fn evaluate(
        proposals: &[AutonomousExperimentProposal],
        policy: ActiveExperimentPolicy,
    ) -> ActiveExperimentSelectionResult {
        AutonomousActiveExperimentationFoundation::select(proposals, policy)
    }
}
