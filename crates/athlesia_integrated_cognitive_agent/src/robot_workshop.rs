//! Bounded multi-action robot workshop integration.
//!
//! The environment owns the mechanism specification. The learning core sees
//! only typed frames and action outcomes. This is a bounded, auditable
//! vertical slice above the existing CrossDomainCausalCore.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::cross_domain_causal_core::{
    ActionId, ActionRule, CanonicalCausalGraph, CausalEdge, CausalNode,
    CrossDomainCausalCore, EdgeKind, Literal, MappingBudget, MappingVerdict, NodeId, NodeKind,
    ObservationVerdict, State, TransitionExample,
};

const MAX_ACTIONS: usize = 16;
const MAX_GUARDS: usize = 3;
const SOURCE_EXAMPLES: usize = 3;
const MAX_PROBES: usize = 8;
const MAPPING_STATES: u64 = 16_384;

pub type WorkshopDigest = u64;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MechanismSpec {
    pub slot: u64,
    pub action: String,
    #[serde(default)]
    pub guards: Vec<String>,
    pub effect: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkshopSpec {
    pub name: String,
    pub seed: u64,
    pub source: Vec<MechanismSpec>,
    pub target: Vec<MechanismSpec>,
    pub holdout: Vec<MechanismSpec>,
}

impl WorkshopSpec {
    pub fn generated(seed: u64) -> Self {
        Self {
            name: format!("robot workshop challenge {}", seed),
            seed,
            source: generated_domain(seed, "src"),
            target: generated_domain(seed.rotate_left(13), "dst"),
            holdout: generated_domain(seed.rotate_right(7), "hold"),
        }
    }

    pub fn from_json(input: &str) -> Result<Self, String> {
        let spec: Self = serde_json::from_str(input).map_err(|e| e.to_string())?;
        spec.validate()?;
        Ok(spec)
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn validate(&self) -> Result<(), String> {
        let mut slot_sets = Vec::new();
        for (name, domain) in [
            ("source", &self.source),
            ("target", &self.target),
            ("holdout", &self.holdout),
        ] {
            if domain.is_empty() || domain.len() > MAX_ACTIONS {
                return Err(format!("{} must have 1..={} actions", name, MAX_ACTIONS));
            }
            let mut slots = BTreeSet::new();
            for mechanism in domain {
                if !slots.insert(mechanism.slot)
                    || mechanism.action.is_empty()
                    || mechanism.effect.is_empty()
                    || mechanism.guards.is_empty()
                    || mechanism.guards.len() > MAX_GUARDS
                {
                    return Err(format!("invalid {} mechanism {}", name, mechanism.slot));
                }
                let mut labels = BTreeSet::new();
                if !labels.insert(mechanism.action.clone())
                    || !labels.insert(mechanism.effect.clone())
                    || mechanism.guards.iter().any(|g| !labels.insert(g.clone()))
                {
                    return Err(format!("duplicate local token in {} {}", name, mechanism.slot));
                }
            }
            slot_sets.push(slots);
        }
        if slot_sets[0] != slot_sets[1] || slot_sets[0] != slot_sets[2] {
            return Err("source, target and holdout slots differ".to_owned());
        }
        Ok(())
    }
}

fn generated_domain(seed: u64, domain: &str) -> Vec<MechanismSpec> {
    let prefix = format!("{}_{}", domain, seed);
    let action = |slot| format!("{}_action_{}", prefix, slot);
    let guard = |slot, index| format!("{}_guard_{}_{}", prefix, slot, index);
    let effect = |slot| format!("{}_effect_{}", prefix, slot);
    vec![
        MechanismSpec {
            slot: 10,
            action: action(10),
            guards: vec![guard(10, 0), guard(10, 1)],
            effect: effect(10),
        },
        MechanismSpec {
            slot: 20,
            action: action(20),
            guards: vec![effect(10)],
            effect: effect(20),
        },
        MechanismSpec {
            slot: 30,
            action: action(30),
            guards: vec![effect(20)],
            effect: effect(30),
        },
        MechanismSpec {
            slot: 40,
            action: action(40),
            guards: vec![effect(30)],
            effect: effect(40),
        },
        MechanismSpec {
            slot: 50,
            action: action(50),
            guards: vec![effect(40)],
            effect: effect(50),
        },
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkshopObjectKind {
    Operation,
    StateField,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkshopObject {
    pub token: String,
    pub kind: WorkshopObjectKind,
    pub role_tag: u64,
    #[serde(default)]
    pub value: Option<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkshopEdgeKind {
    Requires,
    Writes,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkshopEdge {
    pub from: String,
    pub to: String,
    pub kind: WorkshopEdgeKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkshopFrame {
    pub objects: Vec<WorkshopObject>,
    pub edges: Vec<WorkshopEdge>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkshopAction {
    pub token: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkshopLog {
    pub episode: u64,
    pub before: WorkshopFrame,
    pub action: WorkshopAction,
    pub after: WorkshopFrame,
}

#[derive(Clone, Debug)]
struct AbstractFrame {
    graph: CanonicalCausalGraph,
    labels: BTreeMap<String, NodeId>,
    state: State,
    action: ActionId,
}

fn stable_hash(input: &str, salt: u64) -> WorkshopDigest {
    let mut state = 14_695_981_039_346_656_037u64 ^ salt;
    for byte in input.as_bytes() {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(1_099_511_628_211);
    }
    state.max(1)
}

fn action_id(token: &str) -> ActionId {
    stable_hash(token, 0x574f524b_414354)
}

fn role_tag(mechanism: &MechanismSpec, token: &str) -> u64 {
    if token == mechanism.action {
        1_000_000 + mechanism.slot
    } else if token == mechanism.effect {
        3_000_000 + mechanism.slot
    } else {
        let index = mechanism.guards.iter().position(|g| g == token).unwrap_or(0) as u64;
        2_000_000 + mechanism.slot * 16 + index
    }
}

fn abstract_frame(frame: &WorkshopFrame, action: &WorkshopAction) -> Result<AbstractFrame, String> {
    let mut objects = frame.objects.clone();
    objects.sort_by(|a, b| a.role_tag.cmp(&b.role_tag).then_with(|| a.token.cmp(&b.token)));
    if objects.is_empty() {
        return Err("empty frame".to_owned());
    }

    let mut labels = BTreeMap::new();
    let mut nodes = Vec::with_capacity(objects.len());
    let mut values = Vec::new();

    for (index, object) in objects.iter().enumerate() {
        let id = (index + 1) as NodeId;
        if labels.insert(object.token.clone(), id).is_some() {
            return Err(format!("duplicate frame token {}", object.token));
        }
        nodes.push(CausalNode {
            id,
            kind: match object.kind {
                WorkshopObjectKind::Operation => NodeKind::Operation,
                WorkshopObjectKind::StateField => NodeKind::StateField,
            },
            type_tag: object.role_tag,
        });
        if let Some(value) = object.value {
            values.push(Literal::new(u64::from(id), value));
        }
    }

    let edges = frame
        .edges
        .iter()
        .map(|edge| {
            let from = *labels
                .get(&edge.from)
                .ok_or_else(|| format!("missing edge source {}", edge.from))?;
            let to = *labels
                .get(&edge.to)
                .ok_or_else(|| format!("missing edge target {}", edge.to))?;
            Ok(CausalEdge {
                from,
                to,
                kind: match edge.kind {
                    WorkshopEdgeKind::Requires => EdgeKind::Requires,
                    WorkshopEdgeKind::Writes => EdgeKind::Writes,
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let graph = CanonicalCausalGraph::new(nodes, edges).map_err(|e| format!("{e:?}"))?;
    let operation = labels
        .get(&action.token)
        .copied()
        .ok_or_else(|| "action token missing from frame".to_owned())?;
    Ok(AbstractFrame {
        graph,
        labels,
        state: State::new(values),
        action: action_id(&action.token) ^ u64::from(operation),
    })
}

#[derive(Clone, Debug)]
struct WorkshopEnvironment {
    mechanisms: Vec<MechanismSpec>,
    state: BTreeMap<String, bool>,
}

impl WorkshopEnvironment {
    fn new(mechanisms: &[MechanismSpec]) -> Self {
        let mut state = BTreeMap::new();
        for mechanism in mechanisms {
            for token in mechanism.guards.iter().chain(std::iter::once(&mechanism.effect)) {
                state.entry(token.clone()).or_insert(false);
            }
        }
        Self {
            mechanisms: mechanisms.to_vec(),
            state,
        }
    }

    fn reset(&mut self) {
        self.state.values_mut().for_each(|value| *value = false);
    }

    fn set(&mut self, token: &str, value: bool) -> Result<(), String> {
        let value_ref = self
            .state
            .get_mut(token)
            .ok_or_else(|| format!("unknown state token {}", token))?;
        *value_ref = value;
        Ok(())
    }

    fn state(&self, token: &str) -> bool {
        self.state.get(token).copied().unwrap_or(false)
    }

    fn frame(&self, mechanism: &MechanismSpec) -> WorkshopFrame {
        let mut objects = vec![WorkshopObject {
            token: mechanism.action.clone(),
            kind: WorkshopObjectKind::Operation,
            role_tag: role_tag(mechanism, &mechanism.action),
            value: None,
        }];
        for guard in &mechanism.guards {
            objects.push(WorkshopObject {
                token: guard.clone(),
                kind: WorkshopObjectKind::StateField,
                role_tag: role_tag(mechanism, guard),
                value: Some(self.state(guard)),
            });
        }
        objects.push(WorkshopObject {
            token: mechanism.effect.clone(),
            kind: WorkshopObjectKind::StateField,
            role_tag: role_tag(mechanism, &mechanism.effect),
            value: Some(self.state(&mechanism.effect)),
        });
        let mut edges = mechanism
            .guards
            .iter()
            .map(|guard| WorkshopEdge {
                from: mechanism.action.clone(),
                to: guard.clone(),
                kind: WorkshopEdgeKind::Requires,
            })
            .collect::<Vec<_>>();
        edges.push(WorkshopEdge {
            from: mechanism.action.clone(),
            to: mechanism.effect.clone(),
            kind: WorkshopEdgeKind::Writes,
        });
        WorkshopFrame { objects, edges }
    }

    fn dispatch(&mut self, action: &WorkshopAction) -> Result<WorkshopFrame, String> {
        let mechanism = self
            .mechanisms
            .iter()
            .find(|item| item.action == action.token)
            .cloned()
            .ok_or_else(|| format!("unknown action {}", action.token))?;
        if mechanism.guards.iter().all(|guard| self.state(guard)) {
            self.set(&mechanism.effect, true)?;
        }
        Ok(self.frame(&mechanism))
    }
}

#[derive(Clone, Debug)]
struct ActionModel {
    slot: u64,
    source: MechanismSpec,
    target: MechanismSpec,
    source_graph: CanonicalCausalGraph,
    target_graph: CanonicalCausalGraph,
    holdout_graph: CanonicalCausalGraph,
    target_action: ActionId,
    holdout_action: ActionId,
    core: CrossDomainCausalCore,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PortableLiteral {
    pub role: u64,
    pub value: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PortableRule {
    pub slot: u64,
    pub action: u64,
    pub guards: Vec<PortableLiteral>,
    pub effects: Vec<PortableLiteral>,
    pub epoch: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkshopSnapshot {
    pub version: u32,
    pub seed: u64,
    pub spec: WorkshopSpec,
    pub rules: Vec<PortableRule>,
    pub model_digest: WorkshopDigest,
    pub source_model_unchanged: bool,
}

impl WorkshopSnapshot {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    pub fn from_json(input: &str) -> Result<Self, String> {
        serde_json::from_str(input).map_err(|e| e.to_string())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkshopPhase {
    Fresh,
    SourceLearned,
    Transferred,
    TargetSupported,
    Contradicted,
    Revised,
    PlanVerified,
    HoldoutVerified,
    Abstain,
}

impl WorkshopPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Fresh => "FRESH",
            Self::SourceLearned => "SOURCE_LEARNED",
            Self::Transferred => "TRANSFERRED",
            Self::TargetSupported => "TARGET_SUPPORTED",
            Self::Contradicted => "CONTRADICTED",
            Self::Revised => "REVISED",
            Self::PlanVerified => "PLAN_VERIFIED",
            Self::HoldoutVerified => "HOLDOUT_VERIFIED",
            Self::Abstain => "ABSTAIN",
        }
    }
}

#[derive(Clone, Debug)]
pub struct WorkshopSession {
    seed: u64,
    spec: WorkshopSpec,
    source_environment: WorkshopEnvironment,
    target_environment: WorkshopEnvironment,
    holdout_environment: WorkshopEnvironment,
    models: Vec<ActionModel>,
    phase: WorkshopPhase,
    trace: Vec<String>,
    probes: usize,
    target_actions: usize,
    holdout_positive: bool,
    holdout_negative: bool,
}

impl WorkshopSession {
    pub fn generated(seed: u64) -> Result<Self, String> {
        Self::from_spec(WorkshopSpec::generated(seed))
    }

    pub fn from_spec(spec: WorkshopSpec) -> Result<Self, String> {
        spec.validate()?;
        let mut source_environment = WorkshopEnvironment::new(&spec.source);
        let target_environment = WorkshopEnvironment::new(&spec.target);
        let holdout_environment = WorkshopEnvironment::new(&spec.holdout);
        let mut models = Vec::with_capacity(spec.source.len());

        for source in &spec.source {
            let target = spec
                .target
                .iter()
                .find(|item| item.slot == source.slot)
                .cloned()
                .ok_or_else(|| format!("target slot {} missing", source.slot))?;
            let holdout = spec
                .holdout
                .iter()
                .find(|item| item.slot == source.slot)
                .cloned()
                .ok_or_else(|| format!("holdout slot {} missing", source.slot))?;

            let action = WorkshopAction {
                token: source.action.clone(),
            };
            let mut source_graph = None;
            for _ in 0..SOURCE_EXAMPLES {
                source_environment.reset();
                for guard in &source.guards {
                    source_environment.set(guard, true)?;
                }
                source_environment.set(&source.effect, false)?;
                source_graph = Some(
                    abstract_frame(&source_environment.frame(source), &action)?.graph,
                );
            }
            let target_action = WorkshopAction {
                token: target.action.clone(),
            };
            let target_abstract = abstract_frame(
                &WorkshopEnvironment::new(std::slice::from_ref(&target)).frame(&target),
                &target_action,
            )?;
            let target_graph = target_abstract.graph;
            let holdout_action = WorkshopAction {
                token: holdout.action.clone(),
            };
            let holdout_abstract = abstract_frame(
                &WorkshopEnvironment::new(std::slice::from_ref(&holdout)).frame(&holdout),
                &holdout_action,
            )?;
            let holdout_graph = holdout_abstract.graph;

            models.push(ActionModel {
                slot: source.slot,
                source: source.clone(),
                target,
                source_graph: source_graph.ok_or_else(|| "source graph missing".to_owned())?,
                target_graph,
                holdout_graph,
                target_action: target_abstract.action,
                holdout_action: holdout_abstract.action,
                core: CrossDomainCausalCore::new(),
            });
        }

        Ok(Self {
            seed: spec.seed,
            spec,
            source_environment,
            target_environment,
            holdout_environment,
            models,
            phase: WorkshopPhase::Fresh,
            trace: Vec::new(),
            probes: 0,
            target_actions: 0,
            holdout_positive: false,
            holdout_negative: false,
        })
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn phase(&self) -> WorkshopPhase {
        self.phase
    }

    pub fn spec(&self) -> &WorkshopSpec {
        &self.spec
    }

    pub fn trace_jsonl(&self) -> String {
        if self.trace.is_empty() {
            String::new()
        } else {
            self.trace.join("\n") + "\n"
        }
    }

    pub fn run_full(&mut self) -> Result<String, String> {
        self.trace.clear();
        self.probes = 0;
        self.target_actions = 0;
        self.holdout_positive = false;
        self.holdout_negative = false;
        self.phase = WorkshopPhase::Fresh;

        self.stage("OBSERVE", "source traces accepted");
        self.learn_source()?;
        self.phase = WorkshopPhase::SourceLearned;
        self.stage("ABSTRACT", "five action schemas induced from opaque frames");

        self.transfer()?;
        self.phase = WorkshopPhase::Transferred;
        self.stage("TRANSFER", "unique mappings installed for renamed actions");

        self.confirm_target()?;
        self.phase = WorkshopPhase::TargetSupported;
        self.stage("TARGET_TRIAL", "target effects confirmed before sabotage");

        self.sabotage_and_revise()?;
        self.phase = WorkshopPhase::Revised;
        self.stage(
            "REVISE",
            format!("local revision committed after {} probes", self.probes),
        );

        self.execute_plan()?;
        self.phase = WorkshopPhase::PlanVerified;
        self.stage(
            "PLAN",
            format!("composed target plan solved in {} actions", self.target_actions),
        );

        self.holdout_positive = self.run_holdout(true)?;
        self.holdout_negative = self.run_holdout(false)?;
        self.phase = if self.holdout_positive && self.holdout_negative {
            WorkshopPhase::HoldoutVerified
        } else {
            WorkshopPhase::Abstain
        };
        self.stage(
            "HOLDOUT",
            format!(
                "positive={}; negative={}; first_attempt=true; retrained=false",
                self.holdout_positive, self.holdout_negative
            ),
        );

        let snapshot = self.export_snapshot()?;
        self.trace.push(format!(
            "{{\"record\":\"summary\",\"phase\":{},\"seed\":{},\"actions\":{},\"probes\":{},\"holdout_positive\":{},\"holdout_negative\":{},\"snapshot_digest\":{},\"model_digest\":{}}}",
            quote(self.phase.label()),
            self.seed,
            self.target_actions,
            self.probes,
            self.holdout_positive,
            self.holdout_negative,
            stable_hash(&snapshot.to_json()?, 0x534e4150),
            self.model_digest()
        ));
        Ok(self.trace_jsonl())
    }

    pub fn export_snapshot(&self) -> Result<WorkshopSnapshot, String> {
        let mut rules = Vec::with_capacity(self.models.len());
        for model in &self.models {
            let learner = model
                .core
                .target()
                .ok_or_else(|| format!("missing target learner {}", model.slot))?;
            let target_rule = learner
                .rule()
                .ok_or_else(|| format!("missing target rule {}", model.slot))?;
            let mapping = match model
                .target_graph
                .find_unique_mapping(&model.holdout_graph, MappingBudget::new(MAPPING_STATES).unwrap())
            {
                MappingVerdict::Unique(mapping) => mapping,
                other => return Err(format!("holdout mapping failed: {other:?}")),
            };
            let rule = target_rule
                .remap(&mapping, model.holdout_action)
                .ok_or_else(|| format!("rule remap failed {}", model.slot))?;
            rules.push(portable_rule(model.slot, &rule, learner.epoch()));
        }
        rules.sort_by_key(|rule| rule.slot);
        Ok(WorkshopSnapshot {
            version: 1,
            seed: self.seed,
            spec: self.spec.clone(),
            rules,
            model_digest: self.model_digest(),
            source_model_unchanged: self
                .models
                .iter()
                .all(|model| model.core.source().epoch() == 0),
        })
    }

    pub fn verify_imported_snapshot(input: &str) -> Result<String, String> {
        let snapshot = WorkshopSnapshot::from_json(input)?;
        snapshot.spec.validate()?;
        let mut environment = WorkshopEnvironment::new(&snapshot.spec.holdout);
        let rules: BTreeMap<u64, ActionRule> = snapshot
            .rules
            .iter()
            .map(|rule| {
                (
                    rule.slot,
                    ActionRule {
                        action: rule.action,
                        guards: rule
                            .guards
                            .iter()
                            .map(|literal| Literal::new(literal.role, literal.value))
                            .collect(),
                        effects: rule
                            .effects
                            .iter()
                            .map(|literal| Literal::new(literal.role, literal.value))
                            .collect(),
                    },
                )
            })
            .collect();

        let first = snapshot
            .spec
            .holdout
            .first()
            .ok_or_else(|| "holdout is empty".to_owned())?;
        environment.reset();
        for guard in &first.guards {
            environment.set(guard, true)?;
        }
        let mut positive = true;
        for mechanism in &snapshot.spec.holdout {
            let action = WorkshopAction {
                token: mechanism.action.clone(),
            };
            let before = abstract_frame(&environment.frame(mechanism), &action)?;
            let rule = rules
                .get(&mechanism.slot)
                .ok_or_else(|| format!("missing snapshot rule {}", mechanism.slot))?;
            let predicted = rule.predicts(&before.state);
            let after = abstract_frame(&environment.dispatch(&action)?, &action)?;
            if predicted != after.state {
                positive = false;
                break;
            }
        }

        environment.reset();
        let action = WorkshopAction {
            token: first.action.clone(),
        };
        let before = abstract_frame(&environment.frame(first), &action)?;
        let rule = rules
            .get(&first.slot)
            .ok_or_else(|| "missing first snapshot rule".to_owned())?;
        let predicted = rule.predicts(&before.state);
        let after = abstract_frame(&environment.dispatch(&action)?, &action)?;
        let negative = predicted == after.state;
        Ok(format!(
            "{{\"record\":\"import_verification\",\"retrained\":false,\"holdout_positive\":{},\"holdout_negative\":{},\"model_digest\":{}}}\n",
            positive, negative, snapshot.model_digest
        ))
    }

    fn learn_source(&mut self) -> Result<(), String> {
        for model in &mut self.models {
            let action = WorkshopAction {
                token: model.source.action.clone(),
            };
            let mut examples = Vec::new();
            for episode in 0..SOURCE_EXAMPLES {
                self.source_environment.reset();
                for guard in &model.source.guards {
                    self.source_environment.set(guard, true)?;
                }
                self.source_environment.set(&model.source.effect, false)?;
                let before = abstract_frame(
                    &self.source_environment.frame(&model.source),
                    &action,
                )?;
                let after = abstract_frame(
                    &self.source_environment.dispatch(&action)?,
                    &action,
                )?;
                examples.push(TransitionExample::new(
                    episode as u64 + 1,
                    before.action,
                    before.state,
                    after.state,
                ));
            }
            model.core.learn_source(&examples).map_err(|e| e.to_owned())?;
        }
        Ok(())
    }

    fn transfer(&mut self) -> Result<(), String> {
        for model in &mut self.models {
            let status = model.core.transfer(
                &model.source_graph,
                &model.target_graph,
                model.target_action,
                MappingBudget::new(MAPPING_STATES).unwrap(),
            );
            if !matches!(
                status,
                super::cross_domain_causal_core::TransferStatus::StructuralCandidate
            ) {
                return Err(format!("transfer refused for slot {}: {status:?}", model.slot));
            }
        }
        Ok(())
    }

    fn confirm_target(&mut self) -> Result<(), String> {
        for model in &mut self.models {
            self.target_environment.reset();
            for guard in &model.target.guards {
                self.target_environment.set(guard, true)?;
            }
            self.target_environment.set(&model.target.effect, false)?;
            let action = WorkshopAction {
                token: model.target.action.clone(),
            };
            let before = abstract_frame(
                &self.target_environment.frame(&model.target),
                &action,
            )?;
            let after = abstract_frame(
                &self.target_environment.dispatch(&action)?,
                &action,
            )?;
            let result = model.core.target_trial(TransitionExample::new(
                10_000 + model.slot,
                before.action,
                before.state,
                after.state,
            ));
            if !matches!(result, ObservationVerdict::Confirmed) {
                return Err(format!("target confirmation failed {}", model.slot));
            }
        }
        Ok(())
    }

    fn sabotage_and_revise(&mut self) -> Result<(), String> {
        let model_index = 0usize;
        let mechanism = self.models[model_index].target.clone();
        let action = WorkshopAction {
            token: mechanism.action.clone(),
        };

        self.target_environment.reset();
        self.target_environment.set(&mechanism.effect, false)?;
        let before = abstract_frame(&self.target_environment.frame(&mechanism), &action)?;
        let after = abstract_frame(&self.target_environment.dispatch(&action)?, &action)?;
        let contradiction = self.models[model_index].core.target_trial(TransitionExample::new(
            20_010,
            before.action,
            before.state.clone(),
            after.state,
        ));
        if !matches!(contradiction, ObservationVerdict::ContradictionFound { .. }) {
            return Err(format!("sabotage did not contradict: {contradiction:?}"));
        }
        self.phase = WorkshopPhase::Contradicted;
        self.stage("FALSIFY", "CONTRADICTION_FOUND; prior permit quarantined");

        for _ in 0..MAX_PROBES {
            let base = abstract_frame(&self.target_environment.frame(&mechanism), &action)?;
            let probe = self.models[model_index]
                .core
                .choose_target_probe(&base.state)
                .ok_or_else(|| "no discriminating probe within budget".to_owned())?;
            self.probes += 1;
            for token in mechanism.guards.iter().chain(std::iter::once(&mechanism.effect)) {
                let node = base
                    .labels
                    .get(token)
                    .ok_or_else(|| format!("probe token missing {}", token))?;
                if let Some(value) = probe.before.get(u64::from(*node)) {
                    self.target_environment.set(token, value)?;
                }
            }
            let probe_before = abstract_frame(
                &self.target_environment.frame(&mechanism),
                &action,
            )?;
            let probe_after = abstract_frame(
                &self.target_environment.dispatch(&action)?,
                &action,
            )?;
            let result = self.models[model_index].core.apply_target_probe(
                TransitionExample::new(
                    20_010 + self.probes as u64,
                    probe_before.action,
                    probe_before.state,
                    probe_after.state,
                ),
            );
            if matches!(result, ObservationVerdict::Revised { .. }) {
                self.stage("ACTIVE_PROBE", format!("probe {} isolated the guard", self.probes));
                return Ok(());
            }
            if !matches!(result, ObservationVerdict::Abstain) {
                return Err(format!("unexpected probe result {result:?}"));
            }
            self.stage("ACTIVE_PROBE", format!("probe {} retained frontier", self.probes));
        }
        Err("probe budget exhausted".to_owned())
    }

    fn execute_plan(&mut self) -> Result<(), String> {
        self.target_environment.reset();
        for index in 0..self.spec.target.len() {
            let mechanism = self.spec.target[index].clone();
            if index == 0 {
                for guard in &mechanism.guards {
                    self.target_environment.set(guard, true)?;
                }
            }
            let model_index = self
                .models
                .iter()
                .position(|model| model.slot == mechanism.slot)
                .ok_or_else(|| format!("missing model {}", mechanism.slot))?;
            let action = WorkshopAction {
                token: mechanism.action.clone(),
            };
            let before = abstract_frame(
                &self.target_environment.frame(&mechanism),
                &action,
            )?;
            let predicted = self.models[model_index]
                .core
                .target()
                .and_then(|learner| learner.predict(before.action, &before.state))
                .ok_or_else(|| format!("no target prediction {}", mechanism.slot))?;
            let after = abstract_frame(
                &self.target_environment.dispatch(&action)?,
                &action,
            )?;
            if predicted != after.state {
                return Err(format!("target plan mismatch {}", mechanism.slot));
            }
            self.target_actions += 1;
            self.stage("ACT", format!("slot {} confirmed", mechanism.slot));
        }
        Ok(())
    }

    fn run_holdout(&mut self, positive: bool) -> Result<bool, String> {
        self.holdout_environment.reset();
        let first = self
            .spec
            .holdout
            .first()
            .ok_or_else(|| "empty holdout".to_owned())?
            .clone();
        if positive {
            for guard in &first.guards {
                self.holdout_environment.set(guard, true)?;
            }
        }

        for mechanism in &self.spec.holdout {
            let model = self
                .models
                .iter()
                .find(|model| model.slot == mechanism.slot)
                .ok_or_else(|| format!("missing holdout model {}", mechanism.slot))?;
            let mapping = match model
                .target_graph
                .find_unique_mapping(&model.holdout_graph, MappingBudget::new(MAPPING_STATES).unwrap())
            {
                MappingVerdict::Unique(mapping) => mapping,
                other => return Err(format!("holdout map failed: {other:?}")),
            };
            let target_rule = model
                .core
                .target()
                .and_then(|learner| learner.rule())
                .ok_or_else(|| format!("missing rule {}", mechanism.slot))?;
            let holdout_rule = target_rule
                .remap(&mapping, model.holdout_action)
                .ok_or_else(|| format!("cannot remap {}", mechanism.slot))?;
            let action = WorkshopAction {
                token: mechanism.action.clone(),
            };
            let before = abstract_frame(&self.holdout_environment.frame(mechanism), &action)?;
            let predicted = holdout_rule.predicts(&before.state);
            let after = abstract_frame(
                &self.holdout_environment.dispatch(&action)?,
                &action,
            )?;
            if predicted != after.state {
                return Ok(false);
            }
            if !positive && mechanism.slot == first.slot {
                return Ok(true);
            }
        }
        Ok(positive)
    }

    fn model_digest(&self) -> WorkshopDigest {
        self.models.iter().fold(0u64, |acc, model| {
            acc.rotate_left(7)
                ^ model.core.source().model_digest()
                ^ model
                    .core
                    .target()
                    .map(|learner| learner.model_digest())
                    .unwrap_or(0)
        })
    }

    fn stage(&mut self, stage: &str, detail: impl Into<String>) {
        self.trace.push(format!(
            "{{\"record\":\"stage\",\"stage\":{},\"detail\":{}}}",
            quote(stage),
            quote(&detail.into())
        ));
    }
}

fn portable_rule(slot: u64, rule: &ActionRule, epoch: u64) -> PortableRule {
    PortableRule {
        slot,
        action: rule.action,
        guards: rule
            .guards
            .iter()
            .map(|literal| PortableLiteral {
                role: literal.role,
                value: literal.value,
            })
            .collect(),
        effects: rule
            .effects
            .iter()
            .map(|literal| PortableLiteral {
                role: literal.role,
                value: literal.value,
            })
            .collect(),
        epoch,
    }
}

fn quote(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"<invalid>\"".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_worlds_change_tokens_but_keep_structural_slots() {
        let left = WorkshopSpec::generated(1);
        let right = WorkshopSpec::generated(2);
        assert_ne!(left.target[0].action, right.target[0].action);
        assert_eq!(
            left.target.iter().map(|item| item.slot).collect::<Vec<_>>(),
            right.target.iter().map(|item| item.slot).collect::<Vec<_>>()
        );
    }

    #[test]
    fn full_cycle_revises_composes_and_verifies_import() {
        let mut session = WorkshopSession::generated(0xA17E_2026).expect("session");
        let trace = session.run_full().expect("full cycle");
        assert_eq!(session.phase(), WorkshopPhase::HoldoutVerified, "{trace}");
        assert!(trace.contains("CONTRADICTION_FOUND"), "{trace}");
        assert!(trace.contains("\"holdout_positive\":true"), "{trace}");
        assert!(trace.contains("\"holdout_negative\":true"), "{trace}");
        let snapshot = session.export_snapshot().expect("snapshot");
        let imported =
            WorkshopSession::verify_imported_snapshot(&snapshot.to_json().expect("json"))
                .expect("import");
        assert!(imported.contains("\"retrained\":false"), "{imported}");
        assert!(imported.contains("\"holdout_positive\":true"), "{imported}");
    }

    #[test]
    fn invalid_slot_mapping_fails_closed() {
        let mut spec = WorkshopSpec::generated(9);
        spec.holdout[0].slot = 999;
        assert!(WorkshopSession::from_spec(spec).is_err());
    }
}
