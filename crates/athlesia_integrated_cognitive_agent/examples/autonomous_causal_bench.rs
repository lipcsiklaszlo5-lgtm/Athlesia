//! Native-only evidence harness. The private fixture is not part of the solver.
use athlesia_integrated_cognitive_agent::autonomous_causal_agent::*;
use std::time::Instant;

#[path = "../tests/support/agent_world.rs"]
mod agent_world;
use agent_world::{run, World};

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let mut cases = 128;
    let mut multi_effects = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--cases" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing case count".to_owned());
                };
                cases = value.parse::<usize>().map_err(|_| "invalid case count")?;
            }
            "--multi-effects" => multi_effects = true,
            _ => {
                return Err(
                    "usage: autonomous_causal_bench [--cases 1..=10000] [--multi-effects]"
                        .to_owned(),
                )
            }
        }
        index += 1;
    }
    if !(1..=10000).contains(&cases) {
        return Err("case bound must be 1..=10000".to_owned());
    }
    let mut cold_total = 0;
    let mut warm_total = 0;
    let mut transfer_total = 0;
    let mut improved = 0;
    let mut max_memory = 0;
    let mut walls = Vec::new();
    let bounds = if multi_effects {
        AgentBounds {
            max_effects_per_action: 2,
            max_hypotheses_per_action: 1024,
            ..AgentBounds::default()
        }
    } else {
        AgentBounds::default()
    };
    for case in 0..cases {
        let start = Instant::now();
        let mechanism = (case % 64) as u64;
        let mut world = if multi_effects {
            World::multi_effect(5000 + case as u64, case % 2 == 0)
        } else {
            World::variable(5000 + case as u64, mechanism)
        };
        let mut agent = AutonomousCausalAgent::new(world.task(), world.observation(), bounds)
            .map_err(|e| format!("new: {e:?}"))?;
        let (cold, cold_steps, _) = run(&mut agent, &mut world);
        if !cold {
            return Err(format!("cold failed in case {case}"));
        }
        let memory = agent
            .export_memory()
            .map_err(|e| format!("memory: {e:?}"))?;
        max_memory = max_memory.max(memory.len());
        world.reset();
        let mut warm =
            AutonomousCausalAgent::restore(world.task(), world.observation(), bounds, &memory)
                .map_err(|e| format!("restore: {e:?}"))?;
        let (solved, warm_steps, _) = run(&mut warm, &mut world);
        if !solved {
            return Err(format!("warm failed in case {case}"));
        }
        // Use only autonomously collected experience for this transfer.
        // The exact-source truth-table test is a separate, explicitly labelled test.
        let mut unknown = if multi_effects {
            World::multi_effect(25000 + case as u64, case % 2 == 0)
        } else {
            World::variable(25000 + case as u64, mechanism)
        };
        let mut target = warm
            .transfer_to(unknown.task(), unknown.observation())
            .map_err(|e| format!("transfer: {e:?}"))?;
        let (transferred, transfer_steps, _) = run(&mut target, &mut unknown);
        if !transferred {
            return Err(format!("transfer failed in case {case}"));
        }
        cold_total += cold_steps;
        warm_total += warm_steps;
        transfer_total += transfer_steps;
        improved += usize::from(warm_steps < cold_steps);
        walls.push(start.elapsed().as_nanos() as u64);
        if case == 0 {
            println!(
                "{}",
                serde_json::json!({"record":"sample", "mechanism":mechanism,
                "cold_actions":cold_steps, "warm_actions":warm_steps, "transfer_actions":transfer_steps,
                "memory_bytes":memory.len(), "source_evidence":warm.evidence_count(),
                "telemetry":agent.telemetry(), "events":agent.events()})
            );
        }
    }
    walls.sort_unstable();
    let percentile = |p: usize| {
        walls[((walls.len() * p).div_ceil(100))
            .saturating_sub(1)
            .min(walls.len() - 1)]
    };
    println!(
        "{}",
        serde_json::json!({"record":"benchmark", "cases":cases,
        "stationary_mechanism_classes": if multi_effects { 2 } else { cases.min(64) },
        "cold_solved":cases, "warm_solved":cases,
        "transfer_solved":cases, "cold_action_total":cold_total, "warm_action_total":warm_total,
        "transfer_action_total":transfer_total, "strict_warm_improvement_cases":improved,
        "max_serialized_local_evidence_bytes":max_memory,
        "whole_case_wall_ns_p50":percentile(50), "whole_case_wall_ns_p95":percentile(95),
        "scope": if multi_effects { "A2: 4 complete Boolean fluents; 3 opaque actions; bounded two-effect laws; typed isomorphic transfer" } else { "A1: 4 complete Boolean fluents; 3 opaque actions; signed 0..2-guard, single-assignment laws; typed isomorphic transfer" },
        "multi_effects":multi_effects,
        "human_intuition_claim":false, "physical_robot_tested":false, "neural_weights":0})
    );
    Ok(())
}
