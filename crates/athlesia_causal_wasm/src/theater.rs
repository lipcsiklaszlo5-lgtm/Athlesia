//! P9 presentation adapter. All authorization, mapping, receipts and lease
//! invalidation are delegated to the existing kernel. The source plant is an
//! explicitly simulated actuator; the browser target is a real IndexedDB write.
//! Source effect learning and host-provided interlocks are separate contracts.
use athlesia_integrated_cognitive_agent::cross_domain_causal_core::ActionRule;
use athlesia_integrated_cognitive_agent::{
    causal_execution_sync::{
        ExecutionKnowledgeReceiver, ExecutionKnowledgeSigner, ExecutionKnowledgeSync,
        ExecutionKnowledgeVerifier, ExecutionSyncLimits,
    },
    causal_knowledge_sync::SyncLimits,
    causal_proof::CommitmentResult,
    causal_trust_mesh::{
        CausalClaim, CausalClaimStatus, CausalExecutionLease, CausalRevocationSync,
        CausalTrustError, CausalTrustMesh, CausalTrustPolicy,
    },
    cross_domain_causal_core::{
        CanonicalCausalGraph, CausalEdge, CausalLearner, CausalNode, EdgeKind, Literal,
        MappingBudget, NodeKind, State, TransitionExample,
    },
    deterministic_abstraction::{
        ChannelRange, ContinuousFrame, DeterministicAbstraction, DeterministicAbstractionConfig,
        FixedScalar,
    },
    end_to_end_causal_runtime::{
        CategoricalStateBinding, CausalCycleClock, CausalCycleIdentity, CausalRuntimeError,
        EndToEndCausalRuntime, HostAuthorization, SymbolicStatePattern, VerifiedExecutionReceipt,
    },
    hierarchical_causal_graph::{AtomicCausalModule, CausalHierarchy, CompositePlan},
};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

const ALGORITHM: u16 = 1; // Deployment-local identifier: Ed25519, never a test hash.
const ACTION: u64 = 99;
const TARGET_ACTION: u64 = 199;
const MODULE: u32 = 8;
const ROOT: u32 = 101;

fn check<T, E: std::fmt::Debug>(value: Result<T, E>) -> Result<T, String> {
    value.map_err(|error| format!("{error:?}"))
}
fn require(condition: bool, why: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(why.to_owned())
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn decode_key(text: &str) -> Result<[u8; 32], String> {
    require(
        text.len() == 64 && text.is_ascii(),
        "Expected 32-byte hexadecimal key",
    )?;
    let mut out = [0; 32];
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16)
            .map_err(|_| "Invalid hexadecimal key")?;
    }
    Ok(out)
}
fn base(seed: u32, domain: u32) -> u32 {
    (seed % 100_000) * 10 + 10 + domain * 1_000_010
}
fn graph(offset: u32) -> Result<CanonicalCausalGraph, String> {
    check(CanonicalCausalGraph::new(
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
            CausalNode {
                id: offset + 3,
                kind: NodeKind::StateField,
                type_tag: 3,
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
                kind: EdgeKind::Requires,
            },
            CausalEdge {
                from: offset,
                to: offset + 3,
                kind: EdgeKind::Writes,
            },
        ],
    ))
}
fn config() -> Result<DeterministicAbstractionConfig, String> {
    check(DeterministicAbstractionConfig::new(
        vec![
            check(ChannelRange::new(100, 0, 1_000_000))?,
            check(ChannelRange::new(200, 0, 1_000_000))?,
            check(ChannelRange::new(300, 0, 1_000_000))?,
        ],
        2,
        0,
        2,
        16,
        77,
    ))
}
fn frame(output: bool) -> Result<ContinuousFrame, String> {
    check(ContinuousFrame::new(vec![
        FixedScalar::from_micros(100, 900_000),
        FixedScalar::from_micros(200, 900_000),
        FixedScalar::from_micros(300, if output { 900_000 } else { 100_000 }),
    ]))
}
fn runtime(offset: u32) -> Result<EndToEndCausalRuntime, String> {
    let config = config()?;
    let mut patterns = Vec::new();
    for (channel, role) in [(100, offset + 1), (200, offset + 2), (300, offset + 3)] {
        for bin in 0..2 {
            patterns.push(SymbolicStatePattern::new(
                channel,
                bin,
                vec![Literal::new(u64::from(role), bin == 1)],
            ));
        }
    }
    let binding = check(CategoricalStateBinding::new(
        &config,
        &graph(offset)?,
        patterns,
    ))?;
    check(EndToEndCausalRuntime::new(
        DeterministicAbstraction::new(config),
        binding,
    ))
}
fn clock() -> CausalCycleIdentity {
    CausalCycleIdentity {
        context: None,
        clock: CausalCycleClock {
            epoch: 1,
            episode: 1,
            sequence: 1,
        },
    }
}

struct Key {
    id: u64,
    signing: SigningKey,
}
impl ExecutionKnowledgeSigner for Key {
    fn algorithm_id(&self) -> u16 {
        ALGORITHM
    }
    fn key_id(&self) -> u64 {
        self.id
    }
    fn sign(&self, message: &[u8]) -> Option<Vec<u8>> {
        Some(self.signing.sign(message).to_bytes().to_vec())
    }
}
struct Roots(Vec<(u64, VerifyingKey)>);
impl Roots {
    fn new(a: &[u8], b: &[u8]) -> Result<Self, String> {
        let a = <&[u8; 32]>::try_from(a).map_err(|_| "Bad public key A")?;
        let b = <&[u8; 32]>::try_from(b).map_err(|_| "Bad public key B")?;
        require(
            a != b,
            "Distinct public keys required for the two local origins",
        )?;
        Ok(Self(vec![
            (7001, check(VerifyingKey::from_bytes(a))?),
            (7002, check(VerifyingKey::from_bytes(b))?),
        ]))
    }
}
impl ExecutionKnowledgeVerifier for Roots {
    fn verify(&self, algorithm: u16, id: u64, message: &[u8], bytes: &[u8]) -> bool {
        if algorithm != ALGORITHM {
            return false;
        }
        let Ok(signature) = Signature::try_from(bytes) else {
            return false;
        };
        self.0
            .iter()
            .any(|(key_id, key)| *key_id == id && key.verify_strict(message, &signature).is_ok())
    }
}

// Environment-owned software-plant transition, separate from the learner.
fn simulated_actuator(
    before: &State,
    output_role: u64,
    action: u64,
    jammed: bool,
) -> Result<State, String> {
    require(action == ACTION, "Unknown software-plant command")?;
    Ok(if jammed {
        before.clone()
    } else {
        before.with(Literal::new(output_role, true))
    })
}
fn source_trial(
    offset: u32,
    source: &CanonicalCausalGraph,
    rule: &ActionRule,
    epoch: u64,
    jammed: bool,
) -> Result<VerifiedExecutionReceipt, String> {
    let mut hierarchy = CausalHierarchy::default();
    check(hierarchy.add_atomic(check(AtomicCausalModule::new(
        7,
        source.clone(),
        rule.clone(),
    ))?))?;
    let plan = check(hierarchy.compose(100, vec![7]))?;
    let mut runtime = runtime(offset)?;
    let mut identity = clock();
    identity.clock.epoch = epoch;
    check(runtime.prepare(&hierarchy, &plan, &frame(false)?, identity, 500))?;
    let dispatch = check(
        runtime.authorize_and_dispatch(&hierarchy, &|_: &_| HostAuthorization::new(700, epoch)),
    )?;
    require(
        dispatch.atomic_actions() == [ACTION],
        "Unexpected source action",
    )?;
    let before = State::new([
        Literal::new(u64::from(offset + 1), true),
        Literal::new(u64::from(offset + 2), true),
        Literal::new(u64::from(offset + 3), false),
    ]);
    let observed = simulated_actuator(
        &before,
        u64::from(offset + 3),
        dispatch.atomic_actions()[0],
        jammed,
    )?;
    check(
        runtime.resolve_after(&frame(
            observed
                .get(u64::from(offset + 3))
                .ok_or("No observed actuator output")?,
        )?),
    )
}

/// A local demonstration origin. Its private key is generated by the browser
/// CSPRNG and held by an origin Worker, never supplied to the receiver.
pub struct TheaterOrigin {
    seed: u32,
    key: Key,
    source: CanonicalCausalGraph,
    receipts: Vec<VerifiedExecutionReceipt>,
    rule: ActionRule,
    jammed: bool,
    structure: String,
}
impl TheaterOrigin {
    pub fn new(seed: u32, key_id: u64, secret: &[u8]) -> Result<Self, String> {
        require(key_id == 7001 || key_id == 7002, "Unknown local origin")?;
        let secret = <&[u8; 32]>::try_from(secret).map_err(|_| "Bad signing key")?;
        let offset = base(seed, 0);
        let source = graph(offset)?;
        let mut examples = Vec::new();
        // Four executed source-plant interventions. The simulated actuator sets
        // the output. The learner sees before/after states, not this mechanism.
        for n in 0..4 {
            let state = |output| {
                State::new([
                    Literal::new(u64::from(offset + 1), n & 1 != 0),
                    Literal::new(u64::from(offset + 2), n & 2 != 0),
                    Literal::new(u64::from(offset + 3), output),
                ])
            };
            let before = state(false);
            let observed = simulated_actuator(&before, u64::from(offset + 3), ACTION, false)?;
            examples.push(TransitionExample::new(n + 1, ACTION, before, observed));
        }
        let mut learner = CausalLearner::new();
        check(learner.fit(&examples))?;
        let rule = learner.rule().cloned().ok_or("No unique source effect")?;
        require(
            rule.guards.is_empty() && rule.effects.len() == 1,
            "Unexpected learned effect",
        )?;
        let receipt = source_trial(offset, &source, &rule, 1, false)?;
        require(
            receipt.result() == CommitmentResult::Confirmed,
            "Source effect not confirmed",
        )?;
        Ok(Self {
            seed,
            key: Key {
                id: key_id,
                signing: SigningKey::from_bytes(secret),
            },
            structure: source.digest().to_string(),
            source,
            receipts: vec![receipt],
            rule,
            jammed: false,
        })
    }
    pub fn public_key(&self) -> String {
        hex(&self.key.signing.verifying_key().to_bytes())
    }
    pub fn structure(&self) -> &str {
        &self.structure
    }
    pub fn bundle(&self, sequence: u64) -> Result<String, String> {
        check(ExecutionKnowledgeSync.export_jsonl(
            sequence,
            42,
            &self.source,
            &self.receipts,
            &self.key,
            SyncLimits::default(),
            ExecutionSyncLimits::default(),
        ))
    }
    pub fn revocation(&self, sequence: u64) -> Result<String, String> {
        let key = self.key.signing.verifying_key();
        let roots = Roots(vec![(self.key.id, key)]);
        let mut receiver = check(ExecutionKnowledgeReceiver::new(2))?;
        let knowledge = check(receiver.import_jsonl(
            &self.bundle(1)?,
            &graph(base(self.seed, 1))?,
            TARGET_ACTION,
            MappingBudget::new(128).ok_or("Invalid mapping budget")?,
            &roots,
            SyncLimits::default(),
            ExecutionSyncLimits::default(),
        ))?;
        let claim = check(CausalClaim::from_authenticated(&knowledge))?;
        check(CausalRevocationSync.export_jsonl(
            sequence,
            &claim,
            902,
            &self.key,
            ExecutionSyncLimits::default(),
        ))
    }
    pub fn sabotage(&mut self) -> Result<String, String> {
        require(!self.jammed, "Software plant is already jammed")?;
        self.jammed = true;
        let receipt = source_trial(base(self.seed, 0), &self.source, &self.rule, 2, true)?;
        require(
            receipt.result() == CommitmentResult::Contradicted,
            "Fault trial was not a contradiction",
        )?;
        Ok(json!({"source_trial_result":"CONTRADICTION", "environment":"software plant",
            "source_fault_dispatch":receipt.dispatch_digest().to_string(), "revocation":self.revocation(2)?}).to_string())
    }
    pub fn restore(&mut self) -> Result<String, String> {
        require(self.jammed, "Restore requires a prior observed fault")?;
        let receipt = source_trial(base(self.seed, 0), &self.source, &self.rule, 3, false)?;
        require(
            receipt.result() == CommitmentResult::Confirmed,
            "Restored source trial was not confirmed",
        )?;
        self.receipts = vec![receipt];
        self.jammed = false;
        self.bundle(3)
    }
}

pub struct TheaterReceiver {
    seed: u32,
    roots: Roots,
    graph: CanonicalCausalGraph,
    mesh: CausalTrustMesh,
    hierarchy: CausalHierarchy,
    receiver: ExecutionKnowledgeReceiver,
    runtime: EndToEndCausalRuntime,
    plan: Option<CompositePlan>,
    lease: Option<CausalExecutionLease>,
    claim: Option<CausalClaim>,
    structure: String,
    phase: &'static str,
    prepared: String,
    dispatches: u32,
    protocol_verified: bool,
}
impl TheaterReceiver {
    pub fn new(seed: u32, a: &[u8], b: &[u8]) -> Result<Self, String> {
        Ok(Self {
            seed,
            roots: Roots::new(a, b)?,
            graph: graph(base(seed, 1))?,
            mesh: check(CausalTrustMesh::new(
                CausalTrustPolicy::new(2, 8, 4, 8).ok_or("Invalid trust policy")?,
                [7001, 7002],
            ))?,
            hierarchy: CausalHierarchy::default(),
            receiver: check(ExecutionKnowledgeReceiver::new(8))?,
            runtime: runtime(base(seed, 1))?,
            plan: None,
            lease: None,
            claim: None,
            structure: graph(base(seed, 0))?.digest().to_string(),
            phase: "EMPTY",
            prepared: String::new(),
            dispatches: 0,
            protocol_verified: false,
        })
    }
    fn accept(&mut self, bundle: &str) -> Result<(), String> {
        let knowledge = check(self.receiver.import_jsonl(
            bundle,
            &self.graph,
            TARGET_ACTION,
            MappingBudget::new(128).ok_or("Invalid mapping budget")?,
            &self.roots,
            SyncLimits::default(),
            ExecutionSyncLimits::default(),
        ))?;
        require(
            knowledge.candidate().source_graph_digest().to_string() == self.structure,
            "The frozen source graph changed",
        )?;
        let claim = check(CausalClaim::from_authenticated(&knowledge))?;
        if let Some(existing) = &self.claim {
            require(existing == &claim, "Conflicting mapped source claim")?;
        }
        self.claim = Some(claim);
        check(
            self.mesh
                .observe_authenticated(knowledge, &mut self.hierarchy),
        )?;
        Ok(())
    }
    fn prepare(&mut self) -> Result<(), String> {
        let scope = self.claim.as_ref().ok_or("No claim")?.scope();
        require(
            matches!(self.mesh.status(scope), CausalClaimStatus::Admitted { .. }),
            "No two-origin quorum",
        )?;
        check(
            self.mesh
                .install_admitted_atomic(scope, &mut self.hierarchy, MODULE, &self.graph),
        )?;
        let plan = check(self.hierarchy.compose(ROOT, vec![MODULE]))?;
        let prepared =
            check(
                self.runtime
                    .prepare(&self.hierarchy, &plan, &frame(false)?, clock(), 500),
            )?;
        self.prepared = prepared.commitment_digest().to_string();
        self.lease = Some(check(self.mesh.issue_execution_lease(
            &plan,
            &self.hierarchy,
            prepared.commitment_digest(),
        ))?);
        self.plan = Some(plan);
        Ok(())
    }
    fn envelope(&self, status: &str, event: &str) -> Value {
        json!({"schema":"athlesia/causal-theater/1", "engine":"wasm", "status":status,
            "structure_digest":self.structure, "evidence":[{"event":event,
                "boundary":"authorize_and_dispatch_trusted / CausalTrustMesh",
                "receiver_dispatches":self.dispatches}], "phase":self.phase})
    }
    pub fn step(&mut self, request: &str) -> Result<Value, String> {
        require(request.len() <= 262_144, "Request budget exceeded")?;
        let request: Value = check(serde_json::from_str(request))?;
        let op = request["op"].as_str().ok_or("Missing operation")?;
        let text = |field: &str| {
            request[field]
                .as_str()
                .ok_or_else(|| format!("Missing {field}"))
        };
        match op {
            "learn" => {
                require(
                    self.phase == "EMPTY",
                    "Source may be learned once per receiver",
                )?;
                self.accept(text("a")?)?;
                self.accept(text("b")?)?;
                self.prepare()?;
                self.phase = "LEARNED";
                self.protocol_verified = true;
                let mut out =
                    self.envelope("LEARNED", "TWO_ED25519_ORIGINS_IMPORTED_AND_LEASE_PREPARED");
                out["model_id"] = json!(format!("source-effect-{}", self.seed));
                out["source_interventions"] = json!(4);
                out["neural_weights"] = json!(0);
                out["guards"] = json!([]);
                out["host_policy_guards"] = json!(["resource.available", "interlock.closed"]);
                out["rule_scope"] = json!("observed source effect; interlocks are explicit host policy, not learned guards");
                out["kernel_us"] = Value::Null;
                out["protocol_verified"] = json!(true);
                Ok(out)
            }
            "sabotage" => {
                require(
                    self.phase == "LEARNED",
                    "Sabotage requires an exact prepared lease",
                )?;
                let scope = self.claim.as_ref().ok_or("No claim")?.scope();
                let before = self.mesh.scope_epoch(scope).ok_or("No scope epoch")?;
                check(self.mesh.apply_revocation_jsonl(
                    text("revocation")?,
                    &mut self.hierarchy,
                    &self.roots,
                    ExecutionSyncLimits::default(),
                ))?;
                let policy = |_: &_| HostAuthorization::new(700, 1);
                let result = self.runtime.authorize_and_dispatch_trusted(
                    &self.hierarchy,
                    &policy,
                    &self.mesh,
                    self.lease.as_mut().ok_or("Missing lease")?,
                );
                require(
                    matches!(
                        result,
                        Err(CausalRuntimeError::TrustAdmission(
                            CausalTrustError::StaleExecutionLease
                        ))
                    ),
                    "The revoked lease did not fail at the P9 gate",
                )?;
                self.phase = "BLOCKED";
                let mut out = self.envelope(
                    "ABSTAIN",
                    "RUST_P9_REJECTED_STALE_LEASE_BEFORE_EXPOSING_ACTIONS",
                );
                out["lease_revoked"] = json!(true);
                out["dispatch_issued"] = json!(false);
                out["reason"] = json!("STALE_EXECUTION_LEASE");
                out["prepared_commitment"] = json!(self.prepared);
                out["epoch_before"] = json!(before);
                out["epoch_after"] = json!(self.mesh.scope_epoch(scope));
                out["receiver_dispatches"] = json!(self.dispatches);
                Ok(out)
            }
            "remap" => {
                require(
                    self.phase == "BLOCKED",
                    "Recovery requires revoked authority",
                )?;
                self.accept(text("a")?)?;
                let scope = self.claim.as_ref().ok_or("No claim")?.scope();
                check(self.mesh.install_admitted_atomic(
                    scope,
                    &mut self.hierarchy,
                    MODULE,
                    &self.graph,
                ))?;
                let plan = self.plan.as_ref().ok_or("No plan")?;
                let commitment = self
                    .prepared
                    .parse()
                    .map_err(|_| "Bad internal commitment")?;
                require(
                    self.mesh.validate_execution_lease(
                        self.lease.as_ref().ok_or("No old lease")?,
                        plan,
                        &self.hierarchy,
                        commitment,
                    ) == Err(CausalTrustError::StaleExecutionLease),
                    "The old lease unexpectedly revived",
                )?;
                self.lease = Some(check(self.mesh.issue_execution_lease(
                    plan,
                    &self.hierarchy,
                    commitment,
                ))?);
                self.phase = "RECOVERED";
                let mut out =
                    self.envelope("REVALIDATED", "FRESH_LEASE_ISSUED_OLD_LEASE_STILL_STALE");
                out["fresh_lease"] = json!(true);
                out["old_lease_valid"] = json!(false);
                out["dispatch_issued"] = json!(false);
                Ok(out)
            }
            "alien" => {
                require(
                    self.phase == "RECOVERED",
                    "Target domain must follow source revalidation",
                )?;
                self.graph = graph(base(self.seed, 2))?;
                self.runtime = runtime(base(self.seed, 2))?;
                self.hierarchy = CausalHierarchy::default();
                self.plan = None;
                self.lease = None;
                self.claim = None;
                self.mesh = check(CausalTrustMesh::new(
                    CausalTrustPolicy::new(2, 8, 4, 8).ok_or("Invalid trust policy")?,
                    [7001, 7002],
                ))?;
                self.receiver = check(ExecutionKnowledgeReceiver::new(8))?;
                self.accept(text("a")?)?;
                self.accept(text("b")?)?;
                self.prepare()?;
                self.phase = "ALIEN";
                let mut out = self.envelope(
                    "DOMAIN_READY",
                    "CKS_MAPPED_BY_UNIQUE_TYPED_GRAPH_WITH_ZERO_TARGET_TRAINING",
                );
                out["target_training_updates"] = json!(0);
                out["mapping_policy"] = json!("EXPLICIT_TYPED_ROLES");
                out["mapping"] = json!([
                    {"from":"Resource", "to":"Replica", "role":"guard.resource"},
                    {"from":"Valve", "to":"Quorum", "role":"guard.interlock"},
                    {"from":"Door", "to":"Commit", "role":"effect.target"}]);
                Ok(out)
            }
            "dispatch" => {
                require(
                    self.phase == "ALIEN",
                    "Target dispatch requires a fresh target lease",
                )?;
                let allowed = request["host_allows"] == true;
                let policy = |_: &_| {
                    if allowed {
                        HostAuthorization::new(700, 1)
                    } else {
                        None
                    }
                };
                let dispatch = check(self.runtime.authorize_and_dispatch_trusted(
                    &self.hierarchy,
                    &policy,
                    &self.mesh,
                    self.lease.as_mut().ok_or("No lease")?,
                ))?;
                require(dispatch.trust_admission().is_some(), "Missing P9 admission")?;
                self.dispatches += 1;
                self.phase = "AWAITING_DATABASE";
                Ok(
                    json!({"status":"AUTHORIZED", "actions":dispatch.atomic_actions(),
                    "dispatch_digest":dispatch.dispatch_digest().to_string(),
                    "commitment_digest":dispatch.commitment_digest().to_string(),
                    "lease_consumed":self.lease.as_ref().is_some_and(|lease| lease.consumed())}),
                )
            }
            "missing" => {
                require(
                    self.phase == "AWAITING_DATABASE",
                    "No outstanding authorized operation",
                )?;
                let receipt = check(self.runtime.resolve_without_observation())?;
                self.phase = "TERMINAL";
                require(
                    receipt.result() == CommitmentResult::Indeterminate,
                    "Missing outcome was not indeterminate",
                )?;
                Ok(self.envelope("ABSTAIN", "P9_OUTCOME_INDETERMINATE_NO_RETRY"))
            }
            "settle" => {
                require(
                    self.phase == "AWAITING_DATABASE",
                    "No outstanding authorized database operation",
                )?;
                let observed = request["observed"]
                    .as_bool()
                    .ok_or("Observed database state is required")?;
                let receipt = check(self.runtime.resolve_after(&frame(observed)?))?;
                self.phase = "TERMINAL";
                require(
                    receipt.result() == CommitmentResult::Confirmed,
                    "Observed database outcome contradicted the frozen Rust prediction",
                )?;
                let admission = receipt
                    .trust_admission()
                    .ok_or("Receipt lost trust admission")?;
                let mut out = self.envelope(
                    "CONFIRMED",
                    "P9_ADMITTED_DISPATCH_AND_OBSERVED_DATABASE_OUTCOME_CONFIRMED",
                );
                out["proof_result"] = json!("CONFIRMED");
                out["dispatch_issued"] = json!(true);
                out["target_training_updates"] = json!(0);
                out["training_epochs"] = json!(0);
                out["commitment_digest"] =
                    json!(receipt.prepared().commitment_digest().to_string());
                out["plan_digest"] = json!(receipt.prepared().plan_digest().to_string());
                out["lease_digest"] = json!(admission.lease_digest().to_string());
                out["dependency_count"] = json!(admission.dependency_count());
                out["lease_consumed"] =
                    json!(self.lease.as_ref().is_some_and(|lease| lease.consumed()));
                out["receiver_dispatches"] = json!(self.dispatches);
                out["protocol"] = json!("CKS/1");
                out["protocol_verified"] = json!(self.protocol_verified);
                Ok(out)
            }
            _ => Err("Unknown P9 operation".to_owned()),
        }
    }
}

#[wasm_bindgen]
pub struct WasmTheaterOrigin(TheaterOrigin);
#[wasm_bindgen]
impl WasmTheaterOrigin {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, origin: u32, secret: Vec<u8>) -> Result<WasmTheaterOrigin, JsValue> {
        TheaterOrigin::new(seed, u64::from(origin), &secret)
            .map(Self)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn public_key(&self) -> String {
        self.0.public_key()
    }
    pub fn bundle(&self, sequence: u32) -> Result<String, JsValue> {
        self.0
            .bundle(u64::from(sequence))
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn revocation(&self, sequence: u32) -> Result<String, JsValue> {
        self.0
            .revocation(u64::from(sequence))
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn sabotage(&mut self) -> Result<String, JsValue> {
        self.0.sabotage().map_err(|e| JsValue::from_str(&e))
    }
    pub fn restore(&mut self) -> Result<String, JsValue> {
        self.0.restore().map_err(|e| JsValue::from_str(&e))
    }
}
#[wasm_bindgen]
pub struct WasmTheaterReceiver(TheaterReceiver);
#[wasm_bindgen]
impl WasmTheaterReceiver {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32, a: Vec<u8>, b: Vec<u8>) -> Result<WasmTheaterReceiver, JsValue> {
        TheaterReceiver::new(seed, &a, &b)
            .map(Self)
            .map_err(|e| JsValue::from_str(&e))
    }
    pub fn step(&mut self, request: String) -> Result<String, JsValue> {
        self.0
            .step(&request)
            .and_then(|value| check(serde_json::to_string(&value)))
            .map_err(|e| JsValue::from_str(&e))
    }
}
