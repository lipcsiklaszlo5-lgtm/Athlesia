use athlesia_integrated_cognitive_agent::active_context_agent::{
    ActiveContextAgent, ActiveDecision, ActiveError, PartialObservation, TruthValue,
};
use athlesia_integrated_cognitive_agent::autonomous_causal_agent::{
    AgentBounds, AgentTask, SandboxAction,
};
use athlesia_integrated_cognitive_agent::cross_domain_causal_core::{
    CanonicalCausalGraph, CausalEdge, CausalNode, EdgeKind, Literal, NodeKind,
};

const SENSOR: u64 = 11;
const TARGET: u64 = 13;
const ACT: u64 = 17;

fn task() -> AgentTask {
    AgentTask {
        graph: CanonicalCausalGraph::new(
            vec![
                CausalNode { id: SENSOR as u32, kind: NodeKind::StateField, type_tag: 1 },
                CausalNode { id: TARGET as u32, kind: NodeKind::StateField, type_tag: 2 },
                CausalNode { id: ACT as u32, kind: NodeKind::Operation, type_tag: 0 },
            ],
            vec![CausalEdge { from: ACT as u32, to: TARGET as u32, kind: EdgeKind::Calls }],
        )
        .unwrap(),
        actions: vec![SandboxAction { id: ACT, cost: 1, sandbox_allowed: true }],
        goal: vec![Literal::new(TARGET, true)],
    }
}

#[test]
fn unknown_is_preserved_and_inspection_precedes_action() {
    let initial = PartialObservation::new(
        1,
        0,
        [(SENSOR, TruthValue::Unknown), (TARGET, TruthValue::False)],
    );
    let mut agent = ActiveContextAgent::new(
        task(),
        initial,
        vec![Literal::new(TARGET, true)],
        [(SENSOR, 1)],
        AgentBounds::default(),
    )
    .unwrap();
    let ActiveDecision::Inspect(probe) = agent.decide().unwrap() else { panic!() };
    assert_eq!(probe.token, SENSOR);
    assert_eq!(agent.observation().get(SENSOR), TruthValue::Unknown);
    agent.observe_inspection(probe.command, SENSOR, true).unwrap();
    assert_eq!(agent.unknown_count(), 0);
    assert!(matches!(agent.decide().unwrap(), ActiveDecision::Act(_)));
}

#[test]
fn missing_inspection_catalogue_fails_closed() {
    let initial = PartialObservation::new(
        1,
        0,
        [(SENSOR, TruthValue::Unknown), (TARGET, TruthValue::False)],
    );
    let mut agent = ActiveContextAgent::new(
        task(), initial, vec![Literal::new(TARGET, true)], [], AgentBounds::default(),
    )
    .unwrap();
    assert_eq!(agent.decide(), Err(ActiveError::InvalidObservation));
}

#[test]
fn wrong_inspection_receipt_does_not_mutate_state() {
    let initial = PartialObservation::new(
        1,
        0,
        [(SENSOR, TruthValue::Unknown), (TARGET, TruthValue::False)],
    );
    let mut agent = ActiveContextAgent::new(
        task(), initial, vec![Literal::new(TARGET, true)], [(SENSOR, 1)], AgentBounds::default(),
    )
    .unwrap();
    let ActiveDecision::Inspect(probe) = agent.decide().unwrap() else { panic!() };
    assert!(agent.observe_inspection(probe.command + 1, SENSOR, true).is_err());
    assert_eq!(agent.observation().get(SENSOR), TruthValue::Unknown);
}

#[test]
fn prior_evidence_survives_a_partial_next_episode_and_inspection() {
    let initial = PartialObservation::new(
        1,
        0,
        [(SENSOR, TruthValue::True), (TARGET, TruthValue::False)],
    );
    let mut agent = ActiveContextAgent::new(
        task(),
        initial,
        vec![Literal::new(TARGET, true)],
        [(SENSOR, 1)],
        AgentBounds::default(),
    )
    .unwrap();
    let ActiveDecision::Act(proposal) = agent.decide().unwrap() else { panic!() };
    agent
        .observe_action(
            proposal.command,
            PartialObservation::new(
                1,
                1,
                [(SENSOR, TruthValue::True), (TARGET, TruthValue::True)],
            ),
        )
        .unwrap();
    let retained = agent.learner().unwrap().evidence_count();
    assert!(retained > 0);

    agent
        .begin_episode(
            PartialObservation::new(
                2,
                0,
                [(SENSOR, TruthValue::Unknown), (TARGET, TruthValue::False)],
            ),
            vec![Literal::new(TARGET, true)],
        )
        .unwrap();
    let ActiveDecision::Inspect(probe) = agent.decide().unwrap() else { panic!() };
    agent.observe_inspection(probe.command, SENSOR, true).unwrap();
    assert_eq!(agent.learner().unwrap().evidence_count(), retained);
}
