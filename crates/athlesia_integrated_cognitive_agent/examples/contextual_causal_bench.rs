//! Native A3 benchmark: context reuse plus non-destructive epoch revision.
use athlesia_integrated_cognitive_agent::autonomous_causal_agent::{AgentBounds, AgentTask};
use athlesia_integrated_cognitive_agent::contextual_causal_agent::{
    ContextId, ContextObservation, ContextualCausalAgent, ContextualDecision, ContextualVerdict,
};
use athlesia_integrated_cognitive_agent::cross_domain_causal_core::{Literal, State};
use serde_json::json;
use std::time::Instant;

#[path = "../tests/support/agent_world.rs"]
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
) -> Result<(bool, bool, usize), String> {
    agent
        .begin_episode(context, world.observation(), vec![goal])
        .map_err(|error| format!("begin: {error:?}"))?;
    let mut epoch_rolled = false;
    for steps in 0..64 {
        match agent
            .decide()
            .map_err(|error| format!("decide: {error:?}"))?
        {
            ContextualDecision::GoalObserved => {
                return Ok((
                    world.state.get(goal.role) == Some(goal.value),
                    epoch_rolled,
                    steps,
                ));
            }
            ContextualDecision::Abstain(reason) => {
                return Err(format!("abstained: {reason:?}"));
            }
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
                    .map_err(|error| format!("observe: {error:?}"))?;
                epoch_rolled |= matches!(verdict, ContextualVerdict::EpochRolled { .. });
            }
        }
    }
    Err("episode step budget exhausted".to_owned())
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let cases = match args.as_slice() {
        [_] => 128,
        [_, flag, count] if flag == "--cases" => {
            count.parse::<usize>().map_err(|_| "invalid case count")?
        }
        _ => return Err("usage: contextual_causal_bench [--cases 1..=10000]".to_owned()),
    };
    if !(1..=10_000).contains(&cases) {
        return Err("case bound must be 1..=10000".to_owned());
    }

    let context = ContextId(0xA3);
    let mut walls = Vec::with_capacity(cases);
    let mut revised = 0usize;
    let mut first_solved = 0usize;
    let mut post_revision_solved = 0usize;
    let mut reuse_steps = 0usize;
    let mut max_evidence = 0usize;
    for case in 0..cases {
        let start = Instant::now();
        let mut world = World::new(80_000 + case as u64);
        let field = world.fields[0];
        let goal = Literal::new(field, true);
        world.state = state_with(&world, field, false);
        world.initial = world.state.clone();
        world.laws[0].effects = vec![Literal::new(field, true)];
        world.episode = 1;
        world.sequence = 0;
        let mut agent = ContextualCausalAgent::new(task_for(&world, goal), AgentBounds::default());

        let (solved, _, _) = run_episode(&mut agent, context, &mut world, goal)?;
        first_solved += usize::from(solved);

        // The same context now has a changed law. Action 0 is a no-op; action
        // 1 is the newly valid safe route to the same goal.
        world.episode = 2;
        world.sequence = 0;
        world.state = state_with(&world, field, false);
        world.laws[0].effects = vec![Literal::new(field, false)];
        world.laws[1].guards.clear();
        world.laws[1].effects = vec![Literal::new(field, true)];
        let (solved, epoch_rolled, _) = run_episode(&mut agent, context, &mut world, goal)?;
        revised += usize::from(epoch_rolled);
        post_revision_solved += usize::from(solved);

        // A third episode should reuse the new epoch, not repeat the old
        // action blindly.
        world.episode = 3;
        world.sequence = 0;
        world.state = state_with(&world, field, false);
        let (solved, _, steps) = run_episode(&mut agent, context, &mut world, goal)?;
        if !solved {
            return Err(format!("warm post-revision failure in case {case}"));
        }
        reuse_steps += steps;
        max_evidence = max_evidence.max(agent.total_retained_evidence());
        walls.push(start.elapsed().as_nanos() as u64);
    }
    walls.sort_unstable();
    let percentile = |p: usize| {
        walls[((walls.len() * p).div_ceil(100))
            .saturating_sub(1)
            .min(walls.len() - 1)]
    };
    println!(
        "{}",
        json!({
            "record": "contextual_benchmark",
            "cases": cases,
            "first_solved": first_solved,
            "epoch_revisions": revised,
            "post_revision_solved": post_revision_solved,
            "post_revision_reuse_solved": cases,
            "post_revision_reuse_action_total": reuse_steps,
            "max_retained_evidence": max_evidence,
            "wall_ns_p50": percentile(50),
            "wall_ns_p95": percentile(95),
            "contexts": 1,
            "human_intuition_claim": false,
            "neural_weights": 0,
            "scope": "A3: one opaque context token; same-context law drift; retained old epoch; bounded revised epoch"
        })
    );
    Ok(())
}
