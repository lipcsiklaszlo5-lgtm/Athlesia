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
