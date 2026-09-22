//! Passive adapter diagnostics. These records are never cognitive authority.
use std::io::{self, Write};

use athlesia_mindstone_sparse_cognition::CognitiveStructure;
use serde::{Deserialize, Serialize};

use crate::cognitive_interaction_runtime::{
    ArcAgi3CognitiveInteractionCompletion, ArcAgi3UnifiedExecutiveAuthorityKind,
};
use crate::live_environment_runtime::ArcAgi3LiveUnifiedStep;
use crate::{ArcAgi3Action, ArcAgi3ActionId, ArcAgi3GameState, ArcAgi3Observation};

/// Diagnostic-only protocol copies. Deserialization cannot create production actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArcAgi3TraceGameState {
    NotPlayed,
    NotFinished,
    Win,
    GameOver,
}

impl From<ArcAgi3GameState> for ArcAgi3TraceGameState {
    fn from(state: ArcAgi3GameState) -> Self {
        match state {
            ArcAgi3GameState::NotPlayed => Self::NotPlayed,
            ArcAgi3GameState::NotFinished => Self::NotFinished,
            ArcAgi3GameState::Win => Self::Win,
            ArcAgi3GameState::GameOver => Self::GameOver,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArcAgi3TraceActionId {
    Action1,
    Action2,
    Action3,
    Action4,
    Action5,
    Action6,
    Action7,
    Reset,
}

impl From<ArcAgi3ActionId> for ArcAgi3TraceActionId {
    fn from(id: ArcAgi3ActionId) -> Self {
        match id {
            ArcAgi3ActionId::Action1 => Self::Action1,
            ArcAgi3ActionId::Action2 => Self::Action2,
            ArcAgi3ActionId::Action3 => Self::Action3,
            ArcAgi3ActionId::Action4 => Self::Action4,
            ArcAgi3ActionId::Action5 => Self::Action5,
            ArcAgi3ActionId::Action6 => Self::Action6,
            ArcAgi3ActionId::Action7 => Self::Action7,
            ArcAgi3ActionId::Reset => Self::Reset,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArcAgi3TraceAction {
    pub id: ArcAgi3TraceActionId,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub coordinate: Option<[u8; 2]>,
}

impl From<ArcAgi3Action> for ArcAgi3TraceAction {
    fn from(action: ArcAgi3Action) -> Self {
        Self {
            id: action.id().into(),
            coordinate: action.coordinate_data().map(|xy| [xy.x(), xy.y()]),
        }
    }
}

/// Lossless structural identity, including ordering and every atom (not a hash).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArcAgi3TraceStructure {
    Atom(u64),
    Ordered(Vec<Self>),
    Unordered(Vec<Self>),
}

impl From<&CognitiveStructure> for ArcAgi3TraceStructure {
    fn from(value: &CognitiveStructure) -> Self {
        match value {
            CognitiveStructure::Atom(atom) => Self::Atom(*atom),
            CognitiveStructure::Ordered(children) => {
                Self::Ordered(children.iter().map(Self::from).collect())
            }
            CognitiveStructure::Unordered(children) => {
                Self::Unordered(children.iter().map(Self::from).collect())
            }
        }
    }
}

/// Exact observation state excluding the action echo. No inferred world state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArcAgi3TraceObservation {
    pub game_id: String,
    pub state: ArcAgi3TraceGameState,
    pub frames: Vec<Vec<Vec<u8>>>,
    pub levels_completed: u32,
    pub win_levels: u32,
    pub available_actions: Vec<ArcAgi3TraceActionId>,
}

impl From<&ArcAgi3Observation> for ArcAgi3TraceObservation {
    fn from(observation: &ArcAgi3Observation) -> Self {
        Self {
            game_id: observation.game_id().as_str().to_owned(),
            state: observation.state().into(),
            frames: observation
                .frames()
                .frames()
                .iter()
                .map(|frame| {
                    frame
                        .cells()
                        .chunks(frame.width())
                        .map(<[u8]>::to_vec)
                        .collect()
                })
                .collect(),
            levels_completed: observation.levels_completed(),
            win_levels: observation.win_levels(),
            available_actions: observation
                .available_actions()
                .actions()
                .iter()
                .copied()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArcAgi3TraceAuthorityKind {
    LegacyGrounded,
    EvidenceFaithfulEpistemic,
}

impl From<ArcAgi3UnifiedExecutiveAuthorityKind> for ArcAgi3TraceAuthorityKind {
    fn from(value: ArcAgi3UnifiedExecutiveAuthorityKind) -> Self {
        match value {
            ArcAgi3UnifiedExecutiveAuthorityKind::LegacyGrounded => Self::LegacyGrounded,

            ArcAgi3UnifiedExecutiveAuthorityKind::EvidenceFaithfulEpistemic => {
                Self::EvidenceFaithfulEpistemic
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArcAgi3TraceAuthority {
    pub kind: ArcAgi3TraceAuthorityKind,
    pub selected_action: ArcAgi3TraceAction,
    pub cognitive_action: ArcAgi3TraceStructure,
    pub source_state: ArcAgi3TraceStructure,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArcAgi3TraceOutcome {
    pub observation: ArcAgi3TraceObservation,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub observed_action_echo: Option<ArcAgi3TraceAction>,
    pub structurally_changed: bool,
    pub produced_cognitive_completion: bool,
    pub has_cognitive_feedback: bool,
    pub session_event_index: u64,
}

impl ArcAgi3TraceOutcome {
    pub(crate) fn new(
        before: &ArcAgi3TraceObservation,
        completion: &ArcAgi3CognitiveInteractionCompletion,
    ) -> Self {
        let observation = ArcAgi3TraceObservation::from(completion.turn().observation());
        Self {
            structurally_changed: before != &observation,
            observation,
            observed_action_echo: completion
                .turn()
                .observation()
                .last_action()
                .map(Into::into),
            produced_cognitive_completion: true,
            has_cognitive_feedback: completion.has_cognitive_feedback(),
            session_event_index: completion.turn().event_index(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ArcAgi3TraceOperation {
    SuccessorInformedUnified,
    EvidenceFaithfulSuccessor,
    Reset,
}

/// Completed counts are runtime counters: abstention/failure do not advance them.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum ArcAgi3CognitiveTraceEvent {
    Executed {
        completed_cognitive_step_count: u64,
        before: ArcAgi3TraceObservation,
        authority: ArcAgi3TraceAuthority,
        command_action: ArcAgi3TraceAction,
        outcome: ArcAgi3TraceOutcome,
    },
    Abstained {
        operation: ArcAgi3TraceOperation,
        completed_cognitive_step_count: u64,
        before: ArcAgi3TraceObservation,
    },
    Reset {
        completed_cognitive_step_count: u64,
        completed_reset_count: u64,
        before: ArcAgi3TraceObservation,
        command_action: ArcAgi3TraceAction,
        outcome: ArcAgi3TraceOutcome,
    },
    TransportFailure {
        operation: ArcAgi3TraceOperation,
        completed_cognitive_step_count: u64,
        before: ArcAgi3TraceObservation,
        /// Exact Debug rendering of the returned transport error; not an outcome.
        error_debug: String,
        has_pending_command: bool,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        pending_command_action: Option<ArcAgi3TraceAction>,
        produced_cognitive_completion: bool,
    },
}

impl ArcAgi3CognitiveTraceEvent {
    pub(crate) fn executed(before: ArcAgi3TraceObservation, step: &ArcAgi3LiveUnifiedStep) -> Self {
        let outcome = ArcAgi3TraceOutcome::new(&before, step.completion());
        Self::Executed {
            completed_cognitive_step_count: step.completed_cognitive_step_count(),
            before,
            authority: ArcAgi3TraceAuthority {
                kind: step.authority().kind().into(),
                selected_action: step.action().into(),
                cognitive_action: step.cognitive_action().into(),
                source_state: step.source_state().into(),
            },
            command_action: step.command().action().into(),
            outcome,
        }
    }

    /// One deterministic record and one newline; no episode buffering or files.
    pub fn write_jsonl(&self, writer: &mut impl Write) -> io::Result<()> {
        serde_json::to_writer(&mut *writer, self).map_err(io::Error::other)?;
        writer.write_all(b"\n")
    }
}

/// Receives only detached data, after execution/completion or abstention.
/// Errors are diagnostic only and never propagated as execution failures.
pub trait ArcAgi3CognitiveTraceSink {
    fn record(&mut self, event: &ArcAgi3CognitiveTraceEvent) -> io::Result<()>;
}

/// Caller-owned streaming destination. The caller controls flushing and lifetime.
pub struct ArcAgi3JsonlTraceSink<W> {
    pub writer: W,
    pub last_error: Option<String>,
}

impl<W: Write> ArcAgi3JsonlTraceSink<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            last_error: None,
        }
    }
}

impl<W: Write> ArcAgi3CognitiveTraceSink for ArcAgi3JsonlTraceSink<W> {
    fn record(&mut self, event: &ArcAgi3CognitiveTraceEvent) -> io::Result<()> {
        let result = event.write_jsonl(&mut self.writer);
        if let Err(error) = &result {
            self.last_error = Some(error.to_string());
        }
        result
    }
}

pub(crate) fn emit(sink: &mut impl ArcAgi3CognitiveTraceSink, event: &ArcAgi3CognitiveTraceEvent) {
    // A sink's I/O error or unwinding panic cannot replace the live result.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sink.record(event)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_encoding_preserves_variants_order_and_full_width_atoms() {
        let structure = CognitiveStructure::Ordered(vec![
            CognitiveStructure::Atom(u64::MAX),
            CognitiveStructure::Unordered(vec![
                CognitiveStructure::Atom(9),
                CognitiveStructure::Atom(2),
            ]),
        ]);
        let record = ArcAgi3TraceStructure::from(&structure);
        let json = serde_json::to_string(&record).unwrap();
        assert_eq!(
            json,
            r#"{"Ordered":[{"Atom":18446744073709551615},{"Unordered":[{"Atom":9},{"Atom":2}]}]}"#
        );
        assert_eq!(
            serde_json::from_str::<ArcAgi3TraceStructure>(&json).unwrap(),
            record
        );
    }

    #[test]
    fn action_encoding_preserves_coordinates_and_reset() {
        let coordinate = ArcAgi3TraceAction::from(ArcAgi3Action::coordinate(12, 63).unwrap());
        assert_eq!(coordinate.id, ArcAgi3TraceActionId::Action6);
        assert_eq!(coordinate.coordinate, Some([12, 63]));
        assert_eq!(
            serde_json::from_str::<ArcAgi3TraceAction>(
                &serde_json::to_string(&coordinate).unwrap()
            )
            .unwrap(),
            coordinate
        );
        let reset = ArcAgi3TraceAction::from(ArcAgi3Action::reset());
        assert_eq!(reset.id, ArcAgi3TraceActionId::Reset);
        assert_eq!(reset.coordinate, None);
    }

    #[test]
    fn jsonl_sink_retains_write_errors_for_caller_inspection() {
        struct BrokenWriter;
        impl Write for BrokenWriter {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("writer unavailable"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let event = ArcAgi3CognitiveTraceEvent::Abstained {
            operation: ArcAgi3TraceOperation::SuccessorInformedUnified,
            completed_cognitive_step_count: 0,
            before: ArcAgi3TraceObservation {
                game_id: "writer-test".into(),
                state: ArcAgi3TraceGameState::NotFinished,
                frames: vec![vec![vec![0]]],
                levels_completed: 0,
                win_levels: 1,
                available_actions: vec![],
            },
        };
        let mut sink = ArcAgi3JsonlTraceSink::new(BrokenWriter);
        emit(&mut sink, &event);
        assert!(sink.last_error.unwrap().contains("writer unavailable"));
    }
}
