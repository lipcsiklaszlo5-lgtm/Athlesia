#![allow(dead_code)]
//! Independent host environment: rules live ONLY in this integration test.
//! The agent never receives World or Law, only graph/goal/catalogue/observations.
use athlesia_integrated_cognitive_agent::autonomous_causal_agent::*;
use athlesia_integrated_cognitive_agent::cross_domain_causal_core::{
    ActionRule, CanonicalCausalGraph, CausalEdge, CausalNode, EdgeKind, Literal, NodeKind, State,
    TransitionExample,
};

#[derive(Clone)]
pub struct World {
    // The first three fluents are the causal chain; fourth is a distractor.
    pub fields: [u64; 4],
    pub actions: [u64; 3],
    pub laws: Vec<ActionRule>,
    pub graph: CanonicalCausalGraph,
    pub state: State,
    pub initial: State,
    pub goal_value: bool,
    pub episode: u64,
    pub sequence: u64,
}

impl World {
    pub fn new(seed: u64) -> Self {
        // Relabelled, non-overlapping node IDs and shuffled action order.
        let base = 100 + (seed % 100_000) * 100;
        let fields = [base + 3, base + 11, base + 19, base + 29];
        let mut actions = [base + 41, base + 53, base + 67];
        actions.rotate_left((seed % 3) as usize);
        let laws = vec![
            ActionRule {
                action: actions[0],
                guards: vec![],
                effects: vec![Literal::new(fields[0], true)],
            },
            ActionRule {
                action: actions[1],
                guards: vec![Literal::new(fields[0], true)],
                effects: vec![Literal::new(fields[1], true)],
            },
            ActionRule {
                action: actions[2],
                guards: vec![Literal::new(fields[1], true)],
                effects: vec![Literal::new(fields[2], true)],
            },
        ];
        let mut nodes = Vec::new();
        for (i, id) in fields.iter().enumerate() {
            nodes.push(CausalNode {
                id: *id as u32,
                kind: NodeKind::StateField,
                type_tag: (i + 1) as u64,
            });
        }
        for id in actions {
            nodes.push(CausalNode {
                id: id as u32,
                kind: NodeKind::Operation,
                type_tag: 0,
            });
        }
        // Visible affordance structure, not guards: each operation has one
        // observed attachment. No `Requires` guard edge is leaked.
        let edges = actions
            .iter()
            .zip(fields)
            .map(|(a, f)| CausalEdge {
                from: *a as u32,
                to: f as u32,
                kind: EdgeKind::Calls,
            })
            .collect();
        let graph = CanonicalCausalGraph::new(nodes, edges).unwrap();
        let state = State::new(fields.map(|f| Literal::new(f, false)));
        Self {
            fields,
            actions,
            laws,
            graph,
            initial: state.clone(),
            state,
            goal_value: true,
            episode: 1,
            sequence: 0,
        }
    }

    /// A2 fixture: the first action changes the causal field and an otherwise
    /// irrelevant distractor field in the same transition. The solver receives
    /// neither this constructor nor the hidden law.
    pub fn multi_effect(seed: u64, distractor_value: bool) -> Self {
        let mut world = Self::new(seed);
        world.laws[0].effects = vec![
            Literal::new(world.fields[0], true),
            Literal::new(world.fields[3], distractor_value),
        ];
        world
    }

    /// 64 distinct stationary mechanisms: signed effects, signed guards,
    /// optional two-literal guards and an irrelevant fourth fluent.
    pub fn variable(tokens_seed: u64, mechanism: u64) -> Self {
        let mut world = Self::new(tokens_seed);
        let values = [mechanism & 1 != 0, mechanism & 2 != 0, mechanism & 4 != 0];
        let passive = mechanism & 32 != 0;
        for (i, value) in values.iter().enumerate() {
            world.laws[i].effects = vec![Literal::new(world.fields[i], *value)];
        }
        world.laws[1].guards = vec![Literal::new(world.fields[0], values[0])];
        if mechanism & 8 != 0 {
            world.laws[1]
                .guards
                .push(Literal::new(world.fields[3], passive));
        }
        world.laws[2].guards = vec![Literal::new(world.fields[1], values[1])];
        if mechanism & 16 != 0 {
            world.laws[2]
                .guards
                .push(Literal::new(world.fields[0], values[0]));
        }
        world.state = State::new([
            Literal::new(world.fields[0], !values[0]),
            Literal::new(world.fields[1], !values[1]),
            Literal::new(world.fields[2], !values[2]),
            Literal::new(world.fields[3], passive),
        ]);
        world.initial = world.state.clone();
        world.goal_value = values[2];
        world
    }

    pub fn task(&self) -> AgentTask {
        AgentTask {
            graph: self.graph.clone(),
            actions: self
                .actions
                .iter()
                .rev()
                .map(|id| SandboxAction {
                    id: *id,
                    cost: 1,
                    sandbox_allowed: true,
                })
                .collect(),
            goal: vec![Literal::new(self.fields[2], self.goal_value)],
        }
    }

    pub fn observation(&self) -> AgentObservation {
        AgentObservation {
            episode: self.episode,
            sequence: self.sequence,
            state: self.state.clone(),
        }
    }

    pub fn step(&mut self, action: u64) -> AgentObservation {
        let rule = self
            .laws
            .iter()
            .find(|r| r.action == action)
            .expect("catalogue action");
        self.state = environment_outcome(rule, &self.state);
        self.sequence += 1;
        self.observation()
    }

    pub fn reset(&mut self) {
        self.state = self.initial.clone();
        self.episode += 1;
        self.sequence = 0;
    }

    pub fn teach_table(&self, agent: &mut AutonomousCausalAgent) {
        // Deliberately full source evidence for the strongest transfer test.
        // This is NOT claimed to be one-shot or autonomous discovery.
        for mask in 0..16 {
            let before = State::new(
                self.fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Literal::new(*f, mask & (1 << i) != 0)),
            );
            for rule in &self.laws {
                agent
                    .learn_trace(TransitionExample::new(
                        100 + mask,
                        rule.action,
                        before.clone(),
                        environment_outcome(rule, &before),
                    ))
                    .unwrap();
            }
        }
    }
}

pub fn run(
    agent: &mut AutonomousCausalAgent,
    world: &mut World,
) -> (bool, usize, Vec<ActionReason>) {
    let mut reasons = Vec::new();
    for count in 0..128 {
        match agent.decide().expect("valid decision protocol") {
            AgentDecision::GoalObserved => {
                // Host-owned goal check: never trust a solver success label.
                let verified = world.state.get(world.fields[2]) == Some(world.goal_value);
                return (verified, count, reasons);
            }
            AgentDecision::Abstain(_) => return (false, count, reasons),
            AgentDecision::Act(proposal) => {
                reasons.push(proposal.reason);
                let observed = world.step(proposal.action);
                agent
                    .observe(proposal.command, observed)
                    .expect("valid observation");
            }
        }
    }
    (false, 128, reasons)
}

// Independent environment interpreter. It deliberately does not call the
// solver's ActionRule::predicts, so a shared implementation bug cannot make
// the learner and its evaluator agree on the same fabricated outcome.
fn environment_outcome(rule: &ActionRule, before: &State) -> State {
    let enabled = rule.guards.iter().all(|condition| {
        before
            .get(condition.role)
            .is_some_and(|v| v == condition.value)
    });
    State::new(before.literals().map(|old| {
        if enabled {
            rule.effects
                .iter()
                .find(|effect| effect.role == old.role)
                .copied()
                .unwrap_or(old)
        } else {
            old
        }
    }))
}
