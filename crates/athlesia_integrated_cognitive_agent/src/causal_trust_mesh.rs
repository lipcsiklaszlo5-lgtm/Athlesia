//! Deterministic multi-origin trust admission over authenticated causal sync.
//!
//! Authentication proves which key emitted a P7B execution bundle; it does
//! not prove that the causal claim is safe to install. This layer requires a
//! bounded quorum of distinct, locally trusted origins for one exact target
//! rule. Any divergent rule for the same target graph/action quarantines that
//! scope. Quorum loss or conflict atomically evicts an installed atomic module,
//! which invalidates every captured composite dependency frontier.
//!
//! Claim revocations are canonical signed messages. Signature algorithms and
//! trust roots remain host-owned through the P7B signer/verifier traits.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::causal_execution_sync::{
    AuthenticatedSyncedKnowledge, ExecutionKnowledgeSigner, ExecutionKnowledgeVerifier,
    ExecutionSyncLimits,
};
use super::cross_domain_causal_core::{ActionId, CanonicalCausalGraph, Digest, Literal, TokenId};
use super::hierarchical_causal_graph::{CausalHierarchy, HierarchyError, ModuleId};

const REVOCATION_RECORD: &str = "athlesia/causal-revocation/1";
const REVOCATION_VERSION: u16 = 1;
const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const CLAIM_DOMAIN: u64 = 0x4154_4c45_5349_434c;
const REVOCATION_PAYLOAD_DOMAIN: u64 = 0x4154_4c45_5349_5256;

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
pub struct CausalTrustPolicy {
    pub min_distinct_origins: usize,
    pub max_scopes: usize,
    pub max_variants_per_scope: usize,
    pub max_origins_per_claim: usize,
}

impl CausalTrustPolicy {
    pub const fn new(
        min_distinct_origins: usize,
        max_scopes: usize,
        max_variants_per_scope: usize,
        max_origins_per_claim: usize,
    ) -> Option<Self> {
        if min_distinct_origins == 0
            || max_scopes == 0
            || max_variants_per_scope == 0
            || max_origins_per_claim == 0
            || min_distinct_origins > max_origins_per_claim
        {
            None
        } else {
            Some(Self {
                min_distinct_origins,
                max_scopes,
                max_variants_per_scope,
                max_origins_per_claim,
            })
        }
    }

    fn valid(self) -> bool {
        Self::new(
            self.min_distinct_origins,
            self.max_scopes,
            self.max_variants_per_scope,
            self.max_origins_per_claim,
        )
        .is_some()
    }
}

impl Default for CausalTrustPolicy {
    fn default() -> Self {
        Self {
            min_distinct_origins: 2,
            max_scopes: 128,
            max_variants_per_scope: 8,
            max_origins_per_claim: 16,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CausalClaimScope {
    target_graph_digest: Digest,
    target_action: ActionId,
}

impl CausalClaimScope {
    pub fn target_graph_digest(self) -> Digest {
        self.target_graph_digest
    }

    pub fn target_action(self) -> ActionId {
        self.target_action
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CausalClaim {
    scope: CausalClaimScope,
    guards: Vec<Literal>,
    effects: Vec<Literal>,
    digest: Digest,
}

impl CausalClaim {
    pub fn from_authenticated(
        knowledge: &AuthenticatedSyncedKnowledge,
    ) -> Result<Self, CausalTrustError> {
        let candidate = knowledge.candidate();
        let rule = candidate.rule();
        if rule.action != candidate.target_action() || candidate.target_graph_digest() == 0 {
            return Err(CausalTrustError::InvalidClaim);
        }
        Self::from_parts(
            CausalClaimScope {
                target_graph_digest: candidate.target_graph_digest(),
                target_action: candidate.target_action(),
            },
            rule.guards.clone(),
            rule.effects.clone(),
        )
    }

    fn from_parts(
        scope: CausalClaimScope,
        mut guards: Vec<Literal>,
        mut effects: Vec<Literal>,
    ) -> Result<Self, CausalTrustError> {
        if scope.target_graph_digest == 0 || effects.is_empty() {
            return Err(CausalTrustError::InvalidClaim);
        }
        guards.sort();
        effects.sort();
        validate_literals(&guards)?;
        validate_literals(&effects)?;
        let digest = claim_digest(scope, &guards, &effects);
        Ok(Self {
            scope,
            guards,
            effects,
            digest,
        })
    }

    pub fn scope(&self) -> CausalClaimScope {
        self.scope
    }

    pub fn guards(&self) -> &[Literal] {
        &self.guards
    }

    pub fn effects(&self) -> &[Literal] {
        &self.effects
    }

    pub fn digest(&self) -> Digest {
        self.digest
    }
}

fn validate_literals(literals: &[Literal]) -> Result<(), CausalTrustError> {
    if literals
        .iter()
        .any(|literal| literal.role > u64::from(u32::MAX))
        || literals.windows(2).any(|pair| pair[0].role == pair[1].role)
    {
        return Err(CausalTrustError::InvalidClaim);
    }
    Ok(())
}

fn claim_digest(scope: CausalClaimScope, guards: &[Literal], effects: &[Literal]) -> Digest {
    let mut digest = mix(FNV_OFFSET, CLAIM_DOMAIN);
    digest = mix(digest, scope.target_graph_digest);
    digest = mix(digest, scope.target_action);
    digest = mix(digest, guards.len() as u64);
    for guard in guards {
        digest = mix(digest, guard.role);
        digest = mix(digest, u64::from(guard.value));
    }
    digest = mix(digest, effects.len() as u64);
    for effect in effects {
        digest = mix(digest, effect.role);
        digest = mix(digest, u64::from(effect.value));
    }
    digest.max(1)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalClaimStatus {
    Unknown,
    Pending {
        claim_digest: Digest,
        endorsements: usize,
        required: usize,
    },
    Admitted {
        claim_digest: Digest,
        endorsements: usize,
    },
    Quarantined {
        variants: usize,
        endorsements: usize,
        saturated: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalTrustError {
    InvalidPolicy,
    InvalidTrustedOrigin,
    DuplicateTrustedOrigin(u64),
    UntrustedOrigin(u64),
    InvalidClaim,
    StaleOriginSequence {
        origin_key: u64,
        previous: u64,
        requested: u64,
    },
    ScopeBudgetExceeded {
        observed: usize,
        limit: usize,
    },
    OriginBudgetExceeded {
        observed: usize,
        limit: usize,
    },
    NotAdmitted(CausalClaimScope),
    AlreadyInstalled(CausalClaimScope),
    TargetGraphMismatch {
        expected: Digest,
        observed: Digest,
    },
    InvalidLimits,
    EnvelopeTooLarge,
    MalformedRevocation,
    NonCanonicalRevocation,
    UnsupportedRevocationProtocol(u16),
    SignatureBudgetExceeded,
    SigningFailed,
    AuthenticationFailed,
    InvalidRevocation,
    Hierarchy(HierarchyError),
}

impl std::fmt::Display for CausalTrustError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CausalTrustError {}

impl From<HierarchyError> for CausalTrustError {
    fn from(error: HierarchyError) -> Self {
        Self::Hierarchy(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClaimVariant {
    by_origin: BTreeMap<u64, AuthenticatedSyncedKnowledge>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScopeState {
    variants: BTreeMap<CausalClaim, ClaimVariant>,
    saturated_conflict: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InstalledClaim {
    claim: CausalClaim,
    module_id: ModuleId,
}

/// Bounded local admission state. Trusted keys are deployment policy, not
/// learned data. A key contributes at most one vote to each exact claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalTrustMesh {
    policy: CausalTrustPolicy,
    trusted_origins: BTreeSet<u64>,
    last_origin_sequences: BTreeMap<u64, u64>,
    scopes: BTreeMap<CausalClaimScope, ScopeState>,
    installations: BTreeMap<CausalClaimScope, InstalledClaim>,
}

impl CausalTrustMesh {
    pub fn new(
        policy: CausalTrustPolicy,
        trusted_origins: impl IntoIterator<Item = u64>,
    ) -> Result<Self, CausalTrustError> {
        if !policy.valid() {
            return Err(CausalTrustError::InvalidPolicy);
        }
        let mut trusted = BTreeSet::new();
        for origin in trusted_origins {
            if origin == 0 {
                return Err(CausalTrustError::InvalidTrustedOrigin);
            }
            if !trusted.insert(origin) {
                return Err(CausalTrustError::DuplicateTrustedOrigin(origin));
            }
        }
        if trusted.len() < policy.min_distinct_origins
            || trusted.len() > policy.max_origins_per_claim
        {
            return Err(CausalTrustError::InvalidPolicy);
        }
        Ok(Self {
            policy,
            trusted_origins: trusted,
            last_origin_sequences: BTreeMap::new(),
            scopes: BTreeMap::new(),
            installations: BTreeMap::new(),
        })
    }

    pub fn policy(&self) -> CausalTrustPolicy {
        self.policy
    }

    pub fn is_trusted_origin(&self, origin_key: u64) -> bool {
        self.trusted_origins.contains(&origin_key)
    }

    pub fn last_origin_sequence(&self, origin_key: u64) -> Option<u64> {
        self.last_origin_sequences.get(&origin_key).copied()
    }

    pub fn installed_module(&self, scope: CausalClaimScope) -> Option<ModuleId> {
        self.installations
            .get(&scope)
            .map(|installation| installation.module_id)
    }

    pub fn status(&self, scope: CausalClaimScope) -> CausalClaimStatus {
        let Some(state) = self.scopes.get(&scope) else {
            return CausalClaimStatus::Unknown;
        };
        let endorsements = state
            .variants
            .values()
            .map(|variant| variant.by_origin.len())
            .sum();
        if state.saturated_conflict || state.variants.len() > 1 {
            return CausalClaimStatus::Quarantined {
                variants: state.variants.len(),
                endorsements,
                saturated: state.saturated_conflict,
            };
        }
        let Some((claim, variant)) = state.variants.iter().next() else {
            return CausalClaimStatus::Unknown;
        };
        if variant.by_origin.len() >= self.policy.min_distinct_origins {
            CausalClaimStatus::Admitted {
                claim_digest: claim.digest(),
                endorsements: variant.by_origin.len(),
            }
        } else {
            CausalClaimStatus::Pending {
                claim_digest: claim.digest(),
                endorsements: variant.by_origin.len(),
                required: self.policy.min_distinct_origins,
            }
        }
    }

    /// Observe one already-authenticated P7B candidate. The update and any
    /// required hierarchy eviction are one transaction from the caller's
    /// perspective.
    pub fn observe_authenticated(
        &mut self,
        knowledge: AuthenticatedSyncedKnowledge,
        hierarchy: &mut CausalHierarchy,
    ) -> Result<CausalClaimStatus, CausalTrustError> {
        let scope = CausalClaim::from_authenticated(&knowledge)?.scope();
        let mut next = self.clone();
        let status = next.observe_inner(knowledge)?;
        next.reconcile_scope(scope, hierarchy)?;
        *self = next;
        Ok(status)
    }

    fn observe_inner(
        &mut self,
        knowledge: AuthenticatedSyncedKnowledge,
    ) -> Result<CausalClaimStatus, CausalTrustError> {
        let origin = knowledge.origin_key();
        let sequence = knowledge.origin_sequence();
        if !self.is_trusted_origin(origin) {
            return Err(CausalTrustError::UntrustedOrigin(origin));
        }
        if let Some(previous) = self.last_origin_sequence(origin) {
            if sequence <= previous {
                return Err(CausalTrustError::StaleOriginSequence {
                    origin_key: origin,
                    previous,
                    requested: sequence,
                });
            }
        }
        let claim = CausalClaim::from_authenticated(&knowledge)?;
        let scope = claim.scope();
        if !self.scopes.contains_key(&scope) && self.scopes.len() >= self.policy.max_scopes {
            return Err(CausalTrustError::ScopeBudgetExceeded {
                observed: self.scopes.len().saturating_add(1),
                limit: self.policy.max_scopes,
            });
        }
        let state = self.scopes.entry(scope).or_insert_with(|| ScopeState {
            variants: BTreeMap::new(),
            saturated_conflict: false,
        });
        if !state.variants.contains_key(&claim)
            && state.variants.len() >= self.policy.max_variants_per_scope
        {
            state.saturated_conflict = true;
            self.last_origin_sequences.insert(origin, sequence);
            return Ok(self.status(scope));
        }
        let variant = state.variants.entry(claim).or_insert_with(|| ClaimVariant {
            by_origin: BTreeMap::new(),
        });
        if !variant.by_origin.contains_key(&origin)
            && variant.by_origin.len() >= self.policy.max_origins_per_claim
        {
            return Err(CausalTrustError::OriginBudgetExceeded {
                observed: variant.by_origin.len().saturating_add(1),
                limit: self.policy.max_origins_per_claim,
            });
        }
        variant.by_origin.insert(origin, knowledge);
        self.last_origin_sequences.insert(origin, sequence);
        Ok(self.status(scope))
    }

    pub fn install_admitted_atomic(
        &mut self,
        scope: CausalClaimScope,
        hierarchy: &mut CausalHierarchy,
        module_id: ModuleId,
        target_graph: &CanonicalCausalGraph,
    ) -> Result<(), CausalTrustError> {
        if self.installations.contains_key(&scope) {
            return Err(CausalTrustError::AlreadyInstalled(scope));
        }
        if target_graph.digest() != scope.target_graph_digest {
            return Err(CausalTrustError::TargetGraphMismatch {
                expected: scope.target_graph_digest,
                observed: target_graph.digest(),
            });
        }
        let (claim, representative) = self
            .admitted_variant(scope)
            .map(|(claim, variant)| {
                (
                    claim.clone(),
                    variant
                        .by_origin
                        .iter()
                        .next()
                        .expect("admitted variant has an origin")
                        .1
                        .clone(),
                )
            })
            .ok_or(CausalTrustError::NotAdmitted(scope))?;
        representative.install_atomic(hierarchy, module_id, target_graph)?;
        debug_assert!(hierarchy.module_digest(module_id).is_some());
        self.installations
            .insert(scope, InstalledClaim { claim, module_id });
        Ok(())
    }

    fn admitted_variant(&self, scope: CausalClaimScope) -> Option<(&CausalClaim, &ClaimVariant)> {
        let state = self.scopes.get(&scope)?;
        if state.saturated_conflict || state.variants.len() != 1 {
            return None;
        }
        let (claim, variant) = state.variants.iter().next()?;
        (variant.by_origin.len() >= self.policy.min_distinct_origins).then_some((claim, variant))
    }

    fn reconcile_scope(
        &mut self,
        scope: CausalClaimScope,
        hierarchy: &mut CausalHierarchy,
    ) -> Result<(), CausalTrustError> {
        let Some(installation) = self.installations.get(&scope).cloned() else {
            return Ok(());
        };
        let remains_admitted = self
            .admitted_variant(scope)
            .is_some_and(|(claim, _)| claim == &installation.claim);
        if remains_admitted {
            return Ok(());
        }
        if hierarchy.module_digest(installation.module_id).is_some() {
            // The mesh owns this module id after installation. Even an
            // unexpected atomic replacement is removed on loss of admission.
            hierarchy.remove_atomic(installation.module_id)?;
        }
        self.installations.remove(&scope);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn apply_revocation_jsonl<V: ExecutionKnowledgeVerifier>(
        &mut self,
        jsonl: &str,
        hierarchy: &mut CausalHierarchy,
        verifier: &V,
        limits: ExecutionSyncLimits,
    ) -> Result<RevocationOutcome, CausalTrustError> {
        let verified = verify_revocation(jsonl, verifier, limits)?;
        if !self.is_trusted_origin(verified.origin_key) {
            return Err(CausalTrustError::UntrustedOrigin(verified.origin_key));
        }
        if let Some(previous) = self.last_origin_sequence(verified.origin_key) {
            if verified.origin_sequence <= previous {
                return Err(CausalTrustError::StaleOriginSequence {
                    origin_key: verified.origin_key,
                    previous,
                    requested: verified.origin_sequence,
                });
            }
        }
        let scope = verified.claim.scope();
        let mut next = self.clone();
        let mut removed = false;
        let mut remove_scope = false;
        if let Some(state) = next.scopes.get_mut(&scope) {
            let mut remove_variant = false;
            if let Some(variant) = state.variants.get_mut(&verified.claim) {
                removed = variant.by_origin.remove(&verified.origin_key).is_some();
                remove_variant = variant.by_origin.is_empty();
            }
            if remove_variant {
                state.variants.remove(&verified.claim);
            }
            remove_scope = state.variants.is_empty() && !state.saturated_conflict;
        }
        if remove_scope {
            next.scopes.remove(&scope);
        }
        next.last_origin_sequences
            .insert(verified.origin_key, verified.origin_sequence);
        next.reconcile_scope(scope, hierarchy)?;
        let status = next.status(scope);
        *self = next;
        Ok(RevocationOutcome {
            origin_key: verified.origin_key,
            origin_sequence: verified.origin_sequence,
            scope,
            claim_digest: verified.claim.digest(),
            removed,
            status,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevocationOutcome {
    origin_key: u64,
    origin_sequence: u64,
    scope: CausalClaimScope,
    claim_digest: Digest,
    removed: bool,
    status: CausalClaimStatus,
}

impl RevocationOutcome {
    pub fn origin_key(&self) -> u64 {
        self.origin_key
    }

    pub fn origin_sequence(&self) -> u64 {
        self.origin_sequence
    }

    pub fn scope(&self) -> CausalClaimScope {
        self.scope
    }

    pub fn claim_digest(&self) -> Digest {
        self.claim_digest
    }

    pub fn removed(&self) -> bool {
        self.removed
    }

    pub fn status(&self) -> &CausalClaimStatus {
        &self.status
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireRevocationUnsigned {
    record: String,
    protocol_version: u16,
    signature_algorithm: u16,
    origin_key: u64,
    origin_sequence: u64,
    target_graph_digest: Digest,
    target_action: ActionId,
    guards: Vec<(TokenId, bool)>,
    effects: Vec<(TokenId, bool)>,
    claim_digest: Digest,
    reason_digest: Digest,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSignedRevocation {
    unsigned: WireRevocationUnsigned,
    payload_digest: Digest,
    signature: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VerifiedRevocation {
    origin_key: u64,
    origin_sequence: u64,
    claim: CausalClaim,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CausalRevocationSync;

impl CausalRevocationSync {
    pub fn export_jsonl<S: ExecutionKnowledgeSigner>(
        &self,
        origin_sequence: u64,
        claim: &CausalClaim,
        reason_digest: Digest,
        signer: &S,
        limits: ExecutionSyncLimits,
    ) -> Result<String, CausalTrustError> {
        validate_sync_limits(limits)?;
        let signature_algorithm = signer.algorithm_id();
        let origin_key = signer.key_id();
        if signature_algorithm == 0 || origin_key == 0 || origin_sequence == 0 || reason_digest == 0
        {
            return Err(CausalTrustError::InvalidRevocation);
        }
        let unsigned = WireRevocationUnsigned {
            record: REVOCATION_RECORD.to_owned(),
            protocol_version: REVOCATION_VERSION,
            signature_algorithm,
            origin_key,
            origin_sequence,
            target_graph_digest: claim.scope.target_graph_digest,
            target_action: claim.scope.target_action,
            guards: claim
                .guards
                .iter()
                .map(|literal| (literal.role, literal.value))
                .collect(),
            effects: claim
                .effects
                .iter()
                .map(|literal| (literal.role, literal.value))
                .collect(),
            claim_digest: claim.digest,
            reason_digest,
        };
        let payload =
            serde_json::to_vec(&unsigned).map_err(|_| CausalTrustError::MalformedRevocation)?;
        let signature = signer
            .sign(&payload)
            .ok_or(CausalTrustError::SigningFailed)?;
        if signature.is_empty() || signature.len() > limits.max_signature_bytes {
            return Err(CausalTrustError::SignatureBudgetExceeded);
        }
        let envelope = WireSignedRevocation {
            unsigned,
            payload_digest: bytes_digest(REVOCATION_PAYLOAD_DOMAIN, &payload),
            signature,
        };
        let mut jsonl =
            serde_json::to_string(&envelope).map_err(|_| CausalTrustError::MalformedRevocation)?;
        jsonl.push('\n');
        if jsonl.len() > limits.max_envelope_bytes {
            return Err(CausalTrustError::EnvelopeTooLarge);
        }
        Ok(jsonl)
    }
}

fn validate_sync_limits(limits: ExecutionSyncLimits) -> Result<(), CausalTrustError> {
    if limits.max_envelope_bytes == 0 || limits.max_receipts == 0 || limits.max_signature_bytes == 0
    {
        return Err(CausalTrustError::InvalidLimits);
    }
    Ok(())
}

fn verify_revocation<V: ExecutionKnowledgeVerifier>(
    jsonl: &str,
    verifier: &V,
    limits: ExecutionSyncLimits,
) -> Result<VerifiedRevocation, CausalTrustError> {
    validate_sync_limits(limits)?;
    if jsonl.is_empty() || jsonl.len() > limits.max_envelope_bytes {
        return Err(CausalTrustError::EnvelopeTooLarge);
    }
    if !jsonl.ends_with('\n') || jsonl[..jsonl.len() - 1].contains('\n') {
        return Err(CausalTrustError::MalformedRevocation);
    }
    let line = &jsonl[..jsonl.len() - 1];
    let envelope: WireSignedRevocation =
        serde_json::from_str(line).map_err(|_| CausalTrustError::MalformedRevocation)?;
    let canonical =
        serde_json::to_string(&envelope).map_err(|_| CausalTrustError::MalformedRevocation)?;
    if canonical != line {
        return Err(CausalTrustError::NonCanonicalRevocation);
    }
    let unsigned = &envelope.unsigned;
    if unsigned.record != REVOCATION_RECORD {
        return Err(CausalTrustError::UnsupportedRevocationProtocol(0));
    }
    if unsigned.protocol_version != REVOCATION_VERSION {
        return Err(CausalTrustError::UnsupportedRevocationProtocol(
            unsigned.protocol_version,
        ));
    }
    if unsigned.signature_algorithm == 0
        || unsigned.origin_key == 0
        || unsigned.origin_sequence == 0
        || unsigned.reason_digest == 0
    {
        return Err(CausalTrustError::InvalidRevocation);
    }
    if envelope.signature.is_empty() || envelope.signature.len() > limits.max_signature_bytes {
        return Err(CausalTrustError::SignatureBudgetExceeded);
    }
    let payload =
        serde_json::to_vec(unsigned).map_err(|_| CausalTrustError::MalformedRevocation)?;
    if envelope.payload_digest != bytes_digest(REVOCATION_PAYLOAD_DOMAIN, &payload)
        || !verifier.verify(
            unsigned.signature_algorithm,
            unsigned.origin_key,
            &payload,
            &envelope.signature,
        )
    {
        return Err(CausalTrustError::AuthenticationFailed);
    }
    let wire_guards: Vec<Literal> = unsigned
        .guards
        .iter()
        .map(|(role, value)| Literal::new(*role, *value))
        .collect();
    let wire_effects: Vec<Literal> = unsigned
        .effects
        .iter()
        .map(|(role, value)| Literal::new(*role, *value))
        .collect();
    let claim = CausalClaim::from_parts(
        CausalClaimScope {
            target_graph_digest: unsigned.target_graph_digest,
            target_action: unsigned.target_action,
        },
        wire_guards.clone(),
        wire_effects.clone(),
    )?;
    if claim.guards != wire_guards
        || claim.effects != wire_effects
        || claim.digest != unsigned.claim_digest
    {
        return Err(CausalTrustError::InvalidRevocation);
    }
    Ok(VerifiedRevocation {
        origin_key: unsigned.origin_key,
        origin_sequence: unsigned.origin_sequence,
        claim,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal_execution_sync::{ExecutionKnowledgeReceiver, ExecutionKnowledgeSync};
    use crate::causal_knowledge_sync::SyncLimits;
    use crate::contextual_causal_agent::ContextId;
    use crate::cross_domain_causal_core::{
        ActionRule, CausalEdge, CausalNode, EdgeKind, MappingBudget, NodeKind, State,
    };
    use crate::deterministic_abstraction::{
        ChannelRange, ContinuousFrame, DeterministicAbstraction, DeterministicAbstractionConfig,
        FixedScalar,
    };
    use crate::end_to_end_causal_runtime::{
        CategoricalStateBinding, CausalCycleClock, CausalCycleIdentity, EndToEndCausalRuntime,
        HostAuthorization, SymbolicStatePattern,
    };
    use crate::hierarchical_causal_graph::{AtomicCausalModule, CompositePlan};

    const TEST_ALGORITHM: u16 = 91;

    #[derive(Clone, Copy)]
    struct TestKey {
        key_id: u64,
        secret: u64,
    }

    impl TestKey {
        fn signature(self, message: &[u8]) -> Vec<u8> {
            let first = bytes_digest(self.secret, message);
            let second = bytes_digest(first ^ self.secret.rotate_left(13), message);
            let mut signature = first.to_le_bytes().to_vec();
            signature.extend_from_slice(&second.to_le_bytes());
            signature
        }
    }

    impl ExecutionKnowledgeSigner for TestKey {
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

    impl ExecutionKnowledgeVerifier for TestKey {
        fn verify(&self, algorithm_id: u16, key_id: u64, message: &[u8], signature: &[u8]) -> bool {
            algorithm_id == TEST_ALGORITHM
                && key_id == self.key_id
                && signature == (*self).signature(message).as_slice()
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
        .unwrap()
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
        CategoricalStateBinding::new(config, source, patterns).unwrap()
    }

    fn frame(a: i64, b: i64, c: i64) -> ContinuousFrame {
        ContinuousFrame::new(vec![
            FixedScalar::from_micros(300, c),
            FixedScalar::from_micros(100, a),
            FixedScalar::from_micros(200, b),
        ])
        .unwrap()
    }

    fn confirmed_receipt(
        effect: bool,
    ) -> (
        CanonicalCausalGraph,
        Vec<crate::end_to_end_causal_runtime::VerifiedExecutionReceipt>,
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
                        guards: vec![Literal::new(10, true), Literal::new(11, true)],
                        effects: vec![Literal::new(12, effect)],
                    },
                )
                .unwrap(),
            )
            .unwrap();
        let plan: CompositePlan = hierarchy.compose(100, vec![7]).unwrap();
        let abstraction_config = config();
        let state_binding = binding(&abstraction_config, &source);
        let mut runtime = EndToEndCausalRuntime::new(
            DeterministicAbstraction::new(abstraction_config),
            state_binding,
        )
        .unwrap();
        let before_c = if effect { 100_000 } else { 900_000 };
        let after_c = if effect { 900_000 } else { 100_000 };
        runtime
            .prepare(
                &hierarchy,
                &plan,
                &frame(900_000, 900_000, before_c),
                CausalCycleIdentity {
                    context: Some(ContextId(9)),
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
        let receipt = runtime
            .resolve_after(&frame(900_000, 900_000, after_c))
            .unwrap();
        (source, vec![receipt])
    }

    fn authenticated(
        key: TestKey,
        origin_sequence: u64,
        effect: bool,
        target: &CanonicalCausalGraph,
        receiver: &mut ExecutionKnowledgeReceiver,
    ) -> AuthenticatedSyncedKnowledge {
        let (source, receipts) = confirmed_receipt(effect);
        let bundle = ExecutionKnowledgeSync
            .export_jsonl(
                origin_sequence,
                42,
                &source,
                &receipts,
                &key,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            )
            .unwrap();
        receiver
            .import_jsonl(
                &bundle,
                target,
                199,
                MappingBudget::new(128).unwrap(),
                &key,
                SyncLimits::default(),
                ExecutionSyncLimits::default(),
            )
            .unwrap()
    }

    fn policy(max_variants: usize) -> CausalTrustPolicy {
        CausalTrustPolicy::new(2, 8, max_variants, 8).unwrap()
    }

    fn target_before() -> State {
        State::new([
            Literal::new(20, true),
            Literal::new(21, true),
            Literal::new(22, false),
        ])
    }

    #[test]
    fn distinct_trusted_origins_admit_and_install_zero_shot() {
        let key_a = TestKey {
            key_id: 7001,
            secret: 11,
        };
        let key_b = TestKey {
            key_id: 7002,
            secret: 22,
        };
        let target = graph(20);
        let mut receiver = ExecutionKnowledgeReceiver::new(8).unwrap();
        let first = authenticated(key_a, 1, true, &target, &mut receiver);
        let second = authenticated(key_b, 1, true, &target, &mut receiver);
        let scope = CausalClaim::from_authenticated(&first).unwrap().scope();
        let mut hierarchy = CausalHierarchy::default();
        let mut mesh = CausalTrustMesh::new(policy(4), [7001, 7002]).unwrap();
        assert!(matches!(
            mesh.observe_authenticated(first, &mut hierarchy).unwrap(),
            CausalClaimStatus::Pending {
                endorsements: 1,
                required: 2,
                ..
            }
        ));
        assert!(matches!(
            mesh.observe_authenticated(second, &mut hierarchy).unwrap(),
            CausalClaimStatus::Admitted {
                endorsements: 2,
                ..
            }
        ));
        mesh.install_admitted_atomic(scope, &mut hierarchy, 8, &target)
            .unwrap();
        let plan = hierarchy.compose(101, vec![8]).unwrap();
        assert_eq!(
            plan.predict(&hierarchy, &target_before()).unwrap().get(22),
            Some(true)
        );
        assert_eq!(mesh.installed_module(scope), Some(8));
    }

    #[test]
    fn untrusted_and_replayed_origins_cannot_manufacture_quorum() {
        let trusted = TestKey {
            key_id: 7001,
            secret: 11,
        };
        let untrusted = TestKey {
            key_id: 7999,
            secret: 99,
        };
        let target = graph(20);
        let mut receiver = ExecutionKnowledgeReceiver::new(8).unwrap();
        let first = authenticated(trusted, 1, true, &target, &mut receiver);
        let replay = first.clone();
        let outsider = authenticated(untrusted, 1, true, &target, &mut receiver);
        let scope = CausalClaim::from_authenticated(&first).unwrap().scope();
        let mut hierarchy = CausalHierarchy::default();
        let mut mesh = CausalTrustMesh::new(policy(4), [7001, 7002]).unwrap();
        assert_eq!(
            mesh.observe_authenticated(outsider, &mut hierarchy),
            Err(CausalTrustError::UntrustedOrigin(7999))
        );
        mesh.observe_authenticated(first, &mut hierarchy).unwrap();
        assert!(matches!(
            mesh.observe_authenticated(replay, &mut hierarchy),
            Err(CausalTrustError::StaleOriginSequence {
                origin_key: 7001,
                previous: 1,
                requested: 1,
            })
        ));
        assert!(matches!(
            mesh.install_admitted_atomic(scope, &mut hierarchy, 8, &target),
            Err(CausalTrustError::NotAdmitted(observed)) if observed == scope
        ));
    }

    #[test]
    fn conflicting_claim_quarantines_and_evicts_installed_dependency() {
        let keys = [
            TestKey {
                key_id: 7001,
                secret: 11,
            },
            TestKey {
                key_id: 7002,
                secret: 22,
            },
            TestKey {
                key_id: 7003,
                secret: 33,
            },
        ];
        let target = graph(20);
        let mut receiver = ExecutionKnowledgeReceiver::new(8).unwrap();
        let positive_a = authenticated(keys[0], 1, true, &target, &mut receiver);
        let positive_b = authenticated(keys[1], 1, true, &target, &mut receiver);
        let negative = authenticated(keys[2], 1, false, &target, &mut receiver);
        let negative_claim = CausalClaim::from_authenticated(&negative).unwrap();
        let scope = negative_claim.scope();
        let mut hierarchy = CausalHierarchy::default();
        let mut mesh = CausalTrustMesh::new(policy(4), [7001, 7002, 7003]).unwrap();
        mesh.observe_authenticated(positive_a, &mut hierarchy)
            .unwrap();
        mesh.observe_authenticated(positive_b, &mut hierarchy)
            .unwrap();
        mesh.install_admitted_atomic(scope, &mut hierarchy, 8, &target)
            .unwrap();
        let plan = hierarchy.compose(101, vec![8]).unwrap();
        assert!(matches!(
            mesh.observe_authenticated(negative, &mut hierarchy)
                .unwrap(),
            CausalClaimStatus::Quarantined {
                variants: 2,
                saturated: false,
                ..
            }
        ));
        assert_eq!(mesh.installed_module(scope), None);
        assert_eq!(hierarchy.module_digest(8), None);
        assert!(matches!(
            plan.predict(&hierarchy, &target_before()),
            Err(HierarchyError::StalePlan { root: 101 })
        ));

        let revocation = CausalRevocationSync
            .export_jsonl(
                2,
                &negative_claim,
                901,
                &keys[2],
                ExecutionSyncLimits::default(),
            )
            .unwrap();
        let outcome = mesh
            .apply_revocation_jsonl(
                &revocation,
                &mut hierarchy,
                &keys[2],
                ExecutionSyncLimits::default(),
            )
            .unwrap();
        assert!(outcome.removed());
        assert!(matches!(
            outcome.status(),
            CausalClaimStatus::Admitted {
                endorsements: 2,
                ..
            }
        ));
        mesh.install_admitted_atomic(scope, &mut hierarchy, 8, &target)
            .unwrap();
        assert_eq!(
            plan.predict(&hierarchy, &target_before()).unwrap().get(22),
            Some(true)
        );
    }

    #[test]
    fn signed_revocation_drops_quorum_and_replay_or_tampering_fail_closed() {
        let key_a = TestKey {
            key_id: 7001,
            secret: 11,
        };
        let key_b = TestKey {
            key_id: 7002,
            secret: 22,
        };
        let target = graph(20);
        let mut receiver = ExecutionKnowledgeReceiver::new(8).unwrap();
        let first = authenticated(key_a, 1, true, &target, &mut receiver);
        let claim = CausalClaim::from_authenticated(&first).unwrap();
        let second = authenticated(key_b, 1, true, &target, &mut receiver);
        let scope = claim.scope();
        let mut hierarchy = CausalHierarchy::default();
        let mut mesh = CausalTrustMesh::new(policy(4), [7001, 7002]).unwrap();
        mesh.observe_authenticated(first, &mut hierarchy).unwrap();
        mesh.observe_authenticated(second, &mut hierarchy).unwrap();
        mesh.install_admitted_atomic(scope, &mut hierarchy, 8, &target)
            .unwrap();
        let revocation = CausalRevocationSync
            .export_jsonl(2, &claim, 902, &key_a, ExecutionSyncLimits::default())
            .unwrap();
        let mut envelope: WireSignedRevocation =
            serde_json::from_str(revocation.trim_end()).unwrap();
        envelope.payload_digest ^= 1;
        let tampered = format!("{}\n", serde_json::to_string(&envelope).unwrap());
        assert_eq!(
            mesh.apply_revocation_jsonl(
                &tampered,
                &mut hierarchy,
                &key_a,
                ExecutionSyncLimits::default(),
            ),
            Err(CausalTrustError::AuthenticationFailed)
        );
        assert_eq!(mesh.installed_module(scope), Some(8));

        let outcome = mesh
            .apply_revocation_jsonl(
                &revocation,
                &mut hierarchy,
                &key_a,
                ExecutionSyncLimits::default(),
            )
            .unwrap();
        assert!(matches!(
            outcome.status(),
            CausalClaimStatus::Pending {
                endorsements: 1,
                required: 2,
                ..
            }
        ));
        assert_eq!(hierarchy.module_digest(8), None);
        assert!(matches!(
            mesh.apply_revocation_jsonl(
                &revocation,
                &mut hierarchy,
                &key_a,
                ExecutionSyncLimits::default(),
            ),
            Err(CausalTrustError::StaleOriginSequence {
                origin_key: 7001,
                previous: 2,
                requested: 2,
            })
        ));
    }

    #[test]
    fn variant_budget_overflow_is_sticky_quarantine_not_silent_drop() {
        let keys = [
            TestKey {
                key_id: 7001,
                secret: 11,
            },
            TestKey {
                key_id: 7002,
                secret: 22,
            },
            TestKey {
                key_id: 7003,
                secret: 33,
            },
        ];
        let target = graph(20);
        let mut receiver = ExecutionKnowledgeReceiver::new(8).unwrap();
        let positive_a = authenticated(keys[0], 1, true, &target, &mut receiver);
        let positive_b = authenticated(keys[1], 1, true, &target, &mut receiver);
        let negative = authenticated(keys[2], 1, false, &target, &mut receiver);
        let scope = CausalClaim::from_authenticated(&positive_a)
            .unwrap()
            .scope();
        let mut hierarchy = CausalHierarchy::default();
        let mut mesh = CausalTrustMesh::new(policy(1), [7001, 7002, 7003]).unwrap();
        mesh.observe_authenticated(positive_a, &mut hierarchy)
            .unwrap();
        mesh.observe_authenticated(positive_b, &mut hierarchy)
            .unwrap();
        mesh.install_admitted_atomic(scope, &mut hierarchy, 8, &target)
            .unwrap();
        assert!(matches!(
            mesh.observe_authenticated(negative, &mut hierarchy)
                .unwrap(),
            CausalClaimStatus::Quarantined {
                variants: 1,
                saturated: true,
                ..
            }
        ));
        assert_eq!(hierarchy.module_digest(8), None);
        assert!(matches!(
            mesh.status(scope),
            CausalClaimStatus::Quarantined {
                saturated: true,
                ..
            }
        ));
    }
}
