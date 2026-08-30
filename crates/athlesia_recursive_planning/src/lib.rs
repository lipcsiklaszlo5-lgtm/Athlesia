use std::collections::BTreeSet;

use athlesia_recursive::RecursiveUnit;

#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursivePlanningState {
    units: Vec<RecursiveUnit>,
}

impl RecursivePlanningState {
    pub fn new(units: Vec<RecursiveUnit>) -> Self {
        let units = units
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        Self { units }
    }

    pub fn units(&self) -> &[RecursiveUnit] {
        &self.units
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    pub fn contains(&self, unit: &RecursiveUnit) -> bool {
        self.units.binary_search(unit).is_ok()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursivePlanningGoal {
    required_units: Vec<RecursiveUnit>,
}

impl RecursivePlanningGoal {
    pub fn new(required_units: Vec<RecursiveUnit>) -> Option<Self> {
        let required_units = required_units
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        if required_units.is_empty() {
            return None;
        }

        Some(Self { required_units })
    }

    pub fn required_units(&self) -> &[RecursiveUnit] {
        &self.required_units
    }

    pub fn len(&self) -> usize {
        self.required_units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.required_units.is_empty()
    }

    pub fn is_satisfied_by(&self, state: &RecursivePlanningState) -> bool {
        self.required_units.iter().all(|unit| state.contains(unit))
    }

    pub fn missing_units(&self, state: &RecursivePlanningState) -> Vec<RecursiveUnit> {
        self.required_units
            .iter()
            .filter(|unit| !state.contains(unit))
            .cloned()
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursivePlanningTransition {
    from: RecursivePlanningState,
    to: RecursivePlanningState,
    added: Vec<RecursiveUnit>,
    removed: Vec<RecursiveUnit>,
}

impl RecursivePlanningTransition {
    pub fn new(from: RecursivePlanningState, to: RecursivePlanningState) -> Option<Self> {
        if from == to {
            return None;
        }

        let added = to
            .units()
            .iter()
            .filter(|unit| !from.contains(unit))
            .cloned()
            .collect::<Vec<_>>();

        let removed = from
            .units()
            .iter()
            .filter(|unit| !to.contains(unit))
            .cloned()
            .collect::<Vec<_>>();

        Some(Self {
            from,
            to,
            added,
            removed,
        })
    }

    pub fn from(&self) -> &RecursivePlanningState {
        &self.from
    }

    pub fn to(&self) -> &RecursivePlanningState {
        &self.to
    }

    pub fn added(&self) -> &[RecursiveUnit] {
        &self.added
    }

    pub fn removed(&self) -> &[RecursiveUnit] {
        &self.removed
    }

    pub fn cost(&self) -> usize {
        self.added.len() + self.removed.len()
    }

    pub fn is_pure_addition(&self) -> bool {
        !self.added.is_empty() && self.removed.is_empty()
    }

    pub fn is_pure_removal(&self) -> bool {
        self.added.is_empty() && !self.removed.is_empty()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecursivePlanningMemory {
    transitions: BTreeSet<RecursivePlanningTransition>,
}

impl RecursivePlanningMemory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.transitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transitions.is_empty()
    }

    pub fn insert(&mut self, transition: RecursivePlanningTransition) -> bool {
        self.transitions.insert(transition)
    }

    pub fn contains(&self, transition: &RecursivePlanningTransition) -> bool {
        self.transitions.contains(transition)
    }

    pub fn transitions(&self) -> impl Iterator<Item = &RecursivePlanningTransition> {
        self.transitions.iter()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RecursivePlanningSuccessor {
    state: RecursivePlanningState,
    transition: RecursivePlanningTransition,
}

impl RecursivePlanningSuccessor {
    pub fn state(&self) -> &RecursivePlanningState {
        &self.state
    }

    pub fn transition(&self) -> &RecursivePlanningTransition {
        &self.transition
    }

    pub fn cost(&self) -> usize {
        self.transition.cost()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursivePlanningSuccessorGenerator;

impl RecursivePlanningSuccessorGenerator {
    pub const fn new() -> Self {
        Self
    }

    pub fn generate(
        &self,
        memory: &RecursivePlanningMemory,
        state: &RecursivePlanningState,
    ) -> Vec<RecursivePlanningSuccessor> {
        let mut successors = memory
            .transitions()
            .filter(|transition| transition.from() == state)
            .map(|transition| RecursivePlanningSuccessor {
                state: transition.to().clone(),
                transition: transition.clone(),
            })
            .collect::<Vec<_>>();

        successors.sort_by(|left, right| {
            left.cost()
                .cmp(&right.cost())
                .then_with(|| left.state().cmp(right.state()))
                .then_with(|| left.transition().cmp(right.transition()))
        });

        successors
    }
}

impl RecursivePlanningMemory {
    pub fn successors(&self, state: &RecursivePlanningState) -> Vec<RecursivePlanningSuccessor> {
        RecursivePlanningSuccessorGenerator::new().generate(self, state)
    }
}

use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursivePlanningPlan {
    start: RecursivePlanningState,
    goal: RecursivePlanningGoal,
    transitions: Vec<RecursivePlanningTransition>,
    final_state: RecursivePlanningState,
    total_cost: usize,
}

impl RecursivePlanningPlan {
    pub fn start(&self) -> &RecursivePlanningState {
        &self.start
    }

    pub fn goal(&self) -> &RecursivePlanningGoal {
        &self.goal
    }

    pub fn transitions(&self) -> &[RecursivePlanningTransition] {
        &self.transitions
    }

    pub fn final_state(&self) -> &RecursivePlanningState {
        &self.final_state
    }

    pub fn total_cost(&self) -> usize {
        self.total_cost
    }

    pub fn len(&self) -> usize {
        self.transitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.transitions.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursivePlanningSearch;

impl RecursivePlanningSearch {
    pub const fn new() -> Self {
        Self
    }

    pub fn find_plan(
        &self,
        memory: &RecursivePlanningMemory,
        start: &RecursivePlanningState,
        goal: &RecursivePlanningGoal,
    ) -> Option<RecursivePlanningPlan> {
        if goal.is_satisfied_by(start) {
            return Some(RecursivePlanningPlan {
                start: start.clone(),
                goal: goal.clone(),
                transitions: Vec::new(),
                final_state: start.clone(),
                total_cost: 0,
            });
        }

        let mut frontier = BTreeSet::new();

        let mut best_cost = BTreeMap::new();

        frontier.insert((
            0usize,
            start.clone(),
            Vec::<RecursivePlanningTransition>::new(),
        ));

        best_cost.insert(start.clone(), 0usize);

        while let Some((cost, state, path)) = frontier.iter().next().cloned() {
            frontier.remove(&(cost, state.clone(), path.clone()));

            if best_cost.get(&state).is_some_and(|known| cost > *known) {
                continue;
            }

            if goal.is_satisfied_by(&state) {
                return Some(RecursivePlanningPlan {
                    start: start.clone(),
                    goal: goal.clone(),
                    transitions: path,
                    final_state: state,
                    total_cost: cost,
                });
            }

            for successor in memory.successors(&state) {
                let next_state = successor.state().clone();

                let next_cost = cost + successor.cost();

                let should_visit = best_cost
                    .get(&next_state)
                    .is_none_or(|known| next_cost < *known);

                if !should_visit {
                    continue;
                }

                best_cost.insert(next_state.clone(), next_cost);

                let mut next_path = path.clone();

                next_path.push(successor.transition().clone());

                frontier.insert((next_cost, next_state, next_path));
            }
        }

        None
    }
}

impl RecursivePlanningMemory {
    pub fn find_plan(
        &self,
        start: &RecursivePlanningState,
        goal: &RecursivePlanningGoal,
    ) -> Option<RecursivePlanningPlan> {
        RecursivePlanningSearch::new().find_plan(self, start, goal)
    }
}
