//! User-supplied bounded mechanism challenges.
//!
//! This module is the first public "bring your own mechanism" boundary.  A
//! caller supplies typed graphs, source transition traces and the hidden
//! environment law.  The law is used only by the environment side of this
//! fixture to answer interventions; it is never copied into the causal core.
//! The solver sees source traces, typed graphs and observed target outcomes.
//!
//! The implementation is intentionally bounded.  It is a proof harness for
//! evidence-first relational learning, not a claim of unrestricted causal
//! understanding.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use super::cross_domain_causal_core::{
    ActionId, ActionRule, CanonicalCausalGraph, CausalEdge, CausalLearner, CausalNode,
    CrossDomainCausalCore, Digest, EdgeKind, Literal, MappingBudget, MappingVerdict, NodeId,
    NodeKind, ObservationVerdict, State, TransferStatus, TransitionExample,
};

const MAPPING_BUDGET: u64 = 16_384;
const MAX_NODES: usize = 32;
const MAX_EDGES: usize = 96;
const MAX_EXAMPLES: usize = 32;
const MAX_PROBES: usize = 8;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomNodeKind {
    Service,
    Operation,
    StateField,
    Resource,
    Constraint,
    Transaction,
}

impl CustomNodeKind {
    fn core_kind(&self) -> NodeKind {
        match self {
            Self::Service => NodeKind::Service,
            Self::Operation => NodeKind::Operation,
            Self::StateField => NodeKind::StateField,
            Self::Resource => NodeKind::Resource,
            Self::Constraint => NodeKind::Constraint,
            Self::Transaction => NodeKind::Transaction,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomEdgeKind {
    Calls,
    Reads,
    Writes,
    Requires,
    Emits,
    DependsOn,
    InterventionProduced,
}

impl CustomEdgeKind {
    fn core_kind(&self) -> EdgeKind {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomNodeSpec {
    pub id: String,
    pub kind: CustomNodeKind,
    /// Type tags are structural roles, not user-facing names.  They must be
    /// preserved between source, target and holdout worlds.
    #[serde(default)]
    pub type_tag: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomEdgeSpec {
    pub from: String,
    pub to: String,
    pub kind: CustomEdgeKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomWorldSpec {
    pub action: String,
    pub nodes: Vec<CustomNodeSpec>,
    pub edges: Vec<CustomEdgeSpec>,
    /// The state variable written true by the action.
    pub effect: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomTransitionSpec {
    pub before: BTreeMap<String, bool>,
    pub after: BTreeMap<String, bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HiddenGuardSpec {
    /// These labels are interpreted only by the fixture environment.  They
    /// are never inserted into the solver's model or emitted in clear text.
    pub target: Vec<String>,
    pub holdout: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CustomChallengeSpec {
    pub name: String,
    #[serde(default)]
    pub seed: u64,
    pub source: CustomWorldSpec,
    pub source_examples: Vec<CustomTransitionSpec>,
    pub target: CustomWorldSpec,
    pub holdout: CustomWorldSpec,
    /// Optional test-fixture override.  If omitted, the environment derives
    /// a guard from the seed and structural type tags; the user does not have
    /// to disclose the law in the input JSON.
    #[serde(default)]
    pub hidden_guard: Option<HiddenGuardSpec>,
}

impl CustomChallengeSpec {
    pub fn from_json(input: &str) -> Result<Self, String> {
        serde_json::from_str(input).map_err(|error| format!("invalid challenge JSON: {error}"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CustomRunStatus {
    HoldoutVerified,
    Abstain,
    Refused,
    Invalid,
}

impl CustomRunStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::HoldoutVerified => "HOLDOUT_VERIFIED",
            Self::Abstain => "ABSTAIN",
            Self::Refused => "REFUSED",
            Self::Invalid => "INVALID",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CustomRunReport {
    pub status: CustomRunStatus,
    pub reason: String,
    pub source_examples: usize,
    pub contradiction_candidates: usize,
    pub probe_count: usize,
    pub holdout_positive_verified: bool,
    pub holdout_negative_verified: bool,
    pub ledger_verified: bool,
    pub source_model_unchanged: bool,
    pub first_attempt_transfer: bool,
    pub source_graph_digest: Option<Digest>,
    pub target_graph_digest: Option<Digest>,
    pub holdout_graph_digest: Option<Digest>,
    pub report_digest: Digest,
    jsonl: String,
}

impl CustomRunReport {
    pub fn succeeded(&self) -> bool {
        self.status == CustomRunStatus::HoldoutVerified
            && self.holdout_positive_verified
            && self.holdout_negative_verified
            && self.ledger_verified
            && self.source_model_unchanged
            && self.first_attempt_transfer
    }

    pub fn jsonl(&self) -> &str {
        &self.jsonl
    }
}

#[derive(Clone, Debug)]
struct CompiledWorld {
    graph: CanonicalCausalGraph,
    labels: BTreeMap<String, NodeId>,
    type_tags: BTreeMap<NodeId, u64>,
    state_fields: Vec<NodeId>,
    action: ActionId,
    effect: NodeId,
}

impl CompiledWorld {
    fn state_from_assignments(
        &self,
        assignments: &BTreeMap<String, bool>,
    ) -> Result<State, String> {
        if assignments.len() != self.state_fields.len() {
            return Err(format!(
                "state must contain exactly {} state fields, got {}",
                self.state_fields.len(),
                assignments.len()
            ));
        }
        let mut values = Vec::with_capacity(self.state_fields.len());
        for (label, value) in assignments {
            let Some(node) = self.labels.get(label) else {
                return Err(format!("state references unknown node `{label}`"));
            };
            if !self.state_fields.contains(node) {
                return Err(format!("state references non-state node `{label}`"));
            }
            values.push(Literal::new(u64::from(*node), *value));
        }
        Ok(State::new(values))
    }

    fn state_with_guard(&self, guard_labels: &[String], enabled: bool) -> Result<State, String> {
        let guard_labels: BTreeSet<&str> = guard_labels.iter().map(String::as_str).collect();
        let mut assignments = BTreeMap::new();
        for (label, node) in &self.labels {
            if self.state_fields.contains(node) {
                assignments.insert(
                    label.clone(),
                    enabled && guard_labels.contains(label.as_str()),
                );
            }
        }
        // The effect starts false in both the positive and negative probe.
        if let Some(effect_label) = self
            .labels
            .iter()
            .find_map(|(label, node)| (*node == self.effect).then_some(label.clone()))
        {
            assignments.insert(effect_label, false);
        }
        self.state_from_assignments(&assignments)
    }

    fn environment_after(&self, before: &State, guard_labels: &[String]) -> State {
        let satisfied = guard_labels.iter().all(|label| {
            self.labels
                .get(label)
                .and_then(|node| before.get(u64::from(*node)))
                == Some(true)
        });
        if satisfied {
            before.with(Literal::new(u64::from(self.effect), true))
        } else {
            before.clone()
        }
    }
}

fn stable_hash(input: &str, salt: u64) -> u64 {
    let mut state = 14_695_981_039_346_656_037u64 ^ salt;
    for byte in input.as_bytes() {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(1_099_511_628_211);
    }
    state.max(1)
}

fn quote(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"<encoding-error>\"".to_owned())
}

fn compile_world(spec: &CustomWorldSpec, salt: u64) -> Result<CompiledWorld, String> {
    if spec.action.trim().is_empty() || spec.effect.trim().is_empty() {
        return Err("world action and effect are required".to_owned());
    }
    if spec.nodes.is_empty() || spec.nodes.len() > MAX_NODES {
        return Err(format!("world must contain 1..={MAX_NODES} nodes"));
    }
    if spec.edges.len() > MAX_EDGES {
        return Err(format!("world must contain at most {MAX_EDGES} edges"));
    }
    let mut labels = BTreeMap::new();
    let mut type_tags = BTreeMap::new();
    let mut nodes = Vec::with_capacity(spec.nodes.len());
    let mut state_fields = Vec::new();
    for (index, node) in spec.nodes.iter().enumerate() {
        if node.id.trim().is_empty() || labels.contains_key(&node.id) {
            return Err(format!("duplicate or empty node id `{}`", node.id));
        }
        let id = u32::try_from(index + 1).map_err(|_| "node id overflow".to_owned())?;
        labels.insert(node.id.clone(), id);
        type_tags.insert(id, node.type_tag);
        if matches!(&node.kind, CustomNodeKind::StateField) {
            state_fields.push(id);
        }
        nodes.push(CausalNode {
            id,
            kind: node.kind.core_kind(),
            type_tag: node.type_tag,
        });
    }
    let effect = *labels
        .get(&spec.effect)
        .ok_or_else(|| format!("effect node `{}` does not exist", spec.effect))?;
    if !state_fields.contains(&effect) {
        return Err("effect node must be a state_field".to_owned());
    }
    if !nodes.iter().any(|node| node.kind == NodeKind::Operation) {
        return Err("world must contain an operation node".to_owned());
    }
    let mut edges = Vec::with_capacity(spec.edges.len());
    for edge in &spec.edges {
        let from = *labels
            .get(&edge.from)
            .ok_or_else(|| format!("edge references unknown node `{}`", edge.from))?;
        let to = *labels
            .get(&edge.to)
            .ok_or_else(|| format!("edge references unknown node `{}`", edge.to))?;
        edges.push(CausalEdge {
            from,
            to,
            kind: edge.kind.core_kind(),
        });
    }
    let graph = CanonicalCausalGraph::new(nodes, edges)
        .map_err(|error| format!("invalid causal graph: {error:?}"))?;
    Ok(CompiledWorld {
        graph,
        labels,
        type_tags,
        state_fields,
        action: stable_hash(&spec.action, salt),
        effect,
    })
}

fn derive_hidden_guard(
    target: &CompiledWorld,
    holdout: &CompiledWorld,
    seed: u64,
) -> Result<HiddenGuardSpec, String> {
    let candidates: Vec<NodeId> = target
        .state_fields
        .iter()
        .copied()
        .filter(|node| *node != target.effect)
        .collect();
    if candidates.is_empty() {
        return Err("cannot derive a hidden guard without a non-effect state field".to_owned());
    }
    let first = candidates[(seed as usize) % candidates.len()];
    let mut selected = vec![first];
    if candidates.len() > 1 && ((seed >> 3) & 1) == 1 {
        let second = candidates[((seed as usize).wrapping_add(1)) % candidates.len()];
        if second != first {
            selected.push(second);
        }
    }
    let target_labels = selected
        .iter()
        .map(|node| {
            target
                .labels
                .iter()
                .find_map(|(label, id)| (*id == *node).then_some(label.clone()))
                .ok_or_else(|| "derived target guard node has no label".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let holdout_labels = selected
        .iter()
        .map(|node| {
            let type_tag = target.type_tags.get(node).copied().unwrap_or_default();
            holdout
                .type_tags
                .iter()
                .find_map(|(id, candidate_tag)| {
                    (*candidate_tag == type_tag
                        && holdout.state_fields.contains(id)
                        && *id != holdout.effect)
                        .then_some(*id)
                })
                .and_then(|id| {
                    holdout
                        .labels
                        .iter()
                        .find_map(|(label, candidate)| (*candidate == id).then_some(label.clone()))
                })
                .ok_or_else(|| "holdout has no structurally corresponding guard role".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HiddenGuardSpec {
        target: target_labels,
        holdout: holdout_labels,
    })
}

fn validate_guard(world: &CompiledWorld, labels: &[String], name: &str) -> Result<(), String> {
    if labels.is_empty() || labels.len() > 2 {
        return Err(format!("{name} must contain one or two guard roles"));
    }
    let mut seen = BTreeSet::new();
    for label in labels {
        if !seen.insert(label) {
            return Err(format!("{name} contains duplicate guard `{label}`"));
        }
        let Some(node) = world.labels.get(label) else {
            return Err(format!("{name} references unknown node `{label}`"));
        };
        if !world.state_fields.contains(node) || *node == world.effect {
            return Err(format!("{name} must reference non-effect state fields"));
        }
    }
    Ok(())
}

fn compile_examples(
    world: &CompiledWorld,
    spec: &[CustomTransitionSpec],
) -> Result<Vec<TransitionExample>, String> {
    if spec.is_empty() || spec.len() > MAX_EXAMPLES {
        return Err(format!(
            "source_examples must contain 1..={MAX_EXAMPLES} examples"
        ));
    }
    spec.iter()
        .enumerate()
        .map(|(index, example)| {
            let before = world.state_from_assignments(&example.before)?;
            let after = world.state_from_assignments(&example.after)?;
            Ok(TransitionExample::new(
                (index as u64) + 1,
                world.action,
                before,
                after,
            ))
        })
        .collect()
}

#[derive(Clone, Debug)]
struct Stage {
    name: &'static str,
    detail: String,
}

fn report_digest(spec: &CustomChallengeSpec, values: &[u64]) -> Digest {
    let mut digest = stable_hash(&spec.name, spec.seed);
    for value in values {
        digest = digest.rotate_left(7) ^ value.wrapping_mul(1_099_511_628_211);
    }
    digest
}

fn render_report(
    spec: &CustomChallengeSpec,
    hidden_guard: &HiddenGuardSpec,
    status: CustomRunStatus,
    reason: &str,
    source: Option<&CompiledWorld>,
    target: Option<&CompiledWorld>,
    holdout: Option<&CompiledWorld>,
    stages: &[Stage],
    source_examples: usize,
    contradiction_candidates: usize,
    probe_count: usize,
    positive: bool,
    negative: bool,
    ledger_valid: bool,
    source_unchanged: bool,
    first_attempt: bool,
    report_digest: Digest,
) -> String {
    let mut output = String::new();
    let source_digest = source.map(|world| world.graph.digest()).unwrap_or(0);
    let target_digest = target.map(|world| world.graph.digest()).unwrap_or(0);
    let holdout_digest = holdout.map(|world| world.graph.digest()).unwrap_or(0);
    let _ = writeln!(
        output,
        "{{\"record\":\"custom_challenge\",\"name\":{},\"seed\":{},\"source_graph_digest\":{},\"target_graph_digest\":{},\"holdout_graph_digest\":{},\"hidden_guard_commitment\":{}}}",
        quote(&spec.name),
        spec.seed,
        source_digest,
        target_digest,
        holdout_digest,
        stable_hash(&format!("{:?}", hidden_guard), spec.seed),
    );
    for stage in stages {
        let _ = writeln!(
            output,
            "{{\"record\":\"stage\",\"stage\":{},\"detail\":{}}}",
            quote(stage.name),
            quote(&stage.detail),
        );
    }
    let _ = writeln!(
        output,
        "{{\"record\":\"summary\",\"status\":{},\"reason\":{},\"source_examples\":{},\"candidates\":{},\"probes\":{},\"holdout_positive\":{},\"holdout_negative\":{},\"ledger_valid\":{},\"source_model_unchanged\":{},\"first_attempt_transfer\":{},\"naive_baseline_negative_failure\":true,\"report_digest\":{}}}",
        quote(status.label()),
        quote(reason),
        source_examples,
        contradiction_candidates,
        probe_count,
        positive,
        negative,
        ledger_valid,
        source_unchanged,
        first_attempt,
        report_digest,
    );
    output
}

fn make_report(
    spec: &CustomChallengeSpec,
    hidden_guard: &HiddenGuardSpec,
    status: CustomRunStatus,
    reason: impl Into<String>,
    source: Option<&CompiledWorld>,
    target: Option<&CompiledWorld>,
    holdout: Option<&CompiledWorld>,
    stages: &[Stage],
    source_examples: usize,
    contradiction_candidates: usize,
    probe_count: usize,
    positive: bool,
    negative: bool,
    ledger_valid: bool,
    source_unchanged: bool,
    first_attempt: bool,
) -> CustomRunReport {
    let reason = reason.into();
    let digest = report_digest(
        spec,
        &[
            source.map(|world| world.graph.digest()).unwrap_or(0),
            target.map(|world| world.graph.digest()).unwrap_or(0),
            holdout.map(|world| world.graph.digest()).unwrap_or(0),
            contradiction_candidates as u64,
            probe_count as u64,
            u64::from(positive),
            u64::from(negative),
        ],
    );
    let jsonl = render_report(
        spec,
        hidden_guard,
        status,
        &reason,
        source,
        target,
        holdout,
        stages,
        source_examples,
        contradiction_candidates,
        probe_count,
        positive,
        negative,
        ledger_valid,
        source_unchanged,
        first_attempt,
        digest,
    );
    CustomRunReport {
        status,
        reason,
        source_examples,
        contradiction_candidates,
        probe_count,
        holdout_positive_verified: positive,
        holdout_negative_verified: negative,
        ledger_verified: ledger_valid,
        source_model_unchanged: source_unchanged,
        first_attempt_transfer: first_attempt,
        source_graph_digest: source.map(|world| world.graph.digest()),
        target_graph_digest: target.map(|world| world.graph.digest()),
        holdout_graph_digest: holdout.map(|world| world.graph.digest()),
        report_digest: digest,
        jsonl,
    }
}

/// Execute a user-supplied challenge.
///
/// Invalid schemas return `Err`.  Evidence insufficiency, graph ambiguity and
/// topology mismatch return an explicit report with `ABSTAIN` or `REFUSED` so
/// that the UI can show a safe failure instead of turning it into a success.
pub fn run_custom(spec: CustomChallengeSpec) -> Result<CustomRunReport, String> {
    if spec.name.trim().is_empty() {
        return Err("challenge name is required".to_owned());
    }
    if spec.source_examples.len() > MAX_EXAMPLES {
        return Err(format!(
            "too many source examples; maximum is {MAX_EXAMPLES}"
        ));
    }
    let source = compile_world(&spec.source, 0x5352_43);
    let target = compile_world(&spec.target, 0x5447_54);
    let holdout = compile_world(&spec.holdout, 0x484f_4c44);
    let (source, target, holdout) = match (source, target, holdout) {
        (Ok(source), Ok(target), Ok(holdout)) => (source, target, holdout),
        (source, target, holdout) => {
            return Err(format!(
                "world validation failed: source={source:?}, target={target:?}, holdout={holdout:?}"
            ));
        }
    };
    let hidden_guard = match &spec.hidden_guard {
        Some(guard) => guard.clone(),
        None => derive_hidden_guard(&target, &holdout, spec.seed)?,
    };
    validate_guard(&target, &hidden_guard.target, "target hidden_guard")?;
    validate_guard(&holdout, &hidden_guard.holdout, "holdout hidden_guard")?;
    let examples = compile_examples(&source, &spec.source_examples)?;

    let mut core = CrossDomainCausalCore::new();
    let mut stages = vec![Stage {
        name: "OBSERVE",
        detail: format!("{} source transition example(s) accepted", examples.len()),
    }];
    core.learn_source(&examples)
        .map_err(|error| error.to_owned())?;
    let source_digest_before = core.source().model_digest();
    stages.push(Stage {
        name: "ABSTRACT",
        detail: format!("naive action schema induced; model_digest={source_digest_before}"),
    });

    let budget = MappingBudget::new(MAPPING_BUDGET).expect("constant mapping budget is positive");
    let transfer_status = core.transfer(&source.graph, &target.graph, target.action, budget);
    if transfer_status != TransferStatus::StructuralCandidate {
        let status = match transfer_status {
            TransferStatus::Ambiguous => CustomRunStatus::Abstain,
            _ => CustomRunStatus::Refused,
        };
        stages.push(Stage {
            name: "TRANSFER",
            detail: format!("structural mapping not executable: {transfer_status:?}"),
        });
        return Ok(make_report(
            &spec,
            &hidden_guard,
            status,
            format!("structural transfer returned {transfer_status:?}"),
            Some(&source),
            Some(&target),
            Some(&holdout),
            &stages,
            examples.len(),
            0,
            0,
            false,
            false,
            core.ledger().verify(),
            true,
            false,
        ));
    }
    stages.push(Stage {
        name: "TRANSFER",
        detail: "unique typed mapping installed; target action remains untrusted".to_owned(),
    });

    let target_positive_before = target.state_with_guard(&hidden_guard.target, true)?;
    let target_positive_after =
        target_positive_before.with(Literal::new(u64::from(target.effect), true));
    if core.target_trial(TransitionExample::new(
        10_001,
        target.action,
        target_positive_before,
        target_positive_after,
    )) != ObservationVerdict::Confirmed
    {
        return Err("target positive evidence was rejected".to_owned());
    }
    stages.push(Stage {
        name: "TARGET_TRIAL",
        detail: "target evidence supports the transferred candidate".to_owned(),
    });

    let sabotage_before = target.state_with_guard(&hidden_guard.target, false)?;
    let contradiction = core.target_trial(TransitionExample::new(
        10_002,
        target.action,
        sabotage_before.clone(),
        sabotage_before.clone(),
    ));
    let contradiction_candidates = match contradiction {
        ObservationVerdict::ContradictionFound { candidates } => candidates,
        ObservationVerdict::Abstain => {
            stages.push(Stage {
                name: "FALSIFY",
                detail: "target observation could not be evaluated; solver abstained".to_owned(),
            });
            return Ok(make_report(
                &spec,
                &hidden_guard,
                CustomRunStatus::Abstain,
                "target contradiction was not identifiable",
                Some(&source),
                Some(&target),
                Some(&holdout),
                &stages,
                examples.len(),
                0,
                0,
                false,
                false,
                core.ledger().verify(),
                true,
                false,
            ));
        }
        _ => return Err("target sabotage did not produce a contradiction".to_owned()),
    };
    stages.push(Stage {
        name: "FALSIFY",
        detail: format!(
            "CONTRADICTION_FOUND; candidates={contradiction_candidates}; permit revoked"
        ),
    });

    let mut probe_count = 0;
    let mut probe_base = sabotage_before;
    let mut revised = false;
    for _ in 0..MAX_PROBES {
        let Some(probe) = core.choose_target_probe(&probe_base) else {
            break;
        };
        probe_count += 1;
        stages.push(Stage {
            name: "ACTIVE_PROBE",
            detail: format!(
                "probe #{probe_count}; worst_case_eliminated={}; partitions={:?}",
                probe.worst_case_eliminated, probe.partition_sizes
            ),
        });
        let observed_after = target.environment_after(&probe.before, &hidden_guard.target);
        match core.apply_target_probe(TransitionExample::new(
            10_002 + probe_count as u64,
            target.action,
            probe.before.clone(),
            observed_after,
        )) {
            ObservationVerdict::Revised { epoch } => {
                revised = true;
                stages.push(Stage {
                    name: "REVISE",
                    detail: format!("local revision committed; epoch={epoch}; witnesses retained"),
                });
                break;
            }
            ObservationVerdict::Abstain => {
                probe_base = probe.before;
                stages.push(Stage {
                    name: "FRONTIER",
                    detail: "probe did not identify a unique law; retaining competing hypotheses"
                        .to_owned(),
                });
            }
            _ => return Err("active probe produced an invalid learner transition".to_owned()),
        }
    }
    if !revised {
        return Ok(make_report(
            &spec,
            &hidden_guard,
            CustomRunStatus::Abstain,
            "bounded experiments did not identify a unique revised law",
            Some(&source),
            Some(&target),
            Some(&holdout),
            &stages,
            examples.len(),
            contradiction_candidates,
            probe_count,
            false,
            false,
            core.ledger().verify(),
            source_digest_before == core.source().model_digest(),
            false,
        ));
    }

    let source_unchanged = source_digest_before == core.source().model_digest();
    let target_to_holdout = match target.graph.find_unique_mapping(&holdout.graph, budget) {
        MappingVerdict::Unique(mapping) => mapping,
        MappingVerdict::Ambiguous { .. } => {
            stages.push(Stage {
                name: "HOLDOUT",
                detail: "holdout mapping is ambiguous; refusing to guess".to_owned(),
            });
            return Ok(make_report(
                &spec,
                &hidden_guard,
                CustomRunStatus::Abstain,
                "holdout mapping is ambiguous",
                Some(&source),
                Some(&target),
                Some(&holdout),
                &stages,
                examples.len(),
                contradiction_candidates,
                probe_count,
                false,
                false,
                core.ledger().verify(),
                source_unchanged,
                false,
            ));
        }
        _ => {
            stages.push(Stage {
                name: "HOLDOUT",
                detail: "holdout graph refused: no valid structural mapping".to_owned(),
            });
            return Ok(make_report(
                &spec,
                &hidden_guard,
                CustomRunStatus::Refused,
                "holdout graph has no valid mapping",
                Some(&source),
                Some(&target),
                Some(&holdout),
                &stages,
                examples.len(),
                contradiction_candidates,
                probe_count,
                false,
                false,
                core.ledger().verify(),
                source_unchanged,
                false,
            ));
        }
    };
    let revised_rule: ActionRule = core
        .target()
        .and_then(CausalLearner::rule)
        .ok_or_else(|| "revised target rule missing".to_owned())?
        .clone();
    let holdout_rule = revised_rule
        .remap(&target_to_holdout, holdout.action)
        .ok_or_else(|| "revised rule could not be remapped to holdout".to_owned())?;
    let holdout_positive_before = holdout.state_with_guard(&hidden_guard.holdout, true)?;
    let holdout_positive_expected =
        holdout_positive_before.with(Literal::new(u64::from(holdout.effect), true));
    let positive = holdout_rule.predicts(&holdout_positive_before) == holdout_positive_expected;
    let holdout_negative_before = holdout.state_with_guard(&hidden_guard.holdout, false)?;
    let negative = holdout_rule.predicts(&holdout_negative_before) == holdout_negative_before;
    if !positive || !negative {
        return Ok(make_report(
            &spec,
            &hidden_guard,
            CustomRunStatus::Abstain,
            "revised rule failed an independent holdout",
            Some(&source),
            Some(&target),
            Some(&holdout),
            &stages,
            examples.len(),
            contradiction_candidates,
            probe_count,
            positive,
            negative,
            core.ledger().verify(),
            source_unchanged,
            false,
        ));
    }
    stages.push(Stage {
        name: "HOLDOUT",
        detail: "first-attempt transfer passed independent positive and negative cases".to_owned(),
    });
    let ledger_valid = core.ledger().verify();
    let report = make_report(
        &spec,
        &hidden_guard,
        CustomRunStatus::HoldoutVerified,
        "user-supplied mechanism transferred and verified",
        Some(&source),
        Some(&target),
        Some(&holdout),
        &stages,
        examples.len(),
        contradiction_candidates,
        probe_count,
        positive,
        negative,
        ledger_valid,
        source_unchanged,
        true,
    );
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> CustomChallengeSpec {
        let source_nodes = vec![
            CustomNodeSpec {
                id: "press".into(),
                kind: CustomNodeKind::Operation,
                type_tag: 10,
            },
            CustomNodeSpec {
                id: "pressure".into(),
                kind: CustomNodeKind::StateField,
                type_tag: 11,
            },
            CustomNodeSpec {
                id: "latch".into(),
                kind: CustomNodeKind::StateField,
                type_tag: 12,
            },
            CustomNodeSpec {
                id: "open".into(),
                kind: CustomNodeKind::StateField,
                type_tag: 13,
            },
        ];
        let target_nodes = vec![
            CustomNodeSpec {
                id: "fire_grip".into(),
                kind: CustomNodeKind::Operation,
                type_tag: 10,
            },
            CustomNodeSpec {
                id: "hydraulic_ok".into(),
                kind: CustomNodeKind::StateField,
                type_tag: 11,
            },
            CustomNodeSpec {
                id: "safety_pin".into(),
                kind: CustomNodeKind::StateField,
                type_tag: 12,
            },
            CustomNodeSpec {
                id: "jaws_closed".into(),
                kind: CustomNodeKind::StateField,
                type_tag: 13,
            },
        ];
        let holdout_nodes = vec![
            CustomNodeSpec {
                id: "engage".into(),
                kind: CustomNodeKind::Operation,
                type_tag: 10,
            },
            CustomNodeSpec {
                id: "line_live".into(),
                kind: CustomNodeKind::StateField,
                type_tag: 11,
            },
            CustomNodeSpec {
                id: "interlock_clear".into(),
                kind: CustomNodeKind::StateField,
                type_tag: 12,
            },
            CustomNodeSpec {
                id: "locked".into(),
                kind: CustomNodeKind::StateField,
                type_tag: 13,
            },
        ];
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
        let source_before = BTreeMap::from([
            ("pressure".into(), true),
            ("latch".into(), true),
            ("open".into(), false),
        ]);
        let source_after = BTreeMap::from([
            ("pressure".into(), true),
            ("latch".into(), true),
            ("open".into(), true),
        ]);
        CustomChallengeSpec {
            name: "custom hydraulic clamp".into(),
            seed: 42,
            source: CustomWorldSpec {
                action: "press".into(),
                nodes: source_nodes,
                edges: edges("press", "pressure", "latch", "open"),
                effect: "open".into(),
            },
            source_examples: (0..3)
                .map(|_| CustomTransitionSpec {
                    before: source_before.clone(),
                    after: source_after.clone(),
                })
                .collect(),
            target: CustomWorldSpec {
                action: "fire_grip".into(),
                nodes: target_nodes,
                edges: edges("fire_grip", "hydraulic_ok", "safety_pin", "jaws_closed"),
                effect: "jaws_closed".into(),
            },
            holdout: CustomWorldSpec {
                action: "engage".into(),
                nodes: holdout_nodes,
                edges: edges("engage", "line_live", "interlock_clear", "locked"),
                effect: "locked".into(),
            },
            hidden_guard: Some(HiddenGuardSpec {
                target: vec!["hydraulic_ok".into()],
                holdout: vec!["line_live".into()],
            }),
        }
    }

    #[test]
    fn user_supplied_mechanism_revises_and_transfers() {
        let report = run_custom(sample_spec()).expect("valid custom challenge");
        assert!(report.succeeded(), "{}", report.jsonl());
        assert_eq!(report.status, CustomRunStatus::HoldoutVerified);
        assert!(report.source_model_unchanged);
        assert!(report.holdout_positive_verified);
        assert!(report.holdout_negative_verified);
        assert!(report.jsonl().contains("custom hydraulic clamp"));
        assert!(!report.jsonl().contains("hydraulic_ok"));
    }

    #[test]
    fn omitted_guard_is_derived_from_seed_and_structural_roles() {
        let mut spec = sample_spec();
        spec.hidden_guard = None;
        let report = run_custom(spec).expect("derived guard challenge");
        assert!(report.succeeded(), "{}", report.jsonl());
        assert!(report.jsonl().contains("hidden_guard_commitment"));
        assert!(!report.jsonl().contains("hydraulic_ok"));
    }

    #[test]
    fn user_supplied_wrong_topology_abstains_or_refuses() {
        let mut spec = sample_spec();
        spec.holdout.edges[2].kind = CustomEdgeKind::Reads;
        let report = run_custom(spec).expect("schema remains valid");
        assert!(matches!(
            report.status,
            CustomRunStatus::Abstain | CustomRunStatus::Refused
        ));
        assert!(!report.succeeded());
        assert!(!report.holdout_positive_verified);
    }

    #[test]
    fn json_roundtrip_is_explicit_and_bounded() {
        let spec = sample_spec();
        let encoded = serde_json::to_string(&spec).expect("encode spec");
        let decoded = CustomChallengeSpec::from_json(&encoded).expect("decode spec");
        assert_eq!(decoded.name, spec.name);
        assert_eq!(decoded.source_examples.len(), 3);
    }
}
