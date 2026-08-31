use std::collections::BTreeMap;

use athlesia_recursive_planning::RecursivePlanningTransition;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveCounterfactualCandidate {
    transition: RecursivePlanningTransition,
    interaction_cost: usize,
}

impl RecursiveCounterfactualCandidate {
    pub fn new(transition: RecursivePlanningTransition, interaction_cost: usize) -> Option<Self> {
        if interaction_cost == 0 {
            return None;
        }

        Some(Self {
            transition,
            interaction_cost,
        })
    }

    pub fn transition(&self) -> &RecursivePlanningTransition {
        &self.transition
    }

    pub const fn interaction_cost(&self) -> usize {
        self.interaction_cost
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveCounterfactualSet {
    candidates: Vec<RecursiveCounterfactualCandidate>,
}

impl RecursiveCounterfactualSet {
    pub fn new(candidates: Vec<RecursiveCounterfactualCandidate>) -> Self {
        let mut cheapest_by_transition = BTreeMap::<RecursivePlanningTransition, usize>::new();

        for candidate in candidates {
            cheapest_by_transition
                .entry(candidate.transition().clone())
                .and_modify(|cost| {
                    *cost = (*cost).min(candidate.interaction_cost());
                })
                .or_insert(candidate.interaction_cost());
        }

        let candidates = cheapest_by_transition
            .into_iter()
            .map(
                |(transition, interaction_cost)| RecursiveCounterfactualCandidate {
                    transition,
                    interaction_cost,
                },
            )
            .collect();

        Self { candidates }
    }

    pub fn candidates(&self) -> &[RecursiveCounterfactualCandidate] {
        &self.candidates
    }

    pub fn len(&self) -> usize {
        self.candidates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn contains_transition(&self, transition: &RecursivePlanningTransition) -> bool {
        self.candidates
            .binary_search_by(|candidate| candidate.transition().cmp(transition))
            .is_ok()
    }
}

use athlesia_recursive_planning::RecursivePlanningState;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursiveCounterfactualOutcome {
    state: RecursivePlanningState,
}

impl RecursiveCounterfactualOutcome {
    pub fn new(state: RecursivePlanningState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &RecursivePlanningState {
        &self.state
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveCounterfactualProjection {
    candidate: RecursiveCounterfactualCandidate,
    outcomes: Vec<RecursiveCounterfactualOutcome>,
}

impl RecursiveCounterfactualProjection {
    pub fn new(
        candidate: RecursiveCounterfactualCandidate,
        outcomes: Vec<RecursiveCounterfactualOutcome>,
    ) -> Option<Self> {
        if outcomes.is_empty() {
            return None;
        }

        let mut outcomes = outcomes;
        outcomes.sort();
        outcomes.dedup();

        Some(Self {
            candidate,
            outcomes,
        })
    }

    pub fn candidate(&self) -> &RecursiveCounterfactualCandidate {
        &self.candidate
    }

    pub fn outcomes(&self) -> &[RecursiveCounterfactualOutcome] {
        &self.outcomes
    }

    pub fn outcome_count(&self) -> usize {
        self.outcomes.len()
    }

    pub fn is_deterministic(&self) -> bool {
        self.outcomes.len() == 1
    }

    pub fn is_branching(&self) -> bool {
        self.outcomes.len() > 1
    }

    pub fn contains_state(&self, state: &RecursivePlanningState) -> bool {
        self.outcomes
            .binary_search_by(|outcome| outcome.state().cmp(state))
            .is_ok()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveCounterfactualProjectionSet {
    projections: Vec<RecursiveCounterfactualProjection>,
}

impl RecursiveCounterfactualProjectionSet {
    pub fn new(mut projections: Vec<RecursiveCounterfactualProjection>) -> Self {
        projections.sort_by(|left, right| {
            left.candidate()
                .cmp(right.candidate())
                .then_with(|| left.outcomes().cmp(right.outcomes()))
        });

        projections.dedup();

        Self { projections }
    }

    pub fn projections(&self) -> &[RecursiveCounterfactualProjection] {
        &self.projections
    }

    pub fn len(&self) -> usize {
        self.projections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.projections.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveCounterfactualInformationValue {
    projection: RecursiveCounterfactualProjection,
    discrimination_capacity: usize,
}

impl RecursiveCounterfactualInformationValue {
    pub fn evaluate(projection: RecursiveCounterfactualProjection) -> Self {
        let outcome_count = projection.outcome_count();

        let discrimination_capacity =
            outcome_count.saturating_mul(outcome_count.saturating_sub(1)) / 2;

        Self {
            projection,
            discrimination_capacity,
        }
    }

    pub fn projection(&self) -> &RecursiveCounterfactualProjection {
        &self.projection
    }

    pub const fn discrimination_capacity(&self) -> usize {
        self.discrimination_capacity
    }

    pub fn interaction_cost(&self) -> usize {
        self.projection.candidate().interaction_cost()
    }

    pub fn is_informative(&self) -> bool {
        self.discrimination_capacity > 0
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursiveCounterfactualInformationRanking {
    values: Vec<RecursiveCounterfactualInformationValue>,
}

impl RecursiveCounterfactualInformationRanking {
    pub fn new(mut values: Vec<RecursiveCounterfactualInformationValue>) -> Self {
        values.sort_by(|left, right| {
            let left_capacity = left.discrimination_capacity();

            let right_capacity = right.discrimination_capacity();

            let left_cost = left.interaction_cost();

            let right_cost = right.interaction_cost();

            let efficiency_order = right_capacity
                .saturating_mul(left_cost)
                .cmp(&left_capacity.saturating_mul(right_cost));

            efficiency_order
                .then_with(|| right_capacity.cmp(&left_capacity))
                .then_with(|| left_cost.cmp(&right_cost))
                .then_with(|| {
                    left.projection()
                        .candidate()
                        .cmp(right.projection().candidate())
                })
                .then_with(|| {
                    left.projection()
                        .outcomes()
                        .cmp(right.projection().outcomes())
                })
        });

        Self { values }
    }

    pub fn values(&self) -> &[RecursiveCounterfactualInformationValue] {
        &self.values
    }

    pub fn best(&self) -> Option<&RecursiveCounterfactualInformationValue> {
        self.values.first()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecursiveCounterfactualBudget {
    max_interaction_cost: usize,
    max_outcomes: usize,
    max_discrimination_capacity: usize,
}

impl RecursiveCounterfactualBudget {
    pub fn new(
        max_interaction_cost: usize,
        max_outcomes: usize,
        max_discrimination_capacity: usize,
    ) -> Option<Self> {
        if max_interaction_cost == 0 || max_outcomes == 0 {
            return None;
        }

        Some(Self {
            max_interaction_cost,
            max_outcomes,
            max_discrimination_capacity,
        })
    }

    pub const fn max_interaction_cost(&self) -> usize {
        self.max_interaction_cost
    }

    pub const fn max_outcomes(&self) -> usize {
        self.max_outcomes
    }

    pub const fn max_discrimination_capacity(&self) -> usize {
        self.max_discrimination_capacity
    }

    pub fn allows(&self, value: &RecursiveCounterfactualInformationValue) -> bool {
        value.interaction_cost() <= self.max_interaction_cost
            && value.projection().outcome_count() <= self.max_outcomes
            && value.discrimination_capacity() <= self.max_discrimination_capacity
    }

    pub fn apply(
        &self,
        ranking: &RecursiveCounterfactualInformationRanking,
    ) -> RecursiveCounterfactualBudgetedRanking {
        let values = ranking
            .values()
            .iter()
            .filter(|value| self.allows(value))
            .cloned()
            .collect();

        RecursiveCounterfactualBudgetedRanking {
            budget: *self,
            values,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveCounterfactualBudgetedRanking {
    budget: RecursiveCounterfactualBudget,
    values: Vec<RecursiveCounterfactualInformationValue>,
}

impl RecursiveCounterfactualBudgetedRanking {
    pub const fn budget(&self) -> RecursiveCounterfactualBudget {
        self.budget
    }

    pub fn values(&self) -> &[RecursiveCounterfactualInformationValue] {
        &self.values
    }

    pub fn best(&self) -> Option<&RecursiveCounterfactualInformationValue> {
        self.values.first()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecursiveCounterfactualSelectionPolicy {
    beam_width: usize,
}

impl RecursiveCounterfactualSelectionPolicy {
    pub fn new(beam_width: usize) -> Option<Self> {
        if beam_width == 0 {
            return None;
        }

        Some(Self { beam_width })
    }

    pub const fn beam_width(&self) -> usize {
        self.beam_width
    }

    pub fn select(
        &self,
        ranking: &RecursiveCounterfactualBudgetedRanking,
    ) -> RecursiveCounterfactualSelection {
        let selected = ranking
            .values()
            .iter()
            .take(self.beam_width)
            .cloned()
            .collect();

        RecursiveCounterfactualSelection {
            policy: *self,
            budget: ranking.budget(),
            selected,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveCounterfactualSelection {
    policy: RecursiveCounterfactualSelectionPolicy,
    budget: RecursiveCounterfactualBudget,
    selected: Vec<RecursiveCounterfactualInformationValue>,
}

impl RecursiveCounterfactualSelection {
    pub const fn policy(&self) -> RecursiveCounterfactualSelectionPolicy {
        self.policy
    }

    pub const fn budget(&self) -> RecursiveCounterfactualBudget {
        self.budget
    }

    pub fn selected(&self) -> &[RecursiveCounterfactualInformationValue] {
        &self.selected
    }

    pub fn best(&self) -> Option<&RecursiveCounterfactualInformationValue> {
        self.selected.first()
    }

    pub fn len(&self) -> usize {
        self.selected.len()
    }

    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.selected.len() == self.policy.beam_width()
    }
}
