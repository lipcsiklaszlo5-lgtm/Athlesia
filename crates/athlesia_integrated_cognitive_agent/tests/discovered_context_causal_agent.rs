//! A4 acceptance tests: discover an observable mechanism mode without a
//! host-provided context label, and abstain when no observable separator exists.

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

const MODE: u64 = 101;
const X: u64 = 103;
const Y: u64 = 107;
const TARGET: u64 = 109;
const ACT: u64 = 211;

fn task() -> AgentTask {
    AgentTask {
        graph: CanonicalCausalGraph::new(
            vec![
                CausalNode {
                    id: MODE as u32,
                    kind: NodeKind::StateField,
                    type_tag: 1,
                },
                CausalNode {
                    id: X as u32,
                    kind: NodeKind::StateField,
                    type_tag: 2,
                },
                CausalNode {
                    id: Y as u32,
                    kind: NodeKind::StateField,
                    type_tag: 3,
                },
                CausalNode {
                    id: TARGET as u32,
                    kind: NodeKind::StateField,
                    type_tag: 4,
                },
                CausalNode {
                    id: ACT as u32,
                    kind: NodeKind::Operation,
                    type_tag: 0,
                },
            ],
            vec![CausalEdge {
                from: ACT as u32,
                to: TARGET as u32,
                kind: EdgeKind::Calls,
            }],
        )
        .expect("valid graph"),
        actions: vec![SandboxAction {
            id: ACT,
            cost: 1,
            sandbox_allowed: true,
        }],
        goal: vec![Literal::new(TARGET, true)],
    }
}

fn observation(episode: u64, mode: bool, x: bool, y: bool, target: bool) -> AgentObservation {
    AgentObservation {
        episode,
        sequence: 0,
        state: State::new([
            Literal::new(MODE, mode),
            Literal::new(X, x),
            Literal::new(Y, y),
            Literal::new(TARGET, target),
        ]),
    }
}

fn step(
    agent: &mut DiscoveredContextCausalAgent,
    initial: AgentObservation,
    target_after: bool,
) -> DiscoveryVerdict {
    agent
        .ingest_transition(
            vec![Literal::new(TARGET, true)],
            TransitionExample::new(
                initial.episode,
                ACT,
                initial.state.clone(),
                initial.state.with(Literal::new(TARGET, target_after)),
            ),
        )
        .expect("observation")
}

fn live_step(
    agent: &mut DiscoveredContextCausalAgent,
    initial: AgentObservation,
    target_after: bool,
) -> DiscoveryVerdict {
    agent
        .begin_episode(initial.clone(), vec![Literal::new(TARGET, true)])
        .expect("episode begins");
    let DiscoveryDecision::Act(proposal) = agent.decide().expect("decision") else {
        panic!("expected one action")
    };
    agent
        .observe(
            &proposal,
            AgentObservation {
                episode: initial.episode,
                sequence: 1,
                state: initial.state.with(Literal::new(TARGET, target_after)),
            },
        )
        .expect("observation")
}

#[test]
fn discovers_minimal_observable_mode_and_reuses_it() {
    let mut agent = DiscoveredContextCausalAgent::new(
        task(),
        AgentBounds::default(),
        DiscoveryBounds::default(),
    )
    .expect("agent");

    // Mode=false: target iff X && Y. Mode=true: target iff !X && !Y. A
    // single global rule with at most two guards cannot express both branches,
    // while partitioning by MODE can.
    let cases = [
        (false, false, false, false),
        (false, false, true, false),
        (false, true, false, false),
        (false, true, true, true),
        (true, false, false, true),
        (true, false, true, false),
        (true, true, false, false),
        (true, true, true, false),
    ];
    let mut discovered = false;
    for (episode, (mode, x, y, target)) in cases.into_iter().enumerate() {
        let verdict = step(
            &mut agent,
            observation(episode as u64 + 1, mode, x, y, false),
            target,
        );
        discovered |= matches!(verdict, DiscoveryVerdict::ContextDiscovered { .. });
    }
    assert!(discovered, "the mode split must be evidenced by the traces");
    assert_eq!(
        agent.selected_fields(),
        Some([MODE].as_slice()),
        "the smallest unique observable separator is MODE"
    );
    assert_eq!(
        agent.status(),
        &ContextDiscoveryStatus::Identified {
            fields: vec![MODE],
            contexts: 2,
        }
    );

    let verdict = live_step(&mut agent, observation(9, true, false, false, false), true);
    assert!(matches!(
        verdict,
        DiscoveryVerdict::Consistent | DiscoveryVerdict::ContradictionAndLocalRevision
    ));
    assert_eq!(agent.context_count(), 2);
}

#[test]
fn refuses_to_invent_context_when_the_separator_is_unobservable() {
    let mut agent = DiscoveredContextCausalAgent::new(
        task(),
        AgentBounds::default(),
        DiscoveryBounds::default(),
    )
    .expect("agent");
    let first = step(&mut agent, observation(1, false, false, false, false), true);
    assert!(matches!(
        first,
        DiscoveryVerdict::Consistent | DiscoveryVerdict::ContradictionAndLocalRevision
    ));
    let second = step(
        &mut agent,
        observation(2, false, false, false, false),
        false,
    );
    assert!(matches!(second, DiscoveryVerdict::ContextUnavailable));
    assert_eq!(agent.status(), &ContextDiscoveryStatus::Unavailable);
    assert_eq!(agent.selected_fields(), None);
}
