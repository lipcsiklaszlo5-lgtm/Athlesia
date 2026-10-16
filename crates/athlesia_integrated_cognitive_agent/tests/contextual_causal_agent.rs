//! A3 acceptance tests: context-conditioned reuse and non-destructive epochs.
use athlesia_integrated_cognitive_agent::autonomous_causal_agent::{AgentBounds, AgentTask};
use athlesia_integrated_cognitive_agent::contextual_causal_agent::{
    ContextId, ContextObservation, ContextualCausalAgent, ContextualDecision, ContextualError,
    ContextualVerdict,
};
use athlesia_integrated_cognitive_agent::cross_domain_causal_core::{Literal, State};

#[path = "support/agent_world.rs"]
mod agent_world;
use agent_world::World;

fn state_with(world: &World, field: u64, value: bool) -> State {
    State::new(
        world
            .fields
            .iter()
            .map(|role| Literal::new(*role, if *role == field { value } else { false })),
    )
}

fn task_for(world: &World, goal: Literal) -> AgentTask {
    let mut task = world.task();
    task.goal = vec![goal];
    task
}

fn run_episode(
    agent: &mut ContextualCausalAgent,
    context: ContextId,
    world: &mut World,
    goal: Literal,
) -> (bool, Vec<ContextualVerdict>, usize) {
    agent
        .begin_episode(context, world.observation(), vec![goal])
        .expect("episode starts");
    let mut verdicts = Vec::new();
    for steps in 0..64 {
        match agent.decide().expect("decision protocol") {
            ContextualDecision::GoalObserved => {
                return (
                    world.state.get(goal.role) == Some(goal.value),
                    verdicts,
                    steps,
                )
            }
            ContextualDecision::Abstain(reason) => panic!("unexpected abstention: {reason:?}"),
            ContextualDecision::Act(proposal) => {
                let next = world.step(proposal.proposal.action);
                let verdict = agent
                    .observe(
                        &proposal,
                        ContextObservation {
                            context,
                            observation: next,
                        },
                    )
                    .expect("observation protocol");
                verdicts.push(verdict);
            }
        }
    }
    panic!("episode exceeded bounded test steps")
}

#[test]
fn distinct_observed_contexts_reuse_separate_rules_without_cross_contamination() {
    let mut world_a = World::new(71_001);
    world_a.state = state_with(&world_a, world_a.fields[0], false);
    world_a.initial = world_a.state.clone();
    world_a.laws[0].effects = vec![Literal::new(world_a.fields[0], true)];
    world_a.episode = 1;
    world_a.sequence = 0;
    let task = task_for(&world_a, Literal::new(world_a.fields[0], true));
    let mut agent = ContextualCausalAgent::new(task, AgentBounds::default());

    let context_a = ContextId(0xA);
    let context_b = ContextId(0xB);
    let goal_a = Literal::new(world_a.fields[0], true);
    let (solved_a, _, _) = run_episode(&mut agent, context_a, &mut world_a, goal_a);
    assert!(solved_a);

    let mut world_b = world_a.clone();
    world_b.episode = 2;
    world_b.sequence = 0;
    world_b.state = state_with(&world_b, world_b.fields[0], true);
    world_b.initial = world_b.state.clone();
    world_b.laws[0].effects = vec![Literal::new(world_b.fields[0], false)];
    let goal_b = Literal::new(world_b.fields[0], false);
    let (solved_b, _, _) = run_episode(&mut agent, context_b, &mut world_b, goal_b);
    assert!(solved_b);

    world_a.episode = 3;
    world_a.sequence = 0;
    world_a.state = state_with(&world_a, world_a.fields[0], false);
    let (reused_a, _, steps_a) = run_episode(&mut agent, context_a, &mut world_a, goal_a);
    assert!(reused_a);
    assert_eq!(steps_a, 1, "context A should reuse its learned action");
    assert_eq!(agent.context_count(), 2);
    assert_eq!(agent.epoch_count(context_a), 1);
    assert_eq!(agent.epoch_count(context_b), 1);
}

#[test]
fn contradictory_same_context_rolls_epoch_and_preserves_old_evidence() {
    let mut world = World::new(72_001);
    world.state = state_with(&world, world.fields[0], false);
    world.initial = world.state.clone();
    world.laws[0].effects = vec![Literal::new(world.fields[0], true)];
    world.episode = 1;
    world.sequence = 0;
    let field = world.fields[0];
    let goal_true = Literal::new(field, true);
    let mut agent = ContextualCausalAgent::new(task_for(&world, goal_true), AgentBounds::default());
    let (solved, _, _) = run_episode(&mut agent, ContextId(1), &mut world, goal_true);
    assert!(solved);
    let old_evidence = agent.total_retained_evidence();

    // Same context, same action, complete observation, opposite law. The
    // original action is now a no-op, while a second safe action can still
    // reach the goal after the new epoch explores it.
    world.episode = 2;
    world.sequence = 0;
    world.state = state_with(&world, field, false);
    world.laws[0].effects = vec![Literal::new(field, false)];
    world.laws[1].guards.clear();
    world.laws[1].effects = vec![Literal::new(field, true)];
    let (_, verdicts, _) = run_episode(&mut agent, ContextId(1), &mut world, goal_true);
    assert!(verdicts.iter().any(|verdict| {
        matches!(
            verdict,
            ContextualVerdict::EpochRolled {
                previous_epoch: 0,
                new_epoch: 1
            }
        )
    }));
    assert_eq!(agent.epoch_count(ContextId(1)), 2);
    assert!(agent.total_retained_evidence() > old_evidence);
    assert_eq!(agent.current_epoch(), Some(1));

    // The new epoch now reuses the revised law instead of repeating the old
    // failed prediction.
    world.episode = 3;
    world.sequence = 0;
    world.state = state_with(&world, field, false);
    let (recovered, _, steps) = run_episode(&mut agent, ContextId(1), &mut world, goal_true);
    assert!(recovered);
    assert_eq!(steps, 1);
}

#[test]
fn context_mismatch_does_not_consume_a_pending_action() {
    let world = World::new(73_001);
    let goal = Literal::new(world.fields[0], true);
    let mut agent = ContextualCausalAgent::new(task_for(&world, goal), AgentBounds::default());
    agent
        .begin_episode(ContextId(9), world.observation(), vec![goal])
        .unwrap();
    let ContextualDecision::Act(proposal) = agent.decide().unwrap() else {
        panic!("expected an exploratory action")
    };
    let wrong = ContextObservation {
        context: ContextId(10),
        observation: world.observation(),
    };
    assert!(matches!(
        agent.observe(&proposal, wrong),
        Err(ContextualError::ContextMismatch { .. })
    ));
}
