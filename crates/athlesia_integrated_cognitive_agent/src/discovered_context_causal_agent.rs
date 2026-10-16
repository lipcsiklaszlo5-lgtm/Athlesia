//! Bounded discovery of observable context from complete transition traces.
//!
//! A3 accepts an opaque context token at an episode boundary.  This module
//! removes that host-side label, but deliberately keeps the claim narrow: it
//! discovers a small projection of *observable* Boolean state fields.  When
//! the same action from the same complete state has produced incompatible
//! outcomes, the projection search partitions the retained evidence and
//! replays every partition through the ordinary bounded learner.  An ambiguous
//! or unidentifiable partition is reported instead of being guessed.
//!
//! This is not hidden-context inference, perception, stochastic system
//! identification or a claim of human intuition.  It is a deterministic,
//! resource-bounded model-selection layer above the A3 epoch wrapper.

use std::collections::BTreeMap;

use serde::Serialize;

use super::autonomous_causal_agent::{
    AbstainReason, ActionProposal, AgentBounds, AgentError, AgentObservation, AgentTask,
    AutonomousCausalAgent, EvidenceVerdict,
};
use super::contextual_causal_agent::{
    ContextId, ContextObservation, ContextualActionProposal, ContextualCausalAgent,
    ContextualDecision, ContextualError, ContextualVerdict,
};
use super::cross_domain_causal_core::{Literal, NodeKind, State, TokenId, TransitionExample};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiscoveryBounds {
    /// Maximum number of observable fields in a discovered context key.
    /// Values above two are rejected to keep the search auditable.
    pub max_context_fields: usize,
    /// Maximum number of context partitions retained after discovery.
    pub max_contexts: usize,
    /// Maximum number of complete transitions retained for discovery replay.
    pub max_history: usize,
    /// Maximum number of candidate field projections evaluated per revision.
    pub max_candidate_evaluations: usize,
}

impl Default for DiscoveryBounds {
    fn default() -> Self {
        Self {
            max_context_fields: 2,
            max_contexts: 8,
            max_history: 512,
            max_candidate_evaluations: 64,
        }
    }
}

impl DiscoveryBounds {
    fn validate(self) -> Result<Self, DiscoveryError> {
        if self.max_context_fields == 0 || self.max_context_fields > 2 {
            return Err(DiscoveryError::InvalidTask(
                "context field bound must be 1..=2",
            ));
        }
        if self.max_contexts < 2 || self.max_contexts > 32 {
            return Err(DiscoveryError::InvalidTask(
                "context partition bound must be 2..=32",
            ));
        }
        if self.max_history == 0 || self.max_history > 4096 {
            return Err(DiscoveryError::InvalidTask(
                "context history bound must be 1..=4096",
            ));
        }
        if self.max_candidate_evaluations == 0 || self.max_candidate_evaluations > 4096 {
            return Err(DiscoveryError::InvalidTask(
                "candidate evaluation bound must be 1..=4096",
            ));
        }
        Ok(self)
    }
}

/// Canonical, fully observable context key.  The key contains values only;
/// field names remain opaque token IDs and are never interpreted semantically.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ContextSignature {
    fields: Vec<Literal>,
}

impl ContextSignature {
    pub fn fields(&self) -> &[Literal] {
        &self.fields
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum ContextDiscoveryStatus {
    Unidentified,
    Identified {
        fields: Vec<TokenId>,
        contexts: usize,
    },
    Ambiguous,
    Unavailable,
    ResourceExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryActionProposal {
    pub signature: ContextSignature,
    pub epoch: u64,
    pub proposal: ActionProposal,
    context: ContextId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveryDecision {
    GoalObserved,
    Act(DiscoveryActionProposal),
    Abstain(AbstainReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveryVerdict {
    Consistent,
    ContradictionAndLocalRevision,
    TransferFalsified,
    EpochRolled {
        previous_epoch: u64,
        new_epoch: u64,
    },
    ContextDiscovered {
        fields: Vec<TokenId>,
        contexts: usize,
    },
    ContextAmbiguous,
    ContextUnavailable,
    ContextResourceExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscoveryError {
    Contextual(ContextualError),
    InvalidTask(&'static str),
    PendingCommand,
    NoPendingCommand,
    WrongProposal,
    HistoryBudget,
}

impl From<ContextualError> for DiscoveryError {
    fn from(error: ContextualError) -> Self {
        Self::Contextual(error)
    }
}

impl From<AgentError> for DiscoveryError {
    fn from(error: AgentError) -> Self {
        Self::Contextual(ContextualError::Agent(error))
    }
}

#[derive(Clone, Debug)]
struct PendingDiscovery {
    proposal: DiscoveryActionProposal,
    before: AgentObservation,
}

#[derive(Clone, Debug)]
struct CandidatePartition {
    fields: Vec<TokenId>,
    groups: BTreeMap<ContextSignature, Vec<TransitionExample>>,
}

/// Autonomous wrapper whose context key is learned from observable state.
#[derive(Clone, Debug)]
pub struct DiscoveredContextCausalAgent {
    task_template: AgentTask,
    bounds: AgentBounds,
    discovery_bounds: DiscoveryBounds,
    contextual: ContextualCausalAgent,
    candidate_fields: Vec<TokenId>,
    selected_fields: Option<Vec<TokenId>>,
    context_ids: BTreeMap<ContextSignature, ContextId>,
    next_context_id: u64,
    history: Vec<TransitionExample>,
    goal: Option<Vec<Literal>>,
    current: Option<AgentObservation>,
    active_signature: Option<ContextSignature>,
    pending: Option<PendingDiscovery>,
    status: ContextDiscoveryStatus,
    candidate_evaluations: usize,
}

impl DiscoveredContextCausalAgent {
    pub fn new(
        task_template: AgentTask,
        bounds: AgentBounds,
        discovery_bounds: DiscoveryBounds,
    ) -> Result<Self, DiscoveryError> {
        let discovery_bounds = discovery_bounds.validate()?;
        let mut candidate_fields: Vec<_> = task_template
            .graph
            .nodes()
            .iter()
            .filter(|node| node.kind == NodeKind::StateField)
            .map(|node| u64::from(node.id))
            .collect();
        candidate_fields.sort_unstable();
        candidate_fields.dedup();
        if candidate_fields.is_empty() {
            return Err(DiscoveryError::InvalidTask("no observable state fields"));
        }
        if discovery_bounds.max_context_fields > candidate_fields.len() {
            return Err(DiscoveryError::InvalidTask(
                "context field bound exceeds observable fields",
            ));
        }
        Ok(Self {
            contextual: ContextualCausalAgent::new(task_template.clone(), bounds),
            task_template,
            bounds,
            discovery_bounds,
            candidate_fields,
            selected_fields: None,
            context_ids: BTreeMap::new(),
            next_context_id: 1,
            history: Vec::new(),
            goal: None,
            current: None,
            active_signature: None,
            pending: None,
            status: ContextDiscoveryStatus::Unidentified,
            candidate_evaluations: 0,
        })
    }

    pub fn status(&self) -> &ContextDiscoveryStatus {
        &self.status
    }

    pub fn selected_fields(&self) -> Option<&[TokenId]> {
        self.selected_fields.as_deref()
    }

    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    pub fn candidate_evaluations(&self) -> usize {
        self.candidate_evaluations
    }

    pub fn context_count(&self) -> usize {
        self.contextual.context_count()
    }

    pub fn current_epoch(&self) -> Option<u64> {
        self.contextual.current_epoch()
    }

    pub fn total_retained_evidence(&self) -> usize {
        self.contextual.total_retained_evidence()
    }

    pub fn begin_episode(
        &mut self,
        initial: AgentObservation,
        goal: Vec<Literal>,
    ) -> Result<(), DiscoveryError> {
        if self.pending.is_some() {
            return Err(DiscoveryError::PendingCommand);
        }
        let signature =
            self.signature_for_state(&initial.state, self.selected_fields.as_deref())?;
        let context = self.context_id_for(signature.clone());
        self.contextual
            .begin_episode(context, initial.clone(), goal.clone())?;
        self.goal = Some(goal);
        self.current = Some(initial);
        self.active_signature = Some(signature);
        Ok(())
    }

    /// Replay one complete observation trace. This is useful for an existing
    /// event log and for deterministic holdout tests; unlike `decide`, it
    /// does not invent an action or grant execution authority.
    pub fn ingest_transition(
        &mut self,
        goal: Vec<Literal>,
        example: TransitionExample,
    ) -> Result<DiscoveryVerdict, DiscoveryError> {
        if self.pending.is_some() {
            return Err(DiscoveryError::PendingCommand);
        }
        if self.history.len() >= self.discovery_bounds.max_history {
            return Err(DiscoveryError::HistoryBudget);
        }
        let initial = AgentObservation {
            episode: example.episode,
            sequence: 0,
            state: example.before.clone(),
        };
        let signature =
            self.signature_for_state(&initial.state, self.selected_fields.as_deref())?;
        let context = self.context_id_for(signature.clone());
        let base = self
            .contextual
            .learn_trace(context, initial, goal.clone(), example.clone())?;
        self.goal = Some(goal);
        self.current = Some(AgentObservation {
            episode: example.episode,
            sequence: 1,
            state: example.after.clone(),
        });
        self.active_signature = Some(signature);
        self.history.push(example);
        if !matches!(&base, ContextualVerdict::OutOfModelClass) {
            return Ok(map_contextual_verdict(base));
        }
        match self.discover_context()? {
            DiscoveryAttempt::Identified { fields, contexts } => {
                Ok(DiscoveryVerdict::ContextDiscovered { fields, contexts })
            }
            DiscoveryAttempt::Ambiguous => Ok(DiscoveryVerdict::ContextAmbiguous),
            DiscoveryAttempt::Unavailable => Ok(DiscoveryVerdict::ContextUnavailable),
            DiscoveryAttempt::ResourceExhausted => Ok(DiscoveryVerdict::ContextResourceExhausted),
        }
    }

    pub fn decide(&mut self) -> Result<DiscoveryDecision, DiscoveryError> {
        if self.pending.is_some() {
            return Err(DiscoveryError::PendingCommand);
        }
        if self.history.len() >= self.discovery_bounds.max_history {
            return Ok(DiscoveryDecision::Abstain(AbstainReason::EvidenceBudget));
        }
        match self.contextual.decide()? {
            ContextualDecision::GoalObserved => Ok(DiscoveryDecision::GoalObserved),
            ContextualDecision::Abstain(reason) => Ok(DiscoveryDecision::Abstain(reason)),
            ContextualDecision::Act(inner) => {
                let signature = self
                    .active_signature
                    .clone()
                    .ok_or(DiscoveryError::InvalidTask("active signature missing"))?;
                let proposal = DiscoveryActionProposal {
                    signature,
                    epoch: inner.epoch,
                    proposal: inner.proposal,
                    context: inner.context,
                };
                let before = self
                    .current
                    .clone()
                    .ok_or(DiscoveryError::InvalidTask("current observation missing"))?;
                self.pending = Some(PendingDiscovery {
                    proposal: proposal.clone(),
                    before,
                });
                Ok(DiscoveryDecision::Act(proposal))
            }
        }
    }

    pub fn observe(
        &mut self,
        proposal: &DiscoveryActionProposal,
        next: AgentObservation,
    ) -> Result<DiscoveryVerdict, DiscoveryError> {
        let pending = self
            .pending
            .as_ref()
            .ok_or(DiscoveryError::NoPendingCommand)?
            .clone();
        if pending.proposal.context != proposal.context
            || pending.proposal.epoch != proposal.epoch
            || pending.proposal.proposal.command != proposal.proposal.command
            || pending.proposal.proposal.action != proposal.proposal.action
            || pending.proposal.signature != proposal.signature
        {
            return Err(DiscoveryError::WrongProposal);
        }
        let contextual = ContextualActionProposal {
            context: proposal.context,
            epoch: proposal.epoch,
            proposal: proposal.proposal.clone(),
        };
        let base = self.contextual.observe(
            &contextual,
            ContextObservation {
                context: proposal.context,
                observation: next.clone(),
            },
        )?;
        self.pending = None;
        self.current = Some(next.clone());
        if self.history.len() >= self.discovery_bounds.max_history {
            return Err(DiscoveryError::HistoryBudget);
        }
        self.history.push(TransitionExample::new(
            pending.before.episode,
            proposal.proposal.action,
            pending.before.state,
            next.state.clone(),
        ));

        // Context discovery is only attempted after the bounded learner has
        // proved that the current single-regime hypothesis class is empty.
        // A merely surprising but still expressible guard is not enough to
        // manufacture a context split.
        if !matches!(&base, ContextualVerdict::EpochRolled { .. }) {
            return Ok(map_contextual_verdict(base));
        }
        match self.discover_context()? {
            DiscoveryAttempt::Identified { fields, contexts } => {
                Ok(DiscoveryVerdict::ContextDiscovered { fields, contexts })
            }
            DiscoveryAttempt::Ambiguous => Ok(DiscoveryVerdict::ContextAmbiguous),
            DiscoveryAttempt::Unavailable => Ok(DiscoveryVerdict::ContextUnavailable),
            DiscoveryAttempt::ResourceExhausted => Ok(DiscoveryVerdict::ContextResourceExhausted),
        }
    }

    fn signature_for_state(
        &self,
        state: &State,
        fields: Option<&[TokenId]>,
    ) -> Result<ContextSignature, DiscoveryError> {
        let fields = fields.unwrap_or(&[]);
        let mut literals = Vec::with_capacity(fields.len());
        for role in fields {
            let value = state
                .get(*role)
                .ok_or(DiscoveryError::InvalidTask("incomplete observable state"))?;
            literals.push(Literal::new(*role, value));
        }
        Ok(ContextSignature { fields: literals })
    }

    fn context_id_for(&mut self, signature: ContextSignature) -> ContextId {
        if signature.is_empty() {
            return ContextId(0);
        }
        if let Some(id) = self.context_ids.get(&signature) {
            return *id;
        }
        let id = ContextId(self.next_context_id);
        self.next_context_id = self.next_context_id.saturating_add(1);
        self.context_ids.insert(signature, id);
        id
    }

    fn discover_context(&mut self) -> Result<DiscoveryAttempt, DiscoveryError> {
        let mut best: Option<CandidatePartition> = None;
        let mut best_shape: Option<(usize, usize, usize)> = None;
        let mut equal_best = 0usize;
        let mut combinations_checked = 0usize;
        for size in 1..=self.discovery_bounds.max_context_fields {
            let combinations = combinations(&self.candidate_fields, size);
            for fields in combinations {
                combinations_checked = combinations_checked.saturating_add(1);
                if combinations_checked > self.discovery_bounds.max_candidate_evaluations {
                    self.candidate_evaluations = combinations_checked;
                    self.status = ContextDiscoveryStatus::ResourceExhausted;
                    return Ok(DiscoveryAttempt::ResourceExhausted);
                }
                let groups = self.partition(&fields)?;
                if groups.len() < 2 || groups.len() > self.discovery_bounds.max_contexts {
                    continue;
                }
                if !self.separates_any_conflict(&fields) {
                    continue;
                }
                let Some(complexity) = self.partitions_fit(&groups)? else {
                    continue;
                };
                let shape = (fields.len(), groups.len(), complexity);
                match best_shape {
                    None => {
                        best_shape = Some(shape);
                        best = Some(CandidatePartition { fields, groups });
                        equal_best = 1;
                    }
                    Some(current) if shape < current => {
                        best_shape = Some(shape);
                        best = Some(CandidatePartition { fields, groups });
                        equal_best = 1;
                    }
                    Some(current) if shape == current => {
                        equal_best = equal_best.saturating_add(1);
                    }
                    Some(_) => {}
                }
            }
        }
        self.candidate_evaluations = combinations_checked;
        let Some(best) = best else {
            self.status = ContextDiscoveryStatus::Unavailable;
            return Ok(DiscoveryAttempt::Unavailable);
        };
        if equal_best != 1 {
            self.status = ContextDiscoveryStatus::Ambiguous;
            return Ok(DiscoveryAttempt::Ambiguous);
        }
        let Some(goal) = self.goal.clone() else {
            return Err(DiscoveryError::InvalidTask("goal missing during discovery"));
        };
        let mut partitions = BTreeMap::new();
        self.context_ids.clear();
        self.next_context_id = 1;
        for signature in best.groups.keys() {
            let id = self.context_id_for(signature.clone());
            partitions.insert(id, best.groups[signature].clone());
        }
        let active_signature = self.signature_for_state(
            &self
                .current
                .as_ref()
                .ok_or(DiscoveryError::InvalidTask("current observation missing"))?
                .state,
            Some(&best.fields),
        )?;
        let active_context =
            self.context_ids
                .get(&active_signature)
                .copied()
                .ok_or(DiscoveryError::InvalidTask(
                    "active context not partitioned",
                ))?;
        let current = self
            .current
            .clone()
            .ok_or(DiscoveryError::InvalidTask("current observation missing"))?;
        let rebuilt = ContextualCausalAgent::from_partitioned_traces(
            self.task_template.clone(),
            self.bounds,
            partitions,
            goal,
            Some((active_context, current)),
        );
        self.contextual = match rebuilt {
            Ok(agent) => agent,
            Err(ContextualError::Agent(AgentError::Budget(_))) => {
                self.status = ContextDiscoveryStatus::ResourceExhausted;
                return Ok(DiscoveryAttempt::ResourceExhausted);
            }
            Err(_) => {
                self.status = ContextDiscoveryStatus::Unavailable;
                return Ok(DiscoveryAttempt::Unavailable);
            }
        };
        self.selected_fields = Some(best.fields.clone());
        self.active_signature = Some(active_signature);
        self.status = ContextDiscoveryStatus::Identified {
            fields: best.fields.clone(),
            contexts: best.groups.len(),
        };
        Ok(DiscoveryAttempt::Identified {
            fields: best.fields,
            contexts: best.groups.len(),
        })
    }

    fn partition(
        &self,
        fields: &[TokenId],
    ) -> Result<BTreeMap<ContextSignature, Vec<TransitionExample>>, DiscoveryError> {
        let mut groups: BTreeMap<ContextSignature, Vec<TransitionExample>> = BTreeMap::new();
        for transition in &self.history {
            let signature = self.signature_for_state(&transition.before, Some(fields))?;
            groups
                .entry(signature)
                .or_default()
                .push(transition.clone());
        }
        Ok(groups)
    }

    fn separates_any_conflict(&self, fields: &[TokenId]) -> bool {
        self.history.iter().enumerate().any(|(index, left)| {
            self.history[index + 1..].iter().any(|right| {
                right.action == left.action
                    && right.after != left.after
                    && right.before != left.before
                    && self.signature_for_state(&left.before, Some(fields)).ok()
                        != self.signature_for_state(&right.before, Some(fields)).ok()
            })
        })
    }

    fn partitions_fit(
        &mut self,
        groups: &BTreeMap<ContextSignature, Vec<TransitionExample>>,
    ) -> Result<Option<usize>, DiscoveryError> {
        let mut complexity = 0usize;
        for examples in groups.values() {
            let first = examples
                .first()
                .ok_or(DiscoveryError::InvalidTask("empty candidate partition"))?;
            let task = self.task_template.clone();
            let mut agent = AutonomousCausalAgent::new(
                task,
                AgentObservation {
                    episode: first.episode,
                    sequence: 0,
                    state: first.before.clone(),
                },
                self.bounds,
            )?;
            for example in examples {
                if agent.learn_trace(example.clone())? == EvidenceVerdict::OutOfModelClass {
                    return Ok(None);
                }
            }
            for action in &self.task_template.actions {
                complexity = complexity.saturating_add(
                    agent
                        .candidates(action.id)
                        .ok_or(DiscoveryError::InvalidTask("missing action belief"))?
                        .len(),
                );
            }
        }
        Ok(Some(complexity))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DiscoveryAttempt {
    Identified {
        fields: Vec<TokenId>,
        contexts: usize,
    },
    Ambiguous,
    Unavailable,
    ResourceExhausted,
}

fn map_contextual_verdict(verdict: ContextualVerdict) -> DiscoveryVerdict {
    match verdict {
        ContextualVerdict::Consistent => DiscoveryVerdict::Consistent,
        ContextualVerdict::ContradictionAndLocalRevision => {
            DiscoveryVerdict::ContradictionAndLocalRevision
        }
        ContextualVerdict::TransferFalsified => DiscoveryVerdict::TransferFalsified,
        ContextualVerdict::OutOfModelClass => DiscoveryVerdict::ContextUnavailable,
        ContextualVerdict::EpochRolled {
            previous_epoch,
            new_epoch,
        } => DiscoveryVerdict::EpochRolled {
            previous_epoch,
            new_epoch,
        },
    }
}

fn combinations(values: &[TokenId], size: usize) -> Vec<Vec<TokenId>> {
    fn visit(
        values: &[TokenId],
        start: usize,
        size: usize,
        current: &mut Vec<TokenId>,
        output: &mut Vec<Vec<TokenId>>,
    ) {
        if current.len() == size {
            output.push(current.clone());
            return;
        }
        let remaining = size - current.len();
        if values.len().saturating_sub(start) < remaining {
            return;
        }
        let end = values.len() - remaining;
        for index in start..=end {
            current.push(values[index]);
            visit(values, index + 1, size, current, output);
            current.pop();
        }
    }
    let mut output = Vec::new();
    if size > 0 && size <= values.len() {
        visit(values, 0, size, &mut Vec::new(), &mut output);
    }
    output
}
