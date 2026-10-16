//! Bounded autonomous action learning above the existing causal primitives.
//!
//! Input is a visible typed graph, complete Boolean observations, a goal and
//! a host-approved sandbox action catalogue. No environment, mechanism,
//! intervention setter, slot order or hidden guard crosses this interface.
//!
//! Hypothesis language: identity, or up to a bounded number of constant
//! Boolean assignments guarded by a conjunction of at most two signed,
//! distinct, non-effect fluents. The default remains the one-effect A1
//! grammar; callers can opt into two-effect hypotheses explicitly. This is an
//! explicit inductive bias, NOT human intuition or universal AI. All
//! predictions remain relative to that language and the observed scope.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::cross_domain_causal_core::{
    ActionId, ActionRule, CanonicalCausalGraph, Literal, MappingBudget, MappingVerdict, NodeKind,
    State, TokenId, TransitionExample,
};

#[derive(Clone, Copy, Debug)]
pub struct AgentBounds {
    pub max_fluents: usize,
    pub max_actions: usize,
    /// Maximum number of state fields one action may change in the grammar.
    /// A1 defaults to one; A2 permits two as an explicit bounded extension.
    pub max_effects_per_action: usize,
    pub max_hypotheses_per_action: usize,
    pub max_evidence: usize,
    pub max_events: usize,
    pub max_plan_states: usize,
    pub max_plan_depth: usize,
    pub max_steps_per_episode: usize,
    pub max_evaluations_per_call: usize,
    pub max_snapshot_bytes: usize,
    pub mapping_states: u64,
}

impl Default for AgentBounds {
    fn default() -> Self {
        Self {
            max_fluents: 8,
            max_actions: 16,
            max_effects_per_action: 1,
            max_hypotheses_per_action: 2048,
            max_evidence: 512,
            max_events: 2048,
            max_plan_states: 256,
            max_plan_depth: 8,
            max_steps_per_episode: 128,
            max_evaluations_per_call: 2_000_000,
            max_snapshot_bytes: 1_048_576,
            mapping_states: 16_384,
        }
    }
}

/// Approval is supplied by a trusted sandbox host, never inferred from a
/// learned prediction. This module is NOT a physical-robot safety controller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SandboxAction {
    /// Same opaque ID as the operation node in the visible graph.
    pub id: ActionId,
    pub cost: u32,
    pub sandbox_allowed: bool,
}

#[derive(Clone, Debug)]
pub struct AgentTask {
    pub graph: CanonicalCausalGraph,
    pub actions: Vec<SandboxAction>,
    pub goal: Vec<Literal>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentObservation {
    pub episode: u64,
    pub sequence: u64,
    pub state: State,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ActionReason {
    /// All currently retained models agree on each step of this plan.
    GoalPlanInGrammar,
    /// A source analogy is being tried, NOT granted execution authority.
    StructuralTransferTrial,
    /// A real reachable action partitions the remaining hypothesis set.
    DiscriminatingExperiment,
    /// Consensual prefix to reach a discriminating experiment.
    ReachExperiment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionProposal {
    pub command: u64,
    pub action: ActionId,
    pub reason: ActionReason,
    pub predicted: Option<State>,
    pub worst_case_eliminated: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum AbstainReason {
    WorkBudget,
    StepBudget,
    EvidenceBudget,
    EventBudget,
    SearchBudget,
    NoDistinguishingReachableAction,
    OutOfModelClass,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentDecision {
    GoalObserved,
    Act(ActionProposal),
    Abstain(AbstainReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AgentError {
    InvalidTask(&'static str),
    InvalidObservation(&'static str),
    CommandPending,
    NoPendingCommand,
    WrongCommand,
    StaleObservation,
    InvalidSnapshot,
    Budget(AbstainReason),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum EvidenceVerdict {
    Consistent,
    ContradictionAndLocalRevision,
    TransferFalsified,
    OutOfModelClass,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AgentEvent {
    pub index: usize,
    pub episode: u64,
    pub command: u64,
    pub action: ActionId,
    pub verdict: EvidenceVerdict,
    pub old_candidates: usize,
    pub surviving_candidates: usize,
    pub literal_edits: usize,
    pub epoch: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct AgentTelemetry {
    pub hypothesis_evaluations: u64,
    pub visited_plan_states: u64,
    pub dispatched_proposals: u64,
    pub accepted_observations: u64,
    pub contradictions: u64,
    pub transfer_falsifications: u64,
}

#[derive(Clone, Debug)]
struct ActionBelief {
    candidates: Vec<ActionRule>,
    // Analogy never replaces the full target empirical version space.
    priors: Vec<ActionRule>,
    evidence: Vec<TransitionExample>,
    epoch: u64,
}

impl ActionBelief {
    fn effective(&self) -> &[ActionRule] {
        if self.priors.is_empty() {
            &self.candidates
        } else {
            &self.priors
        }
    }
}

#[derive(Clone, Debug)]
struct Pending {
    proposal: ActionProposal,
    before: AgentObservation,
}

/// The caller owns the environment. `decide` cannot dispatch an action or
/// change any fluent. `observe` accepts only the matching next observation.
#[derive(Clone, Debug)]
pub struct AutonomousCausalAgent {
    task: AgentTask,
    bounds: AgentBounds,
    fluents: Vec<TokenId>,
    beliefs: BTreeMap<ActionId, ActionBelief>,
    observation: AgentObservation,
    pending: Option<Pending>,
    next_command: u64,
    episode_steps: usize,
    events: Vec<AgentEvent>,
    telemetry: AgentTelemetry,
}

fn state_key(state: &State) -> Vec<Literal> {
    state.literals().collect()
}

fn satisfied(state: &State, goal: &[Literal]) -> bool {
    goal.iter()
        .all(|lit| state.get(lit.role) == Some(lit.value))
}

fn enumerate_effect_sets(
    roles: &[TokenId],
    start: usize,
    remaining: usize,
    chosen: &mut Vec<Literal>,
    result: &mut Vec<Vec<Literal>>,
) {
    if remaining == 0 {
        result.push(chosen.clone());
        return;
    }
    if roles.len().saturating_sub(start) < remaining {
        return;
    }
    let last = roles.len() - remaining;
    for index in start..=last {
        for value in [false, true] {
            chosen.push(Literal::new(roles[index], value));
            enumerate_effect_sets(roles, index + 1, remaining - 1, chosen, result);
            chosen.pop();
        }
    }
}

fn effect_sets(roles: &[TokenId], max_effects: usize) -> Vec<Vec<Literal>> {
    let mut result = vec![Vec::new()];
    for size in 1..=max_effects.min(roles.len()) {
        enumerate_effect_sets(roles, 0, size, &mut Vec::new(), &mut result);
    }
    result
}

fn grammar(
    action: ActionId,
    roles: &[TokenId],
    max_effects: usize,
    cap: usize,
) -> Result<Vec<ActionRule>, AgentError> {
    let mut result = Vec::new();
    for effects in effect_sets(roles, max_effects) {
        let mut insert = |condition: Vec<Literal>| -> Result<(), AgentError> {
            if result.len() >= cap {
                return Err(AgentError::InvalidTask("hypothesis bound exceeded"));
            }
            result.push(ActionRule {
                action,
                guards: condition,
                effects: effects.clone(),
            });
            Ok(())
        };

        // Guards on a no-op are observationally irrelevant. Keeping one
        // identity rule avoids a large duplicate family and preserves the
        // original A1 grammar exactly.
        if effects.is_empty() {
            insert(Vec::new())?;
            continue;
        }

        insert(Vec::new())?;
        let effect_roles: BTreeSet<_> = effects.iter().map(|literal| literal.role).collect();
        let guards: Vec<_> = roles
            .iter()
            .filter(|role| !effect_roles.contains(role))
            .flat_map(|role| [Literal::new(*role, false), Literal::new(*role, true)])
            .collect();
        for (i, first) in guards.iter().enumerate() {
            insert(vec![*first])?;
            for second in guards
                .iter()
                .skip(i + 1)
                .filter(|literal| literal.role != first.role)
            {
                insert(vec![*first, *second])?;
            }
        }
    }
    result.sort_by(|a, b| {
        (a.guards.len(), a.effects.len(), &a.guards, &a.effects).cmp(&(
            b.guards.len(),
            b.effects.len(),
            &b.guards,
            &b.effects,
        ))
    });
    Ok(result)
}

struct Work {
    used: usize,
    max: usize,
}
impl Work {
    fn prediction(&mut self, rule: &ActionRule, state: &State) -> Result<State, AgentError> {
        if self.used >= self.max {
            return Err(AgentError::Budget(AbstainReason::WorkBudget));
        }
        self.used += 1;
        Ok(rule.predicts(state))
    }
}

fn consensus(
    rules: &[ActionRule],
    state: &State,
    work: &mut Work,
) -> Result<Option<State>, AgentError> {
    let Some(first) = rules.first() else {
        return Ok(None);
    };
    let predicted = work.prediction(first, state)?;
    for rule in rules.iter().skip(1) {
        if work.prediction(rule, state)? != predicted {
            return Ok(None);
        }
    }
    Ok(Some(predicted))
}

#[derive(Clone)]
struct SearchState {
    state: State,
    path: Vec<ActionId>,
    cost: u64,
}

struct Selection {
    action: ActionId,
    reason: ActionReason,
    predicted: Option<State>,
    eliminated: usize,
}

impl AutonomousCausalAgent {
    pub fn new(
        mut task: AgentTask,
        initial: AgentObservation,
        bounds: AgentBounds,
    ) -> Result<Self, AgentError> {
        if bounds.max_fluents == 0
            || bounds.max_fluents > 8
            || bounds.max_actions == 0
            || bounds.max_actions > 16
            || bounds.max_effects_per_action == 0
            || bounds.max_effects_per_action > 2
            || bounds.max_evidence == 0
            || bounds.max_evidence > 4096
            || bounds.max_events == 0
            || bounds.max_events > 16_384
            || bounds.max_plan_states == 0
            || bounds.max_plan_states > 1024
            || bounds.max_plan_depth == 0
            || bounds.max_plan_depth > 32
            || bounds.max_hypotheses_per_action == 0
            || bounds.max_hypotheses_per_action > 4096
            || bounds.max_steps_per_episode == 0
            || bounds.max_evaluations_per_call == 0
            || bounds.mapping_states == 0
            || bounds.max_snapshot_bytes == 0
            || bounds.max_snapshot_bytes > 4_194_304
        {
            return Err(AgentError::InvalidTask("invalid bounds"));
        }
        // The inherited graph is itself validated; cap it before matching.
        if task.graph.nodes().len() > 32 || task.graph.edges().len() > 96 {
            return Err(AgentError::InvalidTask("visible graph bound exceeded"));
        }
        let fluents: Vec<_> = task
            .graph
            .nodes()
            .iter()
            .filter(|n| n.kind == NodeKind::StateField)
            .map(|n| u64::from(n.id))
            .collect();
        if fluents.is_empty() || fluents.len() > bounds.max_fluents {
            return Err(AgentError::InvalidTask("fluent bound exceeded"));
        }
        let operations: BTreeSet<_> = task
            .graph
            .nodes()
            .iter()
            .filter(|n| n.kind == NodeKind::Operation)
            .map(|n| u64::from(n.id))
            .collect();
        if task.actions.is_empty() || task.actions.len() > bounds.max_actions {
            return Err(AgentError::InvalidTask("action bound exceeded"));
        }
        task.actions.sort_by_key(|a| a.id);
        if task.actions.is_empty()
            || task.actions.len() > bounds.max_actions
            || task.actions.windows(2).any(|w| w[0].id == w[1].id)
            || task
                .actions
                .iter()
                .any(|a| a.cost == 0 || !operations.contains(&a.id))
            || task.goal.is_empty()
            || task.goal.len() > fluents.len()
            || task.goal.iter().any(|g| !fluents.contains(&g.role))
        {
            return Err(AgentError::InvalidTask("invalid actions or goal"));
        }
        task.goal.sort();
        if task.goal.windows(2).any(|w| w[0].role == w[1].role) {
            return Err(AgentError::InvalidTask("duplicate goal fluent"));
        }
        validate_state(&initial.state, &fluents)?;
        let mut beliefs = BTreeMap::new();
        for action in &task.actions {
            beliefs.insert(
                action.id,
                ActionBelief {
                    candidates: grammar(
                        action.id,
                        &fluents,
                        bounds.max_effects_per_action,
                        bounds.max_hypotheses_per_action,
                    )?,
                    priors: Vec::new(),
                    evidence: Vec::new(),
                    epoch: 0,
                },
            );
        }
        Ok(Self {
            task,
            bounds,
            fluents,
            beliefs,
            observation: initial,
            pending: None,
            next_command: 1,
            episode_steps: 0,
            events: Vec::new(),
            telemetry: AgentTelemetry::default(),
        })
    }

    pub fn observation(&self) -> &AgentObservation {
        &self.observation
    }
    pub fn events(&self) -> &[AgentEvent] {
        &self.events
    }
    pub fn telemetry(&self) -> AgentTelemetry {
        self.telemetry
    }
    pub fn candidates(&self, action: ActionId) -> Option<&[ActionRule]> {
        self.beliefs.get(&action).map(|b| b.candidates.as_slice())
    }
    pub fn evidence_count(&self) -> usize {
        self.beliefs.values().map(|b| b.evidence.len()).sum()
    }

    /// Optional tutorial evidence, not a rule or a hidden environment spec.
    /// The evidence is untrusted as a theory: every full transition is checked
    /// against the same grammar. Its factual provenance is the caller's duty.
    /// Tutorials do not move the live robot or change its current observation.
    pub fn learn_trace(
        &mut self,
        example: TransitionExample,
    ) -> Result<EvidenceVerdict, AgentError> {
        if self.pending.is_some() {
            return Err(AgentError::CommandPending);
        }
        validate_state(&example.before, &self.fluents)?;
        validate_state(&example.after, &self.fluents)?;
        if !self.beliefs.contains_key(&example.action) {
            return Err(AgentError::InvalidTask("unknown trace action"));
        }
        let context = Pending {
            proposal: ActionProposal {
                command: 0,
                action: example.action,
                reason: ActionReason::DiscriminatingExperiment,
                predicted: None,
                worst_case_eliminated: 0,
            },
            before: AgentObservation {
                episode: example.episode,
                sequence: 0,
                state: example.before,
            },
        };
        let next = AgentObservation {
            episode: example.episode,
            sequence: 1,
            state: example.after,
        };
        self.record_evidence(&context, &next)
    }

    /// Start another task episode in the SAME visible domain without clearing
    /// evidence. Changing the graph requires `transfer_to`, never silent reset.
    pub fn begin_episode(
        &mut self,
        initial: AgentObservation,
        goal: Vec<Literal>,
    ) -> Result<(), AgentError> {
        if self.pending.is_some() {
            return Err(AgentError::CommandPending);
        }
        if initial.episode <= self.observation.episode {
            return Err(AgentError::StaleObservation);
        }
        validate_state(&initial.state, &self.fluents)?;
        if goal.is_empty()
            || goal.len() > self.fluents.len()
            || goal.iter().any(|g| !self.fluents.contains(&g.role))
            || goal.iter().map(|g| g.role).collect::<BTreeSet<_>>().len() != goal.len()
        {
            return Err(AgentError::InvalidTask("invalid episode goal"));
        }
        self.task.goal = goal;
        self.observation = initial;
        self.episode_steps = 0;
        Ok(())
    }

    pub fn decide(&mut self) -> Result<AgentDecision, AgentError> {
        if self.pending.is_some() {
            return Err(AgentError::CommandPending);
        }
        if satisfied(&self.observation.state, &self.task.goal) {
            return Ok(AgentDecision::GoalObserved);
        }
        if self.episode_steps >= self.bounds.max_steps_per_episode {
            return Ok(AgentDecision::Abstain(AbstainReason::StepBudget));
        }
        if self.evidence_count() >= self.bounds.max_evidence {
            return Ok(AgentDecision::Abstain(AbstainReason::EvidenceBudget));
        }
        if self.events.len() >= self.bounds.max_events {
            return Ok(AgentDecision::Abstain(AbstainReason::EventBudget));
        }
        if self.beliefs.values().any(|b| b.candidates.is_empty()) {
            return Ok(AgentDecision::Abstain(AbstainReason::OutOfModelClass));
        }
        let mut work = Work {
            used: 0,
            max: self.bounds.max_evaluations_per_call,
        };
        let result = self.select(&mut work);
        self.telemetry.hypothesis_evaluations += work.used as u64;
        let selection = match result {
            Ok(Ok(s)) => s,
            Ok(Err(reason)) => return Ok(AgentDecision::Abstain(reason)),
            Err(AgentError::Budget(reason)) => return Ok(AgentDecision::Abstain(reason)),
            Err(e) => return Err(e),
        };
        let proposal = ActionProposal {
            command: self.next_command,
            action: selection.action,
            reason: selection.reason,
            predicted: selection.predicted,
            worst_case_eliminated: selection.eliminated,
        };
        self.next_command = self
            .next_command
            .checked_add(1)
            .ok_or(AgentError::Budget(AbstainReason::StepBudget))?;
        self.episode_steps += 1;
        self.telemetry.dispatched_proposals += 1;
        self.pending = Some(Pending {
            proposal: proposal.clone(),
            before: self.observation.clone(),
        });
        Ok(AgentDecision::Act(proposal))
    }

    fn select(&mut self, work: &mut Work) -> Result<Result<Selection, AbstainReason>, AgentError> {
        // Search only consensus transitions, not a memorized action sequence.
        // Every successor is computed with the existing ActionRule::predicts.
        let mut queue = VecDeque::from([SearchState {
            state: self.observation.state.clone(),
            path: vec![],
            cost: 0,
        }]);
        let mut visited = BTreeSet::from([state_key(&self.observation.state)]);
        let mut best_probe: Option<(usize, u64, usize, ActionId, Selection)> = None;
        let mut truncated = false;
        while let Some(node) = queue.pop_front() {
            self.telemetry.visited_plan_states += 1;
            if satisfied(&node.state, &self.task.goal) && !node.path.is_empty() {
                let action = node.path[0];
                let prior = self.beliefs[&action].priors.len() > 0;
                return Ok(Ok(Selection {
                    action,
                    reason: if prior {
                        ActionReason::StructuralTransferTrial
                    } else {
                        ActionReason::GoalPlanInGrammar
                    },
                    predicted: consensus(
                        self.beliefs[&action].effective(),
                        &self.observation.state,
                        work,
                    )?,
                    eliminated: 0,
                }));
            }
            for action in self.task.actions.iter().filter(|a| a.sandbox_allowed) {
                let belief = &self.beliefs[&action.id];
                if let Some(next) = consensus(belief.effective(), &node.state, work)? {
                    if next != node.state && node.path.len() < self.bounds.max_plan_depth {
                        let key = state_key(&next);
                        if !visited.contains(&key) {
                            if visited.len() >= self.bounds.max_plan_states {
                                truncated = true;
                            } else {
                                visited.insert(key);
                                let mut path = node.path.clone();
                                path.push(action.id);
                                queue.push_back(SearchState {
                                    state: next.clone(),
                                    path,
                                    cost: node.cost + u64::from(action.cost),
                                });
                            }
                        }
                    } else if next != node.state {
                        truncated = true;
                    }
                    // If a transferred prior predicts no effect, do not let
                    // it suppress a target experiment forever. Topology does
                    // not establish behavior, including predicted inaction.
                    if belief.priors.is_empty() || next != node.state {
                        continue;
                    }
                }
                // Exact output partitions, not hashes or fabricated entropy.
                let experiment_rules = if belief.priors.is_empty() {
                    belief.effective()
                } else {
                    &belief.candidates
                };
                let mut partitions = BTreeMap::<Vec<Literal>, usize>::new();
                for candidate in experiment_rules {
                    let key = state_key(&work.prediction(candidate, &node.state)?);
                    *partitions.entry(key).or_default() += 1;
                }
                let worst =
                    experiment_rules.len() - partitions.values().copied().max().unwrap_or(0);
                if worst == 0 {
                    continue;
                }
                let cost = node.cost + u64::from(action.cost);
                let samples = belief.evidence.len();
                let first = node.path.first().copied().unwrap_or(action.id);
                let proposal_belief = &self.beliefs[&first];
                let prior = !proposal_belief.priors.is_empty();
                let predicted = if node.path.is_empty() {
                    // This economical model is a falsifiable proposal, not
                    // identified truth. All other candidates are retained.
                    proposal_belief
                        .effective()
                        .first()
                        .map(|r| work.prediction(r, &self.observation.state))
                        .transpose()?
                } else {
                    consensus(proposal_belief.effective(), &self.observation.state, work)?
                };
                let selection = Selection {
                    action: first,
                    reason: if prior {
                        ActionReason::StructuralTransferTrial
                    } else if node.path.is_empty() {
                        ActionReason::DiscriminatingExperiment
                    } else {
                        ActionReason::ReachExperiment
                    },
                    predicted,
                    eliminated: if node.path.is_empty() { worst } else { 0 },
                };
                // Minimax eliminated hypotheses / actual path+action cost.
                // Ties prefer fewer observations, then deterministic action ID.
                let replace = best_probe
                    .as_ref()
                    .map(|(old, old_cost, old_samples, old_id, _)| {
                        (worst as u128) * (*old_cost as u128) > (*old as u128) * (cost as u128)
                            || ((worst as u128) * (*old_cost as u128)
                                == (*old as u128) * (cost as u128)
                                && (samples, action.id) < (*old_samples, *old_id))
                    })
                    .unwrap_or(true);
                if replace {
                    best_probe = Some((worst, cost, samples, action.id, selection));
                }
            }
        }
        Ok(match best_probe {
            Some((_, _, _, _, selection)) => Ok(selection),
            None => Err(if truncated {
                AbstainReason::SearchBudget
            } else {
                AbstainReason::NoDistinguishingReachableAction
            }),
        })
    }

    /// Observations are bound to the pending command. Invalid or exhausted
    /// inputs cannot partially mutate a belief or clear that pending command.
    pub fn observe(
        &mut self,
        command: u64,
        next: AgentObservation,
    ) -> Result<EvidenceVerdict, AgentError> {
        let pending = self
            .pending
            .as_ref()
            .ok_or(AgentError::NoPendingCommand)?
            .clone();
        if pending.proposal.command != command {
            return Err(AgentError::WrongCommand);
        }
        if next.episode != pending.before.episode
            || pending.before.sequence.checked_add(1) != Some(next.sequence)
        {
            return Err(AgentError::StaleObservation);
        }
        validate_state(&next.state, &self.fluents)?;
        let verdict = self.record_evidence(&pending, &next)?;
        self.observation = next;
        self.pending = None;
        Ok(verdict)
    }

    fn record_evidence(
        &mut self,
        pending: &Pending,
        next: &AgentObservation,
    ) -> Result<EvidenceVerdict, AgentError> {
        if self.evidence_count() >= self.bounds.max_evidence {
            return Err(AgentError::Budget(AbstainReason::EvidenceBudget));
        }
        if self.events.len() >= self.bounds.max_events {
            return Err(AgentError::Budget(AbstainReason::EventBudget));
        }
        let action = pending.proposal.action;
        let belief = &self.beliefs[&action];
        let old_candidates = belief.candidates.len();
        let mut work = Work {
            used: 0,
            max: self.bounds.max_evaluations_per_call,
        };
        let filter =
            |rules: &[ActionRule], work: &mut Work| -> Result<Vec<ActionRule>, AgentError> {
                let mut survivors = Vec::new();
                for rule in rules {
                    if work.prediction(rule, &pending.before.state)? == next.state {
                        survivors.push(rule.clone());
                    }
                }
                Ok(survivors)
            };
        let filtered = (|| -> Result<_, AgentError> {
            Ok((
                filter(&belief.candidates, &mut work)?,
                filter(&belief.priors, &mut work)?,
            ))
        })();
        self.telemetry.hypothesis_evaluations += work.used as u64;
        let (mut candidates, mut priors) = filtered?;
        if let Some(old) = belief.effective().first() {
            // Choose the least edited surviving proposal, but keep the ENTIRE
            // version space. Parsimony never deletes a longer consistent law.
            let rank = |a: &ActionRule, b: &ActionRule| {
                (literal_edits(old, a), a.guards.len(), &a.guards, &a.effects).cmp(&(
                    literal_edits(old, b),
                    b.guards.len(),
                    &b.guards,
                    &b.effects,
                ))
            };
            candidates.sort_by(rank);
            priors.sort_by(rank);
        }
        let transfer_failed = !belief.priors.is_empty() && priors.is_empty();
        let mismatch = pending
            .proposal
            .predicted
            .as_ref()
            .map(|s| s != &next.state)
            .unwrap_or(false);
        let edits = match (
            belief.effective().first(),
            priors.first().or(candidates.first()),
        ) {
            (Some(old), Some(new)) => literal_edits(old, new),
            _ => 0,
        };
        let verdict = if candidates.is_empty() {
            EvidenceVerdict::OutOfModelClass
        } else if transfer_failed {
            EvidenceVerdict::TransferFalsified
        } else if mismatch {
            EvidenceVerdict::ContradictionAndLocalRevision
        } else {
            EvidenceVerdict::Consistent
        };
        let example = TransitionExample::new(
            next.episode,
            action,
            pending.before.state.clone(),
            next.state.clone(),
        );
        let belief = self.beliefs.get_mut(&action).expect("validated action");
        belief.candidates = candidates;
        belief.priors = priors;
        belief.evidence.push(example);
        if mismatch || transfer_failed || verdict == EvidenceVerdict::OutOfModelClass {
            belief.epoch += 1;
        }
        self.events.push(AgentEvent {
            index: self.events.len(),
            episode: next.episode,
            command: pending.proposal.command,
            action,
            verdict,
            old_candidates,
            surviving_candidates: belief.candidates.len(),
            literal_edits: edits,
            epoch: belief.epoch,
        });
        self.telemetry.accepted_observations += 1;
        self.telemetry.contradictions += u64::from(mismatch);
        self.telemetry.transfer_falsifications += u64::from(transfer_failed);
        Ok(verdict)
    }

    /// Remap retained knowledge using the EXISTING exact typed graph matcher.
    /// Refuse ambiguous, incompatible and exhausted matches. Even a unique
    /// structural match creates a trial prior, never a physical execution permit.
    pub fn transfer_to(
        &self,
        task: AgentTask,
        initial: AgentObservation,
    ) -> Result<Self, AgentError> {
        if self.pending.is_some() {
            return Err(AgentError::CommandPending);
        }
        let budget = MappingBudget::new(self.bounds.mapping_states)
            .ok_or(AgentError::InvalidTask("mapping budget"))?;
        let mapping = match self.task.graph.find_unique_mapping(&task.graph, budget) {
            MappingVerdict::Unique(m) => m,
            _ => {
                return Err(AgentError::InvalidTask(
                    "transfer ambiguous, incompatible or over budget",
                ))
            }
        };
        let mut target = Self::new(task, initial, self.bounds)?;
        for (source_action, source_belief) in &self.beliefs {
            // No evidence means no transferable knowledge.
            if source_belief.evidence.is_empty() || source_belief.candidates.is_empty() {
                continue;
            }
            let action_node = u32::try_from(*source_action)
                .map_err(|_| AgentError::InvalidTask("action node overflow"))?;
            let target_action = u64::from(
                mapping
                    .map_node(action_node)
                    .ok_or(AgentError::InvalidTask("unmapped operation"))?,
            );
            let target_belief = target
                .beliefs
                .get_mut(&target_action)
                .ok_or(AgentError::InvalidTask("unmapped catalogue action"))?;
            let priors = source_belief
                .candidates
                .iter()
                .map(|r| r.remap(&mapping, target_action))
                .collect::<Option<Vec<_>>>()
                .ok_or(AgentError::InvalidTask("unmapped fluent"))?;
            target_belief.priors = priors;
        }
        Ok(target)
    }

    /// Exact retained evidence; no examples are evicted and no deserialized
    /// rule or claimed success is trusted. Restore replays every witness.
    pub fn export_memory(&self) -> Result<String, AgentError> {
        if self.pending.is_some() {
            return Err(AgentError::CommandPending);
        }
        let memory = MemoryFile {
            version: 1,
            layout: layout(&self.task, self.bounds.max_effects_per_action),
            evidence: self
                .beliefs
                .values()
                .flat_map(|b| b.evidence.iter())
                .map(PortableEvidence::from)
                .collect(),
        };
        let json = serde_json::to_string(&memory).map_err(|_| AgentError::InvalidSnapshot)?;
        if json.len() > self.bounds.max_snapshot_bytes {
            return Err(AgentError::Budget(AbstainReason::EvidenceBudget));
        }
        Ok(json)
    }

    pub fn restore(
        task: AgentTask,
        initial: AgentObservation,
        bounds: AgentBounds,
        json: &str,
    ) -> Result<Self, AgentError> {
        if json.len() > bounds.max_snapshot_bytes {
            return Err(AgentError::InvalidSnapshot);
        }
        let memory: MemoryFile =
            serde_json::from_str(json).map_err(|_| AgentError::InvalidSnapshot)?;
        let mut agent = Self::new(task, initial, bounds)?;
        if memory.version != 1
            || memory.layout != layout(&agent.task, agent.bounds.max_effects_per_action)
            || memory.evidence.len() > bounds.max_evidence
        {
            return Err(AgentError::InvalidSnapshot);
        }
        let mut work = Work {
            used: 0,
            max: bounds.max_evaluations_per_call,
        };
        for record in memory.evidence {
            let before = portable_state(&record.before, &agent.fluents)?;
            let after = portable_state(&record.after, &agent.fluents)?;
            let belief = agent
                .beliefs
                .get_mut(&record.action)
                .ok_or(AgentError::InvalidSnapshot)?;
            let mut retained = Vec::new();
            for rule in &belief.candidates {
                if work.prediction(rule, &before)? == after {
                    retained.push(rule.clone());
                }
            }
            belief.candidates = retained;
            belief.evidence.push(TransitionExample::new(
                record.episode,
                record.action,
                before,
                after,
            ));
        }
        // An empty frontier is restored as empty: no fabricated recovery.
        agent.telemetry.hypothesis_evaluations = work.used as u64;
        Ok(agent)
    }
}

fn validate_state(state: &State, roles: &[TokenId]) -> Result<(), AgentError> {
    if state.literals().map(|l| l.role).collect::<Vec<_>>() != roles {
        return Err(AgentError::InvalidObservation(
            "complete fixed-schema Boolean state required",
        ));
    }
    Ok(())
}

fn literal_edits(old: &ActionRule, new: &ActionRule) -> usize {
    old.guards
        .iter()
        .filter(|l| !new.guards.contains(l))
        .count()
        + new
            .guards
            .iter()
            .filter(|l| !old.guards.contains(l))
            .count()
        + old
            .effects
            .iter()
            .filter(|l| !new.effects.contains(l))
            .count()
        + new
            .effects
            .iter()
            .filter(|l| !old.effects.contains(l))
            .count()
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VisibleLayout {
    nodes: Vec<(u32, u8, u64)>,
    edges: Vec<(u32, u32, u8)>,
    actions: Vec<ActionId>,
    /// The grammar is part of the memory contract. Old A1 snapshots omitted
    /// this field and are interpreted as the default one-effect grammar.
    #[serde(default = "default_effect_bound")]
    max_effects_per_action: usize,
}

fn default_effect_bound() -> usize {
    1
}

fn layout(task: &AgentTask, max_effects_per_action: usize) -> VisibleLayout {
    use super::cross_domain_causal_core::EdgeKind;
    let nodes = task
        .graph
        .nodes()
        .iter()
        .map(|n| {
            (
                n.id,
                match n.kind {
                    NodeKind::Service => 1,
                    NodeKind::Operation => 2,
                    NodeKind::StateField => 3,
                    NodeKind::Resource => 4,
                    NodeKind::Constraint => 5,
                    NodeKind::Transaction => 6,
                },
                n.type_tag,
            )
        })
        .collect();
    let edges = task
        .graph
        .edges()
        .iter()
        .map(|e| {
            (
                e.from,
                e.to,
                match e.kind {
                    EdgeKind::Calls => 1,
                    EdgeKind::Reads => 2,
                    EdgeKind::Writes => 3,
                    EdgeKind::Requires => 4,
                    EdgeKind::Emits => 5,
                    EdgeKind::DependsOn => 6,
                    EdgeKind::InterventionProduced => 7,
                },
            )
        })
        .collect();
    VisibleLayout {
        nodes,
        edges,
        actions: task.actions.iter().map(|a| a.id).collect(),
        max_effects_per_action,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemoryFile {
    version: u32,
    layout: VisibleLayout,
    evidence: Vec<PortableEvidence>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableEvidence {
    episode: u64,
    action: ActionId,
    before: Vec<(TokenId, bool)>,
    after: Vec<(TokenId, bool)>,
}

impl From<&TransitionExample> for PortableEvidence {
    fn from(e: &TransitionExample) -> Self {
        Self {
            episode: e.episode,
            action: e.action,
            before: e.before.literals().map(|l| (l.role, l.value)).collect(),
            after: e.after.literals().map(|l| (l.role, l.value)).collect(),
        }
    }
}

fn portable_state(values: &[(TokenId, bool)], roles: &[TokenId]) -> Result<State, AgentError> {
    if values.iter().map(|v| v.0).collect::<Vec<_>>() != roles {
        return Err(AgentError::InvalidSnapshot);
    }
    Ok(State::new(
        values
            .iter()
            .map(|(role, value)| Literal::new(*role, *value)),
    ))
}
