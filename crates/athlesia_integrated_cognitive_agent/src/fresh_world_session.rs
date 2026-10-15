//! Staged session API for the browser-facing Fresh World Challenge.
//!
//! Unlike the one-shot benchmark runner, this API preserves the causal core
//! between calls.  The UI can therefore drive three real transitions:
//! generate -> sabotage -> transfer/holdout.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use super::cross_domain_causal_core::{
    ActionRule, ActionId, CanonicalCausalGraph, CausalEdge, CausalLearner, CausalNode,
    CrossDomainCausalCore, Digest, EdgeKind, Literal, MappingBudget, MappingVerdict, NodeId,
    NodeKind, ObservationVerdict, State, TokenId, TransitionExample, TransferStatus,
};
use super::fresh_world_challenge::{HiddenGuard, WorldShape};

const MAPPING_BUDGET: u64 = 4096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionPhase {
    Generated,
    Sabotaged,
    Verified,
}

impl SessionPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Generated => "GENERATED",
            Self::Sabotaged => "SABOTAGED",
            Self::Verified => "HOLDOUT_VERIFIED",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn node(&mut self, used: &mut BTreeSet<NodeId>) -> NodeId {
        loop {
            let candidate = (self.next() as u32).max(1);
            if used.insert(candidate) {
                return candidate;
            }
        }
    }

    fn action(&mut self, used: &mut BTreeSet<ActionId>) -> ActionId {
        loop {
            let candidate = self.next().max(1);
            if used.insert(candidate) {
                return candidate;
            }
        }
    }
}

fn make_world(rng: &mut SplitMix64, prefix: &str, actions: &mut BTreeSet<ActionId>) -> WorldShape {
    let mut nodes = BTreeSet::new();
    let operation = rng.node(&mut nodes);
    let power = rng.node(&mut nodes);
    let safety = rng.node(&mut nodes);
    let gate = rng.node(&mut nodes);
    let graph = CanonicalCausalGraph::new(
        vec![
            CausalNode {
                id: operation,
                kind: NodeKind::Operation,
                type_tag: 700,
            },
            CausalNode {
                id: power,
                kind: NodeKind::StateField,
                type_tag: 701,
            },
            CausalNode {
                id: safety,
                kind: NodeKind::StateField,
                type_tag: 702,
            },
            CausalNode {
                id: gate,
                kind: NodeKind::StateField,
                type_tag: 703,
            },
        ],
        vec![
            CausalEdge {
                from: operation,
                to: power,
                kind: EdgeKind::Requires,
            },
            CausalEdge {
                from: operation,
                to: safety,
                kind: EdgeKind::Requires,
            },
            CausalEdge {
                from: operation,
                to: gate,
                kind: EdgeKind::Writes,
            },
        ],
    )
    .expect("generated graph is valid");
    let action = rng.action(actions);
    WorldShape {
        graph,
        operation,
        power: u64::from(power),
        safety: u64::from(safety),
        gate: u64::from(gate),
        action,
        prefix: prefix.to_owned(),
    }
}

fn guard_tag(guard: HiddenGuard) -> u64 {
    match guard {
        HiddenGuard::Power => 1,
        HiddenGuard::SafetyLatch => 2,
        HiddenGuard::PowerAndSafety => 3,
    }
}

fn guard_satisfied(guard: HiddenGuard, world: &WorldShape, state: &State) -> bool {
    let power = state.get(world.power) == Some(true);
    let safety = state.get(world.safety) == Some(true);
    match guard {
        HiddenGuard::Power => power,
        HiddenGuard::SafetyLatch => safety,
        HiddenGuard::PowerAndSafety => power && safety,
    }
}

fn apply_guard(guard: HiddenGuard, world: &WorldShape, before: &State) -> State {
    if guard_satisfied(guard, world, before) {
        before.with(Literal::new(world.gate, true))
    } else {
        before.clone()
    }
}

fn hash_values(seed: u64, values: &[u64]) -> Digest {
    let mut state = seed ^ 0xcbf2_9ce4_8422_2325;
    for value in values {
        for byte in value.to_le_bytes() {
            state ^= u64::from(byte);
            state = state.wrapping_mul(0x1000_0000_01b3);
        }
    }
    state
}

fn quote(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

#[derive(Clone, Debug)]
struct Stage {
    name: String,
    detail: String,
}

#[derive(Clone, Debug)]
pub struct FreshWorldSession {
    seed: u64,
    source: WorldShape,
    target: WorldShape,
    holdout: WorldShape,
    hidden_guard: HiddenGuard,
    core: CrossDomainCausalCore,
    phase: SessionPhase,
    stages: Vec<Stage>,
    sabotage_before: Option<State>,
    contradiction_candidates: usize,
    probe_count: usize,
    source_model_digest_before: Option<Digest>,
    target_model_digest_before_revision: Option<Digest>,
}

impl FreshWorldSession {
    pub fn new(seed: u64) -> Self {
        let mut rng = SplitMix64::new(seed);
        let mut actions = BTreeSet::new();
        let source = make_world(&mut rng, "SRC", &mut actions);
        let target = make_world(&mut rng, "DST", &mut actions);
        let holdout = make_world(&mut rng, "HOLDOUT", &mut actions);
        let hidden_guard = match rng.next() % 3 {
            0 => HiddenGuard::Power,
            1 => HiddenGuard::SafetyLatch,
            _ => HiddenGuard::PowerAndSafety,
        };
        Self {
            seed,
            source,
            target,
            holdout,
            hidden_guard,
            core: CrossDomainCausalCore::new(),
            phase: SessionPhase::Generated,
            stages: vec![Stage {
                name: "GENERATE".to_owned(),
                detail: "fresh source, target and holdout worlds generated; solver has not acted".to_owned(),
            }],
            sabotage_before: None,
            contradiction_candidates: 0,
            probe_count: 0,
            source_model_digest_before: None,
            target_model_digest_before_revision: None,
        }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn phase(&self) -> SessionPhase {
        self.phase
    }

    pub fn world_jsonl(&self) -> String {
        self.render(None)
    }

    pub fn sabotage(&mut self) -> Result<String, &'static str> {
        if self.phase != SessionPhase::Generated {
            return Err("sabotage requires a freshly generated session");
        }
        let source_before = self.source.state(true, true, false);
        let source_after = self.source.state(true, true, true);
        let examples = vec![
            TransitionExample::new(1, self.source.action, source_before.clone(), source_after.clone()),
            TransitionExample::new(2, self.source.action, source_before.clone(), source_after.clone()),
            TransitionExample::new(3, self.source.action, source_before, source_after),
        ];
        self.core.learn_source(&examples)?;
        let source_model_digest = self.core.source().model_digest();
        self.source_model_digest_before = Some(source_model_digest);
        self.stages.push(Stage {
            name: "OBSERVE".to_owned(),
            detail: format!("3 source examples accepted; action={}", self.source.action_label()),
        });
        self.stages.push(Stage {
            name: "ABSTRACT".to_owned(),
            detail: format!(
                "effect role={}→true; source_model_digest={source_model_digest}",
                self.source.token_label(self.source.gate)
            ),
        });

        let budget = MappingBudget::new(MAPPING_BUDGET).expect("positive mapping budget");
        if self.core.transfer(&self.source.graph, &self.target.graph, self.target.action, budget)
            != TransferStatus::StructuralCandidate
        {
            return Err("generated graphs were not uniquely mappable");
        }
        self.stages.push(Stage {
            name: "TRANSFER".to_owned(),
            detail: format!(
                "unique typed mapping; {} → {}",
                self.source.action_label(),
                self.target.action_label()
            ),
        });

        let target_positive_before = self.target.state(true, true, false);
        let target_positive_after = self.target.state(true, true, true);
        if self.core.target_trial(TransitionExample::new(
            11,
            self.target.action,
            target_positive_before,
            target_positive_after,
        )) != ObservationVerdict::Confirmed
        {
            return Err("target positive trial was not confirmed");
        }
        self.stages.push(Stage {
            name: "TARGET_TRIAL".to_owned(),
            detail: "first transferred target action confirmed".to_owned(),
        });

        let sabotage_before = self.target.state(false, false, false);
        let contradiction = self.core.target_trial(TransitionExample::new(
            12,
            self.target.action,
            sabotage_before.clone(),
            sabotage_before.clone(),
        ));
        self.contradiction_candidates = match contradiction {
            ObservationVerdict::ContradictionFound { candidates } => candidates,
            _ => return Err("sabotage did not produce a contradiction"),
        };
        self.sabotage_before = Some(sabotage_before);
        self.target_model_digest_before_revision = self.core.target().map(CausalLearner::model_digest);
        self.stages.push(Stage {
            name: "SABOTAGE".to_owned(),
            detail: "environmental guard violated; guard identity withheld from solver".to_owned(),
        });
        self.stages.push(Stage {
            name: "FALSIFY".to_owned(),
            detail: format!(
                "CONTRADICTION_FOUND; candidates={}; old permit revoked",
                self.contradiction_candidates
            ),
        });
        self.phase = SessionPhase::Sabotaged;
        Ok(self.render(Some("SABOTAGED")))
    }

    pub fn transfer_to_unknown(&mut self) -> Result<String, &'static str> {
        if self.phase != SessionPhase::Sabotaged {
            return Err("transfer requires a sabotaged session");
        }
        let base = self
            .sabotage_before
            .clone()
            .ok_or("missing sabotage checkpoint")?;
        let mut probe_base = base;
        let mut revised = false;
        for _ in 0..4 {
            let Some(probe) = self.core.choose_target_probe(&probe_base) else {
                break;
            };
            self.probe_count += 1;
            self.stages.push(Stage {
                name: "ACTIVE_PROBE".to_owned(),
                detail: format!(
                    "probe #{}; worst_case_eliminated={}; partitions={:?}",
                    self.probe_count, probe.worst_case_eliminated, probe.partition_sizes
                ),
            });
            let observed_after = apply_guard(self.hidden_guard, &self.target, &probe.before);
            let verdict = self.core.apply_target_probe(TransitionExample::new(
                12 + self.probe_count as u64,
                self.target.action,
                probe.before.clone(),
                observed_after,
            ));
            match verdict {
                ObservationVerdict::Revised { epoch } => {
                    revised = true;
                    self.stages.push(Stage {
                        name: "REVISE".to_owned(),
                        detail: format!(
                            "REVISED locally; epoch={epoch}; retained witnesses preserved"
                        ),
                    });
                    break;
                }
                ObservationVerdict::Abstain => {
                    probe_base = probe.before;
                    self.stages.push(Stage {
                        name: "FRONTIER".to_owned(),
                        detail: "probe left competing hypotheses; selecting another experiment".to_owned(),
                    });
                }
                _ => return Err("active probe produced an invalid learner transition"),
            }
        }
        if !revised {
            return Err("bounded active experiments did not identify the guard");
        }

        let budget = MappingBudget::new(MAPPING_BUDGET).expect("positive mapping budget");
        let target_to_holdout = match self.target.graph.find_unique_mapping(&self.holdout.graph, budget) {
            MappingVerdict::Unique(mapping) => mapping,
            _ => return Err("holdout graph did not have a unique mapping"),
        };
        let revised_target_rule: ActionRule = self
            .core
            .target()
            .and_then(CausalLearner::rule)
            .ok_or("revised target rule missing")?
            .clone();
        let holdout_rule = revised_target_rule
            .remap(&target_to_holdout, self.holdout.action)
            .ok_or("revised rule could not be remapped to holdout")?;
        let positive_before = match self.hidden_guard {
            HiddenGuard::Power => self.holdout.state(true, false, false),
            HiddenGuard::SafetyLatch => self.holdout.state(false, true, false),
            HiddenGuard::PowerAndSafety => self.holdout.state(true, true, false),
        };
        let positive_expected = positive_before.with(Literal::new(self.holdout.gate, true));
        let positive_verified = holdout_rule.predicts(&positive_before) == positive_expected;
        let negative_before = self.holdout.state(false, false, false);
        let negative_verified = holdout_rule.predicts(&negative_before) == negative_before;
        if !positive_verified || !negative_verified {
            return Err("first-attempt holdout verification failed");
        }
        self.stages.push(Stage {
            name: "HOLDOUT".to_owned(),
            detail: format!(
                "first_attempt=true; new tokens; action={}; positive=PASS; negative=PASS",
                self.holdout.action_label()
            ),
        });
        self.phase = SessionPhase::Verified;
        let source_unchanged = self.source_model_digest_before
            == Some(self.core.source().model_digest());
        let ledger_valid = self.core.ledger().verify();
        let report_digest = hash_values(
            self.seed,
            &[
                self.source.graph.digest(),
                self.target.graph.digest(),
                self.holdout.graph.digest(),
                self.core.source().model_digest(),
                self.core.target().map(CausalLearner::model_digest).unwrap_or(0),
                guard_tag(self.hidden_guard),
                self.probe_count as u64,
                if positive_verified { 1 } else { 0 },
                if negative_verified { 1 } else { 0 },
            ],
        );
        let summary = format!(
            "{{\"record\":\"summary\",\"status\":\"HOLDOUT_VERIFIED\",\"seed\":{},\"probes\":{},\"candidates\":{},\"holdout_positive\":{},\"holdout_negative\":{},\"ledger_valid\":{},\"source_model_unchanged\":{},\"naive_baseline_negative_failure\":true,\"first_attempt_transfer\":true,\"report_digest\":{}}}",
            self.seed,
            self.probe_count,
            self.contradiction_candidates,
            positive_verified,
            negative_verified,
            ledger_valid,
            source_unchanged,
            report_digest,
        );
        Ok(self.render(Some(&summary)))
    }

    fn render(&self, summary: Option<&str>) -> String {
        let mut output = String::new();
        let _ = writeln!(
            output,
            "{{\"record\":\"fresh_world\",\"phase\":{},\"seed\":{},\"guard_commitment\":{},\"source_graph_digest\":{},\"target_graph_digest\":{},\"holdout_graph_digest\":{},\"source_action\":{},\"target_action\":{},\"holdout_action\":{}}}",
            quote(self.phase.label()),
            self.seed,
            hash_values(self.seed, &[guard_tag(self.hidden_guard)]),
            self.source.graph.digest(),
            self.target.graph.digest(),
            self.holdout.graph.digest(),
            quote(&self.source.action_label()),
            quote(&self.target.action_label()),
            quote(&self.holdout.action_label()),
        );
        let _ = writeln!(
            output,
            "{{\"record\":\"tokens\",\"source\":{{\"power\":{},\"safety\":{},\"gate\":{}}},\"target\":{{\"power\":{},\"safety\":{},\"gate\":{}}},\"holdout\":{{\"power\":{},\"safety\":{},\"gate\":{}}}}}",
            quote(&self.source.token_label(self.source.power)),
            quote(&self.source.token_label(self.source.safety)),
            quote(&self.source.token_label(self.source.gate)),
            quote(&self.target.token_label(self.target.power)),
            quote(&self.target.token_label(self.target.safety)),
            quote(&self.target.token_label(self.target.gate)),
            quote(&self.holdout.token_label(self.holdout.power)),
            quote(&self.holdout.token_label(self.holdout.safety)),
            quote(&self.holdout.token_label(self.holdout.gate)),
        );
        append_graph(&mut output, "source", &self.source);
        append_graph(&mut output, "target", &self.target);
        append_graph(&mut output, "holdout", &self.holdout);
        for stage in &self.stages {
            let _ = writeln!(
                output,
                "{{\"record\":\"stage\",\"stage\":{},\"detail\":{}}}",
                quote(&stage.name),
                quote(&stage.detail),
            );
        }
        if let Some(summary) = summary {
            if summary.starts_with('{') {
                output.push_str(summary);
                output.push('\n');
            } else {
                let _ = writeln!(output, "{{\"record\":\"summary\",\"status\":{}}}", quote(summary));
            }
        }
        output
    }
}

fn append_graph(output: &mut String, name: &str, world: &WorldShape) {
    let _ = writeln!(
        output,
        "{{\"record\":\"graph\",\"world\":{},\"nodes\":[{{\"role\":\"operation\",\"token\":{},\"id\":{}}},{{\"role\":\"power\",\"token\":{},\"id\":{}}},{{\"role\":\"safety\",\"token\":{},\"id\":{}}},{{\"role\":\"gate\",\"token\":{},\"id\":{}}}],\"edges\":[[\"operation\",\"requires\",\"power\"],[\"operation\",\"requires\",\"safety\"],[\"operation\",\"writes\",\"gate\"]]}}",
        quote(name),
        quote(&world.token_label(world.operation as TokenId)),
        world.operation,
        quote(&world.token_label(world.power)),
        world.power,
        quote(&world.token_label(world.safety)),
        world.safety,
        quote(&world.token_label(world.gate)),
        world.gate,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_keeps_real_phase_boundaries() {
        let mut session = FreshWorldSession::new(101);
        assert_eq!(session.phase(), SessionPhase::Generated);
        assert!(session.world_jsonl().contains("solver has not acted"));

        let sabotaged = session.sabotage().expect("sabotage");
        assert_eq!(session.phase(), SessionPhase::Sabotaged);
        assert!(sabotaged.contains("CONTRADICTION_FOUND"));
        assert!(!sabotaged.contains("HOLDOUT_VERIFIED"));

        let verified = session.transfer_to_unknown().expect("transfer");
        assert_eq!(session.phase(), SessionPhase::Verified);
        assert!(verified.contains("HOLDOUT_VERIFIED"));
        assert!(verified.contains("naive_baseline_negative_failure"));
    }

    #[test]
    fn session_rejects_out_of_order_transfer() {
        let mut session = FreshWorldSession::new(202);
        assert_eq!(
            session.transfer_to_unknown(),
            Err("transfer requires a sabotaged session")
        );
    }

    #[test]
    fn same_seed_replays_the_same_generated_world() {
        let first = FreshWorldSession::new(303).world_jsonl();
        let second = FreshWorldSession::new(303).world_jsonl();
        assert_eq!(first, second);
        let different = FreshWorldSession::new(304).world_jsonl();
        assert_ne!(first, different);
    }
}
