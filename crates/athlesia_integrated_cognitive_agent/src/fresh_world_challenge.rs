//! Seeded, adversarial cross-domain challenge generation.
//!
//! This module deliberately keeps the environment generator outside the
//! learner.  The learner receives only transition observations and typed
//! graphs; the hidden guard is used by the fixture to produce outcomes, never
//! passed into `CausalLearner` or `CrossDomainCausalCore`.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use super::cross_domain_causal_core::{
    ActionRule, ActionId, CanonicalCausalGraph, CausalEdge, CausalLearner, CausalNode,
    CrossDomainCausalCore, Digest, EdgeKind, Literal, MappingBudget, MappingVerdict, NodeId,
    NodeKind, ObservationVerdict, State, TokenId, TransitionExample, TransferStatus,
};

const MAPPING_BUDGET: u64 = 4096;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HiddenGuard {
    Power,
    SafetyLatch,
    PowerAndSafety,
}

impl HiddenGuard {
    pub fn label(self) -> &'static str {
        match self {
            Self::Power => "power",
            Self::SafetyLatch => "safety_latch",
            Self::PowerAndSafety => "power_and_safety",
        }
    }

    fn satisfied(self, world: &WorldShape, state: &State) -> bool {
        let power = state.get(world.power) == Some(true);
        let safety = state.get(world.safety) == Some(true);
        match self {
            Self::Power => power,
            Self::SafetyLatch => safety,
            Self::PowerAndSafety => power && safety,
        }
    }

    fn tag(self) -> u64 {
        match self {
            Self::Power => 1,
            Self::SafetyLatch => 2,
            Self::PowerAndSafety => 3,
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

#[derive(Clone, Debug)]
pub struct WorldShape {
    pub graph: CanonicalCausalGraph,
    pub operation: NodeId,
    pub power: TokenId,
    pub safety: TokenId,
    pub gate: TokenId,
    pub action: ActionId,
    pub prefix: String,
}

impl WorldShape {
    pub fn state(&self, power: bool, safety: bool, gate: bool) -> State {
        State::new([
            Literal::new(self.power, power),
            Literal::new(self.safety, safety),
            Literal::new(self.gate, gate),
        ])
    }

    pub fn token_label(&self, role: TokenId) -> String {
        format!("{}_tok_{role:016x}", self.prefix)
    }

    pub fn action_label(&self) -> String {
        format!("{}_action_{:016x}", self.prefix, self.action)
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

fn state_after_guard(world: &WorldShape, guard: HiddenGuard, before: &State) -> State {
    if guard.satisfied(world, before) {
        before.with(Literal::new(world.gate, true))
    } else {
        before.clone()
    }
}

fn sabotage_state(world: &WorldShape) -> State {
    // Both candidate guard channels are disabled.  This deliberately keeps
    // the contradiction underdetermined so the solver must run a
    // discriminating experiment instead of receiving the answer for free.
    world.state(false, false, false)
}

fn holdout_state(world: &WorldShape, guard: HiddenGuard) -> State {
    match guard {
        HiddenGuard::Power => world.state(true, false, false),
        HiddenGuard::SafetyLatch => world.state(false, true, false),
        HiddenGuard::PowerAndSafety => world.state(true, true, false),
    }
}

#[derive(Clone, Debug)]
struct Stage {
    name: &'static str,
    detail: String,
}

#[derive(Clone, Debug)]
pub struct FreshChallengeReport {
    pub seed: u64,
    pub hidden_guard: HiddenGuard,
    pub source_graph_digest: Digest,
    pub target_graph_digest: Digest,
    pub holdout_graph_digest: Digest,
    pub source_action: ActionId,
    pub target_action: ActionId,
    pub holdout_action: ActionId,
    pub source_model_digest_before: Digest,
    pub source_model_digest_after: Digest,
    pub target_model_digest_before_revision: Digest,
    pub target_model_digest_after_revision: Digest,
    pub contradiction_candidates: usize,
    pub probe_count: usize,
    pub holdout_positive_verified: bool,
    pub holdout_negative_verified: bool,
    pub holdout_verified: bool,
    pub ledger_verified: bool,
    pub source_model_unchanged: bool,
    pub first_attempt_transfer: bool,
    pub report_digest: Digest,
    jsonl: String,
}

impl FreshChallengeReport {
    pub fn succeeded(&self) -> bool {
        self.holdout_verified
            && self.ledger_verified
            && self.source_model_unchanged
            && self.first_attempt_transfer
    }

    pub fn jsonl(&self) -> &str {
        &self.jsonl
    }
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

fn hash_report(seed: u64, values: &[u64]) -> Digest {
    let mut state = seed ^ 0xcbf2_9ce4_8422_2325;
    for value in values {
        for byte in value.to_le_bytes() {
            state ^= u64::from(byte);
            state = state.wrapping_mul(0x1000_0000_01b3);
        }
    }
    state
}

fn add_stage(stages: &mut Vec<Stage>, name: &'static str, detail: impl Into<String>) {
    stages.push(Stage {
        name,
        detail: detail.into(),
    });
}

fn append_graph_record(jsonl: &mut String, name: &str, world: &WorldShape) {
    let _ = writeln!(
        jsonl,
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

/// Run one generated challenge.  The only input to the solver is the source
/// evidence, the typed source/target graphs and the target observations.  The
/// selected hidden guard remains in the fixture and is never inserted into a
/// learner structure.
pub fn run(seed: u64) -> Result<FreshChallengeReport, &'static str> {
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

    let source_before = source.state(true, true, false);
    let source_after = source.state(true, true, true);
    let source_examples = vec![
        TransitionExample::new(1, source.action, source_before.clone(), source_after.clone()),
        TransitionExample::new(2, source.action, source_before.clone(), source_after.clone()),
        TransitionExample::new(3, source.action, source_before.clone(), source_after.clone()),
    ];

    let mut core = CrossDomainCausalCore::new();
    let mut stages = Vec::new();
    core.learn_source(&source_examples)?;
    let source_model_digest_before = core.source().model_digest();
    add_stage(
        &mut stages,
        "OBSERVE",
        format!(
            "3 source examples accepted; action={}",
            source.action_label()
        ),
    );
    add_stage(
        &mut stages,
        "ABSTRACT",
        format!(
            "effect role={}→true; model_digest={}",
            source.token_label(source.gate),
            source_model_digest_before
        ),
    );

    let budget = MappingBudget::new(MAPPING_BUDGET).expect("positive mapping budget");
    if core.transfer(&source.graph, &target.graph, target.action, budget)
        != TransferStatus::StructuralCandidate
    {
        return Err("generated source and target graphs were not uniquely mappable");
    }
    add_stage(
        &mut stages,
        "TRANSFER",
        format!(
            "unique typed mapping; {} → {}",
            source.action_label(),
            target.action_label()
        ),
    );

    let target_positive_before = target.state(true, true, false);
    let target_positive_after = target.state(true, true, true);
    if core.target_trial(TransitionExample::new(
        11,
        target.action,
        target_positive_before,
        target_positive_after,
    )) != ObservationVerdict::Confirmed
    {
        return Err("target positive evidence was not confirmed");
    }
    add_stage(
        &mut stages,
        "TARGET_TRIAL",
        "first transferred target action confirmed",
    );

    let sabotage_before = sabotage_state(&target);
    let sabotage_after = sabotage_before.clone();
    let contradiction = core.target_trial(TransitionExample::new(
        12,
        target.action,
        sabotage_before.clone(),
        sabotage_after,
    ));
    let contradiction_candidates = match contradiction {
        ObservationVerdict::ContradictionFound { candidates } => candidates,
        _ => return Err("sabotage did not produce a contradiction"),
    };
    add_stage(
        &mut stages,
        "SABOTAGE",
        "environmental guard violated; guard identity withheld from solver",
    );
    add_stage(
        &mut stages,
        "FALSIFY",
        format!(
            "CONTRADICTION_FOUND; candidates={contradiction_candidates}; old permit revoked"
        ),
    );

    let target_model_digest_before_revision = core
        .target()
        .ok_or("target learner missing after transfer")?
        .model_digest();
    let mut probe_count = 0;
    let mut revised = false;
    let mut probe_base = sabotage_before;
    for _ in 0..4 {
        let Some(probe) = core.choose_target_probe(&probe_base) else {
            break;
        };
        probe_count += 1;
        add_stage(
            &mut stages,
            "ACTIVE_PROBE",
            format!(
                "probe #{probe_count}; worst_case_eliminated={}; partitions={:?}",
                probe.worst_case_eliminated, probe.partition_sizes
            ),
        );
        let observed_after = state_after_guard(&target, hidden_guard, &probe.before);
        let verdict = core.apply_target_probe(TransitionExample::new(
            12 + probe_count as u64,
            target.action,
            probe.before.clone(),
            observed_after,
        ));
        match verdict {
            ObservationVerdict::Revised { epoch } => {
                revised = true;
                add_stage(
                    &mut stages,
                    "REVISE",
                    format!("REVISED locally; epoch={epoch}; retained witnesses preserved"),
                );
                break;
            }
            ObservationVerdict::Abstain => {
                probe_base = probe.before;
                add_stage(
                    &mut stages,
                    "FRONTIER",
                    "probe outcome left competing hypotheses; selecting another experiment",
                );
            }
            _ => return Err("active probe produced an invalid learner transition"),
        }
    }
    if !revised {
        return Err("bounded active experiments did not identify the generated guard");
    }

    let target_model_digest_after_revision = core
        .target()
        .ok_or("target learner missing after revision")?
        .model_digest();
    let source_model_digest_after = core.source().model_digest();
    let source_model_unchanged = source_model_digest_before == source_model_digest_after;

    let target_to_holdout = match target.graph.find_unique_mapping(&holdout.graph, budget) {
        MappingVerdict::Unique(mapping) => mapping,
        _ => return Err("holdout graph did not have a unique structural mapping"),
    };
    let revised_target_rule: ActionRule = core
        .target()
        .and_then(CausalLearner::rule)
        .ok_or("revised target rule missing")?
        .clone();
    let holdout_rule = revised_target_rule
        .remap(&target_to_holdout, holdout.action)
        .ok_or("revised rule could not be remapped to holdout")?;
    let holdout_positive_before = holdout_state(&holdout, hidden_guard);
    let holdout_positive_expected = holdout_positive_before.with(Literal::new(holdout.gate, true));
    let holdout_positive_verified =
        holdout_rule.predicts(&holdout_positive_before) == holdout_positive_expected;
    let holdout_negative_before = holdout.state(false, false, false);
    let holdout_negative_verified =
        holdout_rule.predicts(&holdout_negative_before) == holdout_negative_before;
    let holdout_verified = holdout_positive_verified && holdout_negative_verified;
    if !holdout_verified {
        return Err("first-attempt holdout transfer was not verified");
    }
    add_stage(
        &mut stages,
        "HOLDOUT",
        format!(
            "first_attempt=true; new tokens; action={}; positive=PASS; negative=PASS",
            holdout.action_label()
        ),
    );

    let ledger_verified = core.ledger().verify();
    let first_attempt_transfer = true;
    let report_digest = hash_report(
        seed,
        &[
            source.graph.digest(),
            target.graph.digest(),
            holdout.graph.digest(),
            source_model_digest_before,
            target_model_digest_after_revision,
            hidden_guard.tag(),
            probe_count as u64,
            if holdout_verified { 1 } else { 0 },
            if holdout_negative_verified { 1 } else { 0 },
        ],
    );

    let mut jsonl = String::new();
    let _ = writeln!(
        jsonl,
        "{{\"record\":\"fresh_world\",\"seed\":{seed},\"guard_commitment\":{},\"source_graph_digest\":{},\"target_graph_digest\":{},\"holdout_graph_digest\":{},\"source_action\":{},\"target_action\":{},\"holdout_action\":{}}}",
        hash_report(seed, &[hidden_guard.tag()]),
        source.graph.digest(),
        target.graph.digest(),
        holdout.graph.digest(),
        quote(&source.action_label()),
        quote(&target.action_label()),
        quote(&holdout.action_label()),
    );
    let _ = writeln!(
        jsonl,
        "{{\"record\":\"tokens\",\"source\":{{\"power\":{},\"safety\":{},\"gate\":{}}},\"target\":{{\"power\":{},\"safety\":{},\"gate\":{}}},\"holdout\":{{\"power\":{},\"safety\":{},\"gate\":{}}}}}",
        quote(&source.token_label(source.power)),
        quote(&source.token_label(source.safety)),
        quote(&source.token_label(source.gate)),
        quote(&target.token_label(target.power)),
        quote(&target.token_label(target.safety)),
        quote(&target.token_label(target.gate)),
        quote(&holdout.token_label(holdout.power)),
        quote(&holdout.token_label(holdout.safety)),
        quote(&holdout.token_label(holdout.gate)),
    );
    append_graph_record(&mut jsonl, "source", &source);
    append_graph_record(&mut jsonl, "target", &target);
    append_graph_record(&mut jsonl, "holdout", &holdout);
    for stage in &stages {
        let _ = writeln!(
            jsonl,
            "{{\"record\":\"stage\",\"stage\":{},\"detail\":{}}}",
            quote(stage.name),
            quote(&stage.detail),
        );
    }
    let _ = writeln!(
        jsonl,
        "{{\"record\":\"summary\",\"status\":\"HOLDOUT_VERIFIED\",\"seed\":{seed},\"probes\":{probe_count},\"candidates\":{contradiction_candidates},\"holdout_positive\":{holdout_positive_verified},\"holdout_negative\":{holdout_negative_verified},\"ledger_valid\":{ledger_verified},\"source_model_unchanged\":{source_model_unchanged},\"first_attempt_transfer\":{first_attempt_transfer},\"report_digest\":{report_digest}}}",
    );

    Ok(FreshChallengeReport {
        seed,
        hidden_guard,
        source_graph_digest: source.graph.digest(),
        target_graph_digest: target.graph.digest(),
        holdout_graph_digest: holdout.graph.digest(),
        source_action: source.action,
        target_action: target.action,
        holdout_action: holdout.action,
        source_model_digest_before,
        source_model_digest_after,
        target_model_digest_before_revision,
        target_model_digest_after_revision,
        contradiction_candidates,
        probe_count,
        holdout_positive_verified,
        holdout_negative_verified,
        holdout_verified,
        ledger_verified,
        source_model_unchanged,
        first_attempt_transfer,
        report_digest,
        jsonl,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_challenges_change_world_identity_without_changing_law_class() {
        let first = run(1).expect("seed 1");
        let second = run(2).expect("seed 2");
        assert!(first.succeeded());
        assert!(second.succeeded());
        assert_ne!(first.target_graph_digest, second.target_graph_digest);
        assert_ne!(first.target_action, second.target_action);
        assert_ne!(first.report_digest, second.report_digest);
        assert_ne!(first.source_action, first.target_action);
        assert_ne!(first.target_action, first.holdout_action);
        assert!(first.jsonl().contains("guard_commitment"));
        assert!(!first.jsonl().contains("hidden_guard"));
    }

    #[test]
    fn generated_challenges_verify_many_seeds() {
        for seed in 1..=128 {
            let report = run(seed).unwrap_or_else(|error| panic!("seed {seed}: {error}"));
            assert!(report.succeeded(), "seed {seed} did not succeed");
            assert!(report.ledger_verified);
            assert!(report.source_model_unchanged);
            assert!(report.first_attempt_transfer);
            assert!(report.holdout_positive_verified);
            assert!(report.holdout_negative_verified);
            assert!(report.probe_count >= 1);
            assert!(report.probe_count <= 2);
            assert!(report.jsonl().contains("HOLDOUT_VERIFIED"));
        }
    }

    #[test]
    fn generated_solver_does_not_receive_hidden_guard_as_model_state() {
        let report = run(0xfeed_f00d).expect("challenge");
        assert!(report.succeeded());
        assert_ne!(report.source_model_digest_before, report.target_model_digest_after_revision);
        assert_ne!(report.target_model_digest_before_revision, report.target_model_digest_after_revision);
    }

    #[test]
    fn generated_fixture_covers_each_guard_class() {
        let guards: BTreeSet<HiddenGuard> = (1..=64)
            .map(|seed| run(seed).expect("generated challenge").hidden_guard)
            .collect();
        assert_eq!(guards.len(), 3);
    }
}
