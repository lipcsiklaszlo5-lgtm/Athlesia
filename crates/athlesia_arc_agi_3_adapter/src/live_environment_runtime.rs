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
    executive_request:
        ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'a>,
    confidence:
        CognitiveSignal,
}

impl<'a> ArcAgi3LiveSuccessorInformedUnifiedActionRequest<'a> {
    pub fn new(
        executive_request:
            ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'a>,
        confidence:
            CognitiveSignal,
    ) -> Self {
        Self {
            executive_request,
            confidence,
        }
    }

    pub fn executive_request(
        self,
    ) -> ArcAgi3SuccessorInformedUnifiedExecutiveRequest<'a> {
        self.executive_request
    }

    pub fn confidence(
        self,
    ) -> CognitiveSignal {
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
        authority:
            ArcAgi3UnifiedExecutiveAuthority,
        confidence:
            CognitiveSignal,
    ) -> Result<
        ArcAgi3LiveUnifiedStep,
        ArcAgi3LiveEnvironmentError
    > {
        let next_completed_step_count =
            self
                .completed_cognitive_step_count
                .checked_add(1)
                .ok_or(
                    ArcAgi3LiveEnvironmentError::
                        CognitiveStepCounterOverflow,
                )?;

        /*
         * M48 already chose the authority.
         *
         * Live execution performs no local ranking, source inference,
         * provenance reinterpretation, or utility computation.
         */
        let command =
            self
                .cognitive_runtime
                .begin_unified_executive_authority(
                    &authority,
                )?;

        let completion =
            ArcAgi3EnvironmentTransportBoundary::
                complete_pending(
                    &mut self.transport,
                    &mut self.cognitive_runtime,
                    &command,
                    confidence,
                );

        let completion =
            match completion {
                Ok(completion) =>
                    completion,

                Err(error) => {
                    self.mark_transport_failure(
                        &error,
                    );

                    return Err(
                        ArcAgi3LiveEnvironmentError::
                            Transport(error),
                    );
                }
            };

        self.completed_cognitive_step_count =
            next_completed_step_count;

        Ok(
            ArcAgi3LiveUnifiedStep {
                authority,
                command,
                completion,
                completed_cognitive_step_count:
                    next_completed_step_count,
            },
        )
    }

    pub fn execute_unified(
        &mut self,
        request:
            ArcAgi3LiveUnifiedActionRequest<'_>,
    ) -> Result<
        Option<ArcAgi3LiveUnifiedStep>,
        ArcAgi3LiveEnvironmentError
    > {
        self.ensure_active()?;

        let Some(authority) =
            self
                .cognitive_runtime
                .current_unified_executive_authority(
                    request
                        .exploitation_actions(),
                    request
                        .goal(),
                    request
                        .goal_alignment(),
                    request
                        .exploitation_execution_cost(),
                    request
                        .experiment_authority(),
                    request
                        .policy(),
                )
        else {
            /*
             * Epistemic abstention is not a command.
             *
             * No pending state and no transport side effect.
             */
            return Ok(None);
        };

        self
            .execute_unified_authority(
                authority,
                request
                    .confidence(),
            )
            .map(Some)
    }

    pub fn execute_successor_informed_unified(
        &mut self,
        request:
            ArcAgi3LiveSuccessorInformedUnifiedActionRequest<'_>,
    ) -> Result<
        Option<ArcAgi3LiveUnifiedStep>,
        ArcAgi3LiveEnvironmentError
    > {
        self.ensure_active()?;

        let Some(authority) =
            self
                .cognitive_runtime
                .current_successor_informed_unified_executive_authority(
                    request
                        .executive_request(),
                )
        else {
            /*
             * Cognitive abstention remains side-effect free at the
             * live transport boundary.
             */
            return Ok(None);
        };

        self
            .execute_unified_authority(
                authority,
                request
                    .confidence(),
            )
            .map(Some)
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
        runtime:
            &mut ArcAgi3LiveEnvironmentRuntime<T>,
        request:
            ArcAgi3LiveSuccessorInformedUnifiedActionRequest<'_>,
    ) -> Result<
        Option<ArcAgi3LiveUnifiedStep>,
        ArcAgi3LiveEnvironmentError
    >
    where
        T: ArcAgi3EnvironmentTransport,
    {
        runtime
            .execute_successor_informed_unified(
                request,
            )
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

    use std::cell::{
        Cell,
        RefCell,
    };
    use std::collections::
        VecDeque;

    use crate::
        cognitive_interaction_runtime::
        c16i_successor_informed_two_contract_e2e_tests
            as c16i;

    #[derive(Debug)]
    struct RecordingTransport {
        responses:
            RefCell<
                VecDeque<
                    Result<
                        crate::ArcAgi3Observation,
                        ArcAgi3TransportError
                    >
                >
            >,
        execute_count:
            Cell<usize>,
        executed_actions:
            RefCell<
                Vec<ArcAgi3Action>
            >,
    }

    impl RecordingTransport {
        fn new(
            response:
                crate::ArcAgi3Observation,
        ) -> Self {
            Self {
                responses:
                    RefCell::new(
                        VecDeque::from([
                            Ok(response),
                        ]),
                    ),
                execute_count:
                    Cell::new(0),
                executed_actions:
                    RefCell::new(
                        Vec::new(),
                    ),
            }
        }

        fn execute_count(
            &self,
        ) -> usize {
            self.execute_count
                .get()
        }

        fn last_executed_action(
            &self,
        ) -> Option<ArcAgi3Action> {
            self.executed_actions
                .borrow()
                .last()
                .copied()
        }
    }

    impl ArcAgi3EnvironmentTransport
        for RecordingTransport
    {
        fn start_game(
            &mut self,
            _game_id:
                &ArcAgi3GameId,
            _card_id:
                &str,
        ) -> Result<
            crate::ArcAgi3Observation,
            ArcAgi3TransportError
        > {
            Err(
                ArcAgi3TransportError::
                    ActiveSessionExists,
            )
        }

        fn execute(
            &mut self,
            command:
                &ArcAgi3SessionCommand,
        ) -> Result<
            crate::ArcAgi3Observation,
            ArcAgi3TransportError
        > {
            self.execute_count
                .set(
                    self
                        .execute_count
                        .get()
                        .checked_add(1)
                        .expect(
                            "test transport counter remains bounded",
                        ),
                );

            self.executed_actions
                .borrow_mut()
                .push(
                    command
                        .action(),
                );

            self.responses
                .borrow_mut()
                .pop_front()
                .unwrap_or(
                    Err(
                        ArcAgi3TransportError::
                            NoActiveSession,
                    ),
                )
        }
    }

    fn signal(
        value:
            u16,
    ) -> CognitiveSignal {
        CognitiveSignal::new(
            value,
        )
        .expect(
            "test signal is positive and bounded",
        )
    }

    #[test]
    fn successor_informed_native_m50_reaches_live_transport_with_exact_m51_provenance(
    ) {
        let game =
            "c16i-live-successor-m50";

        /*
         * Canonical C16I fixture:
         *
         * real B2 evidence
         * + real current M50 question
         * + empirically measured C3D progress
         * + retained positive C3F priority
         * + caller-native matching M50 possibility/beliefs.
         */
        let fixture =
            c16i::fixture(
                game,
                8_300_000,
            );

        let (
            cognitive_runtime,
            expected_arc_action,
            expected_cognitive_action,
            expected_source_state,
            native_possibilities,
            beliefs,
        ) =
            fixture
                .into_live_parts();

        /*
         * The actual environment consequence of the selected ACTION1.
         */
        let response =
            c16i::live_response(
                game,
                6,
                Some(
                    expected_arc_action,
                ),
            );

        /*
         * Test-only assembly around the already-mature retained cognitive
         * runtime. No production mutable-cognition escape hatch is added.
         */
        let mut runtime =
            ArcAgi3LiveEnvironmentRuntime {
                transport:
                    RecordingTransport::new(
                        response,
                    ),
                cognitive_runtime,
                completed_cognitive_step_count:
                    0,
                completed_reset_count:
                    0,
                faulted_pending:
                    false,
                fault_disposition:
                    None,
            };

        let goal =
            c16i::live_goal();

        let exploitation_actions:
            [ArcAgi3Action; 0] =
                [];

        let executive_request =
            c16i::live_request(
                &exploitation_actions,
                &goal,
                &native_possibilities,
                &beliefs,
            );

        let request =
            ArcAgi3LiveSuccessorInformedUnifiedActionRequest::
                new(
                    executive_request,
                    signal(900),
                );

        let step =
            runtime
                .execute_successor_informed_unified(
                    request,
                )
                .expect(
                    "successor-informed live execution must not fail",
                )
                .expect(
                    "real native M50 authority must reach live execution",
                );

        assert_eq!(
            step.action(),
            expected_arc_action,
        );

        assert_eq!(
            step.cognitive_action(),
            &expected_cognitive_action,
        );

        assert_eq!(
            step.source_state(),
            &expected_source_state,
            "live authority must preserve exact M51-owned native M50 source provenance",
        );

        assert!(
            step
                .authority()
                .candidate()
                .information_gain()
                > CognitiveSignal::zero(),
            "live winner must carry real native M50 information authority",
        );

        assert_eq!(
            runtime
                .transport()
                .execute_count(),
            1,
            "one selected successor-informed authority produces exactly one transport side effect",
        );

        assert_eq!(
            runtime
                .transport()
                .last_executed_action(),
            Some(
                expected_arc_action,
            ),
        );

        let evidence =
            step
                .completion()
                .turn()
                .evidence()
                .expect(
                    "live completion must retain self-generated cognitive feedback",
                );

        assert_eq!(
            evidence
                .execution_observation()
                .observed_state(),
            &expected_source_state,
        );

        assert_eq!(
            evidence
                .experiment_observation()
                .source_state(),
            &expected_source_state,
        );

        assert_eq!(
            evidence
                .execution_observation()
                .observed_action(),
            &expected_cognitive_action,
        );

        assert_eq!(
            evidence
                .experiment_observation()
                .action(),
            &expected_cognitive_action,
        );
    }
}
