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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompetingHypothesisPrediction {
    hypothesis: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
    confidence: CognitiveSignal,
}

impl CompetingHypothesisPrediction {
    pub fn new(
        hypothesis: CognitiveStructure,
        predicted_outcome: CognitiveStructure,
        confidence: CognitiveSignal,
    ) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            hypothesis,
            predicted_outcome,
            confidence,
        })
    }

    pub fn hypothesis(&self) -> &CognitiveStructure {
        &self.hypothesis
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        &self.predicted_outcome
    }

    pub fn confidence(&self) -> CognitiveSignal {
        self.confidence
    }
}

// ============================================================================
// P4G-C3B-A — EVIDENCE-GROUNDED EPISTEMIC FORECAST REPRESENTATION
// ============================================================================
//
// This representation preserves epistemic abstention without manufacturing
// an outcome, confidence, information gain, learning progress, utility,
// controllability or execution authority.
//
// Historical evidence remains explicit as support/opportunity/counterexample
// counts. State-conditioned forecast status remains a separate fact.
//
// In particular:
//
//   Predicted          -> carries a concrete partial predicted outcome.
//   ContextAbstained   -> carries NO fabricated outcome.
//   NoEffectOpportunity-> carries NO fabricated outcome.
//
// Only Predicted vs ContextAbstained on the SAME target contributes to the
// factorized separation score. NoEffectOpportunity does not.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EpistemicHypothesisForecastStatus {
    Predicted,
    ContextAbstained,
    NoEffectOpportunity,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EpistemicForecastEvidence {
    support_count: u64,
    opportunity_count: u64,
    counterexample_count: u64,
}

impl EpistemicForecastEvidence {
    pub fn new(
        support_count: u64,
        opportunity_count: u64,
        counterexample_count: u64,
    ) -> Option<Self> {
        if support_count == 0 || opportunity_count == 0 {
            return None;
        }

        if support_count.checked_add(counterexample_count)? != opportunity_count {
            return None;
        }

        Some(Self {
            support_count,
            opportunity_count,
            counterexample_count,
        })
    }

    pub fn support_count(self) -> u64 {
        self.support_count
    }

    pub fn opportunity_count(self) -> u64 {
        self.opportunity_count
    }

    pub fn counterexample_count(self) -> u64 {
        self.counterexample_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicHypothesisForecast {
    hypothesis: CognitiveStructure,
    target: CognitiveStructure,
    predicted_outcome: Option<CognitiveStructure>,
    evidence: EpistemicForecastEvidence,
    status: EpistemicHypothesisForecastStatus,
}

impl EpistemicHypothesisForecast {
    fn new(
        hypothesis: CognitiveStructure,
        target: CognitiveStructure,
        predicted_outcome: Option<CognitiveStructure>,
        evidence: EpistemicForecastEvidence,
        status: EpistemicHypothesisForecastStatus,
    ) -> Option<Self> {
        let outcome_contract_holds = matches!(
            (status, predicted_outcome.is_some()),
            (EpistemicHypothesisForecastStatus::Predicted, true)
                | (EpistemicHypothesisForecastStatus::ContextAbstained, false)
                | (
                    EpistemicHypothesisForecastStatus::NoEffectOpportunity,
                    false
                )
        );

        if !outcome_contract_holds {
            return None;
        }

        Some(Self {
            hypothesis,
            target,
            predicted_outcome,
            evidence,
            status,
        })
    }

    pub fn predicted(
        hypothesis: CognitiveStructure,
        target: CognitiveStructure,
        predicted_outcome: CognitiveStructure,
        evidence: EpistemicForecastEvidence,
    ) -> Option<Self> {
        Self::new(
            hypothesis,
            target,
            Some(predicted_outcome),
            evidence,
            EpistemicHypothesisForecastStatus::Predicted,
        )
    }

    pub fn context_abstained(
        hypothesis: CognitiveStructure,
        target: CognitiveStructure,
        evidence: EpistemicForecastEvidence,
    ) -> Option<Self> {
        Self::new(
            hypothesis,
            target,
            None,
            evidence,
            EpistemicHypothesisForecastStatus::ContextAbstained,
        )
    }

    pub fn no_effect_opportunity(
        hypothesis: CognitiveStructure,
        target: CognitiveStructure,
        evidence: EpistemicForecastEvidence,
    ) -> Option<Self> {
        Self::new(
            hypothesis,
            target,
            None,
            evidence,
            EpistemicHypothesisForecastStatus::NoEffectOpportunity,
        )
    }

    pub fn hypothesis(&self) -> &CognitiveStructure {
        &self.hypothesis
    }

    pub fn target(&self) -> &CognitiveStructure {
        &self.target
    }

    pub fn predicted_outcome(&self) -> Option<&CognitiveStructure> {
        self.predicted_outcome.as_ref()
    }

    pub fn evidence(&self) -> EpistemicForecastEvidence {
        self.evidence
    }

    pub fn status(&self) -> EpistemicHypothesisForecastStatus {
        self.status
    }

    pub fn is_predicted(&self) -> bool {
        self.status == EpistemicHypothesisForecastStatus::Predicted
    }

    pub fn is_context_abstained(&self) -> bool {
        self.status == EpistemicHypothesisForecastStatus::ContextAbstained
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedEpistemicExperimentPossibility {
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    forecasts: Vec<EpistemicHypothesisForecast>,
}

impl GroundedEpistemicExperimentPossibility {
    pub fn new(
        source_state: CognitiveStructure,
        action: CognitiveStructure,
        forecasts: Vec<EpistemicHypothesisForecast>,
    ) -> Option<Self> {
        if forecasts.is_empty() {
            return None;
        }

        Some(Self {
            source_state,
            action,
            forecasts,
        })
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn forecasts(&self) -> &[EpistemicHypothesisForecast] {
        &self.forecasts
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EpistemicForecastDiscriminationPolicy {
    max_forecasts: usize,
    max_targets: usize,
}

impl EpistemicForecastDiscriminationPolicy {
    pub fn new(max_forecasts: usize, max_targets: usize) -> Option<Self> {
        if max_forecasts == 0 || max_targets == 0 {
            return None;
        }

        Some(Self {
            max_forecasts,
            max_targets,
        })
    }

    pub fn max_forecasts(self) -> usize {
        self.max_forecasts
    }

    pub fn max_targets(self) -> usize {
        self.max_targets
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicTargetDisagreement {
    target: CognitiveStructure,
    predicted_count: usize,
    context_abstention_count: usize,
    no_effect_opportunity_count: usize,
}

impl EpistemicTargetDisagreement {
    pub fn target(&self) -> &CognitiveStructure {
        &self.target
    }

    pub fn predicted_count(&self) -> usize {
        self.predicted_count
    }

    pub fn context_abstention_count(&self) -> usize {
        self.context_abstention_count
    }

    pub fn no_effect_opportunity_count(&self) -> usize {
        self.no_effect_opportunity_count
    }

    pub fn pairwise_separation_score(&self) -> usize {
        self.predicted_count
            .saturating_mul(self.context_abstention_count)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedEpistemicExperimentDiscrimination {
    action: CognitiveStructure,
    input_forecast_count: usize,
    unique_forecast_count: usize,
    forecast_frontier_truncated: bool,
    target_frontier_truncated: bool,
    disagreements: Vec<EpistemicTargetDisagreement>,
    pairwise_separation_score: usize,
}

impl GroundedEpistemicExperimentDiscrimination {
    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn input_forecast_count(&self) -> usize {
        self.input_forecast_count
    }

    pub fn unique_forecast_count(&self) -> usize {
        self.unique_forecast_count
    }

    pub fn forecast_frontier_truncated(&self) -> bool {
        self.forecast_frontier_truncated
    }

    pub fn target_frontier_truncated(&self) -> bool {
        self.target_frontier_truncated
    }

    pub fn disagreements(&self) -> &[EpistemicTargetDisagreement] {
        &self.disagreements
    }

    pub fn pairwise_separation_score(&self) -> usize {
        self.pairwise_separation_score
    }

    pub fn informative(&self) -> bool {
        !self.forecast_frontier_truncated
            && !self.target_frontier_truncated
            && self.pairwise_separation_score > 0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousEpistemicForecastDiscrimination;

impl AutonomousEpistemicForecastDiscrimination {
    fn forecast_order(
        left: &EpistemicHypothesisForecast,
        right: &EpistemicHypothesisForecast,
    ) -> std::cmp::Ordering {
        format!("{:?}", left.target())
            .cmp(&format!("{:?}", right.target()))
            .then_with(|| {
                format!("{:?}", left.hypothesis()).cmp(&format!("{:?}", right.hypothesis()))
            })
            .then_with(|| left.status().cmp(&right.status()))
            .then_with(|| {
                format!("{:?}", left.predicted_outcome())
                    .cmp(&format!("{:?}", right.predicted_outcome()))
            })
            .then_with(|| {
                left.evidence()
                    .support_count()
                    .cmp(&right.evidence().support_count())
            })
            .then_with(|| {
                left.evidence()
                    .opportunity_count()
                    .cmp(&right.evidence().opportunity_count())
            })
            .then_with(|| {
                left.evidence()
                    .counterexample_count()
                    .cmp(&right.evidence().counterexample_count())
            })
    }

    pub fn evaluate(
        possibility: &GroundedEpistemicExperimentPossibility,
        policy: EpistemicForecastDiscriminationPolicy,
    ) -> GroundedEpistemicExperimentDiscrimination {
        let input_forecast_count = possibility.forecasts().len();

        let mut forecasts = possibility.forecasts().to_vec();

        forecasts.sort_by(Self::forecast_order);
        forecasts.dedup();

        let unique_forecast_count = forecasts.len();

        if unique_forecast_count > policy.max_forecasts() {
            return GroundedEpistemicExperimentDiscrimination {
                action: possibility.action().clone(),
                input_forecast_count,
                unique_forecast_count,
                forecast_frontier_truncated: true,
                target_frontier_truncated: false,
                disagreements: Vec::new(),
                pairwise_separation_score: 0,
            };
        }

        let mut targets = forecasts
            .iter()
            .map(|forecast| forecast.target().clone())
            .collect::<Vec<_>>();

        targets.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));

        targets.dedup();

        if targets.len() > policy.max_targets() {
            return GroundedEpistemicExperimentDiscrimination {
                action: possibility.action().clone(),
                input_forecast_count,
                unique_forecast_count,
                forecast_frontier_truncated: false,
                target_frontier_truncated: true,
                disagreements: Vec::new(),
                pairwise_separation_score: 0,
            };
        }

        let mut disagreements = Vec::with_capacity(targets.len());

        for target in targets {
            let relevant = forecasts
                .iter()
                .filter(|forecast| forecast.target() == &target);

            let mut predicted_count = 0_usize;
            let mut context_abstention_count = 0_usize;
            let mut no_effect_opportunity_count = 0_usize;

            for forecast in relevant {
                match forecast.status() {
                    EpistemicHypothesisForecastStatus::Predicted => {
                        predicted_count = predicted_count.saturating_add(1);
                    }

                    EpistemicHypothesisForecastStatus::ContextAbstained => {
                        context_abstention_count = context_abstention_count.saturating_add(1);
                    }

                    EpistemicHypothesisForecastStatus::NoEffectOpportunity => {
                        no_effect_opportunity_count = no_effect_opportunity_count.saturating_add(1);
                    }
                }
            }

            disagreements.push(EpistemicTargetDisagreement {
                target,
                predicted_count,
                context_abstention_count,
                no_effect_opportunity_count,
            });
        }

        let pairwise_separation_score = disagreements
            .iter()
            .map(EpistemicTargetDisagreement::pairwise_separation_score)
            .fold(0_usize, usize::saturating_add);

        GroundedEpistemicExperimentDiscrimination {
            action: possibility.action().clone(),
            input_forecast_count,
            unique_forecast_count,
            forecast_frontier_truncated: false,
            target_frontier_truncated: false,
            disagreements,
            pairwise_separation_score,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousEpistemicForecastDiscrimination;

impl UniversalAutonomousEpistemicForecastDiscrimination {
    pub fn evaluate(
        possibility: &GroundedEpistemicExperimentPossibility,
        policy: EpistemicForecastDiscriminationPolicy,
    ) -> GroundedEpistemicExperimentDiscrimination {
        AutonomousEpistemicForecastDiscrimination::evaluate(possibility, policy)
    }
}

// ============================================================================
// P4G-C3C-A — REALIZED EPISTEMIC RESOLUTION EVIDENCE
// ============================================================================
//
// C3B established a pre-action epistemic forecast vocabulary without
// fabricating confidence or outcomes for abstaining hypotheses.
//
// C3C-A adds the complementary POST-action vocabulary.
//
// This layer does NOT estimate expected information gain.  It records only
// what an explicit observed target consequence actually establishes:
//
//   Predicted + occurred      -> Supported
//   Predicted + did not occur -> Counterexample
//   ContextAbstained          -> ContextUninformative
//   NoEffectOpportunity       -> NoOpportunityUninformative
//
// An abstaining forecast can never be silently converted into a negative
// prediction.  Missing target observations fail closed rather than turning
// absence of evidence into evidence of absence.

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicTargetObservation {
    target: CognitiveStructure,
    occurred: bool,
}

impl EpistemicTargetObservation {
    pub fn new(target: CognitiveStructure, occurred: bool) -> Self {
        Self { target, occurred }
    }

    pub fn target(&self) -> &CognitiveStructure {
        &self.target
    }

    pub fn occurred(&self) -> bool {
        self.occurred
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedEpistemicOutcomeObservation {
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    targets: Vec<EpistemicTargetObservation>,
}

impl GroundedEpistemicOutcomeObservation {
    pub fn new(
        source_state: CognitiveStructure,
        action: CognitiveStructure,
        mut targets: Vec<EpistemicTargetObservation>,
    ) -> Option<Self> {
        if targets.is_empty() {
            return None;
        }

        targets.sort_by(|left, right| {
            format!("{:?}", left.target())
                .cmp(&format!("{:?}", right.target()))
                .then_with(|| left.occurred().cmp(&right.occurred()))
        });

        let mut canonical: Vec<EpistemicTargetObservation> = Vec::new();

        for observation in targets {
            if let Some(existing) = canonical
                .iter()
                .find(|existing| existing.target() == observation.target())
            {
                if existing.occurred() != observation.occurred() {
                    return None;
                }

                continue;
            }

            canonical.push(observation);
        }

        Some(Self {
            source_state,
            action,
            targets: canonical,
        })
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn targets(&self) -> &[EpistemicTargetObservation] {
        &self.targets
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EpistemicForecastOutcomeAssessmentStatus {
    Supported,
    Counterexample,
    ContextUninformative,
    NoOpportunityUninformative,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicForecastOutcomeAssessment {
    forecast: EpistemicHypothesisForecast,
    observed_target_occurrence: bool,
    status: EpistemicForecastOutcomeAssessmentStatus,
}

impl EpistemicForecastOutcomeAssessment {
    pub fn forecast(&self) -> &EpistemicHypothesisForecast {
        &self.forecast
    }

    pub fn observed_target_occurrence(&self) -> bool {
        self.observed_target_occurrence
    }

    pub fn status(&self) -> EpistemicForecastOutcomeAssessmentStatus {
        self.status
    }

    pub fn falsified(&self) -> bool {
        self.status == EpistemicForecastOutcomeAssessmentStatus::Counterexample
    }

    pub fn empirically_tested(&self) -> bool {
        matches!(
            self.status,
            EpistemicForecastOutcomeAssessmentStatus::Supported
                | EpistemicForecastOutcomeAssessmentStatus::Counterexample
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EpistemicOutcomeResolutionPolicy {
    max_forecasts: usize,
    max_target_observations: usize,
}

impl EpistemicOutcomeResolutionPolicy {
    pub fn new(max_forecasts: usize, max_target_observations: usize) -> Option<Self> {
        if max_forecasts == 0 || max_target_observations == 0 {
            return None;
        }

        Some(Self {
            max_forecasts,
            max_target_observations,
        })
    }

    pub fn max_forecasts(self) -> usize {
        self.max_forecasts
    }

    pub fn max_target_observations(self) -> usize {
        self.max_target_observations
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EpistemicOutcomeResolutionStatus {
    Resolved,
    SourceStateMismatch,
    ActionMismatch,
    ForecastFrontierExceeded,
    ObservationFrontierExceeded,
    ConflictingForecastIdentity,
    MissingTargetObservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicOutcomeResolutionResult {
    status: EpistemicOutcomeResolutionStatus,
    input_forecast_count: usize,
    unique_forecast_count: usize,
    target_observation_count: usize,
    assessments: Vec<EpistemicForecastOutcomeAssessment>,
    supported_prediction_count: usize,
    counterexample_prediction_count: usize,
    context_uninformative_count: usize,
    no_opportunity_uninformative_count: usize,
}

impl EpistemicOutcomeResolutionResult {
    fn rejected(
        status: EpistemicOutcomeResolutionStatus,
        input_forecast_count: usize,
        unique_forecast_count: usize,
        target_observation_count: usize,
    ) -> Self {
        Self {
            status,
            input_forecast_count,
            unique_forecast_count,
            target_observation_count,
            assessments: Vec::new(),
            supported_prediction_count: 0,
            counterexample_prediction_count: 0,
            context_uninformative_count: 0,
            no_opportunity_uninformative_count: 0,
        }
    }

    pub fn status(&self) -> EpistemicOutcomeResolutionStatus {
        self.status
    }

    pub fn resolved(&self) -> bool {
        self.status == EpistemicOutcomeResolutionStatus::Resolved
    }

    pub fn input_forecast_count(&self) -> usize {
        self.input_forecast_count
    }

    pub fn unique_forecast_count(&self) -> usize {
        self.unique_forecast_count
    }

    pub fn target_observation_count(&self) -> usize {
        self.target_observation_count
    }

    pub fn assessments(&self) -> &[EpistemicForecastOutcomeAssessment] {
        &self.assessments
    }

    pub fn supported_prediction_count(&self) -> usize {
        self.supported_prediction_count
    }

    pub fn counterexample_prediction_count(&self) -> usize {
        self.counterexample_prediction_count
    }

    pub fn empirically_tested_prediction_count(&self) -> usize {
        self.supported_prediction_count
            .saturating_add(self.counterexample_prediction_count)
    }

    pub fn context_uninformative_count(&self) -> usize {
        self.context_uninformative_count
    }

    pub fn no_opportunity_uninformative_count(&self) -> usize {
        self.no_opportunity_uninformative_count
    }

    pub fn falsified_hypothesis_count(&self) -> usize {
        self.counterexample_prediction_count
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousEpistemicOutcomeResolution;

impl AutonomousEpistemicOutcomeResolution {
    fn forecast_order(
        left: &EpistemicHypothesisForecast,
        right: &EpistemicHypothesisForecast,
    ) -> std::cmp::Ordering {
        format!("{:?}", left.hypothesis())
            .cmp(&format!("{:?}", right.hypothesis()))
            .then_with(|| format!("{:?}", left.target()).cmp(&format!("{:?}", right.target())))
            .then_with(|| left.status().cmp(&right.status()))
            .then_with(|| {
                format!("{:?}", left.predicted_outcome())
                    .cmp(&format!("{:?}", right.predicted_outcome()))
            })
            .then_with(|| {
                left.evidence()
                    .support_count()
                    .cmp(&right.evidence().support_count())
            })
            .then_with(|| {
                left.evidence()
                    .opportunity_count()
                    .cmp(&right.evidence().opportunity_count())
            })
            .then_with(|| {
                left.evidence()
                    .counterexample_count()
                    .cmp(&right.evidence().counterexample_count())
            })
    }

    pub fn evaluate(
        possibility: &GroundedEpistemicExperimentPossibility,
        observation: &GroundedEpistemicOutcomeObservation,
        policy: EpistemicOutcomeResolutionPolicy,
    ) -> EpistemicOutcomeResolutionResult {
        let input_forecast_count = possibility.forecasts().len();

        let target_observation_count = observation.targets().len();

        if possibility.source_state() != observation.source_state() {
            return EpistemicOutcomeResolutionResult::rejected(
                EpistemicOutcomeResolutionStatus::SourceStateMismatch,
                input_forecast_count,
                0,
                target_observation_count,
            );
        }

        if possibility.action() != observation.action() {
            return EpistemicOutcomeResolutionResult::rejected(
                EpistemicOutcomeResolutionStatus::ActionMismatch,
                input_forecast_count,
                0,
                target_observation_count,
            );
        }

        if input_forecast_count > policy.max_forecasts() {
            return EpistemicOutcomeResolutionResult::rejected(
                EpistemicOutcomeResolutionStatus::ForecastFrontierExceeded,
                input_forecast_count,
                0,
                target_observation_count,
            );
        }

        if target_observation_count > policy.max_target_observations() {
            return EpistemicOutcomeResolutionResult::rejected(
                EpistemicOutcomeResolutionStatus::ObservationFrontierExceeded,
                input_forecast_count,
                0,
                target_observation_count,
            );
        }

        let mut forecasts = possibility.forecasts().to_vec();

        forecasts.sort_by(Self::forecast_order);

        let mut canonical: Vec<EpistemicHypothesisForecast> = Vec::new();

        for forecast in forecasts {
            if let Some(existing) = canonical
                .iter()
                .find(|existing| existing.hypothesis() == forecast.hypothesis())
            {
                if existing != &forecast {
                    return EpistemicOutcomeResolutionResult::rejected(
                        EpistemicOutcomeResolutionStatus::ConflictingForecastIdentity,
                        input_forecast_count,
                        canonical.len(),
                        target_observation_count,
                    );
                }

                continue;
            }

            canonical.push(forecast);
        }

        let unique_forecast_count = canonical.len();

        let mut assessments = Vec::with_capacity(unique_forecast_count);

        let mut supported_prediction_count = 0_usize;
        let mut counterexample_prediction_count = 0_usize;
        let mut context_uninformative_count = 0_usize;
        let mut no_opportunity_uninformative_count = 0_usize;

        for forecast in canonical {
            let Some(target_observation) = observation
                .targets()
                .iter()
                .find(|target_observation| target_observation.target() == forecast.target())
            else {
                return EpistemicOutcomeResolutionResult::rejected(
                    EpistemicOutcomeResolutionStatus::MissingTargetObservation,
                    input_forecast_count,
                    unique_forecast_count,
                    target_observation_count,
                );
            };

            let status = match forecast.status() {
                EpistemicHypothesisForecastStatus::Predicted => {
                    if target_observation.occurred() {
                        supported_prediction_count = supported_prediction_count.saturating_add(1);

                        EpistemicForecastOutcomeAssessmentStatus::Supported
                    } else {
                        counterexample_prediction_count =
                            counterexample_prediction_count.saturating_add(1);

                        EpistemicForecastOutcomeAssessmentStatus::Counterexample
                    }
                }

                EpistemicHypothesisForecastStatus::ContextAbstained => {
                    context_uninformative_count = context_uninformative_count.saturating_add(1);

                    EpistemicForecastOutcomeAssessmentStatus::ContextUninformative
                }

                EpistemicHypothesisForecastStatus::NoEffectOpportunity => {
                    no_opportunity_uninformative_count =
                        no_opportunity_uninformative_count.saturating_add(1);

                    EpistemicForecastOutcomeAssessmentStatus::NoOpportunityUninformative
                }
            };

            assessments.push(EpistemicForecastOutcomeAssessment {
                forecast,
                observed_target_occurrence: target_observation.occurred(),
                status,
            });
        }

        EpistemicOutcomeResolutionResult {
            status: EpistemicOutcomeResolutionStatus::Resolved,
            input_forecast_count,
            unique_forecast_count,
            target_observation_count,
            assessments,
            supported_prediction_count,
            counterexample_prediction_count,
            context_uninformative_count,
            no_opportunity_uninformative_count,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousEpistemicOutcomeResolution;

impl UniversalAutonomousEpistemicOutcomeResolution {
    pub fn evaluate(
        possibility: &GroundedEpistemicExperimentPossibility,
        observation: &GroundedEpistemicOutcomeObservation,
        policy: EpistemicOutcomeResolutionPolicy,
    ) -> EpistemicOutcomeResolutionResult {
        AutonomousEpistemicOutcomeResolution::evaluate(possibility, observation, policy)
    }
}

// ============================================================================
// P4G-C3D-A — REALIZED EPISTEMIC PROGRESS
// ============================================================================
//
// This is NOT expected information gain.
//
// It measures an observed change in epistemic separation after a real
// consequence has already been resolved:
//
//     same source state + same action
//     pre-learning model separation
//         -> real intervention evidence
//         -> post-learning model separation
//
// Reduction and increase remain separate.  An experience that exposes new
// ambiguity is therefore not mislabeled as zero progress.
//
// The realized outcome must correspond exactly to the complete canonical
// pre-action forecast set.  No detached resolution record can be reused.

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EpistemicResolutionProgressStatus {
    Measured,
    OutcomeNotResolved,
    SourceStateMismatch,
    ActionMismatch,
    ResolutionForecastMismatch,
    PreLearningFrontierTruncated,
    PostLearningFrontierTruncated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicResolutionProgressSample {
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    separation_before: usize,
    separation_after: usize,
    realized_separation_reduction: usize,
    realized_separation_increase: usize,
    supported_prediction_count: usize,
    counterexample_prediction_count: usize,
    context_uninformative_count: usize,
    no_opportunity_uninformative_count: usize,
}

impl EpistemicResolutionProgressSample {
    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn separation_before(&self) -> usize {
        self.separation_before
    }

    pub fn separation_after(&self) -> usize {
        self.separation_after
    }

    pub fn realized_separation_reduction(&self) -> usize {
        self.realized_separation_reduction
    }

    pub fn realized_separation_increase(&self) -> usize {
        self.realized_separation_increase
    }

    pub fn supported_prediction_count(&self) -> usize {
        self.supported_prediction_count
    }

    pub fn counterexample_prediction_count(&self) -> usize {
        self.counterexample_prediction_count
    }

    pub fn empirically_tested_prediction_count(&self) -> usize {
        self.supported_prediction_count
            .saturating_add(self.counterexample_prediction_count)
    }

    pub fn context_uninformative_count(&self) -> usize {
        self.context_uninformative_count
    }

    pub fn no_opportunity_uninformative_count(&self) -> usize {
        self.no_opportunity_uninformative_count
    }

    pub fn reduced_uncertainty(&self) -> bool {
        self.realized_separation_reduction > 0
    }

    pub fn increased_uncertainty(&self) -> bool {
        self.realized_separation_increase > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EpistemicResolutionProgressResult {
    status: EpistemicResolutionProgressStatus,
    sample: Option<EpistemicResolutionProgressSample>,
}

impl EpistemicResolutionProgressResult {
    fn rejected(status: EpistemicResolutionProgressStatus) -> Self {
        Self {
            status,
            sample: None,
        }
    }

    pub fn status(&self) -> EpistemicResolutionProgressStatus {
        self.status
    }

    pub fn measured(&self) -> bool {
        self.status == EpistemicResolutionProgressStatus::Measured
    }

    pub fn sample(&self) -> Option<&EpistemicResolutionProgressSample> {
        self.sample.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousEpistemicResolutionProgress;

impl AutonomousEpistemicResolutionProgress {
    fn canonical_forecasts(
        possibility: &GroundedEpistemicExperimentPossibility,
    ) -> Vec<EpistemicHypothesisForecast> {
        let mut forecasts = possibility.forecasts().to_vec();

        forecasts.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));

        forecasts.dedup();
        forecasts
    }

    fn resolved_forecasts(
        resolution: &EpistemicOutcomeResolutionResult,
    ) -> Vec<EpistemicHypothesisForecast> {
        let mut forecasts = resolution
            .assessments()
            .iter()
            .map(|assessment| assessment.forecast().clone())
            .collect::<Vec<_>>();

        forecasts.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));

        forecasts.dedup();
        forecasts
    }

    pub fn measure(
        pre_learning: &GroundedEpistemicExperimentPossibility,
        realized_outcome: &EpistemicOutcomeResolutionResult,
        post_learning: &GroundedEpistemicExperimentPossibility,
        discrimination_policy: EpistemicForecastDiscriminationPolicy,
    ) -> EpistemicResolutionProgressResult {
        if !realized_outcome.resolved() {
            return EpistemicResolutionProgressResult::rejected(
                EpistemicResolutionProgressStatus::OutcomeNotResolved,
            );
        }

        if pre_learning.source_state() != post_learning.source_state() {
            return EpistemicResolutionProgressResult::rejected(
                EpistemicResolutionProgressStatus::SourceStateMismatch,
            );
        }

        if pre_learning.action() != post_learning.action() {
            return EpistemicResolutionProgressResult::rejected(
                EpistemicResolutionProgressStatus::ActionMismatch,
            );
        }

        if Self::canonical_forecasts(pre_learning) != Self::resolved_forecasts(realized_outcome) {
            return EpistemicResolutionProgressResult::rejected(
                EpistemicResolutionProgressStatus::ResolutionForecastMismatch,
            );
        }

        let before = AutonomousEpistemicForecastDiscrimination::evaluate(
            pre_learning,
            discrimination_policy,
        );

        if before.forecast_frontier_truncated() || before.target_frontier_truncated() {
            return EpistemicResolutionProgressResult::rejected(
                EpistemicResolutionProgressStatus::PreLearningFrontierTruncated,
            );
        }

        let after = AutonomousEpistemicForecastDiscrimination::evaluate(
            post_learning,
            discrimination_policy,
        );

        if after.forecast_frontier_truncated() || after.target_frontier_truncated() {
            return EpistemicResolutionProgressResult::rejected(
                EpistemicResolutionProgressStatus::PostLearningFrontierTruncated,
            );
        }

        let separation_before = before.pairwise_separation_score();

        let separation_after = after.pairwise_separation_score();

        let realized_separation_reduction = separation_before.saturating_sub(separation_after);

        let realized_separation_increase = separation_after.saturating_sub(separation_before);

        EpistemicResolutionProgressResult {
            status: EpistemicResolutionProgressStatus::Measured,

            sample: Some(EpistemicResolutionProgressSample {
                source_state: pre_learning.source_state().clone(),
                action: pre_learning.action().clone(),
                separation_before,
                separation_after,
                realized_separation_reduction,
                realized_separation_increase,
                supported_prediction_count: realized_outcome.supported_prediction_count(),
                counterexample_prediction_count: realized_outcome.counterexample_prediction_count(),
                context_uninformative_count: realized_outcome.context_uninformative_count(),
                no_opportunity_uninformative_count: realized_outcome
                    .no_opportunity_uninformative_count(),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousEpistemicResolutionProgress;

impl UniversalAutonomousEpistemicResolutionProgress {
    pub fn measure(
        pre_learning: &GroundedEpistemicExperimentPossibility,
        realized_outcome: &EpistemicOutcomeResolutionResult,
        post_learning: &GroundedEpistemicExperimentPossibility,
        discrimination_policy: EpistemicForecastDiscriminationPolicy,
    ) -> EpistemicResolutionProgressResult {
        AutonomousEpistemicResolutionProgress::measure(
            pre_learning,
            realized_outcome,
            post_learning,
            discrimination_policy,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HypothesisDiscriminationCandidate {
    experiment: AutonomousExperimentProposal,
    predictions: Vec<CompetingHypothesisPrediction>,
}

impl HypothesisDiscriminationCandidate {
    pub fn new(
        experiment: AutonomousExperimentProposal,
        predictions: Vec<CompetingHypothesisPrediction>,
    ) -> Option<Self> {
        if predictions.is_empty() {
            return None;
        }

        Some(Self {
            experiment,
            predictions,
        })
    }

    pub fn experiment(&self) -> &AutonomousExperimentProposal {
        &self.experiment
    }

    pub fn predictions(&self) -> &[CompetingHypothesisPrediction] {
        &self.predictions
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HypothesisDiscriminationBounds {
    max_input_candidates: usize,
    max_candidate_evaluations: usize,
    max_predictions_per_candidate: usize,
    max_selected_candidates: usize,
}

impl HypothesisDiscriminationBounds {
    pub fn new(
        max_input_candidates: usize,
        max_candidate_evaluations: usize,
        max_predictions_per_candidate: usize,
        max_selected_candidates: usize,
    ) -> Option<Self> {
        if max_input_candidates == 0
            || max_candidate_evaluations == 0
            || max_predictions_per_candidate == 0
            || max_selected_candidates == 0
        {
            return None;
        }

        Some(Self {
            max_input_candidates,
            max_candidate_evaluations,
            max_predictions_per_candidate,
            max_selected_candidates,
        })
    }

    pub fn max_input_candidates(self) -> usize {
        self.max_input_candidates
    }

    pub fn max_candidate_evaluations(self) -> usize {
        self.max_candidate_evaluations
    }

    pub fn max_predictions_per_candidate(self) -> usize {
        self.max_predictions_per_candidate
    }

    pub fn max_selected_candidates(self) -> usize {
        self.max_selected_candidates
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HypothesisDiscriminationThresholds {
    minimum_hypothesis_count: usize,
    minimum_distinct_outcomes: usize,
    minimum_prediction_confidence: CognitiveSignal,
    minimum_discrimination_gain: CognitiveSignal,
}

impl HypothesisDiscriminationThresholds {
    pub fn new(
        minimum_hypothesis_count: usize,
        minimum_distinct_outcomes: usize,
        minimum_prediction_confidence: CognitiveSignal,
        minimum_discrimination_gain: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_hypothesis_count < 2
            || minimum_distinct_outcomes < 2
            || minimum_distinct_outcomes > minimum_hypothesis_count
            || minimum_prediction_confidence == CognitiveSignal::zero()
            || minimum_discrimination_gain == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_hypothesis_count,
            minimum_distinct_outcomes,
            minimum_prediction_confidence,
            minimum_discrimination_gain,
        })
    }

    pub fn minimum_hypothesis_count(self) -> usize {
        self.minimum_hypothesis_count
    }

    pub fn minimum_distinct_outcomes(self) -> usize {
        self.minimum_distinct_outcomes
    }

    pub fn minimum_prediction_confidence(self) -> CognitiveSignal {
        self.minimum_prediction_confidence
    }

    pub fn minimum_discrimination_gain(self) -> CognitiveSignal {
        self.minimum_discrimination_gain
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HypothesisDiscriminationPolicy {
    foundation: ActiveExperimentPolicy,
    bounds: HypothesisDiscriminationBounds,
    thresholds: HypothesisDiscriminationThresholds,
}

impl HypothesisDiscriminationPolicy {
    pub fn new(
        foundation: ActiveExperimentPolicy,
        bounds: HypothesisDiscriminationBounds,
        thresholds: HypothesisDiscriminationThresholds,
    ) -> Self {
        Self {
            foundation,
            bounds,
            thresholds,
        }
    }

    pub fn foundation(self) -> ActiveExperimentPolicy {
        self.foundation
    }

    pub fn bounds(self) -> HypothesisDiscriminationBounds {
        self.bounds
    }

    pub fn thresholds(self) -> HypothesisDiscriminationThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedHypothesisDiscriminatingExperiment {
    experiment: AutonomousExperimentProposal,
    qualifying_hypothesis_count: usize,
    distinct_outcome_count: usize,
    discrimination_gain: CognitiveSignal,
}

impl SelectedHypothesisDiscriminatingExperiment {
    pub fn experiment(&self) -> &AutonomousExperimentProposal {
        &self.experiment
    }

    pub fn qualifying_hypothesis_count(&self) -> usize {
        self.qualifying_hypothesis_count
    }

    pub fn distinct_outcome_count(&self) -> usize {
        self.distinct_outcome_count
    }

    pub fn discrimination_gain(&self) -> CognitiveSignal {
        self.discrimination_gain
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HypothesisDiscriminationResult {
    input_candidate_count: usize,
    unique_candidate_count: usize,
    considered_candidate_count: usize,
    input_frontier_truncated: bool,
    evaluation_count: usize,
    evaluation_frontier_truncated: bool,
    rejected_foundation_count: usize,
    rejected_prediction_frontier_count: usize,
    rejected_conflicting_prediction_count: usize,
    rejected_hypothesis_count: usize,
    rejected_distinct_outcome_count: usize,
    rejected_discrimination_gain_count: usize,
    selected_before_frontier: usize,
    selection_frontier_truncated: bool,
    selected: Vec<SelectedHypothesisDiscriminatingExperiment>,
}

impl HypothesisDiscriminationResult {
    pub fn input_candidate_count(&self) -> usize {
        self.input_candidate_count
    }

    pub fn unique_candidate_count(&self) -> usize {
        self.unique_candidate_count
    }

    pub fn considered_candidate_count(&self) -> usize {
        self.considered_candidate_count
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

    pub fn rejected_foundation_count(&self) -> usize {
        self.rejected_foundation_count
    }

    pub fn rejected_prediction_frontier_count(&self) -> usize {
        self.rejected_prediction_frontier_count
    }

    pub fn rejected_conflicting_prediction_count(&self) -> usize {
        self.rejected_conflicting_prediction_count
    }

    pub fn rejected_hypothesis_count(&self) -> usize {
        self.rejected_hypothesis_count
    }

    pub fn rejected_distinct_outcome_count(&self) -> usize {
        self.rejected_distinct_outcome_count
    }

    pub fn rejected_discrimination_gain_count(&self) -> usize {
        self.rejected_discrimination_gain_count
    }

    pub fn selected_before_frontier(&self) -> usize {
        self.selected_before_frontier
    }

    pub fn selection_frontier_truncated(&self) -> bool {
        self.selection_frontier_truncated
    }

    pub fn selected(&self) -> &[SelectedHypothesisDiscriminatingExperiment] {
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
pub struct AutonomousHypothesisDiscrimination;

impl AutonomousHypothesisDiscrimination {
    fn candidate_order(
        left: &HypothesisDiscriminationCandidate,
        right: &HypothesisDiscriminationCandidate,
    ) -> std::cmp::Ordering {
        right
            .experiment()
            .evidence()
            .expected_information_gain()
            .value()
            .cmp(
                &left
                    .experiment()
                    .evidence()
                    .expected_information_gain()
                    .value(),
            )
            .then_with(|| right.predictions().len().cmp(&left.predictions().len()))
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    fn selected_order(
        left: &SelectedHypothesisDiscriminatingExperiment,
        right: &SelectedHypothesisDiscriminatingExperiment,
    ) -> std::cmp::Ordering {
        right
            .discrimination_gain()
            .value()
            .cmp(&left.discrimination_gain().value())
            .then_with(|| {
                right
                    .experiment()
                    .evidence()
                    .expected_information_gain()
                    .value()
                    .cmp(
                        &left
                            .experiment()
                            .evidence()
                            .expected_information_gain()
                            .value(),
                    )
            })
            .then_with(|| {
                format!("{:?}", left.experiment()).cmp(&format!("{:?}", right.experiment()))
            })
    }

    fn signal(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).expect("bounded discrimination signal")
        }
    }

    fn discrimination_gain(
        predictions: &[(CognitiveStructure, CognitiveStructure, CognitiveSignal)],
    ) -> CognitiveSignal {
        if predictions.len() < 2 {
            return CognitiveSignal::zero();
        }

        let mut total_pairs = 0usize;
        let mut disagreeing_pairs = 0usize;

        for left in 0..predictions.len() {
            for right in (left + 1)..predictions.len() {
                total_pairs += 1;

                if predictions[left].1 != predictions[right].1 {
                    disagreeing_pairs += 1;
                }
            }
        }

        if total_pairs == 0 {
            return CognitiveSignal::zero();
        }

        let value = (disagreeing_pairs.saturating_mul(1000) / total_pairs).min(1000) as u16;

        Self::signal(value)
    }

    pub fn discriminate(
        candidates: &[HypothesisDiscriminationCandidate],
        policy: HypothesisDiscriminationPolicy,
    ) -> HypothesisDiscriminationResult {
        let bounds = policy.bounds();
        let thresholds = policy.thresholds();

        let input_candidate_count = candidates.len();

        let mut ranked = candidates.to_vec();

        ranked.sort_by(Self::candidate_order);
        ranked.dedup();

        let unique_candidate_count = ranked.len();

        ranked.truncate(bounds.max_input_candidates());

        let considered_candidate_count = ranked.len();

        let mut evaluation_count = 0;
        let mut evaluation_frontier_truncated = false;

        let mut rejected_foundation_count = 0;
        let mut rejected_prediction_frontier_count = 0;
        let mut rejected_conflicting_prediction_count = 0;
        let mut rejected_hypothesis_count = 0;
        let mut rejected_distinct_outcome_count = 0;
        let mut rejected_discrimination_gain_count = 0;

        let mut selected = Vec::new();

        for candidate in ranked {
            if evaluation_count >= bounds.max_candidate_evaluations() {
                evaluation_frontier_truncated = true;
                break;
            }

            evaluation_count += 1;

            let foundation = AutonomousActiveExperimentationFoundation::select(
                std::slice::from_ref(candidate.experiment()),
                policy.foundation(),
            );

            if foundation.abstained() {
                rejected_foundation_count += 1;
                continue;
            }

            if candidate.predictions().len() > bounds.max_predictions_per_candidate() {
                rejected_prediction_frontier_count += 1;
                continue;
            }

            let mut predictions = candidate.predictions().to_vec();

            predictions.sort_by(|left, right| {
                format!("{:?}", left.hypothesis())
                    .cmp(&format!("{:?}", right.hypothesis()))
                    .then_with(|| right.confidence().value().cmp(&left.confidence().value()))
                    .then_with(|| {
                        format!("{:?}", left.predicted_outcome())
                            .cmp(&format!("{:?}", right.predicted_outcome()))
                    })
            });

            let mut canonical: Vec<(CognitiveStructure, CognitiveStructure, CognitiveSignal)> =
                Vec::new();

            let mut conflicting_prediction = false;

            for prediction in predictions {
                if prediction.confidence().value()
                    < thresholds.minimum_prediction_confidence().value()
                {
                    continue;
                }

                if let Some(existing) = canonical
                    .iter()
                    .find(|existing| existing.0 == *prediction.hypothesis())
                {
                    if existing.1 != *prediction.predicted_outcome() {
                        conflicting_prediction = true;
                        break;
                    }

                    continue;
                }

                canonical.push((
                    prediction.hypothesis().clone(),
                    prediction.predicted_outcome().clone(),
                    prediction.confidence(),
                ));
            }

            if conflicting_prediction {
                rejected_conflicting_prediction_count += 1;
                continue;
            }

            if canonical.len() < thresholds.minimum_hypothesis_count() {
                rejected_hypothesis_count += 1;
                continue;
            }

            let mut outcomes: Vec<CognitiveStructure> = Vec::new();

            for prediction in &canonical {
                if !outcomes.contains(&prediction.1) {
                    outcomes.push(prediction.1.clone());
                }
            }

            if outcomes.len() < thresholds.minimum_distinct_outcomes() {
                rejected_distinct_outcome_count += 1;
                continue;
            }

            let gain = Self::discrimination_gain(&canonical);

            if gain.value() < thresholds.minimum_discrimination_gain().value() {
                rejected_discrimination_gain_count += 1;
                continue;
            }

            selected.push(SelectedHypothesisDiscriminatingExperiment {
                experiment: candidate.experiment().clone(),
                qualifying_hypothesis_count: canonical.len(),
                distinct_outcome_count: outcomes.len(),
                discrimination_gain: gain,
            });
        }

        selected.sort_by(Self::selected_order);

        let selected_before_frontier = selected.len();

        selected.truncate(bounds.max_selected_candidates());

        HypothesisDiscriminationResult {
            input_candidate_count,
            unique_candidate_count,
            considered_candidate_count,
            input_frontier_truncated: unique_candidate_count > considered_candidate_count,
            evaluation_count,
            evaluation_frontier_truncated,
            rejected_foundation_count,
            rejected_prediction_frontier_count,
            rejected_conflicting_prediction_count,
            rejected_hypothesis_count,
            rejected_distinct_outcome_count,
            rejected_discrimination_gain_count,
            selected_before_frontier,
            selection_frontier_truncated: selected_before_frontier > selected.len(),
            selected,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousHypothesisDiscrimination;

impl UniversalAutonomousHypothesisDiscrimination {
    pub fn evaluate(
        candidates: &[HypothesisDiscriminationCandidate],
        policy: HypothesisDiscriminationPolicy,
    ) -> HypothesisDiscriminationResult {
        AutonomousHypothesisDiscrimination::discriminate(candidates, policy)
    }
}

#[cfg(test)]
mod hypothesis_discrimination_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn evidence(info: u16, grounding: u16) -> ExperimentEvidence {
        ExperimentEvidence::new(s(800), s(info), s(800), s(grounding), s(100)).unwrap()
    }

    fn experiment(action: u64, info: u16) -> AutonomousExperimentProposal {
        AutonomousExperimentProposal::new(a(1), a(action), a(900), evidence(info, 900))
    }

    fn prediction(hypothesis: u64, outcome: u64, confidence: u16) -> CompetingHypothesisPrediction {
        CompetingHypothesisPrediction::new(a(hypothesis), a(outcome), s(confidence)).unwrap()
    }

    fn candidate(
        action: u64,
        info: u16,
        predictions: Vec<CompetingHypothesisPrediction>,
    ) -> HypothesisDiscriminationCandidate {
        HypothesisDiscriminationCandidate::new(experiment(action, info), predictions).unwrap()
    }

    fn foundation_policy() -> ActiveExperimentPolicy {
        ActiveExperimentPolicy::new(
            ActiveExperimentBounds::new(32, 32, 32).unwrap(),
            ActiveExperimentThresholds::new(s(500), s(500), s(500), s(500)).unwrap(),
        )
    }

    fn thresholds() -> HypothesisDiscriminationThresholds {
        HypothesisDiscriminationThresholds::new(2, 2, s(500), s(500)).unwrap()
    }

    fn policy() -> HypothesisDiscriminationPolicy {
        HypothesisDiscriminationPolicy::new(
            foundation_policy(),
            HypothesisDiscriminationBounds::new(32, 32, 32, 32).unwrap(),
            thresholds(),
        )
    }

    #[test]
    fn discrimination_contract_requires_competing_grounded_hypotheses() {
        assert_eq!(CompetingHypothesisPrediction::new(a(1), a(2), s(0),), None);

        assert_eq!(HypothesisDiscriminationBounds::new(0, 1, 1, 1), None);

        assert_eq!(
            HypothesisDiscriminationThresholds::new(1, 2, s(500), s(500),),
            None
        );

        assert!(HypothesisDiscriminationCandidate::new(experiment(10, 800), Vec::new(),).is_none());
    }

    #[test]
    fn two_confident_hypotheses_with_distinct_predictions_are_discriminated() {
        let item = candidate(
            10,
            800,
            vec![prediction(100, 200, 900), prediction(101, 201, 900)],
        );

        let result = AutonomousHypothesisDiscrimination::discriminate(&[item], policy());

        assert_eq!(result.selected_count(), 1);

        assert_eq!(result.selected()[0].qualifying_hypothesis_count(), 2);

        assert_eq!(result.selected()[0].distinct_outcome_count(), 2);

        assert_eq!(result.selected()[0].discrimination_gain(), s(1000));
    }

    #[test]
    fn identical_hypothesis_predictions_do_not_create_discrimination() {
        let item = candidate(
            10,
            800,
            vec![prediction(100, 200, 900), prediction(101, 200, 900)],
        );

        let result = AutonomousHypothesisDiscrimination::discriminate(&[item], policy());

        assert!(result.abstained());

        assert_eq!(result.rejected_distinct_outcome_count(), 1);
    }

    #[test]
    fn low_confidence_prediction_cannot_fake_competing_hypothesis_support() {
        let item = candidate(
            10,
            800,
            vec![prediction(100, 200, 900), prediction(101, 201, 400)],
        );

        let result = AutonomousHypothesisDiscrimination::discriminate(&[item], policy());

        assert!(result.abstained());

        assert_eq!(result.rejected_hypothesis_count(), 1);
    }

    #[test]
    fn conflicting_predictions_from_same_hypothesis_are_rejected() {
        let item = candidate(
            10,
            800,
            vec![
                prediction(100, 200, 900),
                prediction(100, 201, 900),
                prediction(101, 202, 900),
            ],
        );

        let result = AutonomousHypothesisDiscrimination::discriminate(&[item], policy());

        assert!(result.abstained());

        assert_eq!(result.rejected_conflicting_prediction_count(), 1);
    }

    #[test]
    fn exact_duplicate_prediction_does_not_inflate_hypothesis_count() {
        let duplicate = prediction(100, 200, 900);

        let item = candidate(
            10,
            800,
            vec![duplicate.clone(), duplicate, prediction(101, 201, 900)],
        );

        let result = AutonomousHypothesisDiscrimination::discriminate(&[item], policy());

        assert_eq!(result.selected_count(), 1);

        assert_eq!(result.selected()[0].qualifying_hypothesis_count(), 2);
    }

    #[test]
    fn partial_three_way_disagreement_has_pairwise_discrimination_gain() {
        let item = candidate(
            10,
            800,
            vec![
                prediction(100, 200, 900),
                prediction(101, 200, 900),
                prediction(102, 201, 900),
            ],
        );

        let result = AutonomousHypothesisDiscrimination::discriminate(&[item], policy());

        assert_eq!(result.selected_count(), 1);

        assert_eq!(result.selected()[0].discrimination_gain(), s(666));
    }

    #[test]
    fn discrimination_gain_ranks_before_generic_information_gain() {
        let partial = candidate(
            10,
            950,
            vec![
                prediction(100, 200, 900),
                prediction(101, 200, 900),
                prediction(102, 201, 900),
            ],
        );

        let full = candidate(
            11,
            700,
            vec![prediction(110, 210, 900), prediction(111, 211, 900)],
        );

        let result =
            AutonomousHypothesisDiscrimination::discriminate(&[partial, full.clone()], policy());

        assert_eq!(result.selected_count(), 2);

        assert_eq!(result.selected()[0].experiment(), full.experiment());

        assert_eq!(result.selected()[0].discrimination_gain(), s(1000));
    }

    #[test]
    fn foundation_grounding_gate_remains_authoritative() {
        let weak = AutonomousExperimentProposal::new(a(1), a(10), a(900), evidence(900, 400));

        let item = HypothesisDiscriminationCandidate::new(
            weak,
            vec![prediction(100, 200, 900), prediction(101, 201, 900)],
        )
        .unwrap();

        let result = AutonomousHypothesisDiscrimination::discriminate(&[item], policy());

        assert!(result.abstained());

        assert_eq!(result.rejected_foundation_count(), 1);
    }

    #[test]
    fn prediction_frontier_causes_abstention_instead_of_partial_guessing() {
        let item = candidate(
            10,
            800,
            vec![
                prediction(100, 200, 900),
                prediction(101, 201, 900),
                prediction(102, 202, 900),
            ],
        );

        let bounded = HypothesisDiscriminationPolicy::new(
            foundation_policy(),
            HypothesisDiscriminationBounds::new(32, 32, 2, 32).unwrap(),
            thresholds(),
        );

        let result = AutonomousHypothesisDiscrimination::discriminate(&[item], bounded);

        assert!(result.abstained());

        assert_eq!(result.rejected_prediction_frontier_count(), 1);
    }

    #[test]
    fn hard_input_evaluation_and_selection_frontiers_are_enforced() {
        let items = vec![
            candidate(
                10,
                900,
                vec![prediction(100, 200, 900), prediction(101, 201, 900)],
            ),
            candidate(
                11,
                800,
                vec![prediction(110, 210, 900), prediction(111, 211, 900)],
            ),
            candidate(
                12,
                700,
                vec![prediction(120, 220, 900), prediction(121, 221, 900)],
            ),
        ];

        let input_policy = HypothesisDiscriminationPolicy::new(
            foundation_policy(),
            HypothesisDiscriminationBounds::new(1, 32, 32, 32).unwrap(),
            thresholds(),
        );

        let input = AutonomousHypothesisDiscrimination::discriminate(&items, input_policy);

        assert_eq!(input.unique_candidate_count(), 3);

        assert_eq!(input.considered_candidate_count(), 1);

        assert!(input.input_frontier_truncated());

        let eval_policy = HypothesisDiscriminationPolicy::new(
            foundation_policy(),
            HypothesisDiscriminationBounds::new(32, 1, 32, 32).unwrap(),
            thresholds(),
        );

        let eval = AutonomousHypothesisDiscrimination::discriminate(&items, eval_policy);

        assert_eq!(eval.evaluation_count(), 1);

        assert!(eval.evaluation_frontier_truncated());

        let selection_policy = HypothesisDiscriminationPolicy::new(
            foundation_policy(),
            HypothesisDiscriminationBounds::new(32, 32, 32, 1).unwrap(),
            thresholds(),
        );

        let selection = AutonomousHypothesisDiscrimination::discriminate(&items, selection_policy);

        assert_eq!(selection.selected_before_frontier(), 3);

        assert_eq!(selection.selected_count(), 1);

        assert!(selection.selection_frontier_truncated());
    }

    #[test]
    fn discrimination_is_order_invariant_non_mutating_and_facade_equivalent() {
        let items = vec![
            candidate(
                10,
                900,
                vec![prediction(100, 200, 900), prediction(101, 201, 900)],
            ),
            candidate(
                11,
                800,
                vec![prediction(110, 210, 900), prediction(111, 211, 900)],
            ),
        ];

        let before = items.clone();

        let mut reversed = items.clone();
        reversed.reverse();

        let p = policy();

        let direct = AutonomousHypothesisDiscrimination::discriminate(&items, p);

        let reordered = AutonomousHypothesisDiscrimination::discriminate(&reversed, p);

        let facade = UniversalAutonomousHypothesisDiscrimination::evaluate(&items, p);

        let repeated = UniversalAutonomousHypothesisDiscrimination::evaluate(&items, p);

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(items, before);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentOutcomeObservation {
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    observed_outcome: CognitiveStructure,
    confidence: CognitiveSignal,
}

impl ExperimentOutcomeObservation {
    pub fn new(
        source_state: CognitiveStructure,
        action: CognitiveStructure,
        observed_outcome: CognitiveStructure,
        confidence: CognitiveSignal,
    ) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            source_state,
            action,
            observed_outcome,
            confidence,
        })
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn observed_outcome(&self) -> &CognitiveStructure {
        &self.observed_outcome
    }

    pub fn confidence(&self) -> CognitiveSignal {
        self.confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutcomeEvidenceUpdatePolicy {
    max_prediction_updates: usize,
    minimum_observation_confidence: CognitiveSignal,
    minimum_prediction_confidence: CognitiveSignal,
    support_recovery: u16,
    contradiction_penalty: u16,
}

impl OutcomeEvidenceUpdatePolicy {
    pub fn new(
        max_prediction_updates: usize,
        minimum_observation_confidence: CognitiveSignal,
        minimum_prediction_confidence: CognitiveSignal,
        support_recovery: u16,
        contradiction_penalty: u16,
    ) -> Option<Self> {
        if max_prediction_updates == 0
            || minimum_observation_confidence == CognitiveSignal::zero()
            || minimum_prediction_confidence == CognitiveSignal::zero()
            || support_recovery == 0
            || contradiction_penalty == 0
        {
            return None;
        }

        Some(Self {
            max_prediction_updates,
            minimum_observation_confidence,
            minimum_prediction_confidence,
            support_recovery,
            contradiction_penalty,
        })
    }

    pub fn max_prediction_updates(self) -> usize {
        self.max_prediction_updates
    }

    pub fn minimum_observation_confidence(self) -> CognitiveSignal {
        self.minimum_observation_confidence
    }

    pub fn minimum_prediction_confidence(self) -> CognitiveSignal {
        self.minimum_prediction_confidence
    }

    pub fn support_recovery(self) -> u16 {
        self.support_recovery
    }

    pub fn contradiction_penalty(self) -> u16 {
        self.contradiction_penalty
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HypothesisOutcomeEvidenceDisposition {
    Supported,
    Contradicted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HypothesisOutcomeEvidenceUpdate {
    hypothesis: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
    prior_confidence: CognitiveSignal,
    revised_confidence: CognitiveSignal,
    disposition: HypothesisOutcomeEvidenceDisposition,
}

impl HypothesisOutcomeEvidenceUpdate {
    pub fn hypothesis(&self) -> &CognitiveStructure {
        &self.hypothesis
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        &self.predicted_outcome
    }

    pub fn prior_confidence(&self) -> CognitiveSignal {
        self.prior_confidence
    }

    pub fn revised_confidence(&self) -> CognitiveSignal {
        self.revised_confidence
    }

    pub fn disposition(&self) -> HypothesisOutcomeEvidenceDisposition {
        self.disposition
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeEvidenceUpdateStatus {
    Applied,
    AttributionMismatch,
    ObservationConfidenceInsufficient,
    PredictionFrontierExceeded,
    ConflictingPredictionEvidence,
    NoQualifyingPredictions,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutcomeEvidenceUpdateResult {
    status: OutcomeEvidenceUpdateStatus,
    input_prediction_count: usize,
    qualifying_prediction_count: usize,
    rejected_prediction_confidence_count: usize,
    supported_count: usize,
    contradicted_count: usize,
    updates: Vec<HypothesisOutcomeEvidenceUpdate>,
}

impl OutcomeEvidenceUpdateResult {
    pub fn status(&self) -> OutcomeEvidenceUpdateStatus {
        self.status
    }

    pub fn input_prediction_count(&self) -> usize {
        self.input_prediction_count
    }

    pub fn qualifying_prediction_count(&self) -> usize {
        self.qualifying_prediction_count
    }

    pub fn rejected_prediction_confidence_count(&self) -> usize {
        self.rejected_prediction_confidence_count
    }

    pub fn supported_count(&self) -> usize {
        self.supported_count
    }

    pub fn contradicted_count(&self) -> usize {
        self.contradicted_count
    }

    pub fn updates(&self) -> &[HypothesisOutcomeEvidenceUpdate] {
        &self.updates
    }

    pub fn applied(&self) -> bool {
        self.status == OutcomeEvidenceUpdateStatus::Applied
    }

    pub fn abstained(&self) -> bool {
        !self.applied()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousOutcomeEvidenceUpdate;

impl AutonomousOutcomeEvidenceUpdate {
    fn signal(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).expect("bounded outcome evidence signal")
        }
    }

    fn empty(
        status: OutcomeEvidenceUpdateStatus,
        input_prediction_count: usize,
        rejected_prediction_confidence_count: usize,
    ) -> OutcomeEvidenceUpdateResult {
        OutcomeEvidenceUpdateResult {
            status,
            input_prediction_count,
            qualifying_prediction_count: 0,
            rejected_prediction_confidence_count,
            supported_count: 0,
            contradicted_count: 0,
            updates: Vec::new(),
        }
    }

    pub fn update(
        candidate: &HypothesisDiscriminationCandidate,
        selected: &SelectedHypothesisDiscriminatingExperiment,
        observation: &ExperimentOutcomeObservation,
        policy: OutcomeEvidenceUpdatePolicy,
    ) -> OutcomeEvidenceUpdateResult {
        let input_prediction_count = candidate.predictions().len();

        if candidate.experiment() != selected.experiment()
            || observation.source_state() != selected.experiment().source_state()
            || observation.action() != selected.experiment().action()
        {
            return Self::empty(
                OutcomeEvidenceUpdateStatus::AttributionMismatch,
                input_prediction_count,
                0,
            );
        }

        if observation.confidence().value() < policy.minimum_observation_confidence().value() {
            return Self::empty(
                OutcomeEvidenceUpdateStatus::ObservationConfidenceInsufficient,
                input_prediction_count,
                0,
            );
        }

        if input_prediction_count > policy.max_prediction_updates() {
            return Self::empty(
                OutcomeEvidenceUpdateStatus::PredictionFrontierExceeded,
                input_prediction_count,
                0,
            );
        }

        let mut predictions = candidate.predictions().to_vec();

        predictions.sort_by(|left, right| {
            format!("{:?}", left.hypothesis())
                .cmp(&format!("{:?}", right.hypothesis()))
                .then_with(|| right.confidence().value().cmp(&left.confidence().value()))
                .then_with(|| {
                    format!("{:?}", left.predicted_outcome())
                        .cmp(&format!("{:?}", right.predicted_outcome()))
                })
        });

        let mut canonical: Vec<CompetingHypothesisPrediction> = Vec::new();

        let mut rejected_prediction_confidence_count = 0;

        for prediction in predictions {
            if prediction.confidence().value() < policy.minimum_prediction_confidence().value() {
                rejected_prediction_confidence_count += 1;
                continue;
            }

            if let Some(existing) = canonical
                .iter()
                .find(|existing| existing.hypothesis() == prediction.hypothesis())
            {
                if existing.predicted_outcome() != prediction.predicted_outcome() {
                    return Self::empty(
                        OutcomeEvidenceUpdateStatus::ConflictingPredictionEvidence,
                        input_prediction_count,
                        rejected_prediction_confidence_count,
                    );
                }

                continue;
            }

            canonical.push(prediction);
        }

        if canonical.is_empty() {
            return Self::empty(
                OutcomeEvidenceUpdateStatus::NoQualifyingPredictions,
                input_prediction_count,
                rejected_prediction_confidence_count,
            );
        }

        let qualifying_prediction_count = canonical.len();

        let mut supported_count = 0;
        let mut contradicted_count = 0;

        let mut updates = Vec::new();

        for prediction in canonical {
            let prior = prediction.confidence();

            let (disposition, revised) =
                if prediction.predicted_outcome() == observation.observed_outcome() {
                    supported_count += 1;

                    let value = prior
                        .value()
                        .saturating_add(policy.support_recovery())
                        .min(1000);

                    (
                        HypothesisOutcomeEvidenceDisposition::Supported,
                        Self::signal(value),
                    )
                } else {
                    contradicted_count += 1;

                    let value = prior.value().saturating_sub(policy.contradiction_penalty());

                    (
                        HypothesisOutcomeEvidenceDisposition::Contradicted,
                        Self::signal(value),
                    )
                };

            updates.push(HypothesisOutcomeEvidenceUpdate {
                hypothesis: prediction.hypothesis().clone(),
                predicted_outcome: prediction.predicted_outcome().clone(),
                prior_confidence: prior,
                revised_confidence: revised,
                disposition,
            });
        }

        updates.sort_by(|left, right| {
            format!("{:?}", left.hypothesis()).cmp(&format!("{:?}", right.hypothesis()))
        });

        OutcomeEvidenceUpdateResult {
            status: OutcomeEvidenceUpdateStatus::Applied,
            input_prediction_count,
            qualifying_prediction_count,
            rejected_prediction_confidence_count,
            supported_count,
            contradicted_count,
            updates,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousOutcomeEvidenceUpdate;

impl UniversalAutonomousOutcomeEvidenceUpdate {
    pub fn evaluate(
        candidate: &HypothesisDiscriminationCandidate,
        selected: &SelectedHypothesisDiscriminatingExperiment,
        observation: &ExperimentOutcomeObservation,
        policy: OutcomeEvidenceUpdatePolicy,
    ) -> OutcomeEvidenceUpdateResult {
        AutonomousOutcomeEvidenceUpdate::update(candidate, selected, observation, policy)
    }
}

#[cfg(test)]
mod outcome_evidence_update_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn experiment(action: u64) -> AutonomousExperimentProposal {
        AutonomousExperimentProposal::new(
            a(1),
            a(action),
            a(900),
            ExperimentEvidence::new(s(800), s(900), s(900), s(900), s(100)).unwrap(),
        )
    }

    fn prediction(hypothesis: u64, outcome: u64, confidence: u16) -> CompetingHypothesisPrediction {
        CompetingHypothesisPrediction::new(a(hypothesis), a(outcome), s(confidence)).unwrap()
    }

    fn candidate(
        action: u64,
        predictions: Vec<CompetingHypothesisPrediction>,
    ) -> HypothesisDiscriminationCandidate {
        HypothesisDiscriminationCandidate::new(experiment(action), predictions).unwrap()
    }

    fn discrimination_policy() -> HypothesisDiscriminationPolicy {
        HypothesisDiscriminationPolicy::new(
            ActiveExperimentPolicy::new(
                ActiveExperimentBounds::new(16, 16, 16).unwrap(),
                ActiveExperimentThresholds::new(s(500), s(500), s(500), s(500)).unwrap(),
            ),
            HypothesisDiscriminationBounds::new(16, 16, 16, 16).unwrap(),
            HypothesisDiscriminationThresholds::new(2, 2, s(500), s(500)).unwrap(),
        )
    }

    fn selected(
        item: &HypothesisDiscriminationCandidate,
    ) -> SelectedHypothesisDiscriminatingExperiment {
        AutonomousHypothesisDiscrimination::discriminate(
            std::slice::from_ref(item),
            discrimination_policy(),
        )
        .selected()[0]
            .clone()
    }

    fn observation(action: u64, outcome: u64, confidence: u16) -> ExperimentOutcomeObservation {
        ExperimentOutcomeObservation::new(a(1), a(action), a(outcome), s(confidence)).unwrap()
    }

    fn policy() -> OutcomeEvidenceUpdatePolicy {
        OutcomeEvidenceUpdatePolicy::new(16, s(500), s(500), 100, 200).unwrap()
    }

    #[test]
    fn outcome_update_requires_positive_policy_and_observation_confidence() {
        assert_eq!(
            ExperimentOutcomeObservation::new(a(1), a(2), a(3), s(0),),
            None
        );

        assert_eq!(
            OutcomeEvidenceUpdatePolicy::new(0, s(500), s(500), 100, 100,),
            None
        );

        assert_eq!(
            OutcomeEvidenceUpdatePolicy::new(1, s(500), s(500), 0, 100,),
            None
        );
    }

    #[test]
    fn exact_observed_outcome_supports_matching_and_contradicts_competitor() {
        let item = candidate(
            10,
            vec![prediction(100, 200, 800), prediction(101, 201, 800)],
        );

        let chosen = selected(&item);

        let result = AutonomousOutcomeEvidenceUpdate::update(
            &item,
            &chosen,
            &observation(10, 200, 900),
            policy(),
        );

        assert!(result.applied());
        assert_eq!(result.supported_count(), 1);
        assert_eq!(result.contradicted_count(), 1);

        let supported = result
            .updates()
            .iter()
            .find(|update| update.disposition() == HypothesisOutcomeEvidenceDisposition::Supported)
            .unwrap();

        assert_eq!(supported.revised_confidence(), s(900));
    }

    #[test]
    fn source_state_mismatch_prevents_outcome_attribution() {
        let item = candidate(
            10,
            vec![prediction(100, 200, 800), prediction(101, 201, 800)],
        );

        let chosen = selected(&item);

        let wrong = ExperimentOutcomeObservation::new(a(999), a(10), a(200), s(900)).unwrap();

        let result = AutonomousOutcomeEvidenceUpdate::update(&item, &chosen, &wrong, policy());

        assert_eq!(
            result.status(),
            OutcomeEvidenceUpdateStatus::AttributionMismatch
        );

        assert!(result.updates().is_empty());
    }

    #[test]
    fn action_mismatch_prevents_outcome_attribution() {
        let item = candidate(
            10,
            vec![prediction(100, 200, 800), prediction(101, 201, 800)],
        );

        let chosen = selected(&item);

        let result = AutonomousOutcomeEvidenceUpdate::update(
            &item,
            &chosen,
            &observation(11, 200, 900),
            policy(),
        );

        assert_eq!(
            result.status(),
            OutcomeEvidenceUpdateStatus::AttributionMismatch
        );
    }

    #[test]
    fn low_confidence_observation_cannot_revise_hypotheses() {
        let item = candidate(
            10,
            vec![prediction(100, 200, 800), prediction(101, 201, 800)],
        );

        let chosen = selected(&item);

        let result = AutonomousOutcomeEvidenceUpdate::update(
            &item,
            &chosen,
            &observation(10, 200, 400),
            policy(),
        );

        assert_eq!(
            result.status(),
            OutcomeEvidenceUpdateStatus::ObservationConfidenceInsufficient
        );

        assert!(result.updates().is_empty());
    }

    #[test]
    fn selected_experiment_must_belong_to_candidate_being_revised() {
        let first = candidate(
            10,
            vec![prediction(100, 200, 800), prediction(101, 201, 800)],
        );

        let second = candidate(
            11,
            vec![prediction(110, 210, 800), prediction(111, 211, 800)],
        );

        let wrong_selected = selected(&second);

        let result = AutonomousOutcomeEvidenceUpdate::update(
            &first,
            &wrong_selected,
            &observation(10, 200, 900),
            policy(),
        );

        assert_eq!(
            result.status(),
            OutcomeEvidenceUpdateStatus::AttributionMismatch
        );
    }

    #[test]
    fn exact_duplicate_prediction_is_deduplicated_without_fake_evidence() {
        let duplicate = prediction(100, 200, 800);

        let item = candidate(
            10,
            vec![duplicate.clone(), duplicate, prediction(101, 201, 800)],
        );

        let chosen = selected(&item);

        let result = AutonomousOutcomeEvidenceUpdate::update(
            &item,
            &chosen,
            &observation(10, 200, 900),
            policy(),
        );

        assert_eq!(result.input_prediction_count(), 3);

        assert_eq!(result.qualifying_prediction_count(), 2);

        assert_eq!(result.updates().len(), 2);
    }

    #[test]
    fn conflicting_predictions_from_same_hypothesis_abort_update() {
        let item = HypothesisDiscriminationCandidate::new(
            experiment(10),
            vec![
                prediction(100, 200, 800),
                prediction(100, 201, 800),
                prediction(101, 202, 800),
            ],
        )
        .unwrap();

        let selected = SelectedHypothesisDiscriminatingExperiment {
            experiment: item.experiment().clone(),
            qualifying_hypothesis_count: 2,
            distinct_outcome_count: 2,
            discrimination_gain: s(1000),
        };

        let result = AutonomousOutcomeEvidenceUpdate::update(
            &item,
            &selected,
            &observation(10, 200, 900),
            policy(),
        );

        assert_eq!(
            result.status(),
            OutcomeEvidenceUpdateStatus::ConflictingPredictionEvidence
        );

        assert!(result.updates().is_empty());
    }

    #[test]
    fn low_confidence_predictions_are_excluded_from_revision() {
        let item = HypothesisDiscriminationCandidate::new(
            experiment(10),
            vec![prediction(100, 200, 800), prediction(101, 201, 400)],
        )
        .unwrap();

        let selected = SelectedHypothesisDiscriminatingExperiment {
            experiment: item.experiment().clone(),
            qualifying_hypothesis_count: 2,
            distinct_outcome_count: 2,
            discrimination_gain: s(1000),
        };

        let result = AutonomousOutcomeEvidenceUpdate::update(
            &item,
            &selected,
            &observation(10, 200, 900),
            policy(),
        );

        assert!(result.applied());

        assert_eq!(result.rejected_prediction_confidence_count(), 1);

        assert_eq!(result.qualifying_prediction_count(), 1);

        assert_eq!(result.updates().len(), 1);
    }

    #[test]
    fn support_and_contradiction_confidence_updates_are_bounded() {
        let item = candidate(
            10,
            vec![prediction(100, 200, 950), prediction(101, 201, 100)],
        );

        let selected = SelectedHypothesisDiscriminatingExperiment {
            experiment: item.experiment().clone(),
            qualifying_hypothesis_count: 2,
            distinct_outcome_count: 2,
            discrimination_gain: s(1000),
        };

        let permissive = OutcomeEvidenceUpdatePolicy::new(16, s(500), s(1), 200, 200).unwrap();

        let result = AutonomousOutcomeEvidenceUpdate::update(
            &item,
            &selected,
            &observation(10, 200, 900),
            permissive,
        );

        let support = result
            .updates()
            .iter()
            .find(|update| update.hypothesis() == &a(100))
            .unwrap();

        let contradiction = result
            .updates()
            .iter()
            .find(|update| update.hypothesis() == &a(101))
            .unwrap();

        assert_eq!(support.revised_confidence(), s(1000));

        assert_eq!(contradiction.revised_confidence(), s(0));
    }

    #[test]
    fn hard_prediction_frontier_abstains_without_partial_revision() {
        let item = candidate(
            10,
            vec![
                prediction(100, 200, 800),
                prediction(101, 201, 800),
                prediction(102, 202, 800),
            ],
        );

        let selected = SelectedHypothesisDiscriminatingExperiment {
            experiment: item.experiment().clone(),
            qualifying_hypothesis_count: 3,
            distinct_outcome_count: 3,
            discrimination_gain: s(1000),
        };

        let bounded = OutcomeEvidenceUpdatePolicy::new(2, s(500), s(500), 100, 200).unwrap();

        let result = AutonomousOutcomeEvidenceUpdate::update(
            &item,
            &selected,
            &observation(10, 200, 900),
            bounded,
        );

        assert_eq!(
            result.status(),
            OutcomeEvidenceUpdateStatus::PredictionFrontierExceeded
        );

        assert!(result.updates().is_empty());
    }

    #[test]
    fn outcome_update_is_order_invariant_non_mutating_and_facade_equivalent() {
        let predictions = vec![
            prediction(100, 200, 800),
            prediction(101, 201, 800),
            prediction(102, 202, 800),
        ];

        let item =
            HypothesisDiscriminationCandidate::new(experiment(10), predictions.clone()).unwrap();

        let mut reversed_predictions = predictions;

        reversed_predictions.reverse();

        let reversed =
            HypothesisDiscriminationCandidate::new(experiment(10), reversed_predictions).unwrap();

        let selected = SelectedHypothesisDiscriminatingExperiment {
            experiment: item.experiment().clone(),
            qualifying_hypothesis_count: 3,
            distinct_outcome_count: 3,
            discrimination_gain: s(1000),
        };

        let observation = observation(10, 201, 900);

        let before_item = item.clone();

        let before_selected = selected.clone();

        let p = policy();

        let direct = AutonomousOutcomeEvidenceUpdate::update(&item, &selected, &observation, p);

        let reordered =
            AutonomousOutcomeEvidenceUpdate::update(&reversed, &selected, &observation, p);

        let facade =
            UniversalAutonomousOutcomeEvidenceUpdate::evaluate(&item, &selected, &observation, p);

        let repeated =
            UniversalAutonomousOutcomeEvidenceUpdate::evaluate(&item, &selected, &observation, p);

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(item, before_item);
        assert_eq!(selected, before_selected);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HypothesisBeliefAvailability {
    Active,
    Suspended,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HypothesisBeliefState {
    hypothesis: CognitiveStructure,
    confidence: CognitiveSignal,
    support_count: usize,
    contradiction_count: usize,
    applied_revision_count: usize,
    availability: HypothesisBeliefAvailability,
}

impl HypothesisBeliefState {
    pub fn new(hypothesis: CognitiveStructure, confidence: CognitiveSignal) -> Option<Self> {
        if confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            hypothesis,
            confidence,
            support_count: 0,
            contradiction_count: 0,
            applied_revision_count: 0,
            availability: HypothesisBeliefAvailability::Active,
        })
    }

    pub fn hypothesis(&self) -> &CognitiveStructure {
        &self.hypothesis
    }

    pub fn confidence(&self) -> CognitiveSignal {
        self.confidence
    }

    pub fn support_count(&self) -> usize {
        self.support_count
    }

    pub fn contradiction_count(&self) -> usize {
        self.contradiction_count
    }

    pub fn applied_revision_count(&self) -> usize {
        self.applied_revision_count
    }

    pub fn availability(&self) -> HypothesisBeliefAvailability {
        self.availability
    }

    pub fn active(&self) -> bool {
        self.availability == HypothesisBeliefAvailability::Active
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HypothesisBeliefRevisionPolicy {
    max_evidence_updates: usize,
    max_belief_updates: usize,
    maximum_single_revision_delta: u16,
    minimum_active_confidence: CognitiveSignal,
    minimum_contradictions_for_suspension: usize,
}

impl HypothesisBeliefRevisionPolicy {
    pub fn new(
        max_evidence_updates: usize,
        max_belief_updates: usize,
        maximum_single_revision_delta: u16,
        minimum_active_confidence: CognitiveSignal,
        minimum_contradictions_for_suspension: usize,
    ) -> Option<Self> {
        if max_evidence_updates == 0
            || max_belief_updates == 0
            || maximum_single_revision_delta == 0
            || minimum_active_confidence == CognitiveSignal::zero()
            || minimum_contradictions_for_suspension < 2
        {
            return None;
        }

        Some(Self {
            max_evidence_updates,
            max_belief_updates,
            maximum_single_revision_delta,
            minimum_active_confidence,
            minimum_contradictions_for_suspension,
        })
    }

    pub fn max_evidence_updates(self) -> usize {
        self.max_evidence_updates
    }

    pub fn max_belief_updates(self) -> usize {
        self.max_belief_updates
    }

    pub fn maximum_single_revision_delta(self) -> u16 {
        self.maximum_single_revision_delta
    }

    pub fn minimum_active_confidence(self) -> CognitiveSignal {
        self.minimum_active_confidence
    }

    pub fn minimum_contradictions_for_suspension(self) -> usize {
        self.minimum_contradictions_for_suspension
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HypothesisBeliefRevisionStatus {
    Applied,
    EvidenceUnavailable,
    EvidenceFrontierExceeded,
    BeliefFrontierExceeded,
    ConflictingEvidenceIdentity,
    DuplicateBeliefIdentity,
    NoMatchingBeliefs,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HypothesisBeliefRevisionResult {
    status: HypothesisBeliefRevisionStatus,
    input_belief_count: usize,
    input_evidence_count: usize,
    unique_evidence_count: usize,
    matched_update_count: usize,
    unmatched_evidence_count: usize,
    beliefs: Vec<HypothesisBeliefState>,
}

impl HypothesisBeliefRevisionResult {
    pub fn status(&self) -> HypothesisBeliefRevisionStatus {
        self.status
    }

    pub fn input_belief_count(&self) -> usize {
        self.input_belief_count
    }

    pub fn input_evidence_count(&self) -> usize {
        self.input_evidence_count
    }

    pub fn unique_evidence_count(&self) -> usize {
        self.unique_evidence_count
    }

    pub fn matched_update_count(&self) -> usize {
        self.matched_update_count
    }

    pub fn unmatched_evidence_count(&self) -> usize {
        self.unmatched_evidence_count
    }

    pub fn beliefs(&self) -> &[HypothesisBeliefState] {
        &self.beliefs
    }

    pub fn applied(&self) -> bool {
        self.status == HypothesisBeliefRevisionStatus::Applied
    }

    pub fn abstained(&self) -> bool {
        !self.applied()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousHypothesisBeliefRevision;

impl AutonomousHypothesisBeliefRevision {
    fn signal(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).expect("bounded belief signal")
        }
    }

    fn belief_order(
        left: &HypothesisBeliefState,
        right: &HypothesisBeliefState,
    ) -> std::cmp::Ordering {
        format!("{:?}", left.hypothesis()).cmp(&format!("{:?}", right.hypothesis()))
    }

    fn evidence_order(
        left: &HypothesisOutcomeEvidenceUpdate,
        right: &HypothesisOutcomeEvidenceUpdate,
    ) -> std::cmp::Ordering {
        format!("{:?}", left.hypothesis())
            .cmp(&format!("{:?}", right.hypothesis()))
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    fn unchanged(
        status: HypothesisBeliefRevisionStatus,
        input_belief_count: usize,
        input_evidence_count: usize,
        unique_evidence_count: usize,
        matched_update_count: usize,
        unmatched_evidence_count: usize,
        mut beliefs: Vec<HypothesisBeliefState>,
    ) -> HypothesisBeliefRevisionResult {
        beliefs.sort_by(Self::belief_order);

        HypothesisBeliefRevisionResult {
            status,
            input_belief_count,
            input_evidence_count,
            unique_evidence_count,
            matched_update_count,
            unmatched_evidence_count,
            beliefs,
        }
    }

    pub fn revise(
        beliefs: &[HypothesisBeliefState],
        evidence: &OutcomeEvidenceUpdateResult,
        policy: HypothesisBeliefRevisionPolicy,
    ) -> HypothesisBeliefRevisionResult {
        let input_belief_count = beliefs.len();
        let input_evidence_count = evidence.updates().len();

        let mut revised = beliefs.to_vec();

        revised.sort_by(Self::belief_order);

        for index in 1..revised.len() {
            if revised[index - 1].hypothesis() == revised[index].hypothesis() {
                return Self::unchanged(
                    HypothesisBeliefRevisionStatus::DuplicateBeliefIdentity,
                    input_belief_count,
                    input_evidence_count,
                    0,
                    0,
                    0,
                    beliefs.to_vec(),
                );
            }
        }

        if !evidence.applied() {
            return Self::unchanged(
                HypothesisBeliefRevisionStatus::EvidenceUnavailable,
                input_belief_count,
                input_evidence_count,
                0,
                0,
                0,
                beliefs.to_vec(),
            );
        }

        if input_evidence_count > policy.max_evidence_updates() {
            return Self::unchanged(
                HypothesisBeliefRevisionStatus::EvidenceFrontierExceeded,
                input_belief_count,
                input_evidence_count,
                input_evidence_count,
                0,
                0,
                beliefs.to_vec(),
            );
        }

        let mut updates = evidence.updates().to_vec();

        updates.sort_by(Self::evidence_order);

        let mut canonical: Vec<HypothesisOutcomeEvidenceUpdate> = Vec::new();

        for update in updates {
            if let Some(existing) = canonical
                .iter()
                .find(|existing| existing.hypothesis() == update.hypothesis())
            {
                if existing != &update {
                    return Self::unchanged(
                        HypothesisBeliefRevisionStatus::ConflictingEvidenceIdentity,
                        input_belief_count,
                        input_evidence_count,
                        canonical.len(),
                        0,
                        0,
                        beliefs.to_vec(),
                    );
                }

                continue;
            }

            canonical.push(update);
        }

        let unique_evidence_count = canonical.len();

        let matched_update_count = canonical
            .iter()
            .filter(|update| {
                revised
                    .iter()
                    .any(|belief| belief.hypothesis() == update.hypothesis())
            })
            .count();

        let unmatched_evidence_count = unique_evidence_count.saturating_sub(matched_update_count);

        if matched_update_count > policy.max_belief_updates() {
            return Self::unchanged(
                HypothesisBeliefRevisionStatus::BeliefFrontierExceeded,
                input_belief_count,
                input_evidence_count,
                unique_evidence_count,
                matched_update_count,
                unmatched_evidence_count,
                beliefs.to_vec(),
            );
        }

        if matched_update_count == 0 {
            return Self::unchanged(
                HypothesisBeliefRevisionStatus::NoMatchingBeliefs,
                input_belief_count,
                input_evidence_count,
                unique_evidence_count,
                0,
                unmatched_evidence_count,
                beliefs.to_vec(),
            );
        }

        for update in canonical {
            let Some(belief) = revised
                .iter_mut()
                .find(|belief| belief.hypothesis() == update.hypothesis())
            else {
                continue;
            };

            let raw_delta = update
                .revised_confidence()
                .value()
                .abs_diff(update.prior_confidence().value());

            let bounded_delta = raw_delta.min(policy.maximum_single_revision_delta());

            match update.disposition() {
                HypothesisOutcomeEvidenceDisposition::Supported => {
                    belief.support_count = belief.support_count.saturating_add(1);

                    belief.confidence = Self::signal(
                        belief
                            .confidence
                            .value()
                            .saturating_add(bounded_delta)
                            .min(1000),
                    );
                }

                HypothesisOutcomeEvidenceDisposition::Contradicted => {
                    belief.contradiction_count = belief.contradiction_count.saturating_add(1);

                    belief.confidence =
                        Self::signal(belief.confidence.value().saturating_sub(bounded_delta));

                    if belief.contradiction_count >= policy.minimum_contradictions_for_suspension()
                        && belief.confidence.value() < policy.minimum_active_confidence().value()
                    {
                        belief.availability = HypothesisBeliefAvailability::Suspended;
                    }
                }
            }

            belief.applied_revision_count = belief.applied_revision_count.saturating_add(1);
        }

        revised.sort_by(Self::belief_order);

        HypothesisBeliefRevisionResult {
            status: HypothesisBeliefRevisionStatus::Applied,
            input_belief_count,
            input_evidence_count,
            unique_evidence_count,
            matched_update_count,
            unmatched_evidence_count,
            beliefs: revised,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousHypothesisBeliefRevision;

impl UniversalAutonomousHypothesisBeliefRevision {
    pub fn evaluate(
        beliefs: &[HypothesisBeliefState],
        evidence: &OutcomeEvidenceUpdateResult,
        policy: HypothesisBeliefRevisionPolicy,
    ) -> HypothesisBeliefRevisionResult {
        AutonomousHypothesisBeliefRevision::revise(beliefs, evidence, policy)
    }
}

#[cfg(test)]
mod hypothesis_belief_revision_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn belief(hypothesis: u64, confidence: u16) -> HypothesisBeliefState {
        HypothesisBeliefState::new(a(hypothesis), s(confidence)).unwrap()
    }

    fn update(
        hypothesis: u64,
        prior: u16,
        revised: u16,
        disposition: HypothesisOutcomeEvidenceDisposition,
    ) -> HypothesisOutcomeEvidenceUpdate {
        HypothesisOutcomeEvidenceUpdate {
            hypothesis: a(hypothesis),
            predicted_outcome: a(hypothesis + 100),
            prior_confidence: s(prior),
            revised_confidence: s(revised),
            disposition,
        }
    }

    fn evidence(updates: Vec<HypothesisOutcomeEvidenceUpdate>) -> OutcomeEvidenceUpdateResult {
        let supported_count = updates
            .iter()
            .filter(|update| {
                update.disposition() == HypothesisOutcomeEvidenceDisposition::Supported
            })
            .count();

        let contradicted_count = updates.len().saturating_sub(supported_count);

        OutcomeEvidenceUpdateResult {
            status: OutcomeEvidenceUpdateStatus::Applied,
            input_prediction_count: updates.len(),
            qualifying_prediction_count: updates.len(),
            rejected_prediction_confidence_count: 0,
            supported_count,
            contradicted_count,
            updates,
        }
    }

    fn unavailable() -> OutcomeEvidenceUpdateResult {
        OutcomeEvidenceUpdateResult {
            status: OutcomeEvidenceUpdateStatus::ObservationConfidenceInsufficient,
            input_prediction_count: 0,
            qualifying_prediction_count: 0,
            rejected_prediction_confidence_count: 0,
            supported_count: 0,
            contradicted_count: 0,
            updates: Vec::new(),
        }
    }

    fn policy() -> HypothesisBeliefRevisionPolicy {
        HypothesisBeliefRevisionPolicy::new(16, 16, 100, s(550), 2).unwrap()
    }

    #[test]
    fn belief_revision_policy_and_initial_state_require_grounded_positive_contract() {
        assert_eq!(HypothesisBeliefState::new(a(1), s(0),), None);

        assert_eq!(
            HypothesisBeliefRevisionPolicy::new(0, 1, 100, s(500), 2,),
            None
        );

        assert_eq!(
            HypothesisBeliefRevisionPolicy::new(1, 1, 100, s(500), 1,),
            None
        );

        let state = belief(1, 700);

        assert!(state.active());
        assert_eq!(state.support_count(), 0);
        assert_eq!(state.contradiction_count(), 0);
    }

    #[test]
    fn support_moves_persistent_belief_by_at_most_single_revision_cap() {
        let result = AutonomousHypothesisBeliefRevision::revise(
            &[belief(1, 500)],
            &evidence(vec![update(
                1,
                500,
                900,
                HypothesisOutcomeEvidenceDisposition::Supported,
            )]),
            policy(),
        );

        assert!(result.applied());

        assert_eq!(result.beliefs()[0].confidence(), s(600));

        assert_eq!(result.beliefs()[0].support_count(), 1);
    }

    #[test]
    fn one_contradiction_cannot_silently_suspend_hypothesis() {
        let result = AutonomousHypothesisBeliefRevision::revise(
            &[belief(1, 600)],
            &evidence(vec![update(
                1,
                800,
                500,
                HypothesisOutcomeEvidenceDisposition::Contradicted,
            )]),
            policy(),
        );

        assert_eq!(result.beliefs()[0].confidence(), s(500));

        assert_eq!(result.beliefs()[0].contradiction_count(), 1);

        assert_eq!(
            result.beliefs()[0].availability(),
            HypothesisBeliefAvailability::Active
        );
    }

    #[test]
    fn repeated_contradictions_can_suspend_low_confidence_hypothesis() {
        let first = AutonomousHypothesisBeliefRevision::revise(
            &[belief(1, 700)],
            &evidence(vec![update(
                1,
                800,
                600,
                HypothesisOutcomeEvidenceDisposition::Contradicted,
            )]),
            policy(),
        );

        let second = AutonomousHypothesisBeliefRevision::revise(
            first.beliefs(),
            &evidence(vec![update(
                1,
                800,
                600,
                HypothesisOutcomeEvidenceDisposition::Contradicted,
            )]),
            policy(),
        );

        assert_eq!(second.beliefs()[0].confidence(), s(500));

        assert_eq!(second.beliefs()[0].contradiction_count(), 2);

        assert_eq!(
            second.beliefs()[0].availability(),
            HypothesisBeliefAvailability::Suspended
        );
    }

    #[test]
    fn later_support_does_not_implicitly_reactivate_suspended_hypothesis() {
        let mut suspended = belief(1, 500);

        suspended.contradiction_count = 2;
        suspended.applied_revision_count = 2;
        suspended.availability = HypothesisBeliefAvailability::Suspended;

        let result = AutonomousHypothesisBeliefRevision::revise(
            &[suspended],
            &evidence(vec![update(
                1,
                500,
                900,
                HypothesisOutcomeEvidenceDisposition::Supported,
            )]),
            policy(),
        );

        assert_eq!(result.beliefs()[0].confidence(), s(600));

        assert_eq!(
            result.beliefs()[0].availability(),
            HypothesisBeliefAvailability::Suspended
        );
    }

    #[test]
    fn unavailable_outcome_evidence_cannot_modify_belief_memory() {
        let beliefs = vec![belief(1, 700), belief(2, 700)];

        let result = AutonomousHypothesisBeliefRevision::revise(&beliefs, &unavailable(), policy());

        assert_eq!(
            result.status(),
            HypothesisBeliefRevisionStatus::EvidenceUnavailable
        );

        assert_eq!(result.beliefs(), beliefs.as_slice());
    }

    #[test]
    fn hard_evidence_frontier_abstains_without_partial_belief_revision() {
        let beliefs = vec![belief(1, 700), belief(2, 700), belief(3, 700)];

        let evidence = evidence(vec![
            update(
                1,
                700,
                600,
                HypothesisOutcomeEvidenceDisposition::Contradicted,
            ),
            update(
                2,
                700,
                600,
                HypothesisOutcomeEvidenceDisposition::Contradicted,
            ),
            update(
                3,
                700,
                600,
                HypothesisOutcomeEvidenceDisposition::Contradicted,
            ),
        ]);

        let bounded = HypothesisBeliefRevisionPolicy::new(2, 16, 100, s(550), 2).unwrap();

        let result = AutonomousHypothesisBeliefRevision::revise(&beliefs, &evidence, bounded);

        assert_eq!(
            result.status(),
            HypothesisBeliefRevisionStatus::EvidenceFrontierExceeded
        );

        assert_eq!(result.beliefs(), beliefs.as_slice());
    }

    #[test]
    fn hard_belief_update_frontier_abstains_without_partial_revision() {
        let beliefs = vec![belief(1, 700), belief(2, 700)];

        let evidence = evidence(vec![
            update(1, 700, 800, HypothesisOutcomeEvidenceDisposition::Supported),
            update(2, 700, 800, HypothesisOutcomeEvidenceDisposition::Supported),
        ]);

        let bounded = HypothesisBeliefRevisionPolicy::new(16, 1, 100, s(550), 2).unwrap();

        let result = AutonomousHypothesisBeliefRevision::revise(&beliefs, &evidence, bounded);

        assert_eq!(
            result.status(),
            HypothesisBeliefRevisionStatus::BeliefFrontierExceeded
        );

        assert_eq!(result.beliefs(), beliefs.as_slice());
    }

    #[test]
    fn unmatched_evidence_cannot_invent_new_persistent_hypothesis() {
        let beliefs = vec![belief(1, 700)];

        let result = AutonomousHypothesisBeliefRevision::revise(
            &beliefs,
            &evidence(vec![update(
                99,
                700,
                900,
                HypothesisOutcomeEvidenceDisposition::Supported,
            )]),
            policy(),
        );

        assert_eq!(
            result.status(),
            HypothesisBeliefRevisionStatus::NoMatchingBeliefs
        );

        assert_eq!(result.beliefs().len(), 1);

        assert_eq!(result.unmatched_evidence_count(), 1);
    }

    #[test]
    fn conflicting_evidence_for_same_hypothesis_abstains_atomically() {
        let beliefs = vec![belief(1, 700)];

        let conflicting = evidence(vec![
            update(1, 700, 800, HypothesisOutcomeEvidenceDisposition::Supported),
            update(
                1,
                700,
                600,
                HypothesisOutcomeEvidenceDisposition::Contradicted,
            ),
        ]);

        let result = AutonomousHypothesisBeliefRevision::revise(&beliefs, &conflicting, policy());

        assert_eq!(
            result.status(),
            HypothesisBeliefRevisionStatus::ConflictingEvidenceIdentity
        );

        assert_eq!(result.beliefs(), beliefs.as_slice());
    }

    #[test]
    fn duplicate_persistent_belief_identity_is_rejected_before_revision() {
        let beliefs = vec![belief(1, 700), belief(1, 600)];

        let result = AutonomousHypothesisBeliefRevision::revise(
            &beliefs,
            &evidence(vec![update(
                1,
                700,
                800,
                HypothesisOutcomeEvidenceDisposition::Supported,
            )]),
            policy(),
        );

        assert_eq!(
            result.status(),
            HypothesisBeliefRevisionStatus::DuplicateBeliefIdentity
        );

        assert_eq!(result.matched_update_count(), 0);
    }

    #[test]
    fn belief_revision_is_order_invariant_non_mutating_and_facade_equivalent() {
        let beliefs = vec![belief(2, 700), belief(1, 700)];

        let before = beliefs.clone();

        let evidence = evidence(vec![
            update(
                2,
                700,
                600,
                HypothesisOutcomeEvidenceDisposition::Contradicted,
            ),
            update(1, 700, 800, HypothesisOutcomeEvidenceDisposition::Supported),
        ]);

        let mut reversed = beliefs.clone();

        reversed.reverse();

        let p = policy();

        let direct = AutonomousHypothesisBeliefRevision::revise(&beliefs, &evidence, p);

        let reordered = AutonomousHypothesisBeliefRevision::revise(&reversed, &evidence, p);

        let facade = UniversalAutonomousHypothesisBeliefRevision::evaluate(&beliefs, &evidence, p);

        let repeated =
            UniversalAutonomousHypothesisBeliefRevision::evaluate(&beliefs, &evidence, p);

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(beliefs, before);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SequentialExperimentControlPolicy {
    discrimination: HypothesisDiscriminationPolicy,
    max_beliefs: usize,
    max_experiment_cycles: usize,
    minimum_winner_confidence: CognitiveSignal,
    minimum_resolution_margin: CognitiveSignal,
}

impl SequentialExperimentControlPolicy {
    pub fn new(
        discrimination: HypothesisDiscriminationPolicy,
        max_beliefs: usize,
        max_experiment_cycles: usize,
        minimum_winner_confidence: CognitiveSignal,
        minimum_resolution_margin: CognitiveSignal,
    ) -> Option<Self> {
        if max_beliefs < 2
            || max_experiment_cycles == 0
            || minimum_winner_confidence == CognitiveSignal::zero()
            || minimum_resolution_margin == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            discrimination,
            max_beliefs,
            max_experiment_cycles,
            minimum_winner_confidence,
            minimum_resolution_margin,
        })
    }

    pub fn discrimination(self) -> HypothesisDiscriminationPolicy {
        self.discrimination
    }

    pub fn max_beliefs(self) -> usize {
        self.max_beliefs
    }

    pub fn max_experiment_cycles(self) -> usize {
        self.max_experiment_cycles
    }

    pub fn minimum_winner_confidence(self) -> CognitiveSignal {
        self.minimum_winner_confidence
    }

    pub fn minimum_resolution_margin(self) -> CognitiveSignal {
        self.minimum_resolution_margin
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequentialExperimentDecision {
    ContinueExperimentation,
    StopResolved,
    StopExperimentBudgetExhausted,
    StopNoDiscriminatingExperiment,
    AbstainBeliefFrontierExceeded,
    AbstainDuplicateBeliefIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SequentialExperimentControlResult {
    decision: SequentialExperimentDecision,
    input_belief_count: usize,
    active_belief_count: usize,
    eligible_candidate_count: usize,
    current_experiment_cycle: usize,
    resolved_winner: Option<CognitiveStructure>,
    discrimination: Option<HypothesisDiscriminationResult>,
    next_experiment: Option<SelectedHypothesisDiscriminatingExperiment>,
}

impl SequentialExperimentControlResult {
    pub fn decision(&self) -> SequentialExperimentDecision {
        self.decision
    }

    pub fn input_belief_count(&self) -> usize {
        self.input_belief_count
    }

    pub fn active_belief_count(&self) -> usize {
        self.active_belief_count
    }

    pub fn eligible_candidate_count(&self) -> usize {
        self.eligible_candidate_count
    }

    pub fn current_experiment_cycle(&self) -> usize {
        self.current_experiment_cycle
    }

    pub fn resolved_winner(&self) -> Option<&CognitiveStructure> {
        self.resolved_winner.as_ref()
    }

    pub fn discrimination(&self) -> Option<&HypothesisDiscriminationResult> {
        self.discrimination.as_ref()
    }

    pub fn next_experiment(&self) -> Option<&SelectedHypothesisDiscriminatingExperiment> {
        self.next_experiment.as_ref()
    }

    pub fn continuing(&self) -> bool {
        self.decision == SequentialExperimentDecision::ContinueExperimentation
    }

    pub fn stopped(&self) -> bool {
        matches!(
            self.decision,
            SequentialExperimentDecision::StopResolved
                | SequentialExperimentDecision::StopExperimentBudgetExhausted
                | SequentialExperimentDecision::StopNoDiscriminatingExperiment
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousSequentialExperimentControl;

impl AutonomousSequentialExperimentControl {
    fn belief_order(
        left: &HypothesisBeliefState,
        right: &HypothesisBeliefState,
    ) -> std::cmp::Ordering {
        right
            .confidence()
            .value()
            .cmp(&left.confidence().value())
            .then_with(|| {
                format!("{:?}", left.hypothesis()).cmp(&format!("{:?}", right.hypothesis()))
            })
    }

    fn base_result(
        decision: SequentialExperimentDecision,
        input_belief_count: usize,
        active_belief_count: usize,
        eligible_candidate_count: usize,
        current_experiment_cycle: usize,
        resolved_winner: Option<CognitiveStructure>,
    ) -> SequentialExperimentControlResult {
        SequentialExperimentControlResult {
            decision,
            input_belief_count,
            active_belief_count,
            eligible_candidate_count,
            current_experiment_cycle,
            resolved_winner,
            discrimination: None,
            next_experiment: None,
        }
    }

    pub fn control(
        beliefs: &[HypothesisBeliefState],
        candidates: &[HypothesisDiscriminationCandidate],
        current_experiment_cycle: usize,
        policy: SequentialExperimentControlPolicy,
    ) -> SequentialExperimentControlResult {
        let input_belief_count = beliefs.len();

        if input_belief_count > policy.max_beliefs() {
            return Self::base_result(
                SequentialExperimentDecision::AbstainBeliefFrontierExceeded,
                input_belief_count,
                0,
                0,
                current_experiment_cycle,
                None,
            );
        }

        let mut canonical_beliefs = beliefs.to_vec();

        canonical_beliefs.sort_by(|left, right| {
            format!("{:?}", left.hypothesis()).cmp(&format!("{:?}", right.hypothesis()))
        });

        for index in 1..canonical_beliefs.len() {
            if canonical_beliefs[index - 1].hypothesis() == canonical_beliefs[index].hypothesis() {
                return Self::base_result(
                    SequentialExperimentDecision::AbstainDuplicateBeliefIdentity,
                    input_belief_count,
                    0,
                    0,
                    current_experiment_cycle,
                    None,
                );
            }
        }

        let mut active: Vec<HypothesisBeliefState> = canonical_beliefs
            .into_iter()
            .filter(HypothesisBeliefState::active)
            .collect();

        active.sort_by(Self::belief_order);

        let active_belief_count = active.len();

        if active_belief_count < 2 {
            return Self::base_result(
                SequentialExperimentDecision::StopResolved,
                input_belief_count,
                active_belief_count,
                0,
                current_experiment_cycle,
                active.first().map(|belief| belief.hypothesis().clone()),
            );
        }

        let winner = &active[0];

        let runner_up = &active[1];

        let confidence_margin = winner
            .confidence()
            .value()
            .saturating_sub(runner_up.confidence().value());

        if winner.confidence().value() >= policy.minimum_winner_confidence().value()
            && confidence_margin >= policy.minimum_resolution_margin().value()
        {
            return Self::base_result(
                SequentialExperimentDecision::StopResolved,
                input_belief_count,
                active_belief_count,
                0,
                current_experiment_cycle,
                Some(winner.hypothesis().clone()),
            );
        }

        if current_experiment_cycle >= policy.max_experiment_cycles() {
            return Self::base_result(
                SequentialExperimentDecision::StopExperimentBudgetExhausted,
                input_belief_count,
                active_belief_count,
                0,
                current_experiment_cycle,
                None,
            );
        }

        let prediction_threshold = policy
            .discrimination()
            .thresholds()
            .minimum_prediction_confidence();

        let mut eligible = Vec::new();

        for candidate in candidates {
            let mut filtered_predictions = Vec::new();

            for prediction in candidate.predictions() {
                if prediction.confidence().value() < prediction_threshold.value() {
                    continue;
                }

                if active
                    .iter()
                    .any(|belief| belief.hypothesis() == prediction.hypothesis())
                {
                    filtered_predictions.push(prediction.clone());
                }
            }

            let mut distinct_hypotheses: Vec<CognitiveStructure> = Vec::new();

            for prediction in &filtered_predictions {
                if !distinct_hypotheses.contains(prediction.hypothesis()) {
                    distinct_hypotheses.push(prediction.hypothesis().clone());
                }
            }

            if distinct_hypotheses.len() < 2 {
                continue;
            }

            if let Some(filtered) = HypothesisDiscriminationCandidate::new(
                candidate.experiment().clone(),
                filtered_predictions,
            ) {
                eligible.push(filtered);
            }
        }

        let eligible_candidate_count = eligible.len();

        if eligible.is_empty() {
            return Self::base_result(
                SequentialExperimentDecision::StopNoDiscriminatingExperiment,
                input_belief_count,
                active_belief_count,
                0,
                current_experiment_cycle,
                None,
            );
        }

        let discrimination =
            AutonomousHypothesisDiscrimination::discriminate(&eligible, policy.discrimination());

        let Some(next_experiment) = discrimination.selected().first().cloned() else {
            return SequentialExperimentControlResult {
                decision: SequentialExperimentDecision::StopNoDiscriminatingExperiment,
                input_belief_count,
                active_belief_count,
                eligible_candidate_count,
                current_experiment_cycle,
                resolved_winner: None,
                discrimination: Some(discrimination),
                next_experiment: None,
            };
        };

        SequentialExperimentControlResult {
            decision: SequentialExperimentDecision::ContinueExperimentation,
            input_belief_count,
            active_belief_count,
            eligible_candidate_count,
            current_experiment_cycle,
            resolved_winner: None,
            discrimination: Some(discrimination),
            next_experiment: Some(next_experiment),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousSequentialExperimentControl;

impl UniversalAutonomousSequentialExperimentControl {
    pub fn evaluate(
        beliefs: &[HypothesisBeliefState],
        candidates: &[HypothesisDiscriminationCandidate],
        current_experiment_cycle: usize,
        policy: SequentialExperimentControlPolicy,
    ) -> SequentialExperimentControlResult {
        AutonomousSequentialExperimentControl::control(
            beliefs,
            candidates,
            current_experiment_cycle,
            policy,
        )
    }
}

#[cfg(test)]
mod sequential_experiment_control_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn belief(hypothesis: u64, confidence: u16) -> HypothesisBeliefState {
        HypothesisBeliefState::new(a(hypothesis), s(confidence)).unwrap()
    }

    fn suspended_belief(hypothesis: u64, confidence: u16) -> HypothesisBeliefState {
        let mut state = belief(hypothesis, confidence);

        state.availability = HypothesisBeliefAvailability::Suspended;

        state
    }

    fn experiment(action: u64, information: u16) -> AutonomousExperimentProposal {
        AutonomousExperimentProposal::new(
            a(1),
            a(action),
            a(900),
            ExperimentEvidence::new(s(800), s(information), s(900), s(900), s(100)).unwrap(),
        )
    }

    fn prediction(hypothesis: u64, outcome: u64, confidence: u16) -> CompetingHypothesisPrediction {
        CompetingHypothesisPrediction::new(a(hypothesis), a(outcome), s(confidence)).unwrap()
    }

    fn candidate(
        action: u64,
        information: u16,
        predictions: Vec<CompetingHypothesisPrediction>,
    ) -> HypothesisDiscriminationCandidate {
        HypothesisDiscriminationCandidate::new(experiment(action, information), predictions)
            .unwrap()
    }

    fn discrimination_policy() -> HypothesisDiscriminationPolicy {
        HypothesisDiscriminationPolicy::new(
            ActiveExperimentPolicy::new(
                ActiveExperimentBounds::new(32, 32, 32).unwrap(),
                ActiveExperimentThresholds::new(s(500), s(500), s(500), s(500)).unwrap(),
            ),
            HypothesisDiscriminationBounds::new(32, 32, 32, 32).unwrap(),
            HypothesisDiscriminationThresholds::new(2, 2, s(500), s(500)).unwrap(),
        )
    }

    fn policy() -> SequentialExperimentControlPolicy {
        SequentialExperimentControlPolicy::new(discrimination_policy(), 16, 8, s(850), s(250))
            .unwrap()
    }

    fn two_way_candidate(
        action: u64,
        information: u16,
        left: u64,
        right: u64,
    ) -> HypothesisDiscriminationCandidate {
        candidate(
            action,
            information,
            vec![
                prediction(left, left + 100, 900),
                prediction(right, right + 100, 900),
            ],
        )
    }

    #[test]
    fn sequential_control_policy_requires_positive_bounded_resolution_contract() {
        assert_eq!(
            SequentialExperimentControlPolicy::new(discrimination_policy(), 1, 8, s(850), s(250),),
            None
        );

        assert_eq!(
            SequentialExperimentControlPolicy::new(discrimination_policy(), 16, 0, s(850), s(250),),
            None
        );

        assert_eq!(
            SequentialExperimentControlPolicy::new(discrimination_policy(), 16, 8, s(0), s(250),),
            None
        );
    }

    #[test]
    fn single_active_hypothesis_stops_as_resolved() {
        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 700), suspended_belief(2, 800)],
            &[],
            0,
            policy(),
        );

        assert_eq!(
            result.decision(),
            SequentialExperimentDecision::StopResolved
        );

        assert_eq!(result.active_belief_count(), 1);

        assert_eq!(result.resolved_winner(), Some(&a(1)));
    }

    #[test]
    fn dominant_high_confidence_belief_stops_without_extra_experiment() {
        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 900), belief(2, 600)],
            &[two_way_candidate(10, 900, 1, 2)],
            0,
            policy(),
        );

        assert_eq!(
            result.decision(),
            SequentialExperimentDecision::StopResolved
        );

        assert_eq!(result.resolved_winner(), Some(&a(1)));

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn unresolved_close_beliefs_continue_with_discriminating_experiment() {
        let item = two_way_candidate(10, 900, 1, 2);

        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 700), belief(2, 650)],
            std::slice::from_ref(&item),
            0,
            policy(),
        );

        assert!(result.continuing());

        assert_eq!(result.eligible_candidate_count(), 1);

        assert_eq!(
            result.next_experiment().unwrap().experiment(),
            item.experiment()
        );
    }

    #[test]
    fn experiment_cycle_budget_stops_before_selecting_another_intervention() {
        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 700), belief(2, 650)],
            &[two_way_candidate(10, 900, 1, 2)],
            8,
            policy(),
        );

        assert_eq!(
            result.decision(),
            SequentialExperimentDecision::StopExperimentBudgetExhausted
        );

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn suspended_hypotheses_are_excluded_from_live_competition() {
        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 700), suspended_belief(2, 900), belief(3, 680)],
            &[two_way_candidate(10, 900, 1, 3)],
            0,
            policy(),
        );

        assert_eq!(result.active_belief_count(), 2);

        assert!(result.continuing());
    }

    #[test]
    fn candidate_separating_only_one_active_belief_from_suspended_belief_is_ineligible() {
        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 700), belief(2, 680), suspended_belief(3, 900)],
            &[two_way_candidate(10, 900, 1, 3)],
            0,
            policy(),
        );

        assert_eq!(
            result.decision(),
            SequentialExperimentDecision::StopNoDiscriminatingExperiment
        );

        assert_eq!(result.eligible_candidate_count(), 0);
    }

    #[test]
    fn identical_predictions_for_active_beliefs_stop_when_no_discrimination_survives() {
        let item = candidate(
            10,
            900,
            vec![prediction(1, 100, 900), prediction(2, 100, 900)],
        );

        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 700), belief(2, 680)],
            &[item],
            0,
            policy(),
        );

        assert_eq!(
            result.decision(),
            SequentialExperimentDecision::StopNoDiscriminatingExperiment
        );

        assert!(result.discrimination().is_some());

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn duplicate_persistent_belief_identity_abstains_before_experiment_control() {
        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 700), belief(1, 650)],
            &[two_way_candidate(10, 900, 1, 2)],
            0,
            policy(),
        );

        assert_eq!(
            result.decision(),
            SequentialExperimentDecision::AbstainDuplicateBeliefIdentity
        );

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn belief_frontier_overflow_abstains_atomically() {
        let bounded =
            SequentialExperimentControlPolicy::new(discrimination_policy(), 2, 8, s(850), s(250))
                .unwrap();

        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 700), belief(2, 680), belief(3, 660)],
            &[two_way_candidate(10, 900, 1, 2)],
            0,
            bounded,
        );

        assert_eq!(
            result.decision(),
            SequentialExperimentDecision::AbstainBeliefFrontierExceeded
        );

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn next_experiment_prefers_stronger_active_hypothesis_discrimination_gain() {
        let partial = candidate(
            10,
            950,
            vec![
                prediction(1, 100, 900),
                prediction(2, 100, 900),
                prediction(3, 101, 900),
            ],
        );

        let full = candidate(
            11,
            700,
            vec![prediction(1, 110, 900), prediction(2, 111, 900)],
        );

        let result = AutonomousSequentialExperimentControl::control(
            &[belief(1, 700), belief(2, 680), belief(3, 660)],
            &[partial, full.clone()],
            0,
            policy(),
        );

        assert!(result.continuing());

        assert_eq!(
            result.next_experiment().unwrap().experiment(),
            full.experiment()
        );

        assert_eq!(
            result.next_experiment().unwrap().discrimination_gain(),
            s(1000)
        );
    }

    #[test]
    fn sequential_control_is_order_invariant_non_mutating_and_facade_equivalent() {
        let beliefs = vec![belief(2, 680), belief(1, 700), belief(3, 660)];

        let candidates = vec![
            two_way_candidate(10, 900, 1, 2),
            two_way_candidate(11, 800, 2, 3),
        ];

        let before_beliefs = beliefs.clone();

        let before_candidates = candidates.clone();

        let mut reversed_beliefs = beliefs.clone();

        reversed_beliefs.reverse();

        let mut reversed_candidates = candidates.clone();

        reversed_candidates.reverse();

        let p = policy();

        let direct = AutonomousSequentialExperimentControl::control(&beliefs, &candidates, 1, p);

        let reordered = AutonomousSequentialExperimentControl::control(
            &reversed_beliefs,
            &reversed_candidates,
            1,
            p,
        );

        let facade =
            UniversalAutonomousSequentialExperimentControl::evaluate(&beliefs, &candidates, 1, p);

        let repeated =
            UniversalAutonomousSequentialExperimentControl::evaluate(&beliefs, &candidates, 1, p);

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(beliefs, before_beliefs);
        assert_eq!(candidates, before_candidates);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedExperimentPossibility {
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    predictions: Vec<CompetingHypothesisPrediction>,
    controllability: CognitiveSignal,
    grounding_confidence: CognitiveSignal,
    execution_cost: CognitiveSignal,
}

impl GroundedExperimentPossibility {
    pub fn new(
        source_state: CognitiveStructure,
        action: CognitiveStructure,
        predictions: Vec<CompetingHypothesisPrediction>,
        controllability: CognitiveSignal,
        grounding_confidence: CognitiveSignal,
        execution_cost: CognitiveSignal,
    ) -> Option<Self> {
        if predictions.is_empty()
            || controllability == CognitiveSignal::zero()
            || grounding_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            source_state,
            action,
            predictions,
            controllability,
            grounding_confidence,
            execution_cost,
        })
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn predictions(&self) -> &[CompetingHypothesisPrediction] {
        &self.predictions
    }

    pub fn controllability(&self) -> CognitiveSignal {
        self.controllability
    }

    pub fn grounding_confidence(&self) -> CognitiveSignal {
        self.grounding_confidence
    }

    pub fn execution_cost(&self) -> CognitiveSignal {
        self.execution_cost
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BeliefDrivenExperimentProposalBounds {
    max_beliefs: usize,
    max_possibilities: usize,
    max_predictions_per_possibility: usize,
    max_generated_candidates: usize,
}

impl BeliefDrivenExperimentProposalBounds {
    pub fn new(
        max_beliefs: usize,
        max_possibilities: usize,
        max_predictions_per_possibility: usize,
        max_generated_candidates: usize,
    ) -> Option<Self> {
        if max_beliefs < 2
            || max_possibilities == 0
            || max_predictions_per_possibility == 0
            || max_generated_candidates == 0
        {
            return None;
        }

        Some(Self {
            max_beliefs,
            max_possibilities,
            max_predictions_per_possibility,
            max_generated_candidates,
        })
    }

    pub fn max_beliefs(self) -> usize {
        self.max_beliefs
    }

    pub fn max_possibilities(self) -> usize {
        self.max_possibilities
    }

    pub fn max_predictions_per_possibility(self) -> usize {
        self.max_predictions_per_possibility
    }

    pub fn max_generated_candidates(self) -> usize {
        self.max_generated_candidates
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BeliefDrivenExperimentProposalPolicy {
    foundation: ActiveExperimentPolicy,
    bounds: BeliefDrivenExperimentProposalBounds,
    minimum_active_belief_confidence: CognitiveSignal,
    minimum_prediction_confidence: CognitiveSignal,
}

impl BeliefDrivenExperimentProposalPolicy {
    pub fn new(
        foundation: ActiveExperimentPolicy,
        bounds: BeliefDrivenExperimentProposalBounds,
        minimum_active_belief_confidence: CognitiveSignal,
        minimum_prediction_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_active_belief_confidence == CognitiveSignal::zero()
            || minimum_prediction_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            foundation,
            bounds,
            minimum_active_belief_confidence,
            minimum_prediction_confidence,
        })
    }

    pub fn foundation(self) -> ActiveExperimentPolicy {
        self.foundation
    }

    pub fn bounds(self) -> BeliefDrivenExperimentProposalBounds {
        self.bounds
    }

    pub fn minimum_active_belief_confidence(self) -> CognitiveSignal {
        self.minimum_active_belief_confidence
    }

    pub fn minimum_prediction_confidence(self) -> CognitiveSignal {
        self.minimum_prediction_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BeliefDrivenExperimentProposalStatus {
    Generated,
    NoActiveCompetition,
    NoDiscriminatingPossibility,
    BeliefFrontierExceeded,
    PossibilityFrontierExceeded,
    DuplicateBeliefIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeliefDrivenExperimentProposalResult {
    status: BeliefDrivenExperimentProposalStatus,
    input_belief_count: usize,
    active_belief_count: usize,
    input_possibility_count: usize,
    evaluated_possibility_count: usize,
    rejected_prediction_frontier_count: usize,
    rejected_conflicting_prediction_count: usize,
    rejected_competition_count: usize,
    rejected_foundation_count: usize,
    generated_before_frontier: usize,
    generation_frontier_truncated: bool,
    generated: Vec<HypothesisDiscriminationCandidate>,
}

impl BeliefDrivenExperimentProposalResult {
    pub fn status(&self) -> BeliefDrivenExperimentProposalStatus {
        self.status
    }

    pub fn input_belief_count(&self) -> usize {
        self.input_belief_count
    }

    pub fn active_belief_count(&self) -> usize {
        self.active_belief_count
    }

    pub fn input_possibility_count(&self) -> usize {
        self.input_possibility_count
    }

    pub fn evaluated_possibility_count(&self) -> usize {
        self.evaluated_possibility_count
    }

    pub fn rejected_prediction_frontier_count(&self) -> usize {
        self.rejected_prediction_frontier_count
    }

    pub fn rejected_conflicting_prediction_count(&self) -> usize {
        self.rejected_conflicting_prediction_count
    }

    pub fn rejected_competition_count(&self) -> usize {
        self.rejected_competition_count
    }

    pub fn rejected_foundation_count(&self) -> usize {
        self.rejected_foundation_count
    }

    pub fn generated_before_frontier(&self) -> usize {
        self.generated_before_frontier
    }

    pub fn generation_frontier_truncated(&self) -> bool {
        self.generation_frontier_truncated
    }

    pub fn generated(&self) -> &[HypothesisDiscriminationCandidate] {
        &self.generated
    }

    pub fn generated_count(&self) -> usize {
        self.generated.len()
    }

    pub fn generated_any(&self) -> bool {
        !self.generated.is_empty()
    }

    pub fn abstained(&self) -> bool {
        !self.generated_any()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousBeliefDrivenExperimentProposal;

impl AutonomousBeliefDrivenExperimentProposal {
    fn signal(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).expect("bounded proposal signal")
        }
    }

    fn empty(
        status: BeliefDrivenExperimentProposalStatus,
        input_belief_count: usize,
        active_belief_count: usize,
        input_possibility_count: usize,
    ) -> BeliefDrivenExperimentProposalResult {
        BeliefDrivenExperimentProposalResult {
            status,
            input_belief_count,
            active_belief_count,
            input_possibility_count,
            evaluated_possibility_count: 0,
            rejected_prediction_frontier_count: 0,
            rejected_conflicting_prediction_count: 0,
            rejected_competition_count: 0,
            rejected_foundation_count: 0,
            generated_before_frontier: 0,
            generation_frontier_truncated: false,
            generated: Vec::new(),
        }
    }

    fn candidate_order(
        left: &HypothesisDiscriminationCandidate,
        right: &HypothesisDiscriminationCandidate,
    ) -> std::cmp::Ordering {
        let left_evidence = left.experiment().evidence();

        let right_evidence = right.experiment().evidence();

        right_evidence
            .expected_information_gain()
            .value()
            .cmp(&left_evidence.expected_information_gain().value())
            .then_with(|| {
                right_evidence
                    .prediction_uncertainty()
                    .value()
                    .cmp(&left_evidence.prediction_uncertainty().value())
            })
            .then_with(|| {
                right_evidence
                    .controllability()
                    .value()
                    .cmp(&left_evidence.controllability().value())
            })
            .then_with(|| {
                right_evidence
                    .grounding_confidence()
                    .value()
                    .cmp(&left_evidence.grounding_confidence().value())
            })
            .then_with(|| {
                left_evidence
                    .execution_cost()
                    .value()
                    .cmp(&right_evidence.execution_cost().value())
            })
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    fn information_gain(predictions: &[CompetingHypothesisPrediction]) -> CognitiveSignal {
        if predictions.len() < 2 {
            return CognitiveSignal::zero();
        }

        let mut total_pairs = 0usize;
        let mut disagreeing_pairs = 0usize;

        for left in 0..predictions.len() {
            for right in (left + 1)..predictions.len() {
                total_pairs += 1;

                if predictions[left].predicted_outcome() != predictions[right].predicted_outcome() {
                    disagreeing_pairs += 1;
                }
            }
        }

        if total_pairs == 0 {
            return CognitiveSignal::zero();
        }

        let value = (disagreeing_pairs.saturating_mul(1000) / total_pairs).min(1000) as u16;

        Self::signal(value)
    }

    fn uncertainty(relevant_beliefs: &[HypothesisBeliefState]) -> CognitiveSignal {
        if relevant_beliefs.len() < 2 {
            return CognitiveSignal::zero();
        }

        let mut confidences: Vec<u16> = relevant_beliefs
            .iter()
            .map(|belief| belief.confidence().value())
            .collect();

        confidences.sort_by(|left, right| right.cmp(left));

        let margin = confidences[0].saturating_sub(confidences[1]);

        let value = 1000u16.saturating_sub(margin).max(1);

        Self::signal(value)
    }

    pub fn generate(
        beliefs: &[HypothesisBeliefState],
        possibilities: &[GroundedExperimentPossibility],
        policy: BeliefDrivenExperimentProposalPolicy,
    ) -> BeliefDrivenExperimentProposalResult {
        let bounds = policy.bounds();

        let input_belief_count = beliefs.len();

        let input_possibility_count = possibilities.len();

        if input_belief_count > bounds.max_beliefs() {
            return Self::empty(
                BeliefDrivenExperimentProposalStatus::BeliefFrontierExceeded,
                input_belief_count,
                0,
                input_possibility_count,
            );
        }

        if input_possibility_count > bounds.max_possibilities() {
            return Self::empty(
                BeliefDrivenExperimentProposalStatus::PossibilityFrontierExceeded,
                input_belief_count,
                0,
                input_possibility_count,
            );
        }

        let mut canonical_beliefs = beliefs.to_vec();

        canonical_beliefs.sort_by(|left, right| {
            format!("{:?}", left.hypothesis()).cmp(&format!("{:?}", right.hypothesis()))
        });

        for index in 1..canonical_beliefs.len() {
            if canonical_beliefs[index - 1].hypothesis() == canonical_beliefs[index].hypothesis() {
                return Self::empty(
                    BeliefDrivenExperimentProposalStatus::DuplicateBeliefIdentity,
                    input_belief_count,
                    0,
                    input_possibility_count,
                );
            }
        }

        let active: Vec<HypothesisBeliefState> = canonical_beliefs
            .into_iter()
            .filter(|belief| {
                belief.active()
                    && belief.confidence().value()
                        >= policy.minimum_active_belief_confidence().value()
            })
            .collect();

        let active_belief_count = active.len();

        if active_belief_count < 2 {
            return Self::empty(
                BeliefDrivenExperimentProposalStatus::NoActiveCompetition,
                input_belief_count,
                active_belief_count,
                input_possibility_count,
            );
        }

        let mut ordered_possibilities = possibilities.to_vec();

        ordered_possibilities.sort_by(|left, right| {
            right
                .grounding_confidence()
                .value()
                .cmp(&left.grounding_confidence().value())
                .then_with(|| {
                    right
                        .controllability()
                        .value()
                        .cmp(&left.controllability().value())
                })
                .then_with(|| {
                    left.execution_cost()
                        .value()
                        .cmp(&right.execution_cost().value())
                })
                .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
        });

        let evaluated_possibility_count = ordered_possibilities.len();

        let mut rejected_prediction_frontier_count = 0;

        let mut rejected_conflicting_prediction_count = 0;

        let mut rejected_competition_count = 0;

        let mut rejected_foundation_count = 0;

        let mut generated = Vec::new();

        for possibility in ordered_possibilities {
            if possibility.predictions().len() > bounds.max_predictions_per_possibility() {
                rejected_prediction_frontier_count += 1;
                continue;
            }

            let mut predictions: Vec<CompetingHypothesisPrediction> = possibility
                .predictions()
                .iter()
                .filter(|prediction| {
                    prediction.confidence().value()
                        >= policy.minimum_prediction_confidence().value()
                        && active
                            .iter()
                            .any(|belief| belief.hypothesis() == prediction.hypothesis())
                })
                .cloned()
                .collect();

            predictions.sort_by(|left, right| {
                format!("{:?}", left.hypothesis())
                    .cmp(&format!("{:?}", right.hypothesis()))
                    .then_with(|| right.confidence().value().cmp(&left.confidence().value()))
                    .then_with(|| {
                        format!("{:?}", left.predicted_outcome())
                            .cmp(&format!("{:?}", right.predicted_outcome()))
                    })
            });

            let mut canonical: Vec<CompetingHypothesisPrediction> = Vec::new();

            let mut conflicting = false;

            for prediction in predictions {
                if let Some(existing) = canonical
                    .iter()
                    .find(|existing| existing.hypothesis() == prediction.hypothesis())
                {
                    if existing.predicted_outcome() != prediction.predicted_outcome() {
                        conflicting = true;
                        break;
                    }

                    continue;
                }

                canonical.push(prediction);
            }

            if conflicting {
                rejected_conflicting_prediction_count += 1;
                continue;
            }

            if canonical.len() < 2 {
                rejected_competition_count += 1;
                continue;
            }

            let information_gain = Self::information_gain(&canonical);

            if information_gain == CognitiveSignal::zero() {
                rejected_competition_count += 1;
                continue;
            }

            let relevant_beliefs: Vec<HypothesisBeliefState> = active
                .iter()
                .filter(|belief| {
                    canonical
                        .iter()
                        .any(|prediction| prediction.hypothesis() == belief.hypothesis())
                })
                .cloned()
                .collect();

            if relevant_beliefs.len() < 2 {
                rejected_competition_count += 1;
                continue;
            }

            let uncertainty = Self::uncertainty(&relevant_beliefs);

            let strongest = relevant_beliefs
                .iter()
                .max_by(|left, right| {
                    left.confidence()
                        .value()
                        .cmp(&right.confidence().value())
                        .then_with(|| {
                            format!("{:?}", right.hypothesis())
                                .cmp(&format!("{:?}", left.hypothesis()))
                        })
                })
                .expect("relevant beliefs are non-empty");

            let predicted_outcome = canonical
                .iter()
                .find(|prediction| prediction.hypothesis() == strongest.hypothesis())
                .expect("strongest belief has prediction")
                .predicted_outcome()
                .clone();

            let prediction_grounding = canonical
                .iter()
                .map(|prediction| prediction.confidence().value())
                .min()
                .expect("canonical predictions are non-empty");

            let grounding_value = possibility
                .grounding_confidence()
                .value()
                .min(prediction_grounding);

            let evidence = ExperimentEvidence::new(
                uncertainty,
                information_gain,
                possibility.controllability(),
                Self::signal(grounding_value),
                possibility.execution_cost(),
            )
            .expect("grounded generated evidence");

            let experiment = AutonomousExperimentProposal::new(
                possibility.source_state().clone(),
                possibility.action().clone(),
                predicted_outcome,
                evidence,
            );

            let foundation = AutonomousActiveExperimentationFoundation::select(
                std::slice::from_ref(&experiment),
                policy.foundation(),
            );

            if foundation.abstained() {
                rejected_foundation_count += 1;
                continue;
            }

            generated.push(
                HypothesisDiscriminationCandidate::new(experiment, canonical)
                    .expect("generated candidate has predictions"),
            );
        }

        generated.sort_by(Self::candidate_order);

        generated.dedup();

        let generated_before_frontier = generated.len();

        generated.truncate(bounds.max_generated_candidates());

        let generation_frontier_truncated = generated_before_frontier > generated.len();

        let status = if generated.is_empty() {
            BeliefDrivenExperimentProposalStatus::NoDiscriminatingPossibility
        } else {
            BeliefDrivenExperimentProposalStatus::Generated
        };

        BeliefDrivenExperimentProposalResult {
            status,
            input_belief_count,
            active_belief_count,
            input_possibility_count,
            evaluated_possibility_count,
            rejected_prediction_frontier_count,
            rejected_conflicting_prediction_count,
            rejected_competition_count,
            rejected_foundation_count,
            generated_before_frontier,
            generation_frontier_truncated,
            generated,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousBeliefDrivenExperimentProposal;

impl UniversalAutonomousBeliefDrivenExperimentProposal {
    pub fn evaluate(
        beliefs: &[HypothesisBeliefState],
        possibilities: &[GroundedExperimentPossibility],
        policy: BeliefDrivenExperimentProposalPolicy,
    ) -> BeliefDrivenExperimentProposalResult {
        AutonomousBeliefDrivenExperimentProposal::generate(beliefs, possibilities, policy)
    }
}

#[cfg(test)]
mod belief_driven_experiment_proposal_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn belief(hypothesis: u64, confidence: u16) -> HypothesisBeliefState {
        HypothesisBeliefState::new(a(hypothesis), s(confidence)).unwrap()
    }

    fn suspended(hypothesis: u64, confidence: u16) -> HypothesisBeliefState {
        let mut value = belief(hypothesis, confidence);

        value.availability = HypothesisBeliefAvailability::Suspended;

        value
    }

    fn prediction(hypothesis: u64, outcome: u64, confidence: u16) -> CompetingHypothesisPrediction {
        CompetingHypothesisPrediction::new(a(hypothesis), a(outcome), s(confidence)).unwrap()
    }

    fn possibility(
        action: u64,
        predictions: Vec<CompetingHypothesisPrediction>,
    ) -> GroundedExperimentPossibility {
        GroundedExperimentPossibility::new(a(1), a(action), predictions, s(900), s(900), s(100))
            .unwrap()
    }

    fn foundation_policy() -> ActiveExperimentPolicy {
        ActiveExperimentPolicy::new(
            ActiveExperimentBounds::new(32, 32, 32).unwrap(),
            ActiveExperimentThresholds::new(s(500), s(500), s(500), s(500)).unwrap(),
        )
    }

    fn policy() -> BeliefDrivenExperimentProposalPolicy {
        BeliefDrivenExperimentProposalPolicy::new(
            foundation_policy(),
            BeliefDrivenExperimentProposalBounds::new(16, 16, 16, 16).unwrap(),
            s(500),
            s(500),
        )
        .unwrap()
    }

    #[test]
    fn proposal_contract_requires_grounded_possibility_and_positive_bounds() {
        assert_eq!(
            GroundedExperimentPossibility::new(a(1), a(2), Vec::new(), s(900), s(900), s(100),),
            None
        );

        assert_eq!(
            GroundedExperimentPossibility::new(
                a(1),
                a(2),
                vec![prediction(1, 10, 900)],
                s(0),
                s(900),
                s(100),
            ),
            None
        );

        assert_eq!(BeliefDrivenExperimentProposalBounds::new(1, 1, 1, 1), None);

        assert_eq!(
            BeliefDrivenExperimentProposalPolicy::new(
                foundation_policy(),
                BeliefDrivenExperimentProposalBounds::new(2, 1, 1, 1).unwrap(),
                s(0),
                s(500),
            ),
            None
        );
    }

    #[test]
    fn competing_active_beliefs_generate_discriminating_experiment_candidate() {
        let possibility = possibility(10, vec![prediction(1, 100, 900), prediction(2, 101, 900)]);

        let result = AutonomousBeliefDrivenExperimentProposal::generate(
            &[belief(1, 700), belief(2, 680)],
            std::slice::from_ref(&possibility),
            policy(),
        );

        assert_eq!(
            result.status(),
            BeliefDrivenExperimentProposalStatus::Generated
        );

        assert_eq!(result.generated_count(), 1);

        assert_eq!(
            result.generated()[0].experiment().action(),
            possibility.action()
        );

        assert_eq!(result.generated()[0].predictions().len(), 2);
    }

    #[test]
    fn suspended_hypothesis_cannot_drive_new_experiment_proposal() {
        let result = AutonomousBeliefDrivenExperimentProposal::generate(
            &[belief(1, 700), suspended(2, 900)],
            &[possibility(
                10,
                vec![prediction(1, 100, 900), prediction(2, 101, 900)],
            )],
            policy(),
        );

        assert_eq!(
            result.status(),
            BeliefDrivenExperimentProposalStatus::NoActiveCompetition
        );

        assert_eq!(result.active_belief_count(), 1);
    }

    #[test]
    fn weak_persistent_belief_cannot_manufacture_active_competition() {
        let result = AutonomousBeliefDrivenExperimentProposal::generate(
            &[belief(1, 700), belief(2, 400)],
            &[possibility(
                10,
                vec![prediction(1, 100, 900), prediction(2, 101, 900)],
            )],
            policy(),
        );

        assert_eq!(
            result.status(),
            BeliefDrivenExperimentProposalStatus::NoActiveCompetition
        );
    }

    #[test]
    fn weak_prediction_cannot_manufacture_discriminating_candidate() {
        let result = AutonomousBeliefDrivenExperimentProposal::generate(
            &[belief(1, 700), belief(2, 680)],
            &[possibility(
                10,
                vec![prediction(1, 100, 900), prediction(2, 101, 400)],
            )],
            policy(),
        );

        assert!(result.abstained());

        assert_eq!(result.rejected_competition_count(), 1);
    }

    #[test]
    fn identical_predicted_outcomes_do_not_create_information_gain() {
        let result = AutonomousBeliefDrivenExperimentProposal::generate(
            &[belief(1, 700), belief(2, 680)],
            &[possibility(
                10,
                vec![prediction(1, 100, 900), prediction(2, 100, 900)],
            )],
            policy(),
        );

        assert!(result.abstained());

        assert_eq!(result.rejected_competition_count(), 1);
    }

    #[test]
    fn conflicting_predictions_from_same_hypothesis_are_rejected() {
        let result = AutonomousBeliefDrivenExperimentProposal::generate(
            &[belief(1, 700), belief(2, 680)],
            &[possibility(
                10,
                vec![
                    prediction(1, 100, 900),
                    prediction(1, 101, 900),
                    prediction(2, 102, 900),
                ],
            )],
            policy(),
        );

        assert!(result.abstained());

        assert_eq!(result.rejected_conflicting_prediction_count(), 1);
    }

    #[test]
    fn exact_duplicate_prediction_does_not_inflate_generated_competition() {
        let duplicate = prediction(1, 100, 900);

        let result = AutonomousBeliefDrivenExperimentProposal::generate(
            &[belief(1, 700), belief(2, 680)],
            &[possibility(
                10,
                vec![duplicate.clone(), duplicate, prediction(2, 101, 900)],
            )],
            policy(),
        );

        assert_eq!(result.generated_count(), 1);

        assert_eq!(result.generated()[0].predictions().len(), 2);
    }

    #[test]
    fn generated_evidence_derives_disagreement_and_preserves_exact_grounded_action() {
        let possibility = possibility(
            77,
            vec![
                prediction(1, 100, 900),
                prediction(2, 100, 900),
                prediction(3, 101, 900),
            ],
        );

        let result = AutonomousBeliefDrivenExperimentProposal::generate(
            &[belief(1, 700), belief(2, 690), belief(3, 680)],
            std::slice::from_ref(&possibility),
            policy(),
        );

        assert_eq!(result.generated_count(), 1);

        let experiment = result.generated()[0].experiment();

        assert_eq!(experiment.source_state(), possibility.source_state());

        assert_eq!(experiment.action(), possibility.action());

        assert_eq!(experiment.evidence().expected_information_gain(), s(666));

        assert_eq!(experiment.evidence().prediction_uncertainty(), s(990));
    }

    #[test]
    fn foundation_gates_remain_authoritative_for_generated_experiments() {
        let weak = GroundedExperimentPossibility::new(
            a(1),
            a(10),
            vec![prediction(1, 100, 900), prediction(2, 101, 900)],
            s(400),
            s(900),
            s(100),
        )
        .unwrap();

        let result = AutonomousBeliefDrivenExperimentProposal::generate(
            &[belief(1, 700), belief(2, 680)],
            &[weak],
            policy(),
        );

        assert!(result.abstained());

        assert_eq!(result.rejected_foundation_count(), 1);
    }

    #[test]
    fn belief_possibility_prediction_and_generation_frontiers_are_hard_bounded() {
        let beliefs = vec![belief(1, 700), belief(2, 680), belief(3, 660)];

        let base_possibility =
            possibility(10, vec![prediction(1, 100, 900), prediction(2, 101, 900)]);

        let belief_policy = BeliefDrivenExperimentProposalPolicy::new(
            foundation_policy(),
            BeliefDrivenExperimentProposalBounds::new(2, 16, 16, 16).unwrap(),
            s(500),
            s(500),
        )
        .unwrap();

        let belief_result = AutonomousBeliefDrivenExperimentProposal::generate(
            &beliefs,
            std::slice::from_ref(&base_possibility),
            belief_policy,
        );

        assert_eq!(
            belief_result.status(),
            BeliefDrivenExperimentProposalStatus::BeliefFrontierExceeded
        );

        let possibility_policy = BeliefDrivenExperimentProposalPolicy::new(
            foundation_policy(),
            BeliefDrivenExperimentProposalBounds::new(16, 1, 16, 16).unwrap(),
            s(500),
            s(500),
        )
        .unwrap();

        let possibility_result = AutonomousBeliefDrivenExperimentProposal::generate(
            &beliefs[..2],
            &[
                base_possibility.clone(),
                possibility(11, vec![prediction(1, 110, 900), prediction(2, 111, 900)]),
            ],
            possibility_policy,
        );

        assert_eq!(
            possibility_result.status(),
            BeliefDrivenExperimentProposalStatus::PossibilityFrontierExceeded
        );

        let prediction_policy = BeliefDrivenExperimentProposalPolicy::new(
            foundation_policy(),
            BeliefDrivenExperimentProposalBounds::new(16, 16, 2, 16).unwrap(),
            s(500),
            s(500),
        )
        .unwrap();

        let prediction_result = AutonomousBeliefDrivenExperimentProposal::generate(
            &beliefs,
            &[possibility(
                12,
                vec![
                    prediction(1, 120, 900),
                    prediction(2, 121, 900),
                    prediction(3, 122, 900),
                ],
            )],
            prediction_policy,
        );

        assert_eq!(prediction_result.rejected_prediction_frontier_count(), 1);

        assert!(prediction_result.abstained());

        let generation_policy = BeliefDrivenExperimentProposalPolicy::new(
            foundation_policy(),
            BeliefDrivenExperimentProposalBounds::new(16, 16, 16, 1).unwrap(),
            s(500),
            s(500),
        )
        .unwrap();

        let generation_result = AutonomousBeliefDrivenExperimentProposal::generate(
            &beliefs[..2],
            &[
                base_possibility,
                possibility(13, vec![prediction(1, 130, 900), prediction(2, 131, 900)]),
            ],
            generation_policy,
        );

        assert_eq!(generation_result.generated_before_frontier(), 2);

        assert_eq!(generation_result.generated_count(), 1);

        assert!(generation_result.generation_frontier_truncated());
    }

    #[test]
    fn proposal_generation_is_order_invariant_non_mutating_and_facade_equivalent() {
        let beliefs = vec![belief(2, 680), belief(1, 700), belief(3, 660)];

        let possibilities = vec![
            possibility(
                10,
                vec![
                    prediction(1, 100, 900),
                    prediction(2, 101, 900),
                    prediction(3, 102, 900),
                ],
            ),
            possibility(11, vec![prediction(1, 110, 900), prediction(2, 111, 900)]),
        ];

        let before_beliefs = beliefs.clone();

        let before_possibilities = possibilities.clone();

        let mut reversed_beliefs = beliefs.clone();

        reversed_beliefs.reverse();

        let mut reversed_possibilities = possibilities.clone();

        reversed_possibilities.reverse();

        for possibility in &mut reversed_possibilities {
            possibility.predictions.reverse();
        }

        let p = policy();

        let direct =
            AutonomousBeliefDrivenExperimentProposal::generate(&beliefs, &possibilities, p);

        let reordered = AutonomousBeliefDrivenExperimentProposal::generate(
            &reversed_beliefs,
            &reversed_possibilities,
            p,
        );

        let facade = UniversalAutonomousBeliefDrivenExperimentProposal::evaluate(
            &beliefs,
            &possibilities,
            p,
        );

        let repeated = UniversalAutonomousBeliefDrivenExperimentProposal::evaluate(
            &beliefs,
            &possibilities,
            p,
        );

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(beliefs, before_beliefs);
        assert_eq!(possibilities, before_possibilities);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LearningProgressMeasurement {
    prediction_error_before: CognitiveSignal,
    prediction_error_after: CognitiveSignal,
    uncertainty_before: CognitiveSignal,
    uncertainty_after: CognitiveSignal,
    evidence_confidence: CognitiveSignal,
}

impl LearningProgressMeasurement {
    pub fn new(
        prediction_error_before: CognitiveSignal,
        prediction_error_after: CognitiveSignal,
        uncertainty_before: CognitiveSignal,
        uncertainty_after: CognitiveSignal,
        evidence_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if evidence_confidence == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            prediction_error_before,
            prediction_error_after,
            uncertainty_before,
            uncertainty_after,
            evidence_confidence,
        })
    }

    pub fn prediction_error_before(self) -> CognitiveSignal {
        self.prediction_error_before
    }

    pub fn prediction_error_after(self) -> CognitiveSignal {
        self.prediction_error_after
    }

    pub fn uncertainty_before(self) -> CognitiveSignal {
        self.uncertainty_before
    }

    pub fn uncertainty_after(self) -> CognitiveSignal {
        self.uncertainty_after
    }

    pub fn evidence_confidence(self) -> CognitiveSignal {
        self.evidence_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentLearningProgressSample {
    evidence_identity: CognitiveStructure,
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    measurement: LearningProgressMeasurement,
}

impl ExperimentLearningProgressSample {
    pub fn new(
        evidence_identity: CognitiveStructure,
        source_state: CognitiveStructure,
        action: CognitiveStructure,
        measurement: LearningProgressMeasurement,
    ) -> Self {
        Self {
            evidence_identity,
            source_state,
            action,
            measurement,
        }
    }

    pub fn evidence_identity(&self) -> &CognitiveStructure {
        &self.evidence_identity
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn measurement(&self) -> LearningProgressMeasurement {
        self.measurement
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LearningProgressBounds {
    max_input_samples: usize,
    max_focuses: usize,
    max_samples_per_focus: usize,
}

impl LearningProgressBounds {
    pub fn new(
        max_input_samples: usize,
        max_focuses: usize,
        max_samples_per_focus: usize,
    ) -> Option<Self> {
        if max_input_samples == 0 || max_focuses == 0 || max_samples_per_focus == 0 {
            return None;
        }

        Some(Self {
            max_input_samples,
            max_focuses,
            max_samples_per_focus,
        })
    }

    pub fn max_input_samples(self) -> usize {
        self.max_input_samples
    }

    pub fn max_focuses(self) -> usize {
        self.max_focuses
    }

    pub fn max_samples_per_focus(self) -> usize {
        self.max_samples_per_focus
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LearningProgressThresholds {
    minimum_evidence_confidence: CognitiveSignal,
    minimum_samples_per_focus: usize,
    minimum_learning_progress: CognitiveSignal,
}

impl LearningProgressThresholds {
    pub fn new(
        minimum_evidence_confidence: CognitiveSignal,
        minimum_samples_per_focus: usize,
        minimum_learning_progress: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_evidence_confidence == CognitiveSignal::zero()
            || minimum_samples_per_focus == 0
            || minimum_learning_progress == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_evidence_confidence,
            minimum_samples_per_focus,
            minimum_learning_progress,
        })
    }

    pub fn minimum_evidence_confidence(self) -> CognitiveSignal {
        self.minimum_evidence_confidence
    }

    pub fn minimum_samples_per_focus(self) -> usize {
        self.minimum_samples_per_focus
    }

    pub fn minimum_learning_progress(self) -> CognitiveSignal {
        self.minimum_learning_progress
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LearningProgressPolicy {
    bounds: LearningProgressBounds,
    thresholds: LearningProgressThresholds,
}

impl LearningProgressPolicy {
    pub fn new(
        bounds: LearningProgressBounds,
        thresholds: LearningProgressThresholds,
    ) -> Option<Self> {
        if thresholds.minimum_samples_per_focus() > bounds.max_samples_per_focus() {
            return None;
        }

        Some(Self { bounds, thresholds })
    }

    pub fn bounds(self) -> LearningProgressBounds {
        self.bounds
    }

    pub fn thresholds(self) -> LearningProgressThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentLearningProgressEstimate {
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    qualifying_sample_count: usize,
    mean_error_reduction: CognitiveSignal,
    mean_uncertainty_reduction: CognitiveSignal,
    mean_evidence_confidence: CognitiveSignal,
    learning_progress: CognitiveSignal,
}

impl ExperimentLearningProgressEstimate {
    pub fn source_state(&self) -> &CognitiveStructure {
        &self.source_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn qualifying_sample_count(&self) -> usize {
        self.qualifying_sample_count
    }

    pub fn mean_error_reduction(&self) -> CognitiveSignal {
        self.mean_error_reduction
    }

    pub fn mean_uncertainty_reduction(&self) -> CognitiveSignal {
        self.mean_uncertainty_reduction
    }

    pub fn mean_evidence_confidence(&self) -> CognitiveSignal {
        self.mean_evidence_confidence
    }

    pub fn learning_progress(&self) -> CognitiveSignal {
        self.learning_progress
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LearningProgressEstimationStatus {
    Estimated,
    NoQualifyingProgress,
    InputFrontierExceeded,
    FocusFrontierExceeded,
    ConflictingEvidenceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningProgressEstimationResult {
    status: LearningProgressEstimationStatus,
    input_sample_count: usize,
    unique_sample_count: usize,
    qualifying_sample_count: usize,
    rejected_confidence_count: usize,
    focus_count: usize,
    rejected_focus_sample_frontier_count: usize,
    rejected_insufficient_sample_count: usize,
    rejected_progress_count: usize,
    estimates: Vec<ExperimentLearningProgressEstimate>,
}

impl LearningProgressEstimationResult {
    pub fn status(&self) -> LearningProgressEstimationStatus {
        self.status
    }

    pub fn input_sample_count(&self) -> usize {
        self.input_sample_count
    }

    pub fn unique_sample_count(&self) -> usize {
        self.unique_sample_count
    }

    pub fn qualifying_sample_count(&self) -> usize {
        self.qualifying_sample_count
    }

    pub fn rejected_confidence_count(&self) -> usize {
        self.rejected_confidence_count
    }

    pub fn focus_count(&self) -> usize {
        self.focus_count
    }

    pub fn rejected_focus_sample_frontier_count(&self) -> usize {
        self.rejected_focus_sample_frontier_count
    }

    pub fn rejected_insufficient_sample_count(&self) -> usize {
        self.rejected_insufficient_sample_count
    }

    pub fn rejected_progress_count(&self) -> usize {
        self.rejected_progress_count
    }

    pub fn estimates(&self) -> &[ExperimentLearningProgressEstimate] {
        &self.estimates
    }

    pub fn estimate_count(&self) -> usize {
        self.estimates.len()
    }

    pub fn estimated_any(&self) -> bool {
        !self.estimates.is_empty()
    }

    pub fn abstained(&self) -> bool {
        !self.estimated_any()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LearningProgressFocusGroup {
    source_state: CognitiveStructure,
    action: CognitiveStructure,
    samples: Vec<ExperimentLearningProgressSample>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousLearningProgressEstimation;

impl AutonomousLearningProgressEstimation {
    fn signal(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).expect("bounded learning progress signal")
        }
    }

    fn empty(
        status: LearningProgressEstimationStatus,
        input_sample_count: usize,
        unique_sample_count: usize,
        qualifying_sample_count: usize,
        rejected_confidence_count: usize,
        focus_count: usize,
    ) -> LearningProgressEstimationResult {
        LearningProgressEstimationResult {
            status,
            input_sample_count,
            unique_sample_count,
            qualifying_sample_count,
            rejected_confidence_count,
            focus_count,
            rejected_focus_sample_frontier_count: 0,
            rejected_insufficient_sample_count: 0,
            rejected_progress_count: 0,
            estimates: Vec::new(),
        }
    }

    fn estimate_order(
        left: &ExperimentLearningProgressEstimate,
        right: &ExperimentLearningProgressEstimate,
    ) -> std::cmp::Ordering {
        right
            .learning_progress()
            .value()
            .cmp(&left.learning_progress().value())
            .then_with(|| {
                right
                    .mean_error_reduction()
                    .value()
                    .cmp(&left.mean_error_reduction().value())
            })
            .then_with(|| {
                right
                    .mean_uncertainty_reduction()
                    .value()
                    .cmp(&left.mean_uncertainty_reduction().value())
            })
            .then_with(|| {
                right
                    .mean_evidence_confidence()
                    .value()
                    .cmp(&left.mean_evidence_confidence().value())
            })
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    pub fn estimate(
        samples: &[ExperimentLearningProgressSample],
        policy: LearningProgressPolicy,
    ) -> LearningProgressEstimationResult {
        let bounds = policy.bounds();

        let thresholds = policy.thresholds();

        let input_sample_count = samples.len();

        if input_sample_count > bounds.max_input_samples() {
            return Self::empty(
                LearningProgressEstimationStatus::InputFrontierExceeded,
                input_sample_count,
                0,
                0,
                0,
                0,
            );
        }

        let mut ordered = samples.to_vec();

        ordered.sort_by(|left, right| {
            format!("{:?}", left.evidence_identity())
                .cmp(&format!("{:?}", right.evidence_identity()))
                .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
        });

        let mut canonical: Vec<ExperimentLearningProgressSample> = Vec::new();

        for sample in ordered {
            if let Some(existing) = canonical
                .iter()
                .find(|existing| existing.evidence_identity() == sample.evidence_identity())
            {
                if existing != &sample {
                    return Self::empty(
                        LearningProgressEstimationStatus::ConflictingEvidenceIdentity,
                        input_sample_count,
                        canonical.len(),
                        0,
                        0,
                        0,
                    );
                }

                continue;
            }

            canonical.push(sample);
        }

        let unique_sample_count = canonical.len();

        let mut rejected_confidence_count = 0;

        let qualifying: Vec<ExperimentLearningProgressSample> = canonical
            .into_iter()
            .filter(|sample| {
                if sample.measurement().evidence_confidence().value()
                    < thresholds.minimum_evidence_confidence().value()
                {
                    rejected_confidence_count += 1;
                    false
                } else {
                    true
                }
            })
            .collect();

        let qualifying_sample_count = qualifying.len();

        let mut groups: Vec<LearningProgressFocusGroup> = Vec::new();

        for sample in qualifying {
            if let Some(group) = groups.iter_mut().find(|group| {
                group.source_state == *sample.source_state() && group.action == *sample.action()
            }) {
                group.samples.push(sample);
            } else {
                groups.push(LearningProgressFocusGroup {
                    source_state: sample.source_state().clone(),
                    action: sample.action().clone(),
                    samples: vec![sample],
                });
            }
        }

        groups.sort_by(|left, right| {
            format!("{:?}", left.source_state)
                .cmp(&format!("{:?}", right.source_state))
                .then_with(|| format!("{:?}", left.action).cmp(&format!("{:?}", right.action)))
        });

        let focus_count = groups.len();

        if focus_count > bounds.max_focuses() {
            return Self::empty(
                LearningProgressEstimationStatus::FocusFrontierExceeded,
                input_sample_count,
                unique_sample_count,
                qualifying_sample_count,
                rejected_confidence_count,
                focus_count,
            );
        }

        let mut rejected_focus_sample_frontier_count = 0;

        let mut rejected_insufficient_sample_count = 0;

        let mut rejected_progress_count = 0;

        let mut estimates = Vec::new();

        for group in groups {
            if group.samples.len() > bounds.max_samples_per_focus() {
                rejected_focus_sample_frontier_count += 1;
                continue;
            }

            if group.samples.len() < thresholds.minimum_samples_per_focus() {
                rejected_insufficient_sample_count += 1;
                continue;
            }

            let mut error_reduction_sum: u32 = 0;

            let mut uncertainty_reduction_sum: u32 = 0;

            let mut confidence_sum: u32 = 0;

            for sample in &group.samples {
                let measurement = sample.measurement();

                error_reduction_sum = error_reduction_sum.saturating_add(u32::from(
                    measurement
                        .prediction_error_before()
                        .value()
                        .saturating_sub(measurement.prediction_error_after().value()),
                ));

                uncertainty_reduction_sum = uncertainty_reduction_sum.saturating_add(u32::from(
                    measurement
                        .uncertainty_before()
                        .value()
                        .saturating_sub(measurement.uncertainty_after().value()),
                ));

                confidence_sum = confidence_sum
                    .saturating_add(u32::from(measurement.evidence_confidence().value()));
            }

            let denominator =
                u32::try_from(group.samples.len()).expect("bounded sample count fits u32");

            let mean_error = (error_reduction_sum / denominator).min(1000) as u16;

            let mean_uncertainty = (uncertainty_reduction_sum / denominator).min(1000) as u16;

            let mean_confidence = (confidence_sum / denominator).min(1000) as u16;

            let progress = mean_error.min(mean_uncertainty);

            if progress < thresholds.minimum_learning_progress().value() {
                rejected_progress_count += 1;
                continue;
            }

            estimates.push(ExperimentLearningProgressEstimate {
                source_state: group.source_state,
                action: group.action,
                qualifying_sample_count: group.samples.len(),
                mean_error_reduction: Self::signal(mean_error),
                mean_uncertainty_reduction: Self::signal(mean_uncertainty),
                mean_evidence_confidence: Self::signal(mean_confidence),
                learning_progress: Self::signal(progress),
            });
        }

        estimates.sort_by(Self::estimate_order);

        let status = if estimates.is_empty() {
            LearningProgressEstimationStatus::NoQualifyingProgress
        } else {
            LearningProgressEstimationStatus::Estimated
        };

        LearningProgressEstimationResult {
            status,
            input_sample_count,
            unique_sample_count,
            qualifying_sample_count,
            rejected_confidence_count,
            focus_count,
            rejected_focus_sample_frontier_count,
            rejected_insufficient_sample_count,
            rejected_progress_count,
            estimates,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousLearningProgressEstimation;

impl UniversalAutonomousLearningProgressEstimation {
    pub fn evaluate(
        samples: &[ExperimentLearningProgressSample],
        policy: LearningProgressPolicy,
    ) -> LearningProgressEstimationResult {
        AutonomousLearningProgressEstimation::estimate(samples, policy)
    }
}

#[cfg(test)]
mod learning_progress_estimation_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn measurement(values: [u16; 5]) -> LearningProgressMeasurement {
        LearningProgressMeasurement::new(
            s(values[0]),
            s(values[1]),
            s(values[2]),
            s(values[3]),
            s(values[4]),
        )
        .unwrap()
    }

    fn progress_sample(
        identity: u64,
        state: u64,
        action: u64,
        values: [u16; 5],
    ) -> ExperimentLearningProgressSample {
        ExperimentLearningProgressSample::new(a(identity), a(state), a(action), measurement(values))
    }

    fn policy() -> LearningProgressPolicy {
        LearningProgressPolicy::new(
            LearningProgressBounds::new(32, 16, 8).unwrap(),
            LearningProgressThresholds::new(s(500), 2, s(50)).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn learning_progress_contract_requires_confident_evidence_and_consistent_bounds() {
        assert_eq!(
            LearningProgressMeasurement::new(s(900), s(800), s(900), s(800), s(0),),
            None
        );

        assert_eq!(LearningProgressBounds::new(0, 1, 1), None);

        assert_eq!(LearningProgressThresholds::new(s(500), 0, s(50),), None);

        assert_eq!(
            LearningProgressPolicy::new(
                LearningProgressBounds::new(10, 10, 1).unwrap(),
                LearningProgressThresholds::new(s(500), 2, s(50),).unwrap(),
            ),
            None
        );
    }

    #[test]
    fn repeated_confident_error_and_uncertainty_reduction_estimates_learning_progress() {
        let result = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(1, 10, 20, [900, 700, 800, 600, 900]),
                progress_sample(2, 10, 20, [800, 600, 700, 500, 900]),
            ],
            policy(),
        );

        assert_eq!(result.status(), LearningProgressEstimationStatus::Estimated);

        assert_eq!(result.estimate_count(), 1);

        assert_eq!(result.estimates()[0].mean_error_reduction(), s(200));

        assert_eq!(result.estimates()[0].mean_uncertainty_reduction(), s(200));

        assert_eq!(result.estimates()[0].learning_progress(), s(200));
    }

    #[test]
    fn high_raw_uncertainty_without_reduction_is_not_learning_progress() {
        let result = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(1, 10, 20, [900, 900, 1000, 1000, 900]),
                progress_sample(2, 10, 20, [900, 900, 1000, 1000, 900]),
            ],
            policy(),
        );

        assert!(result.abstained());

        assert_eq!(result.rejected_progress_count(), 1);
    }

    #[test]
    fn error_reduction_without_uncertainty_reduction_is_not_sufficient_progress() {
        let result = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(1, 10, 20, [900, 600, 900, 900, 900]),
                progress_sample(2, 10, 20, [800, 500, 800, 800, 900]),
            ],
            policy(),
        );

        assert!(result.abstained());

        assert_eq!(result.rejected_progress_count(), 1);
    }

    #[test]
    fn uncertainty_reduction_without_prediction_error_reduction_is_not_sufficient_progress() {
        let result = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(1, 10, 20, [900, 900, 900, 600, 900]),
                progress_sample(2, 10, 20, [800, 800, 800, 500, 900]),
            ],
            policy(),
        );

        assert!(result.abstained());

        assert_eq!(result.rejected_progress_count(), 1);
    }

    #[test]
    fn low_confidence_samples_cannot_manufacture_learning_progress() {
        let result = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(1, 10, 20, [900, 500, 900, 500, 900]),
                progress_sample(2, 10, 20, [900, 500, 900, 500, 400]),
            ],
            policy(),
        );

        assert!(result.abstained());

        assert_eq!(result.rejected_confidence_count(), 1);

        assert_eq!(result.rejected_insufficient_sample_count(), 1);
    }

    #[test]
    fn exact_evidence_duplicate_is_deduplicated_without_fake_repetition() {
        let duplicate = progress_sample(1, 10, 20, [900, 700, 900, 700, 900]);

        let result = AutonomousLearningProgressEstimation::estimate(
            &[
                duplicate.clone(),
                duplicate,
                progress_sample(2, 10, 20, [800, 600, 800, 600, 900]),
            ],
            policy(),
        );

        assert_eq!(result.input_sample_count(), 3);

        assert_eq!(result.unique_sample_count(), 2);

        assert_eq!(result.estimates()[0].qualifying_sample_count(), 2);
    }

    #[test]
    fn conflicting_reuse_of_exact_evidence_identity_abstains_atomically() {
        let result = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(1, 10, 20, [900, 700, 900, 700, 900]),
                progress_sample(1, 10, 20, [900, 500, 900, 500, 900]),
            ],
            policy(),
        );

        assert_eq!(
            result.status(),
            LearningProgressEstimationStatus::ConflictingEvidenceIdentity
        );

        assert!(result.estimates().is_empty());
    }

    #[test]
    fn state_and_action_define_exact_learning_progress_focus_identity() {
        let result = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(1, 10, 20, [900, 700, 900, 700, 900]),
                progress_sample(2, 10, 20, [800, 600, 800, 600, 900]),
                progress_sample(3, 10, 21, [900, 600, 900, 600, 900]),
                progress_sample(4, 10, 21, [800, 500, 800, 500, 900]),
                progress_sample(5, 11, 20, [900, 500, 900, 500, 900]),
                progress_sample(6, 11, 20, [800, 400, 800, 400, 900]),
            ],
            policy(),
        );

        assert_eq!(result.focus_count(), 3);

        assert_eq!(result.estimate_count(), 3);
    }

    #[test]
    fn measured_learning_progress_ranks_before_raw_initial_uncertainty() {
        let result = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(1, 10, 20, [1000, 900, 1000, 900, 900]),
                progress_sample(2, 10, 20, [1000, 900, 1000, 900, 900]),
                progress_sample(3, 10, 21, [700, 400, 700, 400, 900]),
                progress_sample(4, 10, 21, [700, 400, 700, 400, 900]),
            ],
            policy(),
        );

        assert_eq!(result.estimate_count(), 2);

        assert_eq!(result.estimates()[0].action(), &a(21));

        assert_eq!(result.estimates()[0].learning_progress(), s(300));

        assert_eq!(result.estimates()[1].learning_progress(), s(100));
    }

    #[test]
    fn input_focus_and_per_focus_sample_frontiers_are_hard_bounded() {
        let two_samples = vec![
            progress_sample(1, 10, 20, [900, 700, 900, 700, 900]),
            progress_sample(2, 10, 20, [800, 600, 800, 600, 900]),
        ];

        let input_policy = LearningProgressPolicy::new(
            LearningProgressBounds::new(1, 16, 8).unwrap(),
            LearningProgressThresholds::new(s(500), 1, s(50)).unwrap(),
        )
        .unwrap();

        let input = AutonomousLearningProgressEstimation::estimate(&two_samples, input_policy);

        assert_eq!(
            input.status(),
            LearningProgressEstimationStatus::InputFrontierExceeded
        );

        let focus_policy = LearningProgressPolicy::new(
            LearningProgressBounds::new(16, 1, 8).unwrap(),
            LearningProgressThresholds::new(s(500), 1, s(50)).unwrap(),
        )
        .unwrap();

        let focus = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(3, 10, 20, [900, 700, 900, 700, 900]),
                progress_sample(4, 10, 21, [900, 700, 900, 700, 900]),
            ],
            focus_policy,
        );

        assert_eq!(
            focus.status(),
            LearningProgressEstimationStatus::FocusFrontierExceeded
        );

        let per_focus_policy = LearningProgressPolicy::new(
            LearningProgressBounds::new(16, 16, 2).unwrap(),
            LearningProgressThresholds::new(s(500), 2, s(50)).unwrap(),
        )
        .unwrap();

        let per_focus = AutonomousLearningProgressEstimation::estimate(
            &[
                progress_sample(5, 10, 20, [900, 700, 900, 700, 900]),
                progress_sample(6, 10, 20, [800, 600, 800, 600, 900]),
                progress_sample(7, 10, 20, [700, 500, 700, 500, 900]),
            ],
            per_focus_policy,
        );

        assert!(per_focus.abstained());

        assert_eq!(per_focus.rejected_focus_sample_frontier_count(), 1);
    }

    #[test]
    fn learning_progress_estimation_is_order_invariant_non_mutating_and_facade_equivalent() {
        let samples = vec![
            progress_sample(1, 10, 20, [900, 700, 900, 700, 900]),
            progress_sample(2, 10, 20, [800, 600, 800, 600, 900]),
            progress_sample(3, 10, 21, [900, 600, 900, 600, 900]),
            progress_sample(4, 10, 21, [800, 500, 800, 500, 900]),
        ];

        let before = samples.clone();

        let mut reversed = samples.clone();

        reversed.reverse();

        let p = policy();

        let direct = AutonomousLearningProgressEstimation::estimate(&samples, p);

        let reordered = AutonomousLearningProgressEstimation::estimate(&reversed, p);

        let facade = UniversalAutonomousLearningProgressEstimation::evaluate(&samples, p);

        let repeated = UniversalAutonomousLearningProgressEstimation::evaluate(&samples, p);

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(samples, before);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExperimentSequencePlanningBounds {
    max_input_candidates: usize,
    max_learning_progress_estimates: usize,
    max_plan_depth: usize,
    max_expansions: usize,
    max_selected_plans: usize,
}

impl ExperimentSequencePlanningBounds {
    pub fn new(
        max_input_candidates: usize,
        max_learning_progress_estimates: usize,
        max_plan_depth: usize,
        max_expansions: usize,
        max_selected_plans: usize,
    ) -> Option<Self> {
        if max_input_candidates == 0
            || max_learning_progress_estimates == 0
            || max_plan_depth == 0
            || max_expansions == 0
            || max_selected_plans == 0
        {
            return None;
        }

        Some(Self {
            max_input_candidates,
            max_learning_progress_estimates,
            max_plan_depth,
            max_expansions,
            max_selected_plans,
        })
    }

    pub fn max_input_candidates(self) -> usize {
        self.max_input_candidates
    }

    pub fn max_learning_progress_estimates(self) -> usize {
        self.max_learning_progress_estimates
    }

    pub fn max_plan_depth(self) -> usize {
        self.max_plan_depth
    }

    pub fn max_expansions(self) -> usize {
        self.max_expansions
    }

    pub fn max_selected_plans(self) -> usize {
        self.max_selected_plans
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExperimentSequencePlanningPolicy {
    bounds: ExperimentSequencePlanningBounds,
    minimum_discrimination_gain: CognitiveSignal,
}

impl ExperimentSequencePlanningPolicy {
    pub fn new(
        bounds: ExperimentSequencePlanningBounds,
        minimum_discrimination_gain: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_discrimination_gain == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            bounds,
            minimum_discrimination_gain,
        })
    }

    pub fn bounds(self) -> ExperimentSequencePlanningBounds {
        self.bounds
    }

    pub fn minimum_discrimination_gain(self) -> CognitiveSignal {
        self.minimum_discrimination_gain
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedExperimentStep {
    candidate: HypothesisDiscriminationCandidate,
    learning_progress: CognitiveSignal,
    discrimination_gain: CognitiveSignal,
}

impl PlannedExperimentStep {
    pub fn candidate(&self) -> &HypothesisDiscriminationCandidate {
        &self.candidate
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        self.candidate.experiment().source_state()
    }

    pub fn action(&self) -> &CognitiveStructure {
        self.candidate.experiment().action()
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        self.candidate.experiment().predicted_outcome()
    }

    pub fn learning_progress(&self) -> CognitiveSignal {
        self.learning_progress
    }

    pub fn discrimination_gain(&self) -> CognitiveSignal {
        self.discrimination_gain
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentSequencePlan {
    initial_state: CognitiveStructure,
    steps: Vec<PlannedExperimentStep>,
    cumulative_learning_progress: u32,
    cumulative_discrimination_gain: u32,
    cumulative_information_gain: u32,
}

impl ExperimentSequencePlan {
    pub fn initial_state(&self) -> &CognitiveStructure {
        &self.initial_state
    }

    pub fn steps(&self) -> &[PlannedExperimentStep] {
        &self.steps
    }

    pub fn depth(&self) -> usize {
        self.steps.len()
    }

    pub fn cumulative_learning_progress(&self) -> u32 {
        self.cumulative_learning_progress
    }

    pub fn cumulative_discrimination_gain(&self) -> u32 {
        self.cumulative_discrimination_gain
    }

    pub fn cumulative_information_gain(&self) -> u32 {
        self.cumulative_information_gain
    }

    pub fn terminal_state(&self) -> &CognitiveStructure {
        self.steps
            .last()
            .expect("sequence plans contain at least one step")
            .predicted_outcome()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExperimentSequencePlanningStatus {
    Planned,
    NoContinuousPlan,
    CandidateFrontierExceeded,
    LearningProgressFrontierExceeded,
    ExpansionFrontierExceeded,
    ConflictingCandidateIdentity,
    ConflictingLearningProgressFocus,
    ConflictingPredictionIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentSequencePlanningResult {
    status: ExperimentSequencePlanningStatus,
    input_candidate_count: usize,
    unique_candidate_count: usize,
    input_learning_progress_count: usize,
    unique_learning_progress_count: usize,
    rejected_discrimination_count: usize,
    expansion_count: usize,
    plans_before_selection_frontier: usize,
    plans: Vec<ExperimentSequencePlan>,
}

impl ExperimentSequencePlanningResult {
    pub fn status(&self) -> ExperimentSequencePlanningStatus {
        self.status
    }

    pub fn input_candidate_count(&self) -> usize {
        self.input_candidate_count
    }

    pub fn unique_candidate_count(&self) -> usize {
        self.unique_candidate_count
    }

    pub fn input_learning_progress_count(&self) -> usize {
        self.input_learning_progress_count
    }

    pub fn unique_learning_progress_count(&self) -> usize {
        self.unique_learning_progress_count
    }

    pub fn rejected_discrimination_count(&self) -> usize {
        self.rejected_discrimination_count
    }

    pub fn expansion_count(&self) -> usize {
        self.expansion_count
    }

    pub fn plans_before_selection_frontier(&self) -> usize {
        self.plans_before_selection_frontier
    }

    pub fn plans(&self) -> &[ExperimentSequencePlan] {
        &self.plans
    }

    pub fn planned_any(&self) -> bool {
        !self.plans.is_empty()
    }

    pub fn abstained(&self) -> bool {
        !self.planned_any()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SequenceCandidateEvaluation {
    candidate: HypothesisDiscriminationCandidate,
    learning_progress: CognitiveSignal,
    discrimination_gain: CognitiveSignal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExperimentSequenceNode {
    current_state: CognitiveStructure,
    steps: Vec<PlannedExperimentStep>,
    used_experiments: Vec<AutonomousExperimentProposal>,
    cumulative_learning_progress: u32,
    cumulative_discrimination_gain: u32,
    cumulative_information_gain: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousExperimentSequencePlanning;

impl AutonomousExperimentSequencePlanning {
    fn signal(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).expect("bounded sequence planning signal")
        }
    }

    fn same_experiment_identity(
        left: &AutonomousExperimentProposal,
        right: &AutonomousExperimentProposal,
    ) -> bool {
        left.source_state() == right.source_state()
            && left.action() == right.action()
            && left.predicted_outcome() == right.predicted_outcome()
    }

    fn discrimination_gain(
        candidate: &HypothesisDiscriminationCandidate,
    ) -> Result<CognitiveSignal, ()> {
        let mut predictions = candidate.predictions().to_vec();

        predictions.sort_by(|left, right| {
            format!("{:?}", left.hypothesis())
                .cmp(&format!("{:?}", right.hypothesis()))
                .then_with(|| {
                    format!("{:?}", left.predicted_outcome())
                        .cmp(&format!("{:?}", right.predicted_outcome()))
                })
        });

        let mut canonical: Vec<CompetingHypothesisPrediction> = Vec::new();

        for prediction in predictions {
            if let Some(existing) = canonical
                .iter()
                .find(|existing| existing.hypothesis() == prediction.hypothesis())
            {
                if existing.predicted_outcome() != prediction.predicted_outcome() {
                    return Err(());
                }

                continue;
            }

            canonical.push(prediction);
        }

        if canonical.len() < 2 {
            return Ok(CognitiveSignal::zero());
        }

        let mut total_pairs = 0usize;
        let mut disagreeing_pairs = 0usize;

        for left in 0..canonical.len() {
            for right in (left + 1)..canonical.len() {
                total_pairs += 1;

                if canonical[left].predicted_outcome() != canonical[right].predicted_outcome() {
                    disagreeing_pairs += 1;
                }
            }
        }

        if total_pairs == 0 {
            return Ok(CognitiveSignal::zero());
        }

        let value = (disagreeing_pairs.saturating_mul(1000) / total_pairs).min(1000) as u16;

        Ok(Self::signal(value))
    }

    fn evaluation_order(
        left: &SequenceCandidateEvaluation,
        right: &SequenceCandidateEvaluation,
    ) -> std::cmp::Ordering {
        right
            .learning_progress
            .value()
            .cmp(&left.learning_progress.value())
            .then_with(|| {
                right
                    .discrimination_gain
                    .value()
                    .cmp(&left.discrimination_gain.value())
            })
            .then_with(|| {
                right
                    .candidate
                    .experiment()
                    .evidence()
                    .expected_information_gain()
                    .value()
                    .cmp(
                        &left
                            .candidate
                            .experiment()
                            .evidence()
                            .expected_information_gain()
                            .value(),
                    )
            })
            .then_with(|| format!("{:?}", left.candidate).cmp(&format!("{:?}", right.candidate)))
    }

    fn plan_order(
        left: &ExperimentSequencePlan,
        right: &ExperimentSequencePlan,
    ) -> std::cmp::Ordering {
        right
            .cumulative_learning_progress
            .cmp(&left.cumulative_learning_progress)
            .then_with(|| {
                right
                    .cumulative_discrimination_gain
                    .cmp(&left.cumulative_discrimination_gain)
            })
            .then_with(|| {
                right
                    .cumulative_information_gain
                    .cmp(&left.cumulative_information_gain)
            })
            .then_with(|| right.depth().cmp(&left.depth()))
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    fn empty(
        status: ExperimentSequencePlanningStatus,
        input_candidate_count: usize,
        unique_candidate_count: usize,
        input_learning_progress_count: usize,
        unique_learning_progress_count: usize,
        rejected_discrimination_count: usize,
        expansion_count: usize,
    ) -> ExperimentSequencePlanningResult {
        ExperimentSequencePlanningResult {
            status,
            input_candidate_count,
            unique_candidate_count,
            input_learning_progress_count,
            unique_learning_progress_count,
            rejected_discrimination_count,
            expansion_count,
            plans_before_selection_frontier: 0,
            plans: Vec::new(),
        }
    }

    pub fn plan(
        initial_state: &CognitiveStructure,
        candidates: &[HypothesisDiscriminationCandidate],
        learning_progress: &[ExperimentLearningProgressEstimate],
        policy: ExperimentSequencePlanningPolicy,
    ) -> ExperimentSequencePlanningResult {
        let bounds = policy.bounds();

        let input_candidate_count = candidates.len();
        let input_learning_progress_count = learning_progress.len();

        if input_candidate_count > bounds.max_input_candidates() {
            return Self::empty(
                ExperimentSequencePlanningStatus::CandidateFrontierExceeded,
                input_candidate_count,
                0,
                input_learning_progress_count,
                0,
                0,
                0,
            );
        }

        if input_learning_progress_count > bounds.max_learning_progress_estimates() {
            return Self::empty(
                ExperimentSequencePlanningStatus::LearningProgressFrontierExceeded,
                input_candidate_count,
                0,
                input_learning_progress_count,
                0,
                0,
                0,
            );
        }

        let mut canonical_candidates = candidates.to_vec();

        canonical_candidates.sort_by(|left, right| {
            format!("{:?}", left.experiment().source_state())
                .cmp(&format!("{:?}", right.experiment().source_state()))
                .then_with(|| {
                    format!("{:?}", left.experiment().action())
                        .cmp(&format!("{:?}", right.experiment().action()))
                })
                .then_with(|| {
                    format!("{:?}", left.experiment().predicted_outcome())
                        .cmp(&format!("{:?}", right.experiment().predicted_outcome()))
                })
                .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
        });

        let mut unique_candidates: Vec<HypothesisDiscriminationCandidate> = Vec::new();

        for candidate in canonical_candidates {
            if let Some(existing) = unique_candidates.iter().find(|existing| {
                Self::same_experiment_identity(existing.experiment(), candidate.experiment())
            }) {
                if existing != &candidate {
                    return Self::empty(
                        ExperimentSequencePlanningStatus::ConflictingCandidateIdentity,
                        input_candidate_count,
                        unique_candidates.len(),
                        input_learning_progress_count,
                        0,
                        0,
                        0,
                    );
                }

                continue;
            }

            unique_candidates.push(candidate);
        }

        let unique_candidate_count = unique_candidates.len();

        let mut ordered_progress = learning_progress.to_vec();

        ordered_progress.sort_by(|left, right| {
            format!("{:?}", left.source_state())
                .cmp(&format!("{:?}", right.source_state()))
                .then_with(|| format!("{:?}", left.action()).cmp(&format!("{:?}", right.action())))
                .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
        });

        let mut unique_progress: Vec<ExperimentLearningProgressEstimate> = Vec::new();

        for estimate in ordered_progress {
            if let Some(existing) = unique_progress.iter().find(|existing| {
                existing.source_state() == estimate.source_state()
                    && existing.action() == estimate.action()
            }) {
                if existing != &estimate {
                    return Self::empty(
                        ExperimentSequencePlanningStatus::ConflictingLearningProgressFocus,
                        input_candidate_count,
                        unique_candidate_count,
                        input_learning_progress_count,
                        unique_progress.len(),
                        0,
                        0,
                    );
                }

                continue;
            }

            unique_progress.push(estimate);
        }

        let unique_learning_progress_count = unique_progress.len();

        let mut rejected_discrimination_count = 0usize;
        let mut evaluations = Vec::new();

        for candidate in unique_candidates {
            let discrimination_gain = match Self::discrimination_gain(&candidate) {
                Ok(value) => value,
                Err(()) => {
                    return Self::empty(
                        ExperimentSequencePlanningStatus::ConflictingPredictionIdentity,
                        input_candidate_count,
                        unique_candidate_count,
                        input_learning_progress_count,
                        unique_learning_progress_count,
                        rejected_discrimination_count,
                        0,
                    );
                }
            };

            if discrimination_gain.value() < policy.minimum_discrimination_gain().value() {
                rejected_discrimination_count += 1;
                continue;
            }

            let progress = unique_progress
                .iter()
                .find(|estimate| {
                    estimate.source_state() == candidate.experiment().source_state()
                        && estimate.action() == candidate.experiment().action()
                })
                .map(ExperimentLearningProgressEstimate::learning_progress)
                .unwrap_or_else(CognitiveSignal::zero);

            evaluations.push(SequenceCandidateEvaluation {
                candidate,
                learning_progress: progress,
                discrimination_gain,
            });
        }

        evaluations.sort_by(Self::evaluation_order);

        let mut nodes = vec![ExperimentSequenceNode {
            current_state: initial_state.clone(),
            steps: Vec::new(),
            used_experiments: Vec::new(),
            cumulative_learning_progress: 0,
            cumulative_discrimination_gain: 0,
            cumulative_information_gain: 0,
        }];

        let mut plans = Vec::new();
        let mut expansion_count = 0usize;

        for _ in 0..bounds.max_plan_depth() {
            let mut next_nodes = Vec::new();

            for node in nodes {
                let mut options: Vec<SequenceCandidateEvaluation> = evaluations
                    .iter()
                    .filter(|evaluation| {
                        evaluation.candidate.experiment().source_state() == &node.current_state
                            && !node.used_experiments.iter().any(|used| {
                                Self::same_experiment_identity(
                                    used,
                                    evaluation.candidate.experiment(),
                                )
                            })
                    })
                    .cloned()
                    .collect();

                options.sort_by(Self::evaluation_order);

                for evaluation in options {
                    if expansion_count >= bounds.max_expansions() {
                        return Self::empty(
                            ExperimentSequencePlanningStatus::ExpansionFrontierExceeded,
                            input_candidate_count,
                            unique_candidate_count,
                            input_learning_progress_count,
                            unique_learning_progress_count,
                            rejected_discrimination_count,
                            expansion_count,
                        );
                    }

                    expansion_count += 1;

                    let experiment = evaluation.candidate.experiment();

                    let mut steps = node.steps.clone();

                    steps.push(PlannedExperimentStep {
                        candidate: evaluation.candidate.clone(),
                        learning_progress: evaluation.learning_progress,
                        discrimination_gain: evaluation.discrimination_gain,
                    });

                    let mut used_experiments = node.used_experiments.clone();

                    used_experiments.push(experiment.clone());

                    let learning_sum = node
                        .cumulative_learning_progress
                        .saturating_add(u32::from(evaluation.learning_progress.value()));

                    let discrimination_sum = node
                        .cumulative_discrimination_gain
                        .saturating_add(u32::from(evaluation.discrimination_gain.value()));

                    let information_sum = node.cumulative_information_gain.saturating_add(
                        u32::from(experiment.evidence().expected_information_gain().value()),
                    );

                    let next_state = experiment.predicted_outcome().clone();

                    plans.push(ExperimentSequencePlan {
                        initial_state: initial_state.clone(),
                        steps: steps.clone(),
                        cumulative_learning_progress: learning_sum,
                        cumulative_discrimination_gain: discrimination_sum,
                        cumulative_information_gain: information_sum,
                    });

                    next_nodes.push(ExperimentSequenceNode {
                        current_state: next_state,
                        steps,
                        used_experiments,
                        cumulative_learning_progress: learning_sum,
                        cumulative_discrimination_gain: discrimination_sum,
                        cumulative_information_gain: information_sum,
                    });
                }
            }

            if next_nodes.is_empty() {
                break;
            }

            nodes = next_nodes;
        }

        if plans.is_empty() {
            return Self::empty(
                ExperimentSequencePlanningStatus::NoContinuousPlan,
                input_candidate_count,
                unique_candidate_count,
                input_learning_progress_count,
                unique_learning_progress_count,
                rejected_discrimination_count,
                expansion_count,
            );
        }

        plans.sort_by(Self::plan_order);
        plans.dedup();

        let plans_before_selection_frontier = plans.len();

        plans.truncate(bounds.max_selected_plans());

        ExperimentSequencePlanningResult {
            status: ExperimentSequencePlanningStatus::Planned,
            input_candidate_count,
            unique_candidate_count,
            input_learning_progress_count,
            unique_learning_progress_count,
            rejected_discrimination_count,
            expansion_count,
            plans_before_selection_frontier,
            plans,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousExperimentSequencePlanning;

impl UniversalAutonomousExperimentSequencePlanning {
    pub fn evaluate(
        initial_state: &CognitiveStructure,
        candidates: &[HypothesisDiscriminationCandidate],
        learning_progress: &[ExperimentLearningProgressEstimate],
        policy: ExperimentSequencePlanningPolicy,
    ) -> ExperimentSequencePlanningResult {
        AutonomousExperimentSequencePlanning::plan(
            initial_state,
            candidates,
            learning_progress,
            policy,
        )
    }
}

#[cfg(test)]
mod experiment_sequence_planning_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn p(hypothesis: u64, outcome: u64) -> CompetingHypothesisPrediction {
        CompetingHypothesisPrediction::new(a(hypothesis), a(outcome), s(900)).unwrap()
    }

    fn candidate(
        source: u64,
        action: u64,
        outcome: u64,
        information: u16,
        predictions: Vec<CompetingHypothesisPrediction>,
    ) -> HypothesisDiscriminationCandidate {
        HypothesisDiscriminationCandidate::new(
            AutonomousExperimentProposal::new(
                a(source),
                a(action),
                a(outcome),
                ExperimentEvidence::new(s(800), s(information), s(900), s(900), s(100)).unwrap(),
            ),
            predictions,
        )
        .unwrap()
    }

    fn two_way(
        source: u64,
        action: u64,
        outcome: u64,
        information: u16,
    ) -> HypothesisDiscriminationCandidate {
        candidate(
            source,
            action,
            outcome,
            information,
            vec![p(1, 100), p(2, 101)],
        )
    }

    fn progress(state: u64, action: u64, value: u16) -> ExperimentLearningProgressEstimate {
        ExperimentLearningProgressEstimate {
            source_state: a(state),
            action: a(action),
            qualifying_sample_count: 2,
            mean_error_reduction: s(value),
            mean_uncertainty_reduction: s(value),
            mean_evidence_confidence: s(900),
            learning_progress: s(value),
        }
    }

    fn policy() -> ExperimentSequencePlanningPolicy {
        ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(16, 16, 4, 64, 8).unwrap(),
            s(500),
        )
        .unwrap()
    }

    #[test]
    fn sequence_policy_requires_positive_hard_bounds_and_discrimination_gate() {
        assert_eq!(ExperimentSequencePlanningBounds::new(0, 1, 1, 1, 1), None);

        assert_eq!(
            ExperimentSequencePlanningPolicy::new(
                ExperimentSequencePlanningBounds::new(1, 1, 1, 1, 1).unwrap(),
                s(0),
            ),
            None
        );
    }

    #[test]
    fn exact_predicted_state_continuity_builds_multi_step_experiment_sequence() {
        let first = two_way(1, 10, 2, 800);

        let second = two_way(2, 20, 3, 800);

        let result =
            AutonomousExperimentSequencePlanning::plan(&a(1), &[first, second], &[], policy());

        assert_eq!(result.status(), ExperimentSequencePlanningStatus::Planned);

        assert_eq!(result.plans()[0].depth(), 2);

        assert_eq!(result.plans()[0].initial_state(), &a(1));

        assert_eq!(result.plans()[0].terminal_state(), &a(3));
    }

    #[test]
    fn disconnected_candidate_cannot_be_spliced_into_sequence() {
        let first = two_way(1, 10, 2, 800);

        let disconnected = two_way(99, 20, 3, 1000);

        let result = AutonomousExperimentSequencePlanning::plan(
            &a(1),
            &[first, disconnected],
            &[],
            policy(),
        );

        assert_eq!(result.plans()[0].depth(), 1);

        assert_eq!(result.plans()[0].terminal_state(), &a(2));
    }

    #[test]
    fn measured_learning_progress_ranks_before_generic_information_value() {
        let high_information = two_way(1, 10, 2, 1000);

        let learning = two_way(1, 11, 3, 600);

        let bounded = ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(16, 16, 1, 32, 8).unwrap(),
            s(500),
        )
        .unwrap();

        let result = AutonomousExperimentSequencePlanning::plan(
            &a(1),
            &[high_information, learning],
            &[progress(1, 11, 300)],
            bounded,
        );

        assert_eq!(result.plans()[0].steps()[0].action(), &a(11));

        assert_eq!(result.plans()[0].steps()[0].learning_progress(), s(300));
    }

    #[test]
    fn discrimination_gain_ranks_before_information_when_progress_is_equal() {
        let partial = candidate(1, 10, 2, 1000, vec![p(1, 100), p(2, 100), p(3, 101)]);

        let full = candidate(1, 11, 3, 600, vec![p(1, 110), p(2, 111)]);

        let bounded = ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(16, 16, 1, 32, 8).unwrap(),
            s(500),
        )
        .unwrap();

        let result =
            AutonomousExperimentSequencePlanning::plan(&a(1), &[partial, full], &[], bounded);

        assert_eq!(result.plans()[0].steps()[0].action(), &a(11));

        assert_eq!(result.plans()[0].steps()[0].discrimination_gain(), s(1000));
    }

    #[test]
    fn exact_duplicate_candidate_is_deduplicated_without_fake_branching() {
        let item = two_way(1, 10, 2, 800);

        let result =
            AutonomousExperimentSequencePlanning::plan(&a(1), &[item.clone(), item], &[], policy());

        assert_eq!(result.input_candidate_count(), 2);

        assert_eq!(result.unique_candidate_count(), 1);

        assert_eq!(result.expansion_count(), 1);
    }

    #[test]
    fn conflicting_semantic_experiment_identity_abstains_atomically() {
        let first = two_way(1, 10, 2, 700);

        let second = two_way(1, 10, 2, 900);

        let result =
            AutonomousExperimentSequencePlanning::plan(&a(1), &[first, second], &[], policy());

        assert_eq!(
            result.status(),
            ExperimentSequencePlanningStatus::ConflictingCandidateIdentity
        );

        assert!(result.plans().is_empty());
    }

    #[test]
    fn conflicting_learning_progress_focus_abstains_atomically() {
        let result = AutonomousExperimentSequencePlanning::plan(
            &a(1),
            &[two_way(1, 10, 2, 800)],
            &[progress(1, 10, 200), progress(1, 10, 300)],
            policy(),
        );

        assert_eq!(
            result.status(),
            ExperimentSequencePlanningStatus::ConflictingLearningProgressFocus
        );

        assert!(result.abstained());
    }

    #[test]
    fn contradictory_prediction_identity_abstains_before_planning() {
        let conflicting = candidate(1, 10, 2, 900, vec![p(1, 100), p(1, 101), p(2, 102)]);

        let result =
            AutonomousExperimentSequencePlanning::plan(&a(1), &[conflicting], &[], policy());

        assert_eq!(
            result.status(),
            ExperimentSequencePlanningStatus::ConflictingPredictionIdentity
        );

        assert!(result.abstained());
    }

    #[test]
    fn exact_experiment_cannot_repeat_forever_inside_cycle() {
        let cycle = two_way(1, 10, 1, 900);

        let result = AutonomousExperimentSequencePlanning::plan(&a(1), &[cycle], &[], policy());

        assert_eq!(result.plans()[0].depth(), 1);

        assert_eq!(result.expansion_count(), 1);
    }

    #[test]
    fn candidate_progress_depth_expansion_and_plan_frontiers_are_hard_bounded() {
        let item = two_way(1, 10, 2, 900);

        let candidate_bound = ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(1, 16, 4, 64, 8).unwrap(),
            s(500),
        )
        .unwrap();

        let candidate_result = AutonomousExperimentSequencePlanning::plan(
            &a(1),
            &[item.clone(), two_way(1, 11, 3, 900)],
            &[],
            candidate_bound,
        );

        assert_eq!(
            candidate_result.status(),
            ExperimentSequencePlanningStatus::CandidateFrontierExceeded
        );

        let progress_bound = ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(16, 1, 4, 64, 8).unwrap(),
            s(500),
        )
        .unwrap();

        let progress_result = AutonomousExperimentSequencePlanning::plan(
            &a(1),
            std::slice::from_ref(&item),
            &[progress(1, 10, 100), progress(2, 20, 100)],
            progress_bound,
        );

        assert_eq!(
            progress_result.status(),
            ExperimentSequencePlanningStatus::LearningProgressFrontierExceeded
        );

        let expansion_bound = ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(16, 16, 4, 1, 8).unwrap(),
            s(500),
        )
        .unwrap();

        let expansion_result = AutonomousExperimentSequencePlanning::plan(
            &a(1),
            &[item.clone(), two_way(1, 11, 3, 900)],
            &[],
            expansion_bound,
        );

        assert_eq!(
            expansion_result.status(),
            ExperimentSequencePlanningStatus::ExpansionFrontierExceeded
        );

        assert!(expansion_result.abstained());

        let depth_bound = ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(16, 16, 1, 64, 1).unwrap(),
            s(500),
        )
        .unwrap();

        let depth_result = AutonomousExperimentSequencePlanning::plan(
            &a(1),
            &[item, two_way(2, 20, 3, 900), two_way(1, 11, 4, 800)],
            &[],
            depth_bound,
        );

        assert_eq!(depth_result.plans().len(), 1);

        assert_eq!(depth_result.plans()[0].depth(), 1);

        assert!(depth_result.plans_before_selection_frontier() >= 2);
    }

    #[test]
    fn sequence_planning_is_order_invariant_non_mutating_and_facade_equivalent() {
        let candidates = vec![
            two_way(1, 10, 2, 800),
            two_way(1, 11, 3, 700),
            two_way(2, 20, 4, 800),
        ];

        let progress_values = vec![
            progress(1, 10, 200),
            progress(1, 11, 100),
            progress(2, 20, 300),
        ];

        let before_candidates = candidates.clone();

        let before_progress = progress_values.clone();

        let mut reversed_candidates = candidates.clone();

        reversed_candidates.reverse();

        let mut reversed_progress = progress_values.clone();

        reversed_progress.reverse();

        let p = policy();

        let direct =
            AutonomousExperimentSequencePlanning::plan(&a(1), &candidates, &progress_values, p);

        let reordered = AutonomousExperimentSequencePlanning::plan(
            &a(1),
            &reversed_candidates,
            &reversed_progress,
            p,
        );

        let facade = UniversalAutonomousExperimentSequencePlanning::evaluate(
            &a(1),
            &candidates,
            &progress_values,
            p,
        );

        let repeated = UniversalAutonomousExperimentSequencePlanning::evaluate(
            &a(1),
            &candidates,
            &progress_values,
            p,
        );

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(candidates, before_candidates);
        assert_eq!(progress_values, before_progress);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StopContinueExperimentationBounds {
    max_beliefs: usize,
    max_plans: usize,
    max_experiment_cycles: usize,
}

impl StopContinueExperimentationBounds {
    pub fn new(max_beliefs: usize, max_plans: usize, max_experiment_cycles: usize) -> Option<Self> {
        if max_beliefs < 2 || max_plans == 0 || max_experiment_cycles == 0 {
            return None;
        }

        Some(Self {
            max_beliefs,
            max_plans,
            max_experiment_cycles,
        })
    }

    pub fn max_beliefs(self) -> usize {
        self.max_beliefs
    }

    pub fn max_plans(self) -> usize {
        self.max_plans
    }

    pub fn max_experiment_cycles(self) -> usize {
        self.max_experiment_cycles
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StopContinueExperimentationThresholds {
    minimum_active_belief_confidence: CognitiveSignal,
    minimum_resolution_confidence: CognitiveSignal,
    minimum_resolution_margin: CognitiveSignal,
    minimum_learning_progress_to_continue: CognitiveSignal,
    minimum_information_gain_to_continue: CognitiveSignal,
    minimum_discrimination_gain: CognitiveSignal,
}

impl StopContinueExperimentationThresholds {
    pub fn new(
        minimum_active_belief_confidence: CognitiveSignal,
        minimum_resolution_confidence: CognitiveSignal,
        minimum_resolution_margin: CognitiveSignal,
        minimum_learning_progress_to_continue: CognitiveSignal,
        minimum_information_gain_to_continue: CognitiveSignal,
        minimum_discrimination_gain: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_active_belief_confidence == CognitiveSignal::zero()
            || minimum_resolution_confidence == CognitiveSignal::zero()
            || minimum_resolution_margin == CognitiveSignal::zero()
            || minimum_learning_progress_to_continue == CognitiveSignal::zero()
            || minimum_information_gain_to_continue == CognitiveSignal::zero()
            || minimum_discrimination_gain == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_active_belief_confidence,
            minimum_resolution_confidence,
            minimum_resolution_margin,
            minimum_learning_progress_to_continue,
            minimum_information_gain_to_continue,
            minimum_discrimination_gain,
        })
    }

    pub fn minimum_active_belief_confidence(self) -> CognitiveSignal {
        self.minimum_active_belief_confidence
    }

    pub fn minimum_resolution_confidence(self) -> CognitiveSignal {
        self.minimum_resolution_confidence
    }

    pub fn minimum_resolution_margin(self) -> CognitiveSignal {
        self.minimum_resolution_margin
    }

    pub fn minimum_learning_progress_to_continue(self) -> CognitiveSignal {
        self.minimum_learning_progress_to_continue
    }

    pub fn minimum_information_gain_to_continue(self) -> CognitiveSignal {
        self.minimum_information_gain_to_continue
    }

    pub fn minimum_discrimination_gain(self) -> CognitiveSignal {
        self.minimum_discrimination_gain
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StopContinueExperimentationPolicy {
    bounds: StopContinueExperimentationBounds,
    thresholds: StopContinueExperimentationThresholds,
}

impl StopContinueExperimentationPolicy {
    pub fn new(
        bounds: StopContinueExperimentationBounds,
        thresholds: StopContinueExperimentationThresholds,
    ) -> Self {
        Self { bounds, thresholds }
    }

    pub fn bounds(self) -> StopContinueExperimentationBounds {
        self.bounds
    }

    pub fn thresholds(self) -> StopContinueExperimentationThresholds {
        self.thresholds
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExperimentContinuationBasis {
    MeasuredLearningProgress,
    ExpectedInformationGain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopContinueExperimentationDecision {
    ContinueExperimentation,
    StopResolved,
    StopExperimentBudgetExhausted,
    StopNoApplicablePlan,
    StopLearningStalled,
    AbstainBeliefFrontierExceeded,
    AbstainPlanFrontierExceeded,
    AbstainDuplicateBeliefIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StopContinueExperimentationResult {
    decision: StopContinueExperimentationDecision,
    input_belief_count: usize,
    active_belief_count: usize,
    input_plan_count: usize,
    applicable_plan_count: usize,
    rejected_discrimination_count: usize,
    current_experiment_cycle: usize,
    resolved_winner: Option<CognitiveStructure>,
    continuation_basis: Option<ExperimentContinuationBasis>,
    next_plan: Option<ExperimentSequencePlan>,
}

impl StopContinueExperimentationResult {
    pub fn decision(&self) -> StopContinueExperimentationDecision {
        self.decision
    }

    pub fn input_belief_count(&self) -> usize {
        self.input_belief_count
    }

    pub fn active_belief_count(&self) -> usize {
        self.active_belief_count
    }

    pub fn input_plan_count(&self) -> usize {
        self.input_plan_count
    }

    pub fn applicable_plan_count(&self) -> usize {
        self.applicable_plan_count
    }

    pub fn rejected_discrimination_count(&self) -> usize {
        self.rejected_discrimination_count
    }

    pub fn current_experiment_cycle(&self) -> usize {
        self.current_experiment_cycle
    }

    pub fn resolved_winner(&self) -> Option<&CognitiveStructure> {
        self.resolved_winner.as_ref()
    }

    pub fn continuation_basis(&self) -> Option<ExperimentContinuationBasis> {
        self.continuation_basis
    }

    pub fn next_plan(&self) -> Option<&ExperimentSequencePlan> {
        self.next_plan.as_ref()
    }

    pub fn continuing(&self) -> bool {
        self.decision == StopContinueExperimentationDecision::ContinueExperimentation
    }

    pub fn stopped(&self) -> bool {
        matches!(
            self.decision,
            StopContinueExperimentationDecision::StopResolved
                | StopContinueExperimentationDecision::StopExperimentBudgetExhausted
                | StopContinueExperimentationDecision::StopNoApplicablePlan
                | StopContinueExperimentationDecision::StopLearningStalled
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousStopContinueExperimentation;

impl AutonomousStopContinueExperimentation {
    fn plan_order(
        left: &ExperimentSequencePlan,
        right: &ExperimentSequencePlan,
    ) -> std::cmp::Ordering {
        right
            .cumulative_learning_progress()
            .cmp(&left.cumulative_learning_progress())
            .then_with(|| {
                let right_discrimination = right
                    .steps()
                    .first()
                    .map(PlannedExperimentStep::discrimination_gain)
                    .unwrap_or_else(CognitiveSignal::zero);

                let left_discrimination = left
                    .steps()
                    .first()
                    .map(PlannedExperimentStep::discrimination_gain)
                    .unwrap_or_else(CognitiveSignal::zero);

                right_discrimination
                    .value()
                    .cmp(&left_discrimination.value())
            })
            .then_with(|| {
                right
                    .cumulative_information_gain()
                    .cmp(&left.cumulative_information_gain())
            })
            .then_with(|| {
                right
                    .cumulative_discrimination_gain()
                    .cmp(&left.cumulative_discrimination_gain())
            })
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    pub fn control(
        current_state: &CognitiveStructure,
        beliefs: &[HypothesisBeliefState],
        plans: &[ExperimentSequencePlan],
        current_experiment_cycle: usize,
        policy: StopContinueExperimentationPolicy,
    ) -> StopContinueExperimentationResult {
        let bounds = policy.bounds();

        let thresholds = policy.thresholds();

        let input_belief_count = beliefs.len();

        let input_plan_count = plans.len();

        if input_belief_count > bounds.max_beliefs() {
            return StopContinueExperimentationResult {
                decision: StopContinueExperimentationDecision::AbstainBeliefFrontierExceeded,
                input_belief_count,
                active_belief_count: 0,
                input_plan_count,
                applicable_plan_count: 0,
                rejected_discrimination_count: 0,
                current_experiment_cycle,
                resolved_winner: None,
                continuation_basis: None,
                next_plan: None,
            };
        }

        let mut canonical_beliefs = beliefs.to_vec();

        canonical_beliefs.sort_by(|left, right| {
            format!("{:?}", left.hypothesis()).cmp(&format!("{:?}", right.hypothesis()))
        });

        for index in 1..canonical_beliefs.len() {
            if canonical_beliefs[index - 1].hypothesis() == canonical_beliefs[index].hypothesis() {
                return StopContinueExperimentationResult {
                    decision: StopContinueExperimentationDecision::AbstainDuplicateBeliefIdentity,
                    input_belief_count,
                    active_belief_count: 0,
                    input_plan_count,
                    applicable_plan_count: 0,
                    rejected_discrimination_count: 0,
                    current_experiment_cycle,
                    resolved_winner: None,
                    continuation_basis: None,
                    next_plan: None,
                };
            }
        }

        let mut active: Vec<HypothesisBeliefState> = canonical_beliefs
            .into_iter()
            .filter(|belief| {
                belief.active()
                    && belief.confidence().value()
                        >= thresholds.minimum_active_belief_confidence().value()
            })
            .collect();

        active.sort_by(|left, right| {
            right
                .confidence()
                .value()
                .cmp(&left.confidence().value())
                .then_with(|| {
                    format!("{:?}", left.hypothesis()).cmp(&format!("{:?}", right.hypothesis()))
                })
        });

        let active_belief_count = active.len();

        if active_belief_count < 2 {
            return StopContinueExperimentationResult {
                decision: StopContinueExperimentationDecision::StopResolved,
                input_belief_count,
                active_belief_count,
                input_plan_count,
                applicable_plan_count: 0,
                rejected_discrimination_count: 0,
                current_experiment_cycle,
                resolved_winner: active.first().map(|belief| belief.hypothesis().clone()),
                continuation_basis: None,
                next_plan: None,
            };
        }

        let winner = &active[0];

        let runner_up = &active[1];

        let confidence_margin = winner
            .confidence()
            .value()
            .saturating_sub(runner_up.confidence().value());

        if winner.confidence().value() >= thresholds.minimum_resolution_confidence().value()
            && confidence_margin >= thresholds.minimum_resolution_margin().value()
        {
            return StopContinueExperimentationResult {
                decision: StopContinueExperimentationDecision::StopResolved,
                input_belief_count,
                active_belief_count,
                input_plan_count,
                applicable_plan_count: 0,
                rejected_discrimination_count: 0,
                current_experiment_cycle,
                resolved_winner: Some(winner.hypothesis().clone()),
                continuation_basis: None,
                next_plan: None,
            };
        }

        if current_experiment_cycle >= bounds.max_experiment_cycles() {
            return StopContinueExperimentationResult {
                decision: StopContinueExperimentationDecision::StopExperimentBudgetExhausted,
                input_belief_count,
                active_belief_count,
                input_plan_count,
                applicable_plan_count: 0,
                rejected_discrimination_count: 0,
                current_experiment_cycle,
                resolved_winner: None,
                continuation_basis: None,
                next_plan: None,
            };
        }

        if input_plan_count > bounds.max_plans() {
            return StopContinueExperimentationResult {
                decision: StopContinueExperimentationDecision::AbstainPlanFrontierExceeded,
                input_belief_count,
                active_belief_count,
                input_plan_count,
                applicable_plan_count: 0,
                rejected_discrimination_count: 0,
                current_experiment_cycle,
                resolved_winner: None,
                continuation_basis: None,
                next_plan: None,
            };
        }

        let mut rejected_discrimination_count = 0;

        let mut applicable = Vec::new();

        for plan in plans {
            if plan.initial_state() != current_state {
                continue;
            }

            let Some(first_step) = plan.steps().first() else {
                continue;
            };

            if first_step.discrimination_gain().value()
                < thresholds.minimum_discrimination_gain().value()
            {
                rejected_discrimination_count += 1;
                continue;
            }

            applicable.push(plan.clone());
        }

        applicable.sort_by(Self::plan_order);

        applicable.dedup();

        let applicable_plan_count = applicable.len();

        let Some(best_plan) = applicable.first().cloned() else {
            return StopContinueExperimentationResult {
                decision: StopContinueExperimentationDecision::StopNoApplicablePlan,
                input_belief_count,
                active_belief_count,
                input_plan_count,
                applicable_plan_count,
                rejected_discrimination_count,
                current_experiment_cycle,
                resolved_winner: None,
                continuation_basis: None,
                next_plan: None,
            };
        };

        let learning_progress = best_plan.cumulative_learning_progress();

        let information_gain = best_plan.cumulative_information_gain();

        let learning_threshold =
            u32::from(thresholds.minimum_learning_progress_to_continue().value());

        let information_threshold =
            u32::from(thresholds.minimum_information_gain_to_continue().value());

        let continuation_basis = if learning_progress >= learning_threshold {
            Some(ExperimentContinuationBasis::MeasuredLearningProgress)
        } else if information_gain >= information_threshold {
            Some(ExperimentContinuationBasis::ExpectedInformationGain)
        } else {
            None
        };

        let Some(continuation_basis) = continuation_basis else {
            return StopContinueExperimentationResult {
                decision: StopContinueExperimentationDecision::StopLearningStalled,
                input_belief_count,
                active_belief_count,
                input_plan_count,
                applicable_plan_count,
                rejected_discrimination_count,
                current_experiment_cycle,
                resolved_winner: None,
                continuation_basis: None,
                next_plan: None,
            };
        };

        StopContinueExperimentationResult {
            decision: StopContinueExperimentationDecision::ContinueExperimentation,
            input_belief_count,
            active_belief_count,
            input_plan_count,
            applicable_plan_count,
            rejected_discrimination_count,
            current_experiment_cycle,
            resolved_winner: None,
            continuation_basis: Some(continuation_basis),
            next_plan: Some(best_plan),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousStopContinueExperimentation;

impl UniversalAutonomousStopContinueExperimentation {
    pub fn evaluate(
        current_state: &CognitiveStructure,
        beliefs: &[HypothesisBeliefState],
        plans: &[ExperimentSequencePlan],
        current_experiment_cycle: usize,
        policy: StopContinueExperimentationPolicy,
    ) -> StopContinueExperimentationResult {
        AutonomousStopContinueExperimentation::control(
            current_state,
            beliefs,
            plans,
            current_experiment_cycle,
            policy,
        )
    }
}

#[cfg(test)]
mod stop_continue_experimentation_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn belief(hypothesis: u64, confidence: u16) -> HypothesisBeliefState {
        HypothesisBeliefState::new(a(hypothesis), s(confidence)).unwrap()
    }

    fn suspended(hypothesis: u64, confidence: u16) -> HypothesisBeliefState {
        let mut value = belief(hypothesis, confidence);

        value.availability = HypothesisBeliefAvailability::Suspended;

        value
    }

    fn prediction(hypothesis: u64, outcome: u64) -> CompetingHypothesisPrediction {
        CompetingHypothesisPrediction::new(a(hypothesis), a(outcome), s(900)).unwrap()
    }

    fn candidate(
        source: u64,
        action: u64,
        outcome: u64,
        information: u16,
    ) -> HypothesisDiscriminationCandidate {
        HypothesisDiscriminationCandidate::new(
            AutonomousExperimentProposal::new(
                a(source),
                a(action),
                a(outcome),
                ExperimentEvidence::new(s(800), s(information), s(900), s(900), s(100)).unwrap(),
            ),
            vec![prediction(1, 100), prediction(2, 101)],
        )
        .unwrap()
    }

    fn plan(
        initial: u64,
        action: u64,
        terminal: u64,
        learning: u16,
        discrimination: u16,
        information: u16,
    ) -> ExperimentSequencePlan {
        let candidate = candidate(initial, action, terminal, information);

        ExperimentSequencePlan {
            initial_state: a(initial),
            steps: vec![PlannedExperimentStep {
                candidate,
                learning_progress: s(learning),
                discrimination_gain: s(discrimination),
            }],
            cumulative_learning_progress: u32::from(learning),
            cumulative_discrimination_gain: u32::from(discrimination),
            cumulative_information_gain: u32::from(information),
        }
    }

    fn thresholds() -> StopContinueExperimentationThresholds {
        StopContinueExperimentationThresholds::new(s(500), s(850), s(250), s(100), s(600), s(500))
            .unwrap()
    }

    fn policy() -> StopContinueExperimentationPolicy {
        StopContinueExperimentationPolicy::new(
            StopContinueExperimentationBounds::new(16, 16, 8).unwrap(),
            thresholds(),
        )
    }

    #[test]
    fn stop_continue_contract_requires_positive_bounds_and_thresholds() {
        assert_eq!(StopContinueExperimentationBounds::new(1, 1, 1), None);

        assert_eq!(StopContinueExperimentationBounds::new(2, 0, 1), None);

        assert_eq!(
            StopContinueExperimentationThresholds::new(
                s(500),
                s(850),
                s(250),
                s(0),
                s(600),
                s(500),
            ),
            None
        );
    }

    #[test]
    fn single_active_hypothesis_stops_as_resolved_and_suspended_belief_is_excluded() {
        let result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), suspended(2, 900)],
            &[],
            0,
            policy(),
        );

        assert_eq!(
            result.decision(),
            StopContinueExperimentationDecision::StopResolved
        );

        assert_eq!(result.active_belief_count(), 1);

        assert_eq!(result.resolved_winner(), Some(&a(1)));
    }

    #[test]
    fn dominant_high_confidence_belief_stops_without_consuming_another_experiment() {
        let result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 900), belief(2, 600)],
            &[plan(1, 10, 2, 500, 900, 900)],
            0,
            policy(),
        );

        assert_eq!(
            result.decision(),
            StopContinueExperimentationDecision::StopResolved
        );

        assert_eq!(result.resolved_winner(), Some(&a(1)));

        assert!(result.next_plan().is_none());
    }

    #[test]
    fn unresolved_competition_with_measured_learning_progress_continues() {
        let result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[plan(1, 10, 2, 250, 900, 700)],
            1,
            policy(),
        );

        assert!(result.continuing());

        assert_eq!(
            result.continuation_basis(),
            Some(ExperimentContinuationBasis::MeasuredLearningProgress)
        );

        assert!(result.next_plan().is_some());
    }

    #[test]
    fn measured_learning_progress_ranks_before_higher_generic_information_value() {
        let learning = plan(1, 10, 2, 200, 900, 600);

        let information = plan(1, 11, 3, 0, 900, 1000);

        let result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[information, learning.clone()],
            1,
            policy(),
        );

        assert_eq!(result.next_plan().unwrap(), &learning);

        assert_eq!(
            result.continuation_basis(),
            Some(ExperimentContinuationBasis::MeasuredLearningProgress)
        );
    }

    #[test]
    fn novel_high_information_plan_can_continue_before_learning_progress_history_exists() {
        let result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[plan(1, 10, 2, 0, 900, 900)],
            0,
            policy(),
        );

        assert!(result.continuing());

        assert_eq!(
            result.continuation_basis(),
            Some(ExperimentContinuationBasis::ExpectedInformationGain)
        );
    }

    #[test]
    fn low_learning_progress_and_low_information_stop_stalled_experimentation() {
        let result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[plan(1, 10, 2, 50, 900, 500)],
            2,
            policy(),
        );

        assert_eq!(
            result.decision(),
            StopContinueExperimentationDecision::StopLearningStalled
        );

        assert!(result.next_plan().is_none());
    }

    #[test]
    fn experiment_budget_exhaustion_stops_before_plan_selection() {
        let result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[plan(1, 10, 2, 500, 900, 1000)],
            8,
            policy(),
        );

        assert_eq!(
            result.decision(),
            StopContinueExperimentationDecision::StopExperimentBudgetExhausted
        );

        assert!(result.next_plan().is_none());
    }

    #[test]
    fn only_plans_rooted_at_exact_current_state_are_applicable() {
        let result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[plan(99, 10, 2, 500, 900, 1000)],
            1,
            policy(),
        );

        assert_eq!(
            result.decision(),
            StopContinueExperimentationDecision::StopNoApplicablePlan
        );

        assert_eq!(result.applicable_plan_count(), 0);
    }

    #[test]
    fn weak_hypothesis_discrimination_cannot_justify_continuing_experimentation() {
        let result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[plan(1, 10, 2, 500, 400, 1000)],
            1,
            policy(),
        );

        assert_eq!(
            result.decision(),
            StopContinueExperimentationDecision::StopNoApplicablePlan
        );

        assert_eq!(result.rejected_discrimination_count(), 1);
    }

    #[test]
    fn identity_and_input_frontier_failures_abstain_atomically() {
        let belief_bound = StopContinueExperimentationPolicy::new(
            StopContinueExperimentationBounds::new(2, 16, 8).unwrap(),
            thresholds(),
        );

        let belief_result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(2, 680), belief(3, 660)],
            &[],
            0,
            belief_bound,
        );

        assert_eq!(
            belief_result.decision(),
            StopContinueExperimentationDecision::AbstainBeliefFrontierExceeded
        );

        let duplicate_result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(1, 680)],
            &[],
            0,
            policy(),
        );

        assert_eq!(
            duplicate_result.decision(),
            StopContinueExperimentationDecision::AbstainDuplicateBeliefIdentity
        );

        let plan_bound = StopContinueExperimentationPolicy::new(
            StopContinueExperimentationBounds::new(16, 1, 8).unwrap(),
            thresholds(),
        );

        let plan_result = AutonomousStopContinueExperimentation::control(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[plan(1, 10, 2, 200, 900, 800), plan(1, 11, 3, 200, 900, 800)],
            1,
            plan_bound,
        );

        assert_eq!(
            plan_result.decision(),
            StopContinueExperimentationDecision::AbstainPlanFrontierExceeded
        );

        assert!(plan_result.next_plan().is_none());
    }

    #[test]
    fn stop_continue_control_is_order_invariant_non_mutating_and_facade_equivalent() {
        let beliefs = vec![belief(2, 680), belief(1, 700), belief(3, 660)];

        let plans = vec![plan(1, 10, 2, 200, 900, 700), plan(1, 11, 3, 100, 900, 900)];

        let before_beliefs = beliefs.clone();

        let before_plans = plans.clone();

        let mut reversed_beliefs = beliefs.clone();

        reversed_beliefs.reverse();

        let mut reversed_plans = plans.clone();

        reversed_plans.reverse();

        let p = policy();

        let direct = AutonomousStopContinueExperimentation::control(&a(1), &beliefs, &plans, 1, p);

        let reordered = AutonomousStopContinueExperimentation::control(
            &a(1),
            &reversed_beliefs,
            &reversed_plans,
            1,
            p,
        );

        let facade =
            UniversalAutonomousStopContinueExperimentation::evaluate(&a(1), &beliefs, &plans, 1, p);

        let repeated =
            UniversalAutonomousStopContinueExperimentation::evaluate(&a(1), &beliefs, &plans, 1, p);

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(facade, repeated);
        assert_eq!(beliefs, before_beliefs);
        assert_eq!(plans, before_plans);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IntegratedAutonomousExperimentationPolicy {
    proposal: BeliefDrivenExperimentProposalPolicy,
    learning_progress: LearningProgressPolicy,
    sequence_planning: ExperimentSequencePlanningPolicy,
    stop_continue: StopContinueExperimentationPolicy,
}

impl IntegratedAutonomousExperimentationPolicy {
    pub fn new(
        proposal: BeliefDrivenExperimentProposalPolicy,
        learning_progress: LearningProgressPolicy,
        sequence_planning: ExperimentSequencePlanningPolicy,
        stop_continue: StopContinueExperimentationPolicy,
    ) -> Option<Self> {
        if proposal.bounds().max_generated_candidates()
            > sequence_planning.bounds().max_input_candidates()
            || learning_progress.bounds().max_focuses()
                > sequence_planning.bounds().max_learning_progress_estimates()
            || sequence_planning.bounds().max_selected_plans() > stop_continue.bounds().max_plans()
        {
            return None;
        }

        Some(Self {
            proposal,
            learning_progress,
            sequence_planning,
            stop_continue,
        })
    }

    pub fn proposal(self) -> BeliefDrivenExperimentProposalPolicy {
        self.proposal
    }

    pub fn learning_progress(self) -> LearningProgressPolicy {
        self.learning_progress
    }

    pub fn sequence_planning(self) -> ExperimentSequencePlanningPolicy {
        self.sequence_planning
    }

    pub fn stop_continue(self) -> StopContinueExperimentationPolicy {
        self.stop_continue
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegratedAutonomousExperimentationStatus {
    ContinueExperimentation,
    StopResolved,
    StopExperimentBudgetExhausted,
    StopNoUsefulExperiment,
    StopLearningStalled,
    AbstainProposal,
    AbstainLearningProgress,
    AbstainSequencePlanning,
    AbstainControl,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegratedAutonomousExperimentationResult {
    status: IntegratedAutonomousExperimentationStatus,
    proposal: BeliefDrivenExperimentProposalResult,
    learning_progress: Option<LearningProgressEstimationResult>,
    sequence_planning: Option<ExperimentSequencePlanningResult>,
    control: Option<StopContinueExperimentationResult>,
}

impl IntegratedAutonomousExperimentationResult {
    pub fn status(&self) -> IntegratedAutonomousExperimentationStatus {
        self.status
    }

    pub fn proposal(&self) -> &BeliefDrivenExperimentProposalResult {
        &self.proposal
    }

    pub fn learning_progress(&self) -> Option<&LearningProgressEstimationResult> {
        self.learning_progress.as_ref()
    }

    pub fn sequence_planning(&self) -> Option<&ExperimentSequencePlanningResult> {
        self.sequence_planning.as_ref()
    }

    pub fn control(&self) -> Option<&StopContinueExperimentationResult> {
        self.control.as_ref()
    }

    pub fn continuing(&self) -> bool {
        self.status == IntegratedAutonomousExperimentationStatus::ContinueExperimentation
    }

    pub fn stopped(&self) -> bool {
        matches!(
            self.status,
            IntegratedAutonomousExperimentationStatus::StopResolved
                | IntegratedAutonomousExperimentationStatus::StopExperimentBudgetExhausted
                | IntegratedAutonomousExperimentationStatus::StopNoUsefulExperiment
                | IntegratedAutonomousExperimentationStatus::StopLearningStalled
        )
    }

    pub fn abstained(&self) -> bool {
        matches!(
            self.status,
            IntegratedAutonomousExperimentationStatus::AbstainProposal
                | IntegratedAutonomousExperimentationStatus::AbstainLearningProgress
                | IntegratedAutonomousExperimentationStatus::AbstainSequencePlanning
                | IntegratedAutonomousExperimentationStatus::AbstainControl
        )
    }

    pub fn next_plan(&self) -> Option<&ExperimentSequencePlan> {
        self.control.as_ref()?.next_plan()
    }

    pub fn next_experiment(&self) -> Option<&AutonomousExperimentProposal> {
        self.next_plan()?
            .steps()
            .first()
            .map(|step| step.candidate().experiment())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutonomousIntegratedExperimentationCycle;

impl AutonomousIntegratedExperimentationCycle {
    fn proposal_abstained(result: &BeliefDrivenExperimentProposalResult) -> bool {
        matches!(
            result.status(),
            BeliefDrivenExperimentProposalStatus::BeliefFrontierExceeded
                | BeliefDrivenExperimentProposalStatus::PossibilityFrontierExceeded
                | BeliefDrivenExperimentProposalStatus::DuplicateBeliefIdentity
        )
    }

    fn learning_progress_abstained(result: &LearningProgressEstimationResult) -> bool {
        matches!(
            result.status(),
            LearningProgressEstimationStatus::InputFrontierExceeded
                | LearningProgressEstimationStatus::FocusFrontierExceeded
                | LearningProgressEstimationStatus::ConflictingEvidenceIdentity
        )
    }

    fn sequence_abstained(result: &ExperimentSequencePlanningResult) -> bool {
        matches!(
            result.status(),
            ExperimentSequencePlanningStatus::CandidateFrontierExceeded
                | ExperimentSequencePlanningStatus::LearningProgressFrontierExceeded
                | ExperimentSequencePlanningStatus::ExpansionFrontierExceeded
                | ExperimentSequencePlanningStatus::ConflictingCandidateIdentity
                | ExperimentSequencePlanningStatus::ConflictingLearningProgressFocus
                | ExperimentSequencePlanningStatus::ConflictingPredictionIdentity
        )
    }

    fn integrated_status(
        control: &StopContinueExperimentationResult,
    ) -> IntegratedAutonomousExperimentationStatus {
        match control.decision() {
            StopContinueExperimentationDecision::ContinueExperimentation => {
                IntegratedAutonomousExperimentationStatus::ContinueExperimentation
            }
            StopContinueExperimentationDecision::StopResolved => {
                IntegratedAutonomousExperimentationStatus::StopResolved
            }
            StopContinueExperimentationDecision::StopExperimentBudgetExhausted => {
                IntegratedAutonomousExperimentationStatus::StopExperimentBudgetExhausted
            }
            StopContinueExperimentationDecision::StopNoApplicablePlan => {
                IntegratedAutonomousExperimentationStatus::StopNoUsefulExperiment
            }
            StopContinueExperimentationDecision::StopLearningStalled => {
                IntegratedAutonomousExperimentationStatus::StopLearningStalled
            }
            StopContinueExperimentationDecision::AbstainBeliefFrontierExceeded
            | StopContinueExperimentationDecision::AbstainPlanFrontierExceeded
            | StopContinueExperimentationDecision::AbstainDuplicateBeliefIdentity => {
                IntegratedAutonomousExperimentationStatus::AbstainControl
            }
        }
    }

    pub fn run_cycle(
        current_state: &CognitiveStructure,
        beliefs: &[HypothesisBeliefState],
        possibilities: &[GroundedExperimentPossibility],
        learning_samples: &[ExperimentLearningProgressSample],
        current_experiment_cycle: usize,
        policy: IntegratedAutonomousExperimentationPolicy,
    ) -> IntegratedAutonomousExperimentationResult {
        let proposal = AutonomousBeliefDrivenExperimentProposal::generate(
            beliefs,
            possibilities,
            policy.proposal(),
        );

        if Self::proposal_abstained(&proposal) {
            return IntegratedAutonomousExperimentationResult {
                status: IntegratedAutonomousExperimentationStatus::AbstainProposal,
                proposal,
                learning_progress: None,
                sequence_planning: None,
                control: None,
            };
        }

        let learning_progress = AutonomousLearningProgressEstimation::estimate(
            learning_samples,
            policy.learning_progress(),
        );

        if Self::learning_progress_abstained(&learning_progress) {
            return IntegratedAutonomousExperimentationResult {
                status: IntegratedAutonomousExperimentationStatus::AbstainLearningProgress,
                proposal,
                learning_progress: Some(learning_progress),
                sequence_planning: None,
                control: None,
            };
        }

        let sequence_planning = AutonomousExperimentSequencePlanning::plan(
            current_state,
            proposal.generated(),
            learning_progress.estimates(),
            policy.sequence_planning(),
        );

        if Self::sequence_abstained(&sequence_planning) {
            return IntegratedAutonomousExperimentationResult {
                status: IntegratedAutonomousExperimentationStatus::AbstainSequencePlanning,
                proposal,
                learning_progress: Some(learning_progress),
                sequence_planning: Some(sequence_planning),
                control: None,
            };
        }

        let control = AutonomousStopContinueExperimentation::control(
            current_state,
            beliefs,
            sequence_planning.plans(),
            current_experiment_cycle,
            policy.stop_continue(),
        );

        let status = Self::integrated_status(&control);

        IntegratedAutonomousExperimentationResult {
            status,
            proposal,
            learning_progress: Some(learning_progress),
            sequence_planning: Some(sequence_planning),
            control: Some(control),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalAutonomousIntegratedExperimentationCycle;

impl UniversalAutonomousIntegratedExperimentationCycle {
    pub fn evaluate(
        current_state: &CognitiveStructure,
        beliefs: &[HypothesisBeliefState],
        possibilities: &[GroundedExperimentPossibility],
        learning_samples: &[ExperimentLearningProgressSample],
        current_experiment_cycle: usize,
        policy: IntegratedAutonomousExperimentationPolicy,
    ) -> IntegratedAutonomousExperimentationResult {
        AutonomousIntegratedExperimentationCycle::run_cycle(
            current_state,
            beliefs,
            possibilities,
            learning_samples,
            current_experiment_cycle,
            policy,
        )
    }
}

#[cfg(test)]
mod integrated_autonomous_experimentation_cycle_tests {
    use super::*;

    fn s(value: u16) -> CognitiveSignal {
        if value == 0 {
            CognitiveSignal::zero()
        } else {
            CognitiveSignal::new(value).unwrap()
        }
    }

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn belief(hypothesis: u64, confidence: u16) -> HypothesisBeliefState {
        HypothesisBeliefState::new(a(hypothesis), s(confidence)).unwrap()
    }

    fn prediction(hypothesis: u64, outcome: u64) -> CompetingHypothesisPrediction {
        CompetingHypothesisPrediction::new(a(hypothesis), a(outcome), s(900)).unwrap()
    }

    fn possibility(
        source: u64,
        action: u64,
        predictions: Vec<CompetingHypothesisPrediction>,
    ) -> GroundedExperimentPossibility {
        GroundedExperimentPossibility::new(
            a(source),
            a(action),
            predictions,
            s(900),
            s(900),
            s(100),
        )
        .unwrap()
    }

    fn progress_sample(
        identity: u64,
        state: u64,
        action: u64,
        values: [u16; 5],
    ) -> ExperimentLearningProgressSample {
        ExperimentLearningProgressSample::new(
            a(identity),
            a(state),
            a(action),
            LearningProgressMeasurement::new(
                s(values[0]),
                s(values[1]),
                s(values[2]),
                s(values[3]),
                s(values[4]),
            )
            .unwrap(),
        )
    }

    fn foundation_policy() -> ActiveExperimentPolicy {
        ActiveExperimentPolicy::new(
            ActiveExperimentBounds::new(32, 32, 32).unwrap(),
            ActiveExperimentThresholds::new(s(500), s(500), s(500), s(500)).unwrap(),
        )
    }

    fn proposal_policy(max_possibilities: usize) -> BeliefDrivenExperimentProposalPolicy {
        BeliefDrivenExperimentProposalPolicy::new(
            foundation_policy(),
            BeliefDrivenExperimentProposalBounds::new(16, max_possibilities, 16, 16).unwrap(),
            s(500),
            s(500),
        )
        .unwrap()
    }

    fn learning_policy() -> LearningProgressPolicy {
        LearningProgressPolicy::new(
            LearningProgressBounds::new(32, 16, 8).unwrap(),
            LearningProgressThresholds::new(s(500), 2, s(50)).unwrap(),
        )
        .unwrap()
    }

    fn sequence_policy(max_expansions: usize) -> ExperimentSequencePlanningPolicy {
        ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(16, 16, 4, max_expansions, 8).unwrap(),
            s(500),
        )
        .unwrap()
    }

    fn control_policy(minimum_information: u16) -> StopContinueExperimentationPolicy {
        StopContinueExperimentationPolicy::new(
            StopContinueExperimentationBounds::new(16, 8, 8).unwrap(),
            StopContinueExperimentationThresholds::new(
                s(500),
                s(850),
                s(250),
                s(100),
                s(minimum_information),
                s(500),
            )
            .unwrap(),
        )
    }

    fn policy() -> IntegratedAutonomousExperimentationPolicy {
        IntegratedAutonomousExperimentationPolicy::new(
            proposal_policy(16),
            learning_policy(),
            sequence_policy(64),
            control_policy(600),
        )
        .unwrap()
    }

    #[test]
    fn integrated_policy_requires_cross_layer_frontier_compatibility() {
        let incompatible_sequence = ExperimentSequencePlanningPolicy::new(
            ExperimentSequencePlanningBounds::new(4, 16, 4, 64, 8).unwrap(),
            s(500),
        )
        .unwrap();

        assert_eq!(
            IntegratedAutonomousExperimentationPolicy::new(
                proposal_policy(16),
                learning_policy(),
                incompatible_sequence,
                control_policy(600),
            ),
            None
        );

        let incompatible_control = StopContinueExperimentationPolicy::new(
            StopContinueExperimentationBounds::new(16, 4, 8).unwrap(),
            StopContinueExperimentationThresholds::new(
                s(500),
                s(850),
                s(250),
                s(100),
                s(600),
                s(500),
            )
            .unwrap(),
        );

        assert_eq!(
            IntegratedAutonomousExperimentationPolicy::new(
                proposal_policy(16),
                learning_policy(),
                sequence_policy(64),
                incompatible_control,
            ),
            None
        );
    }

    #[test]
    fn unresolved_beliefs_bootstrap_experimentation_from_expected_information_gain() {
        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[possibility(
                1,
                10,
                vec![prediction(1, 100), prediction(2, 101)],
            )],
            &[],
            0,
            policy(),
        );

        assert!(result.continuing());

        assert_eq!(
            result.status(),
            IntegratedAutonomousExperimentationStatus::ContinueExperimentation
        );

        assert_eq!(
            result.control().unwrap().continuation_basis(),
            Some(ExperimentContinuationBasis::ExpectedInformationGain)
        );

        assert_eq!(result.next_experiment().unwrap().action(), &a(10));
    }

    #[test]
    fn measured_learning_progress_drives_continuation_after_experimental_history_exists() {
        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[possibility(
                1,
                10,
                vec![prediction(1, 100), prediction(2, 101)],
            )],
            &[
                progress_sample(1, 1, 10, [900, 700, 900, 700, 900]),
                progress_sample(2, 1, 10, [800, 600, 800, 600, 900]),
            ],
            1,
            policy(),
        );

        assert!(result.continuing());

        assert_eq!(
            result.learning_progress().unwrap().estimates()[0].learning_progress(),
            s(200)
        );

        assert_eq!(
            result.control().unwrap().continuation_basis(),
            Some(ExperimentContinuationBasis::MeasuredLearningProgress)
        );
    }

    #[test]
    fn resolved_belief_space_stops_even_when_no_experiment_can_be_generated() {
        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700)],
            &[],
            &[],
            0,
            policy(),
        );

        assert_eq!(
            result.status(),
            IntegratedAutonomousExperimentationStatus::StopResolved
        );

        assert!(result.stopped());

        assert_eq!(result.control().unwrap().resolved_winner(), Some(&a(1)));

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn exhausted_experiment_budget_stops_integrated_cycle_before_execution() {
        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[possibility(
                1,
                10,
                vec![prediction(1, 100), prediction(2, 101)],
            )],
            &[],
            8,
            policy(),
        );

        assert_eq!(
            result.status(),
            IntegratedAutonomousExperimentationStatus::StopExperimentBudgetExhausted
        );

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn no_discriminating_possibility_stops_without_useless_intervention() {
        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[possibility(
                1,
                10,
                vec![prediction(1, 100), prediction(2, 100)],
            )],
            &[],
            0,
            policy(),
        );

        assert_eq!(result.proposal().generated_count(), 0);

        assert_eq!(
            result.status(),
            IntegratedAutonomousExperimentationStatus::StopNoUsefulExperiment
        );

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn insufficient_learning_and_information_stop_stalled_integrated_experimentation() {
        let strict = IntegratedAutonomousExperimentationPolicy::new(
            proposal_policy(16),
            learning_policy(),
            sequence_policy(64),
            control_policy(900),
        )
        .unwrap();

        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700), belief(2, 680), belief(3, 660)],
            &[possibility(
                1,
                10,
                vec![prediction(1, 100), prediction(2, 100), prediction(3, 101)],
            )],
            &[],
            2,
            strict,
        );

        assert_eq!(
            result.proposal().generated()[0]
                .experiment()
                .evidence()
                .expected_information_gain(),
            s(666)
        );

        assert_eq!(
            result.status(),
            IntegratedAutonomousExperimentationStatus::StopLearningStalled
        );

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn conflicting_learning_progress_provenance_abstains_before_planning_or_control() {
        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[possibility(
                1,
                10,
                vec![prediction(1, 100), prediction(2, 101)],
            )],
            &[
                progress_sample(1, 1, 10, [900, 700, 900, 700, 900]),
                progress_sample(1, 1, 10, [900, 500, 900, 500, 900]),
            ],
            1,
            policy(),
        );

        assert_eq!(
            result.status(),
            IntegratedAutonomousExperimentationStatus::AbstainLearningProgress
        );

        assert!(result.abstained());

        assert!(result.sequence_planning().is_none());

        assert!(result.control().is_none());
    }

    #[test]
    fn proposal_frontier_failure_abstains_atomically_before_downstream_cognition() {
        let bounded = IntegratedAutonomousExperimentationPolicy::new(
            proposal_policy(1),
            learning_policy(),
            sequence_policy(64),
            control_policy(600),
        )
        .unwrap();

        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[
                possibility(1, 10, vec![prediction(1, 100), prediction(2, 101)]),
                possibility(1, 11, vec![prediction(1, 110), prediction(2, 111)]),
            ],
            &[],
            0,
            bounded,
        );

        assert_eq!(
            result.status(),
            IntegratedAutonomousExperimentationStatus::AbstainProposal
        );

        assert!(result.learning_progress().is_none());

        assert!(result.sequence_planning().is_none());

        assert!(result.control().is_none());
    }

    #[test]
    fn bounded_sequence_expansion_failure_abstains_before_execution_control() {
        let bounded = IntegratedAutonomousExperimentationPolicy::new(
            proposal_policy(16),
            learning_policy(),
            sequence_policy(1),
            control_policy(600),
        )
        .unwrap();

        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700), belief(2, 680)],
            &[
                possibility(1, 10, vec![prediction(1, 100), prediction(2, 101)]),
                possibility(1, 11, vec![prediction(1, 110), prediction(2, 111)]),
            ],
            &[],
            0,
            bounded,
        );

        assert_eq!(
            result.status(),
            IntegratedAutonomousExperimentationStatus::AbstainSequencePlanning
        );

        assert_eq!(
            result.sequence_planning().unwrap().status(),
            ExperimentSequencePlanningStatus::ExpansionFrontierExceeded
        );

        assert!(result.control().is_none());
    }

    #[test]
    fn exact_duplicate_belief_identity_abstains_before_experiment_generation_can_fake_competition()
    {
        let result = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &[belief(1, 700), belief(1, 680)],
            &[possibility(
                1,
                10,
                vec![prediction(1, 100), prediction(2, 101)],
            )],
            &[],
            0,
            policy(),
        );

        assert_eq!(
            result.status(),
            IntegratedAutonomousExperimentationStatus::AbstainProposal
        );

        assert_eq!(
            result.proposal().status(),
            BeliefDrivenExperimentProposalStatus::DuplicateBeliefIdentity
        );

        assert!(result.next_experiment().is_none());
    }

    #[test]
    fn integrated_cycle_is_order_invariant_non_mutating_deterministic_and_facade_equivalent() {
        let beliefs = vec![belief(2, 680), belief(1, 700), belief(3, 660)];

        let possibilities = vec![
            possibility(
                1,
                10,
                vec![prediction(1, 100), prediction(2, 101), prediction(3, 102)],
            ),
            possibility(
                1,
                11,
                vec![prediction(1, 110), prediction(2, 111), prediction(3, 112)],
            ),
        ];

        let samples = vec![
            progress_sample(1, 1, 10, [900, 700, 900, 700, 900]),
            progress_sample(2, 1, 10, [800, 600, 800, 600, 900]),
            progress_sample(3, 1, 11, [900, 800, 900, 800, 900]),
            progress_sample(4, 1, 11, [800, 700, 800, 700, 900]),
        ];

        let before_beliefs = beliefs.clone();

        let before_possibilities = possibilities.clone();

        let before_samples = samples.clone();

        let mut reversed_beliefs = beliefs.clone();

        reversed_beliefs.reverse();

        let mut reversed_possibilities = possibilities.clone();

        reversed_possibilities.reverse();

        for possibility in &mut reversed_possibilities {
            possibility.predictions.reverse();
        }

        let mut reversed_samples = samples.clone();

        reversed_samples.reverse();

        let p = policy();

        let direct = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &beliefs,
            &possibilities,
            &samples,
            1,
            p,
        );

        let reordered = AutonomousIntegratedExperimentationCycle::run_cycle(
            &a(1),
            &reversed_beliefs,
            &reversed_possibilities,
            &reversed_samples,
            1,
            p,
        );

        let facade = UniversalAutonomousIntegratedExperimentationCycle::evaluate(
            &a(1),
            &beliefs,
            &possibilities,
            &samples,
            1,
            p,
        );

        let repeated = UniversalAutonomousIntegratedExperimentationCycle::evaluate(
            &a(1),
            &beliefs,
            &possibilities,
            &samples,
            1,
            p,
        );

        assert_eq!(direct, reordered);

        assert_eq!(direct, facade);

        assert_eq!(facade, repeated);

        assert_eq!(beliefs, before_beliefs);

        assert_eq!(possibilities, before_possibilities);

        assert_eq!(samples, before_samples);
    }
}

#[cfg(test)]
mod p4g_c3b_epistemic_forecast_representation_tests {
    use super::*;

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn evidence(support: u64, opportunity: u64, counterexamples: u64) -> EpistemicForecastEvidence {
        EpistemicForecastEvidence::new(support, opportunity, counterexamples)
            .expect("test evidence must be internally exact")
    }

    fn possibility(
        forecasts: Vec<EpistemicHypothesisForecast>,
    ) -> GroundedEpistemicExperimentPossibility {
        GroundedEpistemicExperimentPossibility::new(a(1), a(10), forecasts)
            .expect("test possibility requires forecasts")
    }

    fn policy() -> EpistemicForecastDiscriminationPolicy {
        EpistemicForecastDiscriminationPolicy::new(16, 16).expect("positive C3B-A bounds")
    }

    #[test]
    fn empirical_evidence_contract_is_exact_and_rejects_invented_counts() {
        let exact = evidence(3, 5, 2);

        assert_eq!(exact.support_count(), 3);
        assert_eq!(exact.opportunity_count(), 5);
        assert_eq!(exact.counterexample_count(), 2);

        assert_eq!(EpistemicForecastEvidence::new(0, 1, 1), None,);

        assert_eq!(EpistemicForecastEvidence::new(1, 0, 0), None,);

        assert_eq!(
            EpistemicForecastEvidence::new(2, 5, 2),
            None,
            "support plus counterexamples must equal exact opportunity count",
        );

        assert_eq!(
            EpistemicForecastEvidence::new(u64::MAX, 1, 1),
            None,
            "overflowing evidence arithmetic must fail closed",
        );
    }

    #[test]
    fn contextual_abstention_is_first_class_without_fabricated_outcome_or_confidence() {
        let historical = evidence(2, 2, 0);
        let outcome = a(900);

        let predicted =
            EpistemicHypothesisForecast::predicted(a(100), a(500), outcome.clone(), historical)
                .expect("grounded prediction is valid");

        let abstained = EpistemicHypothesisForecast::context_abstained(a(101), a(500), historical)
            .expect("contextual abstention is valid");

        assert_eq!(
            predicted.status(),
            EpistemicHypothesisForecastStatus::Predicted,
        );

        assert_eq!(predicted.predicted_outcome(), Some(&outcome),);

        assert_eq!(
            abstained.status(),
            EpistemicHypothesisForecastStatus::ContextAbstained,
        );

        assert_eq!(
            abstained.predicted_outcome(),
            None,
            "contextual abstention must never become a synthetic outcome",
        );

        assert_eq!(predicted.evidence(), historical);
        assert_eq!(abstained.evidence(), historical);
        assert_eq!(predicted.target(), abstained.target());
        assert_ne!(predicted.hypothesis(), abstained.hypothesis());
    }

    #[test]
    fn prediction_vs_context_abstention_is_informative_but_no_opportunity_is_not() {
        let historical = evidence(2, 2, 0);

        let predicted =
            EpistemicHypothesisForecast::predicted(a(100), a(500), a(900), historical).unwrap();

        let abstained =
            EpistemicHypothesisForecast::context_abstained(a(101), a(500), historical).unwrap();

        let informative = AutonomousEpistemicForecastDiscrimination::evaluate(
            &possibility(vec![predicted.clone(), abstained]),
            policy(),
        );

        assert!(informative.informative());
        assert_eq!(informative.pairwise_separation_score(), 1);
        assert_eq!(informative.disagreements().len(), 1);
        assert_eq!(informative.disagreements()[0].predicted_count(), 1,);
        assert_eq!(informative.disagreements()[0].context_abstention_count(), 1,);

        let no_opportunity =
            EpistemicHypothesisForecast::no_effect_opportunity(a(101), a(500), historical).unwrap();

        let noninformative = AutonomousEpistemicForecastDiscrimination::evaluate(
            &possibility(vec![predicted, no_opportunity]),
            policy(),
        );

        assert!(!noninformative.informative());
        assert_eq!(noninformative.pairwise_separation_score(), 0,);

        assert_eq!(
            noninformative.disagreements()[0].no_effect_opportunity_count(),
            1,
        );
    }

    #[test]
    fn order_and_exact_duplication_cannot_inflate_epistemic_separation() {
        let historical = evidence(3, 3, 0);

        let predicted =
            EpistemicHypothesisForecast::predicted(a(100), a(500), a(900), historical).unwrap();

        let abstained =
            EpistemicHypothesisForecast::context_abstained(a(101), a(500), historical).unwrap();

        let direct = AutonomousEpistemicForecastDiscrimination::evaluate(
            &possibility(vec![
                predicted.clone(),
                abstained.clone(),
                predicted.clone(),
            ]),
            policy(),
        );

        let reversed = UniversalAutonomousEpistemicForecastDiscrimination::evaluate(
            &possibility(vec![abstained, predicted.clone(), predicted]),
            policy(),
        );

        assert_eq!(direct, reversed);
        assert_eq!(direct.input_forecast_count(), 3);
        assert_eq!(direct.unique_forecast_count(), 2);
        assert_eq!(direct.pairwise_separation_score(), 1);
    }

    #[test]
    fn hard_frontiers_fail_closed_without_partial_epistemic_authority() {
        let historical = evidence(1, 1, 0);

        let forecasts = vec![
            EpistemicHypothesisForecast::predicted(a(100), a(500), a(900), historical).unwrap(),
            EpistemicHypothesisForecast::context_abstained(a(101), a(500), historical).unwrap(),
        ];

        let forecast_bounded = AutonomousEpistemicForecastDiscrimination::evaluate(
            &possibility(forecasts.clone()),
            EpistemicForecastDiscriminationPolicy::new(1, 16).unwrap(),
        );

        assert!(forecast_bounded.forecast_frontier_truncated());
        assert!(!forecast_bounded.informative());
        assert_eq!(forecast_bounded.pairwise_separation_score(), 0,);

        let multi_target = possibility(vec![
            forecasts[0].clone(),
            forecasts[1].clone(),
            EpistemicHypothesisForecast::predicted(a(102), a(501), a(901), historical).unwrap(),
        ]);

        let target_bounded = AutonomousEpistemicForecastDiscrimination::evaluate(
            &multi_target,
            EpistemicForecastDiscriminationPolicy::new(16, 1).unwrap(),
        );

        assert!(target_bounded.target_frontier_truncated());
        assert!(!target_bounded.informative());
        assert_eq!(target_bounded.pairwise_separation_score(), 0,);
    }

    #[test]
    fn epistemic_possibility_requires_real_forecast_and_positive_bounds() {
        assert_eq!(
            GroundedEpistemicExperimentPossibility::new(a(1), a(2), Vec::new(),),
            None,
        );

        assert_eq!(EpistemicForecastDiscriminationPolicy::new(0, 1), None,);

        assert_eq!(EpistemicForecastDiscriminationPolicy::new(1, 0), None,);
    }
}

#[cfg(test)]
mod p4g_c3c_realized_epistemic_resolution_tests {
    use super::*;

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn evidence() -> EpistemicForecastEvidence {
        EpistemicForecastEvidence::new(3, 3, 0).expect("exact test evidence")
    }

    fn predicted(hypothesis: u64, target: u64) -> EpistemicHypothesisForecast {
        EpistemicHypothesisForecast::predicted(
            a(hypothesis),
            a(target),
            a(target + 10_000),
            evidence(),
        )
        .expect("test prediction")
    }

    fn abstained(hypothesis: u64, target: u64) -> EpistemicHypothesisForecast {
        EpistemicHypothesisForecast::context_abstained(a(hypothesis), a(target), evidence())
            .expect("test abstention")
    }

    fn no_opportunity(hypothesis: u64, target: u64) -> EpistemicHypothesisForecast {
        EpistemicHypothesisForecast::no_effect_opportunity(a(hypothesis), a(target), evidence())
            .expect("test no-opportunity forecast")
    }

    fn possibility(
        forecasts: Vec<EpistemicHypothesisForecast>,
    ) -> GroundedEpistemicExperimentPossibility {
        GroundedEpistemicExperimentPossibility::new(a(1), a(10), forecasts)
            .expect("grounded test possibility")
    }

    fn observation(
        targets: Vec<EpistemicTargetObservation>,
    ) -> GroundedEpistemicOutcomeObservation {
        GroundedEpistemicOutcomeObservation::new(a(1), a(10), targets)
            .expect("grounded test outcome")
    }

    fn target(id: u64, occurred: bool) -> EpistemicTargetObservation {
        EpistemicTargetObservation::new(a(id), occurred)
    }

    fn policy() -> EpistemicOutcomeResolutionPolicy {
        EpistemicOutcomeResolutionPolicy::new(32, 32).expect("positive test bounds")
    }

    #[test]
    fn real_occurrence_supports_prediction_and_real_absence_is_counterexample() {
        let result = AutonomousEpistemicOutcomeResolution::evaluate(
            &possibility(vec![predicted(100, 500), predicted(101, 501)]),
            &observation(vec![target(500, true), target(501, false)]),
            policy(),
        );

        assert!(result.resolved());
        assert_eq!(result.supported_prediction_count(), 1,);
        assert_eq!(result.counterexample_prediction_count(), 1,);
        assert_eq!(result.empirically_tested_prediction_count(), 2,);
        assert_eq!(result.falsified_hypothesis_count(), 1,);

        assert!(result.assessments().iter().any(|assessment| {
            assessment.status() == EpistemicForecastOutcomeAssessmentStatus::Supported
        }),);

        assert!(result.assessments().iter().any(|assessment| {
            assessment.status() == EpistemicForecastOutcomeAssessmentStatus::Counterexample
                && assessment.falsified()
        }),);
    }

    #[test]
    fn contextual_abstention_remains_uninformative_even_when_target_occurs_or_fails() {
        for occurred in [false, true] {
            let result = AutonomousEpistemicOutcomeResolution::evaluate(
                &possibility(vec![abstained(100, 500)]),
                &observation(vec![target(500, occurred)]),
                policy(),
            );

            assert!(result.resolved());
            assert_eq!(result.context_uninformative_count(), 1,);
            assert_eq!(result.empirically_tested_prediction_count(), 0,);
            assert_eq!(result.falsified_hypothesis_count(), 0,);

            assert_eq!(
                result.assessments()[0].status(),
                EpistemicForecastOutcomeAssessmentStatus::ContextUninformative,
            );
        }
    }

    #[test]
    fn no_effect_opportunity_never_becomes_negative_prediction_evidence() {
        let result = AutonomousEpistemicOutcomeResolution::evaluate(
            &possibility(vec![no_opportunity(100, 500)]),
            &observation(vec![target(500, false)]),
            policy(),
        );

        assert!(result.resolved());
        assert_eq!(result.no_opportunity_uninformative_count(), 1,);
        assert_eq!(result.counterexample_prediction_count(), 0,);
        assert_eq!(result.falsified_hypothesis_count(), 0,);
    }

    #[test]
    fn missing_target_observation_fails_closed_instead_of_treating_absence_as_nonoccurrence() {
        let result = AutonomousEpistemicOutcomeResolution::evaluate(
            &possibility(vec![predicted(100, 500), predicted(101, 501)]),
            &observation(vec![target(500, true)]),
            policy(),
        );

        assert_eq!(
            result.status(),
            EpistemicOutcomeResolutionStatus::MissingTargetObservation,
        );
        assert!(!result.resolved());
        assert!(result.assessments().is_empty());
        assert_eq!(result.empirically_tested_prediction_count(), 0,);
        assert_eq!(result.falsified_hypothesis_count(), 0,);
    }

    #[test]
    fn source_or_action_mismatch_abstains_atomically() {
        let possibility = possibility(vec![predicted(100, 500)]);

        let wrong_state =
            GroundedEpistemicOutcomeObservation::new(a(999), a(10), vec![target(500, true)])
                .unwrap();

        let wrong_action =
            GroundedEpistemicOutcomeObservation::new(a(1), a(999), vec![target(500, true)])
                .unwrap();

        let state_result =
            AutonomousEpistemicOutcomeResolution::evaluate(&possibility, &wrong_state, policy());

        let action_result =
            AutonomousEpistemicOutcomeResolution::evaluate(&possibility, &wrong_action, policy());

        assert_eq!(
            state_result.status(),
            EpistemicOutcomeResolutionStatus::SourceStateMismatch,
        );
        assert_eq!(
            action_result.status(),
            EpistemicOutcomeResolutionStatus::ActionMismatch,
        );

        assert!(state_result.assessments().is_empty());
        assert!(action_result.assessments().is_empty());
    }

    #[test]
    fn conflicting_target_observation_is_rejected_at_construction() {
        assert_eq!(
            GroundedEpistemicOutcomeObservation::new(
                a(1),
                a(10),
                vec![target(500, true), target(500, false),],
            ),
            None,
        );

        let deduplicated = GroundedEpistemicOutcomeObservation::new(
            a(1),
            a(10),
            vec![target(500, true), target(500, true)],
        )
        .unwrap();

        assert_eq!(deduplicated.targets().len(), 1,);
    }

    #[test]
    fn conflicting_forecast_identity_fails_closed_without_partial_resolution() {
        let same_hypothesis_prediction = predicted(100, 500);

        let same_hypothesis_abstention = abstained(100, 500);

        let result = AutonomousEpistemicOutcomeResolution::evaluate(
            &possibility(vec![same_hypothesis_prediction, same_hypothesis_abstention]),
            &observation(vec![target(500, true)]),
            policy(),
        );

        assert_eq!(
            result.status(),
            EpistemicOutcomeResolutionStatus::ConflictingForecastIdentity,
        );

        assert!(result.assessments().is_empty());
        assert_eq!(result.falsified_hypothesis_count(), 0,);
    }

    #[test]
    fn exact_duplicate_forecasts_do_not_inflate_realized_resolution() {
        let forecast = predicted(100, 500);

        let result = AutonomousEpistemicOutcomeResolution::evaluate(
            &possibility(vec![forecast.clone(), forecast]),
            &observation(vec![target(500, false)]),
            policy(),
        );

        assert!(result.resolved());
        assert_eq!(result.input_forecast_count(), 2,);
        assert_eq!(result.unique_forecast_count(), 1,);
        assert_eq!(result.counterexample_prediction_count(), 1,);
    }

    #[test]
    fn outcome_resolution_is_order_invariant_non_mutating_and_facade_equivalent() {
        let possibility = possibility(vec![
            predicted(100, 500),
            abstained(101, 500),
            predicted(102, 501),
        ]);

        let observation = observation(vec![target(500, false), target(501, true)]);

        let mut reversed_forecasts = possibility.forecasts().to_vec();
        reversed_forecasts.reverse();

        let reversed_possibility = GroundedEpistemicExperimentPossibility::new(
            possibility.source_state().clone(),
            possibility.action().clone(),
            reversed_forecasts,
        )
        .unwrap();

        let mut reversed_targets = observation.targets().to_vec();
        reversed_targets.reverse();

        let reversed_observation = GroundedEpistemicOutcomeObservation::new(
            observation.source_state().clone(),
            observation.action().clone(),
            reversed_targets,
        )
        .unwrap();

        let before_possibility = possibility.clone();
        let before_observation = observation.clone();

        let direct =
            AutonomousEpistemicOutcomeResolution::evaluate(&possibility, &observation, policy());

        let reordered = AutonomousEpistemicOutcomeResolution::evaluate(
            &reversed_possibility,
            &reversed_observation,
            policy(),
        );

        let facade = UniversalAutonomousEpistemicOutcomeResolution::evaluate(
            &possibility,
            &observation,
            policy(),
        );

        assert_eq!(direct, reordered);
        assert_eq!(direct, facade);
        assert_eq!(possibility, before_possibility);
        assert_eq!(observation, before_observation);
    }

    #[test]
    fn hard_frontiers_fail_closed_before_any_partial_outcome_resolution() {
        let possibility = possibility(vec![predicted(100, 500), predicted(101, 501)]);

        let observation = observation(vec![target(500, true), target(501, false)]);

        let forecast_bounded = AutonomousEpistemicOutcomeResolution::evaluate(
            &possibility,
            &observation,
            EpistemicOutcomeResolutionPolicy::new(1, 16).unwrap(),
        );

        assert_eq!(
            forecast_bounded.status(),
            EpistemicOutcomeResolutionStatus::ForecastFrontierExceeded,
        );

        assert!(forecast_bounded.assessments().is_empty(),);

        let observation_bounded = AutonomousEpistemicOutcomeResolution::evaluate(
            &possibility,
            &observation,
            EpistemicOutcomeResolutionPolicy::new(16, 1).unwrap(),
        );

        assert_eq!(
            observation_bounded.status(),
            EpistemicOutcomeResolutionStatus::ObservationFrontierExceeded,
        );

        assert!(observation_bounded.assessments().is_empty(),);

        assert_eq!(EpistemicOutcomeResolutionPolicy::new(0, 1,), None,);

        assert_eq!(EpistemicOutcomeResolutionPolicy::new(1, 0,), None,);
    }
}

#[cfg(test)]
mod p4g_c3d_realized_epistemic_progress_tests {
    use super::*;

    fn a(value: u64) -> CognitiveStructure {
        CognitiveStructure::atom(value)
    }

    fn evidence() -> EpistemicForecastEvidence {
        EpistemicForecastEvidence::new(2, 2, 0).expect("exact test evidence")
    }

    fn predicted(hypothesis: u64, target: u64) -> EpistemicHypothesisForecast {
        EpistemicHypothesisForecast::predicted(
            a(hypothesis),
            a(target),
            a(target + 10_000),
            evidence(),
        )
        .unwrap()
    }

    fn abstained(hypothesis: u64, target: u64) -> EpistemicHypothesisForecast {
        EpistemicHypothesisForecast::context_abstained(a(hypothesis), a(target), evidence())
            .unwrap()
    }

    fn possibility(
        source: u64,
        action: u64,
        forecasts: Vec<EpistemicHypothesisForecast>,
    ) -> GroundedEpistemicExperimentPossibility {
        GroundedEpistemicExperimentPossibility::new(a(source), a(action), forecasts).unwrap()
    }

    fn resolution(
        possibility: &GroundedEpistemicExperimentPossibility,
        target_id: u64,
        occurred: bool,
    ) -> EpistemicOutcomeResolutionResult {
        let observation = GroundedEpistemicOutcomeObservation::new(
            possibility.source_state().clone(),
            possibility.action().clone(),
            vec![EpistemicTargetObservation::new(a(target_id), occurred)],
        )
        .unwrap();

        AutonomousEpistemicOutcomeResolution::evaluate(
            possibility,
            &observation,
            EpistemicOutcomeResolutionPolicy::new(32, 32).unwrap(),
        )
    }

    fn policy() -> EpistemicForecastDiscriminationPolicy {
        EpistemicForecastDiscriminationPolicy::new(32, 32).unwrap()
    }

    #[test]
    fn real_model_update_can_measure_exact_epistemic_separation_reduction() {
        let pre = possibility(1, 10, vec![predicted(100, 500), abstained(101, 500)]);

        let outcome = resolution(&pre, 500, true);

        let post = possibility(1, 10, vec![predicted(100, 500), predicted(101, 500)]);

        let result =
            AutonomousEpistemicResolutionProgress::measure(&pre, &outcome, &post, policy());

        assert!(result.measured());

        let sample = result.sample().unwrap();

        assert_eq!(sample.separation_before(), 1);
        assert_eq!(sample.separation_after(), 0);
        assert_eq!(sample.realized_separation_reduction(), 1,);
        assert_eq!(sample.realized_separation_increase(), 0,);
        assert!(sample.reduced_uncertainty());
        assert!(!sample.increased_uncertainty());
        assert_eq!(sample.empirically_tested_prediction_count(), 1,);
    }

    #[test]
    fn newly_exposed_ambiguity_is_recorded_as_increase_not_silently_clipped() {
        let pre = possibility(1, 10, vec![predicted(100, 500), predicted(101, 500)]);

        let outcome = resolution(&pre, 500, true);

        let post = possibility(1, 10, vec![predicted(100, 500), abstained(101, 500)]);

        let result =
            AutonomousEpistemicResolutionProgress::measure(&pre, &outcome, &post, policy());

        let sample = result.sample().unwrap();

        assert_eq!(sample.separation_before(), 0);
        assert_eq!(sample.separation_after(), 1);
        assert_eq!(sample.realized_separation_reduction(), 0,);
        assert_eq!(sample.realized_separation_increase(), 1,);
        assert!(!sample.reduced_uncertainty());
        assert!(sample.increased_uncertainty());
    }

    #[test]
    fn source_or_action_change_cannot_be_called_learning_progress() {
        let pre = possibility(1, 10, vec![predicted(100, 500)]);

        let outcome = resolution(&pre, 500, true);

        let wrong_source = possibility(2, 10, vec![predicted(100, 500)]);

        let wrong_action = possibility(1, 11, vec![predicted(100, 500)]);

        assert_eq!(
            AutonomousEpistemicResolutionProgress::measure(
                &pre,
                &outcome,
                &wrong_source,
                policy(),
            )
            .status(),
            EpistemicResolutionProgressStatus::
                SourceStateMismatch,
        );

        assert_eq!(
            AutonomousEpistemicResolutionProgress::measure(
                &pre,
                &outcome,
                &wrong_action,
                policy(),
            )
            .status(),
            EpistemicResolutionProgressStatus::
                ActionMismatch,
        );
    }

    #[test]
    fn detached_outcome_resolution_cannot_be_reused_for_another_forecast_frontier() {
        let first = possibility(1, 10, vec![predicted(100, 500)]);

        let second = possibility(1, 10, vec![predicted(999, 500)]);

        let detached = resolution(&second, 500, true);

        let result =
            AutonomousEpistemicResolutionProgress::measure(&first, &detached, &first, policy());

        assert_eq!(
            result.status(),
            EpistemicResolutionProgressStatus::ResolutionForecastMismatch,
        );

        assert!(result.sample().is_none());
    }

    #[test]
    fn unresolved_outcome_cannot_manufacture_progress() {
        let pre = possibility(1, 10, vec![predicted(100, 500)]);

        let wrong_observation = GroundedEpistemicOutcomeObservation::new(
            a(1),
            a(999),
            vec![EpistemicTargetObservation::new(a(500), true)],
        )
        .unwrap();

        let unresolved = AutonomousEpistemicOutcomeResolution::evaluate(
            &pre,
            &wrong_observation,
            EpistemicOutcomeResolutionPolicy::new(32, 32).unwrap(),
        );

        let result =
            AutonomousEpistemicResolutionProgress::measure(&pre, &unresolved, &pre, policy());

        assert_eq!(
            result.status(),
            EpistemicResolutionProgressStatus::OutcomeNotResolved,
        );

        assert!(result.sample().is_none());
    }

    #[test]
    fn hard_discrimination_frontier_fails_closed_before_progress_measurement() {
        let pre = possibility(1, 10, vec![predicted(100, 500), abstained(101, 500)]);

        let outcome = resolution(&pre, 500, true);

        let result = AutonomousEpistemicResolutionProgress::measure(
            &pre,
            &outcome,
            &pre,
            EpistemicForecastDiscriminationPolicy::new(1, 32).unwrap(),
        );

        assert_eq!(
            result.status(),
            EpistemicResolutionProgressStatus::PreLearningFrontierTruncated,
        );

        assert!(result.sample().is_none());
    }

    #[test]
    fn measurement_is_order_invariant_non_mutating_and_facade_equivalent() {
        let pre = possibility(1, 10, vec![predicted(100, 500), abstained(101, 500)]);

        let outcome = resolution(&pre, 500, false);

        let post = possibility(1, 10, vec![predicted(100, 500), predicted(101, 500)]);

        let mut reversed = pre.forecasts().to_vec();
        reversed.reverse();

        let reordered = GroundedEpistemicExperimentPossibility::new(
            pre.source_state().clone(),
            pre.action().clone(),
            reversed,
        )
        .unwrap();

        let before_pre = pre.clone();
        let before_outcome = outcome.clone();
        let before_post = post.clone();

        let direct =
            AutonomousEpistemicResolutionProgress::measure(&pre, &outcome, &post, policy());

        let reordered_result =
            AutonomousEpistemicResolutionProgress::measure(&reordered, &outcome, &post, policy());

        let facade = UniversalAutonomousEpistemicResolutionProgress::measure(
            &pre,
            &outcome,
            &post,
            policy(),
        );

        assert_eq!(direct, reordered_result);
        assert_eq!(direct, facade);

        assert_eq!(pre, before_pre);
        assert_eq!(outcome, before_outcome);
        assert_eq!(post, before_post);
    }
}
