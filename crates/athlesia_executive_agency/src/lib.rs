use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutiveGoal {
    identity: CognitiveStructure,
    priority: CognitiveSignal,
    satisfaction: CognitiveSignal,
}

impl ExecutiveGoal {
    pub fn new(
        identity: CognitiveStructure,
        priority: CognitiveSignal,
        satisfaction: CognitiveSignal,
    ) -> Self {
        Self {
            identity,
            priority,
            satisfaction,
        }
    }

    pub fn identity(&self) -> &CognitiveStructure {
        &self.identity
    }

    pub fn priority(&self) -> CognitiveSignal {
        self.priority
    }

    pub fn satisfaction(&self) -> CognitiveSignal {
        self.satisfaction
    }

    pub fn is_satisfied(&self) -> bool {
        self.satisfaction.value() == 1000
    }

    pub fn remaining_need(&self) -> CognitiveSignal {
        CognitiveSignal::new(1000_u16.saturating_sub(self.satisfaction.value()))
            .expect("remaining goal need stays on signal scale")
    }

    pub fn pressure(&self) -> CognitiveSignal {
        ExecutiveAgency::scaled_product(self.priority, self.remaining_need())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedExecutiveActionCandidate {
    goal_identity: CognitiveStructure,
    action: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
    goal_alignment: CognitiveSignal,
    controllability: CognitiveSignal,
    evidence_confidence: CognitiveSignal,
    information_gain: CognitiveSignal,
    execution_cost: CognitiveSignal,
}

impl GroundedExecutiveActionCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        goal_identity: CognitiveStructure,
        action: CognitiveStructure,
        predicted_outcome: CognitiveStructure,
        goal_alignment: CognitiveSignal,
        controllability: CognitiveSignal,
        evidence_confidence: CognitiveSignal,
        information_gain: CognitiveSignal,
        execution_cost: CognitiveSignal,
    ) -> Self {
        Self {
            goal_identity,
            action,
            predicted_outcome,
            goal_alignment,
            controllability,
            evidence_confidence,
            information_gain,
            execution_cost,
        }
    }

    pub fn goal_identity(&self) -> &CognitiveStructure {
        &self.goal_identity
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        &self.predicted_outcome
    }

    pub fn goal_alignment(&self) -> CognitiveSignal {
        self.goal_alignment
    }

    pub fn controllability(&self) -> CognitiveSignal {
        self.controllability
    }

    pub fn evidence_confidence(&self) -> CognitiveSignal {
        self.evidence_confidence
    }

    pub fn information_gain(&self) -> CognitiveSignal {
        self.information_gain
    }

    pub fn execution_cost(&self) -> CognitiveSignal {
        self.execution_cost
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExecutiveUtilityWeights {
    goal_alignment: u16,
    controllability: u16,
    evidence_confidence: u16,
    information_gain: u16,
    execution_cost: u16,
}

impl ExecutiveUtilityWeights {
    pub fn new(
        goal_alignment: u16,
        controllability: u16,
        evidence_confidence: u16,
        information_gain: u16,
        execution_cost: u16,
    ) -> Option<Self> {
        let benefit_weight = u64::from(goal_alignment)
            + u64::from(controllability)
            + u64::from(evidence_confidence)
            + u64::from(information_gain);

        if benefit_weight == 0 {
            return None;
        }

        Some(Self {
            goal_alignment,
            controllability,
            evidence_confidence,
            information_gain,
            execution_cost,
        })
    }

    pub fn goal_alignment(self) -> u16 {
        self.goal_alignment
    }

    pub fn controllability(self) -> u16 {
        self.controllability
    }

    pub fn evidence_confidence(self) -> u16 {
        self.evidence_confidence
    }

    pub fn information_gain(self) -> u16 {
        self.information_gain
    }

    pub fn execution_cost(self) -> u16 {
        self.execution_cost
    }

    pub fn total_weight(self) -> u64 {
        u64::from(self.goal_alignment)
            + u64::from(self.controllability)
            + u64::from(self.evidence_confidence)
            + u64::from(self.information_gain)
            + u64::from(self.execution_cost)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExecutiveSelectionThresholds {
    minimum_goal_pressure: CognitiveSignal,
    minimum_goal_alignment: CognitiveSignal,
    minimum_controllability: CognitiveSignal,
    minimum_evidence_confidence: CognitiveSignal,
    minimum_priority_adjusted_utility: CognitiveSignal,
}

impl ExecutiveSelectionThresholds {
    pub fn new(
        minimum_goal_pressure: CognitiveSignal,
        minimum_goal_alignment: CognitiveSignal,
        minimum_controllability: CognitiveSignal,
        minimum_evidence_confidence: CognitiveSignal,
        minimum_priority_adjusted_utility: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_goal_pressure == CognitiveSignal::zero()
            || minimum_goal_alignment == CognitiveSignal::zero()
            || minimum_controllability == CognitiveSignal::zero()
            || minimum_evidence_confidence == CognitiveSignal::zero()
            || minimum_priority_adjusted_utility == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_goal_pressure,
            minimum_goal_alignment,
            minimum_controllability,
            minimum_evidence_confidence,
            minimum_priority_adjusted_utility,
        })
    }

    pub fn minimum_goal_pressure(self) -> CognitiveSignal {
        self.minimum_goal_pressure
    }

    pub fn minimum_goal_alignment(self) -> CognitiveSignal {
        self.minimum_goal_alignment
    }

    pub fn minimum_controllability(self) -> CognitiveSignal {
        self.minimum_controllability
    }

    pub fn minimum_evidence_confidence(self) -> CognitiveSignal {
        self.minimum_evidence_confidence
    }

    pub fn minimum_priority_adjusted_utility(self) -> CognitiveSignal {
        self.minimum_priority_adjusted_utility
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ExecutiveAgencyPolicy {
    max_goals: usize,
    max_actions_per_goal: usize,
    max_action_evaluations: usize,
    max_selected_intents: usize,
    weights: ExecutiveUtilityWeights,
    thresholds: ExecutiveSelectionThresholds,
}

impl ExecutiveAgencyPolicy {
    pub fn new(
        max_goals: usize,
        max_actions_per_goal: usize,
        max_action_evaluations: usize,
        max_selected_intents: usize,
        weights: ExecutiveUtilityWeights,
        thresholds: ExecutiveSelectionThresholds,
    ) -> Option<Self> {
        if max_goals == 0
            || max_actions_per_goal == 0
            || max_action_evaluations == 0
            || max_selected_intents == 0
        {
            return None;
        }

        Some(Self {
            max_goals,
            max_actions_per_goal,
            max_action_evaluations,
            max_selected_intents,
            weights,
            thresholds,
        })
    }

    pub fn max_goals(self) -> usize {
        self.max_goals
    }

    pub fn max_actions_per_goal(self) -> usize {
        self.max_actions_per_goal
    }

    pub fn max_action_evaluations(self) -> usize {
        self.max_action_evaluations
    }

    pub fn max_selected_intents(self) -> usize {
        self.max_selected_intents
    }

    pub fn weights(self) -> ExecutiveUtilityWeights {
        self.weights
    }

    pub fn thresholds(self) -> ExecutiveSelectionThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutiveIntent {
    goal_identity: CognitiveStructure,
    action: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
    goal_pressure: CognitiveSignal,
    goal_alignment: CognitiveSignal,
    controllability: CognitiveSignal,
    evidence_confidence: CognitiveSignal,
    information_gain: CognitiveSignal,
    execution_cost: CognitiveSignal,
    gross_utility: CognitiveSignal,
    cost_penalty: CognitiveSignal,
    net_utility: CognitiveSignal,
    priority_adjusted_utility: CognitiveSignal,
}

impl ExecutiveIntent {
    pub fn goal_identity(&self) -> &CognitiveStructure {
        &self.goal_identity
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        &self.predicted_outcome
    }

    pub fn goal_pressure(&self) -> CognitiveSignal {
        self.goal_pressure
    }

    pub fn goal_alignment(&self) -> CognitiveSignal {
        self.goal_alignment
    }

    pub fn controllability(&self) -> CognitiveSignal {
        self.controllability
    }

    pub fn evidence_confidence(&self) -> CognitiveSignal {
        self.evidence_confidence
    }

    pub fn information_gain(&self) -> CognitiveSignal {
        self.information_gain
    }

    pub fn execution_cost(&self) -> CognitiveSignal {
        self.execution_cost
    }

    pub fn gross_utility(&self) -> CognitiveSignal {
        self.gross_utility
    }

    pub fn cost_penalty(&self) -> CognitiveSignal {
        self.cost_penalty
    }

    pub fn net_utility(&self) -> CognitiveSignal {
        self.net_utility
    }

    pub fn priority_adjusted_utility(&self) -> CognitiveSignal {
        self.priority_adjusted_utility
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutiveAgencyResult {
    input_goal_count: usize,
    considered_goal_count: usize,
    goal_frontier_truncated: bool,
    matching_candidate_count: usize,
    candidate_frontier_truncated: bool,
    evaluated_candidate_count: usize,
    evaluation_truncated: bool,
    rejected_by_threshold_count: usize,
    admitted_before_frontier: usize,
    selected: Vec<ExecutiveIntent>,
}

impl ExecutiveAgencyResult {
    pub fn input_goal_count(&self) -> usize {
        self.input_goal_count
    }

    pub fn considered_goal_count(&self) -> usize {
        self.considered_goal_count
    }

    pub fn goal_frontier_truncated(&self) -> bool {
        self.goal_frontier_truncated
    }

    pub fn matching_candidate_count(&self) -> usize {
        self.matching_candidate_count
    }

    pub fn candidate_frontier_truncated(&self) -> bool {
        self.candidate_frontier_truncated
    }

    pub fn evaluated_candidate_count(&self) -> usize {
        self.evaluated_candidate_count
    }

    pub fn evaluation_truncated(&self) -> bool {
        self.evaluation_truncated
    }

    pub fn rejected_by_threshold_count(&self) -> usize {
        self.rejected_by_threshold_count
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[ExecutiveIntent] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn abstained(&self) -> bool {
        self.selected.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ExecutiveAgency;

impl ExecutiveAgency {
    fn full_signal() -> CognitiveSignal {
        CognitiveSignal::new(1000).expect("full signal is valid")
    }

    pub(crate) fn scaled_product(left: CognitiveSignal, right: CognitiveSignal) -> CognitiveSignal {
        let scaled = (u32::from(left.value()) * u32::from(right.value())) / 1000;

        CognitiveSignal::new(scaled as u16).expect("scaled product remains on signal scale")
    }

    fn weighted_component(signal: CognitiveSignal, weight: u16, total_weight: u64) -> u64 {
        (u64::from(signal.value()) * u64::from(weight)) / total_weight
    }

    fn exact_tiebreak(left: &CognitiveStructure, right: &CognitiveStructure) -> std::cmp::Ordering {
        format!("{left:?}").cmp(&format!("{right:?}"))
    }

    fn compare_goal(left: &ExecutiveGoal, right: &ExecutiveGoal) -> std::cmp::Ordering {
        right
            .pressure()
            .value()
            .cmp(&left.pressure().value())
            .then_with(|| right.priority().value().cmp(&left.priority().value()))
            .then_with(|| {
                left.satisfaction()
                    .value()
                    .cmp(&right.satisfaction().value())
            })
            .then_with(|| Self::exact_tiebreak(left.identity(), right.identity()))
    }

    fn compare_candidate(
        left: &GroundedExecutiveActionCandidate,
        right: &GroundedExecutiveActionCandidate,
    ) -> std::cmp::Ordering {
        right
            .goal_alignment()
            .value()
            .cmp(&left.goal_alignment().value())
            .then_with(|| {
                right
                    .controllability()
                    .value()
                    .cmp(&left.controllability().value())
            })
            .then_with(|| {
                right
                    .evidence_confidence()
                    .value()
                    .cmp(&left.evidence_confidence().value())
            })
            .then_with(|| {
                right
                    .information_gain()
                    .value()
                    .cmp(&left.information_gain().value())
            })
            .then_with(|| {
                left.execution_cost()
                    .value()
                    .cmp(&right.execution_cost().value())
            })
            .then_with(|| Self::exact_tiebreak(left.action(), right.action()))
            .then_with(|| Self::exact_tiebreak(left.predicted_outcome(), right.predicted_outcome()))
    }

    fn evaluate(
        goal: &ExecutiveGoal,
        candidate: &GroundedExecutiveActionCandidate,
        policy: ExecutiveAgencyPolicy,
    ) -> Option<ExecutiveIntent> {
        let thresholds = policy.thresholds();

        let goal_pressure = goal.pressure();

        if goal_pressure.value() < thresholds.minimum_goal_pressure().value()
            || candidate.goal_alignment().value() < thresholds.minimum_goal_alignment().value()
            || candidate.controllability().value() < thresholds.minimum_controllability().value()
            || candidate.evidence_confidence().value()
                < thresholds.minimum_evidence_confidence().value()
        {
            return None;
        }

        let weights = policy.weights();

        let total_weight = weights.total_weight();

        let gross_value = Self::weighted_component(
            candidate.goal_alignment(),
            weights.goal_alignment(),
            total_weight,
        ) + Self::weighted_component(
            candidate.controllability(),
            weights.controllability(),
            total_weight,
        ) + Self::weighted_component(
            candidate.evidence_confidence(),
            weights.evidence_confidence(),
            total_weight,
        ) + Self::weighted_component(
            candidate.information_gain(),
            weights.information_gain(),
            total_weight,
        );

        let cost_value = Self::weighted_component(
            candidate.execution_cost(),
            weights.execution_cost(),
            total_weight,
        );

        let gross_utility = CognitiveSignal::new(gross_value.min(1000) as u16)
            .expect("gross utility remains on signal scale");

        let cost_penalty = CognitiveSignal::new(cost_value.min(1000) as u16)
            .expect("cost penalty remains on signal scale");

        let net_utility =
            CognitiveSignal::new(gross_utility.value().saturating_sub(cost_penalty.value()))
                .expect("net utility remains on signal scale");

        let priority_adjusted_utility = Self::scaled_product(net_utility, goal_pressure);

        if priority_adjusted_utility.value()
            < thresholds.minimum_priority_adjusted_utility().value()
        {
            return None;
        }

        Some(ExecutiveIntent {
            goal_identity: goal.identity().clone(),
            action: candidate.action().clone(),
            predicted_outcome: candidate.predicted_outcome().clone(),
            goal_pressure,
            goal_alignment: candidate.goal_alignment(),
            controllability: candidate.controllability(),
            evidence_confidence: candidate.evidence_confidence(),
            information_gain: candidate.information_gain(),
            execution_cost: candidate.execution_cost(),
            gross_utility,
            cost_penalty,
            net_utility,
            priority_adjusted_utility,
        })
    }

    fn compare_intent(left: &ExecutiveIntent, right: &ExecutiveIntent) -> std::cmp::Ordering {
        right
            .priority_adjusted_utility()
            .value()
            .cmp(&left.priority_adjusted_utility().value())
            .then_with(|| {
                right
                    .goal_pressure()
                    .value()
                    .cmp(&left.goal_pressure().value())
            })
            .then_with(|| right.net_utility().value().cmp(&left.net_utility().value()))
            .then_with(|| {
                right
                    .goal_alignment()
                    .value()
                    .cmp(&left.goal_alignment().value())
            })
            .then_with(|| {
                right
                    .controllability()
                    .value()
                    .cmp(&left.controllability().value())
            })
            .then_with(|| {
                right
                    .evidence_confidence()
                    .value()
                    .cmp(&left.evidence_confidence().value())
            })
            .then_with(|| {
                right
                    .information_gain()
                    .value()
                    .cmp(&left.information_gain().value())
            })
            .then_with(|| {
                left.execution_cost()
                    .value()
                    .cmp(&right.execution_cost().value())
            })
            .then_with(|| Self::exact_tiebreak(left.goal_identity(), right.goal_identity()))
            .then_with(|| Self::exact_tiebreak(left.action(), right.action()))
    }

    pub fn select(
        goals: &[ExecutiveGoal],
        candidates: &[GroundedExecutiveActionCandidate],
        policy: ExecutiveAgencyPolicy,
    ) -> ExecutiveAgencyResult {
        if goals.is_empty() || candidates.is_empty() {
            return ExecutiveAgencyResult {
                input_goal_count: goals.len(),
                considered_goal_count: 0,
                goal_frontier_truncated: false,
                matching_candidate_count: 0,
                candidate_frontier_truncated: false,
                evaluated_candidate_count: 0,
                evaluation_truncated: false,
                rejected_by_threshold_count: 0,
                admitted_before_frontier: 0,
                selected: Vec::new(),
            };
        }

        let mut considered_goals = goals
            .iter()
            .filter(|goal| !goal.is_satisfied())
            .collect::<Vec<_>>();

        considered_goals.sort_by(|left, right| Self::compare_goal(left, right));

        considered_goals.truncate(policy.max_goals());

        let considered_goal_count = considered_goals.len();

        let unsatisfied_goal_count = goals.iter().filter(|goal| !goal.is_satisfied()).count();

        let mut matching_candidate_count = 0_usize;

        let mut frontier_candidate_count = 0_usize;

        let mut evaluated_candidate_count = 0_usize;

        let mut rejected_by_threshold_count = 0_usize;

        let mut admitted = Vec::new();

        for goal in considered_goals {
            let mut goal_candidates = candidates
                .iter()
                .filter(|candidate| candidate.goal_identity() == goal.identity())
                .collect::<Vec<_>>();

            goal_candidates.sort_by(|left, right| Self::compare_candidate(left, right));

            goal_candidates.dedup();

            matching_candidate_count =
                matching_candidate_count.saturating_add(goal_candidates.len());

            goal_candidates.truncate(policy.max_actions_per_goal());

            frontier_candidate_count =
                frontier_candidate_count.saturating_add(goal_candidates.len());

            for candidate in goal_candidates {
                if evaluated_candidate_count >= policy.max_action_evaluations() {
                    break;
                }

                evaluated_candidate_count = evaluated_candidate_count.saturating_add(1);

                if let Some(intent) = Self::evaluate(goal, candidate, policy) {
                    admitted.push(intent);
                } else {
                    rejected_by_threshold_count = rejected_by_threshold_count.saturating_add(1);
                }
            }

            if evaluated_candidate_count >= policy.max_action_evaluations() {
                break;
            }
        }

        admitted.sort_by(Self::compare_intent);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_selected_intents());

        ExecutiveAgencyResult {
            input_goal_count: goals.len(),
            considered_goal_count,
            goal_frontier_truncated: unsatisfied_goal_count > considered_goal_count,
            matching_candidate_count,
            candidate_frontier_truncated: matching_candidate_count > frontier_candidate_count,
            evaluated_candidate_count,
            evaluation_truncated: frontier_candidate_count > evaluated_candidate_count,
            rejected_by_threshold_count,
            admitted_before_frontier,
            selected: admitted,
        }
    }

    pub fn full_confidence() -> CognitiveSignal {
        Self::full_signal()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalExecutiveAgency;

impl UniversalExecutiveAgency {
    pub fn evaluate(
        goals: &[ExecutiveGoal],
        candidates: &[GroundedExecutiveActionCandidate],
        policy: ExecutiveAgencyPolicy,
    ) -> ExecutiveAgencyResult {
        ExecutiveAgency::select(goals, candidates, policy)
    }
}
