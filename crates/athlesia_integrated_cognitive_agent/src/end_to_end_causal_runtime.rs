//! Transactional end-to-end causal execution runtime.
//!
//! This module connects the P1-P6 boundaries without introducing a second
//! learner or execution path:
//!
//! `ContinuousFrame -> deterministic abstraction -> explicit graph binding
//!  -> hierarchical plan -> frozen prediction commitment -> host policy
//!  -> one-shot dispatch -> verified outcome`.
//!
//! The runtime never executes an action itself. Atomic actions become visible
//! only in an `AuthorizedCompositeDispatch`, after a host policy has approved
//! the exact commitment and its permit has been consumed. Any missing,
//! malformed, stale, ambiguous, or over-budget input remains fail-closed.

use std::collections::{BTreeMap, BTreeSet};

use super::causal_proof::{
    CommitmentResult, ExecutionPermit, PredictionCommitment, PredictionScope, ProofError,
};
use super::causal_trust_mesh::{
    CausalExecutionLease, CausalTrustAdmission, CausalTrustError, CausalTrustMesh,
};
use super::contextual_causal_agent::ContextId;
use super::cross_domain_causal_core::{
    ActionId, CanonicalCausalGraph, Digest, Literal, State, TokenId,
};
use super::deterministic_abstraction::{
    AbstractionError, ContinuousFrame, DeterministicAbstraction, DeterministicAbstractionConfig,
    SymbolicAbstraction, SymbolicObservation,
};
use super::hierarchical_causal_graph::{
    CausalHierarchy, CompositePlan, HierarchyError, HierarchyProofError, ModuleId,
};

const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const BINDING_DOMAIN: u64 = 0x4154_4c45_5349_4244;
const BOUND_SCHEMA_DOMAIN: u64 = 0x4154_4c45_5349_4253;
const BOUND_OBSERVATION_DOMAIN: u64 = 0x4154_4c45_5349_424f;
const DISPATCH_DOMAIN: u64 = 0x4154_4c45_5349_4450;

fn mix(mut state: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state.max(1)
}

/// One categorical channel value represented as a complete Boolean pattern
/// over graph roles.
///
/// All bins for one channel must use the same target-role set and every bin
/// must have a distinct bit pattern. This permits both binary and one-hot or
/// compact multi-bit categorical encodings without learned projections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymbolicStatePattern {
    channel_role: TokenId,
    bin: u8,
    literals: Vec<Literal>,
}

impl SymbolicStatePattern {
    pub fn new(channel_role: TokenId, bin: u8, mut literals: Vec<Literal>) -> Self {
        literals.sort();
        Self {
            channel_role,
            bin,
            literals,
        }
    }

    pub fn channel_role(&self) -> TokenId {
        self.channel_role
    }
    pub fn bin(&self) -> u8 {
        self.bin
    }
    pub fn literals(&self) -> &[Literal] {
        &self.literals
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingError {
    EmptyBinding,
    DuplicatePattern {
        channel_role: TokenId,
        bin: u8,
    },
    MissingPattern {
        channel_role: TokenId,
        bin: u8,
    },
    UnknownChannel(TokenId),
    InvalidBin {
        channel_role: TokenId,
        bin: u8,
    },
    EmptyPattern {
        channel_role: TokenId,
        bin: u8,
    },
    DuplicateTargetRole {
        channel_role: TokenId,
        bin: u8,
        role: TokenId,
    },
    UnknownGraphRole(TokenId),
    InconsistentRoleSet {
        channel_role: TokenId,
        bin: u8,
    },
    AliasedPattern {
        channel_role: TokenId,
        first_bin: u8,
        second_bin: u8,
    },
    CrossChannelRoleCollision {
        role: TokenId,
        first_channel: TokenId,
        second_channel: TokenId,
    },
    StateBudgetExceeded {
        observed: usize,
        limit: usize,
    },
    SchemaMismatch {
        expected: Digest,
        observed: Digest,
    },
    ObservationChannelMismatch,
    ObservationBinOutOfRange {
        channel_role: TokenId,
        bin: u8,
    },
}

impl std::fmt::Display for BindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for BindingError {}

/// Immutable, validated bridge from abstraction channels to causal graph
/// state roles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CategoricalStateBinding {
    source_schema_digest: Digest,
    target_graph_digest: Digest,
    bins: u8,
    patterns: BTreeMap<(TokenId, u8), Vec<Literal>>,
    channel_roles: Vec<TokenId>,
    target_roles: Vec<TokenId>,
    digest: Digest,
}

impl CategoricalStateBinding {
    pub fn new(
        config: &DeterministicAbstractionConfig,
        target_graph: &CanonicalCausalGraph,
        patterns: Vec<SymbolicStatePattern>,
    ) -> Result<Self, BindingError> {
        if patterns.is_empty() {
            return Err(BindingError::EmptyBinding);
        }

        let known_channels: BTreeSet<TokenId> =
            config.ranges().iter().map(|range| range.role()).collect();
        let graph_roles: BTreeSet<TokenId> = target_graph
            .nodes()
            .iter()
            .map(|node| u64::from(node.id))
            .collect();
        let mut canonical = BTreeMap::new();
        for pattern in patterns {
            if !known_channels.contains(&pattern.channel_role) {
                return Err(BindingError::UnknownChannel(pattern.channel_role));
            }
            if pattern.bin >= config.bins() {
                return Err(BindingError::InvalidBin {
                    channel_role: pattern.channel_role,
                    bin: pattern.bin,
                });
            }
            if pattern.literals.is_empty() {
                return Err(BindingError::EmptyPattern {
                    channel_role: pattern.channel_role,
                    bin: pattern.bin,
                });
            }
            for pair in pattern.literals.windows(2) {
                if pair[0].role == pair[1].role {
                    return Err(BindingError::DuplicateTargetRole {
                        channel_role: pattern.channel_role,
                        bin: pattern.bin,
                        role: pair[0].role,
                    });
                }
            }
            if let Some(role) = pattern
                .literals
                .iter()
                .map(|literal| literal.role)
                .find(|role| !graph_roles.contains(role))
            {
                return Err(BindingError::UnknownGraphRole(role));
            }
            let key = (pattern.channel_role, pattern.bin);
            if canonical.insert(key, pattern.literals).is_some() {
                return Err(BindingError::DuplicatePattern {
                    channel_role: key.0,
                    bin: key.1,
                });
            }
        }

        let mut role_owner = BTreeMap::new();
        let mut target_roles = BTreeSet::new();
        for channel_role in &known_channels {
            let mut expected_roles: Option<Vec<TokenId>> = None;
            let mut seen_values: Vec<Vec<bool>> = Vec::new();
            for bin in 0..config.bins() {
                let literals =
                    canonical
                        .get(&(*channel_role, bin))
                        .ok_or(BindingError::MissingPattern {
                            channel_role: *channel_role,
                            bin,
                        })?;
                let roles: Vec<TokenId> = literals.iter().map(|literal| literal.role).collect();
                if let Some(expected) = &expected_roles {
                    if expected != &roles {
                        return Err(BindingError::InconsistentRoleSet {
                            channel_role: *channel_role,
                            bin,
                        });
                    }
                } else {
                    for role in &roles {
                        if let Some(first_channel) = role_owner.insert(*role, *channel_role) {
                            if first_channel != *channel_role {
                                return Err(BindingError::CrossChannelRoleCollision {
                                    role: *role,
                                    first_channel,
                                    second_channel: *channel_role,
                                });
                            }
                        }
                        target_roles.insert(*role);
                    }
                    expected_roles = Some(roles);
                }
                let values: Vec<bool> = literals.iter().map(|literal| literal.value).collect();
                if let Some(first_bin) = seen_values.iter().position(|seen| seen == &values) {
                    return Err(BindingError::AliasedPattern {
                        channel_role: *channel_role,
                        first_bin: first_bin as u8,
                        second_bin: bin,
                    });
                }
                seen_values.push(values);
            }
        }

        if target_roles.len() > config.max_state_literals() {
            return Err(BindingError::StateBudgetExceeded {
                observed: target_roles.len(),
                limit: config.max_state_literals(),
            });
        }
        let channel_roles: Vec<TokenId> = known_channels.into_iter().collect();
        let target_roles: Vec<TokenId> = target_roles.into_iter().collect();
        let digest = binding_digest(
            config.schema_digest(),
            target_graph.digest(),
            config.bins(),
            &canonical,
        );
        Ok(Self {
            source_schema_digest: config.schema_digest(),
            target_graph_digest: target_graph.digest(),
            bins: config.bins(),
            patterns: canonical,
            channel_roles,
            target_roles,
            digest,
        })
    }

    pub fn source_schema_digest(&self) -> Digest {
        self.source_schema_digest
    }
    pub fn target_graph_digest(&self) -> Digest {
        self.target_graph_digest
    }
    pub fn binding_digest(&self) -> Digest {
        self.digest
    }
    pub fn target_roles(&self) -> &[TokenId] {
        &self.target_roles
    }

    pub fn bind(
        &self,
        observation: &SymbolicObservation,
    ) -> Result<BoundSymbolicObservation, BindingError> {
        if observation.schema_digest() != self.source_schema_digest {
            return Err(BindingError::SchemaMismatch {
                expected: self.source_schema_digest,
                observed: observation.schema_digest(),
            });
        }
        if observation.atoms().len() != self.channel_roles.len()
            || !observation
                .atoms()
                .iter()
                .map(|atom| atom.role)
                .eq(self.channel_roles.iter().copied())
        {
            return Err(BindingError::ObservationChannelMismatch);
        }

        let mut literals = Vec::with_capacity(self.target_roles.len());
        for atom in observation.atoms() {
            if atom.bin >= self.bins {
                return Err(BindingError::ObservationBinOutOfRange {
                    channel_role: atom.role,
                    bin: atom.bin,
                });
            }
            let pattern =
                self.patterns
                    .get(&(atom.role, atom.bin))
                    .ok_or(BindingError::MissingPattern {
                        channel_role: atom.role,
                        bin: atom.bin,
                    })?;
            literals.extend(pattern.iter().copied());
        }
        let state = State::new(literals);
        let schema_digest = mix(
            mix(
                mix(FNV_OFFSET, BOUND_SCHEMA_DOMAIN),
                observation.schema_digest(),
            ),
            self.digest,
        );
        let mut abstraction_digest = mix(
            mix(FNV_OFFSET, BOUND_OBSERVATION_DOMAIN),
            observation.abstraction_digest(),
        );
        abstraction_digest = mix(abstraction_digest, self.digest);
        abstraction_digest = mix(abstraction_digest, state.digest());
        Ok(BoundSymbolicObservation {
            schema_digest,
            abstraction_digest,
            binding_digest: self.digest,
            source_abstraction_digest: observation.abstraction_digest(),
            state,
        })
    }
}

fn binding_digest(
    schema_digest: Digest,
    graph_digest: Digest,
    bins: u8,
    patterns: &BTreeMap<(TokenId, u8), Vec<Literal>>,
) -> Digest {
    let mut digest = mix(FNV_OFFSET, BINDING_DOMAIN);
    digest = mix(digest, schema_digest);
    digest = mix(digest, graph_digest);
    digest = mix(digest, u64::from(bins));
    digest = mix(digest, patterns.len() as u64);
    for ((channel_role, bin), literals) in patterns {
        digest = mix(digest, *channel_role);
        digest = mix(digest, u64::from(*bin));
        digest = mix(digest, literals.len() as u64);
        for literal in literals {
            digest = mix(digest, literal.role);
            digest = mix(digest, u64::from(literal.value));
        }
    }
    digest
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundSymbolicObservation {
    schema_digest: Digest,
    abstraction_digest: Digest,
    binding_digest: Digest,
    source_abstraction_digest: Digest,
    state: State,
}

impl BoundSymbolicObservation {
    pub fn schema_digest(&self) -> Digest {
        self.schema_digest
    }
    pub fn abstraction_digest(&self) -> Digest {
        self.abstraction_digest
    }
    pub fn binding_digest(&self) -> Digest {
        self.binding_digest
    }
    pub fn source_abstraction_digest(&self) -> Digest {
        self.source_abstraction_digest
    }
    pub fn state(&self) -> &State {
        &self.state
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CausalCycleClock {
    pub epoch: u64,
    pub episode: u64,
    pub sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CausalCycleIdentity {
    pub context: Option<ContextId>,
    pub clock: CausalCycleClock,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HostAuthorization {
    policy_digest: Digest,
    nonce: u64,
}

impl HostAuthorization {
    pub fn new(policy_digest: Digest, nonce: u64) -> Option<Self> {
        (policy_digest != 0 && nonce != 0).then_some(Self {
            policy_digest,
            nonce,
        })
    }

    pub fn policy_digest(self) -> Digest {
        self.policy_digest
    }
    pub fn nonce(self) -> u64 {
        self.nonce
    }
}

pub trait CausalExecutionPolicy {
    fn authorize(&self, request: &PreparedCompositeExecution) -> Option<HostAuthorization>;
}

impl<F> CausalExecutionPolicy for F
where
    F: Fn(&PreparedCompositeExecution) -> Option<HostAuthorization>,
{
    fn authorize(&self, request: &PreparedCompositeExecution) -> Option<HostAuthorization> {
        self(request)
    }
}

/// Public pre-dispatch view. The executable atomic action sequence is
/// deliberately absent from this type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedCompositeExecution {
    root_id: ModuleId,
    scope: PredictionScope,
    command: u64,
    proof_action: ActionId,
    plan_digest: Digest,
    before_digest: Digest,
    expected_after_digest: Digest,
    commitment_digest: Digest,
    atomic_action_count: usize,
}

impl PreparedCompositeExecution {
    pub fn root_id(&self) -> ModuleId {
        self.root_id
    }
    pub fn scope(&self) -> PredictionScope {
        self.scope
    }
    pub fn command(&self) -> u64 {
        self.command
    }
    pub fn proof_action(&self) -> ActionId {
        self.proof_action
    }
    pub fn plan_digest(&self) -> Digest {
        self.plan_digest
    }
    pub fn before_digest(&self) -> Digest {
        self.before_digest
    }
    pub fn expected_after_digest(&self) -> Digest {
        self.expected_after_digest
    }
    pub fn commitment_digest(&self) -> Digest {
        self.commitment_digest
    }
    pub fn atomic_action_count(&self) -> usize {
        self.atomic_action_count
    }
}

/// The only runtime value that exposes executable atomic actions.
/// It is intentionally non-Clone and can be emitted only once per cycle.
#[derive(Debug, Eq, PartialEq)]
pub struct AuthorizedCompositeDispatch {
    dispatch_digest: Digest,
    commitment_digest: Digest,
    scope: PredictionScope,
    command: u64,
    proof_action: ActionId,
    atomic_actions: Vec<ActionId>,
    host_policy_digest: Digest,
    nonce: u64,
    trust_admission: Option<CausalTrustAdmission>,
}

impl AuthorizedCompositeDispatch {
    pub fn dispatch_digest(&self) -> Digest {
        self.dispatch_digest
    }
    pub fn commitment_digest(&self) -> Digest {
        self.commitment_digest
    }
    pub fn scope(&self) -> PredictionScope {
        self.scope
    }
    pub fn command(&self) -> u64 {
        self.command
    }
    pub fn proof_action(&self) -> ActionId {
        self.proof_action
    }
    pub fn atomic_actions(&self) -> &[ActionId] {
        &self.atomic_actions
    }
    pub fn host_policy_digest(&self) -> Digest {
        self.host_policy_digest
    }
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn trust_admission(&self) -> Option<CausalTrustAdmission> {
        self.trust_admission
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CausalRuntimePhase {
    Idle,
    Prepared,
    AwaitingOutcome,
    Terminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutcomeEvidence {
    Observed {
        schema_digest: Digest,
        abstraction_digest: Digest,
        after_digest: Digest,
    },
    RejectedAbstraction(AbstractionError),
    RejectedBinding(BindingError),
    Missing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedExecutionReceipt {
    prepared: PreparedCompositeExecution,
    dispatch_digest: Digest,
    result: CommitmentResult,
    evidence: OutcomeEvidence,
    before: State,
    observed_after: Option<State>,
    atomic_actions: Vec<ActionId>,
    host_policy_digest: Digest,
    authorization_nonce: u64,
    trust_admission: Option<CausalTrustAdmission>,
}

impl VerifiedExecutionReceipt {
    pub fn prepared(&self) -> &PreparedCompositeExecution {
        &self.prepared
    }
    pub fn dispatch_digest(&self) -> Digest {
        self.dispatch_digest
    }
    pub fn result(&self) -> CommitmentResult {
        self.result
    }
    pub fn evidence(&self) -> &OutcomeEvidence {
        &self.evidence
    }

    /// Exact symbolic state bound before dispatch. This is retained for
    /// audit/export only and grants no execution authority.
    pub fn before(&self) -> &State {
        &self.before
    }

    /// Exact symbolic outcome when observation succeeded. Indeterminate
    /// receipts deliberately carry no fabricated after-state.
    pub fn observed_after(&self) -> Option<&State> {
        self.observed_after.as_ref()
    }

    /// Atomic sequence that was exposed by the consumed one-shot dispatch.
    pub fn atomic_actions(&self) -> &[ActionId] {
        &self.atomic_actions
    }

    pub fn host_policy_digest(&self) -> Digest {
        self.host_policy_digest
    }

    pub fn authorization_nonce(&self) -> u64 {
        self.authorization_nonce
    }

    pub fn trust_admission(&self) -> Option<CausalTrustAdmission> {
        self.trust_admission
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CausalRuntimeError {
    WrongPhase {
        expected: CausalRuntimePhase,
        observed: CausalRuntimePhase,
    },
    ReplayedOrStaleClock {
        previous: CausalCycleClock,
        requested: CausalCycleClock,
    },
    EmptyCommand,
    Abstraction(AbstractionError),
    Binding(BindingError),
    Hierarchy(HierarchyError),
    HierarchyProof(HierarchyProofError),
    PolicyDenied,
    InvalidAuthorization,
    DispatchPlanChanged,
    TrustAdmission(CausalTrustError),
    Proof(ProofError),
}

impl std::fmt::Display for CausalRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CausalRuntimeError {}

impl From<AbstractionError> for CausalRuntimeError {
    fn from(error: AbstractionError) -> Self {
        Self::Abstraction(error)
    }
}

impl From<BindingError> for CausalRuntimeError {
    fn from(error: BindingError) -> Self {
        Self::Binding(error)
    }
}

impl From<HierarchyError> for CausalRuntimeError {
    fn from(error: HierarchyError) -> Self {
        Self::Hierarchy(error)
    }
}

impl From<HierarchyProofError> for CausalRuntimeError {
    fn from(error: HierarchyProofError) -> Self {
        Self::HierarchyProof(error)
    }
}

impl From<CausalTrustError> for CausalRuntimeError {
    fn from(error: CausalTrustError) -> Self {
        Self::TrustAdmission(error)
    }
}

impl From<ProofError> for CausalRuntimeError {
    fn from(error: ProofError) -> Self {
        Self::Proof(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingExecution {
    plan: CompositePlan,
    prepared: PreparedCompositeExecution,
    before: State,
    atomic_actions: Vec<ActionId>,
    commitment: PredictionCommitment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AwaitingExecution {
    pending: PendingExecution,
    permit: ExecutionPermit,
    dispatch_digest: Digest,
    trust_admission: Option<CausalTrustAdmission>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RuntimeState {
    Idle,
    Prepared(PendingExecution),
    Awaiting(AwaitingExecution),
    Terminal(VerifiedExecutionReceipt),
}

/// Single-flight causal runtime. A new cycle cannot begin until the previous
/// terminal receipt is explicitly cleared.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndToEndCausalRuntime {
    abstraction: DeterministicAbstraction,
    binding: CategoricalStateBinding,
    state: RuntimeState,
    last_clock: Option<CausalCycleClock>,
}

impl EndToEndCausalRuntime {
    pub fn new(
        abstraction: DeterministicAbstraction,
        binding: CategoricalStateBinding,
    ) -> Result<Self, CausalRuntimeError> {
        if abstraction.config().schema_digest() != binding.source_schema_digest() {
            return Err(CausalRuntimeError::Binding(BindingError::SchemaMismatch {
                expected: abstraction.config().schema_digest(),
                observed: binding.source_schema_digest(),
            }));
        }
        Ok(Self {
            abstraction,
            binding,
            state: RuntimeState::Idle,
            last_clock: None,
        })
    }

    pub fn phase(&self) -> CausalRuntimePhase {
        match &self.state {
            RuntimeState::Idle => CausalRuntimePhase::Idle,
            RuntimeState::Prepared(_) => CausalRuntimePhase::Prepared,
            RuntimeState::Awaiting(_) => CausalRuntimePhase::AwaitingOutcome,
            RuntimeState::Terminal(_) => CausalRuntimePhase::Terminal,
        }
    }

    pub fn binding(&self) -> &CategoricalStateBinding {
        &self.binding
    }

    pub fn terminal_receipt(&self) -> Option<&VerifiedExecutionReceipt> {
        match &self.state {
            RuntimeState::Terminal(receipt) => Some(receipt),
            _ => None,
        }
    }

    pub fn prepare(
        &mut self,
        hierarchy: &CausalHierarchy,
        plan: &CompositePlan,
        frame: &ContinuousFrame,
        identity: CausalCycleIdentity,
        command: u64,
    ) -> Result<PreparedCompositeExecution, CausalRuntimeError> {
        self.require_phase(CausalRuntimePhase::Idle)?;
        if command == 0 {
            return Err(CausalRuntimeError::EmptyCommand);
        }
        if let Some(previous) = self.last_clock {
            if identity.clock <= previous {
                return Err(CausalRuntimeError::ReplayedOrStaleClock {
                    previous,
                    requested: identity.clock,
                });
            }
        }

        let mut next_abstraction = self.abstraction.clone();
        let symbolic = next_abstraction.abstract_frame(frame)?;
        let bound = self.binding.bind(&symbolic)?;
        let scope = PredictionScope {
            schema_digest: bound.schema_digest(),
            abstraction_digest: bound.abstraction_digest(),
            context: identity.context,
            epoch: identity.clock.epoch,
            episode: identity.clock.episode,
            sequence: identity.clock.sequence,
        };
        let atomic_actions = plan.execution_actions(hierarchy)?;
        let commitment = plan.commit(hierarchy, scope, command, bound.state())?;
        let expected_after_digest = commitment
            .possible_after()
            .first()
            .expect("composite commitment has one bounded outcome")
            .digest();
        let prepared = PreparedCompositeExecution {
            root_id: plan.root_id(),
            scope,
            command,
            proof_action: plan.proof_action(),
            plan_digest: plan.digest(),
            before_digest: bound.state().digest(),
            expected_after_digest,
            commitment_digest: commitment.commitment_digest(),
            atomic_action_count: atomic_actions.len(),
        };
        self.abstraction = next_abstraction;
        self.last_clock = Some(identity.clock);
        self.state = RuntimeState::Prepared(PendingExecution {
            plan: plan.clone(),
            prepared: prepared.clone(),
            before: bound.state().clone(),
            atomic_actions,
            commitment,
        });
        Ok(prepared)
    }

    pub fn authorize_and_dispatch<P: CausalExecutionPolicy>(
        &mut self,
        hierarchy: &CausalHierarchy,
        policy: &P,
    ) -> Result<AuthorizedCompositeDispatch, CausalRuntimeError> {
        let (pending, authorization) = self.validated_dispatch_request(hierarchy, policy)?;
        self.finalize_dispatch(pending, authorization, None)
    }

    /// Cross the host-policy and causal-proof boundaries only while the exact
    /// P8 trust frontier captured after `prepare` is still live.
    ///
    /// Trust is checked before ordinary hierarchy revalidation so a revoked
    /// or reinstalled transferred dependency reports a stale lease and never
    /// reaches proof-permit consumption. The mutable lease is marked consumed
    /// only after both trust and host policy have accepted the request.
    pub fn authorize_and_dispatch_trusted<P: CausalExecutionPolicy>(
        &mut self,
        hierarchy: &CausalHierarchy,
        policy: &P,
        mesh: &CausalTrustMesh,
        lease: &mut CausalExecutionLease,
    ) -> Result<AuthorizedCompositeDispatch, CausalRuntimeError> {
        self.require_phase(CausalRuntimePhase::Prepared)?;
        let RuntimeState::Prepared(pending) = &self.state else {
            unreachable!("phase checked")
        };
        let trust_admission = mesh.validate_execution_lease(
            lease,
            &pending.plan,
            hierarchy,
            pending.prepared.commitment_digest,
        )?;
        let (pending, authorization) = self.validated_dispatch_request(hierarchy, policy)?;
        lease.consume_validated();
        self.finalize_dispatch(pending, authorization, Some(trust_admission))
    }

    fn validated_dispatch_request<P: CausalExecutionPolicy>(
        &self,
        hierarchy: &CausalHierarchy,
        policy: &P,
    ) -> Result<(PendingExecution, HostAuthorization), CausalRuntimeError> {
        self.require_phase(CausalRuntimePhase::Prepared)?;
        let RuntimeState::Prepared(pending) = self.state.clone() else {
            unreachable!("phase checked")
        };
        let current_actions = pending.plan.execution_actions(hierarchy)?;
        if current_actions != pending.atomic_actions
            || pending.plan.digest() != pending.prepared.plan_digest
        {
            return Err(CausalRuntimeError::DispatchPlanChanged);
        }
        let authorization = policy
            .authorize(&pending.prepared)
            .ok_or(CausalRuntimeError::PolicyDenied)?;
        if authorization.policy_digest == 0 || authorization.nonce == 0 {
            return Err(CausalRuntimeError::InvalidAuthorization);
        }
        Ok((pending, authorization))
    }

    fn finalize_dispatch(
        &mut self,
        mut pending: PendingExecution,
        authorization: HostAuthorization,
        trust_admission: Option<CausalTrustAdmission>,
    ) -> Result<AuthorizedCompositeDispatch, CausalRuntimeError> {
        let mut permit = pending
            .commitment
            .issue_permit(authorization.policy_digest, authorization.nonce);
        pending.commitment.consume_permit(
            &mut permit,
            pending.prepared.scope,
            pending.prepared.command,
            pending.prepared.proof_action,
            &pending.before,
        )?;
        let dispatch_digest = composite_dispatch_digest(
            pending.prepared.commitment_digest,
            &pending.atomic_actions,
            authorization.policy_digest,
            authorization.nonce,
        );
        let dispatch = AuthorizedCompositeDispatch {
            dispatch_digest,
            commitment_digest: pending.prepared.commitment_digest,
            scope: pending.prepared.scope,
            command: pending.prepared.command,
            proof_action: pending.prepared.proof_action,
            atomic_actions: pending.atomic_actions.clone(),
            host_policy_digest: authorization.policy_digest,
            nonce: authorization.nonce,
            trust_admission,
        };
        self.state = RuntimeState::Awaiting(AwaitingExecution {
            pending,
            permit,
            dispatch_digest,
            trust_admission,
        });
        Ok(dispatch)
    }

    pub fn resolve_after(
        &mut self,
        frame: &ContinuousFrame,
    ) -> Result<VerifiedExecutionReceipt, CausalRuntimeError> {
        self.require_phase(CausalRuntimePhase::AwaitingOutcome)?;
        let RuntimeState::Awaiting(mut awaiting) = self.state.clone() else {
            unreachable!("phase checked")
        };
        let mut next_abstraction = self.abstraction.clone();
        let symbolic = match next_abstraction.abstract_frame(frame) {
            Ok(symbolic) => symbolic,
            Err(error) => {
                return self
                    .finish_indeterminate(awaiting, OutcomeEvidence::RejectedAbstraction(error))
            }
        };
        let bound = match self.binding.bind(&symbolic) {
            Ok(bound) => bound,
            Err(error) => {
                return self.finish_indeterminate(awaiting, OutcomeEvidence::RejectedBinding(error))
            }
        };
        let result = awaiting.pending.commitment.resolve(
            &awaiting.permit,
            awaiting.pending.prepared.scope,
            awaiting.pending.prepared.command,
            awaiting.pending.prepared.proof_action,
            &awaiting.pending.before,
            bound.state(),
        )?;
        self.abstraction = next_abstraction;
        let host_policy_digest = awaiting.permit.host_policy_digest();
        let authorization_nonce = awaiting.permit.nonce();
        let observed_after = bound.state().clone();
        let receipt = VerifiedExecutionReceipt {
            prepared: awaiting.pending.prepared,
            dispatch_digest: awaiting.dispatch_digest,
            result,
            evidence: OutcomeEvidence::Observed {
                schema_digest: bound.schema_digest(),
                abstraction_digest: bound.abstraction_digest(),
                after_digest: bound.state().digest(),
            },
            before: awaiting.pending.before,
            observed_after: Some(observed_after),
            atomic_actions: awaiting.pending.atomic_actions,
            host_policy_digest,
            authorization_nonce,
            trust_admission: awaiting.trust_admission,
        };
        self.state = RuntimeState::Terminal(receipt.clone());
        Ok(receipt)
    }

    pub fn resolve_without_observation(
        &mut self,
    ) -> Result<VerifiedExecutionReceipt, CausalRuntimeError> {
        self.require_phase(CausalRuntimePhase::AwaitingOutcome)?;
        let RuntimeState::Awaiting(awaiting) = self.state.clone() else {
            unreachable!("phase checked")
        };
        self.finish_indeterminate(awaiting, OutcomeEvidence::Missing)
    }

    /// Discard a non-dispatched commitment. Its clock remains consumed so the
    /// same scope cannot be replayed later.
    pub fn discard_prepared(&mut self) -> Result<(), CausalRuntimeError> {
        self.require_phase(CausalRuntimePhase::Prepared)?;
        self.state = RuntimeState::Idle;
        Ok(())
    }

    pub fn clear_terminal(&mut self) -> Result<(), CausalRuntimeError> {
        self.require_phase(CausalRuntimePhase::Terminal)?;
        self.state = RuntimeState::Idle;
        Ok(())
    }

    fn finish_indeterminate(
        &mut self,
        mut awaiting: AwaitingExecution,
        evidence: OutcomeEvidence,
    ) -> Result<VerifiedExecutionReceipt, CausalRuntimeError> {
        let result = awaiting.pending.commitment.resolve_indeterminate(
            &awaiting.permit,
            awaiting.pending.prepared.scope,
            awaiting.pending.prepared.command,
            awaiting.pending.prepared.proof_action,
            &awaiting.pending.before,
        )?;
        let host_policy_digest = awaiting.permit.host_policy_digest();
        let authorization_nonce = awaiting.permit.nonce();
        let receipt = VerifiedExecutionReceipt {
            prepared: awaiting.pending.prepared,
            dispatch_digest: awaiting.dispatch_digest,
            result,
            evidence,
            before: awaiting.pending.before,
            observed_after: None,
            atomic_actions: awaiting.pending.atomic_actions,
            host_policy_digest,
            authorization_nonce,
            trust_admission: awaiting.trust_admission,
        };
        self.state = RuntimeState::Terminal(receipt.clone());
        Ok(receipt)
    }

    fn require_phase(&self, expected: CausalRuntimePhase) -> Result<(), CausalRuntimeError> {
        let observed = self.phase();
        if observed != expected {
            return Err(CausalRuntimeError::WrongPhase { expected, observed });
        }
        Ok(())
    }
}

pub(crate) fn composite_dispatch_digest(
    commitment_digest: Digest,
    actions: &[ActionId],
    host_policy_digest: Digest,
    authorization_nonce: u64,
) -> Digest {
    let mut digest = mix(FNV_OFFSET, DISPATCH_DOMAIN);
    digest = mix(digest, commitment_digest);
    digest = mix(digest, host_policy_digest);
    digest = mix(digest, authorization_nonce);
    digest = mix(digest, actions.len() as u64);
    for action in actions {
        digest = mix(digest, *action);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal_knowledge_sync::{JsonlKnowledgeSync, KnowledgeSyncCodec, SyncLimits};
    use crate::cross_domain_causal_core::{
        ActionRule, CausalEdge, CausalNode, EdgeKind, EvidenceLedger, MappingBudget, NodeKind,
        TraceKind, TransitionExample,
    };
    use crate::deterministic_abstraction::{ChannelRange, FixedScalar};
    use crate::hierarchical_causal_graph::AtomicCausalModule;

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
        target: &CanonicalCausalGraph,
    ) -> CategoricalStateBinding {
        let mut patterns = Vec::new();
        for (channel, role) in [(100, 20), (200, 21), (300, 22)] {
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
        CategoricalStateBinding::new(config, target, patterns).expect("binding")
    }

    fn frame(a: i64, b: i64, c: i64) -> ContinuousFrame {
        ContinuousFrame::new(vec![
            FixedScalar::from_micros(300, c),
            FixedScalar::from_micros(100, a),
            FixedScalar::from_micros(200, b),
        ])
        .unwrap()
    }

    fn identity(sequence: u64) -> CausalCycleIdentity {
        CausalCycleIdentity {
            context: Some(ContextId(9)),
            clock: CausalCycleClock {
                epoch: 1,
                episode: 1,
                sequence,
            },
        }
    }

    fn allow(_: &PreparedCompositeExecution) -> Option<HostAuthorization> {
        HostAuthorization::new(700, 1)
    }

    fn local_hierarchy() -> (CausalHierarchy, CompositePlan) {
        let target = graph(20);
        let mut hierarchy = CausalHierarchy::default();
        hierarchy
            .add_atomic(
                AtomicCausalModule::new(
                    7,
                    target,
                    ActionRule {
                        action: 199,
                        guards: vec![
                            Literal::new(20, true),
                            Literal::new(21, true),
                            Literal::new(22, false),
                        ],
                        effects: vec![Literal::new(22, true)],
                    },
                )
                .unwrap(),
            )
            .unwrap();
        let plan = hierarchy.compose(100, vec![7]).unwrap();
        (hierarchy, plan)
    }

    fn synced_hierarchy() -> (CanonicalCausalGraph, CausalHierarchy, CompositePlan) {
        let source = graph(10);
        let target = graph(20);
        let before = State::new([
            Literal::new(10, true),
            Literal::new(11, true),
            Literal::new(12, false),
        ]);
        let after = before.with(Literal::new(12, true));
        let evidence = vec![
            TransitionExample::new(1, 99, before.clone(), after.clone()),
            TransitionExample::new(2, 99, before, after),
        ];
        let mut ledger = EvidenceLedger::new();
        ledger.append(0, TraceKind::Observe, 11);
        ledger.append(0, TraceKind::Abstract, 22);
        let codec = JsonlKnowledgeSync;
        let jsonl = codec
            .export_jsonl(
                7,
                3,
                42,
                &source,
                99,
                &evidence,
                &ledger,
                SyncLimits::default(),
            )
            .unwrap();
        let candidate = codec
            .import_jsonl(
                &jsonl,
                &target,
                199,
                MappingBudget::new(128).unwrap(),
                SyncLimits::default(),
            )
            .unwrap();
        let mut hierarchy = CausalHierarchy::default();
        hierarchy.add_synced_atomic(7, &candidate, &target).unwrap();
        let plan = hierarchy.compose(100, vec![7]).unwrap();
        (target, hierarchy, plan)
    }

    #[test]
    fn synced_knowledge_crosses_continuous_runtime_without_target_training() {
        let (target, hierarchy, plan) = synced_hierarchy();
        let config = config();
        let binding = binding(&config, &target);
        let mut runtime =
            EndToEndCausalRuntime::new(DeterministicAbstraction::new(config), binding).unwrap();
        let prepared = runtime
            .prepare(
                &hierarchy,
                &plan,
                &frame(900_000, 900_000, 100_000),
                identity(1),
                500,
            )
            .unwrap();
        assert_eq!(prepared.atomic_action_count(), 1);
        assert_eq!(runtime.phase(), CausalRuntimePhase::Prepared);
        let dispatch = runtime.authorize_and_dispatch(&hierarchy, &allow).unwrap();
        assert_eq!(dispatch.atomic_actions(), &[199]);
        assert_eq!(runtime.phase(), CausalRuntimePhase::AwaitingOutcome);
        let receipt = runtime
            .resolve_after(&frame(900_000, 900_000, 900_000))
            .unwrap();
        assert_eq!(receipt.result(), CommitmentResult::Confirmed);
        assert_eq!(runtime.phase(), CausalRuntimePhase::Terminal);
    }

    #[test]
    fn denied_policy_and_stale_hierarchy_never_dispatch() {
        let target = graph(20);
        let config = config();
        let binding = binding(&config, &target);
        let (mut hierarchy, plan) = local_hierarchy();
        let mut runtime =
            EndToEndCausalRuntime::new(DeterministicAbstraction::new(config), binding).unwrap();
        runtime
            .prepare(
                &hierarchy,
                &plan,
                &frame(900_000, 900_000, 100_000),
                identity(1),
                500,
            )
            .unwrap();
        let deny = |_: &PreparedCompositeExecution| None;
        assert_eq!(
            runtime.authorize_and_dispatch(&hierarchy, &deny),
            Err(CausalRuntimeError::PolicyDenied)
        );
        assert_eq!(runtime.phase(), CausalRuntimePhase::Prepared);

        hierarchy
            .replace_atomic(
                AtomicCausalModule::new(
                    7,
                    target,
                    ActionRule {
                        action: 200,
                        guards: vec![Literal::new(20, true)],
                        effects: vec![Literal::new(22, false)],
                    },
                )
                .unwrap(),
            )
            .unwrap();
        assert!(matches!(
            runtime.authorize_and_dispatch(&hierarchy, &allow),
            Err(CausalRuntimeError::Hierarchy(HierarchyError::StalePlan {
                root: 100
            }))
        ));
        assert_eq!(runtime.phase(), CausalRuntimePhase::Prepared);
    }

    #[test]
    fn malformed_outcome_is_terminal_indeterminate_and_cannot_replay() {
        let target = graph(20);
        let config = config();
        let binding = binding(&config, &target);
        let (hierarchy, plan) = local_hierarchy();
        let mut runtime =
            EndToEndCausalRuntime::new(DeterministicAbstraction::new(config), binding).unwrap();
        runtime
            .prepare(
                &hierarchy,
                &plan,
                &frame(900_000, 900_000, 100_000),
                identity(1),
                500,
            )
            .unwrap();
        runtime.authorize_and_dispatch(&hierarchy, &allow).unwrap();
        let incomplete =
            ContinuousFrame::new(vec![FixedScalar::from_micros(100, 900_000)]).unwrap();
        let receipt = runtime.resolve_after(&incomplete).unwrap();
        assert_eq!(receipt.result(), CommitmentResult::Indeterminate);
        assert!(matches!(
            receipt.evidence(),
            OutcomeEvidence::RejectedAbstraction(AbstractionError::MissingRole(_))
        ));
        assert!(matches!(
            runtime.resolve_after(&frame(900_000, 900_000, 900_000)),
            Err(CausalRuntimeError::WrongPhase {
                expected: CausalRuntimePhase::AwaitingOutcome,
                observed: CausalRuntimePhase::Terminal,
            })
        ));
    }

    #[test]
    fn contradiction_is_terminal_and_scope_clock_cannot_be_replayed() {
        let target = graph(20);
        let config = config();
        let binding = binding(&config, &target);
        let (hierarchy, plan) = local_hierarchy();
        let mut runtime =
            EndToEndCausalRuntime::new(DeterministicAbstraction::new(config), binding).unwrap();
        runtime
            .prepare(
                &hierarchy,
                &plan,
                &frame(900_000, 900_000, 100_000),
                identity(1),
                500,
            )
            .unwrap();
        runtime.authorize_and_dispatch(&hierarchy, &allow).unwrap();
        let receipt = runtime
            .resolve_after(&frame(900_000, 900_000, 100_000))
            .unwrap();
        assert_eq!(receipt.result(), CommitmentResult::Contradicted);
        runtime.clear_terminal().unwrap();
        assert_eq!(
            runtime.prepare(
                &hierarchy,
                &plan,
                &frame(900_000, 900_000, 100_000),
                identity(1),
                501,
            ),
            Err(CausalRuntimeError::ReplayedOrStaleClock {
                previous: identity(1).clock,
                requested: identity(1).clock,
            })
        );
    }

    #[test]
    fn incomplete_or_aliased_bindings_fail_closed() {
        let target = graph(20);
        let config = config();
        let incomplete = vec![
            SymbolicStatePattern::new(100, 0, vec![Literal::new(20, false)]),
            SymbolicStatePattern::new(100, 1, vec![Literal::new(20, true)]),
        ];
        assert!(matches!(
            CategoricalStateBinding::new(&config, &target, incomplete),
            Err(BindingError::MissingPattern {
                channel_role: 200,
                bin: 0,
            })
        ));

        let mut aliased = Vec::new();
        for (channel, role) in [(100, 20), (200, 21), (300, 22)] {
            aliased.push(SymbolicStatePattern::new(
                channel,
                0,
                vec![Literal::new(role, false)],
            ));
            aliased.push(SymbolicStatePattern::new(
                channel,
                1,
                vec![Literal::new(role, channel != 200)],
            ));
        }
        assert!(matches!(
            CategoricalStateBinding::new(&config, &target, aliased),
            Err(BindingError::AliasedPattern {
                channel_role: 200,
                first_bin: 0,
                second_bin: 1,
            })
        ));
    }
}
