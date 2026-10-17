//! Black-box action-port boundary for causal learning.
//!
//! `CustomChallengeSpec` is useful as a fixture description, but it is too
//! close to a graph-level API for a public intelligence claim.  This module
//! inserts the important boundary: the environment owns the world definition
//! and the hidden guard, while the solver receives only opaque observations,
//! an opaque action token, and the resulting observation.
//!
//! The environment implementation below is deliberately private.  Its public
//! contract is `OpaqueActionPort`; `OpaqueActionSolver` never accepts a world
//! specification, a guard, or a graph.  It reconstructs a bounded structural
//! representation from interaction frames before delegating the verified
//! causal cycle to `CrossDomainCausalCore`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use super::causal_proof::FrozenTransferModel;
use super::cross_domain_causal_core::{
    ActionId, CanonicalCausalGraph, CausalEdge, CausalLearner, CausalNode, CrossDomainCausalCore,
    Digest, EdgeKind, Literal, MappingBudget, MappingVerdict, NodeId, NodeKind, ObservationVerdict,
    State, TransferStatus, TransitionExample,
};
use super::deterministic_abstraction::{
    ContinuousFrame, DeterministicAbstraction, SymbolicAbstraction, SymbolicObservation,
};
use super::custom_mechanism::{
    CustomChallengeSpec, CustomEdgeKind, CustomEdgeSpec, CustomNodeKind, CustomNodeSpec,
    CustomTransitionSpec, CustomWorldSpec, HiddenGuardSpec,
};

const MAPPING_BUDGET: u64 = 16_384;
const MAX_OBJECTS: usize = 32;
const MAX_RELATIONS: usize = 96;
const MAX_SOURCE_TRACES: usize = 32;
const MAX_PROBES: usize = 8;

/// Visible structural kind.  The token itself is intentionally opaque: only
/// the role tag and the observed relation pattern may be used for transfer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpaqueObjectKind {
    Operation,
    StateField,
    Resource,
    Constraint,
    Service,
    Transaction,
}

impl OpaqueObjectKind {
    fn core_kind(self) -> NodeKind {
        match self {
            Self::Operation => NodeKind::Operation,
            Self::StateField => NodeKind::StateField,
            Self::Resource => NodeKind::Resource,
            Self::Constraint => NodeKind::Constraint,
            Self::Service => NodeKind::Service,
            Self::Transaction => NodeKind::Transaction,
        }
    }
}

/// Visible relation kind.  Relation names are structural evidence, not a
/// hidden mechanism rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpaqueRelationKind {
    Calls,
    Reads,
    Writes,
    Requires,
    Emits,
    DependsOn,
    InterventionProduced,
}

impl OpaqueRelationKind {
    fn core_kind(self) -> EdgeKind {
        match self {
            Self::Calls => EdgeKind::Calls,
            Self::Reads => EdgeKind::Reads,
            Self::Writes => EdgeKind::Writes,
            Self::Requires => EdgeKind::Requires,
            Self::Emits => EdgeKind::Emits,
            Self::DependsOn => EdgeKind::DependsOn,
            Self::InterventionProduced => EdgeKind::InterventionProduced,
        }
    }
}

/// A visible object in an observation frame.  `value` is present only for
/// state fields.  No hidden guard or world-level rule is represented here.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpaqueObject {
    pub token: String,
    pub kind: OpaqueObjectKind,
    pub role_tag: u64,
    #[serde(default)]
    pub value: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpaqueRelation {
    pub from: String,
    pub to: String,
    pub kind: OpaqueRelationKind,
}

/// The only world state visible to the solver.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpaqueObservation {
    pub objects: Vec<OpaqueObject>,
    pub relations: Vec<OpaqueRelation>,
}

/// Action token plus the visible operation object it targets.  The token is
/// deliberately free to change between source, target and holdout worlds.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpaqueAction {
    pub token: String,
    pub operation: String,
}

/// A complete interaction trace supplied to the solver.  This is the strict
/// replacement for passing `CustomWorldSpec` into the learning core.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservationLog {
    pub episode: u64,
    pub before: OpaqueObservation,
    pub action: OpaqueAction,
    pub after: OpaqueObservation,
}

/// A host-authorized intervention selected by the solver.  It changes only
/// visible state fields and never exposes the environment's guard.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OpaqueIntervention {
    pub state: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OpaquePortError {
    InvalidAction,
    UnknownToken(String),
    InvalidIntervention(String),
}

impl std::fmt::Display for OpaquePortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAction => write!(f, "invalid action token"),
            Self::UnknownToken(token) => write!(f, "unknown token `{token}`"),
            Self::InvalidIntervention(error) => write!(f, "invalid intervention: {error}"),
        }
    }
}

impl std::error::Error for OpaquePortError {}

/// Environment-facing contract.  The solver is never passed an implementing
/// object; the runner uses the port to collect traces and passes only those
/// traces to `OpaqueActionSolver`.
pub trait OpaqueActionPort {
    fn reset(&mut self) -> OpaqueObservation;
    fn send_action(&mut self, action: &OpaqueAction) -> Result<OpaqueObservation, OpaquePortError>;
    fn apply_intervention(
        &mut self,
        intervention: &OpaqueIntervention,
    ) -> Result<OpaqueObservation, OpaquePortError>;
}

#[derive(Clone, Debug)]
struct ObservationAbstraction {
    graph: CanonicalCausalGraph,
    labels: BTreeMap<String, NodeId>,
    state: State,
    action: ActionId,
}

fn stable_hash(input: &str, salt: u64) -> u64 {
    let mut state = 14_695_981_039_346_656_037u64 ^ salt;
    for byte in input.as_bytes() {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(1_099_511_628_211);
    }
    state.max(1)
}

fn action_id(token: &str) -> ActionId {
    stable_hash(token, 0x4143_5449_4f4e)
}

fn opaque_kind(kind: &CustomNodeKind) -> OpaqueObjectKind {
    match kind {
        CustomNodeKind::Service => OpaqueObjectKind::Service,
        CustomNodeKind::Operation => OpaqueObjectKind::Operation,
        CustomNodeKind::StateField => OpaqueObjectKind::StateField,
        CustomNodeKind::Resource => OpaqueObjectKind::Resource,
        CustomNodeKind::Constraint => OpaqueObjectKind::Constraint,
        CustomNodeKind::Transaction => OpaqueObjectKind::Transaction,
    }
}

fn opaque_relation(kind: &CustomEdgeKind) -> OpaqueRelationKind {
    match kind {
        CustomEdgeKind::Calls => OpaqueRelationKind::Calls,
        CustomEdgeKind::Reads => OpaqueRelationKind::Reads,
        CustomEdgeKind::Writes => OpaqueRelationKind::Writes,
        CustomEdgeKind::Requires => OpaqueRelationKind::Requires,
        CustomEdgeKind::Emits => OpaqueRelationKind::Emits,
        CustomEdgeKind::DependsOn => OpaqueRelationKind::DependsOn,
        CustomEdgeKind::InterventionProduced => OpaqueRelationKind::InterventionProduced,
    }
}

fn abstract_observation(
    observation: &OpaqueObservation,
    action: &OpaqueAction,
) -> Result<ObservationAbstraction, String> {
    if observation.objects.is_empty() || observation.objects.len() > MAX_OBJECTS {
        return Err(format!(
            "observation must contain 1..={MAX_OBJECTS} objects"
        ));
    }
    if observation.relations.len() > MAX_RELATIONS {
        return Err(format!(
            "observation must contain at most {MAX_RELATIONS} relations"
        ));
    }

    let mut objects = observation.objects.clone();
    objects.sort_by(|left, right| left.token.cmp(&right.token));
    let mut labels = BTreeMap::new();
    let mut nodes = Vec::with_capacity(objects.len());
    for (index, object) in objects.iter().enumerate() {
        if object.token.trim().is_empty() || labels.contains_key(&object.token) {
            return Err(format!(
                "duplicate or empty opaque token `{}`",
                object.token
            ));
        }
        let id = u32::try_from(index + 1).map_err(|_| "opaque node id overflow".to_owned())?;
        labels.insert(object.token.clone(), id);
        if object.kind == OpaqueObjectKind::StateField && object.value.is_none() {
            return Err(format!(
                "state field `{}` has no observed value",
                object.token
            ));
        }
        if object.kind != OpaqueObjectKind::StateField && object.value.is_some() {
            return Err(format!(
                "non-state object `{}` has a state value",
                object.token
            ));
        }
        nodes.push(CausalNode {
            id,
            kind: object.kind.core_kind(),
            type_tag: object.role_tag,
        });
    }
    labels
        .get(&action.operation)
        .copied()
        .ok_or_else(|| format!("action references unknown operation `{}`", action.operation))?;
    if objects
        .iter()
        .find(|object| object.token == action.operation)
        .map(|object| object.kind)
        != Some(OpaqueObjectKind::Operation)
    {
        return Err("action operation token is not an operation object".to_owned());
    }

    let mut edges = Vec::with_capacity(observation.relations.len());
    for relation in &observation.relations {
        let from = labels
            .get(&relation.from)
            .copied()
            .ok_or_else(|| format!("relation references unknown source `{}`", relation.from))?;
        let to = labels
            .get(&relation.to)
            .copied()
            .ok_or_else(|| format!("relation references unknown target `{}`", relation.to))?;
        edges.push(CausalEdge {
            from,
            to,
            kind: relation.kind.core_kind(),
        });
    }
    let graph = CanonicalCausalGraph::new(nodes, edges)
        .map_err(|error| format!("invalid observed structure: {error:?}"))?;
    let mut state_literals = Vec::new();
    for object in &objects {
        if object.kind == OpaqueObjectKind::StateField {
            let value = object
                .value
                .ok_or_else(|| format!("state field `{}` has no observed value", object.token))?;
            state_literals.push(Literal::new(u64::from(labels[&object.token]), value));
        }
    }
    let state = State::new(state_literals);
    Ok(ObservationAbstraction {
        graph,
        labels,
        state,
        action: action_id(&action.token),
    })
}

fn transition_from_log(
    log: &ObservationLog,
) -> Result<(ObservationAbstraction, TransitionExample), String> {
    let before = abstract_observation(&log.before, &log.action)?;
    let after = abstract_observation(&log.after, &log.action)?;
    if before.graph != after.graph {
        return Err("observation structure changed within one transition".to_owned());
    }
    if before.action != after.action {
        return Err("action token changed within one transition".to_owned());
    }
    Ok((
        before.clone(),
        TransitionExample::new(
            log.episode,
            before.action,
            before.state.clone(),
            after.state,
        ),
    ))
}

/// Convert a bounded continuous frame before it enters the causal learner.
///
/// The schema and abstraction digests returned inside the observation must be
/// carried into the prediction scope used by causal proof. This keeps the
/// quantization policy and symbolic state in one auditable commitment.
pub fn symbolize_continuous_frame(
    abstraction: &mut DeterministicAbstraction,
    frame: &ContinuousFrame,
) -> Result<SymbolicObservation, String> {
    abstraction
        .abstract_frame(frame)
        .map_err(|error| format!("continuous abstraction rejected: {error:?}"))
}

/// The solver-side object.  Its methods accept only opaque observations and
/// interaction logs; no `CustomWorldSpec`, guard or environment type appears
/// in this API.
#[derive(Clone, Debug)]
pub struct OpaqueActionSolver {
    core: CrossDomainCausalCore,
    source: Option<ObservationAbstraction>,
    target: Option<ObservationAbstraction>,
    source_model_digest: Option<Digest>,
}

impl Default for OpaqueActionSolver {
    fn default() -> Self {
        Self::new()
    }
}

impl OpaqueActionSolver {
    pub fn new() -> Self {
        Self {
            core: CrossDomainCausalCore::new(),
            source: None,
            target: None,
            source_model_digest: None,
        }
    }

    pub fn learn_source(&mut self, logs: &[ObservationLog]) -> Result<(), String> {
        if logs.is_empty() || logs.len() > MAX_SOURCE_TRACES {
            return Err(format!(
                "source trace count must be 1..={MAX_SOURCE_TRACES}"
            ));
        }
        let mut abstractions = Vec::with_capacity(logs.len());
        let mut examples = Vec::with_capacity(logs.len());
        for log in logs {
            let (abstraction, example) = transition_from_log(log)?;
            abstractions.push(abstraction);
            examples.push(example);
        }
        let first = abstractions
            .first()
            .cloned()
            .ok_or_else(|| "empty source trace set".to_owned())?;
        if abstractions
            .iter()
            .any(|item| item.graph != first.graph || item.action != first.action)
        {
            return Err("source traces do not share one structural action schema".to_owned());
        }
        self.core
            .learn_source(&examples)
            .map_err(|error| error.to_owned())?;
        self.source_model_digest = Some(self.core.source().model_digest());
        self.source = Some(first);
        Ok(())
    }

    pub fn install_target(
        &mut self,
        before: &OpaqueObservation,
        action: &OpaqueAction,
    ) -> Result<TransferStatus, String> {
        let target = abstract_observation(before, action)?;
        let source = self
            .source
            .as_ref()
            .ok_or_else(|| "source model has not been learned".to_owned())?;
        let budget = MappingBudget::new(MAPPING_BUDGET).expect("positive mapping budget");
        let status = self
            .core
            .transfer(&source.graph, &target.graph, target.action, budget);
        self.target = Some(target);
        Ok(status)
    }

    pub fn observe_target(&mut self, log: &ObservationLog) -> Result<ObservationVerdict, String> {
        let (abstraction, example) = transition_from_log(log)?;
        let target = self
            .target
            .as_ref()
            .ok_or_else(|| "target structure has not been installed".to_owned())?;
        if abstraction.graph != target.graph || abstraction.action != target.action {
            return Err("target observation does not match installed structure".to_owned());
        }
        Ok(self.core.target_trial(example))
    }

    pub fn choose_probe(
        &self,
        base: &OpaqueObservation,
        action: &OpaqueAction,
    ) -> Result<Option<OpaqueIntervention>, String> {
        let abstraction = abstract_observation(base, action)?;
        let target = self
            .target
            .as_ref()
            .ok_or_else(|| "target structure has not been installed".to_owned())?;
        if abstraction.graph != target.graph || abstraction.action != target.action {
            return Err("probe base does not match target structure".to_owned());
        }
        let Some(probe) = self.core.choose_target_probe(&abstraction.state) else {
            return Ok(None);
        };
        let mut state = BTreeMap::new();
        for (token, node) in &target.labels {
            if let Some(value) = probe.before.get(u64::from(*node)) {
                state.insert(token.clone(), value);
            }
        }
        Ok(Some(OpaqueIntervention { state }))
    }

    pub fn apply_probe(&mut self, log: &ObservationLog) -> Result<ObservationVerdict, String> {
        let (abstraction, example) = transition_from_log(log)?;
        let target = self
            .target
            .as_ref()
            .ok_or_else(|| "target structure has not been installed".to_owned())?;
        if abstraction.graph != target.graph || abstraction.action != target.action {
            return Err("probe observation does not match target structure".to_owned());
        }
        Ok(self.core.apply_target_probe(example))
    }

    /// Freeze the installed target learner before an external action is
    /// authorized. The returned view is detached from later online revision.
    pub fn freeze_target_model(
        &self,
        max_candidates: usize,
        max_outcomes: usize,
    ) -> Result<FrozenTransferModel, String> {
        if !matches!(self.core.status(), TransferStatus::Supported | TransferStatus::Revised) {
            return Err("target proof freeze requires supported or revised transfer".to_owned());
        }
        let target = self
            .core
            .target()
            .ok_or_else(|| "target model has not been installed".to_owned())?;
        FrozenTransferModel::from_learner(target, max_candidates, max_outcomes)
            .map_err(|error| format!("target proof freeze rejected: {error:?}"))
    }

    pub fn verify_holdout(
        &self,
        positive: &ObservationLog,
        negative: &ObservationLog,
    ) -> Result<(bool, bool), String> {
        let target = self
            .target
            .as_ref()
            .ok_or_else(|| "target model has not been installed".to_owned())?;
        let (positive_before, positive_example) = transition_from_log(positive)?;
        let (negative_before, negative_example) = transition_from_log(negative)?;
        if positive_before.graph != negative_before.graph
            || positive_before.action != negative_before.action
        {
            return Err("holdout traces do not share one structure".to_owned());
        }
        let budget = MappingBudget::new(MAPPING_BUDGET).expect("positive mapping budget");
        let mapping = match target
            .graph
            .find_unique_mapping(&positive_before.graph, budget)
        {
            MappingVerdict::Unique(mapping) => mapping,
            MappingVerdict::Ambiguous { .. } => return Ok((false, false)),
            MappingVerdict::NoMatch | MappingVerdict::ResourceLimit { .. } => {
                return Ok((false, false));
            }
        };
        let rule = self
            .core
            .target()
            .and_then(CausalLearner::rule)
            .ok_or_else(|| "target rule is unavailable".to_owned())?;
        let holdout_rule = rule
            .remap(&mapping, positive_before.action)
            .ok_or_else(|| "target rule could not be remapped to holdout".to_owned())?;
        let positive_ok = holdout_rule.predicts(&positive_example.before) == positive_example.after;
        let negative_ok = holdout_rule.predicts(&negative_example.before) == negative_example.after;
        Ok((positive_ok, negative_ok))
    }

    pub fn ledger_verified(&self) -> bool {
        self.core.ledger().verify()
    }

    pub fn source_model_unchanged(&self) -> bool {
        self.source_model_digest == Some(self.core.source().model_digest())
    }

    pub fn ledger_event_count(&self) -> usize {
        self.core.ledger().events().len()
    }
}

#[derive(Clone, Debug)]
struct OpaqueEnvironment {
    observation_template: OpaqueObservation,
    action: OpaqueAction,
    effect: String,
    guard: BTreeSet<String>,
    state: BTreeMap<String, bool>,
}

impl OpaqueEnvironment {
    fn from_world(world: &CustomWorldSpec, guard: &[String]) -> Result<Self, String> {
        if world.action.trim().is_empty() || world.effect.trim().is_empty() {
            return Err("world action and effect are required".to_owned());
        }
        if world.nodes.is_empty() || world.nodes.len() > MAX_OBJECTS {
            return Err(format!("world must contain 1..={MAX_OBJECTS} nodes"));
        }
        if world.edges.len() > MAX_RELATIONS {
            return Err(format!(
                "world must contain at most {MAX_RELATIONS} relations"
            ));
        }
        let mut seen = BTreeSet::new();
        let mut operation = false;
        let mut state = BTreeMap::new();
        let mut objects = Vec::with_capacity(world.nodes.len());
        for node in &world.nodes {
            if node.id.trim().is_empty() || !seen.insert(node.id.clone()) {
                return Err(format!(
                    "duplicate or empty environment token `{}`",
                    node.id
                ));
            }
            let kind = opaque_kind(&node.kind);
            operation |= node.id == world.action && kind == OpaqueObjectKind::Operation;
            if kind == OpaqueObjectKind::StateField {
                state.insert(node.id.clone(), false);
            }
            objects.push(OpaqueObject {
                token: node.id.clone(),
                kind,
                role_tag: node.type_tag,
                value: (kind == OpaqueObjectKind::StateField).then_some(false),
            });
        }
        if !operation {
            return Err("world action must name an operation node".to_owned());
        }
        if !state.contains_key(&world.effect) {
            return Err("world effect must name a state field".to_owned());
        }
        let mut relations = Vec::with_capacity(world.edges.len());
        let node_ids: BTreeSet<&str> = seen.iter().map(String::as_str).collect();
        for edge in &world.edges {
            if !node_ids.contains(edge.from.as_str()) || !node_ids.contains(edge.to.as_str()) {
                return Err("environment relation references unknown token".to_owned());
            }
            relations.push(OpaqueRelation {
                from: edge.from.clone(),
                to: edge.to.clone(),
                kind: opaque_relation(&edge.kind),
            });
        }
        let guard: BTreeSet<String> = guard.iter().cloned().collect();
        if guard.is_empty()
            || guard.len() > 2
            || guard.iter().any(|label| !state.contains_key(label))
        {
            return Err("environment guard must contain one or two state fields".to_owned());
        }
        if guard.contains(&world.effect) {
            return Err("environment guard cannot contain the effect field".to_owned());
        }
        Ok(Self {
            observation_template: OpaqueObservation { objects, relations },
            action: OpaqueAction {
                token: world.action.clone(),
                operation: world.action.clone(),
            },
            effect: world.effect.clone(),
            guard,
            state,
        })
    }

    fn set_state(&mut self, values: &BTreeMap<String, bool>) -> Result<(), String> {
        if values.len() != self.state.len()
            || values.keys().any(|key| !self.state.contains_key(key))
        {
            return Err(
                "intervention/state assignment does not cover exactly the visible state fields"
                    .to_owned(),
            );
        }
        self.state = values.clone();
        Ok(())
    }

    fn observation(&self) -> OpaqueObservation {
        let mut observation = self.observation_template.clone();
        for object in &mut observation.objects {
            if let Some(value) = self.state.get(&object.token) {
                object.value = Some(*value);
            }
        }
        observation
    }
}

impl OpaqueActionPort for OpaqueEnvironment {
    fn reset(&mut self) -> OpaqueObservation {
        for value in self.state.values_mut() {
            *value = false;
        }
        self.observation()
    }

    fn send_action(&mut self, action: &OpaqueAction) -> Result<OpaqueObservation, OpaquePortError> {
        if action != &self.action {
            return Err(OpaquePortError::UnknownToken(action.token.clone()));
        }
        let enabled = self
            .guard
            .iter()
            .all(|role| self.state.get(role).copied() == Some(true));
        if enabled {
            self.state.insert(self.effect.clone(), true);
        }
        Ok(self.observation())
    }

    fn apply_intervention(
        &mut self,
        intervention: &OpaqueIntervention,
    ) -> Result<OpaqueObservation, OpaquePortError> {
        self.set_state(&intervention.state)
            .map_err(OpaquePortError::InvalidIntervention)?;
        self.state.insert(self.effect.clone(), false);
        Ok(self.observation())
    }
}

/// Interactive phase for the browser-facing opaque session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpaqueSessionPhase {
    ReadyForAction,
    TargetConfirmed,
    Contradicted,
    Probing,
    Revised,
    HoldoutVerified,
    Abstain,
}

impl OpaqueSessionPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::ReadyForAction => "READY_FOR_ACTION",
            Self::TargetConfirmed => "TARGET_CONFIRMED",
            Self::Contradicted => "CONTRADICTED",
            Self::Probing => "PROBING",
            Self::Revised => "REVISED",
            Self::HoldoutVerified => "HOLDOUT_VERIFIED",
            Self::Abstain => "ABSTAIN",
        }
    }
}

/// Stateful, host-owned interaction session for the public demonstration.
///
/// The session owns its private fixture environment, while all learner-facing
/// calls still cross the `OpaqueActionPort` boundary. The browser can submit
/// an opaque action token, trigger a host-authorized sabotage, request one
/// discriminating probe, and finally ask for holdout verification. It never
/// receives the graph specification or the hidden guard.
#[derive(Clone, Debug)]
pub struct OpaqueInteractionSession {
    seed: u64,
    solver: OpaqueActionSolver,
    target_environment: OpaqueEnvironment,
    holdout_environment: OpaqueEnvironment,
    target_action: OpaqueAction,
    holdout_action: OpaqueAction,
    holdout_world: CustomWorldSpec,
    holdout_guard: Vec<String>,
    target_before: OpaqueObservation,
    sabotage_before: Option<OpaqueObservation>,
    probe_base: Option<OpaqueObservation>,
    phase: OpaqueSessionPhase,
    source_traces: usize,
    probe_count: usize,
    holdout_positive: bool,
    holdout_negative: bool,
    lines: Vec<String>,
}

impl OpaqueInteractionSession {
    pub fn new(seed: u64) -> Result<Self, String> {
        let fixture = generated_opaque_fixture(seed)?;
        let source_guard = source_guard_from_target(
            &fixture.spec.source,
            &fixture.spec.target,
            &fixture.hidden_guard.target,
        )?;
        let mut source_environment =
            OpaqueEnvironment::from_world(&fixture.spec.source, &source_guard)?;
        let mut target_environment =
            OpaqueEnvironment::from_world(&fixture.spec.target, &fixture.hidden_guard.target)?;
        let holdout_environment =
            OpaqueEnvironment::from_world(&fixture.spec.holdout, &fixture.hidden_guard.holdout)?;
        let source_logs = source_logs(&mut source_environment, &fixture.spec.source_examples)?;
        let mut solver = OpaqueActionSolver::new();
        solver.learn_source(&source_logs)?;

        let target_before_state = session_state(&fixture.spec.target, &fixture.hidden_guard.target);
        let target_before = target_environment
            .apply_intervention(&OpaqueIntervention {
                state: target_before_state,
            })
            .map_err(|error| error.to_string())?;
        let target_action = target_environment.action.clone();
        if solver.install_target(&target_before, &target_action)?
            != TransferStatus::StructuralCandidate
        {
            return Err(
                "generated opaque session could not install a unique target mapping".to_owned(),
            );
        }

        let holdout_action = holdout_environment.action.clone();
        let mut session = Self {
            seed,
            solver,
            target_environment,
            holdout_environment,
            target_action,
            holdout_action,
            holdout_world: fixture.spec.holdout.clone(),
            holdout_guard: fixture.hidden_guard.holdout,
            target_before,
            sabotage_before: None,
            probe_base: None,
            phase: OpaqueSessionPhase::ReadyForAction,
            source_traces: source_logs.len(),
            probe_count: 0,
            holdout_positive: false,
            holdout_negative: false,
            lines: Vec::new(),
        };
        session.push_session_header();
        session.push_stage(
            "OBSERVE",
            format!(
                "{} source traces accepted through the opaque action port",
                session.source_traces
            ),
        );
        session.push_stage(
            "ABSTRACT",
            "structure reconstructed from visible observation frames".to_owned(),
        );
        session.push_stage(
            "TRANSFER",
            "unique target mapping installed; hidden law remains environment-only".to_owned(),
        );
        session.push_observation("target_before_action", &session.target_before.clone());
        Ok(session)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn phase(&self) -> OpaqueSessionPhase {
        self.phase
    }

    pub fn action_token(&self) -> &str {
        &self.target_action.token
    }

    pub fn observation_jsonl(&self) -> String {
        let mut output = self.lines.join("\n");
        if !output.is_empty() {
            output.push('\n');
        }
        let digest = stable_hash(
            &format!(
                "{}:{}:{}:{}:{}:{}",
                self.seed,
                self.phase.label(),
                self.probe_count,
                self.holdout_positive,
                self.holdout_negative,
                self.solver.ledger_event_count()
            ),
            0x53455353,
        );
        let _ = writeln!(
            output,
            "{{\"record\":\"summary\",\"status\":{},\"source_traces\":{},\"probes\":{},\"holdout_positive\":{},\"holdout_negative\":{},\"ledger_verified\":{},\"source_model_unchanged\":{},\"first_attempt_transfer\":true,\"report_digest\":{}}}",
            json_string(self.phase.label()),
            self.source_traces,
            self.probe_count,
            self.holdout_positive,
            self.holdout_negative,
            self.solver.ledger_verified(),
            self.solver.source_model_unchanged(),
            digest,
        );
        output
    }

    /// Submit the opaque action token shown by `target_before_action`.
    pub fn send_action(&mut self, token: &str) -> Result<String, String> {
        if self.phase != OpaqueSessionPhase::ReadyForAction {
            return Err(format!(
                "target action is not available in phase {}",
                self.phase.label()
            ));
        }
        if token != self.target_action.token {
            return Err("unknown opaque action token".to_owned());
        }
        let before = self.target_before.clone();
        let after = self
            .target_environment
            .send_action(&self.target_action)
            .map_err(|error| error.to_string())?;
        let log = ObservationLog {
            episode: 30_001,
            before: before.clone(),
            action: self.target_action.clone(),
            after: after.clone(),
        };
        match self.solver.observe_target(&log)? {
            ObservationVerdict::Confirmed => {
                self.phase = OpaqueSessionPhase::TargetConfirmed;
                let action = self.target_action.clone();
                self.push_action("target_action", &action);
                self.push_observation("target_after_action", &after);
                self.push_stage(
                    "TARGET_TRIAL",
                    "first target action confirmed the transferred effect".to_owned(),
                );
                Ok(self.observation_jsonl())
            }
            other => Err(format!("unexpected target trial verdict: {other:?}")),
        }
    }

    /// Apply a private environment sabotage and expose only its observation.
    pub fn sabotage(&mut self) -> Result<String, String> {
        if self.phase != OpaqueSessionPhase::TargetConfirmed {
            return Err("sabotage requires a confirmed target action".to_owned());
        }
        let sabotage_state = self
            .target_environment
            .state
            .keys()
            .map(|label| (label.clone(), false))
            .collect();
        let before = self
            .target_environment
            .apply_intervention(&OpaqueIntervention {
                state: sabotage_state,
            })
            .map_err(|error| error.to_string())?;
        let after = self
            .target_environment
            .send_action(&self.target_action)
            .map_err(|error| error.to_string())?;
        let log = ObservationLog {
            episode: 30_002,
            before: before.clone(),
            action: self.target_action.clone(),
            after: after.clone(),
        };
        if !matches!(
            self.solver.observe_target(&log)?,
            ObservationVerdict::ContradictionFound { .. }
        ) {
            return Err("sabotage did not produce a contradiction".to_owned());
        }
        self.sabotage_before = Some(before.clone());
        self.probe_base = Some(before.clone());
        self.phase = OpaqueSessionPhase::Contradicted;
        let action = self.target_action.clone();
        self.push_action("sabotaged_action", &action);
        self.push_observation("sabotage_before", &before);
        self.push_observation("sabotage_after", &after);
        self.push_stage(
            "FALSIFY",
            "CONTRADICTION_FOUND; previous target permit quarantined".to_owned(),
        );
        Ok(self.observation_jsonl())
    }

    /// Execute exactly one selected discriminating probe.
    pub fn probe(&mut self) -> Result<String, String> {
        if !matches!(
            self.phase,
            OpaqueSessionPhase::Contradicted | OpaqueSessionPhase::Probing
        ) {
            return Err("a probe requires a contradiction".to_owned());
        }
        let base = self
            .probe_base
            .clone()
            .ok_or_else(|| "probe base is missing".to_owned())?;
        let Some(intervention) = self.solver.choose_probe(&base, &self.target_action)? else {
            self.phase = OpaqueSessionPhase::Abstain;
            self.push_stage(
                "ABSTAIN",
                "no bounded experiment can distinguish the surviving hypotheses".to_owned(),
            );
            return Ok(self.observation_jsonl());
        };
        self.probe_count += 1;
        let before = self
            .target_environment
            .apply_intervention(&intervention)
            .map_err(|error| error.to_string())?;
        let after = self
            .target_environment
            .send_action(&self.target_action)
            .map_err(|error| error.to_string())?;
        let log = ObservationLog {
            episode: 30_002 + self.probe_count as u64,
            before: before.clone(),
            action: self.target_action.clone(),
            after: after.clone(),
        };
        match self.solver.apply_probe(&log)? {
            ObservationVerdict::Revised { epoch } => {
                self.phase = OpaqueSessionPhase::Revised;
                let action = self.target_action.clone();
                self.push_action("discriminating_probe", &action);
                self.push_observation("probe_before", &before);
                self.push_observation("probe_after", &after);
                self.push_stage(
                    "REVISE",
                    format!("local revision committed at epoch={epoch}; prior witnesses retained"),
                );
            }
            ObservationVerdict::Abstain => {
                self.phase = OpaqueSessionPhase::Probing;
                self.probe_base = Some(before.clone());
                let action = self.target_action.clone();
                self.push_action("discriminating_probe", &action);
                self.push_observation("probe_before", &before);
                self.push_observation("probe_after", &after);
                self.push_stage(
                    "ACTIVE_PROBE",
                    format!(
                        "probe #{} retained competing hypotheses; another probe is allowed",
                        self.probe_count
                    ),
                );
            }
            other => return Err(format!("unexpected probe verdict: {other:?}")),
        }
        Ok(self.observation_jsonl())
    }

    /// Evaluate independent positive and negative holdout traces.
    pub fn verify_holdout(&mut self) -> Result<String, String> {
        if self.phase != OpaqueSessionPhase::Revised {
            return Err("holdout requires a committed local revision".to_owned());
        }
        let positive_state = session_state(&self.holdout_world, &self.holdout_guard);
        let positive_before = self
            .holdout_environment
            .apply_intervention(&OpaqueIntervention {
                state: positive_state,
            })
            .map_err(|error| error.to_string())?;
        let positive_after = self
            .holdout_environment
            .send_action(&self.holdout_action)
            .map_err(|error| error.to_string())?;
        let negative_state = session_state(&self.holdout_world, &[]);
        let negative_before = self
            .holdout_environment
            .apply_intervention(&OpaqueIntervention {
                state: negative_state,
            })
            .map_err(|error| error.to_string())?;
        let negative_after = self
            .holdout_environment
            .send_action(&self.holdout_action)
            .map_err(|error| error.to_string())?;
        let positive = ObservationLog {
            episode: 40_001,
            before: positive_before,
            action: self.holdout_action.clone(),
            after: positive_after,
        };
        let negative = ObservationLog {
            episode: 40_002,
            before: negative_before,
            action: self.holdout_action.clone(),
            after: negative_after,
        };
        let (positive_ok, negative_ok) = self.solver.verify_holdout(&positive, &negative)?;
        self.holdout_positive = positive_ok;
        self.holdout_negative = negative_ok;
        self.phase = if positive_ok && negative_ok {
            OpaqueSessionPhase::HoldoutVerified
        } else {
            OpaqueSessionPhase::Abstain
        };
        self.push_stage(
            "HOLDOUT",
            format!("positive={positive_ok}; negative={negative_ok}; no target retraining"),
        );
        Ok(self.observation_jsonl())
    }

    fn push_session_header(&mut self) {
        let action = json_string(&self.target_action.token);
        let _ = self.lines.push(format!(
            "{{\"record\":\"opaque_session\",\"seed\":{},\"phase\":{},\"solver_input\":\"observation_logs_only\",\"action_token\":{}}}",
            self.seed,
            json_string(self.phase.label()),
            action,
        ));
    }

    fn push_stage(&mut self, stage: &str, detail: String) {
        self.lines.push(format!(
            "{{\"record\":\"stage\",\"stage\":{},\"detail\":{}}}",
            json_string(stage),
            json_string(&detail),
        ));
    }

    fn push_action(&mut self, kind: &str, action: &OpaqueAction) {
        let encoded = serde_json::to_string(action).unwrap_or_else(|_| "null".to_owned());
        self.lines.push(format!(
            "{{\"record\":\"action\",\"kind\":{},\"value\":{}}}",
            json_string(kind),
            encoded,
        ));
    }

    fn push_observation(&mut self, kind: &str, observation: &OpaqueObservation) {
        let encoded = serde_json::to_string(observation).unwrap_or_else(|_| "null".to_owned());
        self.lines.push(format!(
            "{{\"record\":\"observation\",\"kind\":{},\"value\":{}}}",
            json_string(kind),
            encoded,
        ));
    }
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"<invalid>\"".to_owned())
}

struct GeneratedOpaqueFixture {
    spec: CustomChallengeSpec,
    hidden_guard: HiddenGuardSpec,
}

fn session_token(seed: u64, domain: &str, role: &str) -> String {
    format!(
        "{}_{}_{}",
        domain,
        role,
        stable_hash(&format!("{seed}:{domain}:{role}"), 0x544f4b45) as u32
    )
}

fn session_world(seed: u64, domain: &str) -> (CustomWorldSpec, Vec<String>) {
    let action = session_token(seed, domain, "act");
    let guard_a = session_token(seed, domain, "guard_a");
    let guard_b = session_token(seed, domain, "guard_b");
    let effect = session_token(seed, domain, "effect");
    let nodes = vec![
        CustomNodeSpec {
            id: action.clone(),
            kind: CustomNodeKind::Operation,
            type_tag: 10,
        },
        CustomNodeSpec {
            id: guard_a.clone(),
            kind: CustomNodeKind::StateField,
            type_tag: 11,
        },
        CustomNodeSpec {
            id: guard_b.clone(),
            kind: CustomNodeKind::StateField,
            type_tag: 12,
        },
        CustomNodeSpec {
            id: effect.clone(),
            kind: CustomNodeKind::StateField,
            type_tag: 13,
        },
    ];
    let edges = vec![
        CustomEdgeSpec {
            from: action.clone(),
            to: guard_a.clone(),
            kind: CustomEdgeKind::Requires,
        },
        CustomEdgeSpec {
            from: action.clone(),
            to: guard_b.clone(),
            kind: CustomEdgeKind::Requires,
        },
        CustomEdgeSpec {
            from: action.clone(),
            to: effect.clone(),
            kind: CustomEdgeKind::Writes,
        },
    ];
    (
        CustomWorldSpec {
            action,
            nodes,
            edges,
            effect,
        },
        vec![guard_a, guard_b],
    )
}

fn session_state(world: &CustomWorldSpec, guard: &[String]) -> BTreeMap<String, bool> {
    world
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, CustomNodeKind::StateField))
        .map(|node| {
            (
                node.id.clone(),
                node.id != world.effect && guard.iter().any(|label| label == &node.id),
            )
        })
        .collect()
}

fn generated_opaque_fixture(seed: u64) -> Result<GeneratedOpaqueFixture, String> {
    let (source, source_roles) = session_world(seed, "source");
    let (target, target_roles) = session_world(seed, "target");
    let (holdout, holdout_roles) = session_world(seed, "holdout");
    let guard_count = if seed & 1 == 0 { 1 } else { 2 };
    let target_guard = target_roles
        .into_iter()
        .take(guard_count)
        .collect::<Vec<_>>();
    let holdout_guard = holdout_roles
        .into_iter()
        .take(guard_count)
        .collect::<Vec<_>>();
    let source_guard = source_roles
        .into_iter()
        .take(guard_count)
        .collect::<Vec<_>>();
    let source_before = session_state(&source, &source_guard);
    let mut source_after = source_before.clone();
    source_after.insert(source.effect.clone(), true);
    let source_examples = (0..3)
        .map(|_| CustomTransitionSpec {
            before: source_before.clone(),
            after: source_after.clone(),
        })
        .collect();
    Ok(GeneratedOpaqueFixture {
        spec: CustomChallengeSpec {
            name: format!("opaque interactive mechanism {seed}"),
            seed,
            source,
            source_examples,
            target,
            holdout,
            hidden_guard: Some(HiddenGuardSpec {
                target: target_guard.clone(),
                holdout: holdout_guard.clone(),
            }),
        },
        hidden_guard: HiddenGuardSpec {
            target: target_guard,
            holdout: holdout_guard,
        },
    })
}

fn labels_with_type_tags(
    source: &CustomWorldSpec,
    source_labels: &[String],
    target: &CustomWorldSpec,
) -> Result<Vec<String>, String> {
    source_labels
        .iter()
        .map(|source_label| {
            let source_node = source
                .nodes
                .iter()
                .find(|node| node.id == *source_label)
                .ok_or_else(|| format!("unknown source guard `{source_label}`"))?;
            target
                .nodes
                .iter()
                .find(|node| {
                    matches!(node.kind, CustomNodeKind::StateField)
                        && node.type_tag == source_node.type_tag
                })
                .map(|node| node.id.clone())
                .ok_or_else(|| format!("no target role for source guard `{source_label}`"))
        })
        .collect()
}

fn derive_guard(
    target: &CustomWorldSpec,
    holdout: &CustomWorldSpec,
    seed: u64,
) -> Result<HiddenGuardSpec, String> {
    let candidates: Vec<_> = target
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, CustomNodeKind::StateField) && node.id != target.effect)
        .collect();
    if candidates.is_empty() {
        return Err("cannot derive guard without non-effect state fields".to_owned());
    }
    let first = candidates[(seed as usize) % candidates.len()];
    let mut target_guard = vec![first.id.clone()];
    if candidates.len() > 1 && ((seed >> 3) & 1) == 1 {
        target_guard.push(
            candidates[(seed as usize + 1) % candidates.len()]
                .id
                .clone(),
        );
    }
    let holdout_guard = labels_with_type_tags(target, &target_guard, holdout)?;
    Ok(HiddenGuardSpec {
        target: target_guard,
        holdout: holdout_guard,
    })
}

fn source_guard_from_target(
    source: &CustomWorldSpec,
    target: &CustomWorldSpec,
    target_guard: &[String],
) -> Result<Vec<String>, String> {
    target_guard
        .iter()
        .map(|target_label| {
            let target_node = target
                .nodes
                .iter()
                .find(|node| node.id == *target_label)
                .ok_or_else(|| format!("unknown target guard `{target_label}`"))?;
            source
                .nodes
                .iter()
                .find(|node| {
                    matches!(node.kind, CustomNodeKind::StateField)
                        && node.type_tag == target_node.type_tag
                })
                .map(|node| node.id.clone())
                .ok_or_else(|| format!("no source role for target guard `{target_label}`"))
        })
        .collect()
}

fn source_logs(
    environment: &mut OpaqueEnvironment,
    examples: &[CustomTransitionSpec],
) -> Result<Vec<ObservationLog>, String> {
    if examples.is_empty() || examples.len() > MAX_SOURCE_TRACES {
        return Err(format!(
            "source traces must contain 1..={MAX_SOURCE_TRACES} examples"
        ));
    }
    let mut logs = Vec::with_capacity(examples.len());
    for (index, example) in examples.iter().enumerate() {
        environment
            .set_state(&example.before)
            .map_err(|error| format!("source example {} before-state: {error}", index + 1))?;
        let before = environment.observation();
        let after = environment
            .send_action(&environment.action.clone())
            .map_err(|error| format!("source example {} action: {error}", index + 1))?;
        let expected = example.after.iter().all(|(role, value)| {
            after
                .objects
                .iter()
                .find(|object| object.token == *role)
                .and_then(|object| object.value)
                == Some(*value)
        });
        if !expected {
            return Err(format!(
                "source example {} cannot be reproduced by the opaque environment",
                index + 1
            ));
        }
        logs.push(ObservationLog {
            episode: index as u64 + 1,
            before,
            action: environment.action.clone(),
            after,
        });
    }
    Ok(logs)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpaqueRunStatus {
    HoldoutVerified,
    Abstain,
    Refused,
}

impl OpaqueRunStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::HoldoutVerified => "HOLDOUT_VERIFIED",
            Self::Abstain => "ABSTAIN",
            Self::Refused => "REFUSED",
        }
    }
}

#[derive(Clone, Debug)]
pub struct OpaqueRunReport {
    pub status: OpaqueRunStatus,
    pub reason: String,
    pub source_traces: usize,
    pub probes: usize,
    pub holdout_positive: bool,
    pub holdout_negative: bool,
    pub ledger_verified: bool,
    pub source_model_unchanged: bool,
    pub first_attempt_transfer: bool,
    jsonl: String,
}

impl OpaqueRunReport {
    pub fn succeeded(&self) -> bool {
        self.status == OpaqueRunStatus::HoldoutVerified
            && self.holdout_positive
            && self.holdout_negative
            && self.ledger_verified
            && self.source_model_unchanged
            && self.first_attempt_transfer
    }

    pub fn jsonl(&self) -> &str {
        &self.jsonl
    }
}

fn opaque_report(
    spec: &CustomChallengeSpec,
    status: OpaqueRunStatus,
    reason: &str,
    stages: &[(&str, String)],
    source_traces: usize,
    probes: usize,
    positive: bool,
    negative: bool,
    ledger: bool,
    source_unchanged: bool,
    first_attempt: bool,
) -> OpaqueRunReport {
    let mut output = String::new();
    let digest = stable_hash(
        &format!(
            "{}:{}:{}:{}:{}",
            spec.name,
            spec.seed,
            status.label(),
            positive,
            negative
        ),
        0x4f50_4151,
    );
    let _ = writeln!(
        output,
        "{{\"record\":\"opaque_challenge\",\"name\":{},\"seed\":{},\"solver_input\":\"observation_logs_only\",\"report_digest\":{}}}",
        serde_json::to_string(&spec.name).unwrap_or_else(|_| "\"<invalid>\"".to_owned()),
        spec.seed,
        digest,
    );
    for (stage, detail) in stages {
        let _ = writeln!(
            output,
            "{{\"record\":\"stage\",\"stage\":{},\"detail\":{}}}",
            serde_json::to_string(stage).unwrap_or_else(|_| "\"<invalid>\"".to_owned()),
            serde_json::to_string(detail).unwrap_or_else(|_| "\"<invalid>\"".to_owned()),
        );
    }
    let _ = writeln!(
        output,
        "{{\"record\":\"summary\",\"status\":{},\"reason\":{},\"source_traces\":{},\"probes\":{},\"holdout_positive\":{},\"holdout_negative\":{},\"ledger_verified\":{},\"source_model_unchanged\":{},\"first_attempt_transfer\":{},\"report_digest\":{}}}",
        serde_json::to_string(status.label()).unwrap_or_else(|_| "\"<invalid>\"".to_owned()),
        serde_json::to_string(reason).unwrap_or_else(|_| "\"<invalid>\"".to_owned()),
        source_traces,
        probes,
        positive,
        negative,
        ledger,
        source_unchanged,
        first_attempt,
        digest,
    );
    OpaqueRunReport {
        status,
        reason: reason.to_owned(),
        source_traces,
        probes,
        holdout_positive: positive,
        holdout_negative: negative,
        ledger_verified: ledger,
        source_model_unchanged: source_unchanged,
        first_attempt_transfer: first_attempt,
        jsonl: output,
    }
}

/// Run the full causal cycle through the opaque port.
///
/// The `CustomChallengeSpec` is consumed by the private fixture environment;
/// the solver itself receives only `ObservationLog` values.  This function is
/// therefore suitable for the first no-leak acceptance test while keeping the
/// existing JSON fixture format usable by the browser demo.
pub fn run_opaque_custom(spec: CustomChallengeSpec) -> Result<OpaqueRunReport, String> {
    let hidden_guard = match spec.hidden_guard.clone() {
        Some(guard) => guard,
        None => derive_guard(&spec.target, &spec.holdout, spec.seed)?,
    };
    let source_guard = source_guard_from_target(&spec.source, &spec.target, &hidden_guard.target)?;
    let mut source_environment = OpaqueEnvironment::from_world(&spec.source, &source_guard)?;
    let mut target_environment = OpaqueEnvironment::from_world(&spec.target, &hidden_guard.target)?;
    let mut holdout_environment =
        OpaqueEnvironment::from_world(&spec.holdout, &hidden_guard.holdout)?;
    let mut stages = Vec::new();
    let source_logs = source_logs(&mut source_environment, &spec.source_examples)?;
    let mut solver = OpaqueActionSolver::new();
    solver.learn_source(&source_logs)?;
    stages.push((
        "OBSERVE",
        format!("{} opaque interaction traces accepted", source_logs.len()),
    ));
    stages.push((
        "ABSTRACT",
        "typed structure reconstructed from observation frames".to_owned(),
    ));

    let positive_intervention = OpaqueIntervention {
        state: hidden_guard
            .target
            .iter()
            .map(|label| (label.clone(), true))
            .chain(
                spec.target
                    .nodes
                    .iter()
                    .filter(|node| {
                        matches!(node.kind, CustomNodeKind::StateField)
                            && node.id != spec.target.effect
                            && !hidden_guard.target.contains(&node.id)
                    })
                    .map(|node| (node.id.clone(), false)),
            )
            .chain(std::iter::once((spec.target.effect.clone(), false)))
            .collect(),
    };
    let target_before = target_environment
        .apply_intervention(&positive_intervention)
        .map_err(|error| error.to_string())?;
    let target_action = target_environment.action.clone();
    let transfer = solver.install_target(&target_before, &target_action)?;
    if transfer != TransferStatus::StructuralCandidate {
        stages.push((
            "TRANSFER",
            format!("structural candidate refused: {transfer:?}"),
        ));
        return Ok(opaque_report(
            &spec,
            OpaqueRunStatus::Refused,
            "opaque structural transfer refused",
            &stages,
            source_logs.len(),
            0,
            false,
            false,
            solver.ledger_verified(),
            solver.source_model_unchanged(),
            false,
        ));
    }
    stages.push((
        "TRANSFER",
        "unique mapping inferred from visible observation structure".to_owned(),
    ));
    let target_positive_after = target_environment
        .send_action(&target_action)
        .map_err(|error| error.to_string())?;
    let target_positive = ObservationLog {
        episode: 10_001,
        before: target_before,
        action: target_action.clone(),
        after: target_positive_after,
    };
    if solver.observe_target(&target_positive)? != ObservationVerdict::Confirmed {
        return Ok(opaque_report(
            &spec,
            OpaqueRunStatus::Abstain,
            "target positive evidence was not confirmed",
            &stages,
            source_logs.len(),
            0,
            false,
            false,
            solver.ledger_verified(),
            solver.source_model_unchanged(),
            false,
        ));
    }
    stages.push((
        "TARGET_TRIAL",
        "positive target outcome confirmed".to_owned(),
    ));

    let sabotage_state = spec
        .target
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, CustomNodeKind::StateField))
        .map(|node| (node.id.clone(), false))
        .collect();
    let sabotage_before = target_environment
        .apply_intervention(&OpaqueIntervention {
            state: sabotage_state,
        })
        .map_err(|error| error.to_string())?;
    let sabotage_after = target_environment
        .send_action(&target_action)
        .map_err(|error| error.to_string())?;
    let sabotage = ObservationLog {
        episode: 10_002,
        before: sabotage_before.clone(),
        action: target_action.clone(),
        after: sabotage_after,
    };
    let contradiction = solver.observe_target(&sabotage)?;
    if !matches!(contradiction, ObservationVerdict::ContradictionFound { .. }) {
        return Ok(opaque_report(
            &spec,
            OpaqueRunStatus::Abstain,
            "sabotage did not produce an identifiable contradiction",
            &stages,
            source_logs.len(),
            0,
            false,
            false,
            solver.ledger_verified(),
            solver.source_model_unchanged(),
            false,
        ));
    }
    stages.push((
        "FALSIFY",
        "CONTRADICTION_FOUND; old permit quarantined".to_owned(),
    ));

    let mut probe_count = 0;
    let mut probe_base = sabotage_before;
    let mut revised = false;
    for _ in 0..MAX_PROBES {
        let Some(intervention) = solver.choose_probe(&probe_base, &target_action)? else {
            break;
        };
        probe_count += 1;
        let probe_before = target_environment
            .apply_intervention(&intervention)
            .map_err(|error| error.to_string())?;
        let probe_after = target_environment
            .send_action(&target_action)
            .map_err(|error| error.to_string())?;
        let probe = ObservationLog {
            episode: 10_002 + probe_count as u64,
            before: probe_before.clone(),
            action: target_action.clone(),
            after: probe_after,
        };
        match solver.apply_probe(&probe)? {
            ObservationVerdict::Revised { epoch } => {
                revised = true;
                stages.push((
                    "REVISE",
                    format!("local revision committed; epoch={epoch}; witnesses retained"),
                ));
                break;
            }
            ObservationVerdict::Abstain => {
                probe_base = probe_before;
                stages.push((
                    "ACTIVE_PROBE",
                    format!("probe #{probe_count} retained competing hypotheses"),
                ));
            }
            other => return Err(format!("unexpected probe verdict: {other:?}")),
        }
    }
    if !revised {
        return Ok(opaque_report(
            &spec,
            OpaqueRunStatus::Abstain,
            "bounded probes did not identify a unique revision",
            &stages,
            source_logs.len(),
            probe_count,
            false,
            false,
            solver.ledger_verified(),
            solver.source_model_unchanged(),
            false,
        ));
    }

    let holdout_positive_state = spec
        .holdout
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, CustomNodeKind::StateField))
        .map(|node| (node.id.clone(), hidden_guard.holdout.contains(&node.id)))
        .collect();
    let holdout_positive_before = holdout_environment
        .apply_intervention(&OpaqueIntervention {
            state: holdout_positive_state,
        })
        .map_err(|error| error.to_string())?;
    let holdout_action = holdout_environment.action.clone();
    let holdout_positive_after = holdout_environment
        .send_action(&holdout_action)
        .map_err(|error| error.to_string())?;
    let holdout_positive = ObservationLog {
        episode: 20_001,
        before: holdout_positive_before,
        action: holdout_action.clone(),
        after: holdout_positive_after,
    };
    let holdout_negative_state = spec
        .holdout
        .nodes
        .iter()
        .filter(|node| matches!(node.kind, CustomNodeKind::StateField))
        .map(|node| (node.id.clone(), false))
        .collect();
    let holdout_negative_before = holdout_environment
        .apply_intervention(&OpaqueIntervention {
            state: holdout_negative_state,
        })
        .map_err(|error| error.to_string())?;
    let holdout_negative_after = holdout_environment
        .send_action(&holdout_action)
        .map_err(|error| error.to_string())?;
    let holdout_negative = ObservationLog {
        episode: 20_002,
        before: holdout_negative_before,
        action: holdout_action,
        after: holdout_negative_after,
    };
    let (positive, negative) = solver.verify_holdout(&holdout_positive, &holdout_negative)?;
    stages.push((
        "HOLDOUT",
        format!("positive={positive}; negative={negative}; first_attempt=true"),
    ));
    let status = if positive && negative {
        OpaqueRunStatus::HoldoutVerified
    } else {
        OpaqueRunStatus::Abstain
    };
    Ok(opaque_report(
        &spec,
        status,
        if status == OpaqueRunStatus::HoldoutVerified {
            "opaque action-port cycle verified"
        } else {
            "holdout evidence was insufficient"
        },
        &stages,
        source_logs.len(),
        probe_count,
        positive,
        negative,
        solver.ledger_verified(),
        solver.source_model_unchanged(),
        positive && negative,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::custom_mechanism::{
        CustomEdgeSpec, CustomNodeKind, CustomNodeSpec, CustomWorldSpec,
    };

    fn sample_spec() -> CustomChallengeSpec {
        let nodes = |operation: &str, first: &str, second: &str, effect: &str| {
            vec![
                CustomNodeSpec {
                    id: operation.into(),
                    kind: CustomNodeKind::Operation,
                    type_tag: 10,
                },
                CustomNodeSpec {
                    id: first.into(),
                    kind: CustomNodeKind::StateField,
                    type_tag: 11,
                },
                CustomNodeSpec {
                    id: second.into(),
                    kind: CustomNodeKind::StateField,
                    type_tag: 12,
                },
                CustomNodeSpec {
                    id: effect.into(),
                    kind: CustomNodeKind::StateField,
                    type_tag: 13,
                },
            ]
        };
        let edges = |operation: &str, first: &str, second: &str, effect: &str| {
            vec![
                CustomEdgeSpec {
                    from: operation.into(),
                    to: first.into(),
                    kind: CustomEdgeKind::Requires,
                },
                CustomEdgeSpec {
                    from: operation.into(),
                    to: second.into(),
                    kind: CustomEdgeKind::Requires,
                },
                CustomEdgeSpec {
                    from: operation.into(),
                    to: effect.into(),
                    kind: CustomEdgeKind::Writes,
                },
            ]
        };
        let before = BTreeMap::from([
            ("pressure".into(), true),
            ("latch".into(), true),
            ("open".into(), false),
        ]);
        let after = BTreeMap::from([
            ("pressure".into(), true),
            ("latch".into(), true),
            ("open".into(), true),
        ]);
        CustomChallengeSpec {
            name: "opaque hydraulic clamp".into(),
            seed: 42,
            source: CustomWorldSpec {
                action: "press".into(),
                nodes: nodes("press", "pressure", "latch", "open"),
                edges: edges("press", "pressure", "latch", "open"),
                effect: "open".into(),
            },
            source_examples: (0..3)
                .map(|_| CustomTransitionSpec {
                    before: before.clone(),
                    after: after.clone(),
                })
                .collect(),
            target: CustomWorldSpec {
                action: "fire_grip".into(),
                nodes: nodes("fire_grip", "hydraulic_ok", "safety_pin", "jaws_closed"),
                edges: edges("fire_grip", "hydraulic_ok", "safety_pin", "jaws_closed"),
                effect: "jaws_closed".into(),
            },
            holdout: CustomWorldSpec {
                action: "engage".into(),
                nodes: nodes("engage", "line_live", "interlock_clear", "locked"),
                edges: edges("engage", "line_live", "interlock_clear", "locked"),
                effect: "locked".into(),
            },
            hidden_guard: None,
        }
    }

    #[test]
    fn opaque_solver_input_contains_no_world_or_guard_fields() {
        let log = ObservationLog {
            episode: 1,
            before: OpaqueObservation {
                objects: vec![OpaqueObject {
                    token: "x".into(),
                    kind: OpaqueObjectKind::StateField,
                    role_tag: 1,
                    value: Some(false),
                }],
                relations: Vec::new(),
            },
            action: OpaqueAction {
                token: "a".into(),
                operation: "op".into(),
            },
            after: OpaqueObservation {
                objects: Vec::new(),
                relations: Vec::new(),
            },
        };
        let encoded = serde_json::to_string(&log).expect("observation log serializes");
        assert!(!encoded.contains("hidden_guard"));
        assert!(!encoded.contains("CustomWorldSpec"));
    }

    #[test]
    fn opaque_port_cycle_verifies_custom_mechanism() {
        let report = run_opaque_custom(sample_spec()).expect("opaque challenge should run");
        assert!(report.succeeded(), "{}", report.jsonl());
        assert_eq!(report.status, OpaqueRunStatus::HoldoutVerified);
        assert!(report.jsonl().contains("observation_logs_only"));
        assert!(!report.jsonl().contains("hydraulic_ok"));
    }

    #[test]
    fn opaque_port_rejects_wrong_holdout_topology() {
        let mut spec = sample_spec();
        spec.holdout.edges[2].kind = CustomEdgeKind::Reads;
        let report = run_opaque_custom(spec).expect("schema is still parseable");
        assert!(!report.succeeded());
        assert!(matches!(
            report.status,
            OpaqueRunStatus::Abstain | OpaqueRunStatus::Refused
        ));
    }

    #[test]
    fn opaque_environment_does_not_leak_guard_in_observation() {
        let spec = sample_spec();
        let guard = derive_guard(&spec.target, &spec.holdout, spec.seed).expect("guard");
        let mut env = OpaqueEnvironment::from_world(&spec.target, &guard.target).expect("env");
        let json = serde_json::to_string(&env.reset()).expect("observation serializes");
        assert!(json.contains("hydraulic_ok"));
        assert!(!json.contains("hidden_guard"));
        assert!(!json.contains("target_guard"));
    }

    #[test]
    fn continuous_adapter_returns_bounded_state_and_scope_digests() {
        let config = crate::deterministic_abstraction::DeterministicAbstractionConfig::new(
            vec![
                crate::deterministic_abstraction::ChannelRange::new(10, 0, 1_000_000)
                    .expect("range"),
            ],
            4,
            25_000,
            1,
            4,
            77,
        )
        .expect("config");
        let schema_digest = config.schema_digest();
        let mut abstraction = DeterministicAbstraction::new(config);
        let frame = ContinuousFrame::new(vec![
            crate::deterministic_abstraction::FixedScalar::from_micros(10, 250_000),
        ])
        .expect("frame");
        let observation =
            symbolize_continuous_frame(&mut abstraction, &frame).expect("symbolic observation");
        assert_eq!(observation.schema_digest(), schema_digest);
        assert_eq!(observation.atoms().len(), 1);
        assert_eq!(observation.state().literals().count(), 4);
        assert_eq!(observation.hypervector().len(), 1);
    }

    #[test]
    fn proof_freeze_requires_supported_target_evidence() {
        let spec = sample_spec();
        let hidden_guard =
            derive_guard(&spec.target, &spec.holdout, spec.seed).expect("guard");
        let source_guard =
            source_guard_from_target(&spec.source, &spec.target, &hidden_guard.target)
                .expect("source guard");
        let mut source_environment =
            OpaqueEnvironment::from_world(&spec.source, &source_guard).expect("source env");
        let source_logs =
            source_logs(&mut source_environment, &spec.source_examples).expect("source logs");
        let mut solver = OpaqueActionSolver::new();
        solver.learn_source(&source_logs).expect("source learning");

        let mut target_environment =
            OpaqueEnvironment::from_world(&spec.target, &hidden_guard.target)
                .expect("target env");
        let target_before = target_environment
            .apply_intervention(&OpaqueIntervention {
                state: session_state(&spec.target, &hidden_guard.target),
            })
            .expect("target before");
        let target_action = target_environment.action.clone();
        assert_eq!(
            solver
                .install_target(&target_before, &target_action)
                .expect("target install"),
            TransferStatus::StructuralCandidate
        );
        assert!(solver.freeze_target_model(8, 8).is_err());

        let target_after = target_environment
            .send_action(&target_action)
            .expect("target action");
        let target_trial = ObservationLog {
            episode: 90_001,
            before: target_before,
            action: target_action,
            after: target_after,
        };
        assert_eq!(
            solver.observe_target(&target_trial).expect("target trial"),
            ObservationVerdict::Confirmed
        );
        assert!(solver.freeze_target_model(8, 8).is_ok());
    }

    #[test]
    fn interactive_session_verifies_multiple_fresh_seeds() {
        for seed in 1..=32 {
            let mut session = OpaqueInteractionSession::new(seed).expect("session");
            let token = session.action_token().to_owned();
            session.send_action(&token).expect("target trial");
            session.sabotage().expect("sabotage");
            for _ in 0..8 {
                if session.phase() == OpaqueSessionPhase::Revised {
                    break;
                }
                session.probe().expect("bounded probe");
            }
            assert_eq!(session.phase(), OpaqueSessionPhase::Revised, "seed={seed}");
            let output = session.verify_holdout().expect("holdout");
            assert!(output.contains("\"holdout_positive\":true"), "seed={seed}");
            assert!(output.contains("\"holdout_negative\":true"), "seed={seed}");
            assert!(!output.contains("hidden_guard"), "seed={seed}");
        }
    }
}
