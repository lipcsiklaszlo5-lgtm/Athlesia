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
