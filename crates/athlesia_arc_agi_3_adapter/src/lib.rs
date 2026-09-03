// ============================================================================
// ATHLESIA M52 — ARC-AGI-3 ADAPTER FOUNDATION
// ============================================================================

pub const ARC_AGI_3_MAX_GRID_DIMENSION: usize = 64;
pub const ARC_AGI_3_MAX_CELL_VALUE: u8 = 15;
pub const ARC_AGI_3_MAX_COORDINATE: u8 = 63;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArcAgi3GameState {
    NotPlayed,
    NotFinished,
    Win,
    GameOver,
}

impl ArcAgi3GameState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Win | Self::GameOver)
    }

    pub fn awaiting_action(self) -> bool {
        self == Self::NotFinished
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArcAgi3ActionId {
    Action1,
    Action2,
    Action3,
    Action4,
    Action5,
    Action6,
    Action7,
    Reset,
}

impl ArcAgi3ActionId {
    pub fn protocol_name(self) -> &'static str {
        match self {
            Self::Action1 => "ACTION1",
            Self::Action2 => "ACTION2",
            Self::Action3 => "ACTION3",
            Self::Action4 => "ACTION4",
            Self::Action5 => "ACTION5",
            Self::Action6 => "ACTION6",
            Self::Action7 => "ACTION7",
            Self::Reset => "RESET",
        }
    }

    pub fn from_protocol_name(name: &str) -> Option<Self> {
        match name {
            "ACTION1" => Some(Self::Action1),
            "ACTION2" => Some(Self::Action2),
            "ACTION3" => Some(Self::Action3),
            "ACTION4" => Some(Self::Action4),
            "ACTION5" => Some(Self::Action5),
            "ACTION6" => Some(Self::Action6),
            "ACTION7" => Some(Self::Action7),
            "RESET" => Some(Self::Reset),
            _ => None,
        }
    }

    pub fn numeric_id(self) -> Option<u8> {
        match self {
            Self::Action1 => Some(1),
            Self::Action2 => Some(2),
            Self::Action3 => Some(3),
            Self::Action4 => Some(4),
            Self::Action5 => Some(5),
            Self::Action6 => Some(6),
            Self::Action7 => Some(7),
            Self::Reset => None,
        }
    }

    pub fn from_numeric_id(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Action1),
            2 => Some(Self::Action2),
            3 => Some(Self::Action3),
            4 => Some(Self::Action4),
            5 => Some(Self::Action5),
            6 => Some(Self::Action6),
            7 => Some(Self::Action7),
            _ => None,
        }
    }

    pub fn requires_coordinate(self) -> bool {
        self == Self::Action6
    }

    pub fn is_environment_action(self) -> bool {
        self != Self::Reset
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ArcAgi3Coordinate {
    x: u8,
    y: u8,
}

impl ArcAgi3Coordinate {
    pub fn new(x: u8, y: u8) -> Option<Self> {
        if x > ARC_AGI_3_MAX_COORDINATE || y > ARC_AGI_3_MAX_COORDINATE {
            return None;
        }

        Some(Self { x, y })
    }

    pub fn x(self) -> u8 {
        self.x
    }

    pub fn y(self) -> u8 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ArcAgi3ActionPayload {
    None,
    Coordinate(ArcAgi3Coordinate),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ArcAgi3Action {
    id: ArcAgi3ActionId,
    payload: ArcAgi3ActionPayload,
}

impl ArcAgi3Action {
    pub fn from_parts(id: ArcAgi3ActionId, payload: ArcAgi3ActionPayload) -> Option<Self> {
        let valid = matches!(
            (id, payload),
            (
                ArcAgi3ActionId::Action6,
                ArcAgi3ActionPayload::Coordinate(_),
            ) | (
                ArcAgi3ActionId::Action1
                    | ArcAgi3ActionId::Action2
                    | ArcAgi3ActionId::Action3
                    | ArcAgi3ActionId::Action4
                    | ArcAgi3ActionId::Action5
                    | ArcAgi3ActionId::Action7
                    | ArcAgi3ActionId::Reset,
                ArcAgi3ActionPayload::None,
            )
        );

        valid.then_some(Self { id, payload })
    }

    pub fn discrete(id: ArcAgi3ActionId) -> Option<Self> {
        if !id.is_environment_action() || id.requires_coordinate() {
            return None;
        }

        Self::from_parts(id, ArcAgi3ActionPayload::None)
    }

    pub fn coordinate(x: u8, y: u8) -> Option<Self> {
        let coordinate = ArcAgi3Coordinate::new(x, y)?;

        Self::from_parts(
            ArcAgi3ActionId::Action6,
            ArcAgi3ActionPayload::Coordinate(coordinate),
        )
    }

    pub fn reset() -> Self {
        Self {
            id: ArcAgi3ActionId::Reset,
            payload: ArcAgi3ActionPayload::None,
        }
    }

    pub fn id(self) -> ArcAgi3ActionId {
        self.id
    }

    pub fn payload(self) -> ArcAgi3ActionPayload {
        self.payload
    }

    pub fn coordinate_data(self) -> Option<ArcAgi3Coordinate> {
        match self.payload {
            ArcAgi3ActionPayload::Coordinate(coordinate) => Some(coordinate),
            ArcAgi3ActionPayload::None => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3AvailableActions {
    actions: Vec<ArcAgi3ActionId>,
}

impl ArcAgi3AvailableActions {
    pub fn new(mut actions: Vec<ArcAgi3ActionId>) -> Option<Self> {
        if actions.contains(&ArcAgi3ActionId::Reset) {
            return None;
        }

        actions.sort_unstable();

        if actions.windows(2).any(|pair| pair[0] == pair[1]) {
            return None;
        }

        Some(Self { actions })
    }

    pub fn actions(&self) -> &[ArcAgi3ActionId] {
        &self.actions
    }

    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    pub fn contains(&self, action: ArcAgi3ActionId) -> bool {
        self.actions.binary_search(&action).is_ok()
    }

    pub fn authorizes(&self, action: ArcAgi3Action) -> bool {
        action.id().is_environment_action() && self.contains(action.id())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3Grid {
    width: usize,
    height: usize,
    cells: Vec<u8>,
}

impl ArcAgi3Grid {
    pub fn from_rows(rows: Vec<Vec<u8>>) -> Option<Self> {
        let height = rows.len();

        if height == 0 || height > ARC_AGI_3_MAX_GRID_DIMENSION {
            return None;
        }

        let width = rows.first()?.len();

        if width == 0 || width > ARC_AGI_3_MAX_GRID_DIMENSION {
            return None;
        }

        if rows.iter().any(|row| row.len() != width) {
            return None;
        }

        let mut cells = Vec::with_capacity(width.saturating_mul(height));

        for row in rows {
            for cell in row {
                if cell > ARC_AGI_3_MAX_CELL_VALUE {
                    return None;
                }

                cells.push(cell);
            }
        }

        Some(Self {
            width,
            height,
            cells,
        })
    }

    pub fn from_flat(width: usize, height: usize, cells: Vec<u8>) -> Option<Self> {
        if width == 0
            || height == 0
            || width > ARC_AGI_3_MAX_GRID_DIMENSION
            || height > ARC_AGI_3_MAX_GRID_DIMENSION
        {
            return None;
        }

        if width.checked_mul(height)? != cells.len() {
            return None;
        }

        if cells.iter().any(|cell| *cell > ARC_AGI_3_MAX_CELL_VALUE) {
            return None;
        }

        Some(Self {
            width,
            height,
            cells,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cells(&self) -> &[u8] {
        &self.cells
    }

    pub fn cell(&self, x: usize, y: usize) -> Option<u8> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let index = y.checked_mul(self.width)?.checked_add(x)?;

        self.cells.get(index).copied()
    }

    pub fn row(&self, y: usize) -> Option<&[u8]> {
        if y >= self.height {
            return None;
        }

        let start = y.checked_mul(self.width)?;
        let end = start.checked_add(self.width)?;

        self.cells.get(start..end)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3FrameSequence {
    frames: Vec<ArcAgi3Grid>,
}

impl ArcAgi3FrameSequence {
    pub fn new(frames: Vec<ArcAgi3Grid>) -> Option<Self> {
        if frames.is_empty() {
            return None;
        }

        Some(Self { frames })
    }

    pub fn frames(&self) -> &[ArcAgi3Grid] {
        &self.frames
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn latest(&self) -> &ArcAgi3Grid {
        self.frames
            .last()
            .expect("validated ARC-AGI-3 frame sequence is nonempty")
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ArcAgi3GameId(String);

impl ArcAgi3GameId {
    pub fn new(value: String) -> Option<Self> {
        if value.is_empty() || value.chars().any(char::is_whitespace) {
            return None;
        }

        Some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArcAgi3Observation {
    game_id: ArcAgi3GameId,
    state: ArcAgi3GameState,
    frames: ArcAgi3FrameSequence,
    levels_completed: u32,
    win_levels: u32,
    available_actions: ArcAgi3AvailableActions,
    last_action: Option<ArcAgi3Action>,
}

impl ArcAgi3Observation {
    pub fn new(
        game_id: ArcAgi3GameId,
        state: ArcAgi3GameState,
        frames: ArcAgi3FrameSequence,
        levels_completed: u32,
        win_levels: u32,
        available_actions: ArcAgi3AvailableActions,
        last_action: Option<ArcAgi3Action>,
    ) -> Self {
        Self {
            game_id,
            state,
            frames,
            levels_completed,
            win_levels,
            available_actions,
            last_action,
        }
    }

    pub fn game_id(&self) -> &ArcAgi3GameId {
        &self.game_id
    }

    pub fn state(&self) -> ArcAgi3GameState {
        self.state
    }

    pub fn frames(&self) -> &ArcAgi3FrameSequence {
        &self.frames
    }

    pub fn levels_completed(&self) -> u32 {
        self.levels_completed
    }

    pub fn win_levels(&self) -> u32 {
        self.win_levels
    }

    pub fn available_actions(&self) -> &ArcAgi3AvailableActions {
        &self.available_actions
    }

    pub fn last_action(&self) -> Option<ArcAgi3Action> {
        self.last_action
    }

    pub fn terminal(&self) -> bool {
        self.state.is_terminal()
    }

    pub fn awaiting_action(&self) -> bool {
        self.state.awaiting_action()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArcAgi3ActionAuthorizationStatus {
    AuthorizedAction,
    AuthorizedReset,
    GameNotActive,
    ActionUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArcAgi3ActionAuthorization {
    status: ArcAgi3ActionAuthorizationStatus,
}

impl ArcAgi3ActionAuthorization {
    pub fn status(self) -> ArcAgi3ActionAuthorizationStatus {
        self.status
    }

    pub fn authorized(self) -> bool {
        matches!(
            self.status,
            ArcAgi3ActionAuthorizationStatus::AuthorizedAction
                | ArcAgi3ActionAuthorizationStatus::AuthorizedReset
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ArcAgi3Protocol;

impl ArcAgi3Protocol {
    pub fn authorize_action(
        observation: &ArcAgi3Observation,
        action: ArcAgi3Action,
    ) -> ArcAgi3ActionAuthorization {
        if action.id() == ArcAgi3ActionId::Reset {
            return ArcAgi3ActionAuthorization {
                status: ArcAgi3ActionAuthorizationStatus::AuthorizedReset,
            };
        }

        if !observation.awaiting_action() {
            return ArcAgi3ActionAuthorization {
                status: ArcAgi3ActionAuthorizationStatus::GameNotActive,
            };
        }

        if !observation.available_actions().authorizes(action) {
            return ArcAgi3ActionAuthorization {
                status: ArcAgi3ActionAuthorizationStatus::ActionUnavailable,
            };
        }

        ArcAgi3ActionAuthorization {
            status: ArcAgi3ActionAuthorizationStatus::AuthorizedAction,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UniversalArcAgi3Protocol;

impl UniversalArcAgi3Protocol {
    pub fn authorize_action(
        observation: &ArcAgi3Observation,
        action: ArcAgi3Action,
    ) -> ArcAgi3ActionAuthorization {
        ArcAgi3Protocol::authorize_action(observation, action)
    }
}
