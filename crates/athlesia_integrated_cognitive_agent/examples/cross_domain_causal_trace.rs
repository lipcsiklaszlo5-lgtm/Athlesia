//! Deterministic, dependency-free replay of the bounded causal cycle.
//!
//! The example is intentionally a runner, not a second learner. It exercises
//! the public `CrossDomainCausalCore` API and prints JSONL records that can be
//! saved, diffed, and replayed independently of the terminal formatting.

use athlesia_integrated_cognitive_agent::cross_domain_causal_core::{
    CausalEdge, CausalLearner, CausalNode, CanonicalCausalGraph, CrossDomainCausalCore,
    EdgeKind, Literal, MappingBudget, NodeKind, ObservationVerdict, State, TraceKind,
    TransitionExample, TransferStatus,
};
use std::time::Instant;

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
    .expect("trace graph is valid")
}

fn source_state(power: bool, safety: bool, gate: bool) -> State {
    State::new([
        Literal::new(1, power),
        Literal::new(2, safety),
        Literal::new(3, gate),
    ])
}

fn target_state(power: bool, safety: bool, gate: bool) -> State {
    State::new([
        Literal::new(10, power),
        Literal::new(11, safety),
        Literal::new(12, gate),
    ])
}

/// Hidden target law used only by the sandbox after the contradiction.
/// The learner must identify this from the selected probe; it is not supplied
/// to the core as model metadata.
fn hidden_target_outcome(before: &State) -> State {
    if before.get(10) == Some(true) || before.get(11) == Some(true) {
        before.with(Literal::new(12, true))
    } else {
        before.clone()
    }
}

fn source_examples() -> Vec<TransitionExample> {
    (1..=3)
        .map(|episode| {
            TransitionExample::new(
                episode,
                9,
                source_state(true, true, false),
                source_state(true, true, true),
            )
        })
        .collect()
}

fn status_name(status: TransferStatus) -> &'static str {
    match status {
        TransferStatus::StructuralCandidate => "STRUCTURAL_CANDIDATE",
        TransferStatus::Supported => "SUPPORTED",
        TransferStatus::Falsified => "FALSIFIED",
        TransferStatus::Revised => "REVISED",
        TransferStatus::Ambiguous => "AMBIGUOUS",
        TransferStatus::Refused => "REFUSED",
    }
}

fn verdict_name(verdict: &ObservationVerdict) -> &'static str {
    match verdict {
        ObservationVerdict::Confirmed => "CONFIRMED",
        ObservationVerdict::ContradictionFound { .. } => "CONTRADICTION_FOUND",
        ObservationVerdict::Revised { .. } => "REVISED",
        ObservationVerdict::Abstain => "ABSTAIN",
    }
}

fn trace_kind_name(kind: TraceKind) -> &'static str {
    match kind {
        TraceKind::Observe => "OBSERVE",
        TraceKind::Abstract => "ABSTRACT",
        TraceKind::Transfer => "TRANSFER",
        TraceKind::Falsify => "FALSIFY",
        TraceKind::Revise => "REVISE",
        TraceKind::PermitRevoked => "PERMIT_REVOKED",
    }
}

fn emit_stage(stage: &str, detail: &str) {
    println!(r#"{{"record":"stage","stage":"{stage}","detail":"{detail}"}}"#);
}

fn emit_ledger(core: &CrossDomainCausalCore) {
    for event in core.ledger().events() {
        println!(
            r#"{{"record":"ledger","sequence":{},"epoch":{},"kind":"{}","payload_digest":{},"previous_hash":{},"record_hash":{}}}"#,
            event.sequence,
            event.epoch,
            trace_kind_name(event.kind),
            event.payload_digest,
            event.previous_hash,
            event.record_hash,
        );
    }
}

fn main() {
    let started = Instant::now();
    let mut core = CrossDomainCausalCore::new();
    let source_examples = source_examples();

    core.learn_source(&source_examples)
        .expect("source examples must form one bounded rule");
    emit_stage("OBSERVE", "3 source transition examples accepted");
    emit_stage("ABSTRACT", "source action rule retained");

    let transfer = core.transfer(
        &graph(1),
        &graph(10),
        19,
        MappingBudget::new(128).expect("positive mapping budget"),
    );
    assert_eq!(transfer, TransferStatus::StructuralCandidate);
    emit_stage("TRANSFER", "unique typed graph mapping installed");
    emit_stage("AUTHORITY", "UNKNOWN until target evidence exists");

    let positive = TransitionExample::new(
        11,
        19,
        target_state(true, true, false),
        target_state(true, true, true),
    );
    let positive_verdict = core.target_trial(positive);
    assert_eq!(positive_verdict, ObservationVerdict::Confirmed);
    emit_stage("TARGET_TRIAL", verdict_name(&positive_verdict));
    assert_eq!(core.status(), TransferStatus::Supported);

    let contradiction_before = target_state(false, false, false);
    let contradiction = TransitionExample::new(
        12,
        19,
        contradiction_before.clone(),
        contradiction_before.clone(),
    );
    let contradiction_verdict = core.target_trial(contradiction);
    assert!(matches!(
        contradiction_verdict,
        ObservationVerdict::ContradictionFound { .. }
    ));
    assert_eq!(core.status(), TransferStatus::Falsified);
    emit_stage("FALSIFY", verdict_name(&contradiction_verdict));
    emit_stage("AUTHORITY", "QUARANTINED; previous permit revoked");

    let probe = core
        .choose_target_probe(&contradiction_before)
        .expect("the bounded frontier must expose a discriminating probe");
    emit_stage("ACTIVE_PROBE", "minimax probe selected");
    let observed_probe = TransitionExample::new(
        13,
        19,
        probe.before.clone(),
        hidden_target_outcome(&probe.before),
    );
    let revision = core.apply_target_probe(observed_probe);
    assert!(matches!(revision, ObservationVerdict::Revised { .. }));
    assert_eq!(core.status(), TransferStatus::Revised);
    emit_stage("REVISE", verdict_name(&revision));

    let ledger_valid = core.ledger().verify();
    assert!(ledger_valid);
    emit_ledger(&core);
    println!(
        r#"{{"record":"summary","status":"{}","epoch":{},"ledger_valid":{},"events":{},"elapsed_ns":{}}}"#,
        status_name(core.status()),
        core.target().map(CausalLearner::epoch).unwrap_or(0),
        ledger_valid,
        core.ledger().events().len(),
        started.elapsed().as_nanos(),
    );
}
