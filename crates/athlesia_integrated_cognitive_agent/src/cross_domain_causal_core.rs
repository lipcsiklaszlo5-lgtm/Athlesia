//! Bounded cross-domain causal cycle.
//!
//! The module is deliberately small and fail-closed.  It provides the first
//! executable contract for the planned `Observe -> Abstract -> Transfer ->
//! Falsify -> Revise` cycle while leaving the existing M47/M49 APIs intact.
//! A transfer is a structural candidate until a target observation supports
//! it; an ambiguous or over-budget match never becomes an executable permit.

use std::collections::{BTreeMap, BTreeSet};

pub type NodeId = u32;
pub type TokenId = u64;
pub type ActionId = u64;
pub type Digest = u64;

const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

fn mix(mut state: u64, value: u64) -> u64 {
    for byte in value.to_le_bytes() {
        state ^= u64::from(byte);
        state = state.wrapping_mul(FNV_PRIME);
    }
    state
}

#[cfg(test)]
fn hash_bytes(bytes: &[u8]) -> Digest {
    bytes.iter().fold(FNV_OFFSET, |state, byte| {
        let state = state ^ u64::from(*byte);
        state.wrapping_mul(FNV_PRIME)
    })
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NodeKind {
    Service,
    Operation,
    StateField,
    Resource,
    Constraint,
    Transaction,
}

impl NodeKind {
    fn tag(self) -> u64 {
        match self {
            Self::Service => 1,
            Self::Operation => 2,
            Self::StateField => 3,
            Self::Resource => 4,
            Self::Constraint => 5,
            Self::Transaction => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EdgeKind {
    Calls,
    Reads,
    Writes,
    Requires,
    Emits,
    DependsOn,
    InterventionProduced,
}

impl EdgeKind {
    fn tag(self) -> u64 {
        match self {
            Self::Calls => 1,
            Self::Reads => 2,
            Self::Writes => 3,
            Self::Requires => 4,
            Self::Emits => 5,
            Self::DependsOn => 6,
            Self::InterventionProduced => 7,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalNode {
    pub id: NodeId,
    pub kind: NodeKind,
    pub type_tag: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CausalEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: EdgeKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MappingBudget {
    pub max_search_states: u64,
}

impl MappingBudget {
    pub const fn new(max_search_states: u64) -> Option<Self> {
        if max_search_states == 0 {
            None
        } else {
            Some(Self { max_search_states })
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphError {
    Empty,
    DuplicateNode(NodeId),
    UnknownEdgeEndpoint(CausalEdge),
    DuplicateEdge(CausalEdge),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalCausalGraph {
    nodes: Vec<CausalNode>,
    edges: Vec<CausalEdge>,
    digest: Digest,
}

impl CanonicalCausalGraph {
    pub fn new(mut nodes: Vec<CausalNode>, mut edges: Vec<CausalEdge>) -> Result<Self, GraphError> {
        if nodes.is_empty() {
            return Err(GraphError::Empty);
        }
        nodes.sort_by_key(|node| node.id);
        for pair in nodes.windows(2) {
            if pair[0].id == pair[1].id {
                return Err(GraphError::DuplicateNode(pair[0].id));
            }
        }
        let ids: BTreeSet<NodeId> = nodes.iter().map(|node| node.id).collect();
        edges.sort();
        for edge in &edges {
            if !ids.contains(&edge.from) || !ids.contains(&edge.to) {
                return Err(GraphError::UnknownEdgeEndpoint(*edge));
            }
        }
        for pair in edges.windows(2) {
            if pair[0] == pair[1] {
                return Err(GraphError::DuplicateEdge(pair[0]));
            }
        }
        let mut graph = Self {
            nodes,
            edges,
            digest: 0,
        };
        graph.digest = graph.compute_digest();
        Ok(graph)
    }

    pub fn nodes(&self) -> &[CausalNode] {
        &self.nodes
    }

    pub fn edges(&self) -> &[CausalEdge] {
        &self.edges
    }

    pub fn digest(&self) -> Digest {
        self.digest
    }

    fn compute_digest(&self) -> Digest {
        let mut digest = FNV_OFFSET;
        for node in &self.nodes {
            digest = mix(digest, u64::from(node.id));
            digest = mix(digest, node.kind.tag());
            digest = mix(digest, node.type_tag);
        }
        for edge in &self.edges {
            digest = mix(digest, u64::from(edge.from));
            digest = mix(digest, u64::from(edge.to));
            digest = mix(digest, edge.kind.tag());
        }
        digest
    }

    fn node(&self, id: NodeId) -> Option<&CausalNode> {
        self.nodes.binary_search_by_key(&id, |node| node.id).ok().map(|i| &self.nodes[i])
    }

    fn has_edge(&self, from: NodeId, to: NodeId, kind: EdgeKind) -> bool {
        self.edges.iter().any(|edge| edge.from == from && edge.to == to && edge.kind == kind)
    }

    fn initial_color(&self, id: NodeId) -> u64 {
        let node = self.node(id).expect("validated node id");
        let color = mix(FNV_OFFSET, node.kind.tag());
        mix(color, node.type_tag)
    }

    fn refined_colors(&self) -> BTreeMap<NodeId, u64> {
        let mut colors: BTreeMap<NodeId, u64> = self
            .nodes
            .iter()
            .map(|node| (node.id, self.initial_color(node.id)))
            .collect();

        for _ in 0..3 {
            let next = self
                .nodes
                .iter()
                .map(|node| {
                    let mut incoming: Vec<(u64, u64)> = self
                        .edges
                        .iter()
                        .filter(|edge| edge.to == node.id)
                        .map(|edge| (edge.kind.tag(), colors[&edge.from]))
                        .collect();
                    let mut outgoing: Vec<(u64, u64)> = self
                        .edges
                        .iter()
                        .filter(|edge| edge.from == node.id)
                        .map(|edge| (edge.kind.tag(), colors[&edge.to]))
                        .collect();
                    incoming.sort();
                    outgoing.sort();
                    let mut color = mix(colors[&node.id], incoming.len() as u64);
                    for (kind, neighbour) in incoming.into_iter().chain(outgoing) {
                        color = mix(color, kind);
                        color = mix(color, neighbour);
                    }
                    (node.id, color)
                })
                .collect();
            colors = next;
        }
        colors
    }

    pub fn find_unique_mapping(
        &self,
        target: &Self,
        budget: MappingBudget,
    ) -> MappingVerdict {
        if self.nodes.len() != target.nodes.len() || self.edges.len() != target.edges.len() {
            return MappingVerdict::NoMatch;
        }
        let source_colors = self.refined_colors();
        let target_colors = target.refined_colors();
        let mut source_classes = BTreeMap::<u64, usize>::new();
        let mut target_classes = BTreeMap::<u64, usize>::new();
        for color in source_colors.values() {
            *source_classes.entry(*color).or_default() += 1;
        }
        for color in target_colors.values() {
            *target_classes.entry(*color).or_default() += 1;
        }
        if source_classes != target_classes {
            return MappingVerdict::NoMatch;
        }

        let mut assigned = BTreeMap::new();
        let mut used = BTreeSet::new();
        let mut states = 0u64;
        let mut solutions = Vec::new();
        search_mapping(
            self,
            target,
            &source_colors,
            &target_colors,
            budget.max_search_states,
            &mut states,
            &mut assigned,
            &mut used,
            &mut solutions,
        );
        // One solution is not proof of uniqueness if the remaining search was
        // cut off. Two solutions already prove ambiguity; otherwise abstain.
        if states >= budget.max_search_states && solutions.len() < 2 {
            return MappingVerdict::ResourceLimit { visited_states: states };
        }
        match solutions.len() {
            0 => MappingVerdict::NoMatch,
            1 => MappingVerdict::Unique(CausalMapping {
                pairs: solutions.remove(0),
                source_digest: self.digest,
                target_digest: target.digest,
                visited_states: states,
            }),
            _ => MappingVerdict::Ambiguous { visited_states: states },
        }
    }
}

fn partial_mapping_compatible(
    source: &CanonicalCausalGraph,
    target: &CanonicalCausalGraph,
    source_id: NodeId,
    target_id: NodeId,
    assigned: &BTreeMap<NodeId, NodeId>,
) -> bool {
    let source_node = source.node(source_id).expect("validated node id");
    let target_node = target.node(target_id).expect("validated node id");
    if source_node.kind != target_node.kind || source_node.type_tag != target_node.type_tag {
        return false;
    }
    for edge in source.edges.iter().filter(|edge| edge.from == source_id) {
        if let Some(mapped_to) = assigned.get(&edge.to) {
            if !target.has_edge(target_id, *mapped_to, edge.kind) {
                return false;
            }
        }
    }
    for edge in source.edges.iter().filter(|edge| edge.to == source_id) {
        if let Some(mapped_from) = assigned.get(&edge.from) {
            if !target.has_edge(*mapped_from, target_id, edge.kind) {
                return false;
            }
        }
    }
    true
}

fn search_mapping(
    source: &CanonicalCausalGraph,
    target: &CanonicalCausalGraph,
    source_colors: &BTreeMap<NodeId, u64>,
    target_colors: &BTreeMap<NodeId, u64>,
    max_states: u64,
    states: &mut u64,
    assigned: &mut BTreeMap<NodeId, NodeId>,
    used: &mut BTreeSet<NodeId>,
    solutions: &mut Vec<Vec<(NodeId, NodeId)>>,
) {
    if *states >= max_states || solutions.len() > 1 {
        return;
    }
    *states += 1;
    if assigned.len() == source.nodes.len() {
        let valid = source.edges.iter().all(|edge| {
            let from = assigned[&edge.from];
            let to = assigned[&edge.to];
            target.has_edge(from, to, edge.kind)
        });
        if valid {
            solutions.push(assigned.iter().map(|(a, b)| (*a, *b)).collect());
        }
        return;
    }

    let mut best_source = None;
    let mut best_candidates = Vec::new();
    for node in &source.nodes {
        if assigned.contains_key(&node.id) {
            continue;
        }
        let candidates: Vec<NodeId> = target
            .nodes
            .iter()
            .filter(|candidate| {
                !used.contains(&candidate.id)
                    && source_colors[&node.id] == target_colors[&candidate.id]
                    && partial_mapping_compatible(source, target, node.id, candidate.id, assigned)
            })
            .map(|candidate| candidate.id)
            .collect();
        if best_source.is_none() || candidates.len() < best_candidates.len() {
            best_source = Some(node.id);
            best_candidates = candidates;
        }
    }
    let Some(source_id) = best_source else { return };
    for target_id in best_candidates {
        assigned.insert(source_id, target_id);
        used.insert(target_id);
        search_mapping(
            source,
            target,
            source_colors,
            target_colors,
            max_states,
            states,
            assigned,
            used,
            solutions,
        );
        used.remove(&target_id);
        assigned.remove(&source_id);
        if solutions.len() > 1 || *states >= max_states {
            break;
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalMapping {
    pairs: Vec<(NodeId, NodeId)>,
    source_digest: Digest,
    target_digest: Digest,
    visited_states: u64,
}

impl CausalMapping {
    pub fn pairs(&self) -> &[(NodeId, NodeId)] { &self.pairs }
    pub fn source_digest(&self) -> Digest { self.source_digest }
    pub fn target_digest(&self) -> Digest { self.target_digest }
    pub fn visited_states(&self) -> u64 { self.visited_states }
    pub fn map_node(&self, source: NodeId) -> Option<NodeId> {
        self.pairs.iter().find(|(from, _)| *from == source).map(|(_, to)| *to)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MappingVerdict {
    Unique(CausalMapping),
    Ambiguous { visited_states: u64 },
    NoMatch,
    ResourceLimit { visited_states: u64 },
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Literal {
    pub role: TokenId,
    pub value: bool,
}

impl Literal {
    pub const fn new(role: TokenId, value: bool) -> Self { Self { role, value } }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    values: BTreeMap<TokenId, bool>,
}

impl State {
    pub fn new(values: impl IntoIterator<Item = Literal>) -> Self {
        Self { values: values.into_iter().map(|literal| (literal.role, literal.value)).collect() }
    }

    pub fn get(&self, role: TokenId) -> Option<bool> { self.values.get(&role).copied() }

    pub fn with(&self, literal: Literal) -> Self {
        let mut values = self.values.clone();
        values.insert(literal.role, literal.value);
        Self { values }
    }

    pub fn literals(&self) -> impl Iterator<Item = Literal> + '_ {
        self.values.iter().map(|(role, value)| Literal::new(*role, *value))
    }

    pub fn digest(&self) -> Digest {
        self.literals().fold(FNV_OFFSET, |state, literal| {
            mix(mix(state, literal.role), u64::from(literal.value))
        })
    }

    pub fn differs_from(&self, other: &Self) -> Vec<Literal> {
        let roles: BTreeSet<TokenId> = self.values.keys().chain(other.values.keys()).copied().collect();
        roles.into_iter().filter_map(|role| {
            (self.get(role) != other.get(role)).then(|| Literal::new(role, other.get(role).unwrap_or(false)))
        }).collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionRule {
    pub action: ActionId,
    pub guards: Vec<Literal>,
    pub effects: Vec<Literal>,
}

impl ActionRule {
    pub fn predicts(&self, before: &State) -> State {
        if self.guards.iter().all(|guard| before.get(guard.role) == Some(guard.value)) {
            self.effects.iter().fold(before.clone(), |state, effect| state.with(*effect))
        } else {
            before.clone()
        }
    }

    pub fn remap(&self, mapping: &CausalMapping, target_action: ActionId) -> Option<Self> {
        let remap_literal = |literal: Literal| {
            mapping.map_node(literal.role as NodeId).map(|role| Literal::new(u64::from(role), literal.value))
        };
        Some(Self {
            action: target_action,
            guards: self.guards.iter().copied().map(remap_literal).collect::<Option<Vec<_>>>()?,
            effects: self.effects.iter().copied().map(remap_literal).collect::<Option<Vec<_>>>()?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionExample {
    pub episode: u64,
    pub action: ActionId,
    pub before: State,
    pub after: State,
}

impl TransitionExample {
    pub fn new(episode: u64, action: ActionId, before: State, after: State) -> Self {
        Self { episode, action, before, after }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Probe {
    pub before: State,
    pub worst_case_eliminated: usize,
    pub partition_sizes: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObservationVerdict {
    Confirmed,
    ContradictionFound { candidates: usize },
    Revised { epoch: u64 },
    Abstain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalLearner {
    rule: Option<ActionRule>,
    version_space: Vec<ActionRule>,
    witnesses: Vec<TransitionExample>,
    pending: Vec<ActionRule>,
    quarantined: bool,
    epoch: u64,
}

impl Default for CausalLearner {
    fn default() -> Self { Self::new() }
}

impl CausalLearner {
    pub fn new() -> Self {
        Self { rule: None, version_space: Vec::new(), witnesses: Vec::new(), pending: Vec::new(), quarantined: false, epoch: 0 }
    }

    pub fn epoch(&self) -> u64 { self.epoch }
    pub fn rule(&self) -> Option<&ActionRule> { self.rule.as_ref() }
    pub fn version_space(&self) -> &[ActionRule] { &self.version_space }
    pub fn quarantined(&self) -> bool { self.quarantined }

    fn install_transferred_rule(&mut self, rule: ActionRule) {
        self.rule = Some(rule.clone());
        self.version_space = vec![rule];
        self.witnesses.clear();
        self.pending.clear();
        self.quarantined = false;
        self.epoch = 0;
    }

    pub fn model_digest(&self) -> Digest {
        let mut digest = mix(FNV_OFFSET, self.epoch);
        if let Some(rule) = &self.rule {
            digest = mix(digest, rule.action);
            for guard in &rule.guards {
                digest = mix(mix(digest, guard.role), u64::from(guard.value));
            }
            for effect in &rule.effects {
                digest = mix(mix(digest, effect.role), u64::from(effect.value));
            }
        }
        digest
    }

    pub fn fit(&mut self, examples: &[TransitionExample]) -> Result<(), &'static str> {
        if examples.is_empty() { return Err("empty evidence"); }
        let action = examples[0].action;
        if examples.iter().any(|example| example.action != action) { return Err("mixed actions"); }
        let effects = examples[0].before.differs_from(&examples[0].after);
        if effects.is_empty() || examples.iter().any(|example| example.before.differs_from(&example.after) != effects) {
            return Err("inconsistent effect signature");
        }
        let rule = ActionRule { action, guards: Vec::new(), effects };
        if examples.iter().any(|example| rule.predicts(&example.before) != example.after) { return Err("positive witness rejected"); }
        self.rule = Some(rule.clone());
        self.version_space = vec![rule];
        self.witnesses = examples.to_vec();
        self.pending.clear();
        self.quarantined = false;
        Ok(())
    }

    pub fn predict(&self, action: ActionId, before: &State) -> Option<State> {
        if self.quarantined || self.rule.as_ref()?.action != action { return None; }
        Some(self.rule.as_ref()?.predicts(before))
    }

    fn hypotheses_for(&self, contradiction: &TransitionExample) -> Vec<ActionRule> {
        let Some(rule) = &self.rule else { return Vec::new(); };
        // An effect role describes the value being changed by the action;
        // treating that same role as a newly inferred precondition creates
        // spurious self-guard combinations (for example `gate == false`).
        // Candidate guards are therefore drawn only from non-effect roles.
        let effect_roles: BTreeSet<TokenId> = rule.effects.iter().map(|effect| effect.role).collect();
        let roles: Vec<TokenId> = contradiction
            .before
            .literals()
            .map(|literal| literal.role)
            .filter(|role| !effect_roles.contains(role))
            .collect();
        let literals: Vec<Literal> = roles
            .into_iter()
            .flat_map(|role| [Literal::new(role, false), Literal::new(role, true)])
            .collect();
        let mut candidates = Vec::new();
        for first in 0..=literals.len() {
            for second in first..=literals.len() {
                if first == literals.len() && second != literals.len() { continue; }
                let mut guards = Vec::new();
                if first < literals.len() { guards.push(literals[first]); }
                if second < literals.len() && second != first { guards.push(literals[second]); }
                if guards.len() > 2 { continue; }
                guards.sort();
                let candidate = ActionRule { action: rule.action, guards, effects: rule.effects.clone() };
                if candidate.predicts(&contradiction.before) != contradiction.after { continue; }
                if self.witnesses.iter().all(|witness| candidate.predicts(&witness.before) == witness.after) {
                    if !candidates.contains(&candidate) { candidates.push(candidate); }
                }
            }
        }
        // Keep the complete bounded frontier (up to two guard literals), not
        // only the shortest edit.  The one-literal candidates may be
        // observationally indistinguishable from a two-literal conjunction;
        // dropping the latter would turn an unresolved contradiction into an
        // arbitrary choice before the active probe runs.
        candidates.sort_by_key(|candidate| (candidate.guards.len(), candidate.guards.clone()));
        candidates
    }

    pub fn observe(&mut self, example: TransitionExample) -> ObservationVerdict {
        let Some(predicted) = self.predict(example.action, &example.before) else { return ObservationVerdict::Abstain; };
        if predicted == example.after {
            self.witnesses.push(example);
            return ObservationVerdict::Confirmed;
        }
        self.pending = self.hypotheses_for(&example);
        self.version_space = self.pending.clone();
        self.quarantined = true;
        ObservationVerdict::ContradictionFound { candidates: self.pending.len() }
    }

    pub fn choose_probe(&self, base: &State) -> Option<Probe> {
        if !self.quarantined || self.pending.len() < 2 { return None; }
        let roles: Vec<TokenId> = self.pending.iter().flat_map(|rule| rule.guards.iter().map(|g| g.role)).collect::<BTreeSet<_>>().into_iter().collect();
        if roles.len() > 8 { return None; }
        let mut best: Option<Probe> = None;
        for mask in 0..(1usize << roles.len()) {
            let probe_state = roles.iter().enumerate().fold(base.clone(), |state, (index, role)| state.with(Literal::new(*role, (mask & (1 << index)) != 0)));
            let mut partitions: BTreeMap<Digest, usize> = BTreeMap::new();
            for candidate in &self.pending { *partitions.entry(candidate.predicts(&probe_state).digest()).or_default() += 1; }
            if partitions.len() < 2 { continue; }
            let largest = partitions.values().copied().max().unwrap_or(0);
            let worst = self.pending.len().saturating_sub(largest);
            let partition_sizes: Vec<usize> = partitions.values().copied().collect();
            let replace = best.as_ref().map(|current| worst > current.worst_case_eliminated).unwrap_or(true);
            if replace { best = Some(Probe { before: probe_state, worst_case_eliminated: worst, partition_sizes }); }
        }
        best
    }

    pub fn apply_probe(&mut self, example: TransitionExample) -> ObservationVerdict {
        if !self.quarantined || self.pending.is_empty() { return ObservationVerdict::Abstain; }
        self.pending.retain(|candidate| candidate.predicts(&example.before) == example.after);
        self.version_space = self.pending.clone();
        if self.pending.len() != 1 { return ObservationVerdict::Abstain; }
        let rule = self.pending[0].clone();
        if self.witnesses.iter().any(|witness| rule.predicts(&witness.before) != witness.after) { return ObservationVerdict::Abstain; }
        self.rule = Some(rule.clone());
        self.witnesses.push(example);
        self.pending.clear();
        self.version_space = vec![rule];
        self.quarantined = false;
        self.epoch = self.epoch.saturating_add(1);
        ObservationVerdict::Revised { epoch: self.epoch }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferStatus {
    StructuralCandidate,
    Supported,
    Falsified,
    Revised,
    Ambiguous,
    Refused,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossDomainCausalCore {
    source: CausalLearner,
    target: Option<CausalLearner>,
    mapping: Option<CausalMapping>,
    status: TransferStatus,
    ledger: EvidenceLedger,
}

impl Default for CrossDomainCausalCore {
    fn default() -> Self { Self::new() }
}

impl CrossDomainCausalCore {
    pub fn new() -> Self {
        Self { source: CausalLearner::new(), target: None, mapping: None, status: TransferStatus::Refused, ledger: EvidenceLedger::new() }
    }

    pub fn source(&self) -> &CausalLearner { &self.source }
    pub fn target(&self) -> Option<&CausalLearner> { self.target.as_ref() }
    pub fn status(&self) -> TransferStatus { self.status }
    pub fn ledger(&self) -> &EvidenceLedger { &self.ledger }

    pub fn learn_source(&mut self, examples: &[TransitionExample]) -> Result<(), &'static str> {
        self.source.fit(examples)?;
        let payload = examples.first().map(|example| example.before.digest()).unwrap_or(0);
        self.ledger.append(self.source.epoch(), TraceKind::Observe, payload);
        self.ledger.append(self.source.epoch(), TraceKind::Abstract, self.source.model_digest());
        Ok(())
    }

    pub fn transfer(
        &mut self,
        source_graph: &CanonicalCausalGraph,
        target_graph: &CanonicalCausalGraph,
        target_action: ActionId,
        budget: MappingBudget,
    ) -> TransferStatus {
        let verdict = source_graph.find_unique_mapping(target_graph, budget);
        let MappingVerdict::Unique(mapping) = verdict else {
            self.status = match verdict {
                MappingVerdict::Ambiguous { .. } => TransferStatus::Ambiguous,
                _ => TransferStatus::Refused,
            };
            return self.status;
        };
        let Some(source_rule) = self.source.rule() else {
            self.status = TransferStatus::Refused;
            return self.status;
        };
        let Some(target_rule) = source_rule.remap(&mapping, target_action) else {
            self.status = TransferStatus::Refused;
            return self.status;
        };
        let mut target = CausalLearner::new();
        target.install_transferred_rule(target_rule);
        self.target = Some(target);
        self.mapping = Some(mapping.clone());
        self.status = TransferStatus::StructuralCandidate;
        self.ledger.append(self.source.epoch(), TraceKind::Transfer, mapping.target_digest());
        self.status
    }

    pub fn target_trial(&mut self, example: TransitionExample) -> ObservationVerdict {
        let (verdict, epoch, predicted_digest, contradicted) = {
            let Some(target) = self.target.as_mut() else { return ObservationVerdict::Abstain; };
            let Some(predicted) = target.predict(example.action, &example.before) else { return ObservationVerdict::Abstain; };
            let contradicted = predicted != example.after;
            let verdict = target.observe(example);
            (verdict, target.epoch(), predicted.digest(), contradicted)
        };
        if contradicted {
            self.status = TransferStatus::Falsified;
            self.ledger.append(epoch, TraceKind::Falsify, predicted_digest);
            self.ledger.append(epoch, TraceKind::PermitRevoked, predicted_digest);
        } else {
            self.status = TransferStatus::Supported;
            self.ledger.append(epoch, TraceKind::Observe, predicted_digest);
        }
        verdict
    }

    pub fn choose_target_probe(&self, base: &State) -> Option<Probe> {
        self.target.as_ref()?.choose_probe(base)
    }

    pub fn apply_target_probe(&mut self, example: TransitionExample) -> ObservationVerdict {
        let (verdict, epoch, model_digest) = {
            let Some(target) = self.target.as_mut() else { return ObservationVerdict::Abstain; };
            let verdict = target.apply_probe(example);
            (verdict, target.epoch(), target.model_digest())
        };
        if matches!(verdict, ObservationVerdict::Revised { .. }) {
            self.status = TransferStatus::Revised;
            self.ledger.append(epoch, TraceKind::Revise, model_digest);
        }
        verdict
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TraceKind { Observe, Abstract, Transfer, Falsify, Revise, PermitRevoked }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceEvent {
    pub sequence: u64,
    pub epoch: u64,
    pub kind: TraceKind,
    pub payload_digest: Digest,
    pub previous_hash: Digest,
    pub record_hash: Digest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceLedger { events: Vec<TraceEvent>, tip: Digest }

impl Default for EvidenceLedger { fn default() -> Self { Self::new() } }

impl EvidenceLedger {
    pub fn new() -> Self { Self { events: Vec::new(), tip: 0 } }
    pub fn events(&self) -> &[TraceEvent] { &self.events }
    pub fn append(&mut self, epoch: u64, kind: TraceKind, payload_digest: Digest) -> &TraceEvent {
        let sequence = self.events.len() as u64;
        let mut hash = mix(self.tip, sequence);
        hash = mix(hash, epoch);
        hash = mix(hash, kind as u64);
        hash = mix(hash, payload_digest);
        self.events.push(TraceEvent { sequence, epoch, kind, payload_digest, previous_hash: self.tip, record_hash: hash });
        self.tip = hash;
        self.events.last().expect("just appended")
    }
    pub fn verify(&self) -> bool {
        let mut previous = 0;
        for event in &self.events {
            if event.previous_hash != previous { return false; }
            let mut hash = mix(previous, event.sequence);
            hash = mix(hash, event.epoch);
            hash = mix(hash, event.kind as u64);
            hash = mix(hash, event.payload_digest);
            if hash != event.record_hash { return false; }
            previous = event.record_hash;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(offset: u32) -> CanonicalCausalGraph {
        CanonicalCausalGraph::new(
            vec![
                CausalNode { id: offset, kind: NodeKind::Operation, type_tag: 7 },
                CausalNode { id: offset + 1, kind: NodeKind::StateField, type_tag: 1 },
                CausalNode { id: offset + 2, kind: NodeKind::StateField, type_tag: 2 },
            ],
            vec![
                CausalEdge { from: offset, to: offset + 1, kind: EdgeKind::Requires },
                CausalEdge { from: offset, to: offset + 2, kind: EdgeKind::Writes },
            ],
        ).unwrap()
    }

    fn state(power: bool, safety: bool, gate: bool) -> State {
        State::new([Literal::new(1, power), Literal::new(2, safety), Literal::new(3, gate)])
    }

    fn state_at(base: u64, power: bool, safety: bool, gate: bool) -> State {
        State::new([
            Literal::new(base, power),
            Literal::new(base + 1, safety),
            Literal::new(base + 2, gate),
        ])
    }

    #[test]
    fn cross_domain_mapping_is_unique_and_edge_preserving() {
        let source = graph(10);
        let target = graph(20);
        let mapping = source.find_unique_mapping(&target, MappingBudget::new(128).unwrap());
        let MappingVerdict::Unique(mapping) = mapping else { panic!("expected unique mapping"); };
        assert_eq!(mapping.map_node(10), Some(20));
        assert_eq!(mapping.map_node(12), Some(22));
    }

    #[test]
    fn full_cross_domain_cycle_requires_target_evidence_before_revision() {
        let mut core = CrossDomainCausalCore::new();
        let source_examples = vec![
            TransitionExample::new(1, 9, state(true, true, false), state(true, true, true)),
            TransitionExample::new(2, 9, state(true, true, false), state(true, true, true)),
            TransitionExample::new(3, 9, state(true, true, false), state(true, true, true)),
        ];
        core.learn_source(&source_examples).unwrap();
        assert_eq!(core.transfer(&graph(1), &graph(10), 19, MappingBudget::new(128).unwrap()), TransferStatus::StructuralCandidate);

        let target_positive = TransitionExample::new(11, 19, state_at(10, true, true, false), state_at(10, true, true, true));
        assert_eq!(core.target_trial(target_positive), ObservationVerdict::Confirmed);
        assert_eq!(core.status(), TransferStatus::Supported);

        let before = state_at(10, false, false, false);
        assert_eq!(core.target_trial(TransitionExample::new(12, 19, before.clone(), before.clone())), ObservationVerdict::ContradictionFound { candidates: 3 });
        assert_eq!(core.status(), TransferStatus::Falsified);
        let probe = core.choose_target_probe(&before).expect("active probe");
        let observed = {
            let rule = core.target().unwrap().version_space().iter().find(|rule| rule.guards.iter().all(|guard| probe.before.get(guard.role) == Some(guard.value))).unwrap();
            TransitionExample::new(13, 19, probe.before.clone(), rule.predicts(&probe.before))
        };
        assert_eq!(core.apply_target_probe(observed), ObservationVerdict::Revised { epoch: 1 });
        assert_eq!(core.status(), TransferStatus::Revised);
        assert!(core.ledger().verify());
    }

    #[test]
    fn contradiction_probe_and_local_revision_preserve_old_witnesses() {
        let mut learner = CausalLearner::new();
        let positives = vec![
            TransitionExample::new(1, 9, state(true, true, false), state(true, true, true)),
            TransitionExample::new(2, 9, state(true, true, false), state(true, true, true)),
            TransitionExample::new(3, 9, state(true, true, false), state(true, true, true)),
        ];
        learner.fit(&positives).unwrap();
        let contradiction = TransitionExample::new(4, 9, state(false, false, false), state(false, false, false));
        assert_eq!(learner.observe(contradiction), ObservationVerdict::ContradictionFound { candidates: 3 });
        let probe = learner.choose_probe(&state(false, false, false)).expect("a discriminating probe");
        assert!(probe.worst_case_eliminated > 0);
        let observed = TransitionExample::new(5, 9, probe.before.clone(), state_from_prediction(&learner, &probe.before));
        assert_eq!(learner.apply_probe(observed), ObservationVerdict::Revised { epoch: 1 });
        assert!(!learner.quarantined());
        assert_eq!(learner.predict(9, &state(true, true, false)), Some(state(true, true, true)));
    }

    fn state_from_prediction(learner: &CausalLearner, before: &State) -> State {
        learner.version_space().iter().find(|rule| rule.guards.iter().all(|guard| before.get(guard.role) == Some(guard.value))).map(|rule| rule.predicts(before)).unwrap_or_else(|| before.clone())
    }

    #[test]
    fn ledger_is_tamper_evident() {
        let mut ledger = EvidenceLedger::new();
        ledger.append(0, TraceKind::Observe, hash_bytes(b"source"));
        ledger.append(0, TraceKind::Abstract, hash_bytes(b"ucir"));
        ledger.append(1, TraceKind::Revise, hash_bytes(b"guard"));
        assert!(ledger.verify());
        ledger.events[1].payload_digest ^= 1;
        assert!(!ledger.verify());
    }
}
