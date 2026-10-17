//! Authenticated execution-receipt transfer layered over CKS/1.
//!
//! P7A closes one physical causal cycle with a terminal receipt. This module
//! admits only confirmed receipts, binds their exact symbolic before/after
//! states and dispatch authority into a canonical signed envelope, and then
//! delegates structural transfer to the existing CKS/1 codec.
//!
//! Cryptography is deliberately host-supplied through signer/verifier traits.
//! The kernel does not mislabel its deterministic `u64` integrity digests as
//! signatures. Production hosts can bind these traits to Ed25519, a TPM, an
//! HSM, or another audited trust root without changing causal semantics.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::causal_knowledge_sync::{
    transition_evidence_digest, JsonlKnowledgeSync, KnowledgeSyncCodec, SyncError, SyncLimits,
    SyncedKnowledgeCandidate,
};
use super::causal_proof::CommitmentResult;
use super::cross_domain_causal_core::{
    ActionId, CanonicalCausalGraph, Digest, EvidenceLedger, Literal, MappingBudget, State, TokenId,
    TraceKind, TransitionExample,
};
use super::end_to_end_causal_runtime::{
    composite_dispatch_digest, OutcomeEvidence, VerifiedExecutionReceipt,
};
use super::hierarchical_causal_graph::{CausalHierarchy, HierarchyError, ModuleId};

const RECORD_TAG: &str = "athlesia/cks-execution/1";
const PROTOCOL_VERSION: u16 = 1;
const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const RECEIPT_DOMAIN: u64 = 0x4154_4c45_5349_5250;
const RECEIPT_CHAIN_DOMAIN: u64 = 0x4154_4c45_5349_5243;
const CKS_TEXT_DOMAIN: u64 = 0x4154_4c45_5349_4354;
const SIGNED_PAYLOAD_DOMAIN: u64 = 0x4154_4c45_5349_5347;

fn mix(mut state: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state.max(1)
}

fn bytes_digest(domain: u64, bytes: &[u8]) -> Digest {
    let mut digest = mix(FNV_OFFSET, domain);
    digest = mix(digest, bytes.len() as u64);
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(FNV_PRIME);
    }
    digest.max(1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionSyncLimits {
    pub max_envelope_bytes: usize,
    pub max_receipts: usize,
    pub max_signature_bytes: usize,
}

impl ExecutionSyncLimits {
    fn valid(self) -> bool {
        self.max_envelope_bytes > 0 && self.max_receipts > 0 && self.max_signature_bytes > 0
    }
}

impl Default for ExecutionSyncLimits {
    fn default() -> Self {
        Self {
            max_envelope_bytes: 5 * 1024 * 1024,
            max_receipts: 512,
            max_signature_bytes: 512,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionSyncError {
    InvalidLimits,
    InvalidInput(&'static str),
    EnvelopeTooLarge,
    MalformedJson,
    NonCanonicalEnvelope,
    UnsupportedProtocol(u16),
    ReceiptBudgetExceeded,
    SignatureBudgetExceeded,
    SigningFailed,
    AuthenticationFailed,
    UnconfirmedReceipt,
    MissingObservedState,
    ReceiptIntegrityMismatch,
    DuplicateReceipt,
    MixedCompositePlan,
    InvalidState,
    NestedBundleMismatch,
    ReplayedOriginSequence {
        origin_key: u64,
        previous: u64,
        requested: u64,
    },
    OriginBudgetExceeded {
        observed: usize,
        limit: usize,
    },
    Cks(SyncError),
}

impl std::fmt::Display for ExecutionSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ExecutionSyncError {}

impl From<SyncError> for ExecutionSyncError {
    fn from(error: SyncError) -> Self {
        Self::Cks(error)
    }
}

/// Host-owned signing authority. Algorithm identifiers are deployment policy;
/// zero is reserved and rejected by the kernel.
pub trait ExecutionKnowledgeSigner {
    fn algorithm_id(&self) -> u16;
    fn key_id(&self) -> u64;
    fn sign(&self, message: &[u8]) -> Option<Vec<u8>>;
}

/// Host-owned trust store and signature verifier. Returning `false` is a hard
/// fail-closed rejection and cannot yield a synced candidate.
pub trait ExecutionKnowledgeVerifier {
    fn verify(&self, algorithm_id: u16, key_id: u64, message: &[u8], signature: &[u8]) -> bool;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireReceipt {
    scope_schema_digest: Digest,
    scope_abstraction_digest: Digest,
    context: Option<u64>,
    epoch: u64,
    episode: u64,
    sequence: u64,
    command: u64,
    proof_action: ActionId,
    plan_digest: Digest,
    commitment_digest: Digest,
    before_digest: Digest,
    expected_after_digest: Digest,
    atomic_actions: Vec<ActionId>,
    host_policy_digest: Digest,
    authorization_nonce: u64,
    dispatch_digest: Digest,
    outcome_schema_digest: Digest,
    outcome_abstraction_digest: Digest,
    after_digest: Digest,
    before: Vec<(TokenId, bool)>,
    after: Vec<(TokenId, bool)>,
    receipt_digest: Digest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireUnsignedBundle {
    record: String,
    protocol_version: u16,
    signature_algorithm: u16,
    origin_key: u64,
    origin_sequence: u64,
    module_id: u64,
    source_graph_digest: Digest,
    proof_action: ActionId,
    plan_digest: Digest,
    receipt_count: u64,
    receipt_chain_tip: Digest,
    evidence_digest: Digest,
    cks_text_digest: Digest,
    receipts: Vec<WireReceipt>,
    cks_jsonl: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSignedBundle {
    unsigned: WireUnsignedBundle,
    payload_digest: Digest,
    signature: Vec<u8>,
}

fn state_to_wire(
    state: &State,
    max_literals: usize,
) -> Result<Vec<(TokenId, bool)>, ExecutionSyncError> {
    let values: Vec<(TokenId, bool)> = state
        .literals()
        .map(|literal| (literal.role, literal.value))
        .collect();
    if values.is_empty() || values.len() > max_literals {
        return Err(ExecutionSyncError::InvalidState);
    }
    Ok(values)
}

fn state_from_wire(
    values: &[(TokenId, bool)],
    max_literals: usize,
) -> Result<State, ExecutionSyncError> {
    if values.is_empty() || values.len() > max_literals {
        return Err(ExecutionSyncError::InvalidState);
    }
    let mut previous = None;
    for (role, _) in values {
        if previous.is_some_and(|last| *role <= last) || *role > u64::from(u32::MAX) {
            return Err(ExecutionSyncError::InvalidState);
        }
        previous = Some(*role);
    }
    Ok(State::new(
        values
            .iter()
            .map(|(role, value)| Literal::new(*role, *value)),
    ))
}

fn mix_state(mut digest: Digest, values: &[(TokenId, bool)]) -> Digest {
    digest = mix(digest, values.len() as u64);
    for (role, value) in values {
        digest = mix(digest, *role);
        digest = mix(digest, u64::from(*value));
    }
    digest
}

fn wire_receipt_digest(receipt: &WireReceipt) -> Digest {
    let mut digest = mix(FNV_OFFSET, RECEIPT_DOMAIN);
    digest = mix(digest, receipt.scope_schema_digest);
    digest = mix(digest, receipt.scope_abstraction_digest);
    match receipt.context {
        Some(context) => {
            digest = mix(digest, 1);
            digest = mix(digest, context);
        }
        None => digest = mix(digest, 0),
    }
    digest = mix(digest, receipt.epoch);
    digest = mix(digest, receipt.episode);
    digest = mix(digest, receipt.sequence);
    digest = mix(digest, receipt.command);
    digest = mix(digest, receipt.proof_action);
    digest = mix(digest, receipt.plan_digest);
    digest = mix(digest, receipt.commitment_digest);
    digest = mix(digest, receipt.before_digest);
    digest = mix(digest, receipt.expected_after_digest);
    digest = mix(digest, receipt.atomic_actions.len() as u64);
    for action in &receipt.atomic_actions {
        digest = mix(digest, *action);
    }
    digest = mix(digest, receipt.host_policy_digest);
    digest = mix(digest, receipt.authorization_nonce);
    digest = mix(digest, receipt.dispatch_digest);
    digest = mix(digest, receipt.outcome_schema_digest);
    digest = mix(digest, receipt.outcome_abstraction_digest);
    digest = mix(digest, receipt.after_digest);
    digest = mix_state(digest, &receipt.before);
    mix_state(digest, &receipt.after)
}

fn receipt_order_key(receipt: &WireReceipt) -> (u64, u64, u64, bool, u64) {
    (
        receipt.epoch,
        receipt.episode,
        receipt.sequence,
        receipt.context.is_some(),
        receipt.context.unwrap_or(0),
    )
}

fn receipt_from_runtime(
    receipt: &VerifiedExecutionReceipt,
    cks_limits: SyncLimits,
) -> Result<WireReceipt, ExecutionSyncError> {
    if receipt.result() != CommitmentResult::Confirmed {
        return Err(ExecutionSyncError::UnconfirmedReceipt);
    }
    let (outcome_schema_digest, outcome_abstraction_digest, outcome_after_digest) =
        match receipt.evidence() {
            OutcomeEvidence::Observed {
                schema_digest,
                abstraction_digest,
                after_digest,
            } => (*schema_digest, *abstraction_digest, *after_digest),
            _ => return Err(ExecutionSyncError::MissingObservedState),
        };
    let after = receipt
        .observed_after()
        .ok_or(ExecutionSyncError::MissingObservedState)?;
    let prepared = receipt.prepared();
    let scope = prepared.scope();
    if prepared.before_digest() != receipt.before().digest()
        || prepared.expected_after_digest() != after.digest()
        || outcome_after_digest != after.digest()
        || outcome_schema_digest != scope.schema_digest
        || scope.schema_digest == 0
        || scope.abstraction_digest == 0
        || outcome_abstraction_digest == 0
        || prepared.atomic_action_count() != receipt.atomic_actions().len()
        || receipt.atomic_actions().is_empty()
        || receipt.host_policy_digest() == 0
        || receipt.authorization_nonce() == 0
        || receipt.dispatch_digest()
            != composite_dispatch_digest(
                prepared.commitment_digest(),
                receipt.atomic_actions(),
                receipt.host_policy_digest(),
                receipt.authorization_nonce(),
            )
    {
        return Err(ExecutionSyncError::ReceiptIntegrityMismatch);
    }
    let before = state_to_wire(receipt.before(), cks_limits.max_state_literals)?;
    let after = state_to_wire(after, cks_limits.max_state_literals)?;
    if before.iter().map(|item| item.0).collect::<Vec<_>>()
        != after.iter().map(|item| item.0).collect::<Vec<_>>()
    {
        return Err(ExecutionSyncError::InvalidState);
    }
    let mut wire = WireReceipt {
        scope_schema_digest: scope.schema_digest,
        scope_abstraction_digest: scope.abstraction_digest,
        context: scope.context.map(|context| context.0),
        epoch: scope.epoch,
        episode: scope.episode,
        sequence: scope.sequence,
        command: prepared.command(),
        proof_action: prepared.proof_action(),
        plan_digest: prepared.plan_digest(),
        commitment_digest: prepared.commitment_digest(),
        before_digest: prepared.before_digest(),
        expected_after_digest: prepared.expected_after_digest(),
        atomic_actions: receipt.atomic_actions().to_vec(),
        host_policy_digest: receipt.host_policy_digest(),
        authorization_nonce: receipt.authorization_nonce(),
        dispatch_digest: receipt.dispatch_digest(),
        outcome_schema_digest,
        outcome_abstraction_digest,
        after_digest: outcome_after_digest,
        before,
        after,
        receipt_digest: 0,
    };
    wire.receipt_digest = wire_receipt_digest(&wire);
    Ok(wire)
}

fn validate_wire_receipt(
    receipt: &WireReceipt,
    cks_limits: SyncLimits,
) -> Result<(State, State), ExecutionSyncError> {
    let before = state_from_wire(&receipt.before, cks_limits.max_state_literals)?;
    let after = state_from_wire(&receipt.after, cks_limits.max_state_literals)?;
    if receipt.before.iter().map(|item| item.0).collect::<Vec<_>>()
        != receipt.after.iter().map(|item| item.0).collect::<Vec<_>>()
        || receipt.command == 0
        || receipt.proof_action == 0
        || receipt.plan_digest == 0
        || receipt.commitment_digest == 0
        || receipt.atomic_actions.is_empty()
        || receipt.host_policy_digest == 0
        || receipt.authorization_nonce == 0
        || receipt.scope_schema_digest == 0
        || receipt.scope_abstraction_digest == 0
        || receipt.outcome_schema_digest != receipt.scope_schema_digest
        || receipt.outcome_abstraction_digest == 0
        || receipt.before_digest != before.digest()
        || receipt.expected_after_digest != after.digest()
        || receipt.after_digest != after.digest()
        || receipt.dispatch_digest
            != composite_dispatch_digest(
                receipt.commitment_digest,
                &receipt.atomic_actions,
                receipt.host_policy_digest,
                receipt.authorization_nonce,
            )
        || receipt.receipt_digest != wire_receipt_digest(receipt)
    {
        return Err(ExecutionSyncError::ReceiptIntegrityMismatch);
    }
    Ok((before, after))
}

fn validate_receipt_batch(
    receipts: &[WireReceipt],
    limits: ExecutionSyncLimits,
    cks_limits: SyncLimits,
) -> Result<(Vec<TransitionExample>, Digest, ActionId, Digest), ExecutionSyncError> {
    if receipts.is_empty() {
        return Err(ExecutionSyncError::InvalidInput("receipt batch is empty"));
    }
    if receipts.len() > limits.max_receipts {
        return Err(ExecutionSyncError::ReceiptBudgetExceeded);
    }
    let first = &receipts[0];
    let mut previous_order = None;
    let mut identities = BTreeSet::new();
    let mut evidence = Vec::with_capacity(receipts.len());
    let mut chain_tip = mix(FNV_OFFSET, RECEIPT_CHAIN_DOMAIN);
    for receipt in receipts {
        let order = receipt_order_key(receipt);
        if previous_order.is_some_and(|previous| order <= previous) || !identities.insert(order) {
            return Err(ExecutionSyncError::DuplicateReceipt);
        }
        previous_order = Some(order);
        if receipt.proof_action != first.proof_action
            || receipt.plan_digest != first.plan_digest
            || receipt.atomic_actions != first.atomic_actions
        {
            return Err(ExecutionSyncError::MixedCompositePlan);
        }
        let (before, after) = validate_wire_receipt(receipt, cks_limits)?;
        evidence.push(TransitionExample::new(
            receipt.receipt_digest,
            receipt.proof_action,
            before,
            after,
        ));
        chain_tip = mix(chain_tip, receipt.receipt_digest);
    }
    Ok((evidence, chain_tip, first.proof_action, first.plan_digest))
}

/// Stateless exporter. Import is intentionally stateful through
/// `ExecutionKnowledgeReceiver`, which owns replay protection.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExecutionKnowledgeSync;

impl ExecutionKnowledgeSync {
    #[allow(clippy::too_many_arguments)]
    pub fn export_jsonl<S: ExecutionKnowledgeSigner>(
        &self,
        origin_sequence: u64,
        module_id: u64,
        source_graph: &CanonicalCausalGraph,
        receipts: &[VerifiedExecutionReceipt],
        signer: &S,
        cks_limits: SyncLimits,
        limits: ExecutionSyncLimits,
    ) -> Result<String, ExecutionSyncError> {
        if !limits.valid() {
            return Err(ExecutionSyncError::InvalidLimits);
        }
        if origin_sequence == 0 || module_id == 0 {
            return Err(ExecutionSyncError::InvalidInput(
                "origin sequence and module id must be non-zero",
            ));
        }
        let algorithm_id = signer.algorithm_id();
        let origin_key = signer.key_id();
        if algorithm_id == 0 || origin_key == 0 {
            return Err(ExecutionSyncError::InvalidInput(
                "signer identity must be non-zero",
            ));
        }
        if receipts.is_empty() {
            return Err(ExecutionSyncError::InvalidInput("receipt batch is empty"));
        }
        if receipts.len() > limits.max_receipts {
            return Err(ExecutionSyncError::ReceiptBudgetExceeded);
        }
        let mut wire_receipts = receipts
            .iter()
            .map(|receipt| receipt_from_runtime(receipt, cks_limits))
            .collect::<Result<Vec<_>, _>>()?;
        wire_receipts.sort_by_key(receipt_order_key);
        let (evidence, receipt_chain_tip, proof_action, plan_digest) =
            validate_receipt_batch(&wire_receipts, limits, cks_limits)?;
        let evidence_digest = transition_evidence_digest(&evidence);
        let mut ledger = EvidenceLedger::new();
        for receipt in &wire_receipts {
            ledger.append(receipt.epoch, TraceKind::Observe, receipt.receipt_digest);
        }
        let final_epoch = wire_receipts
            .last()
            .map(|receipt| receipt.epoch)
            .unwrap_or(0);
        ledger.append(final_epoch, TraceKind::Abstract, plan_digest);
        let cks_jsonl = JsonlKnowledgeSync.export_jsonl(
            origin_key,
            origin_sequence,
            module_id,
            source_graph,
            proof_action,
            &evidence,
            &ledger,
            cks_limits,
        )?;
        let unsigned = WireUnsignedBundle {
            record: RECORD_TAG.to_owned(),
            protocol_version: PROTOCOL_VERSION,
            signature_algorithm: algorithm_id,
            origin_key,
            origin_sequence,
            module_id,
            source_graph_digest: source_graph.digest(),
            proof_action,
            plan_digest,
            receipt_count: u64::try_from(wire_receipts.len())
                .map_err(|_| ExecutionSyncError::ReceiptBudgetExceeded)?,
            receipt_chain_tip,
            evidence_digest,
            cks_text_digest: bytes_digest(CKS_TEXT_DOMAIN, cks_jsonl.as_bytes()),
            receipts: wire_receipts,
            cks_jsonl,
        };
        let payload =
            serde_json::to_vec(&unsigned).map_err(|_| ExecutionSyncError::MalformedJson)?;
        let signature = signer
            .sign(&payload)
            .ok_or(ExecutionSyncError::SigningFailed)?;
        if signature.is_empty() || signature.len() > limits.max_signature_bytes {
            return Err(ExecutionSyncError::SignatureBudgetExceeded);
        }
        let envelope = WireSignedBundle {
            payload_digest: bytes_digest(SIGNED_PAYLOAD_DOMAIN, &payload),
            unsigned,
            signature,
        };
        let mut jsonl =
            serde_json::to_string(&envelope).map_err(|_| ExecutionSyncError::MalformedJson)?;
        jsonl.push('\n');
        if jsonl.len() > limits.max_envelope_bytes {
            return Err(ExecutionSyncError::EnvelopeTooLarge);
        }
        Ok(jsonl)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedSyncedKnowledge {
    candidate: SyncedKnowledgeCandidate,
    signature_algorithm: u16,
    origin_key: u64,
    origin_sequence: u64,
    module_id: u64,
    receipt_count: usize,
    receipt_chain_tip: Digest,
    plan_digest: Digest,
    payload_digest: Digest,
}

impl AuthenticatedSyncedKnowledge {
    pub fn candidate(&self) -> &SyncedKnowledgeCandidate {
        &self.candidate
    }

    pub fn signature_algorithm(&self) -> u16 {
        self.signature_algorithm
    }

    pub fn origin_key(&self) -> u64 {
        self.origin_key
    }

    pub fn origin_sequence(&self) -> u64 {
        self.origin_sequence
    }

    pub fn module_id(&self) -> u64 {
        self.module_id
    }

    pub fn receipt_count(&self) -> usize {
        self.receipt_count
    }

    pub fn receipt_chain_tip(&self) -> Digest {
        self.receipt_chain_tip
    }

    pub fn plan_digest(&self) -> Digest {
        self.plan_digest
    }

    pub fn payload_digest(&self) -> Digest {
        self.payload_digest
    }

    /// Install through the existing hierarchy adapter while preserving an
    /// explicit authenticated type boundary at the call site.
    pub fn install_atomic(
        &self,
        hierarchy: &mut CausalHierarchy,
        module_id: ModuleId,
        target_graph: &CanonicalCausalGraph,
    ) -> Result<(), HierarchyError> {
        hierarchy.add_synced_atomic(module_id, &self.candidate, target_graph)
    }
}

/// Stateful receiving boundary. An authenticated origin sequence advances
/// only after every receipt and the nested CKS bundle have been validated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionKnowledgeReceiver {
    max_origins: usize,
    last_sequences: BTreeMap<u64, u64>,
}

impl ExecutionKnowledgeReceiver {
    pub fn new(max_origins: usize) -> Result<Self, ExecutionSyncError> {
        if max_origins == 0 {
            return Err(ExecutionSyncError::InvalidLimits);
        }
        Ok(Self {
            max_origins,
            last_sequences: BTreeMap::new(),
        })
    }

    pub fn last_sequence(&self, origin_key: u64) -> Option<u64> {
        self.last_sequences.get(&origin_key).copied()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn import_jsonl<V: ExecutionKnowledgeVerifier>(
        &mut self,
        jsonl: &str,
        target_graph: &CanonicalCausalGraph,
        target_action: ActionId,
        mapping_budget: MappingBudget,
        verifier: &V,
        cks_limits: SyncLimits,
        limits: ExecutionSyncLimits,
    ) -> Result<AuthenticatedSyncedKnowledge, ExecutionSyncError> {
        if !limits.valid() {
            return Err(ExecutionSyncError::InvalidLimits);
        }
        if jsonl.is_empty() || jsonl.len() > limits.max_envelope_bytes {
            return Err(ExecutionSyncError::EnvelopeTooLarge);
        }
        if !jsonl.ends_with('\n') || jsonl[..jsonl.len() - 1].contains('\n') {
            return Err(ExecutionSyncError::MalformedJson);
        }
        let line = &jsonl[..jsonl.len() - 1];
        let envelope: WireSignedBundle =
            serde_json::from_str(line).map_err(|_| ExecutionSyncError::MalformedJson)?;
        let canonical =
            serde_json::to_string(&envelope).map_err(|_| ExecutionSyncError::MalformedJson)?;
        if canonical != line {
            return Err(ExecutionSyncError::NonCanonicalEnvelope);
        }
        let unsigned = &envelope.unsigned;
        if unsigned.record != RECORD_TAG {
            return Err(ExecutionSyncError::UnsupportedProtocol(0));
        }
        if unsigned.protocol_version != PROTOCOL_VERSION {
            return Err(ExecutionSyncError::UnsupportedProtocol(
                unsigned.protocol_version,
            ));
        }
        if unsigned.signature_algorithm == 0
            || unsigned.origin_key == 0
            || unsigned.origin_sequence == 0
            || unsigned.module_id == 0
        {
            return Err(ExecutionSyncError::InvalidInput(
                "invalid authenticated identity",
            ));
        }
        if envelope.signature.is_empty() || envelope.signature.len() > limits.max_signature_bytes {
            return Err(ExecutionSyncError::SignatureBudgetExceeded);
        }
        if unsigned.receipts.is_empty()
            || unsigned.receipts.len() > limits.max_receipts
            || unsigned.receipt_count != u64::try_from(unsigned.receipts.len()).unwrap_or(u64::MAX)
        {
            return Err(ExecutionSyncError::ReceiptBudgetExceeded);
        }
        let payload =
            serde_json::to_vec(unsigned).map_err(|_| ExecutionSyncError::MalformedJson)?;
        if envelope.payload_digest != bytes_digest(SIGNED_PAYLOAD_DOMAIN, &payload)
            || !verifier.verify(
                unsigned.signature_algorithm,
                unsigned.origin_key,
                &payload,
                &envelope.signature,
            )
        {
            return Err(ExecutionSyncError::AuthenticationFailed);
        }
        if let Some(previous) = self.last_sequence(unsigned.origin_key) {
            if unsigned.origin_sequence <= previous {
                return Err(ExecutionSyncError::ReplayedOriginSequence {
                    origin_key: unsigned.origin_key,
                    previous,
                    requested: unsigned.origin_sequence,
                });
            }
        } else if self.last_sequences.len() >= self.max_origins {
            return Err(ExecutionSyncError::OriginBudgetExceeded {
                observed: self.last_sequences.len().saturating_add(1),
                limit: self.max_origins,
            });
        }
        if unsigned.cks_text_digest != bytes_digest(CKS_TEXT_DOMAIN, unsigned.cks_jsonl.as_bytes())
        {
            return Err(ExecutionSyncError::NestedBundleMismatch);
        }
        let (evidence, receipt_chain_tip, proof_action, plan_digest) =
            validate_receipt_batch(&unsigned.receipts, limits, cks_limits)?;
        if receipt_chain_tip != unsigned.receipt_chain_tip
            || proof_action != unsigned.proof_action
            || plan_digest != unsigned.plan_digest
            || transition_evidence_digest(&evidence) != unsigned.evidence_digest
        {
            return Err(ExecutionSyncError::ReceiptIntegrityMismatch);
        }
        let candidate = JsonlKnowledgeSync.import_jsonl(
            &unsigned.cks_jsonl,
            target_graph,
            target_action,
            mapping_budget,
            cks_limits,
        )?;
        if candidate.origin_key() != unsigned.origin_key
            || candidate.origin_sequence() != unsigned.origin_sequence
            || candidate.module_id() != unsigned.module_id
            || candidate.source_graph_digest() != unsigned.source_graph_digest
            || candidate.evidence_count() != evidence.len()
            || candidate.evidence_digest() != unsigned.evidence_digest
            || candidate.source_evidence() != evidence.as_slice()
        {
            return Err(ExecutionSyncError::NestedBundleMismatch);
        }
        self.last_sequences
            .insert(unsigned.origin_key, unsigned.origin_sequence);
        Ok(AuthenticatedSyncedKnowledge {
            candidate,
            signature_algorithm: unsigned.signature_algorithm,
            origin_key: unsigned.origin_key,
            origin_sequence: unsigned.origin_sequence,
            module_id: unsigned.module_id,
            receipt_count: evidence.len(),
            receipt_chain_tip,
            plan_digest,
            payload_digest: envelope.payload_digest,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contextual_causal_agent::ContextId;
    use crate::cross_domain_causal_core::{ActionRule, CausalEdge, CausalNode, EdgeKind, NodeKind};
    use crate::deterministic_abstraction::{
        ChannelRange, ContinuousFrame, DeterministicAbstraction, DeterministicAbstractionConfig,
        FixedScalar,
    };
    use crate::end_to_end_causal_runtime::{
        CategoricalStateBinding, CausalCycleClock, CausalCycleIdentity, EndToEndCausalRuntime,
        HostAuthorization, SymbolicStatePattern,
    };
    use crate::hierarchical_causal_graph::{AtomicCausalModule, CompositePlan};

    const TEST_ALGORITHM: u16 = 77;

    #[derive(Clone, Copy)]
    struct TestAuthenticator {
        key_id: u64,
        secret: u64,
    }

    impl TestAuthenticator {
        fn signature(self, message: &[u8]) -> Vec<u8> {
            let first = bytes_digest(self.secret, message);
            let second = bytes_digest(first ^ self.secret.rotate_left(17), message);
            let mut signature = first.to_le_bytes().to_vec();
            signature.extend_from_slice(&second.to_le_bytes());
            signature
        }
    }

    impl ExecutionKnowledgeSigner for TestAuthenticator {
        fn algorithm_id(&self) -> u16 {
            TEST_ALGORITHM
        }

        fn key_id(&self) -> u64 {
            self.key_id
        }

        fn sign(&self, message: &[u8]) -> Option<Vec<u8>> {
            Some((*self).signature(message))
        }
    }

    impl ExecutionKnowledgeVerifier for TestAuthenticator {
        fn verify(&self, algorithm_id: u16, key_id: u64, message: &[u8], signature: &[u8]) -> bool {
            let expected = (*self).signature(message);
            algorithm_id == TEST_ALGORITHM
                && key_id == self.key_id
                && signature == expected.as_slice()
        }
    }

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

    fn config() -> DeterministicAbstractionConfig {
        DeterministicAbstractionConfig::new(
            vec![
                ChannelRange::new(100, 0, 1_000_000).unwrap(),
                ChannelRange::new(200, 0, 1_000_000).unwrap(),
                ChannelRange::new(300, 0, 1_000_000).unwrap(),
            ],
            2,
            0,
            2,
            16,
            77,
        )
        .unwrap()
    }

    fn binding(
        config: &DeterministicAbstractionConfig,
        source: &CanonicalCausalGraph,
    ) -> CategoricalStateBinding {
        let mut patterns = Vec::new();
        for (channel, role) in [(100, 10), (200, 11), (300, 12)] {
            patterns.push(SymbolicStatePattern::new(
                channel,
                0,
                vec![Literal::new(role, false)],
            ));
            patterns.push(SymbolicStatePattern::new(
                channel,
                1,
                vec![Literal::new(role, true)],
            ));
        }
        CategoricalStateBinding::new(config, source, patterns).expect("binding")
    }

    fn frame(a: i64, b: i64, c: i64) -> ContinuousFrame {
        ContinuousFrame::new(vec![
            FixedScalar::from_micros(300, c),
            FixedScalar::from_micros(100, a),
            FixedScalar::from_micros(200, b),
        ])
        .unwrap()
    }

    fn source_runtime() -> (
        CanonicalCausalGraph,
        CausalHierarchy,
        CompositePlan,
        EndToEndCausalRuntime,
    ) {
        let source = graph(10);
        let mut hierarchy = CausalHierarchy::default();
        hierarchy
            .add_atomic(
                AtomicCausalModule::new(
                    7,
                    source.clone(),
                    ActionRule {
                        action: 99,
                        guards: vec![
                            Literal::new(10, true),
                            Literal::new(11, true),
                            Literal::new(12, false),
                        ],
                        effects: vec![Literal::new(12, true)],
                    },
                )
                .unwrap(),
            )
            .unwrap();
        let plan = hierarchy.compose(100, vec![7]).unwrap();
        let config = config();
        let state_binding = binding(&config, &source);
        let runtime =
            EndToEndCausalRuntime::new(DeterministicAbstraction::new(config), state_binding)
                .unwrap();
        (source, hierarchy, plan, runtime)
    }

    fn confirmed_receipts(count: u64) -> (CanonicalCausalGraph, Vec<VerifiedExecutionReceipt>) {
        let (source, hierarchy, plan, mut runtime) = source_runtime();
        let mut receipts = Vec::new();
        for sequence in 1..=count {
            runtime
                .prepare(
                    &hierarchy,
                    &plan,
                    &frame(900_000, 900_000, 100_000),
                    CausalCycleIdentity {
                        context: Some(ContextId(9)),
                        clock: CausalCycleClock {
                            epoch: 1,
                            episode: sequence,
                            sequence,
                        },
                    },
                    500,
                )
                .unwrap();
            let policy = |_: &crate::end_to_end_causal_runtime::PreparedCompositeExecution| {
                HostAuthorization::new(700, sequence)
            };
            runtime.authorize_and_dispatch(&hierarchy, &policy).unwrap();
            receipts.push(
                runtime
                    .resolve_after(&frame(900_000, 900_000, 900_000))
                    .unwrap(),
            );
            runtime.clear_terminal().unwrap();
        }
        (source, receipts)
    }

    fn authenticated_bundle(authenticator: &TestAuthenticator) -> (String, CanonicalCausalGraph) {
        let (source, receipts) = confirmed_receipts(2);
        let bundle = ExecutionKnowledgeSync
            .export_jsonl(
                3,
                42,
                &source,
                &receipts,
                authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            )
            .unwrap();
        (bundle, source)
    }

    #[test]
    fn confirmed_composite_receipts_authenticate_and_transfer_without_target_training() {
        let authenticator = TestAuthenticator {
            key_id: 7001,
            secret: 0x55aa,
        };
        let (bundle, _) = authenticated_bundle(&authenticator);
        let target = graph(20);
        let mut receiver = ExecutionKnowledgeReceiver::new(8).unwrap();
        let authenticated = receiver
            .import_jsonl(
                &bundle,
                &target,
                199,
                MappingBudget::new(128).unwrap(),
                &authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            )
            .unwrap();
        assert_eq!(authenticated.origin_key(), 7001);
        assert_eq!(authenticated.origin_sequence(), 3);
        assert_eq!(authenticated.receipt_count(), 2);
        assert_eq!(receiver.last_sequence(7001), Some(3));
        let before = State::new([
            Literal::new(20, true),
            Literal::new(21, true),
            Literal::new(22, false),
        ]);
        assert_eq!(
            authenticated
                .candidate()
                .predict(&before, 199)
                .unwrap()
                .get(22),
            Some(true)
        );
        let mut hierarchy = CausalHierarchy::default();
        authenticated
            .install_atomic(&mut hierarchy, 8, &target)
            .unwrap();
        let plan = hierarchy.compose(101, vec![8]).unwrap();
        assert_eq!(
            plan.predict(&hierarchy, &before).unwrap().get(22),
            Some(true)
        );
    }

    #[test]
    fn indeterminate_or_contradicted_receipts_never_become_knowledge() {
        let authenticator = TestAuthenticator {
            key_id: 7001,
            secret: 0x55aa,
        };
        let (source, hierarchy, plan, mut runtime) = source_runtime();
        runtime
            .prepare(
                &hierarchy,
                &plan,
                &frame(900_000, 900_000, 100_000),
                CausalCycleIdentity {
                    context: None,
                    clock: CausalCycleClock {
                        epoch: 1,
                        episode: 1,
                        sequence: 1,
                    },
                },
                500,
            )
            .unwrap();
        let policy = |_: &crate::end_to_end_causal_runtime::PreparedCompositeExecution| {
            HostAuthorization::new(700, 1)
        };
        runtime.authorize_and_dispatch(&hierarchy, &policy).unwrap();
        let indeterminate = runtime.resolve_without_observation().unwrap();
        assert_eq!(indeterminate.result(), CommitmentResult::Indeterminate);
        assert_eq!(
            ExecutionKnowledgeSync.export_jsonl(
                1,
                42,
                &source,
                &[indeterminate],
                &authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            ),
            Err(ExecutionSyncError::UnconfirmedReceipt)
        );

        let (source, hierarchy, plan, mut runtime) = source_runtime();
        runtime
            .prepare(
                &hierarchy,
                &plan,
                &frame(900_000, 900_000, 100_000),
                CausalCycleIdentity {
                    context: None,
                    clock: CausalCycleClock {
                        epoch: 1,
                        episode: 1,
                        sequence: 1,
                    },
                },
                500,
            )
            .unwrap();
        runtime.authorize_and_dispatch(&hierarchy, &policy).unwrap();
        let contradicted = runtime
            .resolve_after(&frame(900_000, 900_000, 100_000))
            .unwrap();
        assert_eq!(contradicted.result(), CommitmentResult::Contradicted);
        assert_eq!(
            ExecutionKnowledgeSync.export_jsonl(
                1,
                42,
                &source,
                &[contradicted],
                &authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            ),
            Err(ExecutionSyncError::UnconfirmedReceipt)
        );
    }

    #[test]
    fn tampering_untrusted_keys_and_replay_fail_closed() {
        let authenticator = TestAuthenticator {
            key_id: 7001,
            secret: 0x55aa,
        };
        let (bundle, _) = authenticated_bundle(&authenticator);
        let target = graph(20);
        let tampered = bundle.replacen("\"module_id\":42", "\"module_id\":43", 1);
        let mut receiver = ExecutionKnowledgeReceiver::new(8).unwrap();
        assert_eq!(
            receiver.import_jsonl(
                &tampered,
                &target,
                199,
                MappingBudget::new(128).unwrap(),
                &authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            ),
            Err(ExecutionSyncError::AuthenticationFailed)
        );
        assert_eq!(receiver.last_sequence(7001), None);

        let untrusted = TestAuthenticator {
            key_id: 7001,
            secret: 0xdead,
        };
        assert_eq!(
            receiver.import_jsonl(
                &bundle,
                &target,
                199,
                MappingBudget::new(128).unwrap(),
                &untrusted,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            ),
            Err(ExecutionSyncError::AuthenticationFailed)
        );

        receiver
            .import_jsonl(
                &bundle,
                &target,
                199,
                MappingBudget::new(128).unwrap(),
                &authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            )
            .unwrap();
        assert_eq!(
            receiver.import_jsonl(
                &bundle,
                &target,
                199,
                MappingBudget::new(128).unwrap(),
                &authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            ),
            Err(ExecutionSyncError::ReplayedOriginSequence {
                origin_key: 7001,
                previous: 3,
                requested: 3,
            })
        );
    }

    #[test]
    fn signed_metadata_mismatch_duplicates_and_budgets_are_rejected() {
        let authenticator = TestAuthenticator {
            key_id: 7001,
            secret: 0x55aa,
        };
        let (bundle, _) = authenticated_bundle(&authenticator);
        let line = bundle.strip_suffix('\n').unwrap();
        let mut envelope: WireSignedBundle = serde_json::from_str(line).unwrap();
        envelope.unsigned.evidence_digest ^= 1;
        let payload = serde_json::to_vec(&envelope.unsigned).unwrap();
        envelope.payload_digest = bytes_digest(SIGNED_PAYLOAD_DOMAIN, &payload);
        envelope.signature = authenticator.sign(&payload).unwrap();
        let inconsistent = format!("{}\n", serde_json::to_string(&envelope).unwrap());
        let mut receiver = ExecutionKnowledgeReceiver::new(8).unwrap();
        assert_eq!(
            receiver.import_jsonl(
                &inconsistent,
                &graph(20),
                199,
                MappingBudget::new(128).unwrap(),
                &authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            ),
            Err(ExecutionSyncError::ReceiptIntegrityMismatch)
        );
        assert_eq!(receiver.last_sequence(7001), None);

        let (source, receipts) = confirmed_receipts(1);
        assert_eq!(
            ExecutionKnowledgeSync.export_jsonl(
                4,
                42,
                &source,
                &[receipts[0].clone(), receipts[0].clone()],
                &authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            ),
            Err(ExecutionSyncError::DuplicateReceipt)
        );
        let (_, receipts) = confirmed_receipts(2);
        assert_eq!(
            ExecutionKnowledgeSync.export_jsonl(
                4,
                42,
                &source,
                &receipts,
                &authenticator,
                SyncLimits::default(),
                ExecutionSyncLimits {
                    max_receipts: 1,
                    ..ExecutionSyncLimits::default()
                },
            ),
            Err(ExecutionSyncError::ReceiptBudgetExceeded)
        );
    }
}
