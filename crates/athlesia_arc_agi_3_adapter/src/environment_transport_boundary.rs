use crate::cognitive_interaction_runtime::{
    ArcAgi3CognitiveInteractionCompletion, ArcAgi3CognitiveInteractionError,
    ArcAgi3CognitiveInteractionRuntime,
};
use crate::interactive_session_runtime::ArcAgi3SessionCommand;
use crate::{
    ArcAgi3Action, ArcAgi3ActionId, ArcAgi3AvailableActions, ArcAgi3Coordinate,
    ArcAgi3FrameSequence, ArcAgi3GameId, ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation,
};
use athlesia_mindstone_sparse_cognition::CognitiveSignal;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3TransportFailureDisposition {
    NotDispatched,
    RejectedByEnvironment,
    DispatchIndeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArcAgi3RestDecodeError {
    InvalidJson(String),
    InvalidGameId,
    InvalidGuid,
    InvalidState(String),
    InvalidFrame,
    InvalidAvailableActions,
    InvalidActionId(u8),
    InvalidActionPayload,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ArcAgi3TransportError {
    InvalidBaseUrl,
    EmptyApiKey,
    InvalidCardId,
    InvalidGuid,
    ActiveSessionExists,
    NoActiveSession,
    PendingCommandMismatch,
    HttpTransport {
        message: String,
        disposition: ArcAgi3TransportFailureDisposition,
    },
    HttpStatus {
        status: u16,
        body: String,
        disposition: ArcAgi3TransportFailureDisposition,
    },
    InvalidRemoteResponse {
        error: ArcAgi3RestDecodeError,
        disposition: ArcAgi3TransportFailureDisposition,
    },
    SessionIdentityMismatch {
        expected: String,
        actual: String,
        disposition: ArcAgi3TransportFailureDisposition,
    },
    CognitiveInitializationRejected {
        error: ArcAgi3CognitiveInteractionError,
        disposition: ArcAgi3TransportFailureDisposition,
    },
    CognitiveCompletionRejected {
        error: ArcAgi3CognitiveInteractionError,
        disposition: ArcAgi3TransportFailureDisposition,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3RestSession {
    game_id: ArcAgi3GameId,
    card_id: String,
    guid: String,
}

impl ArcAgi3RestSession {
    pub fn new(
        game_id: ArcAgi3GameId,
        card_id: String,
        guid: String,
    ) -> Result<Self, ArcAgi3TransportError> {
        if card_id.is_empty() || card_id.chars().any(char::is_whitespace) {
            return Err(ArcAgi3TransportError::InvalidCardId);
        }

        if guid.is_empty() || guid.chars().any(char::is_whitespace) {
            return Err(ArcAgi3TransportError::InvalidGuid);
        }

        Ok(Self {
            game_id,
            card_id,
            guid,
        })
    }

    pub fn game_id(&self) -> &ArcAgi3GameId {
        &self.game_id
    }

    pub fn card_id(&self) -> &str {
        &self.card_id
    }

    pub fn guid(&self) -> &str {
        &self.guid
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3RestRequest {
    endpoint: String,
    game_id: String,
    card_id: Option<String>,
    guid: Option<String>,
    coordinate: Option<ArcAgi3Coordinate>,
}

impl ArcAgi3RestRequest {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn game_id(&self) -> &str {
        &self.game_id
    }

    pub fn card_id(&self) -> Option<&str> {
        self.card_id.as_deref()
    }

    pub fn guid(&self) -> Option<&str> {
        self.guid.as_deref()
    }

    pub fn coordinate(&self) -> Option<ArcAgi3Coordinate> {
        self.coordinate
    }

    fn wire_body(&self) -> ArcAgi3RestRequestBody<'_> {
        let coordinate = self.coordinate.as_ref();

        ArcAgi3RestRequestBody {
            game_id: &self.game_id,
            card_id: self.card_id.as_deref(),
            guid: self.guid.as_deref(),
            x: coordinate.map(|value| value.x()),
            y: coordinate.map(|value| value.y()),
        }
    }
}

#[derive(Serialize)]
struct ArcAgi3RestRequestBody<'a> {
    game_id: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    card_id: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    guid: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3RestObservation {
    observation: ArcAgi3Observation,
    guid: String,
}

impl ArcAgi3RestObservation {
    pub fn observation(&self) -> &ArcAgi3Observation {
        &self.observation
    }

    pub fn guid(&self) -> &str {
        &self.guid
    }

    pub fn into_parts(self) -> (ArcAgi3Observation, String) {
        (self.observation, self.guid)
    }
}

#[derive(Deserialize)]
struct ArcAgi3WireObservation {
    game_id: String,
    guid: String,
    frame: Vec<Vec<Vec<u8>>>,
    state: String,
    levels_completed: u64,
    win_levels: u64,

    #[serde(default)]
    action_input: Option<ArcAgi3WireActionInput>,

    available_actions: Vec<u8>,
}

#[derive(Deserialize)]
struct ArcAgi3WireActionInput {
    id: u8,

    #[serde(default)]
    data: Option<ArcAgi3WireActionData>,
}

#[derive(Deserialize)]
struct ArcAgi3WireActionData {
    #[serde(default)]
    x: Option<u8>,

    #[serde(default)]
    y: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3RestProtocol;

impl ArcAgi3RestProtocol {
    fn action_id_from_numeric(value: u8) -> Result<ArcAgi3ActionId, ArcAgi3RestDecodeError> {
        match value {
            0 => Ok(ArcAgi3ActionId::Reset),
            1 => Ok(ArcAgi3ActionId::Action1),
            2 => Ok(ArcAgi3ActionId::Action2),
            3 => Ok(ArcAgi3ActionId::Action3),
            4 => Ok(ArcAgi3ActionId::Action4),
            5 => Ok(ArcAgi3ActionId::Action5),
            6 => Ok(ArcAgi3ActionId::Action6),
            7 => Ok(ArcAgi3ActionId::Action7),
            _ => Err(ArcAgi3RestDecodeError::InvalidActionId(value)),
        }
    }

    fn decode_action(
        action: ArcAgi3WireActionInput,
    ) -> Result<ArcAgi3Action, ArcAgi3RestDecodeError> {
        let id = Self::action_id_from_numeric(action.id)?;

        let x = action.data.as_ref().and_then(|data| data.x);
        let y = action.data.as_ref().and_then(|data| data.y);

        match id {
            ArcAgi3ActionId::Action6 => {
                let (Some(x), Some(y)) = (x, y) else {
                    return Err(ArcAgi3RestDecodeError::InvalidActionPayload);
                };

                ArcAgi3Action::coordinate(x, y).ok_or(ArcAgi3RestDecodeError::InvalidActionPayload)
            }

            ArcAgi3ActionId::Reset => {
                if x.is_some() || y.is_some() {
                    return Err(ArcAgi3RestDecodeError::InvalidActionPayload);
                }

                Ok(ArcAgi3Action::reset())
            }

            _ => {
                if x.is_some() || y.is_some() {
                    return Err(ArcAgi3RestDecodeError::InvalidActionPayload);
                }

                ArcAgi3Action::discrete(id).ok_or(ArcAgi3RestDecodeError::InvalidActionPayload)
            }
        }
    }

    fn decode_state(state: &str) -> Result<ArcAgi3GameState, ArcAgi3RestDecodeError> {
        match state {
            "NOT_STARTED" => Ok(ArcAgi3GameState::NotPlayed),
            "NOT_FINISHED" => Ok(ArcAgi3GameState::NotFinished),
            "WIN" => Ok(ArcAgi3GameState::Win),
            "GAME_OVER" => Ok(ArcAgi3GameState::GameOver),
            _ => Err(ArcAgi3RestDecodeError::InvalidState(state.to_string())),
        }
    }

    pub fn start_request(
        game_id: &ArcAgi3GameId,
        card_id: &str,
    ) -> Result<ArcAgi3RestRequest, ArcAgi3TransportError> {
        if card_id.is_empty() || card_id.chars().any(char::is_whitespace) {
            return Err(ArcAgi3TransportError::InvalidCardId);
        }

        Ok(ArcAgi3RestRequest {
            endpoint: "/api/cmd/RESET".to_string(),
            game_id: game_id.as_str().to_string(),
            card_id: Some(card_id.to_string()),
            guid: None,
            coordinate: None,
        })
    }

    pub fn command_request(
        session: &ArcAgi3RestSession,
        command: &ArcAgi3SessionCommand,
    ) -> Result<ArcAgi3RestRequest, ArcAgi3TransportError> {
        let action = command.action();

        let coordinate =
            if action.id() == ArcAgi3ActionId::Action6 {
                Some(action.coordinate_data().ok_or(
                    ArcAgi3TransportError::InvalidRemoteResponse {
                        error: ArcAgi3RestDecodeError::InvalidActionPayload,
                        disposition: ArcAgi3TransportFailureDisposition::NotDispatched,
                    },
                )?)
            } else {
                None
            };

        let card_id = if command.is_reset() {
            Some(session.card_id().to_string())
        } else {
            None
        };

        Ok(ArcAgi3RestRequest {
            endpoint: format!("/api/cmd/{}", action.id().protocol_name()),
            game_id: session.game_id().as_str().to_string(),
            card_id,
            guid: Some(session.guid().to_string()),
            coordinate,
        })
    }

    pub fn decode_observation_json(
        json: &str,
    ) -> Result<ArcAgi3RestObservation, ArcAgi3RestDecodeError> {
        let wire: ArcAgi3WireObservation = serde_json::from_str(json)
            .map_err(|error| ArcAgi3RestDecodeError::InvalidJson(error.to_string()))?;

        let game_id =
            ArcAgi3GameId::new(wire.game_id).ok_or(ArcAgi3RestDecodeError::InvalidGameId)?;

        if wire.guid.is_empty() || wire.guid.chars().any(char::is_whitespace) {
            return Err(ArcAgi3RestDecodeError::InvalidGuid);
        }

        let state = Self::decode_state(&wire.state)?;

        let frames = wire
            .frame
            .into_iter()
            .map(|rows| ArcAgi3Grid::from_rows(rows).ok_or(ArcAgi3RestDecodeError::InvalidFrame))
            .collect::<Result<Vec<_>, _>>()?;

        let frames =
            ArcAgi3FrameSequence::new(frames).ok_or(ArcAgi3RestDecodeError::InvalidFrame)?;

        let available_actions = wire
            .available_actions
            .into_iter()
            .map(Self::action_id_from_numeric)
            .collect::<Result<Vec<_>, _>>()?;

        if available_actions.contains(&ArcAgi3ActionId::Reset) {
            return Err(ArcAgi3RestDecodeError::InvalidAvailableActions);
        }

        let available_actions = ArcAgi3AvailableActions::new(available_actions)
            .ok_or(ArcAgi3RestDecodeError::InvalidAvailableActions)?;

        let last_action = wire.action_input.map(Self::decode_action).transpose()?;

        let levels_completed = wire
            .levels_completed
            .try_into()
            .map_err(|_| ArcAgi3RestDecodeError::InvalidFrame)?;

        let win_levels = wire
            .win_levels
            .try_into()
            .map_err(|_| ArcAgi3RestDecodeError::InvalidFrame)?;

        Ok(ArcAgi3RestObservation {
            observation: ArcAgi3Observation::new(
                game_id,
                state,
                frames,
                levels_completed,
                win_levels,
                available_actions,
                last_action,
            ),
            guid: wire.guid,
        })
    }
}

pub trait ArcAgi3EnvironmentTransport {
    fn start_game(
        &mut self,
        game_id: &ArcAgi3GameId,
        card_id: &str,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError>;

    fn execute(
        &mut self,
        command: &ArcAgi3SessionCommand,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError>;
}

pub struct ArcAgi3RestTransport {
    client: Client,
    base_url: String,
    api_key: String,
    session: Option<ArcAgi3RestSession>,
}

impl ArcAgi3RestTransport {
    pub fn new(base_url: String, api_key: String) -> Result<Self, ArcAgi3TransportError> {
        let parsed =
            reqwest::Url::parse(&base_url).map_err(|_| ArcAgi3TransportError::InvalidBaseUrl)?;

        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(ArcAgi3TransportError::InvalidBaseUrl);
        }

        if api_key.is_empty() || api_key.chars().any(char::is_whitespace) {
            return Err(ArcAgi3TransportError::EmptyApiKey);
        }

        let client = Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|error| ArcAgi3TransportError::HttpTransport {
                message: error.to_string(),
                disposition: ArcAgi3TransportFailureDisposition::NotDispatched,
            })?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            session: None,
        })
    }

    pub fn session(&self) -> Option<&ArcAgi3RestSession> {
        self.session.as_ref()
    }

    fn send(
        &self,
        request: &ArcAgi3RestRequest,
    ) -> Result<ArcAgi3RestObservation, ArcAgi3TransportError> {
        let response = self
            .client
            .post(format!("{}{}", self.base_url, request.endpoint()))
            .header("X-API-Key", &self.api_key)
            .json(&request.wire_body())
            .send()
            .map_err(|error| ArcAgi3TransportError::HttpTransport {
                message: error.to_string(),
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
            })?;

        let status = response.status();

        let body = response
            .text()
            .map_err(|error| ArcAgi3TransportError::HttpTransport {
                message: error.to_string(),
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
            })?;

        if !status.is_success() {
            let disposition = if status.is_client_error() {
                ArcAgi3TransportFailureDisposition::RejectedByEnvironment
            } else {
                ArcAgi3TransportFailureDisposition::DispatchIndeterminate
            };

            return Err(ArcAgi3TransportError::HttpStatus {
                status: status.as_u16(),
                body,
                disposition,
            });
        }

        ArcAgi3RestProtocol::decode_observation_json(&body).map_err(|error| {
            ArcAgi3TransportError::InvalidRemoteResponse {
                error,
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
            }
        })
    }
}

impl ArcAgi3EnvironmentTransport for ArcAgi3RestTransport {
    fn start_game(
        &mut self,
        game_id: &ArcAgi3GameId,
        card_id: &str,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        if self.session.is_some() {
            return Err(ArcAgi3TransportError::ActiveSessionExists);
        }

        let request = ArcAgi3RestProtocol::start_request(game_id, card_id)?;

        let response = self.send(&request)?;

        let (observation, guid) = response.into_parts();

        self.session = Some(ArcAgi3RestSession::new(
            observation.game_id().clone(),
            card_id.to_string(),
            guid,
        )?);

        Ok(observation)
    }

    fn execute(
        &mut self,
        command: &ArcAgi3SessionCommand,
    ) -> Result<ArcAgi3Observation, ArcAgi3TransportError> {
        let session = self
            .session
            .clone()
            .ok_or(ArcAgi3TransportError::NoActiveSession)?;

        let request = ArcAgi3RestProtocol::command_request(&session, command)?;

        let response = self.send(&request)?;

        if response.observation().game_id() != session.game_id() {
            return Err(ArcAgi3TransportError::SessionIdentityMismatch {
                expected: session.game_id().as_str().to_string(),
                actual: response.observation().game_id().as_str().to_string(),
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
            });
        }

        let (observation, guid) = response.into_parts();

        self.session = Some(ArcAgi3RestSession::new(
            observation.game_id().clone(),
            session.card_id().to_string(),
            guid,
        )?);

        Ok(observation)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3EnvironmentTransportBoundary;

impl ArcAgi3EnvironmentTransportBoundary {
    pub fn start_runtime<T: ArcAgi3EnvironmentTransport>(
        transport: &mut T,
        game_id: &ArcAgi3GameId,
        card_id: &str,
        first_perceptual_observation_index: u64,
    ) -> Result<ArcAgi3CognitiveInteractionRuntime, ArcAgi3TransportError> {
        let observation = transport.start_game(game_id, card_id)?;

        ArcAgi3CognitiveInteractionRuntime::new(observation, first_perceptual_observation_index)
            .map_err(
                |error| ArcAgi3TransportError::CognitiveInitializationRejected {
                    error,
                    disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
                },
            )
    }

    pub fn complete_pending<T: ArcAgi3EnvironmentTransport>(
        transport: &mut T,
        runtime: &mut ArcAgi3CognitiveInteractionRuntime,
        command: &ArcAgi3SessionCommand,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3CognitiveInteractionCompletion, ArcAgi3TransportError> {
        if runtime.session().pending_action() != Some(command.action()) {
            return Err(ArcAgi3TransportError::PendingCommandMismatch);
        }

        let observation = transport.execute(command)?;

        runtime
            .complete_environment_turn(observation, confidence)
            .map_err(|error| ArcAgi3TransportError::CognitiveCompletionRejected {
                error,
                disposition: ArcAgi3TransportFailureDisposition::DispatchIndeterminate,
            })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3EnvironmentTransportBoundary;

impl UniversalArcAgi3EnvironmentTransportBoundary {
    pub fn start_runtime<T: ArcAgi3EnvironmentTransport>(
        transport: &mut T,
        game_id: &ArcAgi3GameId,
        card_id: &str,
        first_perceptual_observation_index: u64,
    ) -> Result<ArcAgi3CognitiveInteractionRuntime, ArcAgi3TransportError> {
        ArcAgi3EnvironmentTransportBoundary::start_runtime(
            transport,
            game_id,
            card_id,
            first_perceptual_observation_index,
        )
    }

    pub fn complete_pending<T: ArcAgi3EnvironmentTransport>(
        transport: &mut T,
        runtime: &mut ArcAgi3CognitiveInteractionRuntime,
        command: &ArcAgi3SessionCommand,
        confidence: CognitiveSignal,
    ) -> Result<ArcAgi3CognitiveInteractionCompletion, ArcAgi3TransportError> {
        ArcAgi3EnvironmentTransportBoundary::complete_pending(
            transport, runtime, command, confidence,
        )
    }
}
