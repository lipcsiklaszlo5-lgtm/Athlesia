//! Bounded context-conditioned causal memory above the autonomous A1/A2 agent.
//!
//! A context is an observed, opaque host token (for example a tool mode or a
//! task phase). The kernel does not infer its meaning and never receives a
//! hidden mechanism label. Each context owns an ordered set of model epochs.
//! When a complete observation is outside the active epoch's hypothesis class,
//! that epoch is retained and a fresh epoch is opened from the observed state.
//! This is a bounded form of non-destructive online revision, not a claim of
//! human-level intuition or unrestricted continual learning.

use std::collections::BTreeMap;

use serde::Serialize;

use super::autonomous_causal_agent::{
    AbstainReason, ActionProposal, AgentBounds, AgentDecision, AgentError, AgentObservation,
    AgentTask, AutonomousCausalAgent, EvidenceVerdict,
};
use super::cross_domain_causal_core::{Literal, TransitionExample};

/// Opaque context supplied by the host at episode boundaries.
///
/// The solver may compare context IDs for equality, but is not allowed to
/// assign them semantic meaning. A context is evidence, not an oracle.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ContextId(pub u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextObservation {
    pub context: ContextId,
    pub observation: AgentObservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextualActionProposal {
    pub context: ContextId,
    pub epoch: u64,
    pub proposal: ActionProposal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualDecision {
    GoalObserved,
    Act(ContextualActionProposal),
    Abstain(AbstainReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ContextualVerdict {
    Consistent,
    ContradictionAndLocalRevision,
    TransferFalsified,
    OutOfModelClass,
    /// The old epoch remains available for audit/replay; a new epoch starts
    /// from the complete counterexample observation.
    EpochRolled {
        previous_epoch: u64,
        new_epoch: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualError {
    Agent(AgentError),
    NoActiveContext,
    CommandPending,
    NoPendingCommand,
    WrongProposal,
    ContextMismatch {
        expected: ContextId,
        observed: ContextId,
    },
    EpochMismatch {
        expected: u64,
        observed: u64,
    },
}

impl From<AgentError> for ContextualError {
    fn from(error: AgentError) -> Self {
        Self::Agent(error)
    }
}

#[derive(Clone, Debug)]
struct ContextEpoch {
    number: u64,
    agent: AutonomousCausalAgent,
}

#[derive(Clone, Debug)]
struct ContextMemory {
    goal: Vec<Literal>,
    epochs: Vec<ContextEpoch>,
}

#[derive(Clone, Copy, Debug)]
struct PendingContext {
    context: ContextId,
    epoch: u64,
    command: u64,
}

/// Context-indexed, non-destructive online causal learner.
///
/// The wrapper deliberately delegates hypothesis generation, active action
/// choice, observation binding, transfer priors and resource budgets to the
/// existing `AutonomousCausalAgent`. Its only additional authority is epoch
/// bookkeeping after a complete, validated out-of-class observation.
#[derive(Clone, Debug)]
pub struct ContextualCausalAgent {
    task_template: AgentTask,
    bounds: AgentBounds,
    contexts: BTreeMap<ContextId, ContextMemory>,
    active_context: Option<ContextId>,
    pending: Option<PendingContext>,
}

impl ContextualCausalAgent {
    pub fn new(task_template: AgentTask, bounds: AgentBounds) -> Self {
        Self {
            task_template,
            bounds,
            contexts: BTreeMap::new(),
            active_context: None,
            pending: None,
        }
    }

    /// Start a new episode in a context. Existing context memory is reused;
    /// unseen contexts receive a clean bounded learner.
    pub fn begin_episode(
        &mut self,
        context: ContextId,
        initial: AgentObservation,
        goal: Vec<Literal>,
    ) -> Result<(), ContextualError> {
        if self.pending.is_some() {
            return Err(ContextualError::CommandPending);
        }
        if let Some(memory) = self.contexts.get_mut(&context) {
            let epoch = memory.epochs.last_mut().expect("context has an epoch");
            epoch.agent.begin_episode(initial, goal.clone())?;
            memory.goal = goal;
        } else {
            let mut task = self.task_template.clone();
            task.goal = goal.clone();
            let agent = AutonomousCausalAgent::new(task, initial, self.bounds)?;
            self.contexts.insert(
                context,
                ContextMemory {
                    goal,
                    epochs: vec![ContextEpoch { number: 0, agent }],
                },
            );
        }
        self.active_context = Some(context);
        Ok(())
    }

    pub fn current_context(&self) -> Option<ContextId> {
        self.active_context
    }

    pub fn current_epoch(&self) -> Option<u64> {
        self.active_context
            .and_then(|context| self.contexts.get(&context))
            .and_then(|memory| memory.epochs.last())
            .map(|epoch| epoch.number)
    }

    pub fn context_count(&self) -> usize {
        self.contexts.len()
    }

    pub fn epoch_count(&self, context: ContextId) -> usize {
        self.contexts
            .get(&context)
            .map(|memory| memory.epochs.len())
            .unwrap_or(0)
    }

    pub fn evidence_count(&self, context: ContextId) -> usize {
        self.contexts
            .get(&context)
            .and_then(|memory| memory.epochs.last())
            .map(|epoch| epoch.agent.evidence_count())
            .unwrap_or(0)
    }

    pub fn total_retained_evidence(&self) -> usize {
        self.contexts
            .values()
            .flat_map(|memory| memory.epochs.iter())
            .map(|epoch| epoch.agent.evidence_count())
            .sum()
    }

    /// Ingest one complete, externally observed transition without issuing a
    /// command. This is the replay/trace ingestion path used by the discovery
    /// layer; it never treats the transition as a trusted rule.
    pub(crate) fn learn_trace(
        &mut self,
        context: ContextId,
        initial: AgentObservation,
        goal: Vec<Literal>,
        example: TransitionExample,
    ) -> Result<ContextualVerdict, ContextualError> {
        if self.pending.is_some() {
            return Err(ContextualError::CommandPending);
        }
        if !self.contexts.contains_key(&context) {
            let mut task = self.task_template.clone();
            task.goal = goal.clone();
            let agent = AutonomousCausalAgent::new(task, initial, self.bounds)?;
            self.contexts.insert(
                context,
                ContextMemory {
                    goal: goal.clone(),
                    epochs: vec![ContextEpoch { number: 0, agent }],
                },
            );
        }
        let memory = self
            .contexts
            .get_mut(&context)
            .ok_or(ContextualError::NoActiveContext)?;
        memory.goal = goal;
        let epoch = memory.epochs.last_mut().expect("context has an epoch");
        let verdict = epoch.agent.learn_trace(example)?;
        self.active_context = Some(context);
        Ok(match verdict {
            EvidenceVerdict::Consistent => ContextualVerdict::Consistent,
            EvidenceVerdict::ContradictionAndLocalRevision => {
                ContextualVerdict::ContradictionAndLocalRevision
            }
            EvidenceVerdict::TransferFalsified => ContextualVerdict::TransferFalsified,
            EvidenceVerdict::OutOfModelClass => ContextualVerdict::OutOfModelClass,
        })
    }

    /// Rebuild a context-indexed memory from independently validated traces.
    ///
    /// This is crate-private on purpose: callers must not be able to inject a
    /// claimed model. The observable-context layer uses it only after replaying
    /// every transition through the same bounded learner. The active context
    /// may start from a current observation so discovery can continue in the
    /// episode in which the counterexample was observed.
    pub(crate) fn from_partitioned_traces(
        task_template: AgentTask,
        bounds: AgentBounds,
        partitions: BTreeMap<ContextId, Vec<TransitionExample>>,
        goal: Vec<Literal>,
        active: Option<(ContextId, AgentObservation)>,
    ) -> Result<Self, ContextualError> {
        if partitions.is_empty() {
            return Err(ContextualError::Agent(AgentError::InvalidTask(
                "empty context partition",
            )));
        }
        let active_context = active.as_ref().map(|(context, _)| *context);
        let mut contexts = BTreeMap::new();
        for (context, examples) in partitions {
            let first = examples
                .first()
                .ok_or(ContextualError::Agent(AgentError::InvalidTask(
                    "empty context partition",
                )))?;
            let initial = active
                .as_ref()
                .filter(|(active_id, _)| *active_id == context)
                .map(|(_, observation)| observation.clone())
                .unwrap_or_else(|| AgentObservation {
                    episode: first.episode,
                    sequence: 0,
                    state: first.before.clone(),
                });
            let mut task = task_template.clone();
            task.goal = goal.clone();
            let mut agent = AutonomousCausalAgent::new(task, initial, bounds)?;
            for example in examples {
                if agent.learn_trace(example)? == EvidenceVerdict::OutOfModelClass {
                    return Err(ContextualError::Agent(AgentError::InvalidTask(
                        "context partition is outside hypothesis class",
                    )));
                }
            }
            contexts.insert(
                context,
                ContextMemory {
                    goal: goal.clone(),
                    epochs: vec![ContextEpoch { number: 0, agent }],
                },
            );
        }
        Ok(Self {
            task_template,
            bounds,
            contexts,
            active_context,
            pending: None,
        })
    }

    pub fn decide(&mut self) -> Result<ContextualDecision, ContextualError> {
        if self.pending.is_some() {
            return Err(ContextualError::CommandPending);
        }
        let context = self
            .active_context
            .ok_or(ContextualError::NoActiveContext)?;
        let memory = self
            .contexts
            .get_mut(&context)
            .ok_or(ContextualError::NoActiveContext)?;
        let epoch = memory.epochs.last_mut().expect("context has an epoch");
        match epoch.agent.decide()? {
            AgentDecision::GoalObserved => Ok(ContextualDecision::GoalObserved),
            AgentDecision::Abstain(reason) => Ok(ContextualDecision::Abstain(reason)),
            AgentDecision::Act(proposal) => {
                let contextual = ContextualActionProposal {
                    context,
                    epoch: epoch.number,
                    proposal,
                };
                self.pending = Some(PendingContext {
                    context,
                    epoch: epoch.number,
                    command: contextual.proposal.command,
                });
                Ok(ContextualDecision::Act(contextual))
            }
        }
    }

    /// Bind a complete observation to the exact context, epoch and command.
    /// An out-of-class result opens a new epoch without deleting the old one.
    pub fn observe(
        &mut self,
        proposal: &ContextualActionProposal,
        next: ContextObservation,
    ) -> Result<ContextualVerdict, ContextualError> {
        let pending = self.pending.ok_or(ContextualError::NoPendingCommand)?;
        if pending.context != proposal.context
            || pending.epoch != proposal.epoch
            || pending.command != proposal.proposal.command
        {
            return Err(ContextualError::WrongProposal);
        }
        if next.context != pending.context {
            return Err(ContextualError::ContextMismatch {
                expected: pending.context,
                observed: next.context,
            });
        }
        let context = pending.context;
        let memory = self
            .contexts
            .get_mut(&context)
            .ok_or(ContextualError::NoActiveContext)?;
        let epoch = memory.epochs.last_mut().expect("context has an epoch");
        if epoch.number != pending.epoch {
            return Err(ContextualError::EpochMismatch {
                expected: pending.epoch,
                observed: epoch.number,
            });
        }
        let verdict = epoch
            .agent
            .observe(proposal.proposal.command, next.observation.clone())?;
        self.pending = None;
        if verdict != EvidenceVerdict::OutOfModelClass {
            return Ok(match verdict {
                EvidenceVerdict::Consistent => ContextualVerdict::Consistent,
                EvidenceVerdict::ContradictionAndLocalRevision => {
                    ContextualVerdict::ContradictionAndLocalRevision
                }
                EvidenceVerdict::TransferFalsified => ContextualVerdict::TransferFalsified,
                EvidenceVerdict::OutOfModelClass => unreachable!(),
            });
        }

        let previous_epoch = epoch.number;
        let new_epoch = previous_epoch.checked_add(1).ok_or(ContextualError::Agent(
            AgentError::InvalidTask("epoch overflow"),
        ))?;
        let mut task = self.task_template.clone();
        task.goal = memory.goal.clone();
        let fresh = AutonomousCausalAgent::new(task, next.observation, self.bounds)?;
        memory.epochs.push(ContextEpoch {
            number: new_epoch,
            agent: fresh,
        });
        Ok(ContextualVerdict::EpochRolled {
            previous_epoch,
            new_epoch,
        })
    }
}
