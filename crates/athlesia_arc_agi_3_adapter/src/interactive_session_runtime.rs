use crate::{
    cognitive_protocol_bridge::{
        ArcAgi3CognitiveBridgeError, ArcAgi3CognitiveCodecError, ArcAgi3CognitiveProtocolBridge,
    },
    ArcAgi3Action, ArcAgi3ActionAuthorizationStatus, ArcAgi3ActionId, ArcAgi3GameState,
    ArcAgi3Observation, ArcAgi3Protocol,
};
use athlesia_integrated_cognitive_agent::{
    EnvironmentActionDispatch, EnvironmentInteractionEvidence,
};
use athlesia_mindstone_sparse_cognition::CognitiveSignal;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3InteractiveSessionStatus {
    NotStarted,
    Active,
    Won,
    GameOver,
    AwaitingEnvironmentResponse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3InteractiveSessionError {
    PendingCommandExists,
    NoPendingCommand,
    GameNotActive,
    ActionUnavailable,
    ExecutiveDispatchCannotReset,
    GameIdentityMismatch,
    ReportedActionMismatch,
    TurnCounterOverflow,
    ActionCounterOverflow,
    ResetCounterOverflow,
    ActionCodec(ArcAgi3CognitiveCodecError),
    FeedbackBridge(ArcAgi3CognitiveBridgeError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3SessionCommand {
    action: ArcAgi3Action,
}

impl ArcAgi3SessionCommand {
    fn new(action: ArcAgi3Action) -> Self {
        Self { action }
    }

    pub fn action(&self) -> ArcAgi3Action {
        self.action
    }

    pub fn action_id(&self) -> ArcAgi3ActionId {
        self.action.id()
    }

    pub fn is_reset(&self) -> bool {
        self.action.id() == ArcAgi3ActionId::Reset
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PendingCommandOrigin {
    Protocol,
    Executive(EnvironmentActionDispatch),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingCommand {
    command: ArcAgi3SessionCommand,
    origin: PendingCommandOrigin,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3CompletedTurn {
    event_index: u64,
    action: ArcAgi3Action,
    observation: ArcAgi3Observation,
    evidence: Option<EnvironmentInteractionEvidence>,
}

impl ArcAgi3CompletedTurn {
    pub fn event_index(&self) -> u64 {
        self.event_index
    }

    pub fn action(&self) -> ArcAgi3Action {
        self.action
    }

    pub fn observation(&self) -> &ArcAgi3Observation {
        &self.observation
    }

    pub fn evidence(&self) -> Option<&EnvironmentInteractionEvidence> {
        self.evidence.as_ref()
    }

    pub fn has_cognitive_feedback(&self) -> bool {
        self.evidence.is_some()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3InteractiveSession {
    observation: ArcAgi3Observation,
    pending: Option<PendingCommand>,
    completed_turn_count: u64,
    completed_action_count: u64,
    completed_reset_count: u64,
}

impl ArcAgi3InteractiveSession {
    pub fn new(initial_observation: ArcAgi3Observation) -> Self {
        Self {
            observation: initial_observation,
            pending: None,
            completed_turn_count: 0,
            completed_action_count: 0,
            completed_reset_count: 0,
        }
    }

    pub fn observation(&self) -> &ArcAgi3Observation {
        &self.observation
    }

    pub fn status(&self) -> ArcAgi3InteractiveSessionStatus {
        if self.pending.is_some() {
            return ArcAgi3InteractiveSessionStatus::AwaitingEnvironmentResponse;
        }

        match self.observation.state() {
            ArcAgi3GameState::NotPlayed => ArcAgi3InteractiveSessionStatus::NotStarted,
            ArcAgi3GameState::NotFinished => ArcAgi3InteractiveSessionStatus::Active,
            ArcAgi3GameState::Win => ArcAgi3InteractiveSessionStatus::Won,
            ArcAgi3GameState::GameOver => ArcAgi3InteractiveSessionStatus::GameOver,
        }
    }

    pub fn completed_turn_count(&self) -> u64 {
        self.completed_turn_count
    }

    pub fn completed_action_count(&self) -> u64 {
        self.completed_action_count
    }

    pub fn completed_reset_count(&self) -> u64 {
        self.completed_reset_count
    }

    pub fn has_pending_command(&self) -> bool {
        self.pending.is_some()
    }

    pub fn pending_action(&self) -> Option<ArcAgi3Action> {
        self.pending
            .as_ref()
            .map(|pending| pending.command.action())
    }

    fn ensure_no_pending(&self) -> Result<(), ArcAgi3InteractiveSessionError> {
        if self.pending.is_some() {
            Err(ArcAgi3InteractiveSessionError::PendingCommandExists)
        } else {
            Ok(())
        }
    }

    fn authorize_action(
        &self,
        action: ArcAgi3Action,
    ) -> Result<(), ArcAgi3InteractiveSessionError> {
        match ArcAgi3Protocol::authorize_action(&self.observation, action).status() {
            ArcAgi3ActionAuthorizationStatus::AuthorizedAction
            | ArcAgi3ActionAuthorizationStatus::AuthorizedReset => Ok(()),
            ArcAgi3ActionAuthorizationStatus::GameNotActive => {
                Err(ArcAgi3InteractiveSessionError::GameNotActive)
            }
            ArcAgi3ActionAuthorizationStatus::ActionUnavailable => {
                Err(ArcAgi3InteractiveSessionError::ActionUnavailable)
            }
        }
    }

    fn begin_with_origin(
        &mut self,
        action: ArcAgi3Action,
        origin: PendingCommandOrigin,
    ) -> Result<ArcAgi3SessionCommand, ArcAgi3InteractiveSessionError> {
        self.ensure_no_pending()?;
        self.authorize_action(action)?;

        let command = ArcAgi3SessionCommand::new(action);

        self.pending = Some(PendingCommand {
            command: command.clone(),
            origin,
        });

        Ok(command)
    }

    pub fn begin_reset(&mut self) -> Result<ArcAgi3SessionCommand, ArcAgi3InteractiveSessionError> {
        self.begin_with_origin(ArcAgi3Action::reset(), PendingCommandOrigin::Protocol)
    }

    pub fn begin_action(
        &mut self,
        action: ArcAgi3Action,
    ) -> Result<ArcAgi3SessionCommand, ArcAgi3InteractiveSessionError> {
        self.begin_with_origin(action, PendingCommandOrigin::Protocol)
    }

    pub fn begin_dispatch(
        &mut self,
        dispatch: &EnvironmentActionDispatch,
    ) -> Result<ArcAgi3SessionCommand, ArcAgi3InteractiveSessionError> {
        self.ensure_no_pending()?;

        let action = ArcAgi3CognitiveProtocolBridge::decode_dispatch(dispatch)
            .map_err(ArcAgi3InteractiveSessionError::ActionCodec)?;

        if action.id() == ArcAgi3ActionId::Reset {
            return Err(ArcAgi3InteractiveSessionError::ExecutiveDispatchCannotReset);
        }

        self.authorize_action(action)?;

        let command = ArcAgi3SessionCommand::new(action);

        self.pending = Some(PendingCommand {
            command: command.clone(),
            origin: PendingCommandOrigin::Executive(dispatch.clone()),
        });

        Ok(command)
    }

    fn validate_response(
        &self,
        pending: &PendingCommand,
        observation: &ArcAgi3Observation,
    ) -> Result<(), ArcAgi3InteractiveSessionError> {
        if observation.game_id() != self.observation.game_id() {
            return Err(ArcAgi3InteractiveSessionError::GameIdentityMismatch);
        }

        if let Some(reported_action) = observation.last_action() {
            if reported_action != pending.command.action() {
                return Err(ArcAgi3InteractiveSessionError::ReportedActionMismatch);
            }
        }

        Ok(())
    }

    pub fn complete_turn(
        &mut self,
        observation: ArcAgi3Observation,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3CompletedTurn, ArcAgi3InteractiveSessionError> {
        let pending = self
            .pending
            .clone()
            .ok_or(ArcAgi3InteractiveSessionError::NoPendingCommand)?;

        self.validate_response(&pending, &observation)?;

        let event_index = self
            .completed_turn_count
            .checked_add(1)
            .ok_or(ArcAgi3InteractiveSessionError::TurnCounterOverflow)?;

        let action = pending.command.action();

        let next_action_count = if action.id() == ArcAgi3ActionId::Reset {
            self.completed_action_count
        } else {
            self.completed_action_count
                .checked_add(1)
                .ok_or(ArcAgi3InteractiveSessionError::ActionCounterOverflow)?
        };

        let next_reset_count = if action.id() == ArcAgi3ActionId::Reset {
            self.completed_reset_count
                .checked_add(1)
                .ok_or(ArcAgi3InteractiveSessionError::ResetCounterOverflow)?
        } else {
            self.completed_reset_count
        };

        let evidence = match &pending.origin {
            PendingCommandOrigin::Protocol => None,
            PendingCommandOrigin::Executive(dispatch) => Some(
                ArcAgi3CognitiveProtocolBridge::bind_feedback(
                    dispatch,
                    event_index,
                    &observation,
                    confidence,
                )
                .map_err(ArcAgi3InteractiveSessionError::FeedbackBridge)?,
            ),
        };

        let completed = ArcAgi3CompletedTurn {
            event_index,
            action,
            observation: observation.clone(),
            evidence,
        };

        self.observation = observation;
        self.pending = None;
        self.completed_turn_count = event_index;
        self.completed_action_count = next_action_count;
        self.completed_reset_count = next_reset_count;

        Ok(completed)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3InteractiveSession;

impl UniversalArcAgi3InteractiveSession {
    pub fn begin_dispatch(
        session: &mut ArcAgi3InteractiveSession,
        dispatch: &EnvironmentActionDispatch,
    ) -> Result<ArcAgi3SessionCommand, ArcAgi3InteractiveSessionError> {
        session.begin_dispatch(dispatch)
    }

    pub fn complete_turn(
        session: &mut ArcAgi3InteractiveSession,
        observation: ArcAgi3Observation,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3CompletedTurn, ArcAgi3InteractiveSessionError> {
        session.complete_turn(observation, confidence)
    }
}
