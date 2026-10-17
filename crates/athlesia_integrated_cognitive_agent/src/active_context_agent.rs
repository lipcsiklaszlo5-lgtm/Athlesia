//! Bounded active perception above the autonomous causal learner.
//!
//! The existing learner requires a complete Boolean state at the action-port
//! boundary.  Real sensors often provide a partial state, so this module adds
//! one small, auditable layer: every field is `True`, `False`, or `Unknown`;
//! an unknown is never coerced to false.  Before dispatching a learned action
//! the wrapper selects a host-approved inspection for an unknown field.  Once
//! the host returns the inspection result, the complete state is handed to the
//! existing learner unchanged.
//!
//! This is active perception for a bounded Boolean world, not a claim of
//! general perception or human intuition.  Inspection values are supplied by
//! the host and are never guessed by the solver.

use std::collections::BTreeMap;

use serde::Serialize;

use super::autonomous_causal_agent::{
    AbstainReason, ActionProposal, AgentBounds, AgentDecision, AgentError, AgentObservation,
    AgentTask, AutonomousCausalAgent, EvidenceVerdict,
};
use super::cross_domain_causal_core::{Literal, State, TokenId};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum TruthValue {
    True,
    False,
    Unknown,
}

impl TruthValue {
    pub fn from_option(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::True,
            Some(false) => Self::False,
            None => Self::Unknown,
        }
    }

    pub fn as_option(self) -> Option<bool> {
        match self {
            Self::True => Some(true),
            Self::False => Some(false),
            Self::Unknown => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartialObservation {
    pub episode: u64,
    pub sequence: u64,
    values: BTreeMap<TokenId, TruthValue>,
}

impl PartialObservation {
    pub fn new(
        episode: u64,
        sequence: u64,
        fields: impl IntoIterator<Item = (TokenId, TruthValue)>,
    ) -> Self {
        Self {
            episode,
            sequence,
            values: fields.into_iter().collect(),
        }
    }

    pub fn get(&self, token: TokenId) -> TruthValue {
        self.values
            .get(&token)
            .copied()
            .unwrap_or(TruthValue::Unknown)
    }

    pub fn unknown_fields(&self) -> impl Iterator<Item = TokenId> + '_ {
        self.values.iter().filter_map(|(token, value)| {
            (*value == TruthValue::Unknown).then_some(*token)
        })
    }

    pub fn fields(&self) -> impl Iterator<Item = (TokenId, TruthValue)> + '_ {
        self.values.iter().map(|(token, value)| (*token, *value))
    }

    pub fn with(&self, token: TokenId, value: bool) -> Self {
        let mut next = self.clone();
        next.values.insert(token, TruthValue::from_option(Some(value)));
        next
    }

    pub fn complete_state(&self) -> Option<State> {
        let literals = self
            .values
            .iter()
            .map(|(token, value)| Some(Literal::new(*token, value.as_option()?)))
            .collect::<Option<Vec<_>>>()?;
        Some(State::new(literals))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct InspectionAction {
    pub command: u64,
    pub token: TokenId,
    pub cost: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActiveDecision {
    GoalObserved,
    Inspect(InspectionAction),
    Act(ActionProposal),
    Abstain(ActiveAbstain),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ActiveAbstain {
    UnknownState,
    NoInspectableField,
    Learner(AbstainReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActiveError {
    InvalidTask,
    InvalidObservation,
    CommandPending,
    NoPendingCommand,
    WrongInspection,
    StaleObservation,
    Learner(AgentError),
}

impl From<AgentError> for ActiveError {
    fn from(value: AgentError) -> Self {
        Self::Learner(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct ActiveTelemetry {
    pub inspections_selected: u64,
    pub inspections_completed: u64,
    pub actions_dispatched: u64,
    pub unknown_fields_seen: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pending {
    Inspection(InspectionAction),
    Action(u64),
}

/// Host-facing active perception controller.
#[derive(Clone, Debug)]
pub struct ActiveContextAgent {
    task: AgentTask,
    bounds: AgentBounds,
    observation: PartialObservation,
    goal: Vec<Literal>,
    inspectable: Vec<(TokenId, u32)>,
    pending: Option<Pending>,
    next_command: u64,
    learner: Option<AutonomousCausalAgent>,
    retained_memory: Option<String>,
    telemetry: ActiveTelemetry,
}

impl ActiveContextAgent {
    pub fn new(
        task: AgentTask,
        initial: PartialObservation,
        goal: Vec<Literal>,
        inspectable: impl IntoIterator<Item = (TokenId, u32)>,
        bounds: AgentBounds,
    ) -> Result<Self, ActiveError> {
        if goal.is_empty() || initial.values.is_empty() {
            return Err(ActiveError::InvalidTask);
        }
        let mut inspectable: Vec<_> = inspectable.into_iter().collect();
        inspectable.sort_unstable();
        inspectable.dedup();
        if inspectable.iter().any(|(_, cost)| *cost == 0) {
            return Err(ActiveError::InvalidTask);
        }
        let mut agent = Self {
            task,
            bounds,
            observation: initial,
            goal,
            inspectable,
            pending: None,
            next_command: 1,
            learner: None,
            retained_memory: None,
            telemetry: ActiveTelemetry {
                inspections_selected: 0,
                inspections_completed: 0,
                actions_dispatched: 0,
                unknown_fields_seen: 0,
            },
        };
        agent.rebuild_learner_if_complete()?;
        Ok(agent)
    }

    pub fn observation(&self) -> &PartialObservation {
        &self.observation
    }

    pub fn telemetry(&self) -> ActiveTelemetry {
        self.telemetry
    }

    pub fn learner(&self) -> Option<&AutonomousCausalAgent> {
        self.learner.as_ref()
    }

    pub fn unknown_count(&self) -> usize {
        self.observation.unknown_fields().count()
    }

    pub fn begin_episode(
        &mut self,
        observation: PartialObservation,
        goal: Vec<Literal>,
    ) -> Result<(), ActiveError> {
        if self.pending.is_some() {
            return Err(ActiveError::CommandPending);
        }
        if observation.episode <= self.observation.episode || goal.is_empty() {
            return Err(ActiveError::InvalidObservation);
        }
        self.goal = goal.clone();
        if let Some(learner) = self.learner.take() {
            self.retained_memory = Some(learner.export_memory()?);
        }
        self.observation = observation.clone();
        self.rebuild_learner_if_complete()
    }

    /// Decide whether to inspect an unknown field or hand control to the
    /// complete-state causal learner.
    pub fn decide(&mut self) -> Result<ActiveDecision, ActiveError> {
        if self.pending.is_some() {
            return Err(ActiveError::CommandPending);
        }
        if self.goal.iter().all(|goal| {
            self.observation.get(goal.role) == TruthValue::True
        }) {
            return Ok(ActiveDecision::GoalObserved);
        }
        let unknown: Vec<_> = self.observation.unknown_fields().collect();
        self.telemetry.unknown_fields_seen = self
            .telemetry
            .unknown_fields_seen
            .saturating_add(unknown.len() as u64);
        if !unknown.is_empty() {
            let selected = self
                .inspectable
                .iter()
                .filter(|(token, _)| unknown.contains(token))
                .min_by_key(|(token, cost)| (*cost, *token))
                .copied()
                .ok_or(ActiveError::InvalidObservation)?;
            let action = InspectionAction {
                command: self.next_command,
                token: selected.0,
                cost: selected.1,
            };
            self.next_command = self.next_command.saturating_add(1);
            self.pending = Some(Pending::Inspection(action));
            self.telemetry.inspections_selected = self
                .telemetry
                .inspections_selected
                .saturating_add(1);
            return Ok(ActiveDecision::Inspect(action));
        }
        let learner = self.learner.as_mut().ok_or(ActiveError::InvalidObservation)?;
        match learner.decide()? {
            AgentDecision::GoalObserved => Ok(ActiveDecision::GoalObserved),
            AgentDecision::Act(proposal) => {
                self.pending = Some(Pending::Action(proposal.command));
                self.telemetry.actions_dispatched = self
                    .telemetry
                    .actions_dispatched
                    .saturating_add(1);
                Ok(ActiveDecision::Act(proposal))
            }
            AgentDecision::Abstain(reason) => Ok(ActiveDecision::Abstain(
                ActiveAbstain::Learner(reason),
            )),
        }
    }

    pub fn observe_inspection(
        &mut self,
        command: u64,
        token: TokenId,
        value: bool,
    ) -> Result<(), ActiveError> {
        let Pending::Inspection(expected) = self.pending.ok_or(ActiveError::NoPendingCommand)?
        else {
            return Err(ActiveError::WrongInspection);
        };
        if expected.command != command || expected.token != token {
            return Err(ActiveError::WrongInspection);
        }
        self.observation = self.observation.with(token, value);
        self.pending = None;
        self.telemetry.inspections_completed = self
            .telemetry
            .inspections_completed
            .saturating_add(1);
        self.rebuild_learner_if_complete()
    }

    pub fn observe_action(
        &mut self,
        command: u64,
        next: PartialObservation,
    ) -> Result<EvidenceVerdict, ActiveError> {
        let Pending::Action(expected) = self.pending.ok_or(ActiveError::NoPendingCommand)?
        else {
            return Err(ActiveError::WrongInspection);
        };
        if expected != command
            || next.episode != self.observation.episode
            || next.sequence != self.observation.sequence.saturating_add(1)
            || next.unknown_fields().next().is_some()
        {
            return Err(ActiveError::StaleObservation);
        }
        let state = next.complete_state().ok_or(ActiveError::InvalidObservation)?;
        let learner = self.learner.as_mut().ok_or(ActiveError::InvalidObservation)?;
        let verdict = learner.observe(command, AgentObservation {
            episode: next.episode,
            sequence: next.sequence,
            state,
        })?;
        self.observation = next;
        self.pending = None;
        Ok(verdict)
    }

    fn rebuild_learner_if_complete(&mut self) -> Result<(), ActiveError> {
        let Some(state) = self.observation.complete_state() else {
            self.learner = None;
            return Ok(());
        };
        let mut task = self.task.clone();
        task.goal = self.goal.clone();
        let initial = AgentObservation {
            episode: self.observation.episode,
            sequence: self.observation.sequence,
            state,
        };
        self.learner = Some(if let Some(memory) = self.retained_memory.take() {
            AutonomousCausalAgent::restore(task, initial, self.bounds, &memory)?
        } else {
            AutonomousCausalAgent::new(task, initial, self.bounds)?
        });
        Ok(())
    }
}
