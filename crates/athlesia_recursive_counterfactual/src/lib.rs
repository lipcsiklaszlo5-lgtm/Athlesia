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
