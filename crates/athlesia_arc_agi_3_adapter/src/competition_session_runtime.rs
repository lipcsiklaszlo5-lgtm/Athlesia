use reqwest::blocking::{Client, Response};
use reqwest::header::ACCEPT;
use reqwest::Url;
use serde_json::{Map, Value};

use crate::bounded_episode_runtime::{
    ArcAgi3BoundedEpisodeError, ArcAgi3BoundedEpisodePolicy, ArcAgi3BoundedEpisodeResult,
    ArcAgi3BoundedEpisodeRuntime,
};
use crate::environment_transport_boundary::ArcAgi3EnvironmentTransport;
use crate::live_environment_runtime::{
    ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError, ArcAgi3LiveEnvironmentRuntime,
};
use crate::ArcAgi3GameId;

pub const ARC_AGI_3_SCORECARD_OPAQUE_MAX_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArcAgi3CompetitionProtocolError {
    InvalidSourceUrl,
    OpaqueSerializationFailed(String),
    OpaqueTooLarge {
        actual_bytes: usize,
        maximum_bytes: usize,
    },
    InvalidCardId,
    MalformedResponse(String),
    NonFiniteScore,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ArcAgi3ScorecardId(String);

impl ArcAgi3ScorecardId {
    pub fn new(value: String) -> Result<Self, ArcAgi3CompetitionProtocolError> {
        if value.trim().is_empty() {
            return Err(ArcAgi3CompetitionProtocolError::InvalidCardId);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArcAgi3CompetitionMetadata {
    source_url: Option<String>,
    tags: Vec<String>,
    opaque: Option<Value>,
}

impl ArcAgi3CompetitionMetadata {
    pub fn new(
        source_url: Option<String>,
        tags: Vec<String>,
        opaque: Option<Value>,
    ) -> Result<Self, ArcAgi3CompetitionProtocolError> {
        if let Some(source_url) = source_url.as_deref() {
            if source_url.trim().is_empty() || Url::parse(source_url).is_err() {
                return Err(ArcAgi3CompetitionProtocolError::InvalidSourceUrl);
            }
        }

        if let Some(opaque) = opaque.as_ref() {
            let serialized = serde_json::to_vec(opaque).map_err(|error| {
                ArcAgi3CompetitionProtocolError::OpaqueSerializationFailed(error.to_string())
            })?;

            if serialized.len() > ARC_AGI_3_SCORECARD_OPAQUE_MAX_BYTES {
                return Err(ArcAgi3CompetitionProtocolError::OpaqueTooLarge {
                    actual_bytes: serialized.len(),
                    maximum_bytes: ARC_AGI_3_SCORECARD_OPAQUE_MAX_BYTES,
                });
            }
        }

        Ok(Self {
            source_url,
            tags,
            opaque,
        })
    }

    pub fn source_url(&self) -> Option<&str> {
        self.source_url.as_deref()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn opaque(&self) -> Option<&Value> {
        self.opaque.as_ref()
    }

    pub fn competition_mode(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ArcAgi3ScorecardSummary {
    card_id: ArcAgi3ScorecardId,
    score: f64,
    environments: Vec<Value>,
    total_environments_completed: u64,
    total_environments: u64,
    total_levels_completed: u64,
    total_levels: u64,
    total_actions: u64,
    competition_mode: Option<bool>,
    published_at: Option<String>,
    raw: Value,
}

impl ArcAgi3ScorecardSummary {
    pub fn card_id(&self) -> &ArcAgi3ScorecardId {
        &self.card_id
    }

    pub fn score(&self) -> f64 {
        self.score
    }

    pub fn environments(&self) -> &[Value] {
        &self.environments
    }

    pub fn total_environments_completed(&self) -> u64 {
        self.total_environments_completed
    }

    pub fn total_environments(&self) -> u64 {
        self.total_environments
    }

    pub fn total_levels_completed(&self) -> u64 {
        self.total_levels_completed
    }

    pub fn total_levels(&self) -> u64 {
        self.total_levels
    }

    pub fn total_actions(&self) -> u64 {
        self.total_actions
    }

    pub fn competition_mode(&self) -> Option<bool> {
        self.competition_mode
    }

    pub fn published_at(&self) -> Option<&str> {
        self.published_at.as_deref()
    }

    pub fn raw(&self) -> &Value {
        &self.raw
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3ScorecardRestProtocol;

impl ArcAgi3ScorecardRestProtocol {
    pub const OPEN_PATH: &'static str = "api/scorecard/open";
    pub const CLOSE_PATH: &'static str = "api/scorecard/close";

    pub fn open_request(metadata: &ArcAgi3CompetitionMetadata) -> Value {
        let mut object = Map::new();

        if let Some(source_url) = metadata.source_url() {
            object.insert(
                "source_url".to_string(),
                Value::String(source_url.to_string()),
            );
        }

        if !metadata.tags().is_empty() {
            object.insert(
                "tags".to_string(),
                Value::Array(metadata.tags().iter().cloned().map(Value::String).collect()),
            );
        }

        if let Some(opaque) = metadata.opaque() {
            object.insert("opaque".to_string(), opaque.clone());
        }

        object.insert("competition_mode".to_string(), Value::Bool(true));

        Value::Object(object)
    }

    pub fn close_request(card_id: &ArcAgi3ScorecardId) -> Value {
        serde_json::json!({
            "card_id": card_id.as_str(),
        })
    }

    pub fn decode_open_response(
        value: Value,
    ) -> Result<ArcAgi3ScorecardId, ArcAgi3CompetitionProtocolError> {
        let object = value.as_object().ok_or_else(|| {
            ArcAgi3CompetitionProtocolError::MalformedResponse(
                "open scorecard response must be an object".to_string(),
            )
        })?;

        let card_id = object
            .get("card_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ArcAgi3CompetitionProtocolError::MalformedResponse(
                    "open scorecard response requires string card_id".to_string(),
                )
            })?;

        ArcAgi3ScorecardId::new(card_id.to_string())
    }

    fn required_u64(
        object: &Map<String, Value>,
        key: &str,
    ) -> Result<u64, ArcAgi3CompetitionProtocolError> {
        object.get(key).and_then(Value::as_u64).ok_or_else(|| {
            ArcAgi3CompetitionProtocolError::MalformedResponse(format!(
                "scorecard summary requires non-negative integer {key}"
            ))
        })
    }

    pub fn decode_summary(
        value: Value,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionProtocolError> {
        let object = value.as_object().ok_or_else(|| {
            ArcAgi3CompetitionProtocolError::MalformedResponse(
                "scorecard summary must be an object".to_string(),
            )
        })?;

        let card_id = object
            .get("card_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ArcAgi3CompetitionProtocolError::MalformedResponse(
                    "scorecard summary requires string card_id".to_string(),
                )
            })?;

        let score = object.get("score").and_then(Value::as_f64).ok_or_else(|| {
            ArcAgi3CompetitionProtocolError::MalformedResponse(
                "scorecard summary requires numeric score".to_string(),
            )
        })?;

        if !score.is_finite() {
            return Err(ArcAgi3CompetitionProtocolError::NonFiniteScore);
        }

        let environments = object
            .get("environments")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                ArcAgi3CompetitionProtocolError::MalformedResponse(
                    "scorecard summary requires environments array".to_string(),
                )
            })?
            .clone();

        let competition_mode = match object.get("competition_mode") {
            None | Some(Value::Null) => None,
            Some(Value::Bool(value)) => Some(*value),
            Some(_) => {
                return Err(ArcAgi3CompetitionProtocolError::MalformedResponse(
                    "competition_mode must be boolean or null".to_string(),
                ));
            }
        };

        let published_at = match object.get("published_at") {
            None | Some(Value::Null) => None,
            Some(Value::String(value)) => Some(value.clone()),
            Some(_) => {
                return Err(ArcAgi3CompetitionProtocolError::MalformedResponse(
                    "published_at must be string or null".to_string(),
                ));
            }
        };

        Ok(ArcAgi3ScorecardSummary {
            card_id: ArcAgi3ScorecardId::new(card_id.to_string())?,
            score,
            environments,
            total_environments_completed: Self::required_u64(
                object,
                "total_environments_completed",
            )?,
            total_environments: Self::required_u64(object, "total_environments")?,
            total_levels_completed: Self::required_u64(object, "total_levels_completed")?,
            total_levels: Self::required_u64(object, "total_levels")?,
            total_actions: Self::required_u64(object, "total_actions")?,
            competition_mode,
            published_at,
            raw: value,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3CompetitionFailureDisposition {
    NotDispatched,
    RejectedByServer,
    DispatchIndeterminate,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArcAgi3CompetitionTransportError {
    InvalidConfiguration(String),
    HttpTransport {
        message: String,
        disposition: ArcAgi3CompetitionFailureDisposition,
    },
    HttpStatus {
        status: u16,
        body: String,
        disposition: ArcAgi3CompetitionFailureDisposition,
    },
    SuccessDecode {
        message: String,
        disposition: ArcAgi3CompetitionFailureDisposition,
    },
    Protocol(ArcAgi3CompetitionProtocolError),
}

impl ArcAgi3CompetitionTransportError {
    pub fn disposition(&self) -> Option<ArcAgi3CompetitionFailureDisposition> {
        match self {
            Self::HttpTransport { disposition, .. }
            | Self::HttpStatus { disposition, .. }
            | Self::SuccessDecode { disposition, .. } => Some(*disposition),

            Self::InvalidConfiguration(_) | Self::Protocol(_) => None,
        }
    }
}

impl From<ArcAgi3CompetitionProtocolError> for ArcAgi3CompetitionTransportError {
    fn from(value: ArcAgi3CompetitionProtocolError) -> Self {
        Self::Protocol(value)
    }
}

pub trait ArcAgi3ScorecardTransport {
    fn open_scorecard(
        &mut self,
        metadata: &ArcAgi3CompetitionMetadata,
    ) -> Result<ArcAgi3ScorecardId, ArcAgi3CompetitionTransportError>;

    fn get_scorecard(
        &mut self,
        card_id: &ArcAgi3ScorecardId,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError>;

    fn close_scorecard(
        &mut self,
        card_id: &ArcAgi3ScorecardId,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError>;
}

#[derive(Debug)]
pub struct ArcAgi3RestScorecardTransport {
    client: Client,
    base_url: Url,
    api_key: String,
}

impl ArcAgi3RestScorecardTransport {
    pub fn new(base_url: &str, api_key: &str) -> Result<Self, ArcAgi3CompetitionTransportError> {
        if api_key.trim().is_empty() {
            return Err(ArcAgi3CompetitionTransportError::InvalidConfiguration(
                "ARC-AGI-3 API key must not be empty".to_string(),
            ));
        }

        let mut base_url = Url::parse(base_url).map_err(|error| {
            ArcAgi3CompetitionTransportError::InvalidConfiguration(format!(
                "invalid ARC-AGI-3 base URL: {error}"
            ))
        })?;

        if !base_url.path().ends_with('/') {
            let mut path = base_url.path().to_string();

            path.push('/');
            base_url.set_path(&path);
        }

        let client = Client::builder()
            .cookie_store(true)
            .build()
            .map_err(|error| {
                ArcAgi3CompetitionTransportError::InvalidConfiguration(format!(
                    "failed to build scorecard HTTP client: {error}"
                ))
            })?;

        Ok(Self {
            client,
            base_url,
            api_key: api_key.to_string(),
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    fn endpoint(&self, path: &str) -> Result<Url, ArcAgi3CompetitionTransportError> {
        self.base_url.join(path).map_err(|error| {
            ArcAgi3CompetitionTransportError::InvalidConfiguration(format!(
                "failed to build scorecard endpoint: {error}"
            ))
        })
    }

    fn scorecard_endpoint(
        &self,
        card_id: &ArcAgi3ScorecardId,
    ) -> Result<Url, ArcAgi3CompetitionTransportError> {
        let mut url = self.base_url.clone();

        {
            let mut segments = url.path_segments_mut().map_err(|_| {
                ArcAgi3CompetitionTransportError::InvalidConfiguration(
                    "base URL cannot accept path segments".to_string(),
                )
            })?;

            segments.pop_if_empty();
            segments.push("api");
            segments.push("scorecard");
            segments.push(card_id.as_str());
        }

        Ok(url)
    }

    fn response_value(
        response: Response,
        side_effecting: bool,
    ) -> Result<Value, ArcAgi3CompetitionTransportError> {
        let status = response.status();

        let body =
            response
                .text()
                .map_err(|error| ArcAgi3CompetitionTransportError::HttpTransport {
                    message: error.to_string(),
                    disposition: if side_effecting {
                        ArcAgi3CompetitionFailureDisposition::DispatchIndeterminate
                    } else {
                        ArcAgi3CompetitionFailureDisposition::NotDispatched
                    },
                })?;

        if !status.is_success() {
            return Err(ArcAgi3CompetitionTransportError::HttpStatus {
                status: status.as_u16(),
                body,
                disposition: if status.is_client_error() {
                    ArcAgi3CompetitionFailureDisposition::RejectedByServer
                } else if side_effecting {
                    ArcAgi3CompetitionFailureDisposition::DispatchIndeterminate
                } else {
                    ArcAgi3CompetitionFailureDisposition::RejectedByServer
                },
            });
        }

        serde_json::from_str(&body).map_err(|error| {
            ArcAgi3CompetitionTransportError::SuccessDecode {
                message: error.to_string(),
                disposition: if side_effecting {
                    ArcAgi3CompetitionFailureDisposition::DispatchIndeterminate
                } else {
                    ArcAgi3CompetitionFailureDisposition::NotDispatched
                },
            }
        })
    }

    fn post(&self, url: Url, body: &Value) -> Result<Value, ArcAgi3CompetitionTransportError> {
        let response = self
            .client
            .post(url)
            .header("X-API-Key", &self.api_key)
            .header(ACCEPT, "application/json")
            .json(body)
            .send()
            .map_err(|error| ArcAgi3CompetitionTransportError::HttpTransport {
                message: error.to_string(),
                disposition: ArcAgi3CompetitionFailureDisposition::DispatchIndeterminate,
            })?;

        Self::response_value(response, true)
    }

    fn get(&self, url: Url) -> Result<Value, ArcAgi3CompetitionTransportError> {
        let response = self
            .client
            .get(url)
            .header("X-API-Key", &self.api_key)
            .header(ACCEPT, "application/json")
            .send()
            .map_err(|error| ArcAgi3CompetitionTransportError::HttpTransport {
                message: error.to_string(),
                disposition: ArcAgi3CompetitionFailureDisposition::NotDispatched,
            })?;

        Self::response_value(response, false)
    }
}

impl ArcAgi3ScorecardTransport for ArcAgi3RestScorecardTransport {
    fn open_scorecard(
        &mut self,
        metadata: &ArcAgi3CompetitionMetadata,
    ) -> Result<ArcAgi3ScorecardId, ArcAgi3CompetitionTransportError> {
        let url = self.endpoint(ArcAgi3ScorecardRestProtocol::OPEN_PATH)?;

        let value = self.post(url, &ArcAgi3ScorecardRestProtocol::open_request(metadata))?;

        ArcAgi3ScorecardRestProtocol::decode_open_response(value).map_err(Into::into)
    }

    fn get_scorecard(
        &mut self,
        card_id: &ArcAgi3ScorecardId,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError> {
        let url = self.scorecard_endpoint(card_id)?;

        let value = self.get(url)?;

        ArcAgi3ScorecardRestProtocol::decode_summary(value).map_err(Into::into)
    }

    fn close_scorecard(
        &mut self,
        card_id: &ArcAgi3ScorecardId,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionTransportError> {
        let url = self.endpoint(ArcAgi3ScorecardRestProtocol::CLOSE_PATH)?;

        let value = self.post(url, &ArcAgi3ScorecardRestProtocol::close_request(card_id))?;

        ArcAgi3ScorecardRestProtocol::decode_summary(value).map_err(Into::into)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3CompetitionSessionStatus {
    Open,
    CloseFaulted,
    Closed,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ArcAgi3CompetitionSessionError {
    ScorecardTransport(ArcAgi3CompetitionTransportError),
    SessionNotOpen(ArcAgi3CompetitionSessionStatus),
    CardIdentityMismatch { expected: String, observed: String },
    CompetitionModeMismatch,
    LiveGameStart(ArcAgi3LiveEnvironmentError),
    CloseStillUnresolved,
}

impl From<ArcAgi3CompetitionTransportError> for ArcAgi3CompetitionSessionError {
    fn from(value: ArcAgi3CompetitionTransportError) -> Self {
        Self::ScorecardTransport(value)
    }
}

#[derive(Debug)]
pub struct ArcAgi3CompetitionSession<S>
where
    S: ArcAgi3ScorecardTransport,
{
    scorecard_transport: S,
    card_id: ArcAgi3ScorecardId,
    status: ArcAgi3CompetitionSessionStatus,
    close_failure_disposition: Option<ArcAgi3CompetitionFailureDisposition>,
    final_summary: Option<ArcAgi3ScorecardSummary>,
}

impl<S> ArcAgi3CompetitionSession<S>
where
    S: ArcAgi3ScorecardTransport,
{
    pub fn open(
        mut scorecard_transport: S,
        metadata: &ArcAgi3CompetitionMetadata,
    ) -> Result<Self, ArcAgi3CompetitionSessionError> {
        let card_id = scorecard_transport.open_scorecard(metadata)?;

        Ok(Self {
            scorecard_transport,
            card_id,
            status: ArcAgi3CompetitionSessionStatus::Open,
            close_failure_disposition: None,
            final_summary: None,
        })
    }

    pub fn card_id(&self) -> &ArcAgi3ScorecardId {
        &self.card_id
    }

    pub fn status(&self) -> ArcAgi3CompetitionSessionStatus {
        self.status
    }

    pub fn close_failure_disposition(&self) -> Option<ArcAgi3CompetitionFailureDisposition> {
        self.close_failure_disposition
    }

    pub fn final_summary(&self) -> Option<&ArcAgi3ScorecardSummary> {
        self.final_summary.as_ref()
    }

    pub fn scorecard_transport(&self) -> &S {
        &self.scorecard_transport
    }

    fn ensure_open(&self) -> Result<(), ArcAgi3CompetitionSessionError> {
        if self.status != ArcAgi3CompetitionSessionStatus::Open {
            return Err(ArcAgi3CompetitionSessionError::SessionNotOpen(self.status));
        }

        Ok(())
    }

    fn validate_summary(
        &self,
        summary: &ArcAgi3ScorecardSummary,
    ) -> Result<(), ArcAgi3CompetitionSessionError> {
        if summary.card_id() != &self.card_id {
            return Err(ArcAgi3CompetitionSessionError::CardIdentityMismatch {
                expected: self.card_id.as_str().to_string(),
                observed: summary.card_id().as_str().to_string(),
            });
        }

        if summary.competition_mode() == Some(false) {
            return Err(ArcAgi3CompetitionSessionError::CompetitionModeMismatch);
        }

        Ok(())
    }

    pub fn poll(&mut self) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionSessionError> {
        if self.status == ArcAgi3CompetitionSessionStatus::Closed {
            return self
                .final_summary
                .clone()
                .ok_or(ArcAgi3CompetitionSessionError::CloseStillUnresolved);
        }

        let summary = self.scorecard_transport.get_scorecard(&self.card_id)?;

        self.validate_summary(&summary)?;

        if summary.published_at().is_some() {
            self.status = ArcAgi3CompetitionSessionStatus::Closed;

            self.close_failure_disposition = None;

            self.final_summary = Some(summary.clone());
        } else if self.status == ArcAgi3CompetitionSessionStatus::CloseFaulted {
            return Err(ArcAgi3CompetitionSessionError::CloseStillUnresolved);
        }

        Ok(summary)
    }

    pub fn start_game<'a, E>(
        &'a mut self,
        environment_transport: E,
        game_id: &ArcAgi3GameId,
        first_perceptual_index: u64,
    ) -> Result<ArcAgi3CompetitionGame<'a, S, E>, ArcAgi3CompetitionSessionError>
    where
        E: ArcAgi3EnvironmentTransport,
    {
        self.ensure_open()?;

        let runtime = ArcAgi3LiveEnvironmentRuntime::start(
            environment_transport,
            game_id,
            self.card_id.as_str(),
            first_perceptual_index,
        )
        .map_err(ArcAgi3CompetitionSessionError::LiveGameStart)?;

        Ok(ArcAgi3CompetitionGame {
            session: self,
            runtime,
        })
    }

    pub fn close(&mut self) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionSessionError> {
        self.ensure_open()?;

        let summary = match self.scorecard_transport.close_scorecard(&self.card_id) {
            Ok(summary) => summary,

            Err(error) => {
                self.status = ArcAgi3CompetitionSessionStatus::CloseFaulted;

                self.close_failure_disposition = error.disposition();

                return Err(ArcAgi3CompetitionSessionError::ScorecardTransport(error));
            }
        };

        if let Err(error) = self.validate_summary(&summary) {
            self.status = ArcAgi3CompetitionSessionStatus::CloseFaulted;

            self.close_failure_disposition = None;

            return Err(error);
        }

        self.status = ArcAgi3CompetitionSessionStatus::Closed;

        self.close_failure_disposition = None;

        self.final_summary = Some(summary.clone());

        Ok(summary)
    }

    pub fn reconcile_close_failure(
        &mut self,
    ) -> Result<ArcAgi3ScorecardSummary, ArcAgi3CompetitionSessionError> {
        if self.status != ArcAgi3CompetitionSessionStatus::CloseFaulted {
            return Err(ArcAgi3CompetitionSessionError::SessionNotOpen(self.status));
        }

        self.poll()
    }

    pub fn into_parts(self) -> (S, ArcAgi3ScorecardId, Option<ArcAgi3ScorecardSummary>) {
        (self.scorecard_transport, self.card_id, self.final_summary)
    }
}

pub struct ArcAgi3CompetitionGame<'a, S, E>
where
    S: ArcAgi3ScorecardTransport,
    E: ArcAgi3EnvironmentTransport,
{
    session: &'a mut ArcAgi3CompetitionSession<S>,
    runtime: ArcAgi3LiveEnvironmentRuntime<E>,
}

impl<'a, S, E> ArcAgi3CompetitionGame<'a, S, E>
where
    S: ArcAgi3ScorecardTransport,
    E: ArcAgi3EnvironmentTransport,
{
    pub fn card_id(&self) -> &ArcAgi3ScorecardId {
        self.session.card_id()
    }

    pub fn runtime(&self) -> &ArcAgi3LiveEnvironmentRuntime<E> {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut ArcAgi3LiveEnvironmentRuntime<E> {
        &mut self.runtime
    }

    pub fn run_bounded_with<F>(
        &mut self,
        policy: ArcAgi3BoundedEpisodePolicy,
        execute_step: F,
    ) -> Result<ArcAgi3BoundedEpisodeResult, ArcAgi3BoundedEpisodeError>
    where
        F: FnMut(
            &mut ArcAgi3LiveEnvironmentRuntime<E>,
        ) -> Result<ArcAgi3LiveCognitiveStep, ArcAgi3LiveEnvironmentError>,
    {
        ArcAgi3BoundedEpisodeRuntime::run_with(&mut self.runtime, policy, execute_step)
    }

    pub fn finish(self) -> ArcAgi3LiveEnvironmentRuntime<E> {
        self.runtime
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3CompetitionSessionRuntime;

impl UniversalArcAgi3CompetitionSessionRuntime {
    pub fn open<S>(
        scorecard_transport: S,
        metadata: &ArcAgi3CompetitionMetadata,
    ) -> Result<ArcAgi3CompetitionSession<S>, ArcAgi3CompetitionSessionError>
    where
        S: ArcAgi3ScorecardTransport,
    {
        ArcAgi3CompetitionSession::open(scorecard_transport, metadata)
    }
}
