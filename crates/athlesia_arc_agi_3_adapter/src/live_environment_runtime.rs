use crate::cognitive_interaction_runtime::{
    ArcAgi3CognitiveInteractionCompletion, ArcAgi3CognitiveInteractionError,
    ArcAgi3CognitiveInteractionRuntime, ArcAgi3CognitiveInteractionStep,
    ArcAgi3ExperimentDispatchAuthority, ArcAgi3SuccessorInformedUnifiedExecutiveRequest,
    ArcAgi3UnifiedExecutiveAuthority,
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

#[derive(Clone, Copy, Debug)]
pub struct ArcAgi3LiveSuccessorInformedUnifiedActionRequest<'a> {
    executive_request: ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'a>,
    confidence: CognitiveSignal,
}

impl<'a> ArcAgi3LiveSuccessorInformedUnifiedActionRequest<'a> {
    pub fn new(
        executive_request: ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'a>,
        confidence: CognitiveSignal,
    ) -> Self {
        Self {
            executive_request,
            confidence,
        }
    }

    pub fn executive_request(self) -> ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'a> {
        self.executive_request
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

    fn execute_unified_authority(
        &mut self,
        authority: ArcAgi3UnifiedExecutiveAuthority,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3LiveUnifiedStep, ArcAgi3LiveEnvironmentError> {
        let next_completed_step_count = self
            .completed_cognitive_step_count
            .checked_add(1)
            .ok_or(ArcAgi3LiveEnvironmentError::CognitiveStepCounterOverflow)?;

        /*
         * M48 already chose the authority.
         *
         * Live execution performs no local ranking, source inference,
         * provenance reinterpretation, or utility computation.
         */
        let command = self
            .cognitive_runtime
            .begin_unified_executive_authority(&authority)?;

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

        self.completed_cognitive_step_count = next_completed_step_count;

        Ok(ArcAgi3LiveUnifiedStep {
            authority,
            command,
            completion,
            completed_cognitive_step_count: next_completed_step_count,
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

        self.execute_unified_authority(authority, request.confidence())
            .map(Some)
    }

    pub fn execute_successor_informed_unified(
        &mut self,
        request: ArcAgi3LiveSuccessorInformedUnifiedActionRequest<'_>,
    ) -> Result<Option<ArcAgi3LiveUnifiedStep>, ArcAgi3LiveEnvironmentError> {
        self.ensure_active()?;

        let Some(authority) = self
            .cognitive_runtime
            .current_successor_informed_unified_executive_authority(request.executive_request())
        else {
            /*
             * Cognitive abstention remains side-effect free at the
             * live transport boundary.
             */
            return Ok(None);
        };

        self.execute_unified_authority(authority, request.confidence())
            .map(Some)
    }

    /// Opt-in passive recording around the unchanged production successor path.
    pub fn execute_successor_informed_unified_with_trace(
        &mut self,
        request: ArcAgi3LiveSuccessorInformedUnifiedActionRequest<'_>,
        sink: &mut impl crate::cognitive_trace::ArcAgi3CognitiveTraceSink,
    ) -> Result<Option<ArcAgi3LiveUnifiedStep>, ArcAgi3LiveEnvironmentError> {
        use crate::cognitive_trace::{
            emit, ArcAgi3CognitiveTraceEvent as Event, ArcAgi3TraceObservation,
            ArcAgi3TraceOperation,
        };
        let before = ArcAgi3TraceObservation::from(self.cognitive_runtime.observation());
        let result = self.execute_successor_informed_unified(request);
        let event = match &result {
            Ok(Some(step)) => Some(Event::executed(before, step)),
            Ok(None) => Some(Event::Abstained {
                completed_cognitive_step_count: self.completed_cognitive_step_count,
                before,
            }),
            Err(ArcAgi3LiveEnvironmentError::Transport(error)) => Some(Event::TransportFailure {
                operation: ArcAgi3TraceOperation::SuccessorInformedUnified,
                completed_cognitive_step_count: self.completed_cognitive_step_count,
                before,
                error_debug: format!("{error:?}"),
                has_pending_command: self.cognitive_runtime.session().has_pending_command(),
                pending_command_action: self
                    .cognitive_runtime
                    .session()
                    .pending_action()
                    .map(Into::into),
                produced_cognitive_completion: false,
            }),
            // A precondition/cognitive rejection is not a transport failure or abstention.
            Err(_) => None,
        };
        if let Some(event) = event {
            emit(sink, &event);
        }
        result
    }

    /// Reset diagnostics use the actual returned completion, without new dispatch.
    pub fn reset_with_trace(
        &mut self,
        confidence: CognitiveSignal,
        sink: &mut impl crate::cognitive_trace::ArcAgi3CognitiveTraceSink,
    ) -> Result<ArcAgi3CognitiveInteractionCompletion, ArcAgi3LiveEnvironmentError> {
        use crate::cognitive_trace::{
            emit, ArcAgi3CognitiveTraceEvent as Event, ArcAgi3TraceObservation,
            ArcAgi3TraceOperation, ArcAgi3TraceOutcome,
        };
        let before = ArcAgi3TraceObservation::from(self.cognitive_runtime.observation());
        let result = self.reset(confidence);
        let event = match &result {
            Ok(completion) => Some(Event::Reset {
                completed_cognitive_step_count: self.completed_cognitive_step_count,
                completed_reset_count: self.completed_reset_count,
                outcome: ArcAgi3TraceOutcome::new(&before, completion),
                before,
                command_action: completion.turn().action().into(),
            }),
            Err(ArcAgi3LiveEnvironmentError::Transport(error)) => Some(Event::TransportFailure {
                operation: ArcAgi3TraceOperation::Reset,
                completed_cognitive_step_count: self.completed_cognitive_step_count,
                before,
                error_debug: format!("{error:?}"),
                has_pending_command: self.cognitive_runtime.session().has_pending_command(),
                pending_command_action: self
                    .cognitive_runtime
                    .session()
                    .pending_action()
                    .map(Into::into),
                produced_cognitive_completion: false,
            }),
            Err(_) => None,
        };
        if let Some(event) = event {
            emit(sink, &event);
        }
        result
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

    pub fn execute_successor_informed_unified<T>(
        runtime: &mut ArcAgi3LiveEnvironmentRuntime<T>,
        request: ArcAgi3LiveSuccessorInformedUnifiedActionRequest<'_>,
    ) -> Result<Option<ArcAgi3LiveUnifiedStep>, ArcAgi3LiveEnvironmentError>
    where
        T: ArcAgi3EnvironmentTransport,
    {
        runtime.execute_successor_informed_unified(request)
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

#[cfg(test)]
mod successor_informed_live_dispatch_tests {
    use super::*;

    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;

    use crate::cognitive_interaction_runtime::c16i_successor_informed_two_contract_e2e_tests as c16i;

    #[derive(Debug)]
    struct RecordingTransport {
        responses: RefCell<VecDeque<Result<crate::ArcAgi3Observation, ArcAgi3TransportError>>>,
        execute_count: Cell<usize>,
        executed_actions: RefCell<Vec<ArcAgi3Action>>,
    }

    impl RecordingTransport {
        fn new(response: crate::ArcAgi3Observation) -> Self {
            Self {
                responses: RefCell::new(VecDeque::from([Ok(response)])),
                execute_count: Cell::new(0),
                executed_actions: RefCell::new(Vec::new()),
            }
        }

        fn execute_count(&self) -> usize {
            self.execute_count.get()
        }

        fn last_executed_action(&self) -> Option<ArcAgi3Action> {
            self.executed_actions.borrow().last().copied()
        }
    }

    impl ArcAgi3EnvironmentTransport for RecordingTransport {
        fn start_game(
            &mut self,
            _game_id: &ArcAgi3GameId,
            _card_id: &str,
        ) -> Result<crate::ArcAgi3Observation, ArcAgi3TransportError> {
            Err(ArcAgi3TransportError::ActiveSessionExists)
        }

        fn execute(
            &mut self,
            command: &ArcAgi3SessionCommand,
        ) -> Result<crate::ArcAgi3Observation, ArcAgi3TransportError> {
            self.execute_count.set(
                self.execute_count
                    .get()
                    .checked_add(1)
                    .expect("test transport counter remains bounded"),
            );

            self.executed_actions.borrow_mut().push(command.action());

            self.responses
                .borrow_mut()
                .pop_front()
                .unwrap_or(Err(ArcAgi3TransportError::NoActiveSession))
        }
    }

    fn signal(value: u16) -> CognitiveSignal {
        CognitiveSignal::new(value).expect("test signal is positive and bounded")
    }

    #[test]
    fn successor_informed_native_m50_reaches_live_transport_with_exact_m51_provenance() {
        let game = "c16i-live-successor-m50";

        /*
         * Canonical C16I fixture:
         *
         * real B2 evidence
         * + real current M50 question
         * + empirically measured C3D progress
         * + retained positive C3F priority
         * + caller-native matching M50 possibility/beliefs.
         */
        let fixture = c16i::fixture(game, 8_300_000);

        let (
            cognitive_runtime,
            expected_arc_action,
            expected_cognitive_action,
            expected_source_state,
            native_possibilities,
            beliefs,
        ) = fixture.into_live_parts();

        /*
         * The actual environment consequence of the selected ACTION1.
         */
        let response = c16i::live_response(game, 6, Some(expected_arc_action));

        /*
         * Test-only assembly around the already-mature retained cognitive
         * runtime. No production mutable-cognition escape hatch is added.
         */
        let mut runtime = ArcAgi3LiveEnvironmentRuntime {
            transport: RecordingTransport::new(response),
            cognitive_runtime,
            completed_cognitive_step_count: 0,
            completed_reset_count: 0,
            faulted_pending: false,
            fault_disposition: None,
        };

        let goal = c16i::live_goal();

        let exploitation_actions: [ArcAgi3Action; 0] = [];

        let executive_request = c16i::live_request(
            &exploitation_actions,
            &goal,
            &native_possibilities,
            &beliefs,
        );

        let request =
            ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(executive_request, signal(900));

        let step = runtime
            .execute_successor_informed_unified(request)
            .expect("successor-informed live execution must not fail")
            .expect("real native M50 authority must reach live execution");

        assert_eq!(step.action(), expected_arc_action,);

        assert_eq!(step.cognitive_action(), &expected_cognitive_action,);

        assert_eq!(
            step.source_state(),
            &expected_source_state,
            "live authority must preserve exact M51-owned native M50 source provenance",
        );

        assert!(
            step.authority()
                .legacy_candidate()
                .expect("legacy live regression must expose legacy candidate")
                .information_gain()
                > CognitiveSignal::zero(),
            "live winner must carry real native M50 information authority",
        );

        assert_eq!(
            runtime.transport().execute_count(),
            1,
            "one selected successor-informed authority produces exactly one transport side effect",
        );

        assert_eq!(
            runtime.transport().last_executed_action(),
            Some(expected_arc_action,),
        );

        let evidence = step
            .completion()
            .turn()
            .evidence()
            .expect("live completion must retain self-generated cognitive feedback");

        assert_eq!(
            evidence.execution_observation().observed_state(),
            &expected_source_state,
        );

        assert_eq!(
            evidence.experiment_observation().source_state(),
            &expected_source_state,
        );

        assert_eq!(
            evidence.execution_observation().observed_action(),
            &expected_cognitive_action,
        );

        assert_eq!(
            evidence.experiment_observation().action(),
            &expected_cognitive_action,
        );
    }
    use crate::cognitive_trace::{
        ArcAgi3CognitiveTraceEvent as TraceEvent, ArcAgi3CognitiveTraceSink, ArcAgi3JsonlTraceSink,
        ArcAgi3TraceObservation, ArcAgi3TraceStructure,
    };

    #[derive(Default)]
    struct Collector(Vec<TraceEvent>);

    impl ArcAgi3CognitiveTraceSink for Collector {
        fn record(&mut self, event: &TraceEvent) -> std::io::Result<()> {
            self.0.push(event.clone());
            Ok(())
        }
    }

    fn live_runtime(
        cognition: ArcAgi3CognitiveInteractionRuntime,
        response: crate::ArcAgi3Observation,
    ) -> ArcAgi3LiveEnvironmentRuntime<RecordingTransport> {
        ArcAgi3LiveEnvironmentRuntime {
            transport: RecordingTransport::new(response),
            cognitive_runtime: cognition,
            completed_cognitive_step_count: 0,
            completed_reset_count: 0,
            faulted_pending: false,
            fault_disposition: None,
        }
    }

    #[test]
    fn trace_preserves_exact_successor_authority_completion_and_all_retained_cognition() {
        let game = "trace-passive-successor";
        let (cognition, action, cognitive_action, source, possibilities, beliefs) =
            c16i::fixture(game, 8_300_000).into_live_parts();
        let response = c16i::live_response(game, 6, Some(action));
        let before = ArcAgi3TraceObservation::from(cognition.observation());
        let mut plain = live_runtime(cognition.clone(), response.clone());
        let mut traced = live_runtime(cognition, response);
        let goal = c16i::live_goal();
        let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
            c16i::live_request(&[], &goal, &possibilities, &beliefs),
            signal(900),
        );
        let selected = plain
            .cognitive_runtime()
            .current_successor_informed_unified_executive_authority(request.executive_request())
            .expect("real fixture selects an M48 authority");
        let plain_step = plain
            .execute_successor_informed_unified(request)
            .unwrap()
            .unwrap();
        let mut sink = Collector::default();
        let traced_step = traced
            .execute_successor_informed_unified_with_trace(request, &mut sink)
            .unwrap()
            .unwrap();
        assert_eq!(plain_step, traced_step);
        assert_eq!(traced_step.authority(), &selected);
        assert_eq!(plain.transport().execute_count(), 1);
        assert_eq!(traced.transport().execute_count(), 1);
        assert_eq!(plain.cognitive_runtime(), traced.cognitive_runtime());
        assert_eq!(
            plain.completed_cognitive_step_count(),
            traced.completed_cognitive_step_count()
        );
        assert_eq!(sink.0.len(), 1);
        let TraceEvent::Executed {
            authority,
            command_action,
            outcome,
            completed_cognitive_step_count,
            before: recorded_before,
        } = &sink.0[0]
        else {
            panic!("expected execution");
        };
        assert_eq!(recorded_before, &before);
        assert_eq!(*completed_cognitive_step_count, 1);
        assert_eq!(authority.selected_action, action.into());
        assert_eq!(*command_action, traced_step.command().action().into());
        assert_eq!(
            authority.cognitive_action,
            ArcAgi3TraceStructure::from(&cognitive_action)
        );
        assert_eq!(authority.source_state, ArcAgi3TraceStructure::from(&source));
        assert_eq!(
            authority.source_state,
            ArcAgi3TraceStructure::from(selected.source_state())
        );
        assert_eq!(
            authority.cognitive_action,
            ArcAgi3TraceStructure::from(selected.cognitive_action())
        );
        assert_eq!(outcome.observed_action_echo, Some(action.into()));
        assert_eq!(
            outcome.observation,
            ArcAgi3TraceObservation::from(traced.cognitive_runtime().observation())
        );
        assert_eq!(outcome.structurally_changed, before != outcome.observation);
        assert!(outcome.produced_cognitive_completion);
        assert!(outcome.has_cognitive_feedback);

        // Repeating selection on identical retained state cannot acquire a diagnostic candidate.
        assert_eq!(
            plain
                .cognitive_runtime()
                .current_successor_informed_unified_executive_authority(
                    request.executive_request()
                ),
            traced
                .cognitive_runtime()
                .current_successor_informed_unified_executive_authority(
                    request.executive_request()
                )
        );
        assert_jsonl_round_trip(&sink.0[0]);
    }

    fn assert_jsonl_round_trip(event: &TraceEvent) {
        let mut sink = ArcAgi3JsonlTraceSink::new(Vec::new());
        sink.record(event).unwrap();
        sink.record(event).unwrap();
        let text = String::from_utf8(sink.writer).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], lines[1]);
        let decoded: TraceEvent = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(&decoded, event);
        let mut encoded = Vec::new();
        decoded.write_jsonl(&mut encoded).unwrap();
        assert_eq!(encoded, format!("{}\n", lines[0]).as_bytes());
    }

    #[test]
    fn trace_abstention_cannot_create_an_action_or_mutate_cognition() {
        let game = "trace-abstention";
        let (cognition, _, _, _, _, _) = c16i::fixture(game, 8_300_000).into_live_parts();
        let snapshot = cognition.clone();
        let response = c16i::live_response(game, 6, None);
        let mut plain = live_runtime(cognition.clone(), response.clone());
        let mut traced = live_runtime(cognition, response);
        let goal = c16i::live_goal();
        let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
            c16i::live_request(&[], &goal, &[], &[]),
            signal(900),
        );
        let mut sink = Collector::default();
        assert_eq!(
            plain.execute_successor_informed_unified(request).unwrap(),
            None
        );
        assert_eq!(
            traced
                .execute_successor_informed_unified_with_trace(request, &mut sink)
                .unwrap(),
            None
        );
        assert_eq!(plain.transport().execute_count(), 0);
        assert_eq!(traced.transport().execute_count(), 0);
        assert_eq!(plain.cognitive_runtime(), &snapshot);
        assert_eq!(traced.cognitive_runtime(), &snapshot);
        assert!(!traced.cognitive_runtime().session().has_pending_command());
        assert_eq!(traced.completed_cognitive_step_count(), 0);
        assert_eq!(sink.0.len(), 1);
        assert!(matches!(sink.0[0], TraceEvent::Abstained { .. }));
        let json = serde_json::to_value(&sink.0[0]).unwrap();
        let keys: Vec<_> = json
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            vec!["before", "completed_cognitive_step_count", "event"]
        );
        assert_jsonl_round_trip(&sink.0[0]);
    }

    #[test]
    fn trace_reset_and_transport_failure_preserve_results_and_state() {
        for reset in [false, true] {
            for fail in [false, true] {
                let game = "trace-reset-failure";
                let (cognition, action, _, _, possibilities, beliefs) =
                    c16i::fixture(game, 8_300_000).into_live_parts();
                let response = c16i::live_response(
                    game,
                    6,
                    Some(if reset {
                        ArcAgi3Action::reset()
                    } else {
                        action
                    }),
                );
                let mut plain = live_runtime(cognition.clone(), response.clone());
                let mut traced = live_runtime(cognition, response);
                if fail {
                    for runtime in [&mut plain, &mut traced] {
                        *runtime.transport.responses.borrow_mut() =
                            VecDeque::from([Err(ArcAgi3TransportError::HttpTransport {
                                message: "fixture transport failed".into(),
                                disposition:
                                    ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
                            })]);
                    }
                }
                let mut sink = Collector::default();
                if reset {
                    assert_eq!(
                        plain.reset(signal(900)),
                        traced.reset_with_trace(signal(900), &mut sink)
                    );
                } else {
                    let goal = c16i::live_goal();
                    let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
                        c16i::live_request(&[], &goal, &possibilities, &beliefs),
                        signal(900),
                    );
                    assert_eq!(
                        plain.execute_successor_informed_unified(request),
                        traced.execute_successor_informed_unified_with_trace(request, &mut sink)
                    );
                }
                assert_eq!(plain.transport().execute_count(), 1);
                assert_eq!(traced.transport().execute_count(), 1);
                assert_eq!(plain.cognitive_runtime(), traced.cognitive_runtime());
                assert_eq!(plain.status(), traced.status());
                assert_eq!(plain.fault_disposition(), traced.fault_disposition());
                assert_eq!(
                    plain.completed_reset_count(),
                    traced.completed_reset_count()
                );
                assert_eq!(
                    plain.completed_cognitive_step_count(),
                    traced.completed_cognitive_step_count()
                );
                assert_eq!(sink.0.len(), 1);
                if fail {
                    let TraceEvent::TransportFailure {
                        has_pending_command,
                        pending_command_action,
                        produced_cognitive_completion,
                        error_debug,
                        ..
                    } = &sink.0[0]
                    else {
                        panic!("expected transport failure");
                    };
                    assert!(*has_pending_command);
                    assert!(!produced_cognitive_completion);
                    assert_eq!(
                        *pending_command_action,
                        Some(
                            (if reset {
                                ArcAgi3Action::reset()
                            } else {
                                action
                            })
                            .into()
                        )
                    );
                    assert!(error_debug.contains("DispatchIndeterminate"));
                } else if reset {
                    assert!(matches!(
                        sink.0[0],
                        TraceEvent::Reset {
                            completed_reset_count: 1,
                            ..
                        }
                    ));
                }
                assert_jsonl_round_trip(&sink.0[0]);
            }
        }
    }

    #[test]
    fn trace_sink_errors_and_unwinding_panics_cannot_change_execution() {
        struct BrokenSink(bool);
        impl ArcAgi3CognitiveTraceSink for BrokenSink {
            fn record(&mut self, _: &TraceEvent) -> std::io::Result<()> {
                if self.0 {
                    panic!("diagnostic sink panic");
                }
                Err(std::io::Error::other("diagnostic sink error"))
            }
        }
        for panic in [false, true] {
            let game = "trace-broken-sink";
            let (cognition, action, _, _, possibilities, beliefs) =
                c16i::fixture(game, 8_300_000).into_live_parts();
            let response = c16i::live_response(game, 6, Some(action));
            let mut plain = live_runtime(cognition.clone(), response.clone());
            let mut traced = live_runtime(cognition, response);
            let goal = c16i::live_goal();
            let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
                c16i::live_request(&[], &goal, &possibilities, &beliefs),
                signal(900),
            );
            assert_eq!(
                plain.execute_successor_informed_unified(request),
                traced
                    .execute_successor_informed_unified_with_trace(request, &mut BrokenSink(panic))
            );
            assert_eq!(plain.cognitive_runtime(), traced.cognitive_runtime());
            assert_eq!(traced.transport().execute_count(), 1);
        }
    }
    #[test]
    fn trace_reports_unchanged_observations_and_exact_terminal_states() {
        for state in [
            ArcAgi3GameState::NotFinished,
            ArcAgi3GameState::Win,
            ArcAgi3GameState::GameOver,
        ] {
            let game = "trace-terminal";
            let (cognition, action, _, _, possibilities, beliefs) =
                c16i::fixture(game, 8_300_000).into_live_parts();
            let before = cognition.observation();
            let response = crate::ArcAgi3Observation::new(
                before.game_id().clone(),
                state,
                before.frames().clone(),
                before.levels_completed(),
                before.win_levels(),
                before.available_actions().clone(),
                Some(action),
            );
            let mut plain = live_runtime(cognition.clone(), response.clone());
            let mut traced = live_runtime(cognition, response);
            let goal = c16i::live_goal();
            let request = ArcAgi3LiveSuccessorInformedUnifiedActionRequest::new(
                c16i::live_request(&[], &goal, &possibilities, &beliefs),
                signal(900),
            );
            let mut sink = Collector::default();
            let expected = plain
                .execute_successor_informed_unified(request)
                .unwrap()
                .unwrap();
            let actual = traced
                .execute_successor_informed_unified_with_trace(request, &mut sink)
                .unwrap()
                .unwrap();
            assert_eq!(actual, expected);
            assert_eq!(plain.cognitive_runtime(), traced.cognitive_runtime());
            let TraceEvent::Executed { outcome, .. } = &sink.0[0] else {
                panic!("expected executed");
            };
            assert_eq!(outcome.observation.state, state.into());
            assert_eq!(
                outcome.structurally_changed,
                state != ArcAgi3GameState::NotFinished
            );
            assert_jsonl_round_trip(&sink.0[0]);
            if state.is_terminal() {
                let count = sink.0.len();
                assert_eq!(
                    plain.execute_successor_informed_unified(request),
                    traced.execute_successor_informed_unified_with_trace(request, &mut sink)
                );
                assert_eq!(
                    sink.0.len(),
                    count,
                    "precondition errors are not transport failures"
                );
                assert_eq!(traced.transport().execute_count(), 1);
            }
        }
    }
}

#[cfg(test)]
#[path = "successor_episode_runtime_tests.rs"]
mod successor_episode_tests;
