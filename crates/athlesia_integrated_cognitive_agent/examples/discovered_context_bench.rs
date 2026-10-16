//! A4 native benchmark: discover an observable mechanism mode from traces.
use athlesia_integrated_cognitive_agent::autonomous_causal_agent::{
    AgentBounds, AgentObservation, AgentTask, SandboxAction,
};
use athlesia_integrated_cognitive_agent::cross_domain_causal_core::{
    CanonicalCausalGraph, CausalEdge, CausalNode, EdgeKind, Literal, NodeKind, State,
    TransitionExample,
};
use athlesia_integrated_cognitive_agent::discovered_context_causal_agent::{
    ContextDiscoveryStatus, DiscoveredContextCausalAgent, DiscoveryBounds, DiscoveryDecision,
    DiscoveryVerdict,
};
use serde_json::json;
use std::time::Instant;

#[derive(Clone, Copy)]
struct Fixture {
    mode: u64,
    x: u64,
    y: u64,
    target: u64,
    action: u64,
}

fn fixture(case: usize) -> Fixture {
    let base = 10_000 + (case as u64) * 100;
    Fixture {
        mode: base + 1,
        x: base + 3,
        y: base + 7,
        target: base + 11,
        action: base + 19,
    }
}

fn task(f: Fixture) -> AgentTask {
    AgentTask {
        graph: CanonicalCausalGraph::new(
            vec![(f.mode, 1), (f.x, 2), (f.y, 3), (f.target, 4)]
                .into_iter()
                .map(|(id, type_tag)| CausalNode {
                    id: id as u32,
                    kind: NodeKind::StateField,
                    type_tag,
                })
                .chain(std::iter::once(CausalNode {
                    id: f.action as u32,
                    kind: NodeKind::Operation,
                    type_tag: 0,
                }))
                .collect(),
            vec![CausalEdge {
                from: f.action as u32,
                to: f.target as u32,
                kind: EdgeKind::Calls,
            }],
        )
        .expect("fixture graph"),
        actions: vec![SandboxAction {
            id: f.action,
            cost: 1,
            sandbox_allowed: true,
        }],
        goal: vec![Literal::new(f.target, true)],
    }
}

fn observation(f: Fixture, episode: u64, mode: bool, x: bool, y: bool) -> AgentObservation {
    AgentObservation {
        episode,
        sequence: 0,
        state: State::new([
            Literal::new(f.mode, mode),
            Literal::new(f.x, x),
            Literal::new(f.y, y),
            Literal::new(f.target, false),
        ]),
    }
}

fn transitions(_f: Fixture) -> [(bool, bool, bool, bool); 8] {
    [
        (false, false, false, false),
        (false, false, true, false),
        (false, true, false, false),
        (false, true, true, true),
        (true, false, false, true),
        (true, false, true, false),
        (true, true, false, false),
        (true, true, true, false),
    ]
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let cases = match args.as_slice() {
        [_] => 128,
        [_, flag, count] if flag == "--cases" => {
            count.parse::<usize>().map_err(|_| "invalid case count")?
        }
        _ => return Err("usage: discovered_context_bench [--cases 1..=10000]".to_owned()),
    };
    if !(1..=10_000).contains(&cases) {
        return Err("case bound must be 1..=10000".to_owned());
    }

    let mut solved = 0usize;
    let mut discovered = 0usize;
    let mut unavailable = 0usize;
    let mut elapsed = Vec::with_capacity(cases);
    let mut max_history = 0usize;
    for case in 0..cases {
        let start = Instant::now();
        let f = fixture(case);
        let goal = Literal::new(f.target, true);
        let mut agent = DiscoveredContextCausalAgent::new(
            task(f),
            AgentBounds::default(),
            DiscoveryBounds::default(),
        )
        .map_err(|error| format!("new: {error:?}"))?;
        let mut saw_discovery = false;
        for (episode, (mode, x, y, target)) in transitions(f).into_iter().enumerate() {
            let initial = observation(f, episode as u64 + 1, mode, x, y);
            let example = TransitionExample::new(
                initial.episode,
                f.action,
                initial.state.clone(),
                initial.state.with(Literal::new(f.target, target)),
            );
            let verdict = agent
                .ingest_transition(vec![goal], example)
                .map_err(|error| format!("ingest: {error:?}"))?;
            saw_discovery |= matches!(verdict, DiscoveryVerdict::ContextDiscovered { .. });
        }
        discovered += usize::from(saw_discovery);
        unavailable += usize::from(matches!(
            agent.status(),
            ContextDiscoveryStatus::Unavailable
                | ContextDiscoveryStatus::Ambiguous
                | ContextDiscoveryStatus::ResourceExhausted
        ));

        let initial = observation(f, 9, true, false, false);
        agent
            .begin_episode(initial.clone(), vec![goal])
            .map_err(|error| format!("begin: {error:?}"))?;
        if let DiscoveryDecision::Act(proposal) = agent
            .decide()
            .map_err(|error| format!("decide: {error:?}"))?
        {
            let next = AgentObservation {
                episode: 9,
                sequence: 1,
                state: initial.state.with(Literal::new(f.target, true)),
            };
            agent
                .observe(&proposal, next)
                .map_err(|error| format!("observe: {error:?}"))?;
            if matches!(
                agent
                    .decide()
                    .map_err(|error| format!("verify: {error:?}"))?,
                DiscoveryDecision::GoalObserved
            ) {
                solved += 1;
            }
        }
        max_history = max_history.max(agent.history_count());
        elapsed.push(start.elapsed().as_nanos());
    }
    elapsed.sort_unstable();
    let percentile = |p: usize| elapsed[((elapsed.len() - 1) * p) / 100];
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "benchmark": "Athlesia bounded observable-context discovery A4",
            "cases": cases,
            "discovered": discovered,
            "post_discovery_solved": solved,
            "unavailable_or_ambiguous": unavailable,
            "max_history": max_history,
            "elapsed_ns_p50": percentile(50),
            "elapsed_ns_p95": percentile(95),
            "neural_weights": 0,
            "human_intuition_claim": false,
            "scope": "complete Boolean traces; 1..2 observable context fields; <=2 guard grammar; bounded replay",
        }))
        .expect("json")
    );
    Ok(())
}
