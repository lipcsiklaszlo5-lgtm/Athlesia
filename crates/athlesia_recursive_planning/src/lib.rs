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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecursivePlanningExecutionStatus {
    Running,
    GoalReached,
    Exhausted,
    InvalidState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursivePlanningExecution {
    plan: RecursivePlanningPlan,
    current_state: RecursivePlanningState,
    next_transition_index: usize,
    accumulated_cost: usize,
    status: RecursivePlanningExecutionStatus,
}

impl RecursivePlanningExecution {
    pub fn new(plan: RecursivePlanningPlan) -> Self {
        let status = if plan.goal().is_satisfied_by(plan.start()) {
            RecursivePlanningExecutionStatus::GoalReached
        } else if plan.transitions().is_empty() {
            RecursivePlanningExecutionStatus::Exhausted
        } else {
            RecursivePlanningExecutionStatus::Running
        };

        Self {
            current_state: plan.start().clone(),
            plan,
            next_transition_index: 0,
            accumulated_cost: 0,
            status,
        }
    }

    pub fn plan(&self) -> &RecursivePlanningPlan {
        &self.plan
    }

    pub fn current_state(&self) -> &RecursivePlanningState {
        &self.current_state
    }

    pub const fn next_transition_index(&self) -> usize {
        self.next_transition_index
    }

    pub const fn accumulated_cost(&self) -> usize {
        self.accumulated_cost
    }

    pub const fn status(&self) -> RecursivePlanningExecutionStatus {
        self.status
    }

    pub fn is_finished(&self) -> bool {
        self.status != RecursivePlanningExecutionStatus::Running
    }

    pub fn next_transition(&self) -> Option<&RecursivePlanningTransition> {
        if self.status != RecursivePlanningExecutionStatus::Running {
            return None;
        }

        self.plan.transitions().get(self.next_transition_index)
    }

    pub fn step(&mut self) -> RecursivePlanningExecutionStatus {
        if self.status != RecursivePlanningExecutionStatus::Running {
            return self.status;
        }

        let Some(transition) = self.plan.transitions().get(self.next_transition_index) else {
            self.status = if self.plan.goal().is_satisfied_by(&self.current_state) {
                RecursivePlanningExecutionStatus::GoalReached
            } else {
                RecursivePlanningExecutionStatus::Exhausted
            };

            return self.status;
        };

        if transition.from() != &self.current_state {
            self.status = RecursivePlanningExecutionStatus::InvalidState;

            return self.status;
        }

        self.current_state = transition.to().clone();

        self.accumulated_cost += transition.cost();

        self.next_transition_index += 1;

        if self.plan.goal().is_satisfied_by(&self.current_state) {
            self.status = RecursivePlanningExecutionStatus::GoalReached;
        } else if self.next_transition_index >= self.plan.transitions().len() {
            self.status = RecursivePlanningExecutionStatus::Exhausted;
        }

        self.status
    }

    pub fn run_to_completion(&mut self) -> RecursivePlanningExecutionStatus {
        while self.status == RecursivePlanningExecutionStatus::Running {
            self.step();
        }

        self.status
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecursiveReplanningOutcome {
    PlanPreserved,
    Replanned,
    GoalReached,
    Unreachable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveReplanningResult {
    outcome: RecursiveReplanningOutcome,
    observed_state: RecursivePlanningState,
    execution: Option<RecursivePlanningExecution>,
}

impl RecursiveReplanningResult {
    pub const fn outcome(&self) -> RecursiveReplanningOutcome {
        self.outcome
    }

    pub fn observed_state(&self) -> &RecursivePlanningState {
        &self.observed_state
    }

    pub fn execution(&self) -> Option<&RecursivePlanningExecution> {
        self.execution.as_ref()
    }

    pub fn into_execution(self) -> Option<RecursivePlanningExecution> {
        self.execution
    }

    pub fn has_plan(&self) -> bool {
        self.execution.is_some()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursivePlanningReplanner;

impl RecursivePlanningReplanner {
    pub const fn new() -> Self {
        Self
    }

    pub fn reconcile(
        &self,
        memory: &RecursivePlanningMemory,
        execution: &RecursivePlanningExecution,
        observed_state: RecursivePlanningState,
    ) -> RecursiveReplanningResult {
        let goal = execution.plan().goal();

        if goal.is_satisfied_by(&observed_state) {
            let plan = memory
                .find_plan(&observed_state, goal)
                .expect("already-satisfied goals always yield zero-step plans");

            return RecursiveReplanningResult {
                outcome: RecursiveReplanningOutcome::GoalReached,
                observed_state,
                execution: Some(RecursivePlanningExecution::new(plan)),
            };
        }

        if execution.status() == RecursivePlanningExecutionStatus::Running
            && execution.current_state() == &observed_state
        {
            return RecursiveReplanningResult {
                outcome: RecursiveReplanningOutcome::PlanPreserved,
                observed_state,
                execution: Some(execution.clone()),
            };
        }

        match memory.find_plan(&observed_state, goal) {
            Some(plan) => RecursiveReplanningResult {
                outcome: RecursiveReplanningOutcome::Replanned,
                observed_state,
                execution: Some(RecursivePlanningExecution::new(plan)),
            },
            None => RecursiveReplanningResult {
                outcome: RecursiveReplanningOutcome::Unreachable,
                observed_state,
                execution: None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecursivePlanningActiveOutcome {
    PreservedAndAdvanced,
    ReplannedAndAdvanced,
    GoalReached,
    Unreachable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursivePlanningActiveCycleResult {
    outcome: RecursivePlanningActiveOutcome,
    reconciliation: RecursiveReplanningOutcome,
    observed_state: RecursivePlanningState,
    execution: Option<RecursivePlanningExecution>,
}

impl RecursivePlanningActiveCycleResult {
    pub const fn outcome(&self) -> RecursivePlanningActiveOutcome {
        self.outcome
    }

    pub const fn reconciliation(&self) -> RecursiveReplanningOutcome {
        self.reconciliation
    }

    pub fn observed_state(&self) -> &RecursivePlanningState {
        &self.observed_state
    }

    pub fn execution(&self) -> Option<&RecursivePlanningExecution> {
        self.execution.as_ref()
    }

    pub fn into_execution(self) -> Option<RecursivePlanningExecution> {
        self.execution
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursivePlanningActiveCycle;

impl RecursivePlanningActiveCycle {
    pub const fn new() -> Self {
        Self
    }

    pub fn tick(
        &self,
        memory: &RecursivePlanningMemory,
        execution: &RecursivePlanningExecution,
        observed_state: RecursivePlanningState,
    ) -> RecursivePlanningActiveCycleResult {
        let reconciliation =
            RecursivePlanningReplanner::new().reconcile(memory, execution, observed_state.clone());

        let reconciliation_outcome = reconciliation.outcome();

        match reconciliation_outcome {
            RecursiveReplanningOutcome::Unreachable => RecursivePlanningActiveCycleResult {
                outcome: RecursivePlanningActiveOutcome::Unreachable,
                reconciliation: reconciliation_outcome,
                observed_state,
                execution: None,
            },

            RecursiveReplanningOutcome::GoalReached => RecursivePlanningActiveCycleResult {
                outcome: RecursivePlanningActiveOutcome::GoalReached,
                reconciliation: reconciliation_outcome,
                observed_state,
                execution: reconciliation.into_execution(),
            },

            RecursiveReplanningOutcome::PlanPreserved | RecursiveReplanningOutcome::Replanned => {
                let mut next_execution = reconciliation
                    .into_execution()
                    .expect("preserved or replanned outcomes carry execution");

                let status = next_execution.step();

                let outcome = if status == RecursivePlanningExecutionStatus::GoalReached {
                    RecursivePlanningActiveOutcome::GoalReached
                } else if reconciliation_outcome == RecursiveReplanningOutcome::PlanPreserved {
                    RecursivePlanningActiveOutcome::PreservedAndAdvanced
                } else {
                    RecursivePlanningActiveOutcome::ReplannedAndAdvanced
                };

                RecursivePlanningActiveCycleResult {
                    outcome,
                    reconciliation: reconciliation_outcome,
                    observed_state,
                    execution: Some(next_execution),
                }
            }
        }
    }
}
