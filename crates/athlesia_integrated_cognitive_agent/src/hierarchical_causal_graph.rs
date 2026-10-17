//! Bounded hierarchical and compositional causal graphs.
//!
//! This layer keeps `CanonicalCausalGraph` and `ActionRule` as the only
//! atomic substrates. A composite is an ordered reference to existing
//! modules, not a second graph language. Composition is therefore explicit,
//! deterministic, bounded, and easy to invalidate when a child changes.
//!
//! A `CompositePlan` is a prediction view only. It never grants execution
//! authority; the frozen proof boundary remains in `causal_proof`.

use std::collections::{BTreeMap, BTreeSet};

use super::causal_knowledge_sync::SyncedKnowledgeCandidate;
use super::causal_proof::{
    FrozenPredictionError, PredictionCommitment, PredictionScope,
};
use super::cross_domain_causal_core::{
    ActionId, ActionRule, CanonicalCausalGraph, Digest, Literal, NodeId, State, TokenId,
};

pub type ModuleId = u32;

const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const ATOMIC_DOMAIN: u64 = 0x4154_4c45_5349_414d;
const COMPOSITE_DOMAIN: u64 = 0x4154_4c45_5349_434f;
const PLAN_DOMAIN: u64 = 0x4154_4c45_5349_5050;

pub const MAX_HIERARCHY_MODULES: usize = 256;
pub const MAX_COMPOSITION_STEPS: usize = 64;
pub const MAX_HIERARCHY_DEPTH: usize = 16;
pub const MAX_HIERARCHY_STATE_LITERALS: usize = 256;
pub const MAX_MODULE_GRAPH_NODES: usize = 256;
pub const MAX_MODULE_GRAPH_EDGES: usize = 512;

fn mix(mut state: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state.max(1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HierarchyLimits {
    pub max_modules: usize,
    pub max_steps: usize,
    pub max_depth: usize,
    pub max_state_literals: usize,
    pub max_graph_nodes: usize,
    pub max_graph_edges: usize,
}

impl HierarchyLimits {
    pub const fn new(
        max_modules: usize,
        max_steps: usize,
        max_depth: usize,
        max_state_literals: usize,
        max_graph_nodes: usize,
        max_graph_edges: usize,
    ) -> Option<Self> {
        if max_modules == 0
            || max_modules > MAX_HIERARCHY_MODULES
            || max_steps == 0
            || max_steps > MAX_COMPOSITION_STEPS
            || max_depth == 0
            || max_depth > MAX_HIERARCHY_DEPTH
            || max_state_literals == 0
            || max_state_literals > MAX_HIERARCHY_STATE_LITERALS
            || max_graph_nodes == 0
            || max_graph_nodes > MAX_MODULE_GRAPH_NODES
            || max_graph_edges == 0
            || max_graph_edges > MAX_MODULE_GRAPH_EDGES
        {
            return None;
        }
        Some(Self {
            max_modules,
            max_steps,
            max_depth,
            max_state_literals,
            max_graph_nodes,
            max_graph_edges,
        })
    }

    fn valid(self) -> bool {
        Self::new(
            self.max_modules,
            self.max_steps,
            self.max_depth,
            self.max_state_literals,
            self.max_graph_nodes,
            self.max_graph_edges,
        )
        .is_some()
    }
}

impl Default for HierarchyLimits {
    fn default() -> Self {
        Self {
            max_modules: 64,
            max_steps: 16,
            max_depth: 8,
            max_state_literals: 64,
            max_graph_nodes: 64,
            max_graph_edges: 128,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HierarchyError {
    InvalidLimits,
    DuplicateModule(ModuleId),
    MissingModule(ModuleId),
    ModuleBudgetExceeded { observed: usize, limit: usize },
    EmptyComposition,
    CompositionBudgetExceeded { observed: usize, limit: usize },
    DepthExceeded { observed: usize, limit: usize },
    CycleDetected(ModuleId),
    NotComposite(ModuleId),
    NotAtomic(ModuleId),
    GraphBudgetExceeded {
        nodes: usize,
        edges: usize,
    },
    EmptyEffects { module_id: ModuleId },
    InvalidRole {
        module_id: ModuleId,
        role: TokenId,
    },
    DuplicateGuardRole {
        module_id: ModuleId,
        role: TokenId,
    },
    DuplicateEffectRole {
        module_id: ModuleId,
        role: TokenId,
    },
    StaleDependency {
        parent: ModuleId,
        child: ModuleId,
    },
    StalePlan {
        root: ModuleId,
    },
    InvalidPlan(ModuleId),
    MissingGuardState {
        module_id: ModuleId,
        role: TokenId,
    },
    GuardUnsatisfied {
        module_id: ModuleId,
        role: TokenId,
    },
    StateBudgetExceeded { observed: usize, limit: usize },
    SyncedTargetGraphMismatch { expected: Digest, observed: Digest },
    SyncedActionMismatch { expected: ActionId, observed: ActionId },
}

impl std::fmt::Display for HierarchyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for HierarchyError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HierarchyProofError {
    Hierarchy(HierarchyError),
    Prediction(FrozenPredictionError),
}

impl std::fmt::Display for HierarchyProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for HierarchyProofError {}

impl From<HierarchyError> for HierarchyProofError {
    fn from(error: HierarchyError) -> Self {
        Self::Hierarchy(error)
    }
}

impl From<FrozenPredictionError> for HierarchyProofError {
    fn from(error: FrozenPredictionError) -> Self {
        Self::Prediction(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomicCausalModule {
    id: ModuleId,
    graph: CanonicalCausalGraph,
    rule: ActionRule,
    digest: Digest,
}

impl AtomicCausalModule {
    pub fn new(
        id: ModuleId,
        graph: CanonicalCausalGraph,
        rule: ActionRule,
    ) -> Result<Self, HierarchyError> {
        if rule.effects.is_empty() {
            return Err(HierarchyError::EmptyEffects { module_id: id });
        }
        let graph_ids: BTreeSet<NodeId> = graph.nodes().iter().map(|node| node.id).collect();
        validate_roles(id, &graph_ids, &rule.guards, true)?;
        validate_roles(id, &graph_ids, &rule.effects, false)?;
        let digest = atomic_digest(id, &graph, &rule);
        Ok(Self {
            id,
            graph,
            rule,
            digest,
        })
    }

    /// Bind a zero-shot CKS candidate to the exact target graph before it can
    /// enter a hierarchy. The candidate remains immutable; this only creates
    /// the same atomic substrate used by locally learned rules.
    pub fn from_synced_candidate(
        id: ModuleId,
        candidate: &SyncedKnowledgeCandidate,
        target_graph: &CanonicalCausalGraph,
    ) -> Result<Self, HierarchyError> {
        let observed = target_graph.digest();
        if candidate.target_graph_digest() != observed {
            return Err(HierarchyError::SyncedTargetGraphMismatch {
                expected: candidate.target_graph_digest(),
                observed,
            });
        }
        if candidate.rule().action != candidate.target_action() {
            return Err(HierarchyError::SyncedActionMismatch {
                expected: candidate.target_action(),
                observed: candidate.rule().action,
            });
        }
        Self::new(id, target_graph.clone(), candidate.rule().clone())
    }

    pub fn id(&self) -> ModuleId {
        self.id
    }

    pub fn graph(&self) -> &CanonicalCausalGraph {
        &self.graph
    }

    pub fn rule(&self) -> &ActionRule {
        &self.rule
    }

    pub fn digest(&self) -> Digest {
        self.digest
    }

}

fn validate_roles(
    module_id: ModuleId,
    graph_ids: &BTreeSet<NodeId>,
    literals: &[Literal],
    guards: bool,
) -> Result<(), HierarchyError> {
    let mut seen = BTreeSet::new();
    for literal in literals {
        let node = u32::try_from(literal.role).map_err(|_| HierarchyError::InvalidRole {
            module_id,
            role: literal.role,
        })?;
        if !graph_ids.contains(&node) {
            return Err(HierarchyError::InvalidRole {
                module_id,
                role: literal.role,
            });
        }
        if !seen.insert(literal.role) {
            return Err(if guards {
                HierarchyError::DuplicateGuardRole {
                    module_id,
                    role: literal.role,
                }
            } else {
                HierarchyError::DuplicateEffectRole {
                    module_id,
                    role: literal.role,
                }
            });
        }
    }
    Ok(())
}

fn rule_digest(mut digest: Digest, rule: &ActionRule) -> Digest {
    digest = mix(digest, rule.action);
    let mut guards = rule.guards.clone();
    guards.sort();
    let mut effects = rule.effects.clone();
    effects.sort();
    digest = mix(digest, guards.len() as u64);
    digest = mix(digest, effects.len() as u64);
    for literal in guards.into_iter().chain(effects) {
        digest = mix(digest, literal.role);
        digest = mix(digest, u64::from(literal.value));
    }
    digest
}

fn atomic_digest(id: ModuleId, graph: &CanonicalCausalGraph, rule: &ActionRule) -> Digest {
    let digest = mix(mix(FNV_OFFSET, ATOMIC_DOMAIN), u64::from(id));
    rule_digest(mix(digest, graph.digest()), rule)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositeCausalModule {
    id: ModuleId,
    steps: Vec<ModuleId>,
    child_digests: Vec<Digest>,
    depth: usize,
    digest: Digest,
}

impl CompositeCausalModule {
    fn new(
        id: ModuleId,
        steps: Vec<ModuleId>,
        child_digests: Vec<Digest>,
        depth: usize,
    ) -> Self {
        let mut digest = mix(mix(FNV_OFFSET, COMPOSITE_DOMAIN), u64::from(id));
        digest = mix(digest, depth as u64);
        for (step, child_digest) in steps.iter().zip(child_digests.iter()) {
            digest = mix(digest, u64::from(*step));
            digest = mix(digest, *child_digest);
        }
        Self {
            id,
            steps,
            child_digests,
            depth,
            digest,
        }
    }

    pub fn id(&self) -> ModuleId {
        self.id
    }

    pub fn steps(&self) -> &[ModuleId] {
        &self.steps
    }

    pub fn child_digests(&self) -> &[Digest] {
        &self.child_digests
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn digest(&self) -> Digest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum HierarchyModule {
    Atomic(AtomicCausalModule),
    Composite(CompositeCausalModule),
}

impl HierarchyModule {
    fn digest(&self) -> Digest {
        match self {
            Self::Atomic(module) => module.digest(),
            Self::Composite(module) => module.digest(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositePlan {
    root_id: ModuleId,
    atomic_steps: Vec<ModuleId>,
    dependency_digests: Vec<(ModuleId, Digest)>,
    digest: Digest,
}

impl CompositePlan {
    pub fn root_id(&self) -> ModuleId {
        self.root_id
    }

    pub fn atomic_steps(&self) -> &[ModuleId] {
        &self.atomic_steps
    }

    pub fn dependency_digests(&self) -> &[(ModuleId, Digest)] {
        &self.dependency_digests
    }

    pub fn digest(&self) -> Digest {
        self.digest
    }

    /// Stable synthetic action identity for the proof boundary. It identifies
    /// this exact composite plan and is not an executable atomic action.
    pub fn proof_action(&self) -> ActionId {
        mix(mix(FNV_OFFSET, PLAN_DOMAIN), self.digest)
    }

    /// Return the exact executable atomic action sequence behind this plan.
    ///
    /// The sequence is exposed only after the complete dependency frontier is
    /// revalidated. Callers must still cross the causal proof boundary before
    /// dispatching any returned action.
    pub fn execution_actions(
        &self,
        hierarchy: &CausalHierarchy,
    ) -> Result<Vec<ActionId>, HierarchyError> {
        hierarchy.validate_plan(self)?;
        self.atomic_steps
            .iter()
            .map(|module_id| match hierarchy.modules.get(module_id) {
                Some(HierarchyModule::Atomic(module)) => Ok(module.rule().action),
                _ => Err(HierarchyError::InvalidPlan(self.root_id)),
            })
            .collect()
    }

    /// Freeze the deterministic composite result through the existing proof
    /// boundary. The returned commitment still requires a host permit before
    /// any external executor may resolve it.
    pub fn commit(
        &self,
        hierarchy: &CausalHierarchy,
        scope: PredictionScope,
        command: u64,
        before: &State,
    ) -> Result<PredictionCommitment, HierarchyProofError> {
        let after = self.predict(hierarchy, before)?;
        PredictionCommitment::from_bounded_outcomes(
            scope,
            command,
            self.proof_action(),
            self.digest,
            before,
            vec![after],
            1,
        )
        .map_err(HierarchyProofError::from)
    }

    /// Predict the sequential causal result. This is not an execution permit.
    pub fn predict(
        &self,
        hierarchy: &CausalHierarchy,
        before: &State,
    ) -> Result<State, HierarchyError> {
        hierarchy.validate_plan(self)?;
        ensure_state_budget(before, hierarchy.limits.max_state_literals)?;
        let mut state = before.clone();
        for module_id in &self.atomic_steps {
            let Some(HierarchyModule::Atomic(module)) = hierarchy.modules.get(module_id) else {
                return Err(HierarchyError::InvalidPlan(self.root_id));
            };
            for guard in &module.rule.guards {
                match state.get(guard.role) {
                    None => {
                        return Err(HierarchyError::MissingGuardState {
                            module_id: *module_id,
                            role: guard.role,
                        })
                    }
                    Some(value) if value != guard.value => {
                        return Err(HierarchyError::GuardUnsatisfied {
                            module_id: *module_id,
                            role: guard.role,
                        })
                    }
                    Some(_) => {}
                }
            }
            state = module
                .rule
                .effects
                .iter()
                .fold(state, |current, effect| current.with(*effect));
            ensure_state_budget(&state, hierarchy.limits.max_state_literals)?;
        }
        Ok(state)
    }
}

fn ensure_state_budget(state: &State, limit: usize) -> Result<(), HierarchyError> {
    let observed = state.literals().count();
    if observed > limit {
        return Err(HierarchyError::StateBudgetExceeded { observed, limit });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalHierarchy {
    limits: HierarchyLimits,
    modules: BTreeMap<ModuleId, HierarchyModule>,
}

impl Default for CausalHierarchy {
    fn default() -> Self {
        Self::new(HierarchyLimits::default()).expect("default hierarchy limits are valid")
    }
}

impl CausalHierarchy {
    pub fn new(limits: HierarchyLimits) -> Result<Self, HierarchyError> {
        if !limits.valid() {
            return Err(HierarchyError::InvalidLimits);
        }
        Ok(Self {
            limits,
            modules: BTreeMap::new(),
        })
    }

    pub fn limits(&self) -> HierarchyLimits {
        self.limits
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn module_digest(&self, id: ModuleId) -> Option<Digest> {
        self.modules.get(&id).map(HierarchyModule::digest)
    }

    pub fn add_atomic(&mut self, module: AtomicCausalModule) -> Result<(), HierarchyError> {
        if self.modules.contains_key(&module.id()) {
            return Err(HierarchyError::DuplicateModule(module.id()));
        }
        self.check_module_budget(module.graph().nodes().len(), module.graph().edges().len())?;
        if self.modules.len() >= self.limits.max_modules {
            return Err(HierarchyError::ModuleBudgetExceeded {
                observed: self.modules.len().saturating_add(1),
                limit: self.limits.max_modules,
            });
        }
        let module_id = module.id();
        self.modules
            .insert(module_id, HierarchyModule::Atomic(module));
        Ok(())
    }

    /// Install a verified CKS candidate as an atomic module. No target
    /// training or execution authority is introduced by this adapter.
    pub fn add_synced_atomic(
        &mut self,
        module_id: ModuleId,
        candidate: &SyncedKnowledgeCandidate,
        target_graph: &CanonicalCausalGraph,
    ) -> Result<(), HierarchyError> {
        let module = AtomicCausalModule::from_synced_candidate(
            module_id,
            candidate,
            target_graph,
        )?;
        self.add_atomic(module)
    }

    pub fn replace_atomic(&mut self, module: AtomicCausalModule) -> Result<(), HierarchyError> {
        let Some(existing) = self.modules.get(&module.id()) else {
            return Err(HierarchyError::MissingModule(module.id()));
        };
        if !matches!(existing, HierarchyModule::Atomic(_)) {
            return Err(HierarchyError::NotAtomic(module.id()));
        }
        self.check_module_budget(module.graph().nodes().len(), module.graph().edges().len())?;
        let module_id = module.id();
        self.modules
            .insert(module_id, HierarchyModule::Atomic(module));
        Ok(())
    }

    /// Remove one atomic module through a fail-closed internal boundary.
    ///
    /// Existing composite modules are deliberately retained. Their captured
    /// child digest frontier will fail validation until the exact dependency
    /// is deliberately restored or the composite is recomposed.
    pub(crate) fn remove_atomic(&mut self, id: ModuleId) -> Result<(), HierarchyError> {
        let Some(existing) = self.modules.get(&id) else {
            return Err(HierarchyError::MissingModule(id));
        };
        if !matches!(existing, HierarchyModule::Atomic(_)) {
            return Err(HierarchyError::NotAtomic(id));
        }
        self.modules.remove(&id);
        Ok(())
    }

    pub fn compose(
        &mut self,
        id: ModuleId,
        steps: Vec<ModuleId>,
    ) -> Result<CompositePlan, HierarchyError> {
        if self.modules.contains_key(&id) {
            return Err(HierarchyError::DuplicateModule(id));
        }
        if self.modules.len() >= self.limits.max_modules {
            return Err(HierarchyError::ModuleBudgetExceeded {
                observed: self.modules.len().saturating_add(1),
                limit: self.limits.max_modules,
            });
        }
        let module = self.build_composite(id, steps)?;
        self.modules.insert(id, HierarchyModule::Composite(module));
        self.plan(id)
    }

    pub fn recompose(
        &mut self,
        id: ModuleId,
        steps: Vec<ModuleId>,
    ) -> Result<CompositePlan, HierarchyError> {
        let Some(existing) = self.modules.get(&id) else {
            return Err(HierarchyError::MissingModule(id));
        };
        if !matches!(existing, HierarchyModule::Composite(_)) {
            return Err(HierarchyError::NotComposite(id));
        }
        let module = self.build_composite(id, steps)?;
        self.modules.insert(id, HierarchyModule::Composite(module));
        self.plan(id)
    }

    pub fn plan(&self, root_id: ModuleId) -> Result<CompositePlan, HierarchyError> {
        match self.modules.get(&root_id) {
            None => return Err(HierarchyError::MissingModule(root_id)),
            Some(HierarchyModule::Atomic(_)) => return Err(HierarchyError::NotComposite(root_id)),
            Some(HierarchyModule::Composite(_)) => {}
        }
        let mut dependencies = BTreeMap::new();
        let mut atomic_steps = Vec::new();
        let mut visiting = BTreeSet::new();
        self.collect_module(
            root_id,
            &mut visiting,
            &mut dependencies,
            &mut atomic_steps,
        )?;
        let dependency_digests: Vec<(ModuleId, Digest)> = dependencies.into_iter().collect();
        let digest = plan_digest(root_id, &atomic_steps, &dependency_digests);
        Ok(CompositePlan {
            root_id,
            atomic_steps,
            dependency_digests,
            digest,
        })
    }

    fn check_module_budget(&self, nodes: usize, edges: usize) -> Result<(), HierarchyError> {
        if nodes > self.limits.max_graph_nodes || edges > self.limits.max_graph_edges {
            return Err(HierarchyError::GraphBudgetExceeded { nodes, edges });
        }
        Ok(())
    }

    fn build_composite(
        &self,
        id: ModuleId,
        steps: Vec<ModuleId>,
    ) -> Result<CompositeCausalModule, HierarchyError> {
        if steps.is_empty() {
            return Err(HierarchyError::EmptyComposition);
        }
        if steps.len() > self.limits.max_steps {
            return Err(HierarchyError::CompositionBudgetExceeded {
                observed: steps.len(),
                limit: self.limits.max_steps,
            });
        }
        let mut child_digests = Vec::with_capacity(steps.len());
        let mut max_child_depth = 0usize;
        let mut visiting = BTreeSet::new();
        visiting.insert(id);
        for child in &steps {
            let Some(child_module) = self.modules.get(child) else {
                return Err(HierarchyError::MissingModule(*child));
            };
            child_digests.push(child_module.digest());
            max_child_depth = max_child_depth.max(self.depth_from(*child, &mut visiting)?);
        }
        let depth = max_child_depth.saturating_add(1);
        if depth > self.limits.max_depth {
            return Err(HierarchyError::DepthExceeded {
                observed: depth,
                limit: self.limits.max_depth,
            });
        }
        Ok(CompositeCausalModule::new(
            id,
            steps,
            child_digests,
            depth,
        ))
    }

    fn depth_from(
        &self,
        id: ModuleId,
        visiting: &mut BTreeSet<ModuleId>,
    ) -> Result<usize, HierarchyError> {
        if !visiting.insert(id) {
            return Err(HierarchyError::CycleDetected(id));
        }
        let Some(module) = self.modules.get(&id).cloned() else {
            return Err(HierarchyError::MissingModule(id));
        };
        let depth = match module {
            HierarchyModule::Atomic(_) => 1,
            HierarchyModule::Composite(composite) => {
                let mut max_child_depth = 0usize;
                for child in composite.steps {
                    max_child_depth = max_child_depth.max(self.depth_from(child, visiting)?);
                }
                max_child_depth.saturating_add(1)
            }
        };
        visiting.remove(&id);
        if depth > self.limits.max_depth {
            return Err(HierarchyError::DepthExceeded {
                observed: depth,
                limit: self.limits.max_depth,
            });
        }
        Ok(depth)
    }

    fn collect_module(
        &self,
        id: ModuleId,
        visiting: &mut BTreeSet<ModuleId>,
        dependencies: &mut BTreeMap<ModuleId, Digest>,
        atomic_steps: &mut Vec<ModuleId>,
    ) -> Result<(), HierarchyError> {
        if !visiting.insert(id) {
            return Err(HierarchyError::CycleDetected(id));
        }
        let Some(module) = self.modules.get(&id).cloned() else {
            return Err(HierarchyError::MissingModule(id));
        };
        dependencies.insert(id, module.digest());
        match module {
            HierarchyModule::Atomic(_) => atomic_steps.push(id),
            HierarchyModule::Composite(composite) => {
                if composite.steps.len() != composite.child_digests.len() {
                    return Err(HierarchyError::InvalidPlan(id));
                }
                for (child, expected_digest) in composite
                    .steps
                    .iter()
                    .zip(composite.child_digests.iter())
                {
                    let Some(current_digest) = self.module_digest(*child) else {
                        return Err(HierarchyError::MissingModule(*child));
                    };
                    if current_digest != *expected_digest {
                        return Err(HierarchyError::StaleDependency {
                            parent: id,
                            child: *child,
                        });
                    }
                    self.collect_module(*child, visiting, dependencies, atomic_steps)?;
                }
            }
        }
        visiting.remove(&id);
        Ok(())
    }

    fn validate_plan(&self, plan: &CompositePlan) -> Result<(), HierarchyError> {
        let current = self
            .plan(plan.root_id)
            .map_err(|_| HierarchyError::StalePlan { root: plan.root_id })?;
        if &current != plan {
            return Err(HierarchyError::StalePlan { root: plan.root_id });
        }
        Ok(())
    }
}

fn plan_digest(
    root_id: ModuleId,
    atomic_steps: &[ModuleId],
    dependencies: &[(ModuleId, Digest)],
) -> Digest {
    let mut digest = mix(mix(FNV_OFFSET, PLAN_DOMAIN), u64::from(root_id));
    for step in atomic_steps {
        digest = mix(digest, u64::from(*step));
    }
    for (module, dependency_digest) in dependencies {
        digest = mix(digest, u64::from(*module));
        digest = mix(digest, *dependency_digest);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal_proof::{CommitmentResult, ProofError};
    use crate::contextual_causal_agent::ContextId;
    use crate::cross_domain_causal_core::{
        ActionId, CausalEdge, CausalNode, EdgeKind, NodeKind,
    };

    fn graph() -> CanonicalCausalGraph {
        CanonicalCausalGraph::new(
            vec![
                CausalNode {
                    id: 1,
                    kind: NodeKind::Operation,
                    type_tag: 7,
                },
                CausalNode {
                    id: 2,
                    kind: NodeKind::StateField,
                    type_tag: 1,
                },
                CausalNode {
                    id: 3,
                    kind: NodeKind::StateField,
                    type_tag: 2,
                },
            ],
            vec![
                CausalEdge {
                    from: 1,
                    to: 2,
                    kind: EdgeKind::Requires,
                },
                CausalEdge {
                    from: 1,
                    to: 3,
                    kind: EdgeKind::Writes,
                },
            ],
        )
        .expect("valid graph")
    }

    fn atomic(id: ModuleId, action: ActionId, guard: Literal, effect: Literal) -> AtomicCausalModule {
        AtomicCausalModule::new(
            id,
            graph(),
            ActionRule {
                action,
                guards: vec![guard],
                effects: vec![effect],
            },
        )
        .expect("valid atomic module")
    }

    #[test]
    fn sequential_composition_predicts_without_execution_authority() {
        let mut hierarchy = CausalHierarchy::default();
        hierarchy
            .add_atomic(atomic(1, 10, Literal::new(1, true), Literal::new(2, true)))
            .expect("first module");
        hierarchy
            .add_atomic(atomic(2, 11, Literal::new(2, true), Literal::new(3, true)))
            .expect("second module");
        let plan = hierarchy.compose(100, vec![1, 2]).expect("composition");
        let before = State::new([
            Literal::new(1, true),
            Literal::new(2, false),
            Literal::new(3, false),
        ]);
        let after = plan.predict(&hierarchy, &before).expect("prediction");
        assert_eq!(after.get(2), Some(true));
        assert_eq!(after.get(3), Some(true));
        assert_eq!(plan.atomic_steps(), &[1, 2]);
        assert_eq!(plan, hierarchy.plan(100).expect("same plan"));
    }

    #[test]
    fn composite_plan_uses_frozen_proof_boundary_before_resolution() {
        let mut hierarchy = CausalHierarchy::default();
        hierarchy
            .add_atomic(atomic(1, 10, Literal::new(1, true), Literal::new(2, true)))
            .expect("module");
        let plan = hierarchy.compose(100, vec![1]).expect("composition");
        let before = State::new([Literal::new(1, true), Literal::new(2, false)]);
        let after = plan.predict(&hierarchy, &before).expect("prediction");
        let scope = PredictionScope {
            schema_digest: 10,
            abstraction_digest: 11,
            context: Some(ContextId(12)),
            epoch: 2,
            episode: 7,
            sequence: 1,
        };
        let mut commitment = plan
            .commit(&hierarchy, scope, 900, &before)
            .expect("commitment");
        assert_eq!(commitment.model_digest(), plan.digest());
        assert_eq!(commitment.action(), plan.proof_action());
        let mut permit = commitment.issue_permit(700, 4);
        assert_eq!(
            commitment.consume_permit(
                &mut permit,
                scope,
                900,
                plan.proof_action() ^ 1,
                &before,
            ),
            Err(ProofError::WrongAction)
        );
        commitment
            .consume_permit(&mut permit, scope, 900, plan.proof_action(), &before)
            .expect("exact permit");
        assert_eq!(
            commitment
                .resolve(
                    &permit,
                    scope,
                    900,
                    plan.proof_action(),
                    &before,
                    &after,
                )
                .expect("resolve"),
            CommitmentResult::Confirmed
        );
    }

    #[test]
    fn child_revision_invalidates_parent_plan_until_recomposed() {
        let mut hierarchy = CausalHierarchy::default();
        hierarchy
            .add_atomic(atomic(1, 10, Literal::new(1, true), Literal::new(2, true)))
            .expect("first module");
        hierarchy
            .add_atomic(atomic(2, 11, Literal::new(2, true), Literal::new(3, true)))
            .expect("second module");
        let old_plan = hierarchy.compose(100, vec![1, 2]).expect("composition");
        hierarchy
            .replace_atomic(atomic(2, 11, Literal::new(2, true), Literal::new(3, false)))
            .expect("child revision");
        let before = State::new([
            Literal::new(1, true),
            Literal::new(2, false),
            Literal::new(3, false),
        ]);
        assert_eq!(
            old_plan.predict(&hierarchy, &before),
            Err(HierarchyError::StalePlan { root: 100 })
        );
        let new_plan = hierarchy.recompose(100, vec![1, 2]).expect("recomposition");
        assert_eq!(
            new_plan.predict(&hierarchy, &before).expect("new prediction").get(3),
            Some(false)
        );
    }

    #[test]
    fn missing_and_unsatisfied_guards_fail_closed() {
        let mut hierarchy = CausalHierarchy::default();
        hierarchy
            .add_atomic(atomic(1, 10, Literal::new(1, true), Literal::new(2, true)))
            .expect("module");
        let plan = hierarchy.compose(100, vec![1]).expect("composition");
        assert!(matches!(
            plan.predict(&hierarchy, &State::new(Vec::<Literal>::new())),
            Err(HierarchyError::MissingGuardState {
                module_id: 1,
                role: 1
            })
        ));
        assert!(matches!(
            plan.predict(&hierarchy, &State::new([Literal::new(1, false)])),
            Err(HierarchyError::GuardUnsatisfied {
                module_id: 1,
                role: 1
            })
        ));
    }

    #[test]
    fn bounds_and_nested_composition_are_deterministic() {
        let limits = HierarchyLimits::new(16, 1, 4, 16, 16, 16).expect("limits");
        let mut bounded = CausalHierarchy::new(limits).expect("hierarchy");
        bounded
            .add_atomic(atomic(1, 10, Literal::new(1, true), Literal::new(2, true)))
            .expect("first module");
        bounded
            .add_atomic(atomic(2, 11, Literal::new(2, true), Literal::new(3, true)))
            .expect("second module");
        assert_eq!(
            bounded.compose(100, vec![1, 2]),
            Err(HierarchyError::CompositionBudgetExceeded {
                observed: 2,
                limit: 1
            })
        );

        let mut hierarchy = CausalHierarchy::default();
        hierarchy
            .add_atomic(atomic(1, 10, Literal::new(1, true), Literal::new(2, true)))
            .expect("first module");
        hierarchy
            .add_atomic(atomic(2, 11, Literal::new(2, true), Literal::new(3, true)))
            .expect("second module");
        let child = hierarchy.compose(10, vec![1, 2]).expect("child");
        let parent = hierarchy.compose(20, vec![10, 1]).expect("parent");
        assert_eq!(child.atomic_steps(), &[1, 2]);
        assert_eq!(parent.atomic_steps(), &[1, 2, 1]);
        let before = State::new([
            Literal::new(1, true),
            Literal::new(2, false),
            Literal::new(3, false),
        ]);
        assert_eq!(
            parent.predict(&hierarchy, &before).expect("nested prediction").get(3),
            Some(true)
        );
    }

    #[test]
    fn invalid_graph_role_and_duplicate_modules_are_rejected() {
        let invalid = AtomicCausalModule::new(
            1,
            graph(),
            ActionRule {
                action: 1,
                guards: vec![Literal::new(99, true)],
                effects: vec![Literal::new(2, true)],
            },
        );
        assert!(matches!(invalid, Err(HierarchyError::InvalidRole { .. })));

        let mut hierarchy = CausalHierarchy::default();
        let module = atomic(1, 10, Literal::new(1, true), Literal::new(2, true));
        hierarchy.add_atomic(module.clone()).expect("first module");
        assert_eq!(
            hierarchy.add_atomic(module),
            Err(HierarchyError::DuplicateModule(1))
        );
    }
}
