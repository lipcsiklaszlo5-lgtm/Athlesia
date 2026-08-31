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
