//! User-authored causal arena over the bounded autonomous learner.
//!
//! This is the missing integration boundary between a mechanism a visitor
//! writes and the existing A1--A5 learner.  The arena deliberately keeps two
//! objects separate:
//!
//! * `ArenaEnvironment` owns the law and executes an action;
//! * `AutonomousCausalAgent` receives only a typed graph, a complete observed
//!   state, the goal, and the resulting observation.
//!
//! The law is therefore not a rule table handed to the solver.  It is a host
//! side oracle used to answer opaque action-port calls.  The implementation
//! is bounded Boolean causal learning, not unrestricted AGI.  Its value is
//! that a user-authored mechanism exercises the same learner, evidence
//! retention, local revision and typed transfer path as the tests.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::autonomous_causal_agent::{
    AgentBounds, AgentDecision, AgentObservation, AgentTask, AutonomousCausalAgent,
    EvidenceVerdict, SandboxAction,
};
use super::cross_domain_causal_core::{
    ActionId, CausalEdge, CausalNode, CanonicalCausalGraph, EdgeKind, Literal, NodeKind, State,
    TokenId,
};

const MAX_TOKENS: usize = 12;
const MAX_ACTIONS: usize = 12;
const MAX_GUARDS: usize = 2;
const MAX_EFFECTS: usize = 2;
const MAX_EPISODE_STEPS: usize = 64;

/// A user-visible state literal.  Names are only environment labels; the
/// learner sees the corresponding opaque token IDs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArenaLiteral {
    pub token: String,
    pub value: bool,
}

/// A complete Boolean action law owned by the environment.  It is never
/// copied into `AgentTask`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArenaLaw {
    #[serde(default)]
    pub guards: Vec<ArenaLiteral>,
    #[serde(default)]
    pub effects: Vec<ArenaLiteral>,
}

/// A token role is a stable structural slot, not a semantic name.  Two
/// worlds can rename `battery` to `plasma` while preserving `type_tag`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArenaToken {
    pub name: String,
    pub type_tag: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArenaAction {
    pub name: String,
    pub type_tag: u64,
    pub cost: u32,
    pub law: ArenaLaw,
}

/// A complete environment description.  The law is intentionally included
/// in this host-side fixture format so a user can author a level.  The
/// compiler below splits it into a hidden environment and an observation-only
/// `AgentTask` before any decision is requested.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArenaWorld {
    pub name: String,
    pub tokens: Vec<ArenaToken>,
    pub actions: Vec<ArenaAction>,
    pub initial: BTreeMap<String, bool>,
    pub goal: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArenaChallenge {
    pub name: String,
    pub source: ArenaWorld,
    pub target: ArenaWorld,
    pub holdout: ArenaWorld,
    /// Optional law mutation applied only after the source episode.  This is
    /// the explicit sabotage lever for an authored challenge.
    #[serde(default)]
    pub sabotage: Option<ArenaLawOverride>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArenaLawOverride {
    pub action: String,
    pub law: ArenaLaw,
    /// Optional post-sabotage state, applied by the host environment only.
    #[serde(default)]
    pub state: Option<BTreeMap<String, bool>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ArenaStatus {
    Solved,
    RevisedAndSolved,
    TransferVerified,
    Abstained,
    Refused,
}

impl ArenaStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Solved => "SOLVED",
            Self::RevisedAndSolved => "REVISED_AND_SOLVED",
            Self::TransferVerified => "TRANSFER_VERIFIED",
            Self::Abstained => "ABSTAIN",
            Self::Refused => "REFUSED",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArenaEvent {
    pub record: String,
    pub index: usize,
    pub stage: String,
    pub episode: u64,
    pub command: Option<u64>,
    pub action: Option<ActionId>,
    pub action_name: Option<String>,
    pub reason: Option<String>,
    pub verdict: Option<String>,
    pub before_digest: Option<u64>,
    pub predicted_digest: Option<u64>,
    pub observed_digest: Option<u64>,
    pub candidates_before: Option<usize>,
    pub candidates_after: Option<usize>,
    pub epoch: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArenaReport {
    pub status: ArenaStatus,
    pub challenge: String,
    pub source_solved: bool,
    pub sabotage_observed: bool,
    pub contradiction_found: bool,
    pub local_revision: bool,
    pub transfer_first_trial: bool,
    pub holdout_solved: bool,
    pub ledger_valid: bool,
    pub source_evidence: usize,
    pub retained_memory_bytes: usize,
    pub hypothesis_evaluations: u64,
    pub visited_plan_states: u64,
    pub action_count: u64,
    pub report_digest: u64,
    pub events: Vec<ArenaEvent>,
}

/// A non-consuming view used by the live browser arena.  It contains only
/// observations, decisions and bounded work counters; the environment law is
/// never serialized into this snapshot.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ArenaSnapshot {
    pub phase: String,
    pub status: String,
    pub source_solved: bool,
    pub sabotage_observed: bool,
    pub contradiction_found: bool,
    pub local_revision: bool,
    pub transfer_first_trial: bool,
    pub holdout_solved: bool,
    pub source_evidence: usize,
    pub retained_memory_bytes: usize,
    pub hypothesis_evaluations: u64,
    pub visited_plan_states: u64,
    pub action_count: u64,
    pub events: Vec<ArenaEvent>,
}

impl ArenaReport {
    /// Recompute the audit invariants without consulting the environment law
    /// or the learner internals.  This is intentionally a small independent
    /// verifier: it checks event ordering, required action receipts and the
    /// report digest, not whether a hidden outcome was physically truthful.
    pub fn verify(&self) -> bool {
        self.ledger_valid
            && verify_events(&self.events)
            && self.report_digest
                == digest_report(
                    &self.challenge,
                    &self.events,
                    self.source_solved,
                    self.sabotage_observed,
                    self.contradiction_found,
                    self.local_revision,
                    self.transfer_first_trial,
                    self.holdout_solved,
                )
    }

    pub fn succeeded(&self) -> bool {
        self.source_solved
            && self.sabotage_observed
            && self.contradiction_found
            && self.local_revision
            && self.transfer_first_trial
            && self.holdout_solved
            && self.verify()
    }

    /// JSONL is deliberately generated from structured events.  The browser
    /// and a verifier can consume the same evidence without inventing UI
    /// semantics.
    pub fn jsonl(&self) -> String {
        let mut out = String::new();
        for event in &self.events {
            out.push_str(&serde_json::to_string(event).unwrap_or_else(|_| {
                "{\"record\":\"event_encoding_error\"}".to_owned()
            }));
            out.push('\n');
        }
        out.push_str(
            &serde_json::to_string(&serde_json::json!({
                "record": "summary",
                "status": self.status.label(),
                "challenge": &self.challenge,
                "source_solved": self.source_solved,
                "sabotage_observed": self.sabotage_observed,
                "contradiction_found": self.contradiction_found,
                "local_revision": self.local_revision,
                "transfer_first_trial": self.transfer_first_trial,
                "holdout_solved": self.holdout_solved,
                "ledger_valid": self.ledger_valid,
                "source_evidence": self.source_evidence,
                "retained_memory_bytes": self.retained_memory_bytes,
                "hypothesis_evaluations": self.hypothesis_evaluations,
                "visited_plan_states": self.visited_plan_states,
                "action_count": self.action_count,
                "report_digest": self.report_digest,
            }))
            .unwrap_or_else(|_| "{\"record\":\"summary_encoding_error\"}".to_owned()),
        );
        out.push('\n');
        out
    }
}

#[derive(Clone, Debug)]
struct CompiledWorld {
    name: String,
    graph: CanonicalCausalGraph,
    token_ids: BTreeMap<String, TokenId>,
    action_ids: BTreeMap<String, ActionId>,
    action_costs: BTreeMap<ActionId, u32>,
    laws: BTreeMap<ActionId, CompiledLaw>,
    initial: State,
    goal: Vec<Literal>,
}

#[derive(Clone, Debug)]
struct CompiledLaw {
    guards: Vec<Literal>,
    effects: Vec<Literal>,
}

impl CompiledWorld {
    fn task(&self) -> AgentTask {
        AgentTask {
            graph: self.graph.clone(),
            actions: self
                .action_ids
                .values()
                .map(|id| SandboxAction {
                    id: *id,
                    cost: *self.action_costs.get(id).expect("compiled action cost"),
                    sandbox_allowed: true,
                })
                .collect(),
            goal: self.goal.clone(),
        }
    }

    fn action_name(&self, action: ActionId) -> Option<String> {
        self.action_ids
            .iter()
            .find_map(|(name, id)| (*id == action).then_some(name.clone()))
    }
}

#[derive(Clone, Debug)]
struct ArenaEnvironment {
    world: CompiledWorld,
    state: State,
}

impl ArenaEnvironment {
    fn new(world: CompiledWorld) -> Self {
        let state = world.initial.clone();
        Self { world, state }
    }

    fn reset(&mut self) {
        self.state = self.world.initial.clone();
    }

    fn set_law(&mut self, action: ActionId, law: CompiledLaw) -> Result<(), String> {
        if !self.world.laws.contains_key(&action) {
            return Err("sabotage references an unknown action".to_owned());
        }
        self.world.laws.insert(action, law);
        Ok(())
    }

    fn apply(&mut self, action: ActionId) -> State {
        let before = self.state.clone();
        let Some(law) = self.world.laws.get(&action) else {
            return before;
        };
        if law
            .guards
            .iter()
            .all(|guard| before.get(guard.role) == Some(guard.value))
        {
            self.state = law
                .effects
                .iter()
                .fold(before, |state, effect| state.with(*effect));
        }
        self.state.clone()
    }
}

#[derive(Clone, Debug)]
pub struct CognitiveArenaSession {
    challenge_name: String,
    environment: ArenaEnvironment,
    learner: AutonomousCausalAgent,
    episode: u64,
    sequence: u64,
    events: Vec<ArenaEvent>,
    source_solved: bool,
    sabotage_observed: bool,
    contradiction_found: bool,
    local_revision: bool,
    transfer_first_trial: bool,
    holdout_solved: bool,
    retained_source_evidence: usize,
    retained_action_count: u64,
    retained_hypothesis_evaluations: u64,
    retained_visited_plan_states: u64,
    retained_memory_bytes: usize,
}

fn stable_id(value: &str, salt: u64) -> u64 {
    let mut state = 14_695_981_039_346_656_037u64 ^ salt;
    for byte in value.as_bytes() {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(1_099_511_628_211);
    }
    state.max(1)
}

fn validate_literal_list(
    literals: &[ArenaLiteral],
    token_ids: &BTreeMap<String, TokenId>,
    limit: usize,
    label: &str,
) -> Result<Vec<Literal>, String> {
    if literals.len() > limit {
        return Err(format!("{label} exceeds bounded limit {limit}"));
    }
    let mut seen = BTreeSet::new();
    literals
        .iter()
        .map(|literal| {
            let token = *token_ids
                .get(&literal.token)
                .ok_or_else(|| format!("{label} references unknown token `{}`", literal.token))?;
            if !seen.insert(token) {
                return Err(format!("{label} contains duplicate token `{}`", literal.token));
            }
            Ok(Literal::new(token, literal.value))
        })
        .collect()
}

fn compile_world(world: &ArenaWorld, _salt: u64) -> Result<CompiledWorld, String> {
    if world.name.trim().is_empty() {
        return Err("world name is required".to_owned());
    }
    if world.tokens.is_empty() || world.tokens.len() > MAX_TOKENS {
        return Err(format!("world must contain 1..={MAX_TOKENS} tokens"));
    }
    if world.actions.is_empty() || world.actions.len() > MAX_ACTIONS {
        return Err(format!("world must contain 1..={MAX_ACTIONS} actions"));
    }
    let mut token_ids = BTreeMap::new();
    let mut nodes = Vec::with_capacity(world.tokens.len() + world.actions.len());
    for (index, token) in world.tokens.iter().enumerate() {
        if token.name.trim().is_empty() || token_ids.contains_key(&token.name) {
            return Err(format!("duplicate or empty token `{}`", token.name));
        }
        let id = u64::try_from(index + 1).map_err(|_| "token id overflow".to_owned())?;
        token_ids.insert(token.name.clone(), id);
        nodes.push(CausalNode {
            id: id as u32,
            kind: NodeKind::StateField,
            type_tag: token.type_tag,
        });
    }

    let action_base = world.tokens.len() + 1;
    let mut action_ids = BTreeMap::new();
    let mut action_costs = BTreeMap::new();
    for (index, action) in world.actions.iter().enumerate() {
        if action.name.trim().is_empty() || action_ids.contains_key(&action.name) {
            return Err(format!("duplicate or empty action `{}`", action.name));
        }
        if action.cost == 0 {
            return Err(format!("action `{}` has zero cost", action.name));
        }
        let node_id = u32::try_from(action_base + index)
            .map_err(|_| "action node id overflow".to_owned())?;
        // The action-port contract requires the opaque action ID to equal the
        // corresponding operation node ID in the visible graph.  The mapper
        // remaps this local ID across worlds; the user-facing name remains
        // private to the host environment.
        let action_id = u64::from(node_id);
        if action_ids.values().any(|existing| *existing == action_id) {
            return Err("action id collision; rename actions or change the salt".to_owned());
        }
        action_ids.insert(action.name.clone(), action_id);
        action_costs.insert(action_id, action.cost);
        nodes.push(CausalNode {
            id: node_id,
            kind: NodeKind::Operation,
            type_tag: action.type_tag,
        });
    }

    // The visible graph reveals only typed participation.  It deliberately
    // does not encode which guards are currently true or which effect wins.
    let mut edges = Vec::new();
    for (index, _) in world.actions.iter().enumerate() {
        let action_node = (action_base + index) as u32;
        for token_node in 1..=world.tokens.len() as u32 {
            edges.push(CausalEdge {
                from: action_node,
                to: token_node,
                kind: EdgeKind::DependsOn,
            });
        }
    }
    let graph = CanonicalCausalGraph::new(nodes, edges)
        .map_err(|error| format!("invalid arena graph: {error:?}"))?;

    let initial = state_from_map(&world.initial, &token_ids, "initial")?;
    let goal = world
        .goal
        .iter()
        .map(|(name, value)| {
            token_ids
                .get(name)
                .copied()
                .map(|id| Literal::new(id, *value))
                .ok_or_else(|| format!("goal references unknown token `{name}`"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if goal.is_empty() || goal.len() > world.tokens.len() {
        return Err("goal must contain at least one token".to_owned());
    }

    let mut laws = BTreeMap::new();
    for action in &world.actions {
        let action_id = action_ids[&action.name];
        let guards = validate_literal_list(&action.law.guards, &token_ids, MAX_GUARDS, "guards")?;
        let effects = validate_literal_list(&action.law.effects, &token_ids, MAX_EFFECTS, "effects")?;
        if effects.is_empty() {
            return Err(format!("action `{}` must have an effect", action.name));
        }
        laws.insert(action_id, CompiledLaw { guards, effects });
    }
    Ok(CompiledWorld {
        name: world.name.clone(),
        graph,
        token_ids,
        action_ids,
        action_costs,
        laws,
        initial,
        goal,
    })
}

fn state_from_map(
    values: &BTreeMap<String, bool>,
    token_ids: &BTreeMap<String, TokenId>,
    label: &str,
) -> Result<State, String> {
    if values.len() != token_ids.len() {
        return Err(format!(
            "{label} must define all {} tokens; got {}",
            token_ids.len(),
            values.len()
        ));
    }
    let mut literals = Vec::with_capacity(values.len());
    for (name, value) in values {
        let id = token_ids
            .get(name)
            .copied()
            .ok_or_else(|| format!("{label} references unknown token `{name}`"))?;
        literals.push(Literal::new(id, *value));
    }
    Ok(State::new(literals))
}

fn compile_law(world: &CompiledWorld, law: &ArenaLaw) -> Result<CompiledLaw, String> {
    Ok(CompiledLaw {
        guards: validate_literal_list(&law.guards, &world.token_ids, MAX_GUARDS, "sabotage guards")?,
        effects: validate_literal_list(&law.effects, &world.token_ids, MAX_EFFECTS, "sabotage effects")?,
    })
}

fn push_event(events: &mut Vec<ArenaEvent>, mut event: ArenaEvent) {
    event.index = events.len();
    events.push(event);
}

fn stage_event(stage: &str, episode: u64) -> ArenaEvent {
    ArenaEvent {
        record: "stage".to_owned(),
        index: 0,
        stage: stage.to_owned(),
        episode,
        command: None,
        action: None,
        action_name: None,
        reason: None,
        verdict: None,
        before_digest: None,
        predicted_digest: None,
        observed_digest: None,
        candidates_before: None,
        candidates_after: None,
        epoch: None,
    }
}

impl CognitiveArenaSession {
    pub fn new(challenge: &ArenaChallenge) -> Result<Self, String> {
        let source = compile_world(&challenge.source, 0x534f55524345);
        let target = compile_world(&challenge.target, 0x544152474554);
        let holdout = compile_world(&challenge.holdout, 0x484f4c444f5554);
        let (source, target, holdout) = match (source, target, holdout) {
            (Ok(source), Ok(target), Ok(holdout)) => (source, target, holdout),
            (source, target, holdout) => {
                return Err(format!(
                    "challenge worlds invalid: source={source:?}, target={target:?}, holdout={holdout:?}"
                ));
            }
        };
        let initial = AgentObservation {
            episode: 1,
            sequence: 0,
            state: source.initial.clone(),
        };
        let learner = AutonomousCausalAgent::new(
            source.task(),
            initial,
            AgentBounds {
                max_steps_per_episode: MAX_EPISODE_STEPS,
                max_effects_per_action: 2,
                ..AgentBounds::default()
            },
        )
        .map_err(|error| format!("learner init: {error:?}"))?;
        let mut events = Vec::new();
        let mut observe = stage_event("OBSERVE", 1);
        observe.reason = Some(format!("source world `{}` exposed through opaque state/action port", source.name));
        push_event(&mut events, observe);
        let mut abstracted = stage_event("ABSTRACT", 1);
        abstracted.reason = Some("solver starts with a bounded version space; no law was injected".to_owned());
        push_event(&mut events, abstracted);
        // Keep target and holdout compiled alive through the challenge object
        // during transfer; the initial session owns only the source world.
        let _ = (target, holdout);
        Ok(Self {
            challenge_name: challenge.name.clone(),
            environment: ArenaEnvironment::new(source),
            learner,
            episode: 1,
            sequence: 0,
            events,
            source_solved: false,
            sabotage_observed: false,
            contradiction_found: false,
            local_revision: false,
            transfer_first_trial: false,
            holdout_solved: false,
            retained_source_evidence: 0,
            retained_action_count: 0,
            retained_hypothesis_evaluations: 0,
            retained_visited_plan_states: 0,
            retained_memory_bytes: 0,
        })
    }

    pub fn learner(&self) -> &AutonomousCausalAgent {
        &self.learner
    }

    pub fn events(&self) -> &[ArenaEvent] {
        &self.events
    }

    /// Advance exactly one real learner/environment interaction.  The caller
    /// may sabotage between calls; no pre-recorded sequence is consumed.
    pub fn step(&mut self) -> Result<bool, String> {
        let solved = self.run_episode(1)?;
        if solved {
            self.source_solved = true;
        }
        Ok(solved)
    }

    pub fn snapshot(&self) -> ArenaSnapshot {
        let telemetry = self.learner.telemetry();
        let memory = self.learner.export_memory().unwrap_or_default();
        let phase = if self.holdout_solved && self.transfer_first_trial {
            "HOLDOUT_VERIFIED"
        } else if self.holdout_solved {
            "HOLDOUT_SOLVED"
        } else if self.transfer_first_trial {
            "TRANSFER_SOLVED"
        } else if self.local_revision {
            "REVISED"
        } else if self.contradiction_found {
            "CONTRADICTION_FOUND"
        } else if self.sabotage_observed {
            "SABOTAGED"
        } else if self.source_solved {
            "SOURCE_SOLVED"
        } else if self.events.iter().any(|event| event.stage == "ACTION") {
            "OBSERVING"
        } else {
            "READY"
        };
        let status = if self.holdout_solved && self.transfer_first_trial {
            ArenaStatus::TransferVerified
        } else if self.local_revision && self.source_solved {
            ArenaStatus::RevisedAndSolved
        } else if self.source_solved {
            ArenaStatus::Solved
        } else if self.sabotage_observed {
            ArenaStatus::Abstained
        } else {
            ArenaStatus::Refused
        };
        ArenaSnapshot {
            phase: phase.to_owned(),
            status: status.label().to_owned(),
            source_solved: self.source_solved,
            sabotage_observed: self.sabotage_observed,
            contradiction_found: self.contradiction_found,
            local_revision: self.local_revision,
            transfer_first_trial: self.transfer_first_trial,
            holdout_solved: self.holdout_solved,
            source_evidence: self
                .retained_source_evidence
                .saturating_add(telemetry.accepted_observations as usize),
            retained_memory_bytes: self.retained_memory_bytes.saturating_add(memory.len()),
            hypothesis_evaluations: self
                .retained_hypothesis_evaluations
                .saturating_add(telemetry.hypothesis_evaluations),
            visited_plan_states: self
                .retained_visited_plan_states
                .saturating_add(telemetry.visited_plan_states),
            action_count: self
                .retained_action_count
                .saturating_add(telemetry.dispatched_proposals),
            events: self.events.clone(),
        }
    }

    pub fn snapshot_json(&self) -> String {
        serde_json::to_string(&self.snapshot())
            .unwrap_or_else(|_| "{\"phase\":\"ENCODING_ERROR\"}".to_owned())
    }

    pub fn run_episode(&mut self, max_steps: usize) -> Result<bool, String> {
        for _ in 0..max_steps {
            let decision = self
                .learner
                .decide()
                .map_err(|error| format!("decide: {error:?}"))?;
            match decision {
                AgentDecision::GoalObserved => {
                    push_event(&mut self.events, stage_event("GOAL_OBSERVED", self.episode));
                    return Ok(true);
                }
                AgentDecision::Abstain(reason) => {
                    let mut event = stage_event("ABSTAIN", self.episode);
                    event.reason = Some(format!("{reason:?}"));
                    push_event(&mut self.events, event);
                    return Ok(false);
                }
                AgentDecision::Act(proposal) => {
                    let before = self.environment.state.clone();
                    let predicted = proposal.predicted.clone();
                    let action_name = self.environment.world.action_name(proposal.action);
                    let after = self.environment.apply(proposal.action);
                    self.sequence = self.sequence.saturating_add(1);
                    let verdict = self
                        .learner
                        .observe(
                            proposal.command,
                            AgentObservation {
                                episode: self.episode,
                                sequence: self.sequence,
                                state: after.clone(),
                            },
                        )
                        .map_err(|error| format!("observe: {error:?}"))?;
                    let mut event = stage_event("ACTION", self.episode);
                    event.command = Some(proposal.command);
                    event.action = Some(proposal.action);
                    event.action_name = action_name;
                    event.reason = Some(format!("{:?}", proposal.reason));
                    event.verdict = Some(format!("{verdict:?}"));
                    event.before_digest = Some(before.digest());
                    event.predicted_digest = predicted.as_ref().map(State::digest);
                    event.observed_digest = Some(after.digest());
                    if let Some(agent_event) = self.learner.events().last() {
                        event.candidates_before = Some(agent_event.old_candidates);
                        event.candidates_after = Some(agent_event.surviving_candidates);
                        event.epoch = Some(agent_event.epoch);
                    }
                    if matches!(verdict, EvidenceVerdict::ContradictionAndLocalRevision) {
                        self.contradiction_found = true;
                    }
                    if matches!(verdict, EvidenceVerdict::ContradictionAndLocalRevision)
                        || matches!(verdict, EvidenceVerdict::TransferFalsified)
                    {
                        self.local_revision = true;
                    }
                    push_event(&mut self.events, event);
                }
            }
        }
        Ok(false)
    }

    pub fn sabotage(&mut self, override_law: &ArenaLawOverride) -> Result<(), String> {
        if self.episode != 1 || self.learner.observation().sequence == 0 {
            return Err("sabotage requires a completed source interaction".to_owned());
        }
        let action = *self
            .environment
            .world
            .action_ids
            .get(&override_law.action)
            .ok_or_else(|| "sabotage action is unknown".to_owned())?;
        let law = compile_law(&self.environment.world, &override_law.law)?;
        self.environment.set_law(action, law)?;
        self.episode = 2;
        self.sequence = 0;
        self.environment.reset();
        if let Some(values) = &override_law.state {
            self.environment.state = state_from_map(
                values,
                &self.environment.world.token_ids,
                "sabotage state",
            )?;
        }
        self.learner
            .begin_episode(
                AgentObservation {
                    episode: self.episode,
                    sequence: 0,
                    state: self.environment.state.clone(),
                },
                self.environment.world.goal.clone(),
            )
            .map_err(|error| format!("begin sabotaged episode: {error:?}"))?;
        self.sabotage_observed = true;
        let mut event = stage_event("SABOTAGE", self.episode);
        event.action = Some(action);
        event.action_name = Some(override_law.action.clone());
        event.reason = Some("environment law changed without notifying the solver".to_owned());
        push_event(&mut self.events, event);
        Ok(())
    }

    pub fn transfer_to_world(
        &self,
        world: &ArenaWorld,
        salt: u64,
    ) -> Result<CognitiveArenaSession, String> {
        let telemetry = self.learner.telemetry();
        let memory_bytes = self.learner.export_memory().unwrap_or_default().len();
        let target = compile_world(world, salt)?;
        let initial = AgentObservation {
            episode: 1,
            sequence: 0,
            state: target.initial.clone(),
        };
        let learner = self
            .learner
            .transfer_to(target.task(), initial)
            .map_err(|error| format!("structural transfer refused: {error:?}"))?;
        let mut events = self.events.clone();
        let mut event = stage_event("TRANSFER", 1);
        event.reason = Some("unique typed mapping installed; target behavior remains untrusted".to_owned());
        push_event(&mut events, event);
        Ok(Self {
            challenge_name: self.challenge_name.clone(),
            environment: ArenaEnvironment::new(target),
            learner,
            episode: 1,
            sequence: 0,
            events,
            source_solved: self.source_solved,
            sabotage_observed: self.sabotage_observed,
            contradiction_found: self.contradiction_found,
            local_revision: self.local_revision,
            transfer_first_trial: self.transfer_first_trial,
            holdout_solved: false,
            retained_source_evidence: self
                .retained_source_evidence
                .saturating_add(telemetry.accepted_observations as usize),
            retained_action_count: self
                .retained_action_count
                .saturating_add(telemetry.dispatched_proposals),
            retained_hypothesis_evaluations: self
                .retained_hypothesis_evaluations
                .saturating_add(telemetry.hypothesis_evaluations),
            retained_visited_plan_states: self
                .retained_visited_plan_states
                .saturating_add(telemetry.visited_plan_states),
            retained_memory_bytes: self.retained_memory_bytes.saturating_add(memory_bytes),
        })
    }

    pub fn mark_source_solved(&mut self, solved: bool) {
        self.source_solved = solved;
    }

    pub fn mark_transfer_first_trial(&mut self) {
        self.transfer_first_trial = true;
    }

    pub fn mark_holdout_solved(&mut self, solved: bool) {
        self.holdout_solved = solved;
    }

    pub fn report(self) -> ArenaReport {
        let telemetry = self.learner.telemetry();
        let memory = self.learner.export_memory().unwrap_or_default();
        let ledger_valid = verify_events(&self.events);
        let report_digest = digest_report(
            &self.challenge_name,
            &self.events,
            self.source_solved,
            self.sabotage_observed,
            self.contradiction_found,
            self.local_revision,
            self.transfer_first_trial,
            self.holdout_solved,
        );
        let status = if self.holdout_solved && self.transfer_first_trial {
            ArenaStatus::TransferVerified
        } else if self.local_revision && self.source_solved {
            ArenaStatus::RevisedAndSolved
        } else if self.source_solved {
            ArenaStatus::Solved
        } else if self.sabotage_observed {
            ArenaStatus::Abstained
        } else {
            ArenaStatus::Refused
        };
        ArenaReport {
            status,
            challenge: self.challenge_name,
            source_solved: self.source_solved,
            sabotage_observed: self.sabotage_observed,
            contradiction_found: self.contradiction_found,
            local_revision: self.local_revision,
            transfer_first_trial: self.transfer_first_trial,
            holdout_solved: self.holdout_solved,
            ledger_valid,
            source_evidence: self
                .retained_source_evidence
                .saturating_add(telemetry.accepted_observations as usize),
            retained_memory_bytes: self.retained_memory_bytes.saturating_add(memory.len()),
            hypothesis_evaluations: self
                .retained_hypothesis_evaluations
                .saturating_add(telemetry.hypothesis_evaluations),
            visited_plan_states: self
                .retained_visited_plan_states
                .saturating_add(telemetry.visited_plan_states),
            action_count: self
                .retained_action_count
                .saturating_add(telemetry.dispatched_proposals),
            report_digest,
            events: self.events,
        }
    }
}

fn verify_events(events: &[ArenaEvent]) -> bool {
    if events.is_empty() || events.iter().enumerate().any(|(index, event)| event.index != index) {
        return false;
    }
    let mut saw_action = false;
    let mut saw_sabotage = false;
    for event in events {
        match event.stage.as_str() {
            "ACTION" => saw_action = true,
            "SABOTAGE" => {
                if !saw_action {
                    return false;
                }
                saw_sabotage = true;
            }
            "TRANSFER" => {
                if !saw_action {
                    return false;
                }
            }
            _ => {}
        }
        if event.stage == "ACTION"
            && (event.command.is_none()
                || event.action.is_none()
                || event.before_digest.is_none()
                || event.observed_digest.is_none())
        {
            return false;
        }
    }
    !saw_sabotage || saw_action
}

fn digest_report(
    challenge: &str,
    events: &[ArenaEvent],
    source_solved: bool,
    sabotage_observed: bool,
    contradiction_found: bool,
    local_revision: bool,
    transfer_first_trial: bool,
    holdout_solved: bool,
) -> u64 {
    let mut state = stable_id(challenge, 0x4152454e41);
    for event in events {
        state ^= stable_id(&event.stage, event.index as u64 + 1).rotate_left(7);
        state = state.wrapping_mul(1_099_511_628_211);
        state ^= event.command.unwrap_or_default();
        state ^= event.action.unwrap_or_default().rotate_left(11);
        state ^= event.observed_digest.unwrap_or_default().rotate_left(17);
    }
    for flag in [
        source_solved,
        sabotage_observed,
        contradiction_found,
        local_revision,
        transfer_first_trial,
        holdout_solved,
    ] {
        state = state.rotate_left(5) ^ u64::from(flag);
    }
    state.max(1)
}

/// Deterministic, deliberately non-canned fixture used by the native
/// verifier and as a default challenge for the browser.
pub fn generated_challenge(seed: u64) -> ArenaChallenge {
    let action = |prefix: &str, slot: usize, tag: u64, guards: Vec<ArenaLiteral>, effect: ArenaLiteral| {
        ArenaAction {
            name: format!("{prefix}_{seed:08x}_{slot}"),
            type_tag: tag,
            cost: 1,
            law: ArenaLaw { guards, effects: vec![effect] },
        }
    };
    let make_world = |prefix: &str, initial: [bool; 3], goal: bool| {
        let names = [
            format!("{prefix}_{seed:08x}_energy"),
            format!("{prefix}_{seed:08x}_gate"),
            format!("{prefix}_{seed:08x}_latch"),
        ];
        let tokens = names
            .iter()
            .zip([101u64, 102, 103])
            .map(|(name, type_tag)| ArenaToken {
                name: name.clone(),
                type_tag,
            })
            .collect::<Vec<_>>();
        let literals = |index: usize, value: bool| ArenaLiteral { token: names[index].clone(), value };
        let mut actions = vec![
            action("charge", 0, 201, vec![], literals(0, true)),
            action(
                "open",
                1,
                202,
                vec![literals(0, true)],
                literals(1, goal),
            ),
            action("latch", 2, 203, vec![], literals(2, true)),
        ];
        // The repair action is intentionally more expensive.  After the law
        // drift the minimax selector must first test the cheap `open` action,
        // exposing the contradiction before it spends work repairing the
        // interlock.
        actions[2].cost = 10;
        let mut initial_map = BTreeMap::new();
        for (name, value) in names.iter().zip(initial) {
            initial_map.insert(name.clone(), value);
        }
        let mut goal_map = BTreeMap::new();
        goal_map.insert(names[1].clone(), goal);
        ArenaWorld {
            name: format!("{prefix}_{seed:08x}"),
            tokens,
            actions,
            initial: initial_map,
            goal: goal_map,
        }
    };
    let source = make_world("source", [false, false, true], true);
    let target = make_world("target", [true, false, true], true);
    let holdout = make_world("holdout", [true, false, true], true);
    let open_name = source.actions[1].name.clone();
    let source_names = source.tokens.iter().map(|token| token.name.clone()).collect::<Vec<_>>();
    ArenaChallenge {
        name: format!("fresh-world-{seed:08x}"),
        source,
        target,
        holdout,
        sabotage: Some(ArenaLawOverride {
            action: open_name,
            law: ArenaLaw {
                guards: vec![
                    ArenaLiteral { token: source_names[0].clone(), value: true },
                    ArenaLiteral { token: source_names[2].clone(), value: true },
                ],
                effects: vec![ArenaLiteral { token: source_names[1].clone(), value: true }],
            },
            state: Some({
                let mut state = BTreeMap::new();
                state.insert(source_names[0].clone(), false);
                state.insert(source_names[1].clone(), false);
                state.insert(source_names[2].clone(), false);
                state
            }),
        }),
    }
}

/// Parse a user-authored challenge at the boundary and execute it through the
/// same host-owned environment/solver split as the generated fixture.
pub fn parse_challenge_json(input: &str) -> Result<ArenaChallenge, String> {
    serde_json::from_str(input).map_err(|error| format!("invalid arena JSON: {error}"))
}

pub fn run_challenge_json(input: &str) -> Result<ArenaReport, String> {
    let challenge = parse_challenge_json(input)?;
    let sabotage = challenge
        .sabotage
        .as_ref()
        .ok_or_else(|| "challenge must declare a sabotage law override".to_owned())?;
    let target = challenge.target.clone();
    let holdout = challenge.holdout.clone();
    let mut source = CognitiveArenaSession::new(&challenge)?;
    let source_solved = source.run_episode(MAX_EPISODE_STEPS)?;
    source.mark_source_solved(source_solved);
    source.sabotage(sabotage)?;
    let _ = source.run_episode(MAX_EPISODE_STEPS)?;
    let mut target_session = source.transfer_to_world(&target, 0x544152474554)?;
    let target_event_start = target_session.events.len();
    let target_solved = target_session.run_episode(MAX_EPISODE_STEPS)?;
    let target_first = target_session.events[target_event_start..]
        .iter()
        .find(|event| event.stage == "ACTION")
        .and_then(|event| event.reason.as_deref())
        == Some("StructuralTransferTrial");
    if target_first && target_solved {
        target_session.mark_transfer_first_trial();
    }
    let mut holdout_session = target_session.transfer_to_world(&holdout, 0x484f4c444f5554)?;
    let holdout_solved = holdout_session.run_episode(MAX_EPISODE_STEPS)?;
    holdout_session.mark_holdout_solved(holdout_solved);
    Ok(holdout_session.report())
}

/// Run the full source -> sabotage -> revise -> transfer -> holdout path.
/// This is intentionally an orchestration function, not a pre-recorded log:
/// every `ACTION` event is emitted only after the real learner decides and the
/// host environment returns an observation.
pub fn run_generated(seed: u64) -> Result<ArenaReport, String> {
    let challenge = generated_challenge(seed);
    run_challenge_json(&serde_json::to_string(&challenge).map_err(|error| error.to_string())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_world_is_solved_by_real_decisions() {
        let report = run_generated(0xA17E_2026).expect("generated arena");
        assert!(report.source_solved, "{}", report.jsonl());
        assert!(report.verify(), "{}", report.jsonl());
        assert!(report.succeeded(), "{}", report.jsonl());
        assert!(report.events.iter().any(|event| event.stage == "ACTION"));
    }

    #[test]
    fn sabotage_is_observed_and_old_evidence_is_retained() {
        let report = run_generated(17).expect("generated arena");
        assert!(report.sabotage_observed);
        assert!(report.contradiction_found, "{}", report.jsonl());
        assert!(report.local_revision, "{}", report.jsonl());
        assert!(report.source_evidence >= 3);
    }

    #[test]
    fn fresh_names_do_not_change_the_structural_law_class() {
        let first = generated_challenge(1);
        let second = generated_challenge(2);
        assert_ne!(first.source.name, second.source.name);
        let first_report = run_generated(1).expect("first");
        let second_report = run_generated(2).expect("second");
        assert!(first_report.succeeded(), "{}", first_report.jsonl());
        assert!(second_report.succeeded(), "{}", second_report.jsonl());
        assert_ne!(first_report.report_digest, second_report.report_digest);
    }

    #[test]
    fn verifier_rejects_incomplete_action_event() {
        let mut report = run_generated(9).expect("generated arena");
        let index = report
            .events
            .iter()
            .position(|event| event.stage == "ACTION")
            .expect("action");
        report.events[index].command = None;
        assert!(!verify_events(&report.events));
    }

    #[test]
    fn stateful_session_changes_only_after_real_calls() {
        let challenge = generated_challenge(0xBEEF);
        let mut session = CognitiveArenaSession::new(&challenge).expect("session");
        assert_eq!(session.snapshot().phase, "READY");
        let initial_event_count = session.events().len();

        session.step().expect("first live step");
        assert!(session.events().len() > initial_event_count);
        assert!(session.snapshot().source_evidence > 0);

        session
            .sabotage(challenge.sabotage.as_ref().expect("sabotage"))
            .expect("live sabotage");
        let after_sabotage = session.snapshot();
        assert!(after_sabotage.sabotage_observed);
        assert!(after_sabotage.events.iter().any(|event| event.stage == "SABOTAGE"));
    }
}
