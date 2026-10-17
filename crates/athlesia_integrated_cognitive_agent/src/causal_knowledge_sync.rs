//! Bounded Causal Knowledge Sync (CKS/1) over canonical JSONL.
//!
//! The codec transports replayable evidence and a typed source graph. Import
//! verifies the complete record chain, replays the source evidence, and only
//! then performs the existing exact graph mapping. The returned candidate is
//! immutable and has no observe, fit, revise, or execution-permit API.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::cross_domain_causal_core::{
    ActionId, ActionRule, CausalEdge, CausalLearner, CausalNode, CanonicalCausalGraph, Digest,
    EdgeKind, EvidenceLedger, Literal, MappingBudget, NodeKind, State, TokenId, TraceKind,
    TransitionExample,
};

const RECORD_TAG: &str = "athlesia/cks/1";
const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

/// Hard parser and replay limits. A limit breach rejects the whole bundle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyncLimits {
    pub max_bundle_bytes: usize,
    pub max_record_bytes: usize,
    pub max_records: usize,
    pub max_evidence: usize,
    pub max_trace_events: usize,
    pub max_nodes: usize,
    pub max_edges: usize,
    pub max_state_literals: usize,
}

impl Default for SyncLimits {
    fn default() -> Self {
        Self {
            max_bundle_bytes: 4 * 1024 * 1024,
            max_record_bytes: 64 * 1024,
            max_records: 32_768,
            max_evidence: 512,
            max_trace_events: 2_048,
            max_nodes: 256,
            max_edges: 512,
            max_state_literals: 64,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncError {
    InvalidInput(&'static str),
    MalformedJson,
    NonCanonicalRecord,
    UnsupportedProtocol(u16),
    BundleTooLarge,
    RecordTooLarge,
    RecordBudgetExceeded,
    EvidenceBudgetExceeded,
    TraceBudgetExceeded,
    InvalidSequence,
    IntegrityMismatch,
    MissingCommit,
    LedgerInvalid,
    LedgerMismatch,
    DuplicateEvidence,
    InvalidState,
    InvalidGraph,
    ModelRejected,
    IncompatibleGraph,
    AmbiguousMapping,
    MappingResourceLimit,
    OutsideScope,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for SyncError {}

/// A transport-independent codec seam. The implementation deliberately
/// carries data, not executable code or remote authority.
pub trait KnowledgeSyncCodec {
    fn export_jsonl(
        &self,
        origin_key: u64,
        origin_sequence: u64,
        module_id: u64,
        graph: &CanonicalCausalGraph,
        action: ActionId,
        evidence: &[TransitionExample],
        ledger: &EvidenceLedger,
        limits: SyncLimits,
    ) -> Result<String, SyncError>;

    fn import_jsonl(
        &self,
        jsonl: &str,
        target_graph: &CanonicalCausalGraph,
        target_action: ActionId,
        mapping_budget: MappingBudget,
        limits: SyncLimits,
    ) -> Result<SyncedKnowledgeCandidate, SyncError>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct JsonlKnowledgeSync;

/// Public protocol name; the codec remains transport-independent.
pub type CausalKnowledgeSync = JsonlKnowledgeSync;

/// A source-derived, target-bound model candidate. It is intentionally not a
/// permit and has no mutating learning methods.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncedKnowledgeCandidate {
    origin_key: u64,
    origin_sequence: u64,
    module_id: u64,
    source_graph_digest: Digest,
    target_graph_digest: Digest,
    target_action: ActionId,
    evidence_count: usize,
    bundle_digest: Digest,
    target_roles: Vec<TokenId>,
    rule: ActionRule,
}

impl SyncedKnowledgeCandidate {
    pub fn origin_key(&self) -> u64 {
        self.origin_key
    }

    pub fn origin_sequence(&self) -> u64 {
        self.origin_sequence
    }

    pub fn module_id(&self) -> u64 {
        self.module_id
    }

    pub fn source_graph_digest(&self) -> Digest {
        self.source_graph_digest
    }

    pub fn target_graph_digest(&self) -> Digest {
        self.target_graph_digest
    }

    pub fn target_action(&self) -> ActionId {
        self.target_action
    }

    pub fn evidence_count(&self) -> usize {
        self.evidence_count
    }

    pub fn bundle_digest(&self) -> Digest {
        self.bundle_digest
    }

    pub fn rule(&self) -> &ActionRule {
        &self.rule
    }

    /// Predict inside the exact target state/action scope. This method never
    /// updates the candidate and never grants execution authority.
    pub fn predict(&self, before: &State, action: ActionId) -> Result<State, SyncError> {
        if action != self.target_action {
            return Err(SyncError::OutsideScope);
        }
        let roles: Vec<TokenId> = before.literals().map(|literal| literal.role).collect();
        if roles != self.target_roles {
            return Err(SyncError::InvalidState);
        }
        Ok(self.rule.predicts(before))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireEnvelope {
    record: String,
    sequence: u64,
    previous_digest: Digest,
    body: WireBody,
    digest: Digest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WireBody {
    Manifest {
        protocol_version: u16,
        origin_key: u64,
        origin_sequence: u64,
        module_id: u64,
        source_graph_digest: Digest,
        action: ActionId,
        evidence_count: u64,
        trace_count: u64,
        graph: WireGraph,
    },
    Evidence {
        episode: u64,
        action: ActionId,
        before: Vec<(TokenId, bool)>,
        after: Vec<(TokenId, bool)>,
    },
    Trace {
        sequence: u64,
        epoch: u64,
        trace_kind: u8,
        payload_digest: Digest,
        previous_hash: Digest,
        record_hash: Digest,
    },
    Commit {
        evidence_count: u64,
        trace_count: u64,
        chain_tip: Digest,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WireGraph {
    nodes: Vec<WireNode>,
    edges: Vec<WireEdge>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WireNode {
    id: u32,
    kind: u8,
    type_tag: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WireEdge {
    from: u32,
    to: u32,
    kind: u8,
}

#[derive(Serialize)]
struct UnsignedEnvelope<'a> {
    record: &'a str,
    sequence: u64,
    previous_digest: Digest,
    body: &'a WireBody,
}

fn chain_digest(previous: Digest, bytes: &[u8]) -> Digest {
    let mut state = FNV_OFFSET;
    for byte in previous.to_le_bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    for byte in bytes {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state.max(1)
}

fn digest_for(sequence: u64, previous_digest: Digest, body: &WireBody) -> Digest {
    let unsigned = UnsignedEnvelope {
        record: RECORD_TAG,
        sequence,
        previous_digest,
        body,
    };
    let bytes = serde_json::to_vec(&unsigned).expect("CKS wire body is serializable");
    chain_digest(previous_digest, &bytes)
}

fn append_record(
    output: &mut String,
    sequence: &mut u64,
    previous_digest: &mut Digest,
    body: WireBody,
) {
    let digest = digest_for(*sequence, *previous_digest, &body);
    let envelope = WireEnvelope {
        record: RECORD_TAG.to_owned(),
        sequence: *sequence,
        previous_digest: *previous_digest,
        body,
        digest,
    };
    let line = serde_json::to_string(&envelope).expect("CKS envelope is serializable");
    output.push_str(&line);
    output.push('\n');
    *sequence = sequence.saturating_add(1);
    *previous_digest = digest;
}

fn parse_record(line: &str) -> Result<WireEnvelope, SyncError> {
    let envelope: WireEnvelope = serde_json::from_str(line).map_err(|_| SyncError::MalformedJson)?;
    if envelope.record != RECORD_TAG {
        return Err(SyncError::UnsupportedProtocol(0));
    }
    let canonical = serde_json::to_string(&envelope).map_err(|_| SyncError::MalformedJson)?;
    if canonical != line {
        return Err(SyncError::NonCanonicalRecord);
    }
    Ok(envelope)
}

fn node_kind_tag(kind: NodeKind) -> u8 {
    match kind {
        NodeKind::Service => 1,
        NodeKind::Operation => 2,
        NodeKind::StateField => 3,
        NodeKind::Resource => 4,
        NodeKind::Constraint => 5,
        NodeKind::Transaction => 6,
    }
}

fn node_kind_from_tag(tag: u8) -> Option<NodeKind> {
    Some(match tag {
        1 => NodeKind::Service,
        2 => NodeKind::Operation,
        3 => NodeKind::StateField,
        4 => NodeKind::Resource,
        5 => NodeKind::Constraint,
        6 => NodeKind::Transaction,
        _ => return None,
    })
}

fn edge_kind_tag(kind: EdgeKind) -> u8 {
    match kind {
        EdgeKind::Calls => 1,
        EdgeKind::Reads => 2,
        EdgeKind::Writes => 3,
        EdgeKind::Requires => 4,
        EdgeKind::Emits => 5,
        EdgeKind::DependsOn => 6,
        EdgeKind::InterventionProduced => 7,
    }
}

fn edge_kind_from_tag(tag: u8) -> Option<EdgeKind> {
    Some(match tag {
        1 => EdgeKind::Calls,
        2 => EdgeKind::Reads,
        3 => EdgeKind::Writes,
        4 => EdgeKind::Requires,
        5 => EdgeKind::Emits,
        6 => EdgeKind::DependsOn,
        7 => EdgeKind::InterventionProduced,
        _ => return None,
    })
}

fn graph_to_wire(graph: &CanonicalCausalGraph, limits: SyncLimits) -> Result<WireGraph, SyncError> {
    if graph.nodes().len() > limits.max_nodes {
        return Err(SyncError::InvalidGraph);
    }
    if graph.edges().len() > limits.max_edges {
        return Err(SyncError::InvalidGraph);
    }
    Ok(WireGraph {
        nodes: graph
            .nodes()
            .iter()
            .map(|node| WireNode {
                id: node.id,
                kind: node_kind_tag(node.kind),
                type_tag: node.type_tag,
            })
            .collect(),
        edges: graph
            .edges()
            .iter()
            .map(|edge| WireEdge {
                from: edge.from,
                to: edge.to,
                kind: edge_kind_tag(edge.kind),
            })
            .collect(),
    })
}

fn graph_from_wire(wire: WireGraph, limits: SyncLimits) -> Result<CanonicalCausalGraph, SyncError> {
    if wire.nodes.len() > limits.max_nodes {
        return Err(SyncError::InvalidGraph);
    }
    if wire.edges.len() > limits.max_edges {
        return Err(SyncError::InvalidGraph);
    }
    let nodes = wire
        .nodes
        .into_iter()
        .map(|node| {
            Some(CausalNode {
                id: node.id,
                kind: node_kind_from_tag(node.kind)?,
                type_tag: node.type_tag,
            })
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(SyncError::InvalidGraph)?;
    let edges = wire
        .edges
        .into_iter()
        .map(|edge| {
            Some(CausalEdge {
                from: edge.from,
                to: edge.to,
                kind: edge_kind_from_tag(edge.kind)?,
            })
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(SyncError::InvalidGraph)?;
    CanonicalCausalGraph::new(nodes, edges).map_err(|_| SyncError::InvalidGraph)
}

fn state_to_wire(state: &State, limits: SyncLimits) -> Result<Vec<(TokenId, bool)>, SyncError> {
    let values: Vec<(TokenId, bool)> = state
        .literals()
        .map(|literal| (literal.role, literal.value))
        .collect();
    if values.len() > limits.max_state_literals {
        return Err(SyncError::EvidenceBudgetExceeded);
    }
    Ok(values)
}

fn state_from_wire(
    values: &[(TokenId, bool)],
    limits: SyncLimits,
) -> Result<State, SyncError> {
    if values.len() > limits.max_state_literals {
        return Err(SyncError::EvidenceBudgetExceeded);
    }
    let mut previous = None;
    for (role, _) in values {
        if previous.is_some_and(|last| *role <= last) {
            return Err(SyncError::InvalidState);
        }
        if *role > u64::from(u32::MAX) {
            return Err(SyncError::InvalidState);
        }
        previous = Some(*role);
    }
    Ok(State::new(
        values
            .iter()
            .map(|(role, value)| Literal::new(*role, *value)),
    ))
}

fn validate_evidence(
    graph: &CanonicalCausalGraph,
    action: ActionId,
    evidence: &[TransitionExample],
    limits: SyncLimits,
) -> Result<(), SyncError> {
    if evidence.is_empty() {
        return Err(SyncError::InvalidInput("evidence is empty"));
    }
    if evidence.len() > limits.max_evidence {
        return Err(SyncError::EvidenceBudgetExceeded);
    }
    let graph_ids: BTreeSet<u32> = graph.nodes().iter().map(|node| node.id).collect();
    let mut identities = BTreeSet::new();
    let mut expected_roles = None;
    for example in evidence {
        if example.action != action {
            return Err(SyncError::InvalidInput("evidence mixes action identities"));
        }
        let before = state_to_wire(&example.before, limits)?;
        let after = state_to_wire(&example.after, limits)?;
        if before.iter().map(|item| item.0).collect::<Vec<_>>()
            != after.iter().map(|item| item.0).collect::<Vec<_>>()
        {
            return Err(SyncError::InvalidState);
        }
        let roles: Vec<TokenId> = before.iter().map(|item| item.0).collect();
        if let Some(expected) = &expected_roles {
            if expected != &roles {
                return Err(SyncError::InvalidState);
            }
        } else {
            expected_roles = Some(roles);
        }
        if before
            .iter()
            .chain(after.iter())
            .any(|(role, _)| *role > u64::from(u32::MAX) || !graph_ids.contains(&(*role as u32)))
        {
            return Err(SyncError::InvalidState);
        }
        if !identities.insert((example.episode, example.action, before, after)) {
            return Err(SyncError::DuplicateEvidence);
        }
    }
    Ok(())
}

fn trace_kind_tag(kind: TraceKind) -> u8 {
    match kind {
        TraceKind::Observe => 1,
        TraceKind::Abstract => 2,
        TraceKind::Transfer => 3,
        TraceKind::Falsify => 4,
        TraceKind::Revise => 5,
        TraceKind::PermitRevoked => 6,
    }
}

fn trace_kind_from_tag(tag: u8) -> Option<TraceKind> {
    Some(match tag {
        1 => TraceKind::Observe,
        2 => TraceKind::Abstract,
        3 => TraceKind::Transfer,
        4 => TraceKind::Falsify,
        5 => TraceKind::Revise,
        6 => TraceKind::PermitRevoked,
        _ => return None,
    })
}

impl KnowledgeSyncCodec for JsonlKnowledgeSync {
    fn export_jsonl(
        &self,
        origin_key: u64,
        origin_sequence: u64,
        module_id: u64,
        graph: &CanonicalCausalGraph,
        action: ActionId,
        evidence: &[TransitionExample],
        ledger: &EvidenceLedger,
        limits: SyncLimits,
    ) -> Result<String, SyncError> {
        validate_evidence(graph, action, evidence, limits)?;
        if ledger.events().is_empty() || !ledger.verify() {
            return Err(SyncError::LedgerInvalid);
        }
        if ledger.events().len() > limits.max_trace_events {
            return Err(SyncError::TraceBudgetExceeded);
        }
        let graph = graph_to_wire(graph, limits)?;
        let evidence_count =
            u64::try_from(evidence.len()).map_err(|_| SyncError::EvidenceBudgetExceeded)?;
        let trace_count =
            u64::try_from(ledger.events().len()).map_err(|_| SyncError::TraceBudgetExceeded)?;
        let source_graph_digest = graph_digest(&graph)?;
        let mut output = String::new();
        let mut sequence = 0;
        let mut previous_digest = 0;
        append_record(
            &mut output,
            &mut sequence,
            &mut previous_digest,
            WireBody::Manifest {
                protocol_version: 1,
                origin_key,
                origin_sequence,
                module_id,
                source_graph_digest,
                action,
                evidence_count,
                trace_count,
                graph,
            },
        );
        for example in evidence {
            append_record(
                &mut output,
                &mut sequence,
                &mut previous_digest,
                WireBody::Evidence {
                    episode: example.episode,
                    action: example.action,
                    before: state_to_wire(&example.before, limits)?,
                    after: state_to_wire(&example.after, limits)?,
                },
            );
        }
        for event in ledger.events() {
            append_record(
                &mut output,
                &mut sequence,
                &mut previous_digest,
                WireBody::Trace {
                    sequence: event.sequence,
                    epoch: event.epoch,
                    trace_kind: trace_kind_tag(event.kind),
                    payload_digest: event.payload_digest,
                    previous_hash: event.previous_hash,
                    record_hash: event.record_hash,
                },
            );
        }
        let chain_tip = previous_digest;
        append_record(
            &mut output,
            &mut sequence,
            &mut previous_digest,
            WireBody::Commit {
                evidence_count,
                trace_count,
                chain_tip,
            },
        );
        if sequence as usize > limits.max_records {
            return Err(SyncError::RecordBudgetExceeded);
        }
        if output.len() > limits.max_bundle_bytes {
            return Err(SyncError::BundleTooLarge);
        }
        Ok(output)
    }

    fn import_jsonl(
        &self,
        jsonl: &str,
        target_graph: &CanonicalCausalGraph,
        target_action: ActionId,
        mapping_budget: MappingBudget,
        limits: SyncLimits,
    ) -> Result<SyncedKnowledgeCandidate, SyncError> {
        if jsonl.is_empty() || jsonl.len() > limits.max_bundle_bytes {
            return Err(SyncError::BundleTooLarge);
        }
        if target_graph.nodes().len() > limits.max_nodes
            || target_graph.edges().len() > limits.max_edges
        {
            return Err(SyncError::InvalidGraph);
        }
        let mut expected_sequence = 0;
        let mut previous_digest = 0;
        let mut manifest = None;
        let mut evidence = Vec::new();
        let mut traces = Vec::new();
        let mut commit = None;
        let mut saw_trace = false;
        let mut saw_commit = false;
        for line in jsonl.lines() {
            if line.is_empty() {
                return Err(SyncError::MalformedJson);
            }
            if line.len() > limits.max_record_bytes {
                return Err(SyncError::RecordTooLarge);
            }
            if expected_sequence as usize >= limits.max_records {
                return Err(SyncError::RecordBudgetExceeded);
            }
            let envelope = parse_record(line)?;
            if envelope.sequence != expected_sequence
                || envelope.previous_digest != previous_digest
                || envelope.digest
                    != digest_for(envelope.sequence, envelope.previous_digest, &envelope.body)
            {
                return Err(if envelope.sequence != expected_sequence {
                    SyncError::InvalidSequence
                } else {
                    SyncError::IntegrityMismatch
                });
            }
            match envelope.body {
                body @ WireBody::Manifest { .. } => {
                    if expected_sequence != 0 || manifest.is_some() || saw_commit {
                        return Err(SyncError::InvalidSequence);
                    }
                    manifest = Some(body);
                }
                body @ WireBody::Evidence { .. } => {
                    if manifest.is_none() || saw_trace || saw_commit {
                        return Err(SyncError::InvalidSequence);
                    }
                    evidence.push(body);
                    if evidence.len() > limits.max_evidence {
                        return Err(SyncError::EvidenceBudgetExceeded);
                    }
                }
                body @ WireBody::Trace { .. } => {
                    if manifest.is_none() || saw_commit {
                        return Err(SyncError::InvalidSequence);
                    }
                    saw_trace = true;
                    traces.push(body);
                    if traces.len() > limits.max_trace_events {
                        return Err(SyncError::TraceBudgetExceeded);
                    }
                }
                body @ WireBody::Commit { .. } => {
                    if manifest.is_none() || commit.is_some() {
                        return Err(SyncError::InvalidSequence);
                    }
                    saw_commit = true;
                    commit = Some((body, envelope.previous_digest));
                }
            }
            previous_digest = envelope.digest;
            expected_sequence = expected_sequence.saturating_add(1);
        }
        let Some(WireBody::Manifest {
            protocol_version,
            origin_key,
            origin_sequence,
            module_id,
            source_graph_digest,
            action,
            evidence_count,
            trace_count,
            graph,
        }) = manifest
        else {
            return Err(SyncError::MalformedJson);
        };
        if protocol_version != 1 {
            return Err(SyncError::UnsupportedProtocol(protocol_version));
        }
        let Some((WireBody::Commit {
            evidence_count: committed_evidence,
            trace_count: committed_trace,
            chain_tip,
        }, commit_previous)) = commit
        else {
            return Err(SyncError::MissingCommit);
        };
        if chain_tip != commit_previous
            || evidence_count != u64::try_from(evidence.len()).unwrap_or(u64::MAX)
            || trace_count != u64::try_from(traces.len()).unwrap_or(u64::MAX)
            || committed_evidence != evidence_count
            || committed_trace != trace_count
        {
            return Err(SyncError::IntegrityMismatch);
        }
        let source_graph = graph_from_wire(graph, limits)?;
        if source_graph.digest() != source_graph_digest {
            return Err(SyncError::IntegrityMismatch);
        }
        let mut examples = Vec::with_capacity(evidence.len());
        for body in &evidence {
            let WireBody::Evidence {
                episode,
                action: evidence_action,
                before,
                after,
            } = body
            else {
                return Err(SyncError::InvalidSequence);
            };
            examples.push(TransitionExample::new(
                *episode,
                *evidence_action,
                state_from_wire(before, limits)?,
                state_from_wire(after, limits)?,
            ));
        }
        validate_evidence(&source_graph, action, &examples, limits)?;
        let mut replayed_ledger = EvidenceLedger::new();
        for body in &traces {
            let WireBody::Trace {
                sequence,
                epoch,
                trace_kind,
                payload_digest,
                previous_hash,
                record_hash,
            } = body
            else {
                return Err(SyncError::InvalidSequence);
            };
            let decoded_kind =
                trace_kind_from_tag(*trace_kind).ok_or(SyncError::LedgerMismatch)?;
            let event = replayed_ledger.append(*epoch, decoded_kind, *payload_digest);
            if event.sequence != *sequence
                || event.previous_hash != *previous_hash
                || event.record_hash != *record_hash
            {
                return Err(SyncError::LedgerMismatch);
            }
        }
        if !replayed_ledger.verify() {
            return Err(SyncError::LedgerMismatch);
        }
        let mut learner = CausalLearner::new();
        learner.fit(&examples).map_err(|_| SyncError::ModelRejected)?;
        let source_rule = learner.rule().cloned().ok_or(SyncError::ModelRejected)?;
        let mapping = match source_graph.find_unique_mapping(target_graph, mapping_budget) {
            super::cross_domain_causal_core::MappingVerdict::Unique(mapping) => mapping,
            super::cross_domain_causal_core::MappingVerdict::Ambiguous { .. } => {
                return Err(SyncError::AmbiguousMapping)
            }
            super::cross_domain_causal_core::MappingVerdict::ResourceLimit { .. } => {
                return Err(SyncError::MappingResourceLimit)
            }
            super::cross_domain_causal_core::MappingVerdict::NoMatch => {
                return Err(SyncError::IncompatibleGraph)
            }
        };
        let rule = source_rule
            .remap(&mapping, target_action)
            .ok_or(SyncError::IncompatibleGraph)?;
        let source_roles: Vec<TokenId> = examples[0]
            .before
            .literals()
            .map(|literal| literal.role)
            .collect();
        let target_roles = source_roles
            .into_iter()
            .map(|role| {
                let node = u32::try_from(role).map_err(|_| SyncError::InvalidState)?;
                mapping
                    .map_node(node)
                    .map(u64::from)
                    .ok_or(SyncError::IncompatibleGraph)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(SyncedKnowledgeCandidate {
            origin_key,
            origin_sequence,
            module_id,
            source_graph_digest,
            target_graph_digest: target_graph.digest(),
            target_action,
            evidence_count: examples.len(),
            bundle_digest: previous_digest,
            target_roles,
            rule,
        })
    }
}

fn graph_digest(graph: &WireGraph) -> Result<Digest, SyncError> {
    let nodes = graph
        .nodes
        .iter()
        .map(|node| -> Result<CausalNode, SyncError> {
            Ok(CausalNode {
                id: node.id,
                kind: node_kind_from_tag(node.kind).ok_or(SyncError::InvalidGraph)?,
                type_tag: node.type_tag,
            })
        })
        .collect::<Result<Vec<_>, SyncError>>()?;
    let edges = graph
        .edges
        .iter()
        .map(|edge| -> Result<CausalEdge, SyncError> {
            Ok(CausalEdge {
                from: edge.from,
                to: edge.to,
                kind: edge_kind_from_tag(edge.kind).ok_or(SyncError::InvalidGraph)?,
            })
        })
        .collect::<Result<Vec<_>, SyncError>>()?;
    Ok(CanonicalCausalGraph::new(nodes, edges)
        .map_err(|_| SyncError::InvalidGraph)?
        .digest())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hierarchical_causal_graph::{CausalHierarchy, HierarchyError};

    fn graph(offset: u32) -> CanonicalCausalGraph {
        CanonicalCausalGraph::new(
            vec![
                CausalNode {
                    id: offset,
                    kind: NodeKind::Operation,
                    type_tag: 7,
                },
                CausalNode {
                    id: offset + 1,
                    kind: NodeKind::StateField,
                    type_tag: 1,
                },
                CausalNode {
                    id: offset + 2,
                    kind: NodeKind::StateField,
                    type_tag: 2,
                },
            ],
            vec![
                CausalEdge {
                    from: offset,
                    to: offset + 1,
                    kind: EdgeKind::Requires,
                },
                CausalEdge {
                    from: offset,
                    to: offset + 2,
                    kind: EdgeKind::Writes,
                },
            ],
        )
        .expect("graph")
    }

    fn source_examples() -> Vec<TransitionExample> {
        let before = State::new([
            Literal::new(10, true),
            Literal::new(11, true),
            Literal::new(12, false),
        ]);
        let after = before.with(Literal::new(12, true));
        vec![
            TransitionExample::new(1, 99, before.clone(), after.clone()),
            TransitionExample::new(2, 99, before, after),
        ]
    }

    fn ledger() -> EvidenceLedger {
        let mut ledger = EvidenceLedger::new();
        ledger.append(0, TraceKind::Observe, 11);
        ledger.append(0, TraceKind::Abstract, 22);
        ledger
    }

    #[test]
    fn jsonl_roundtrip_maps_and_predicts_without_target_training() {
        let codec = JsonlKnowledgeSync;
        let source = graph(10);
        let target = graph(20);
        let evidence = source_examples();
        let jsonl = codec
            .export_jsonl(
                7,
                3,
                42,
                &source,
                99,
                &evidence,
                &ledger(),
                SyncLimits::default(),
            )
            .expect("export");
        let candidate = codec
            .import_jsonl(
                &jsonl,
                &target,
                199,
                MappingBudget::new(128).expect("budget"),
                SyncLimits::default(),
            )
            .expect("import");
        let before = State::new([
            Literal::new(20, true),
            Literal::new(21, true),
            Literal::new(22, false),
        ]);
        let expected = before.with(Literal::new(22, true));
        assert_eq!(candidate.predict(&before, 199), Ok(expected));
        assert_eq!(candidate.evidence_count(), 2);
        assert_eq!(
            codec.export_jsonl(
                7,
                3,
                42,
                &source,
                99,
                &evidence,
                &ledger(),
                SyncLimits::default()
            ),
            Ok(jsonl)
        );
    }

    #[test]
    fn tampering_or_non_canonical_records_fail_closed() {
        let codec = JsonlKnowledgeSync;
        let source = graph(10);
        let target = graph(20);
        let evidence = source_examples();
        let jsonl = codec
            .export_jsonl(
                1,
                1,
                1,
                &source,
                99,
                &evidence,
                &ledger(),
                SyncLimits::default(),
            )
            .expect("export");
        let tampered = jsonl.replacen("\"action\":99", "\"action\":98", 1);
        assert!(matches!(
            codec.import_jsonl(
                &tampered,
                &target,
                199,
                MappingBudget::new(128).expect("budget"),
                SyncLimits::default(),
            ),
            Err(SyncError::IntegrityMismatch)
        ));
        let non_canonical = jsonl.replacen("{\"record\"", "{ \"record\"", 1);
        assert!(matches!(
            codec.import_jsonl(
                &non_canonical,
                &target,
                199,
                MappingBudget::new(128).expect("budget"),
                SyncLimits::default(),
            ),
            Err(SyncError::NonCanonicalRecord)
        ));
    }

    #[test]
    fn duplicate_evidence_and_budget_overflow_are_rejected_before_mapping() {
        let codec = JsonlKnowledgeSync;
        let source = graph(10);
        let mut evidence = source_examples();
        evidence[1] = evidence[0].clone();
        assert_eq!(
            codec.export_jsonl(
                1,
                1,
                1,
                &source,
                99,
                &evidence,
                &ledger(),
                SyncLimits::default(),
            ),
            Err(SyncError::DuplicateEvidence)
        );
        let evidence = source_examples();
        let limits = SyncLimits {
            max_bundle_bytes: 32,
            ..SyncLimits::default()
        };
        assert_eq!(
            codec.export_jsonl(1, 1, 1, &source, 99, &evidence, &ledger(), limits),
            Err(SyncError::BundleTooLarge)
        );
    }

    #[test]
    fn synced_candidate_enters_hierarchy_without_target_training() {
        let codec = JsonlKnowledgeSync;
        let source = graph(10);
        let target = graph(20);
        let jsonl = codec
            .export_jsonl(
                7,
                3,
                42,
                &source,
                99,
                &source_examples(),
                &ledger(),
                SyncLimits::default(),
            )
            .expect("export");
        let candidate = codec
            .import_jsonl(
                &jsonl,
                &target,
                199,
                MappingBudget::new(128).expect("budget"),
                SyncLimits::default(),
            )
            .expect("import");

        let mut hierarchy = CausalHierarchy::default();
        hierarchy
            .add_synced_atomic(7, &candidate, &target)
            .expect("bind synced module");
        let plan = hierarchy.compose(100, vec![7]).expect("compose");
        let before = State::new([
            Literal::new(20, true),
            Literal::new(21, true),
            Literal::new(22, false),
        ]);
        assert_eq!(
            plan.predict(&hierarchy, &before).expect("predict").get(22),
            Some(true)
        );
        assert_eq!(
            hierarchy.add_synced_atomic(8, &candidate, &source),
            Err(HierarchyError::SyncedTargetGraphMismatch {
                expected: candidate.target_graph_digest(),
                observed: source.digest(),
            })
        );
    }
}
