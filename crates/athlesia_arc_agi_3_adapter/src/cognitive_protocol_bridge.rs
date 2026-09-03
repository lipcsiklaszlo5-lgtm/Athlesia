use crate::{
    ArcAgi3Action, ArcAgi3ActionAuthorization, ArcAgi3ActionId, ArcAgi3ActionPayload,
    ArcAgi3AvailableActions, ArcAgi3Coordinate, ArcAgi3FrameSequence, ArcAgi3GameId,
    ArcAgi3GameState, ArcAgi3Grid, ArcAgi3Observation, ArcAgi3Protocol,
};
use athlesia_integrated_cognitive_agent::{
    EnvironmentActionDispatch, EnvironmentInteractionBoundary, EnvironmentInteractionEvidence,
    EnvironmentInteractionObservation,
};
use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};

const TAG_ACTION: u64 = 0xA352_0001;
const TAG_PAYLOAD_NONE: u64 = 0xA352_0002;
const TAG_PAYLOAD_COORDINATE: u64 = 0xA352_0003;
const TAG_OBSERVATION: u64 = 0xA352_0010;
const TAG_GAME_ID: u64 = 0xA352_0011;
const TAG_FRAMES: u64 = 0xA352_0012;
const TAG_GRID: u64 = 0xA352_0013;
const TAG_AVAILABLE_ACTIONS: u64 = 0xA352_0014;
const TAG_LAST_ACTION_NONE: u64 = 0xA352_0015;
const TAG_LAST_ACTION_SOME: u64 = 0xA352_0016;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3CognitiveCodecError {
    ExpectedOrderedStructure,
    ExpectedAtom,
    UnexpectedTag,
    InvalidFieldCount,
    IntegerOutOfRange,
    InvalidUtf8,
    InvalidGameId,
    InvalidGameState,
    InvalidActionId,
    InvalidActionPayload,
    InvalidAction,
    InvalidGrid,
    InvalidFrameSequence,
    InvalidAvailableActions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3CognitiveBridgeError {
    Codec(ArcAgi3CognitiveCodecError),
    InvalidConfidence,
    EvidenceRejected,
}

impl From<ArcAgi3CognitiveCodecError> for ArcAgi3CognitiveBridgeError {
    fn from(value: ArcAgi3CognitiveCodecError) -> Self {
        Self::Codec(value)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3CognitiveProtocolBridge;

impl ArcAgi3CognitiveProtocolBridge {
    fn tagged(tag: u64, fields: Vec<CognitiveStructure>) -> CognitiveStructure {
        let mut children = Vec::with_capacity(fields.len().saturating_add(1));
        children.push(CognitiveStructure::atom(tag));
        children.extend(fields);
        CognitiveStructure::Ordered(children)
    }

    fn atom_value(structure: &CognitiveStructure) -> Result<u64, ArcAgi3CognitiveCodecError> {
        match structure {
            CognitiveStructure::Atom(value) => Ok(*value),
            CognitiveStructure::Ordered(_) | CognitiveStructure::Unordered(_) => {
                Err(ArcAgi3CognitiveCodecError::ExpectedAtom)
            }
        }
    }

    fn tagged_fields(
        structure: &CognitiveStructure,
        expected_tag: u64,
    ) -> Result<&[CognitiveStructure], ArcAgi3CognitiveCodecError> {
        let CognitiveStructure::Ordered(children) = structure else {
            return Err(ArcAgi3CognitiveCodecError::ExpectedOrderedStructure);
        };

        let Some((tag, fields)) = children.split_first() else {
            return Err(ArcAgi3CognitiveCodecError::InvalidFieldCount);
        };

        if Self::atom_value(tag)? != expected_tag {
            return Err(ArcAgi3CognitiveCodecError::UnexpectedTag);
        }

        Ok(fields)
    }

    fn action_id_code(id: ArcAgi3ActionId) -> u64 {
        match id {
            ArcAgi3ActionId::Action1 => 1,
            ArcAgi3ActionId::Action2 => 2,
            ArcAgi3ActionId::Action3 => 3,
            ArcAgi3ActionId::Action4 => 4,
            ArcAgi3ActionId::Action5 => 5,
            ArcAgi3ActionId::Action6 => 6,
            ArcAgi3ActionId::Action7 => 7,
            ArcAgi3ActionId::Reset => 8,
        }
    }

    fn decode_action_id(code: u64) -> Result<ArcAgi3ActionId, ArcAgi3CognitiveCodecError> {
        match code {
            1 => Ok(ArcAgi3ActionId::Action1),
            2 => Ok(ArcAgi3ActionId::Action2),
            3 => Ok(ArcAgi3ActionId::Action3),
            4 => Ok(ArcAgi3ActionId::Action4),
            5 => Ok(ArcAgi3ActionId::Action5),
            6 => Ok(ArcAgi3ActionId::Action6),
            7 => Ok(ArcAgi3ActionId::Action7),
            8 => Ok(ArcAgi3ActionId::Reset),
            _ => Err(ArcAgi3CognitiveCodecError::InvalidActionId),
        }
    }

    fn encode_payload(payload: ArcAgi3ActionPayload) -> CognitiveStructure {
        match payload {
            ArcAgi3ActionPayload::None => Self::tagged(TAG_PAYLOAD_NONE, Vec::new()),
            ArcAgi3ActionPayload::Coordinate(coordinate) => Self::tagged(
                TAG_PAYLOAD_COORDINATE,
                vec![
                    CognitiveStructure::atom(u64::from(coordinate.x())),
                    CognitiveStructure::atom(u64::from(coordinate.y())),
                ],
            ),
        }
    }

    fn decode_payload(
        structure: &CognitiveStructure,
    ) -> Result<ArcAgi3ActionPayload, ArcAgi3CognitiveCodecError> {
        match structure {
            CognitiveStructure::Ordered(children) => {
                let Some((tag_structure, fields)) = children.split_first() else {
                    return Err(ArcAgi3CognitiveCodecError::InvalidActionPayload);
                };

                let tag = Self::atom_value(tag_structure)?;

                match tag {
                    TAG_PAYLOAD_NONE => {
                        if !fields.is_empty() {
                            return Err(ArcAgi3CognitiveCodecError::InvalidActionPayload);
                        }

                        Ok(ArcAgi3ActionPayload::None)
                    }
                    TAG_PAYLOAD_COORDINATE => {
                        if fields.len() != 2 {
                            return Err(ArcAgi3CognitiveCodecError::InvalidActionPayload);
                        }

                        let x = u8::try_from(Self::atom_value(&fields[0])?)
                            .map_err(|_| ArcAgi3CognitiveCodecError::IntegerOutOfRange)?;

                        let y = u8::try_from(Self::atom_value(&fields[1])?)
                            .map_err(|_| ArcAgi3CognitiveCodecError::IntegerOutOfRange)?;

                        let coordinate = ArcAgi3Coordinate::new(x, y)
                            .ok_or(ArcAgi3CognitiveCodecError::InvalidActionPayload)?;

                        Ok(ArcAgi3ActionPayload::Coordinate(coordinate))
                    }
                    _ => Err(ArcAgi3CognitiveCodecError::UnexpectedTag),
                }
            }
            CognitiveStructure::Atom(_) | CognitiveStructure::Unordered(_) => {
                Err(ArcAgi3CognitiveCodecError::ExpectedOrderedStructure)
            }
        }
    }

    pub fn encode_action(action: ArcAgi3Action) -> CognitiveStructure {
        Self::tagged(
            TAG_ACTION,
            vec![
                CognitiveStructure::atom(Self::action_id_code(action.id())),
                Self::encode_payload(action.payload()),
            ],
        )
    }

    pub fn decode_action(
        structure: &CognitiveStructure,
    ) -> Result<ArcAgi3Action, ArcAgi3CognitiveCodecError> {
        let fields = Self::tagged_fields(structure, TAG_ACTION)?;

        if fields.len() != 2 {
            return Err(ArcAgi3CognitiveCodecError::InvalidFieldCount);
        }

        let id = Self::decode_action_id(Self::atom_value(&fields[0])?)?;
        let payload = Self::decode_payload(&fields[1])?;

        ArcAgi3Action::from_parts(id, payload).ok_or(ArcAgi3CognitiveCodecError::InvalidAction)
    }

    fn encode_game_id(game_id: &ArcAgi3GameId) -> CognitiveStructure {
        let bytes = game_id.as_str().as_bytes();

        let mut fields = Vec::with_capacity(bytes.len().saturating_add(1));
        fields.push(CognitiveStructure::atom(bytes.len() as u64));

        fields.extend(
            bytes
                .iter()
                .map(|byte| CognitiveStructure::atom(u64::from(*byte))),
        );

        Self::tagged(TAG_GAME_ID, fields)
    }

    fn decode_game_id(
        structure: &CognitiveStructure,
    ) -> Result<ArcAgi3GameId, ArcAgi3CognitiveCodecError> {
        let fields = Self::tagged_fields(structure, TAG_GAME_ID)?;

        let Some((length_structure, byte_structures)) = fields.split_first() else {
            return Err(ArcAgi3CognitiveCodecError::InvalidFieldCount);
        };

        let expected_length = usize::try_from(Self::atom_value(length_structure)?)
            .map_err(|_| ArcAgi3CognitiveCodecError::IntegerOutOfRange)?;

        if expected_length != byte_structures.len() {
            return Err(ArcAgi3CognitiveCodecError::InvalidFieldCount);
        }

        let bytes = byte_structures
            .iter()
            .map(|structure| {
                u8::try_from(Self::atom_value(structure)?)
                    .map_err(|_| ArcAgi3CognitiveCodecError::IntegerOutOfRange)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let value =
            String::from_utf8(bytes).map_err(|_| ArcAgi3CognitiveCodecError::InvalidUtf8)?;

        ArcAgi3GameId::new(value).ok_or(ArcAgi3CognitiveCodecError::InvalidGameId)
    }

    fn game_state_code(state: ArcAgi3GameState) -> u64 {
        match state {
            ArcAgi3GameState::NotPlayed => 0,
            ArcAgi3GameState::NotFinished => 1,
            ArcAgi3GameState::Win => 2,
            ArcAgi3GameState::GameOver => 3,
        }
    }

    fn decode_game_state(code: u64) -> Result<ArcAgi3GameState, ArcAgi3CognitiveCodecError> {
        match code {
            0 => Ok(ArcAgi3GameState::NotPlayed),
            1 => Ok(ArcAgi3GameState::NotFinished),
            2 => Ok(ArcAgi3GameState::Win),
            3 => Ok(ArcAgi3GameState::GameOver),
            _ => Err(ArcAgi3CognitiveCodecError::InvalidGameState),
        }
    }

    fn encode_grid(grid: &ArcAgi3Grid) -> CognitiveStructure {
        let mut fields = Vec::with_capacity(grid.cells().len().saturating_add(2));

        fields.push(CognitiveStructure::atom(grid.width() as u64));
        fields.push(CognitiveStructure::atom(grid.height() as u64));

        fields.extend(
            grid.cells()
                .iter()
                .map(|cell| CognitiveStructure::atom(u64::from(*cell))),
        );

        Self::tagged(TAG_GRID, fields)
    }

    fn decode_grid(
        structure: &CognitiveStructure,
    ) -> Result<ArcAgi3Grid, ArcAgi3CognitiveCodecError> {
        let fields = Self::tagged_fields(structure, TAG_GRID)?;

        if fields.len() < 2 {
            return Err(ArcAgi3CognitiveCodecError::InvalidFieldCount);
        }

        let width = usize::try_from(Self::atom_value(&fields[0])?)
            .map_err(|_| ArcAgi3CognitiveCodecError::IntegerOutOfRange)?;

        let height = usize::try_from(Self::atom_value(&fields[1])?)
            .map_err(|_| ArcAgi3CognitiveCodecError::IntegerOutOfRange)?;

        let cells = fields[2..]
            .iter()
            .map(|structure| {
                u8::try_from(Self::atom_value(structure)?)
                    .map_err(|_| ArcAgi3CognitiveCodecError::IntegerOutOfRange)
            })
            .collect::<Result<Vec<_>, _>>()?;

        ArcAgi3Grid::from_flat(width, height, cells).ok_or(ArcAgi3CognitiveCodecError::InvalidGrid)
    }

    fn encode_frames(frames: &ArcAgi3FrameSequence) -> CognitiveStructure {
        Self::tagged(
            TAG_FRAMES,
            frames.frames().iter().map(Self::encode_grid).collect(),
        )
    }

    fn decode_frames(
        structure: &CognitiveStructure,
    ) -> Result<ArcAgi3FrameSequence, ArcAgi3CognitiveCodecError> {
        let fields = Self::tagged_fields(structure, TAG_FRAMES)?;

        let frames = fields
            .iter()
            .map(Self::decode_grid)
            .collect::<Result<Vec<_>, _>>()?;

        ArcAgi3FrameSequence::new(frames).ok_or(ArcAgi3CognitiveCodecError::InvalidFrameSequence)
    }

    fn encode_available_actions(actions: &ArcAgi3AvailableActions) -> CognitiveStructure {
        Self::tagged(
            TAG_AVAILABLE_ACTIONS,
            actions
                .actions()
                .iter()
                .map(|action| CognitiveStructure::atom(Self::action_id_code(*action)))
                .collect(),
        )
    }

    fn decode_available_actions(
        structure: &CognitiveStructure,
    ) -> Result<ArcAgi3AvailableActions, ArcAgi3CognitiveCodecError> {
        let fields = Self::tagged_fields(structure, TAG_AVAILABLE_ACTIONS)?;

        let actions = fields
            .iter()
            .map(|structure| Self::decode_action_id(Self::atom_value(structure)?))
            .collect::<Result<Vec<_>, _>>()?;

        ArcAgi3AvailableActions::new(actions)
            .ok_or(ArcAgi3CognitiveCodecError::InvalidAvailableActions)
    }

    fn encode_last_action(action: Option<ArcAgi3Action>) -> CognitiveStructure {
        match action {
            None => Self::tagged(TAG_LAST_ACTION_NONE, Vec::new()),
            Some(action) => Self::tagged(TAG_LAST_ACTION_SOME, vec![Self::encode_action(action)]),
        }
    }

    fn decode_last_action(
        structure: &CognitiveStructure,
    ) -> Result<Option<ArcAgi3Action>, ArcAgi3CognitiveCodecError> {
        let CognitiveStructure::Ordered(children) = structure else {
            return Err(ArcAgi3CognitiveCodecError::ExpectedOrderedStructure);
        };

        let Some((tag_structure, fields)) = children.split_first() else {
            return Err(ArcAgi3CognitiveCodecError::InvalidFieldCount);
        };

        let tag = Self::atom_value(tag_structure)?;

        match tag {
            TAG_LAST_ACTION_NONE => {
                if !fields.is_empty() {
                    return Err(ArcAgi3CognitiveCodecError::InvalidFieldCount);
                }

                Ok(None)
            }
            TAG_LAST_ACTION_SOME => {
                if fields.len() != 1 {
                    return Err(ArcAgi3CognitiveCodecError::InvalidFieldCount);
                }

                Ok(Some(Self::decode_action(&fields[0])?))
            }
            _ => Err(ArcAgi3CognitiveCodecError::UnexpectedTag),
        }
    }

    pub fn encode_observation(observation: &ArcAgi3Observation) -> CognitiveStructure {
        Self::tagged(
            TAG_OBSERVATION,
            vec![
                Self::encode_game_id(observation.game_id()),
                CognitiveStructure::atom(Self::game_state_code(observation.state())),
                Self::encode_frames(observation.frames()),
                CognitiveStructure::atom(u64::from(observation.levels_completed())),
                CognitiveStructure::atom(u64::from(observation.win_levels())),
                Self::encode_available_actions(observation.available_actions()),
                Self::encode_last_action(observation.last_action()),
            ],
        )
    }

    pub fn decode_observation(
        structure: &CognitiveStructure,
    ) -> Result<ArcAgi3Observation, ArcAgi3CognitiveCodecError> {
        let fields = Self::tagged_fields(structure, TAG_OBSERVATION)?;

        if fields.len() != 7 {
            return Err(ArcAgi3CognitiveCodecError::InvalidFieldCount);
        }

        let game_id = Self::decode_game_id(&fields[0])?;
        let state = Self::decode_game_state(Self::atom_value(&fields[1])?)?;
        let frames = Self::decode_frames(&fields[2])?;

        let levels_completed = u32::try_from(Self::atom_value(&fields[3])?)
            .map_err(|_| ArcAgi3CognitiveCodecError::IntegerOutOfRange)?;

        let win_levels = u32::try_from(Self::atom_value(&fields[4])?)
            .map_err(|_| ArcAgi3CognitiveCodecError::IntegerOutOfRange)?;

        let available_actions = Self::decode_available_actions(&fields[5])?;
        let last_action = Self::decode_last_action(&fields[6])?;

        Ok(ArcAgi3Observation::new(
            game_id,
            state,
            frames,
            levels_completed,
            win_levels,
            available_actions,
            last_action,
        ))
    }

    pub fn authorize_cognitive_action(
        observation: &ArcAgi3Observation,
        encoded_action: &CognitiveStructure,
    ) -> Result<ArcAgi3ActionAuthorization, ArcAgi3CognitiveCodecError> {
        let action = Self::decode_action(encoded_action)?;
        Ok(ArcAgi3Protocol::authorize_action(observation, action))
    }

    pub fn decode_dispatch(
        dispatch: &EnvironmentActionDispatch,
    ) -> Result<ArcAgi3Action, ArcAgi3CognitiveCodecError> {
        Self::decode_action(dispatch.action())
    }

    pub fn environment_observation(
        event_index: u64,
        observation: &ArcAgi3Observation,
        confidence: CognitiveSignal,
    ) -> Result<EnvironmentInteractionObservation, ArcAgi3CognitiveBridgeError> {
        EnvironmentInteractionObservation::new(
            event_index,
            Self::encode_observation(observation),
            confidence,
        )
        .ok_or(ArcAgi3CognitiveBridgeError::InvalidConfidence)
    }

    pub fn bind_feedback(
        dispatch: &EnvironmentActionDispatch,
        event_index: u64,
        observation: &ArcAgi3Observation,
        confidence: CognitiveSignal,
    ) -> Result<EnvironmentInteractionEvidence, ArcAgi3CognitiveBridgeError> {
        let environment_observation =
            Self::environment_observation(event_index, observation, confidence)?;

        EnvironmentInteractionBoundary::bind_observation(dispatch, &environment_observation)
            .ok_or(ArcAgi3CognitiveBridgeError::EvidenceRejected)
    }
}
