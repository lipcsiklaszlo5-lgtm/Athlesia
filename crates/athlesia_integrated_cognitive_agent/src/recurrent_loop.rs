use super::{
    BoundedRecurrentAgentLoopPolicy, CognitiveCycleStateTransitionRequest, CognitiveStructure,
    IntegratedAgentPolicy, OnlineCognitiveOrchestration, OnlineCognitiveOrchestrationInput,
    OnlineCognitiveOrchestrationResult, RecurrentFeedbackEvidence,
};

// ============================================================================
// M51 — ONLINE RECURRENT COGNITIVE CONTROL LOOP
// ============================================================================

pub struct OnlineRecurrentCognitiveStepInput<'a> {
    runtime: OnlineCognitiveOrchestrationInput<'a>,
    transition_request: &'a CognitiveCycleStateTransitionRequest,
    feedback: Option<RecurrentFeedbackEvidence>,
}

impl<'a> OnlineRecurrentCognitiveStepInput<'a> {
    pub fn new(
        runtime: OnlineCognitiveOrchestrationInput<'a>,
        transition_request: &'a CognitiveCycleStateTransitionRequest,
        feedback: Option<RecurrentFeedbackEvidence>,
    ) -> Self {
        Self {
            runtime,
            transition_request,
            feedback,
        }
    }

    pub fn transition_request(&self) -> &CognitiveCycleStateTransitionRequest {
        self.transition_request
    }

    pub fn feedback(&self) -> Option<&RecurrentFeedbackEvidence> {
        self.feedback.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OnlineRecurrentCognitiveLoopStatus {
    Completed,
    StepFrontierExceeded,
    UnexpectedFirstStepFeedback,
    MissingPredecessorFeedback,
    FeedbackStepIndexMismatch,
    FeedbackAnchorMismatch,
    FeedbackAuthorityProvenanceMismatch,
    OnlineStepRejected,
}

#[derive(Debug, PartialEq)]
pub struct OnlineRecurrentCognitiveLoopResult {
    status: OnlineRecurrentCognitiveLoopStatus,
    initial_anchor_state: CognitiveStructure,
    final_anchor_state: CognitiveStructure,
    steps: Vec<OnlineCognitiveOrchestrationResult>,
    rejected_step_index: Option<usize>,
}

impl OnlineRecurrentCognitiveLoopResult {
    fn rejected_before_execution(
        status: OnlineRecurrentCognitiveLoopStatus,
        initial_anchor_state: &CognitiveStructure,
        current_anchor_state: &CognitiveStructure,
        steps: Vec<OnlineCognitiveOrchestrationResult>,
        rejected_step_index: usize,
    ) -> Self {
        Self {
            status,
            initial_anchor_state: initial_anchor_state.clone(),
            final_anchor_state: current_anchor_state.clone(),
            steps,
            rejected_step_index: Some(rejected_step_index),
        }
    }

    pub fn status(&self) -> OnlineRecurrentCognitiveLoopStatus {
        self.status
    }

    pub fn initial_anchor_state(&self) -> &CognitiveStructure {
        &self.initial_anchor_state
    }

    pub fn final_anchor_state(&self) -> &CognitiveStructure {
        &self.final_anchor_state
    }

    pub fn steps(&self) -> &[OnlineCognitiveOrchestrationResult] {
        &self.steps
    }

    pub fn executed_step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn completed_step_count(&self) -> usize {
        if self.status == OnlineRecurrentCognitiveLoopStatus::OnlineStepRejected {
            self.steps.len().saturating_sub(1)
        } else {
            self.steps.len()
        }
    }

    pub fn rejected_step_index(&self) -> Option<usize> {
        self.rejected_step_index
    }

    pub fn completed(&self) -> bool {
        self.status == OnlineRecurrentCognitiveLoopStatus::Completed
    }

    pub fn rejected(&self) -> bool {
        !self.completed()
    }
}

pub struct OnlineRecurrentCognitiveLoop;

impl OnlineRecurrentCognitiveLoop {
    pub fn run<'a>(
        initial_anchor_state: &CognitiveStructure,
        step_inputs: Vec<OnlineRecurrentCognitiveStepInput<'a>>,
        cycle_policy: IntegratedAgentPolicy,
        loop_policy: BoundedRecurrentAgentLoopPolicy,
    ) -> OnlineRecurrentCognitiveLoopResult {
        if step_inputs.len() > loop_policy.max_steps() {
            return OnlineRecurrentCognitiveLoopResult {
                status: OnlineRecurrentCognitiveLoopStatus::StepFrontierExceeded,
                initial_anchor_state: initial_anchor_state.clone(),
                final_anchor_state: initial_anchor_state.clone(),
                steps: Vec::new(),
                rejected_step_index: Some(loop_policy.max_steps()),
            };
        }

        let mut current_anchor_state = initial_anchor_state.clone();

        let mut completed_steps: Vec<OnlineCognitiveOrchestrationResult> =
            Vec::with_capacity(step_inputs.len());

        let mut predecessor_anchor: Option<CognitiveStructure> = None;

        let mut predecessor_authority_provenance: Option<CognitiveStructure> = None;

        for (step_index, step_input) in step_inputs.into_iter().enumerate() {
            if step_index == 0 {
                if step_input.feedback.is_some() {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::UnexpectedFirstStepFeedback,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                }
            } else {
                let Some(feedback) = step_input.feedback.as_ref() else {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::MissingPredecessorFeedback,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                };

                if feedback.predecessor_step_index() != step_index - 1 {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::FeedbackStepIndexMismatch,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                }

                if feedback.predecessor_next_anchor_state()
                    != predecessor_anchor
                        .as_ref()
                        .expect("accepted predecessor must retain anchor")
                {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::FeedbackAnchorMismatch,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                }

                if feedback.predecessor_authority_provenance()
                    != predecessor_authority_provenance.as_ref()
                {
                    return OnlineRecurrentCognitiveLoopResult::rejected_before_execution(
                        OnlineRecurrentCognitiveLoopStatus::FeedbackAuthorityProvenanceMismatch,
                        initial_anchor_state,
                        &current_anchor_state,
                        completed_steps,
                        step_index,
                    );
                }
            }

            let authority_provenance = step_input
                .transition_request
                .expected_authority_provenance()
                .cloned();

            let result = OnlineCognitiveOrchestration::run(
                &current_anchor_state,
                step_input.runtime,
                cycle_policy,
                step_input.transition_request,
            );

            if result.rejected() {
                completed_steps.push(result);

                return OnlineRecurrentCognitiveLoopResult {
                    status: OnlineRecurrentCognitiveLoopStatus::OnlineStepRejected,
                    initial_anchor_state: initial_anchor_state.clone(),
                    final_anchor_state: current_anchor_state,
                    steps: completed_steps,
                    rejected_step_index: Some(step_index),
                };
            }

            let Some(next_anchor_state) = result.next_anchor_state().cloned() else {
                completed_steps.push(result);

                return OnlineRecurrentCognitiveLoopResult {
                    status: OnlineRecurrentCognitiveLoopStatus::OnlineStepRejected,
                    initial_anchor_state: initial_anchor_state.clone(),
                    final_anchor_state: current_anchor_state,
                    steps: completed_steps,
                    rejected_step_index: Some(step_index),
                };
            };

            predecessor_anchor = Some(next_anchor_state.clone());

            predecessor_authority_provenance = authority_provenance;

            current_anchor_state = next_anchor_state;

            completed_steps.push(result);
        }

        OnlineRecurrentCognitiveLoopResult {
            status: OnlineRecurrentCognitiveLoopStatus::Completed,
            initial_anchor_state: initial_anchor_state.clone(),
            final_anchor_state: current_anchor_state,
            steps: completed_steps,
            rejected_step_index: None,
        }
    }
}

pub struct UniversalOnlineRecurrentCognitiveLoop;

impl UniversalOnlineRecurrentCognitiveLoop {
    pub fn evaluate<'a>(
        initial_anchor_state: &CognitiveStructure,
        step_inputs: Vec<OnlineRecurrentCognitiveStepInput<'a>>,
        cycle_policy: IntegratedAgentPolicy,
        loop_policy: BoundedRecurrentAgentLoopPolicy,
    ) -> OnlineRecurrentCognitiveLoopResult {
        OnlineRecurrentCognitiveLoop::run(
            initial_anchor_state,
            step_inputs,
            cycle_policy,
            loop_policy,
        )
    }
}
