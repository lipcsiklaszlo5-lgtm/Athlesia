use athlesia_integrated_cognitive_agent::active_context_agent::{
    ActiveContextAgent, ActiveDecision, PartialObservation, TruthValue,
};
use athlesia_integrated_cognitive_agent::autonomous_causal_agent::AgentBounds;
use athlesia_integrated_cognitive_agent::cross_domain_causal_core::{
    CanonicalCausalGraph, CausalEdge, CausalNode, EdgeKind, Literal, NodeKind,
};
use athlesia_integrated_cognitive_agent::autonomous_causal_agent::{AgentTask, SandboxAction};
use std::time::Instant;

fn main() {
    let task = AgentTask {
        graph: CanonicalCausalGraph::new(
            vec![
                CausalNode { id: 11, kind: NodeKind::StateField, type_tag: 1 },
                CausalNode { id: 13, kind: NodeKind::StateField, type_tag: 2 },
                CausalNode { id: 17, kind: NodeKind::Operation, type_tag: 0 },
            ],
            vec![CausalEdge { from: 17, to: 13, kind: EdgeKind::Calls }],
        ).unwrap(),
        actions: vec![SandboxAction { id: 17, cost: 1, sandbox_allowed: true }],
        goal: vec![Literal::new(13, true)],
    };
    let start = Instant::now();
    let mut agent = ActiveContextAgent::new(
        task,
        PartialObservation::new(1, 0, [(11, TruthValue::Unknown), (13, TruthValue::False)]),
        vec![Literal::new(13, true)],
        [(11, 1)],
        AgentBounds::default(),
    ).unwrap();
    let ActiveDecision::Inspect(probe) = agent.decide().unwrap() else { panic!() };
    agent.observe_inspection(probe.command, probe.token, true).unwrap();
    let decision = agent.decide().unwrap();
    println!(r#"{{"record":"active_context","decision":"{:?}","unknown":{},"elapsed_ns":{}}}"#, decision, agent.unknown_count(), start.elapsed().as_nanos());
}
