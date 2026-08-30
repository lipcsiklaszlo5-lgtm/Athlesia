use athlesia_recursive_planning::{
    RecursivePlanningExecution, RecursivePlanningGoal, RecursivePlanningMemory,
    RecursivePlanningPlan, RecursivePlanningState,
};

use athlesia_recursive_revision::RecursiveCompetingModels;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveControlRequest {
    models: RecursiveCompetingModels,
    start: RecursivePlanningState,
    goal: RecursivePlanningGoal,
}

impl RecursiveControlRequest {
    pub fn new(
        models: RecursiveCompetingModels,
        start: RecursivePlanningState,
        goal: RecursivePlanningGoal,
    ) -> Self {
        Self {
            models,
            start,
            goal,
        }
    }

    pub fn models(&self) -> &RecursiveCompetingModels {
        &self.models
    }

    pub fn start(&self) -> &RecursivePlanningState {
        &self.start
    }

    pub fn goal(&self) -> &RecursivePlanningGoal {
        &self.goal
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveControlDecision {
    request: RecursiveControlRequest,
    plan: RecursivePlanningPlan,
    execution: RecursivePlanningExecution,
}

impl RecursiveControlDecision {
    pub fn request(&self) -> &RecursiveControlRequest {
        &self.request
    }

    pub fn plan(&self) -> &RecursivePlanningPlan {
        &self.plan
    }

    pub fn execution(&self) -> &RecursivePlanningExecution {
        &self.execution
    }

    pub fn into_execution(self) -> RecursivePlanningExecution {
        self.execution
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveControlPlanner;

impl RecursiveControlPlanner {
    pub const fn new() -> Self {
        Self
    }

    pub fn prepare(
        &self,
        planning_memory: &RecursivePlanningMemory,
        request: RecursiveControlRequest,
    ) -> Option<RecursiveControlDecision> {
        let plan = planning_memory.find_plan(request.start(), request.goal())?;

        let execution = RecursivePlanningExecution::new(plan.clone());

        Some(RecursiveControlDecision {
            request,
            plan,
            execution,
        })
    }
}

use athlesia_recursive::RecursiveConcept;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveControlObjective {
    model: RecursiveConcept,
    goal: RecursivePlanningGoal,
}

impl RecursiveControlObjective {
    pub fn new(model: RecursiveConcept, goal: RecursivePlanningGoal) -> Self {
        Self { model, goal }
    }

    pub fn model(&self) -> &RecursiveConcept {
        &self.model
    }

    pub fn goal(&self) -> &RecursivePlanningGoal {
        &self.goal
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecursiveControlPolicyDecision {
    objective: RecursiveControlObjective,
    control: RecursiveControlDecision,
}

impl RecursiveControlPolicyDecision {
    pub fn objective(&self) -> &RecursiveControlObjective {
        &self.objective
    }

    pub fn control(&self) -> &RecursiveControlDecision {
        &self.control
    }

    pub fn execution(&self) -> &RecursivePlanningExecution {
        self.control.execution()
    }

    pub fn into_execution(self) -> RecursivePlanningExecution {
        self.control.into_execution()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RecursiveControlModelPolicy;

impl RecursiveControlModelPolicy {
    pub const fn new() -> Self {
        Self
    }

    pub fn select_objective(
        &self,
        models: &RecursiveCompetingModels,
        objectives: &[RecursiveControlObjective],
    ) -> Option<RecursiveControlObjective> {
        let best = models.best()?;

        objectives
            .iter()
            .filter(|objective| objective.model() == best.concept())
            .min_by(|left, right| left.goal().cmp(right.goal()))
            .cloned()
    }

    pub fn prepare(
        &self,
        planning_memory: &RecursivePlanningMemory,
        models: &RecursiveCompetingModels,
        start: &RecursivePlanningState,
        objectives: &[RecursiveControlObjective],
    ) -> Option<RecursiveControlPolicyDecision> {
        let objective = self.select_objective(models, objectives)?;

        let request =
            RecursiveControlRequest::new(models.clone(), start.clone(), objective.goal().clone());

        let control = RecursiveControlPlanner::new().prepare(planning_memory, request)?;

        Some(RecursiveControlPolicyDecision { objective, control })
    }
}
