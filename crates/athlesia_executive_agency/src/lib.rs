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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GoalPersistencePolicy {
    max_stalled_cycles: usize,
    switch_margin: CognitiveSignal,
    max_challengers: usize,
}

impl GoalPersistencePolicy {
    pub fn new(
        max_stalled_cycles: usize,
        switch_margin: CognitiveSignal,
        max_challengers: usize,
    ) -> Option<Self> {
        if max_stalled_cycles == 0
            || switch_margin == CognitiveSignal::zero()
            || max_challengers == 0
        {
            return None;
        }

        Some(Self {
            max_stalled_cycles,
            switch_margin,
            max_challengers,
        })
    }

    pub fn max_stalled_cycles(self) -> usize {
        self.max_stalled_cycles
    }

    pub fn switch_margin(self) -> CognitiveSignal {
        self.switch_margin
    }

    pub fn max_challengers(self) -> usize {
        self.max_challengers
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GoalPersistenceDecision {
    Abstained,
    Established,
    Continued,
    SwitchedChallenge,
    SwitchedGoalSatisfied,
    SwitchedGoalUnavailable,
    SwitchedIncumbentUnavailable,
    SwitchedStalled,
    ReleasedGoalSatisfied,
    ReleasedGoalUnavailable,
    ReleasedIncumbentUnavailable,
    ReleasedStalled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistentExecutiveCommitment {
    goal_identity: CognitiveStructure,
    action: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
    last_satisfaction: CognitiveSignal,
    stalled_cycles: usize,
    age_cycles: usize,
    current_priority_adjusted_utility: CognitiveSignal,
}

impl PersistentExecutiveCommitment {
    fn from_intent(intent: &ExecutiveIntent, satisfaction: CognitiveSignal) -> Self {
        Self {
            goal_identity: intent.goal_identity().clone(),
            action: intent.action().clone(),
            predicted_outcome: intent.predicted_outcome().clone(),
            last_satisfaction: satisfaction,
            stalled_cycles: 0,
            age_cycles: 1,
            current_priority_adjusted_utility: intent.priority_adjusted_utility(),
        }
    }

    fn refreshed(
        &self,
        intent: &ExecutiveIntent,
        satisfaction: CognitiveSignal,
        stalled_cycles: usize,
    ) -> Self {
        Self {
            goal_identity: self.goal_identity.clone(),
            action: self.action.clone(),
            predicted_outcome: self.predicted_outcome.clone(),
            last_satisfaction: satisfaction,
            stalled_cycles,
            age_cycles: self.age_cycles.saturating_add(1),
            current_priority_adjusted_utility: intent.priority_adjusted_utility(),
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

    pub fn last_satisfaction(&self) -> CognitiveSignal {
        self.last_satisfaction
    }

    pub fn stalled_cycles(&self) -> usize {
        self.stalled_cycles
    }

    pub fn age_cycles(&self) -> usize {
        self.age_cycles
    }

    pub fn current_priority_adjusted_utility(&self) -> CognitiveSignal {
        self.current_priority_adjusted_utility
    }

    pub fn matches_intent(&self, intent: &ExecutiveIntent) -> bool {
        self.goal_identity() == intent.goal_identity()
            && self.action() == intent.action()
            && self.predicted_outcome() == intent.predicted_outcome()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalPersistenceResult {
    input_had_commitment: bool,
    decision: GoalPersistenceDecision,
    incumbent_available: bool,
    progress_observed: bool,
    total_challenger_count: usize,
    considered_challenger_count: usize,
    challenger_frontier_truncated: bool,
    switch_margin_satisfied: bool,
    commitment: Option<PersistentExecutiveCommitment>,
}

impl GoalPersistenceResult {
    pub fn input_had_commitment(&self) -> bool {
        self.input_had_commitment
    }

    pub fn decision(&self) -> GoalPersistenceDecision {
        self.decision
    }

    pub fn incumbent_available(&self) -> bool {
        self.incumbent_available
    }

    pub fn progress_observed(&self) -> bool {
        self.progress_observed
    }

    pub fn total_challenger_count(&self) -> usize {
        self.total_challenger_count
    }

    pub fn considered_challenger_count(&self) -> usize {
        self.considered_challenger_count
    }

    pub fn challenger_frontier_truncated(&self) -> bool {
        self.challenger_frontier_truncated
    }

    pub fn switch_margin_satisfied(&self) -> bool {
        self.switch_margin_satisfied
    }

    pub fn commitment(&self) -> Option<&PersistentExecutiveCommitment> {
        self.commitment.as_ref()
    }

    pub fn has_commitment(&self) -> bool {
        self.commitment.is_some()
    }

    pub fn abstained(&self) -> bool {
        self.commitment.is_none()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct GoalPersistenceTransitionContext {
    input_had_commitment: bool,
    decision: GoalPersistenceDecision,
    incumbent_available: bool,
    progress_observed: bool,
    total_challenger_count: usize,
    considered_challenger_count: usize,
    switch_margin_satisfied: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GoalPersistence;

impl GoalPersistence {
    fn expanded_agency_policy(policy: ExecutiveAgencyPolicy) -> ExecutiveAgencyPolicy {
        ExecutiveAgencyPolicy::new(
            policy.max_goals(),
            policy.max_actions_per_goal(),
            policy.max_action_evaluations(),
            policy.max_action_evaluations(),
            policy.weights(),
            policy.thresholds(),
        )
        .expect(
            "validated executive agency policy expands final frontier within existing evaluation bound",
        )
    }

    fn goal_for_commitment<'a>(
        goals: &'a [ExecutiveGoal],
        commitment: &PersistentExecutiveCommitment,
    ) -> Option<&'a ExecutiveGoal> {
        goals
            .iter()
            .find(|goal| goal.identity() == commitment.goal_identity())
    }

    fn incumbent_intent<'a>(
        intents: &'a [ExecutiveIntent],
        commitment: &PersistentExecutiveCommitment,
    ) -> Option<&'a ExecutiveIntent> {
        intents
            .iter()
            .find(|intent| commitment.matches_intent(intent))
    }

    fn bounded_challengers<'a>(
        intents: &'a [ExecutiveIntent],
        commitment: Option<&PersistentExecutiveCommitment>,
        max_challengers: usize,
    ) -> (usize, Vec<&'a ExecutiveIntent>) {
        let mut challengers = intents
            .iter()
            .filter(|intent| {
                commitment
                    .map(|incumbent| !incumbent.matches_intent(intent))
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();

        let total = challengers.len();

        challengers.truncate(max_challengers);

        (total, challengers)
    }

    fn satisfaction_for_intent(
        goals: &[ExecutiveGoal],
        intent: &ExecutiveIntent,
    ) -> CognitiveSignal {
        goals
            .iter()
            .find(|goal| goal.identity() == intent.goal_identity())
            .map(ExecutiveGoal::satisfaction)
            .unwrap_or_else(CognitiveSignal::zero)
    }

    fn switch_margin_satisfied(
        incumbent: &ExecutiveIntent,
        challenger: &ExecutiveIntent,
        margin: CognitiveSignal,
    ) -> bool {
        u32::from(challenger.priority_adjusted_utility().value())
            >= u32::from(incumbent.priority_adjusted_utility().value()) + u32::from(margin.value())
    }

    fn switched_result(
        context: GoalPersistenceTransitionContext,
        challenger: &ExecutiveIntent,
        goals: &[ExecutiveGoal],
    ) -> GoalPersistenceResult {
        let satisfaction = Self::satisfaction_for_intent(goals, challenger);

        GoalPersistenceResult {
            input_had_commitment: context.input_had_commitment,
            decision: context.decision,
            incumbent_available: context.incumbent_available,
            progress_observed: context.progress_observed,
            total_challenger_count: context.total_challenger_count,
            considered_challenger_count: context.considered_challenger_count,
            challenger_frontier_truncated: context.total_challenger_count
                > context.considered_challenger_count,
            switch_margin_satisfied: context.switch_margin_satisfied,
            commitment: Some(PersistentExecutiveCommitment::from_intent(
                challenger,
                satisfaction,
            )),
        }
    }

    fn released_result(
        decision: GoalPersistenceDecision,
        incumbent_available: bool,
        progress_observed: bool,
        total_challenger_count: usize,
        considered_challenger_count: usize,
    ) -> GoalPersistenceResult {
        GoalPersistenceResult {
            input_had_commitment: true,
            decision,
            incumbent_available,
            progress_observed,
            total_challenger_count,
            considered_challenger_count,
            challenger_frontier_truncated: total_challenger_count > considered_challenger_count,
            switch_margin_satisfied: false,
            commitment: None,
        }
    }

    pub fn select(
        prior: Option<&PersistentExecutiveCommitment>,
        goals: &[ExecutiveGoal],
        candidates: &[GroundedExecutiveActionCandidate],
        agency_policy: ExecutiveAgencyPolicy,
        persistence_policy: GoalPersistencePolicy,
    ) -> GoalPersistenceResult {
        let expanded_policy = Self::expanded_agency_policy(agency_policy);

        let agency = ExecutiveAgency::select(goals, candidates, expanded_policy);

        let intents = agency.selected();

        let Some(incumbent) = prior else {
            let (total_challenger_count, challengers) =
                Self::bounded_challengers(intents, None, persistence_policy.max_challengers());

            let considered_challenger_count = challengers.len();

            if let Some(challenger) = challengers.first() {
                return Self::switched_result(
                    GoalPersistenceTransitionContext {
                        input_had_commitment: false,
                        decision: GoalPersistenceDecision::Established,
                        incumbent_available: false,
                        progress_observed: false,
                        total_challenger_count,
                        considered_challenger_count,
                        switch_margin_satisfied: false,
                    },
                    challenger,
                    goals,
                );
            }

            return GoalPersistenceResult {
                input_had_commitment: false,
                decision: GoalPersistenceDecision::Abstained,
                incumbent_available: false,
                progress_observed: false,
                total_challenger_count,
                considered_challenger_count,
                challenger_frontier_truncated: total_challenger_count > considered_challenger_count,
                switch_margin_satisfied: false,
                commitment: None,
            };
        };

        let current_goal = Self::goal_for_commitment(goals, incumbent);

        if current_goal.is_none() {
            let (total_challenger_count, challengers) = Self::bounded_challengers(
                intents,
                Some(incumbent),
                persistence_policy.max_challengers(),
            );

            let considered_challenger_count = challengers.len();

            if let Some(challenger) = challengers.first() {
                return Self::switched_result(
                    GoalPersistenceTransitionContext {
                        input_had_commitment: true,
                        decision: GoalPersistenceDecision::SwitchedGoalUnavailable,
                        incumbent_available: false,
                        progress_observed: false,
                        total_challenger_count,
                        considered_challenger_count,
                        switch_margin_satisfied: false,
                    },
                    challenger,
                    goals,
                );
            }

            return Self::released_result(
                GoalPersistenceDecision::ReleasedGoalUnavailable,
                false,
                false,
                total_challenger_count,
                considered_challenger_count,
            );
        }

        let current_goal = current_goal.expect("goal existence checked");

        if current_goal.is_satisfied() {
            let (total_challenger_count, challengers) = Self::bounded_challengers(
                intents,
                Some(incumbent),
                persistence_policy.max_challengers(),
            );

            let considered_challenger_count = challengers.len();

            if let Some(challenger) = challengers.first() {
                return Self::switched_result(
                    GoalPersistenceTransitionContext {
                        input_had_commitment: true,
                        decision: GoalPersistenceDecision::SwitchedGoalSatisfied,
                        incumbent_available: false,
                        progress_observed: true,
                        total_challenger_count,
                        considered_challenger_count,
                        switch_margin_satisfied: false,
                    },
                    challenger,
                    goals,
                );
            }

            return Self::released_result(
                GoalPersistenceDecision::ReleasedGoalSatisfied,
                false,
                true,
                total_challenger_count,
                considered_challenger_count,
            );
        }

        let progress_observed =
            current_goal.satisfaction().value() > incumbent.last_satisfaction().value();

        let next_stalled_cycles = if progress_observed {
            0
        } else {
            incumbent.stalled_cycles().saturating_add(1)
        };

        let current_incumbent_intent = Self::incumbent_intent(intents, incumbent);

        let (total_challenger_count, challengers) = Self::bounded_challengers(
            intents,
            Some(incumbent),
            persistence_policy.max_challengers(),
        );

        let considered_challenger_count = challengers.len();

        let Some(current_incumbent_intent) = current_incumbent_intent else {
            if let Some(challenger) = challengers.first() {
                return Self::switched_result(
                    GoalPersistenceTransitionContext {
                        input_had_commitment: true,
                        decision: GoalPersistenceDecision::SwitchedIncumbentUnavailable,
                        incumbent_available: false,
                        progress_observed,
                        total_challenger_count,
                        considered_challenger_count,
                        switch_margin_satisfied: false,
                    },
                    challenger,
                    goals,
                );
            }

            return Self::released_result(
                GoalPersistenceDecision::ReleasedIncumbentUnavailable,
                false,
                progress_observed,
                total_challenger_count,
                considered_challenger_count,
            );
        };

        if next_stalled_cycles >= persistence_policy.max_stalled_cycles() {
            if let Some(challenger) = challengers.first() {
                return Self::switched_result(
                    GoalPersistenceTransitionContext {
                        input_had_commitment: true,
                        decision: GoalPersistenceDecision::SwitchedStalled,
                        incumbent_available: true,
                        progress_observed,
                        total_challenger_count,
                        considered_challenger_count,
                        switch_margin_satisfied: false,
                    },
                    challenger,
                    goals,
                );
            }

            return Self::released_result(
                GoalPersistenceDecision::ReleasedStalled,
                true,
                progress_observed,
                total_challenger_count,
                considered_challenger_count,
            );
        }

        if let Some(challenger) = challengers.first() {
            let switch_margin_satisfied = Self::switch_margin_satisfied(
                current_incumbent_intent,
                challenger,
                persistence_policy.switch_margin(),
            );

            if switch_margin_satisfied {
                return Self::switched_result(
                    GoalPersistenceTransitionContext {
                        input_had_commitment: true,
                        decision: GoalPersistenceDecision::SwitchedChallenge,
                        incumbent_available: true,
                        progress_observed,
                        total_challenger_count,
                        considered_challenger_count,
                        switch_margin_satisfied: true,
                    },
                    challenger,
                    goals,
                );
            }
        }

        GoalPersistenceResult {
            input_had_commitment: true,
            decision: GoalPersistenceDecision::Continued,
            incumbent_available: true,
            progress_observed,
            total_challenger_count,
            considered_challenger_count,
            challenger_frontier_truncated: total_challenger_count > considered_challenger_count,
            switch_margin_satisfied: false,
            commitment: Some(incumbent.refreshed(
                current_incumbent_intent,
                current_goal.satisfaction(),
                next_stalled_cycles,
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalGoalPersistence;

impl UniversalGoalPersistence {
    pub fn evaluate(
        prior: Option<&PersistentExecutiveCommitment>,
        goals: &[ExecutiveGoal],
        candidates: &[GroundedExecutiveActionCandidate],
        agency_policy: ExecutiveAgencyPolicy,
        persistence_policy: GoalPersistencePolicy,
    ) -> GoalPersistenceResult {
        GoalPersistence::select(prior, goals, candidates, agency_policy, persistence_policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GoalConflictArbitrationThresholds {
    minimum_conflict_strength: CognitiveSignal,
    minimum_evidence_confidence: CognitiveSignal,
}

impl GoalConflictArbitrationThresholds {
    pub fn new(
        minimum_conflict_strength: CognitiveSignal,
        minimum_evidence_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_conflict_strength == CognitiveSignal::zero()
            || minimum_evidence_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_conflict_strength,
            minimum_evidence_confidence,
        })
    }

    pub fn minimum_conflict_strength(self) -> CognitiveSignal {
        self.minimum_conflict_strength
    }

    pub fn minimum_evidence_confidence(self) -> CognitiveSignal {
        self.minimum_evidence_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GoalConflictArbitrationPolicy {
    max_conflicts: usize,
    max_intents: usize,
    max_pair_evaluations: usize,
    max_selected_intents: usize,
    continuity_bonus: CognitiveSignal,
    thresholds: GoalConflictArbitrationThresholds,
}

impl GoalConflictArbitrationPolicy {
    pub fn new(
        max_conflicts: usize,
        max_intents: usize,
        max_pair_evaluations: usize,
        max_selected_intents: usize,
        continuity_bonus: CognitiveSignal,
        thresholds: GoalConflictArbitrationThresholds,
    ) -> Option<Self> {
        if max_conflicts == 0
            || max_intents == 0
            || max_pair_evaluations == 0
            || max_selected_intents == 0
        {
            return None;
        }

        Some(Self {
            max_conflicts,
            max_intents,
            max_pair_evaluations,
            max_selected_intents,
            continuity_bonus,
            thresholds,
        })
    }

    pub fn max_conflicts(self) -> usize {
        self.max_conflicts
    }

    pub fn max_intents(self) -> usize {
        self.max_intents
    }

    pub fn max_pair_evaluations(self) -> usize {
        self.max_pair_evaluations
    }

    pub fn max_selected_intents(self) -> usize {
        self.max_selected_intents
    }

    pub fn continuity_bonus(self) -> CognitiveSignal {
        self.continuity_bonus
    }

    pub fn thresholds(self) -> GoalConflictArbitrationThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalConflictEvidence {
    left_goal: CognitiveStructure,
    right_goal: CognitiveStructure,
    conflict_strength: CognitiveSignal,
    evidence_confidence: CognitiveSignal,
}

impl GoalConflictEvidence {
    pub fn new(
        left_goal: CognitiveStructure,
        right_goal: CognitiveStructure,
        conflict_strength: CognitiveSignal,
        evidence_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if left_goal == right_goal
            || conflict_strength == CognitiveSignal::zero()
            || evidence_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            left_goal,
            right_goal,
            conflict_strength,
            evidence_confidence,
        })
    }

    pub fn left_goal(&self) -> &CognitiveStructure {
        &self.left_goal
    }

    pub fn right_goal(&self) -> &CognitiveStructure {
        &self.right_goal
    }

    pub fn conflict_strength(&self) -> CognitiveSignal {
        self.conflict_strength
    }

    pub fn evidence_confidence(&self) -> CognitiveSignal {
        self.evidence_confidence
    }

    pub fn conflicts(&self, first: &CognitiveStructure, second: &CognitiveStructure) -> bool {
        (self.left_goal() == first && self.right_goal() == second)
            || (self.left_goal() == second && self.right_goal() == first)
    }

    fn equivalent(&self, other: &Self) -> bool {
        self.conflict_strength() == other.conflict_strength()
            && self.evidence_confidence() == other.evidence_confidence()
            && self.conflicts(other.left_goal(), other.right_goal())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbitratedExecutiveIntent {
    intent: ExecutiveIntent,
    continuity_applied: bool,
    arbitration_score: CognitiveSignal,
}

impl ArbitratedExecutiveIntent {
    pub fn intent(&self) -> &ExecutiveIntent {
        &self.intent
    }

    pub fn goal_identity(&self) -> &CognitiveStructure {
        self.intent.goal_identity()
    }

    pub fn action(&self) -> &CognitiveStructure {
        self.intent.action()
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        self.intent.predicted_outcome()
    }

    pub fn base_utility(&self) -> CognitiveSignal {
        self.intent.priority_adjusted_utility()
    }

    pub fn continuity_applied(&self) -> bool {
        self.continuity_applied
    }

    pub fn arbitration_score(&self) -> CognitiveSignal {
        self.arbitration_score
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuppressedGoalConflict {
    winner_goal: CognitiveStructure,
    loser_goal: CognitiveStructure,
    conflict_strength: CognitiveSignal,
    evidence_confidence: CognitiveSignal,
}

impl SuppressedGoalConflict {
    pub fn winner_goal(&self) -> &CognitiveStructure {
        &self.winner_goal
    }

    pub fn loser_goal(&self) -> &CognitiveStructure {
        &self.loser_goal
    }

    pub fn conflict_strength(&self) -> CognitiveSignal {
        self.conflict_strength
    }

    pub fn evidence_confidence(&self) -> CognitiveSignal {
        self.evidence_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalConflictArbitrationResult {
    input_intent_count: usize,
    considered_intent_count: usize,
    intent_frontier_truncated: bool,
    input_conflict_count: usize,
    eligible_conflict_count: usize,
    considered_conflict_count: usize,
    conflict_frontier_truncated: bool,
    evaluated_intent_count: usize,
    pair_evaluation_count: usize,
    pair_evaluation_truncated: bool,
    admitted_before_frontier: usize,
    selected: Vec<ArbitratedExecutiveIntent>,
    suppressed: Vec<SuppressedGoalConflict>,
}

impl GoalConflictArbitrationResult {
    pub fn input_intent_count(&self) -> usize {
        self.input_intent_count
    }

    pub fn considered_intent_count(&self) -> usize {
        self.considered_intent_count
    }

    pub fn intent_frontier_truncated(&self) -> bool {
        self.intent_frontier_truncated
    }

    pub fn input_conflict_count(&self) -> usize {
        self.input_conflict_count
    }

    pub fn eligible_conflict_count(&self) -> usize {
        self.eligible_conflict_count
    }

    pub fn considered_conflict_count(&self) -> usize {
        self.considered_conflict_count
    }

    pub fn conflict_frontier_truncated(&self) -> bool {
        self.conflict_frontier_truncated
    }

    pub fn evaluated_intent_count(&self) -> usize {
        self.evaluated_intent_count
    }

    pub fn pair_evaluation_count(&self) -> usize {
        self.pair_evaluation_count
    }

    pub fn pair_evaluation_truncated(&self) -> bool {
        self.pair_evaluation_truncated
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[ArbitratedExecutiveIntent] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn suppressed(&self) -> &[SuppressedGoalConflict] {
        &self.suppressed
    }

    pub fn suppressed_count(&self) -> usize {
        self.suppressed.len()
    }

    pub fn abstained(&self) -> bool {
        self.selected.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct GoalConflictArbitration;

impl GoalConflictArbitration {
    fn exact_tiebreak(left: &CognitiveStructure, right: &CognitiveStructure) -> std::cmp::Ordering {
        format!("{left:?}").cmp(&format!("{right:?}"))
    }

    fn canonical_pair<'a>(
        first: &'a CognitiveStructure,
        second: &'a CognitiveStructure,
    ) -> (&'a CognitiveStructure, &'a CognitiveStructure) {
        if Self::exact_tiebreak(first, second) != std::cmp::Ordering::Greater {
            (first, second)
        } else {
            (second, first)
        }
    }

    fn compare_conflict(
        left: &GoalConflictEvidence,
        right: &GoalConflictEvidence,
    ) -> std::cmp::Ordering {
        let (left_first, left_second) = Self::canonical_pair(left.left_goal(), left.right_goal());

        let (right_first, right_second) =
            Self::canonical_pair(right.left_goal(), right.right_goal());

        right
            .conflict_strength()
            .value()
            .cmp(&left.conflict_strength().value())
            .then_with(|| {
                right
                    .evidence_confidence()
                    .value()
                    .cmp(&left.evidence_confidence().value())
            })
            .then_with(|| Self::exact_tiebreak(left_first, right_first))
            .then_with(|| Self::exact_tiebreak(left_second, right_second))
    }

    fn matches_commitment(
        intent: &ExecutiveIntent,
        commitment: &PersistentExecutiveCommitment,
    ) -> bool {
        intent.goal_identity() == commitment.goal_identity()
            && intent.action() == commitment.action()
            && intent.predicted_outcome() == commitment.predicted_outcome()
    }

    fn adjusted_score(
        intent: &ExecutiveIntent,
        commitment: Option<&PersistentExecutiveCommitment>,
        continuity_bonus: CognitiveSignal,
    ) -> (bool, CognitiveSignal) {
        let continuity_applied = commitment
            .map(|current| Self::matches_commitment(intent, current))
            .unwrap_or(false);

        let base = u32::from(intent.priority_adjusted_utility().value());

        let bonus = if continuity_applied {
            u32::from(continuity_bonus.value())
        } else {
            0
        };

        let score = CognitiveSignal::new(base.saturating_add(bonus).min(1000) as u16)
            .expect("bounded arbitration score remains on signal scale");

        (continuity_applied, score)
    }

    fn compare_intent(
        left: &ArbitratedExecutiveIntent,
        right: &ArbitratedExecutiveIntent,
    ) -> std::cmp::Ordering {
        right
            .arbitration_score()
            .value()
            .cmp(&left.arbitration_score().value())
            .then_with(|| right.continuity_applied().cmp(&left.continuity_applied()))
            .then_with(|| {
                right
                    .base_utility()
                    .value()
                    .cmp(&left.base_utility().value())
            })
            .then_with(|| {
                right
                    .intent()
                    .goal_pressure()
                    .value()
                    .cmp(&left.intent().goal_pressure().value())
            })
            .then_with(|| Self::exact_tiebreak(left.goal_identity(), right.goal_identity()))
            .then_with(|| Self::exact_tiebreak(left.action(), right.action()))
            .then_with(|| Self::exact_tiebreak(left.predicted_outcome(), right.predicted_outcome()))
    }

    fn ranked_intents(
        intents: &[ExecutiveIntent],
        commitment: Option<&PersistentExecutiveCommitment>,
        policy: GoalConflictArbitrationPolicy,
    ) -> Vec<ArbitratedExecutiveIntent> {
        let mut ranked = intents
            .iter()
            .map(|intent| {
                let (continuity_applied, arbitration_score) =
                    Self::adjusted_score(intent, commitment, policy.continuity_bonus());

                ArbitratedExecutiveIntent {
                    intent: intent.clone(),
                    continuity_applied,
                    arbitration_score,
                }
            })
            .collect::<Vec<_>>();

        ranked.sort_by(Self::compare_intent);

        ranked.truncate(policy.max_intents());

        ranked
    }

    fn considered_conflicts(
        conflicts: &[GoalConflictEvidence],
        policy: GoalConflictArbitrationPolicy,
    ) -> (usize, Vec<&GoalConflictEvidence>) {
        let thresholds = policy.thresholds();

        let mut eligible = conflicts
            .iter()
            .filter(|evidence| {
                evidence.conflict_strength().value()
                    >= thresholds.minimum_conflict_strength().value()
                    && evidence.evidence_confidence().value()
                        >= thresholds.minimum_evidence_confidence().value()
            })
            .collect::<Vec<_>>();

        eligible.sort_by(|left, right| Self::compare_conflict(left, right));

        eligible.dedup_by(|left, right| left.equivalent(right));

        let eligible_count = eligible.len();

        eligible.truncate(policy.max_conflicts());

        (eligible_count, eligible)
    }

    fn find_conflict<'a>(
        conflicts: &'a [&GoalConflictEvidence],
        first: &CognitiveStructure,
        second: &CognitiveStructure,
    ) -> Option<&'a GoalConflictEvidence> {
        conflicts
            .iter()
            .copied()
            .find(|evidence| evidence.conflicts(first, second))
    }

    pub fn arbitrate(
        intents: &[ExecutiveIntent],
        conflicts: &[GoalConflictEvidence],
        commitment: Option<&PersistentExecutiveCommitment>,
        policy: GoalConflictArbitrationPolicy,
    ) -> GoalConflictArbitrationResult {
        let ranked = Self::ranked_intents(intents, commitment, policy);

        let considered_intent_count = ranked.len();

        let (eligible_conflict_count, considered_conflicts) =
            Self::considered_conflicts(conflicts, policy);

        let considered_conflict_count = considered_conflicts.len();

        let mut admitted: Vec<ArbitratedExecutiveIntent> = Vec::new();

        let mut suppressed: Vec<SuppressedGoalConflict> = Vec::new();

        let mut evaluated_intent_count = 0_usize;

        let mut pair_evaluation_count = 0_usize;

        let mut pair_evaluation_truncated = false;

        'candidate_loop: for candidate in ranked {
            if !considered_conflicts.is_empty() {
                for winner in &admitted {
                    if pair_evaluation_count >= policy.max_pair_evaluations() {
                        pair_evaluation_truncated = true;

                        break 'candidate_loop;
                    }

                    pair_evaluation_count = pair_evaluation_count.saturating_add(1);

                    if let Some(evidence) = Self::find_conflict(
                        &considered_conflicts,
                        winner.goal_identity(),
                        candidate.goal_identity(),
                    ) {
                        evaluated_intent_count = evaluated_intent_count.saturating_add(1);

                        suppressed.push(SuppressedGoalConflict {
                            winner_goal: winner.goal_identity().clone(),
                            loser_goal: candidate.goal_identity().clone(),
                            conflict_strength: evidence.conflict_strength(),
                            evidence_confidence: evidence.evidence_confidence(),
                        });

                        continue 'candidate_loop;
                    }
                }
            }

            evaluated_intent_count = evaluated_intent_count.saturating_add(1);

            admitted.push(candidate);
        }

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_selected_intents());

        GoalConflictArbitrationResult {
            input_intent_count: intents.len(),
            considered_intent_count,
            intent_frontier_truncated: intents.len() > considered_intent_count,
            input_conflict_count: conflicts.len(),
            eligible_conflict_count,
            considered_conflict_count,
            conflict_frontier_truncated: eligible_conflict_count > considered_conflict_count,
            evaluated_intent_count,
            pair_evaluation_count,
            pair_evaluation_truncated,
            admitted_before_frontier,
            selected: admitted,
            suppressed,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalGoalConflictArbitration;

impl UniversalGoalConflictArbitration {
    pub fn evaluate(
        intents: &[ExecutiveIntent],
        conflicts: &[GoalConflictEvidence],
        commitment: Option<&PersistentExecutiveCommitment>,
        policy: GoalConflictArbitrationPolicy,
    ) -> GoalConflictArbitrationResult {
        GoalConflictArbitration::arbitrate(intents, conflicts, commitment, policy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedIntentionStep {
    required_state: CognitiveStructure,
    action: CognitiveStructure,
    predicted_outcome: CognitiveStructure,
    evidence_confidence: CognitiveSignal,
    controllability: CognitiveSignal,
    execution_cost: CognitiveSignal,
}

impl GroundedIntentionStep {
    pub fn new(
        required_state: CognitiveStructure,
        action: CognitiveStructure,
        predicted_outcome: CognitiveStructure,
        evidence_confidence: CognitiveSignal,
        controllability: CognitiveSignal,
        execution_cost: CognitiveSignal,
    ) -> Self {
        Self {
            required_state,
            action,
            predicted_outcome,
            evidence_confidence,
            controllability,
            execution_cost,
        }
    }

    pub fn required_state(&self) -> &CognitiveStructure {
        &self.required_state
    }

    pub fn action(&self) -> &CognitiveStructure {
        &self.action
    }

    pub fn predicted_outcome(&self) -> &CognitiveStructure {
        &self.predicted_outcome
    }

    pub fn evidence_confidence(&self) -> CognitiveSignal {
        self.evidence_confidence
    }

    pub fn controllability(&self) -> CognitiveSignal {
        self.controllability
    }

    pub fn execution_cost(&self) -> CognitiveSignal {
        self.execution_cost
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiStepIntentionCandidate {
    goal_identity: CognitiveStructure,
    steps: Vec<GroundedIntentionStep>,
    terminal_goal_alignment: CognitiveSignal,
}

impl MultiStepIntentionCandidate {
    pub fn new(
        goal_identity: CognitiveStructure,
        steps: Vec<GroundedIntentionStep>,
        terminal_goal_alignment: CognitiveSignal,
    ) -> Option<Self> {
        if steps.len() < 2 || terminal_goal_alignment == CognitiveSignal::zero() {
            return None;
        }

        Some(Self {
            goal_identity,
            steps,
            terminal_goal_alignment,
        })
    }

    pub fn goal_identity(&self) -> &CognitiveStructure {
        &self.goal_identity
    }

    pub fn steps(&self) -> &[GroundedIntentionStep] {
        &self.steps
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn terminal_goal_alignment(&self) -> CognitiveSignal {
        self.terminal_goal_alignment
    }

    pub fn first_step(&self) -> &GroundedIntentionStep {
        &self.steps[0]
    }

    fn raw_cost(&self) -> u32 {
        self.steps
            .iter()
            .map(|step| u32::from(step.execution_cost().value()))
            .sum()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MultiStepIntentionThresholds {
    minimum_step_evidence_confidence: CognitiveSignal,
    minimum_step_controllability: CognitiveSignal,
    minimum_terminal_goal_alignment: CognitiveSignal,
    minimum_plan_confidence: CognitiveSignal,
}

impl MultiStepIntentionThresholds {
    pub fn new(
        minimum_step_evidence_confidence: CognitiveSignal,
        minimum_step_controllability: CognitiveSignal,
        minimum_terminal_goal_alignment: CognitiveSignal,
        minimum_plan_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_step_evidence_confidence == CognitiveSignal::zero()
            || minimum_step_controllability == CognitiveSignal::zero()
            || minimum_terminal_goal_alignment == CognitiveSignal::zero()
            || minimum_plan_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_step_evidence_confidence,
            minimum_step_controllability,
            minimum_terminal_goal_alignment,
            minimum_plan_confidence,
        })
    }

    pub fn minimum_step_evidence_confidence(self) -> CognitiveSignal {
        self.minimum_step_evidence_confidence
    }

    pub fn minimum_step_controllability(self) -> CognitiveSignal {
        self.minimum_step_controllability
    }

    pub fn minimum_terminal_goal_alignment(self) -> CognitiveSignal {
        self.minimum_terminal_goal_alignment
    }

    pub fn minimum_plan_confidence(self) -> CognitiveSignal {
        self.minimum_plan_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MultiStepIntentionPolicy {
    max_source_intents: usize,
    max_candidates: usize,
    max_steps_per_intention: usize,
    max_step_evaluations: usize,
    max_selected_intentions: usize,
    thresholds: MultiStepIntentionThresholds,
}

impl MultiStepIntentionPolicy {
    pub fn new(
        max_source_intents: usize,
        max_candidates: usize,
        max_steps_per_intention: usize,
        max_step_evaluations: usize,
        max_selected_intentions: usize,
        thresholds: MultiStepIntentionThresholds,
    ) -> Option<Self> {
        if max_source_intents == 0
            || max_candidates == 0
            || max_steps_per_intention < 2
            || max_step_evaluations == 0
            || max_selected_intentions == 0
        {
            return None;
        }

        Some(Self {
            max_source_intents,
            max_candidates,
            max_steps_per_intention,
            max_step_evaluations,
            max_selected_intentions,
            thresholds,
        })
    }

    pub fn max_source_intents(self) -> usize {
        self.max_source_intents
    }

    pub fn max_candidates(self) -> usize {
        self.max_candidates
    }

    pub fn max_steps_per_intention(self) -> usize {
        self.max_steps_per_intention
    }

    pub fn max_step_evaluations(self) -> usize {
        self.max_step_evaluations
    }

    pub fn max_selected_intentions(self) -> usize {
        self.max_selected_intentions
    }

    pub fn thresholds(self) -> MultiStepIntentionThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutableMultiStepIntention {
    source_intent: ArbitratedExecutiveIntent,
    goal_identity: CognitiveStructure,
    steps: Vec<GroundedIntentionStep>,
    weakest_step_evidence_confidence: CognitiveSignal,
    weakest_step_controllability: CognitiveSignal,
    terminal_goal_alignment: CognitiveSignal,
    path_confidence: CognitiveSignal,
    execution_cost_penalty: CognitiveSignal,
    net_intention_score: CognitiveSignal,
}

impl ExecutableMultiStepIntention {
    pub fn source_intent(&self) -> &ArbitratedExecutiveIntent {
        &self.source_intent
    }

    pub fn goal_identity(&self) -> &CognitiveStructure {
        &self.goal_identity
    }

    pub fn steps(&self) -> &[GroundedIntentionStep] {
        &self.steps
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn first_step(&self) -> &GroundedIntentionStep {
        &self.steps[0]
    }

    pub fn weakest_step_evidence_confidence(&self) -> CognitiveSignal {
        self.weakest_step_evidence_confidence
    }

    pub fn weakest_step_controllability(&self) -> CognitiveSignal {
        self.weakest_step_controllability
    }

    pub fn terminal_goal_alignment(&self) -> CognitiveSignal {
        self.terminal_goal_alignment
    }

    pub fn path_confidence(&self) -> CognitiveSignal {
        self.path_confidence
    }

    pub fn execution_cost_penalty(&self) -> CognitiveSignal {
        self.execution_cost_penalty
    }

    pub fn net_intention_score(&self) -> CognitiveSignal {
        self.net_intention_score
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiStepIntentionResult {
    input_source_intent_count: usize,
    considered_source_intent_count: usize,
    source_frontier_truncated: bool,
    input_candidate_count: usize,
    unique_candidate_count: usize,
    considered_candidate_count: usize,
    candidate_frontier_truncated: bool,
    step_evaluation_count: usize,
    step_evaluation_truncated: bool,
    rejected_source_mismatch_count: usize,
    rejected_over_step_bound_count: usize,
    rejected_structural_chain_count: usize,
    rejected_threshold_count: usize,
    admitted_before_frontier: usize,
    selected: Vec<ExecutableMultiStepIntention>,
}

impl MultiStepIntentionResult {
    pub fn input_source_intent_count(&self) -> usize {
        self.input_source_intent_count
    }

    pub fn considered_source_intent_count(&self) -> usize {
        self.considered_source_intent_count
    }

    pub fn source_frontier_truncated(&self) -> bool {
        self.source_frontier_truncated
    }

    pub fn input_candidate_count(&self) -> usize {
        self.input_candidate_count
    }

    pub fn unique_candidate_count(&self) -> usize {
        self.unique_candidate_count
    }

    pub fn considered_candidate_count(&self) -> usize {
        self.considered_candidate_count
    }

    pub fn candidate_frontier_truncated(&self) -> bool {
        self.candidate_frontier_truncated
    }

    pub fn step_evaluation_count(&self) -> usize {
        self.step_evaluation_count
    }

    pub fn step_evaluation_truncated(&self) -> bool {
        self.step_evaluation_truncated
    }

    pub fn rejected_source_mismatch_count(&self) -> usize {
        self.rejected_source_mismatch_count
    }

    pub fn rejected_over_step_bound_count(&self) -> usize {
        self.rejected_over_step_bound_count
    }

    pub fn rejected_structural_chain_count(&self) -> usize {
        self.rejected_structural_chain_count
    }

    pub fn rejected_threshold_count(&self) -> usize {
        self.rejected_threshold_count
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[ExecutableMultiStepIntention] {
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
pub struct MultiStepIntention;

impl MultiStepIntention {
    fn exact_tiebreak(left: &CognitiveStructure, right: &CognitiveStructure) -> std::cmp::Ordering {
        format!("{left:?}").cmp(&format!("{right:?}"))
    }

    fn compare_source(
        left: &ArbitratedExecutiveIntent,
        right: &ArbitratedExecutiveIntent,
    ) -> std::cmp::Ordering {
        right
            .arbitration_score()
            .value()
            .cmp(&left.arbitration_score().value())
            .then_with(|| {
                right
                    .base_utility()
                    .value()
                    .cmp(&left.base_utility().value())
            })
            .then_with(|| Self::exact_tiebreak(left.goal_identity(), right.goal_identity()))
            .then_with(|| Self::exact_tiebreak(left.action(), right.action()))
            .then_with(|| Self::exact_tiebreak(left.predicted_outcome(), right.predicted_outcome()))
    }

    fn compare_candidate(
        left: &MultiStepIntentionCandidate,
        right: &MultiStepIntentionCandidate,
    ) -> std::cmp::Ordering {
        right
            .terminal_goal_alignment()
            .value()
            .cmp(&left.terminal_goal_alignment().value())
            .then_with(|| left.raw_cost().cmp(&right.raw_cost()))
            .then_with(|| left.step_count().cmp(&right.step_count()))
            .then_with(|| Self::exact_tiebreak(left.goal_identity(), right.goal_identity()))
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    fn compare_selected(
        left: &ExecutableMultiStepIntention,
        right: &ExecutableMultiStepIntention,
    ) -> std::cmp::Ordering {
        right
            .net_intention_score()
            .value()
            .cmp(&left.net_intention_score().value())
            .then_with(|| {
                right
                    .path_confidence()
                    .value()
                    .cmp(&left.path_confidence().value())
            })
            .then_with(|| {
                left.execution_cost_penalty()
                    .value()
                    .cmp(&right.execution_cost_penalty().value())
            })
            .then_with(|| {
                right
                    .source_intent()
                    .arbitration_score()
                    .value()
                    .cmp(&left.source_intent().arbitration_score().value())
            })
            .then_with(|| Self::exact_tiebreak(left.goal_identity(), right.goal_identity()))
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    fn ranked_sources(
        source_intents: &[ArbitratedExecutiveIntent],
        policy: MultiStepIntentionPolicy,
    ) -> Vec<ArbitratedExecutiveIntent> {
        let mut ranked = source_intents.to_vec();

        ranked.sort_by(Self::compare_source);

        ranked.truncate(policy.max_source_intents());

        ranked
    }

    fn ranked_candidates(
        candidates: &[MultiStepIntentionCandidate],
        policy: MultiStepIntentionPolicy,
    ) -> (usize, Vec<MultiStepIntentionCandidate>) {
        let mut ranked = candidates.to_vec();

        ranked.sort_by(Self::compare_candidate);

        ranked.dedup();

        let unique_count = ranked.len();

        ranked.truncate(policy.max_candidates());

        (unique_count, ranked)
    }

    fn source_for_candidate<'a>(
        source_intents: &'a [ArbitratedExecutiveIntent],
        candidate: &MultiStepIntentionCandidate,
    ) -> Option<&'a ArbitratedExecutiveIntent> {
        let first = candidate.first_step();

        source_intents.iter().find(|source| {
            source.goal_identity() == candidate.goal_identity()
                && source.action() == first.action()
                && source.predicted_outcome() == first.predicted_outcome()
        })
    }

    fn structurally_continuous(candidate: &MultiStepIntentionCandidate) -> bool {
        candidate
            .steps()
            .windows(2)
            .all(|pair| pair[1].required_state() == pair[0].predicted_outcome())
    }

    fn signal_from_value(value: u16) -> CognitiveSignal {
        CognitiveSignal::new(value).expect("bounded cognitive value remains valid")
    }

    fn weakest_evidence(candidate: &MultiStepIntentionCandidate) -> CognitiveSignal {
        let value = candidate
            .steps()
            .iter()
            .map(|step| step.evidence_confidence().value())
            .min()
            .expect("multi-step candidate always has steps");

        Self::signal_from_value(value)
    }

    fn weakest_controllability(candidate: &MultiStepIntentionCandidate) -> CognitiveSignal {
        let value = candidate
            .steps()
            .iter()
            .map(|step| step.controllability().value())
            .min()
            .expect("multi-step candidate always has steps");

        Self::signal_from_value(value)
    }

    fn cost_penalty(candidate: &MultiStepIntentionCandidate) -> CognitiveSignal {
        Self::signal_from_value(candidate.raw_cost().min(1000) as u16)
    }

    fn path_confidence(
        source: &ArbitratedExecutiveIntent,
        weakest_evidence: CognitiveSignal,
        weakest_controllability: CognitiveSignal,
        terminal_goal_alignment: CognitiveSignal,
    ) -> CognitiveSignal {
        let source_and_evidence =
            ExecutiveAgency::scaled_product(source.arbitration_score(), weakest_evidence);

        let controlled =
            ExecutiveAgency::scaled_product(source_and_evidence, weakest_controllability);

        ExecutiveAgency::scaled_product(controlled, terminal_goal_alignment)
    }

    fn evaluate_candidate(
        source: &ArbitratedExecutiveIntent,
        candidate: &MultiStepIntentionCandidate,
        policy: MultiStepIntentionPolicy,
    ) -> Option<ExecutableMultiStepIntention> {
        let thresholds = policy.thresholds();

        let weakest_step_evidence_confidence = Self::weakest_evidence(candidate);

        let weakest_step_controllability = Self::weakest_controllability(candidate);

        if weakest_step_evidence_confidence.value()
            < thresholds.minimum_step_evidence_confidence().value()
            || weakest_step_controllability.value()
                < thresholds.minimum_step_controllability().value()
            || candidate.terminal_goal_alignment().value()
                < thresholds.minimum_terminal_goal_alignment().value()
        {
            return None;
        }

        let path_confidence = Self::path_confidence(
            source,
            weakest_step_evidence_confidence,
            weakest_step_controllability,
            candidate.terminal_goal_alignment(),
        );

        if path_confidence.value() < thresholds.minimum_plan_confidence().value() {
            return None;
        }

        let execution_cost_penalty = Self::cost_penalty(candidate);

        let net_intention_score = Self::signal_from_value(
            path_confidence
                .value()
                .saturating_sub(execution_cost_penalty.value()),
        );

        Some(ExecutableMultiStepIntention {
            source_intent: source.clone(),
            goal_identity: candidate.goal_identity().clone(),
            steps: candidate.steps().to_vec(),
            weakest_step_evidence_confidence,
            weakest_step_controllability,
            terminal_goal_alignment: candidate.terminal_goal_alignment(),
            path_confidence,
            execution_cost_penalty,
            net_intention_score,
        })
    }

    pub fn select(
        source_intents: &[ArbitratedExecutiveIntent],
        candidates: &[MultiStepIntentionCandidate],
        policy: MultiStepIntentionPolicy,
    ) -> MultiStepIntentionResult {
        let ranked_sources = Self::ranked_sources(source_intents, policy);

        let considered_source_intent_count = ranked_sources.len();

        let (unique_candidate_count, ranked_candidates) =
            Self::ranked_candidates(candidates, policy);

        let considered_candidate_count = ranked_candidates.len();

        let mut step_evaluation_count = 0_usize;

        let mut step_evaluation_truncated = false;

        let mut rejected_source_mismatch_count = 0_usize;

        let mut rejected_over_step_bound_count = 0_usize;

        let mut rejected_structural_chain_count = 0_usize;

        let mut rejected_threshold_count = 0_usize;

        let mut admitted: Vec<ExecutableMultiStepIntention> = Vec::new();

        for candidate in ranked_candidates {
            let Some(source) = Self::source_for_candidate(&ranked_sources, &candidate) else {
                rejected_source_mismatch_count = rejected_source_mismatch_count.saturating_add(1);

                continue;
            };

            if candidate.step_count() > policy.max_steps_per_intention() {
                rejected_over_step_bound_count = rejected_over_step_bound_count.saturating_add(1);

                continue;
            }

            let required_evaluations = candidate.step_count();

            if step_evaluation_count.saturating_add(required_evaluations)
                > policy.max_step_evaluations()
            {
                step_evaluation_truncated = true;

                break;
            }

            step_evaluation_count = step_evaluation_count.saturating_add(required_evaluations);

            if !Self::structurally_continuous(&candidate) {
                rejected_structural_chain_count = rejected_structural_chain_count.saturating_add(1);

                continue;
            }

            if let Some(intention) = Self::evaluate_candidate(source, &candidate, policy) {
                admitted.push(intention);
            } else {
                rejected_threshold_count = rejected_threshold_count.saturating_add(1);
            }
        }

        admitted.sort_by(Self::compare_selected);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_selected_intentions());

        MultiStepIntentionResult {
            input_source_intent_count: source_intents.len(),
            considered_source_intent_count,
            source_frontier_truncated: source_intents.len() > considered_source_intent_count,
            input_candidate_count: candidates.len(),
            unique_candidate_count,
            considered_candidate_count,
            candidate_frontier_truncated: unique_candidate_count > considered_candidate_count,
            step_evaluation_count,
            step_evaluation_truncated,
            rejected_source_mismatch_count,
            rejected_over_step_bound_count,
            rejected_structural_chain_count,
            rejected_threshold_count,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalMultiStepIntention;

impl UniversalMultiStepIntention {
    pub fn evaluate(
        source_intents: &[ArbitratedExecutiveIntent],
        candidates: &[MultiStepIntentionCandidate],
        policy: MultiStepIntentionPolicy,
    ) -> MultiStepIntentionResult {
        MultiStepIntention::select(source_intents, candidates, policy)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedExecutionObservation {
    observed_state: CognitiveStructure,
    observed_action: CognitiveStructure,
    observed_outcome: CognitiveStructure,
    observation_confidence: CognitiveSignal,
}

impl GroundedExecutionObservation {
    pub fn new(
        observed_state: CognitiveStructure,
        observed_action: CognitiveStructure,
        observed_outcome: CognitiveStructure,
        observation_confidence: CognitiveSignal,
    ) -> Self {
        Self {
            observed_state,
            observed_action,
            observed_outcome,
            observation_confidence,
        }
    }

    pub fn observed_state(&self) -> &CognitiveStructure {
        &self.observed_state
    }

    pub fn observed_action(&self) -> &CognitiveStructure {
        &self.observed_action
    }

    pub fn observed_outcome(&self) -> &CognitiveStructure {
        &self.observed_outcome
    }

    pub fn observation_confidence(&self) -> CognitiveSignal {
        self.observation_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ExecutionDeviationKind {
    StateMismatch,
    ActionMismatch,
    OutcomeMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentionExecutionDeviation {
    step_index: usize,
    kind: ExecutionDeviationKind,
    expected_state: CognitiveStructure,
    observed_state: CognitiveStructure,
    expected_action: CognitiveStructure,
    observed_action: CognitiveStructure,
    expected_outcome: CognitiveStructure,
    observed_outcome: CognitiveStructure,
    observation_confidence: CognitiveSignal,
}

impl IntentionExecutionDeviation {
    pub fn step_index(&self) -> usize {
        self.step_index
    }

    pub fn kind(&self) -> ExecutionDeviationKind {
        self.kind
    }

    pub fn expected_state(&self) -> &CognitiveStructure {
        &self.expected_state
    }

    pub fn observed_state(&self) -> &CognitiveStructure {
        &self.observed_state
    }

    pub fn expected_action(&self) -> &CognitiveStructure {
        &self.expected_action
    }

    pub fn observed_action(&self) -> &CognitiveStructure {
        &self.observed_action
    }

    pub fn expected_outcome(&self) -> &CognitiveStructure {
        &self.expected_outcome
    }

    pub fn observed_outcome(&self) -> &CognitiveStructure {
        &self.observed_outcome
    }

    pub fn observation_confidence(&self) -> CognitiveSignal {
        self.observation_confidence
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IntentionExecutionStatus {
    Pending,
    Inconclusive,
    Advanced,
    Completed,
    Deviated,
    StepBoundExceeded,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IntentionExecutionMonitoringPolicy {
    max_steps_per_intention: usize,
    max_observations: usize,
    minimum_observation_confidence: CognitiveSignal,
}

impl IntentionExecutionMonitoringPolicy {
    pub fn new(
        max_steps_per_intention: usize,
        max_observations: usize,
        minimum_observation_confidence: CognitiveSignal,
    ) -> Option<Self> {
        if max_steps_per_intention == 0
            || max_observations == 0
            || minimum_observation_confidence == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            max_steps_per_intention,
            max_observations,
            minimum_observation_confidence,
        })
    }

    pub fn max_steps_per_intention(self) -> usize {
        self.max_steps_per_intention
    }

    pub fn max_observations(self) -> usize {
        self.max_observations
    }

    pub fn minimum_observation_confidence(self) -> CognitiveSignal {
        self.minimum_observation_confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentionExecutionMonitoringResult {
    status: IntentionExecutionStatus,
    input_observation_count: usize,
    considered_observation_count: usize,
    observation_frontier_truncated: bool,
    confirmed_step_count: usize,
    low_confidence_observation_count: usize,
    next_step_index: Option<usize>,
    remaining_step_count: usize,
    deviation: Option<IntentionExecutionDeviation>,
}

impl IntentionExecutionMonitoringResult {
    pub fn status(&self) -> IntentionExecutionStatus {
        self.status
    }

    pub fn input_observation_count(&self) -> usize {
        self.input_observation_count
    }

    pub fn considered_observation_count(&self) -> usize {
        self.considered_observation_count
    }

    pub fn observation_frontier_truncated(&self) -> bool {
        self.observation_frontier_truncated
    }

    pub fn confirmed_step_count(&self) -> usize {
        self.confirmed_step_count
    }

    pub fn low_confidence_observation_count(&self) -> usize {
        self.low_confidence_observation_count
    }

    pub fn next_step_index(&self) -> Option<usize> {
        self.next_step_index
    }

    pub fn remaining_step_count(&self) -> usize {
        self.remaining_step_count
    }

    pub fn deviation(&self) -> Option<&IntentionExecutionDeviation> {
        self.deviation.as_ref()
    }

    pub fn completed(&self) -> bool {
        self.status == IntentionExecutionStatus::Completed
    }

    pub fn deviated(&self) -> bool {
        self.status == IntentionExecutionStatus::Deviated
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct IntentionExecutionMonitor;

impl IntentionExecutionMonitor {
    fn deviation(
        step_index: usize,
        expected: &GroundedIntentionStep,
        observation: &GroundedExecutionObservation,
    ) -> Option<IntentionExecutionDeviation> {
        let kind = if observation.observed_state() != expected.required_state() {
            Some(ExecutionDeviationKind::StateMismatch)
        } else if observation.observed_action() != expected.action() {
            Some(ExecutionDeviationKind::ActionMismatch)
        } else if observation.observed_outcome() != expected.predicted_outcome() {
            Some(ExecutionDeviationKind::OutcomeMismatch)
        } else {
            None
        };

        kind.map(|kind| IntentionExecutionDeviation {
            step_index,
            kind,
            expected_state: expected.required_state().clone(),
            observed_state: observation.observed_state().clone(),
            expected_action: expected.action().clone(),
            observed_action: observation.observed_action().clone(),
            expected_outcome: expected.predicted_outcome().clone(),
            observed_outcome: observation.observed_outcome().clone(),
            observation_confidence: observation.observation_confidence(),
        })
    }

    fn result_status(
        confirmed_step_count: usize,
        low_confidence_observation_count: usize,
    ) -> IntentionExecutionStatus {
        if confirmed_step_count > 0 {
            IntentionExecutionStatus::Advanced
        } else if low_confidence_observation_count > 0 {
            IntentionExecutionStatus::Inconclusive
        } else {
            IntentionExecutionStatus::Pending
        }
    }

    pub fn monitor(
        intention: &ExecutableMultiStepIntention,
        observations: &[GroundedExecutionObservation],
        policy: IntentionExecutionMonitoringPolicy,
    ) -> IntentionExecutionMonitoringResult {
        let input_observation_count = observations.len();

        if intention.step_count() > policy.max_steps_per_intention() {
            return IntentionExecutionMonitoringResult {
                status: IntentionExecutionStatus::StepBoundExceeded,
                input_observation_count,
                considered_observation_count: 0,
                observation_frontier_truncated: false,
                confirmed_step_count: 0,
                low_confidence_observation_count: 0,
                next_step_index: None,
                remaining_step_count: intention.step_count(),
                deviation: None,
            };
        }

        let observation_limit = observations.len().min(policy.max_observations());

        let observation_frontier_truncated = observations.len() > observation_limit;

        let mut confirmed_step_count = 0_usize;

        let mut considered_observation_count = 0_usize;

        let mut low_confidence_observation_count = 0_usize;

        let mut deviation = None;

        for observation in observations.iter().take(observation_limit) {
            if confirmed_step_count >= intention.step_count() {
                break;
            }

            considered_observation_count = considered_observation_count.saturating_add(1);

            if observation.observation_confidence().value()
                < policy.minimum_observation_confidence().value()
            {
                low_confidence_observation_count =
                    low_confidence_observation_count.saturating_add(1);

                continue;
            }

            let expected = &intention.steps()[confirmed_step_count];

            if let Some(detected) = Self::deviation(confirmed_step_count, expected, observation) {
                deviation = Some(detected);

                break;
            }

            confirmed_step_count = confirmed_step_count.saturating_add(1);

            if confirmed_step_count == intention.step_count() {
                break;
            }
        }

        if deviation.is_some() {
            let remaining_step_count = intention.step_count().saturating_sub(confirmed_step_count);

            return IntentionExecutionMonitoringResult {
                status: IntentionExecutionStatus::Deviated,
                input_observation_count,
                considered_observation_count,
                observation_frontier_truncated,
                confirmed_step_count,
                low_confidence_observation_count,
                next_step_index: Some(confirmed_step_count),
                remaining_step_count,
                deviation,
            };
        }

        if confirmed_step_count == intention.step_count() {
            return IntentionExecutionMonitoringResult {
                status: IntentionExecutionStatus::Completed,
                input_observation_count,
                considered_observation_count,
                observation_frontier_truncated,
                confirmed_step_count,
                low_confidence_observation_count,
                next_step_index: None,
                remaining_step_count: 0,
                deviation: None,
            };
        }

        IntentionExecutionMonitoringResult {
            status: Self::result_status(confirmed_step_count, low_confidence_observation_count),
            input_observation_count,
            considered_observation_count,
            observation_frontier_truncated,
            confirmed_step_count,
            low_confidence_observation_count,
            next_step_index: Some(confirmed_step_count),
            remaining_step_count: intention.step_count().saturating_sub(confirmed_step_count),
            deviation: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalIntentionExecutionMonitor;

impl UniversalIntentionExecutionMonitor {
    pub fn evaluate(
        intention: &ExecutableMultiStepIntention,
        observations: &[GroundedExecutionObservation],
        policy: IntentionExecutionMonitoringPolicy,
    ) -> IntentionExecutionMonitoringResult {
        IntentionExecutionMonitor::monitor(intention, observations, policy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DeviationReplanningStatus {
    NotTriggered,
    EvidenceInsufficient,
    NoViableReplacement,
    ReplacementSelected,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeviationReplanningThresholds {
    minimum_observation_confidence: CognitiveSignal,
    minimum_replacement_path_confidence: CognitiveSignal,
    minimum_adjusted_replan_score: CognitiveSignal,
}

impl DeviationReplanningThresholds {
    pub fn new(
        minimum_observation_confidence: CognitiveSignal,
        minimum_replacement_path_confidence: CognitiveSignal,
        minimum_adjusted_replan_score: CognitiveSignal,
    ) -> Option<Self> {
        if minimum_observation_confidence == CognitiveSignal::zero()
            || minimum_replacement_path_confidence == CognitiveSignal::zero()
            || minimum_adjusted_replan_score == CognitiveSignal::zero()
        {
            return None;
        }

        Some(Self {
            minimum_observation_confidence,
            minimum_replacement_path_confidence,
            minimum_adjusted_replan_score,
        })
    }

    pub fn minimum_observation_confidence(self) -> CognitiveSignal {
        self.minimum_observation_confidence
    }

    pub fn minimum_replacement_path_confidence(self) -> CognitiveSignal {
        self.minimum_replacement_path_confidence
    }

    pub fn minimum_adjusted_replan_score(self) -> CognitiveSignal {
        self.minimum_adjusted_replan_score
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeviationReplanningPolicy {
    max_candidates: usize,
    max_candidate_evaluations: usize,
    max_steps_per_replacement: usize,
    max_selected_replans: usize,
    thresholds: DeviationReplanningThresholds,
}

impl DeviationReplanningPolicy {
    pub fn new(
        max_candidates: usize,
        max_candidate_evaluations: usize,
        max_steps_per_replacement: usize,
        max_selected_replans: usize,
        thresholds: DeviationReplanningThresholds,
    ) -> Option<Self> {
        if max_candidates == 0
            || max_candidate_evaluations == 0
            || max_steps_per_replacement < 2
            || max_selected_replans == 0
        {
            return None;
        }

        Some(Self {
            max_candidates,
            max_candidate_evaluations,
            max_steps_per_replacement,
            max_selected_replans,
            thresholds,
        })
    }

    pub fn max_candidates(self) -> usize {
        self.max_candidates
    }

    pub fn max_candidate_evaluations(self) -> usize {
        self.max_candidate_evaluations
    }

    pub fn max_steps_per_replacement(self) -> usize {
        self.max_steps_per_replacement
    }

    pub fn max_selected_replans(self) -> usize {
        self.max_selected_replans
    }

    pub fn thresholds(self) -> DeviationReplanningThresholds {
        self.thresholds
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroundedDeviationReplan {
    replacement: ExecutableMultiStepIntention,
    deviation_step_index: usize,
    recovery_state: CognitiveStructure,
    deviation_observation_confidence: CognitiveSignal,
    adjusted_replan_score: CognitiveSignal,
}

impl GroundedDeviationReplan {
    pub fn replacement(&self) -> &ExecutableMultiStepIntention {
        &self.replacement
    }

    pub fn deviation_step_index(&self) -> usize {
        self.deviation_step_index
    }

    pub fn recovery_state(&self) -> &CognitiveStructure {
        &self.recovery_state
    }

    pub fn deviation_observation_confidence(&self) -> CognitiveSignal {
        self.deviation_observation_confidence
    }

    pub fn adjusted_replan_score(&self) -> CognitiveSignal {
        self.adjusted_replan_score
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviationReplanningResult {
    status: DeviationReplanningStatus,
    input_candidate_count: usize,
    unique_candidate_count: usize,
    considered_candidate_count: usize,
    candidate_frontier_truncated: bool,
    candidate_evaluation_count: usize,
    candidate_evaluation_truncated: bool,
    rejected_goal_mismatch_count: usize,
    rejected_recovery_anchor_count: usize,
    rejected_step_bound_count: usize,
    rejected_threshold_count: usize,
    admitted_before_frontier: usize,
    selected: Vec<GroundedDeviationReplan>,
}

impl DeviationReplanningResult {
    pub fn status(&self) -> DeviationReplanningStatus {
        self.status
    }

    pub fn input_candidate_count(&self) -> usize {
        self.input_candidate_count
    }

    pub fn unique_candidate_count(&self) -> usize {
        self.unique_candidate_count
    }

    pub fn considered_candidate_count(&self) -> usize {
        self.considered_candidate_count
    }

    pub fn candidate_frontier_truncated(&self) -> bool {
        self.candidate_frontier_truncated
    }

    pub fn candidate_evaluation_count(&self) -> usize {
        self.candidate_evaluation_count
    }

    pub fn candidate_evaluation_truncated(&self) -> bool {
        self.candidate_evaluation_truncated
    }

    pub fn rejected_goal_mismatch_count(&self) -> usize {
        self.rejected_goal_mismatch_count
    }

    pub fn rejected_recovery_anchor_count(&self) -> usize {
        self.rejected_recovery_anchor_count
    }

    pub fn rejected_step_bound_count(&self) -> usize {
        self.rejected_step_bound_count
    }

    pub fn rejected_threshold_count(&self) -> usize {
        self.rejected_threshold_count
    }

    pub fn admitted_before_frontier(&self) -> usize {
        self.admitted_before_frontier
    }

    pub fn selected(&self) -> &[GroundedDeviationReplan] {
        &self.selected
    }

    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    pub fn triggered(&self) -> bool {
        matches!(
            self.status,
            DeviationReplanningStatus::NoViableReplacement
                | DeviationReplanningStatus::ReplacementSelected
        )
    }

    pub fn abstained(&self) -> bool {
        self.selected.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct DeviationReplanner;

impl DeviationReplanner {
    fn exact_tiebreak(left: &CognitiveStructure, right: &CognitiveStructure) -> std::cmp::Ordering {
        format!("{left:?}").cmp(&format!("{right:?}"))
    }

    fn compare_candidate(
        left: &ExecutableMultiStepIntention,
        right: &ExecutableMultiStepIntention,
    ) -> std::cmp::Ordering {
        right
            .net_intention_score()
            .value()
            .cmp(&left.net_intention_score().value())
            .then_with(|| {
                right
                    .path_confidence()
                    .value()
                    .cmp(&left.path_confidence().value())
            })
            .then_with(|| {
                left.execution_cost_penalty()
                    .value()
                    .cmp(&right.execution_cost_penalty().value())
            })
            .then_with(|| Self::exact_tiebreak(left.goal_identity(), right.goal_identity()))
            .then_with(|| {
                Self::exact_tiebreak(
                    left.first_step().required_state(),
                    right.first_step().required_state(),
                )
            })
            .then_with(|| {
                Self::exact_tiebreak(left.first_step().action(), right.first_step().action())
            })
            .then_with(|| format!("{left:?}").cmp(&format!("{right:?}")))
    }

    fn compare_replan(
        left: &GroundedDeviationReplan,
        right: &GroundedDeviationReplan,
    ) -> std::cmp::Ordering {
        right
            .adjusted_replan_score()
            .value()
            .cmp(&left.adjusted_replan_score().value())
            .then_with(|| Self::compare_candidate(left.replacement(), right.replacement()))
    }

    fn ranked_candidates(
        candidates: &[ExecutableMultiStepIntention],
        policy: DeviationReplanningPolicy,
    ) -> (usize, Vec<ExecutableMultiStepIntention>) {
        let mut ranked = candidates.to_vec();

        ranked.sort_by(Self::compare_candidate);

        ranked.dedup();

        let unique_count = ranked.len();

        ranked.truncate(policy.max_candidates());

        (unique_count, ranked)
    }

    fn empty_result(
        status: DeviationReplanningStatus,
        input_candidate_count: usize,
    ) -> DeviationReplanningResult {
        DeviationReplanningResult {
            status,
            input_candidate_count,
            unique_candidate_count: 0,
            considered_candidate_count: 0,
            candidate_frontier_truncated: false,
            candidate_evaluation_count: 0,
            candidate_evaluation_truncated: false,
            rejected_goal_mismatch_count: 0,
            rejected_recovery_anchor_count: 0,
            rejected_step_bound_count: 0,
            rejected_threshold_count: 0,
            admitted_before_frontier: 0,
            selected: Vec::new(),
        }
    }

    fn adjusted_score(
        replacement: &ExecutableMultiStepIntention,
        deviation: &IntentionExecutionDeviation,
    ) -> CognitiveSignal {
        ExecutiveAgency::scaled_product(
            replacement.net_intention_score(),
            deviation.observation_confidence(),
        )
    }

    pub fn replan(
        prior_intention: &ExecutableMultiStepIntention,
        monitoring: &IntentionExecutionMonitoringResult,
        candidates: &[ExecutableMultiStepIntention],
        policy: DeviationReplanningPolicy,
    ) -> DeviationReplanningResult {
        let input_candidate_count = candidates.len();

        if monitoring.status() != IntentionExecutionStatus::Deviated {
            return Self::empty_result(
                DeviationReplanningStatus::NotTriggered,
                input_candidate_count,
            );
        }

        let Some(deviation) = monitoring.deviation() else {
            return Self::empty_result(
                DeviationReplanningStatus::NotTriggered,
                input_candidate_count,
            );
        };

        if deviation.observation_confidence().value()
            < policy.thresholds().minimum_observation_confidence().value()
        {
            return Self::empty_result(
                DeviationReplanningStatus::EvidenceInsufficient,
                input_candidate_count,
            );
        }

        let (unique_candidate_count, ranked_candidates) =
            Self::ranked_candidates(candidates, policy);

        let considered_candidate_count = ranked_candidates.len();

        let mut candidate_evaluation_count = 0_usize;

        let mut candidate_evaluation_truncated = false;

        let mut rejected_goal_mismatch_count = 0_usize;

        let mut rejected_recovery_anchor_count = 0_usize;

        let mut rejected_step_bound_count = 0_usize;

        let mut rejected_threshold_count = 0_usize;

        let mut admitted: Vec<GroundedDeviationReplan> = Vec::new();

        for replacement in ranked_candidates {
            if candidate_evaluation_count >= policy.max_candidate_evaluations() {
                candidate_evaluation_truncated = true;

                break;
            }

            candidate_evaluation_count = candidate_evaluation_count.saturating_add(1);

            if replacement.goal_identity() != prior_intention.goal_identity() {
                rejected_goal_mismatch_count = rejected_goal_mismatch_count.saturating_add(1);

                continue;
            }

            if replacement.step_count() > policy.max_steps_per_replacement() {
                rejected_step_bound_count = rejected_step_bound_count.saturating_add(1);

                continue;
            }

            if replacement.first_step().required_state() != deviation.observed_outcome() {
                rejected_recovery_anchor_count = rejected_recovery_anchor_count.saturating_add(1);

                continue;
            }

            let adjusted_replan_score = Self::adjusted_score(&replacement, deviation);

            let thresholds = policy.thresholds();

            if replacement.path_confidence().value()
                < thresholds.minimum_replacement_path_confidence().value()
                || adjusted_replan_score.value()
                    < thresholds.minimum_adjusted_replan_score().value()
            {
                rejected_threshold_count = rejected_threshold_count.saturating_add(1);

                continue;
            }

            admitted.push(GroundedDeviationReplan {
                replacement,
                deviation_step_index: deviation.step_index(),
                recovery_state: deviation.observed_outcome().clone(),
                deviation_observation_confidence: deviation.observation_confidence(),
                adjusted_replan_score,
            });
        }

        admitted.sort_by(Self::compare_replan);

        let admitted_before_frontier = admitted.len();

        admitted.truncate(policy.max_selected_replans());

        let status = if admitted.is_empty() {
            DeviationReplanningStatus::NoViableReplacement
        } else {
            DeviationReplanningStatus::ReplacementSelected
        };

        DeviationReplanningResult {
            status,
            input_candidate_count,
            unique_candidate_count,
            considered_candidate_count,
            candidate_frontier_truncated: unique_candidate_count > considered_candidate_count,
            candidate_evaluation_count,
            candidate_evaluation_truncated,
            rejected_goal_mismatch_count,
            rejected_recovery_anchor_count,
            rejected_step_bound_count,
            rejected_threshold_count,
            admitted_before_frontier,
            selected: admitted,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct UniversalDeviationReplanner;

impl UniversalDeviationReplanner {
    pub fn evaluate(
        prior_intention: &ExecutableMultiStepIntention,
        monitoring: &IntentionExecutionMonitoringResult,
        candidates: &[ExecutableMultiStepIntention],
        policy: DeviationReplanningPolicy,
    ) -> DeviationReplanningResult {
        DeviationReplanner::replan(prior_intention, monitoring, candidates, policy)
    }
}
