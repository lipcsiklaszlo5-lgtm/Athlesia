use crate::cognitive_interaction_runtime::{
    ArcAgi3CognitiveInteractionCompletion, ArcAgi3CognitiveInteractionError,
    ArcAgi3CognitiveInteractionRuntime, ArcAgi3CognitiveInteractionStep,
    ArcAgi3ExperimentDispatchAuthority, ArcAgi3UnifiedExecutiveAuthority,
};
use crate::environment_transport_boundary::{
    ArcAgi3EnvironmentTransport, ArcAgi3EnvironmentTransportBoundary, ArcAgi3TransportError,
    ArcAgi3TransportFailureDisposition,
};
use crate::interactive_session_runtime::{ArcAgi3InteractiveSessionError, ArcAgi3SessionCommand};
use crate::{ArcAgi3Action, ArcAgi3GameId, ArcAgi3GameState};
use athlesia_integrated_cognitive_agent::{
    CognitiveCycleStateTransitionRequest, IntegratedAgentPolicy, OnlineCognitiveOrchestrationInput,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3LiveEnvironmentStatus {
    NotStarted,
    Active,
    Won,
    GameOver,
    FaultedPending,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ArcAgi3LiveEnvironmentError {
    GameNotActive,
    FaultedPending(Option<ArcAgi3TransportFailureDisposition>),
    Cognitive(ArcAgi3CognitiveInteractionError),
    Session(ArcAgi3InteractiveSessionError),
    Transport(ArcAgi3TransportError),
    CognitiveStepCounterOverflow,
    ResetCounterOverflow,
}

impl From<ArcAgi3CognitiveInteractionError> for ArcAgi3LiveEnvironmentError {
    fn from(error: ArcAgi3CognitiveInteractionError) -> Self {
        Self::Cognitive(error)
    }
}

impl From<ArcAgi3InteractiveSessionError> for ArcAgi3LiveEnvironmentError {
    fn from(error: ArcAgi3InteractiveSessionError) -> Self {
        Self::Session(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3LiveCognitiveStep {
    cognitive_step: ArcAgi3CognitiveInteractionStep,
    completion: ArcAgi3CognitiveInteractionCompletion,
    completed_cognitive_step_count: u64,
}

impl ArcAgi3LiveCognitiveStep {
    pub fn cognitive_step(&self) -> &ArcAgi3CognitiveInteractionStep {
        &self.cognitive_step
    }

    pub fn completion(&self) -> &ArcAgi3CognitiveInteractionCompletion {
        &self.completion
    }

    pub fn completed_cognitive_step_count(&self) -> u64 {
        self.completed_cognitive_step_count
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ArcAgi3LiveUnifiedActionRequest<'a> {
    exploitation_actions: &'a [ArcAgi3Action],
    goal: &'a athlesia_executive_agency::ExecutiveGoal,
    goal_alignment: CognitiveSignal,
    exploitation_execution_cost: CognitiveSignal,
    experiment_authority: Option<ArcAgi3ExperimentDispatchAuthority<'a>>,
    policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
    confidence: CognitiveSignal,
}

impl<'a> ArcAgi3LiveUnifiedActionRequest<'a> {
    pub fn new(
        exploitation_actions: &'a [ArcAgi3Action],
        goal: &'a athlesia_executive_agency::ExecutiveGoal,
        goal_alignment: CognitiveSignal,
        exploitation_execution_cost: CognitiveSignal,
        experiment_authority: Option<ArcAgi3ExperimentDispatchAuthority<'a>>,
        policy: athlesia_executive_agency::ExecutiveAgencyPolicy,
        confidence: CognitiveSignal,
    ) -> Self {
        Self {
            exploitation_actions,
            goal,
            goal_alignment,
            exploitation_execution_cost,
            experiment_authority,
            policy,
            confidence,
        }
    }

    pub fn exploitation_actions(self) -> &'a [ArcAgi3Action] {
        self.exploitation_actions
    }

    pub fn goal(self) -> &'a athlesia_executive_agency::ExecutiveGoal {
        self.goal
    }

    pub fn goal_alignment(self) -> CognitiveSignal {
        self.goal_alignment
    }

    pub fn exploitation_execution_cost(self) -> CognitiveSignal {
        self.exploitation_execution_cost
    }

    pub fn experiment_authority(self) -> Option<ArcAgi3ExperimentDispatchAuthority<'a>> {
        self.experiment_authority
    }

    pub fn policy(self) -> athlesia_executive_agency::ExecutiveAgencyPolicy {
        self.policy
    }

    pub fn confidence(self) -> CognitiveSignal {
        self.confidence
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3LiveUnifiedStep {
    authority: ArcAgi3UnifiedExecutiveAuthority,
    command: ArcAgi3SessionCommand,
    completion: ArcAgi3CognitiveInteractionCompletion,
    completed_cognitive_step_count: u64,
}

impl ArcAgi3LiveUnifiedStep {
    pub fn authority(&self) -> &ArcAgi3UnifiedExecutiveAuthority {
        &self.authority
    }

    pub fn action(&self) -> ArcAgi3Action {
        self.authority.action()
    }

    pub fn source_state(&self) -> &CognitiveStructure {
        self.authority.source_state()
    }

    pub fn cognitive_action(&self) -> &CognitiveStructure {
        self.authority.cognitive_action()
    }

    pub fn command(&self) -> &ArcAgi3SessionCommand {
        &self.command
    }

    pub fn completion(&self) -> &ArcAgi3CognitiveInteractionCompletion {
        &self.completion
    }

    pub fn completed_cognitive_step_count(&self) -> u64 {
        self.completed_cognitive_step_count
    }
}

pub struct ArcAgi3LiveEnvironmentRuntime<T>
where
    T: ArcAgi3EnvironmentTransport,
{
    transport: T,
    cognitive_runtime: ArcAgi3CognitiveInteractionRuntime,
    completed_cognitive_step_count: u64,
    completed_reset_count: u64,
    faulted_pending: bool,
    fault_disposition: Option<ArcAgi3TransportFailureDisposition>,
}

impl<T> ArcAgi3LiveEnvironmentRuntime<T>
where
    T: ArcAgi3EnvironmentTransport,
{
    pub fn start(
        mut transport: T,
        game_id: &ArcAgi3GameId,
        card_id: &str,
        first_perceptual_observation_index: u64,
    ) -> Result<Self, ArcAgi3LiveEnvironmentError> {
        let cognitive_runtime = ArcAgi3EnvironmentTransportBoundary::start_runtime(
            &mut transport,
            game_id,
            card_id,
            first_perceptual_observation_index,
        )
        .map_err(ArcAgi3LiveEnvironmentError::Transport)?;

        Ok(Self {
            transport,
            cognitive_runtime,
            completed_cognitive_step_count: 0,
            completed_reset_count: 0,
            faulted_pending: false,
            fault_disposition: None,
        })
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn cognitive_runtime(&self) -> &ArcAgi3CognitiveInteractionRuntime {
        &self.cognitive_runtime
    }

    pub fn completed_cognitive_step_count(&self) -> u64 {
        self.completed_cognitive_step_count
    }

    pub fn completed_reset_count(&self) -> u64 {
        self.completed_reset_count
    }

    pub fn fault_disposition(&self) -> Option<ArcAgi3TransportFailureDisposition> {
        self.fault_disposition
    }

    pub fn status(&self) -> ArcAgi3LiveEnvironmentStatus {
        if self.faulted_pending {
            return ArcAgi3LiveEnvironmentStatus::FaultedPending;
        }

        match self.cognitive_runtime.observation().state() {
            ArcAgi3GameState::NotPlayed => ArcAgi3LiveEnvironmentStatus::NotStarted,
            ArcAgi3GameState::NotFinished => ArcAgi3LiveEnvironmentStatus::Active,
            ArcAgi3GameState::Win => ArcAgi3LiveEnvironmentStatus::Won,
            ArcAgi3GameState::GameOver => ArcAgi3LiveEnvironmentStatus::GameOver,
        }
    }

    pub fn into_parts(self) -> (T, ArcAgi3CognitiveInteractionRuntime) {
        (self.transport, self.cognitive_runtime)
    }

    fn transport_failure_disposition(
        error: &ArcAgi3TransportError,
    ) -> Option<ArcAgi3TransportFailureDisposition> {
        match error {
            ArcAgi3TransportError::HttpTransport { disposition, .. }
            | ArcAgi3TransportError::HttpStatus { disposition, .. }
            | ArcAgi3TransportError::InvalidRemoteResponse { disposition, .. }
            | ArcAgi3TransportError::SessionIdentityMismatch { disposition, .. }
            | ArcAgi3TransportError::CognitiveInitializationRejected { disposition, .. }
            | ArcAgi3TransportError::CognitiveCompletionRejected { disposition, .. } => {
                Some(*disposition)
            }

            ArcAgi3TransportError::InvalidBaseUrl
            | ArcAgi3TransportError::EmptyApiKey
            | ArcAgi3TransportError::InvalidCardId
            | ArcAgi3TransportError::InvalidGuid
            | ArcAgi3TransportError::ActiveSessionExists
            | ArcAgi3TransportError::NoActiveSession
            | ArcAgi3TransportError::PendingCommandMismatch => None,
        }
    }

    fn ensure_not_faulted(&self) -> Result<(), ArcAgi3LiveEnvironmentError> {
        if self.faulted_pending {
            Err(ArcAgi3LiveEnvironmentError::FaultedPending(
                self.fault_disposition,
            ))
        } else {
            Ok(())
        }
    }

    fn ensure_active(&self) -> Result<(), ArcAgi3LiveEnvironmentError> {
        self.ensure_not_faulted()?;

        if self.status() != ArcAgi3LiveEnvironmentStatus::Active {
            return Err(ArcAgi3LiveEnvironmentError::GameNotActive);
        }

        Ok(())
    }

    fn mark_transport_failure(&mut self, error: &ArcAgi3TransportError) {
        self.faulted_pending = true;
        self.fault_disposition = Self::transport_failure_disposition(error);
    }

    fn complete_cognitive_step(
        &mut self,
        cognitive_step: ArcAgi3CognitiveInteractionStep,
        confidence: CognitiveSignal,
        next_completed_step_count: u64,
    ) -> Result<ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError> {
        let completion = ArcAgi3EnvironmentTransportBoundary::complete_pending(
            &mut self.transport,
            &mut self.cognitive_runtime,
            cognitive_step.command(),
            confidence,
        );

        let completion = match completion {
            Ok(completion) => completion,

            Err(error) => {
                self.mark_transport_failure(&error);

                return Err(ArcAgi3LiveEnvironmentError::Transport(error));
            }
        };

        self.completed_cognitive_step_count = next_completed_step_count;

        Ok(ArcAgi3LiveCognitiveStep {
            cognitive_step,
            completion,
            completed_cognitive_step_count: next_completed_step_count,
        })
    }

    pub fn execute_with<F>(
        &mut self,
        confidence: CognitiveSignal,
        begin: F,
    ) -> Result<ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError>
    where
        F: FnOnce(
            &mut ArcAgi3CognitiveInteractionRuntime,
        )
            -> Result<ArcAgi3CognitiveInteractionStep, ArcAgi3CognitiveInteractionError>,
    {
        self.ensure_active()?;

        let next_completed_step_count = self
            .completed_cognitive_step_count
            .checked_add(1)
            .ok_or(ArcAgi3LiveEnvironmentError::CognitiveStepCounterOverflow)?;

        let cognitive_step = begin(&mut self.cognitive_runtime)?;

        self.complete_cognitive_step(cognitive_step, confidence, next_completed_step_count)
    }

    pub fn execute_step<'a>(
        &mut self,
        anchor_state: &CognitiveStructure,
        input: OnlineCognitiveOrchestrationInput<'a>,
        cycle_policy: IntegratedAgentPolicy,
        transition_request: &CognitiveCycleStateTransitionRequest,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError> {
        self.execute_with(confidence, |runtime| {
            runtime.run_and_begin(anchor_state, input, cycle_policy, transition_request)
        })
    }

    pub fn execute_unified(
        &mut self,
        request: ArcAgi3LiveUnifiedActionRequest<'_>,
    ) -> Result<Option<ArcAgi3LiveUnifiedStep>, ArcAgi3LiveEnvironmentError> {
        self.ensure_active()?;

        let Some(authority) = self.cognitive_runtime.current_unified_executive_authority(
            request.exploitation_actions(),
            request.goal(),
            request.goal_alignment(),
            request.exploitation_execution_cost(),
            request.experiment_authority(),
            request.policy(),
        ) else {
            /*
             * Epistemic abstention is not a command.
             *
             * No pending state and no transport side effect.
             */
            return Ok(None);
        };

        let next_completed_step_count = self
            .completed_cognitive_step_count
            .checked_add(1)
            .ok_or(ArcAgi3LiveEnvironmentError::CognitiveStepCounterOverflow)?;

        /*
         * C1/M48 already chose the authority.
         * This layer performs no local action ranking.
         */
        let command = self
            .cognitive_runtime
            .begin_unified_executive_authority(&authority)?;

        let completion = ArcAgi3EnvironmentTransportBoundary::complete_pending(
            &mut self.transport,
            &mut self.cognitive_runtime,
            &command,
            request.confidence(),
        );

        let completion = match completion {
            Ok(completion) => completion,

            Err(error) => {
                self.mark_transport_failure(&error);

                return Err(ArcAgi3LiveEnvironmentError::Transport(error));
            }
        };

        self.completed_cognitive_step_count = next_completed_step_count;

        Ok(Some(ArcAgi3LiveUnifiedStep {
            authority,
            command,
            completion,
            completed_cognitive_step_count: next_completed_step_count,
        }))
    }

    pub fn reset(
        &mut self,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3CognitiveInteractionCompletion, ArcAgi3LiveEnvironmentError> {
        self.ensure_not_faulted()?;

        let next_reset_count = self
            .completed_reset_count
            .checked_add(1)
            .ok_or(ArcAgi3LiveEnvironmentError::ResetCounterOverflow)?;

        let command = self.cognitive_runtime.begin_reset()?;

        let completion = ArcAgi3EnvironmentTransportBoundary::complete_pending(
            &mut self.transport,
            &mut self.cognitive_runtime,
            &command,
            confidence,
        );

        let completion = match completion {
            Ok(completion) => completion,

            Err(error) => {
                self.mark_transport_failure(&error);

                return Err(ArcAgi3LiveEnvironmentError::Transport(error));
            }
        };

        self.completed_reset_count = next_reset_count;

        Ok(completion)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3LiveEnvironmentRuntime;

impl UniversalArcAgi3LiveEnvironmentRuntime {
    pub fn start<T>(
        transport: T,
        game_id: &ArcAgi3GameId,
        card_id: &str,
        first_perceptual_observation_index: u64,
    ) -> Result<ArcAgi3LiveEnvironmentRuntime<T>, ArcAgi3LiveEnvironmentError>
    where
        T: ArcAgi3EnvironmentTransport,
    {
        ArcAgi3LiveEnvironmentRuntime::start(
            transport,
            game_id,
            card_id,
            first_perceptual_observation_index,
        )
    }

    pub fn execute_unified<T>(
        runtime: &mut ArcAgi3LiveEnvironmentRuntime<T>,
        request: ArcAgi3LiveUnifiedActionRequest<'_>,
    ) -> Result<Option<ArcAgi3LiveUnifiedStep>, ArcAgi3LiveEnvironmentError>
    where
        T: ArcAgi3EnvironmentTransport,
    {
        runtime.execute_unified(request)
    }

    pub fn reset<T>(
        runtime: &mut ArcAgi3LiveEnvironmentRuntime<T>,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3CognitiveInteractionCompletion, ArcAgi3LiveEnvironmentError>
    where
        T: ArcAgi3EnvironmentTransport,
    {
        runtime.reset(confidence)
    }
}
