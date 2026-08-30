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
