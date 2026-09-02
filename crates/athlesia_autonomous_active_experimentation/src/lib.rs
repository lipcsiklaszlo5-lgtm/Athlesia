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
