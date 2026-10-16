//! Independent host environment: rules live ONLY in this integration test.
//! The agent never receives World or Law, only graph/goal/catalogue/observations.
use athlesia_integrated_cognitive_agent::autonomous_causal_agent::*;
use athlesia_integrated_cognitive_agent::cross_domain_causal_core::{
    CanonicalCausalGraph, CausalNode, Literal, MappingBudget, MappingVerdict, NodeKind, State,
    TransitionExample,
};

#[path = "support/agent_world.rs"]
mod agent_world;
use agent_world::{run, World};

#[test]
fn empty_memory_solves_multi_action_goal_without_a_given_sequence() {
    for seed in 1..=100 {
        let mut world = World::new(seed);
        let mut agent =
            AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
                .unwrap();
        let (solved, count, _) = run(&mut agent, &mut world);
        assert!(
            solved,
            "cold seed={seed}, steps={count}, events={:?}",
            agent.events()
        );
        assert!(count >= 3);
        assert!(agent.evidence_count() > 0);
    }
}

#[test]
fn fuzz_128_cases_change_mechanisms_not_only_names() {
    let mut solved = 0;
    for seed in 0..128 {
        let mut world = World::variable(seed + 700, seed % 64);
        let mut agent =
            AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
                .unwrap();
        let (ok, steps, _) = run(&mut agent, &mut world);
        assert!(
            ok,
            "mechanism={}, tokens={}, steps={}, events={:?}",
            seed % 64,
            seed + 700,
            steps,
            agent.events()
        );
        solved += usize::from(ok);
    }
    assert_eq!(solved, 128);
}

#[test]
fn signed_two_guard_models_transfer_after_source_identification() {
    for mechanism in 0..64 {
        let source = World::variable(500 + mechanism, mechanism);
        let mut learned =
            AutonomousCausalAgent::new(source.task(), source.observation(), AgentBounds::default())
                .unwrap();
        source.teach_table(&mut learned);
        let mut world = World::variable(1500 + mechanism, mechanism);
        let mut target = learned
            .transfer_to(world.task(), world.observation())
            .unwrap();
        let (solved, steps, _) = run(&mut target, &mut world);
        assert!(solved, "mechanism={mechanism}");
        assert_eq!(steps, 3);
    }
}

#[test]
fn identical_observed_failure_is_not_retried_from_identical_state() {
    let mut world = World::new(2);
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    // Supply one REAL negative experience; no rule or successful plan supplied.
    let action = world.actions[2];
    agent
        .learn_trace(TransitionExample::new(
            1,
            action,
            world.state.clone(),
            world.state.clone(),
        ))
        .unwrap();
    let AgentDecision::Act(proposal) = agent.decide().unwrap() else {
        panic!()
    };
    assert_ne!(proposal.action, action);
    let _ = world.step(proposal.action);
}

#[test]
fn xor_outside_the_language_is_not_certified_by_full_truth_table() {
    let world = World::new(79);
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    for mask in 0..4 {
        let a = mask & 1 != 0;
        let b = mask & 2 != 0;
        let before = world
            .state
            .with(Literal::new(world.fields[0], a))
            .with(Literal::new(world.fields[1], b));
        let after = if a ^ b {
            before.with(Literal::new(world.fields[2], true))
        } else {
            before.clone()
        };
        agent
            .learn_trace(TransitionExample::new(
                mask,
                world.actions[2],
                before,
                after,
            ))
            .unwrap();
    }
    assert!(agent.candidates(world.actions[2]).unwrap().is_empty());
    assert_eq!(
        agent.decide().unwrap(),
        AgentDecision::Abstain(AbstainReason::OutOfModelClass)
    );
}

#[test]
fn repeat_task_reuses_retained_evidence_and_never_silently_forgets() {
    let mut improved = 0;
    for seed in 1..=30 {
        let mut world = World::new(seed);
        let mut agent =
            AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
                .unwrap();
        let (first, cold_steps, _) = run(&mut agent, &mut world);
        assert!(first);
        let retained = agent.evidence_count();
        world.reset();
        agent
            .begin_episode(world.observation(), world.task().goal)
            .unwrap();
        let (second, warm_steps, _) = run(&mut agent, &mut world);
        assert!(second);
        assert!(
            warm_steps <= cold_steps,
            "seed={seed}: warm={warm_steps}, cold={cold_steps}"
        );
        improved += usize::from(warm_steps < cold_steps);
        assert!(agent.evidence_count() >= retained);
    }
    assert!(improved > 0, "warm advantage must be measured, not assumed");
}

#[test]
fn exact_source_evidence_transfers_to_disjoint_tokens_first_attempt() {
    for seed in 1..=100 {
        let source = World::new(seed);
        let mut agent =
            AutonomousCausalAgent::new(source.task(), source.observation(), AgentBounds::default())
                .unwrap();
        source.teach_table(&mut agent);
        for action in source.actions {
            assert_eq!(agent.candidates(action).unwrap().len(), 1);
        }
        let saved = agent.export_memory().unwrap();
        let mut target_world = World::new(seed + 1000);
        let mut target = agent
            .transfer_to(target_world.task(), target_world.observation())
            .unwrap();
        let (solved, steps, reasons) = run(&mut target, &mut target_world);
        assert!(solved, "seed {seed}");
        assert_eq!(steps, 3);
        assert!(reasons
            .iter()
            .all(|r| *r == ActionReason::StructuralTransferTrial));
        assert_eq!(
            agent.export_memory().unwrap(),
            saved,
            "source memory cannot be rewritten by target experience"
        );
    }
}

#[test]
fn same_topology_different_law_falsifies_analogy_and_keeps_source_intact() {
    let source = World::new(7);
    let mut agent =
        AutonomousCausalAgent::new(source.task(), source.observation(), AgentBounds::default())
            .unwrap();
    source.teach_table(&mut agent);
    let memory = agent.export_memory().unwrap();
    let mut world = World::new(1007);
    // Identical topology; swap which operation enables the other.
    world.laws[0].guards = vec![Literal::new(world.fields[1], true)];
    world.laws[1].guards.clear();
    let mut target = agent
        .transfer_to(world.task(), world.observation())
        .unwrap();
    let (solved, _, _) = run(&mut target, &mut world);
    assert!(
        solved,
        "target must recover through its own evidence: {:?}",
        target.events()
    );
    assert!(target
        .events()
        .iter()
        .any(|e| e.verdict == EvidenceVerdict::TransferFalsified));
    assert_eq!(agent.export_memory().unwrap(), memory);
}

#[test]
fn source_prediction_of_inaction_does_not_suppress_a_target_experiment() {
    let mut source = World::new(11);
    source.laws[0].guards = vec![Literal::new(source.fields[1], true)];
    source.laws[1].guards = vec![Literal::new(source.fields[0], true)];
    let mut agent =
        AutonomousCausalAgent::new(source.task(), source.observation(), AgentBounds::default())
            .unwrap();
    source.teach_table(&mut agent);
    let mut world = World::new(1011);
    world.laws[0].guards.clear();
    let mut target = agent
        .transfer_to(world.task(), world.observation())
        .unwrap();
    let (solved, _, _) = run(&mut target, &mut world);
    assert!(solved);
    assert!(target.telemetry().transfer_falsifications > 0);
}

#[test]
fn one_trace_creates_candidates_not_unjustified_unique_identification() {
    let world = World::new(17);
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    let rule = &world.laws[0];
    agent
        .learn_trace(TransitionExample::new(
            1,
            rule.action,
            world.state.clone(),
            rule.predicts(&world.state),
        ))
        .unwrap();
    assert!(agent.candidates(rule.action).unwrap().len() > 1);
}

#[test]
fn contradictory_probe_revises_only_the_affected_action_and_keeps_every_witness() {
    let world = World::new(19);
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    // Successful example with both guards true is consistent with several laws.
    let action = world.actions[2];
    let before = world
        .state
        .with(Literal::new(world.fields[0], true))
        .with(Literal::new(world.fields[1], true));
    let after = before.with(Literal::new(world.fields[2], true));
    agent
        .learn_trace(TransitionExample::new(
            1,
            action,
            before.clone(),
            after.clone(),
        ))
        .unwrap();
    let unrelated = agent.candidates(world.actions[0]).unwrap().to_vec();
    let false_before = world.state.with(Literal::new(world.fields[0], true));
    agent
        .learn_trace(TransitionExample::new(
            2,
            action,
            false_before.clone(),
            false_before.clone(),
        ))
        .unwrap();
    assert_eq!(agent.candidates(world.actions[0]).unwrap(), unrelated);
    for candidate in agent.candidates(action).unwrap() {
        assert_eq!(candidate.predicts(&before), after);
        assert_eq!(candidate.predicts(&false_before), false_before);
    }
    assert_eq!(agent.evidence_count(), 2);
}

#[test]
fn active_selection_obeys_cost_and_rejects_unsafe_catalogue_actions() {
    let world = World::new(23);
    let mut task = world.task();
    task.actions.iter_mut().for_each(|a| a.cost = 100);
    let cheapest = world.actions[1];
    task.actions
        .iter_mut()
        .find(|a| a.id == cheapest)
        .unwrap()
        .cost = 1;
    let mut agent =
        AutonomousCausalAgent::new(task.clone(), world.observation(), AgentBounds::default())
            .unwrap();
    let AgentDecision::Act(proposal) = agent.decide().unwrap() else {
        panic!("must choose a probe")
    };
    assert_eq!(proposal.action, cheapest);
    assert!(proposal.worst_case_eliminated > 0);
    task.actions
        .iter_mut()
        .for_each(|a| a.sandbox_allowed = false);
    let mut denied =
        AutonomousCausalAgent::new(task, world.observation(), AgentBounds::default()).unwrap();
    assert_eq!(
        denied.decide().unwrap(),
        AgentDecision::Abstain(AbstainReason::NoDistinguishingReachableAction)
    );
}

#[test]
fn wrong_command_stale_sequence_and_cross_episode_are_rejected_without_mutation() {
    let mut world = World::new(29);
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    let AgentDecision::Act(proposal) = agent.decide().unwrap() else {
        panic!()
    };
    assert_eq!(agent.decide(), Err(AgentError::CommandPending));
    assert_eq!(
        agent.begin_episode(world.observation(), world.task().goal),
        Err(AgentError::CommandPending)
    );
    let next = world.step(proposal.action);
    assert_eq!(
        agent.observe(proposal.command + 1, next.clone()),
        Err(AgentError::WrongCommand)
    );
    let mut stale = next.clone();
    stale.sequence = 0;
    assert_eq!(
        agent.observe(proposal.command, stale),
        Err(AgentError::StaleObservation)
    );
    let mut other = next.clone();
    other.episode += 1;
    assert_eq!(
        agent.observe(proposal.command, other),
        Err(AgentError::StaleObservation)
    );
    assert_eq!(agent.evidence_count(), 0);
    agent.observe(proposal.command, next.clone()).unwrap();
    assert_eq!(
        agent.observe(proposal.command, next),
        Err(AgentError::NoPendingCommand)
    );
}

#[test]
fn unknown_or_partial_states_are_never_interpreted_as_false() {
    let world = World::new(31);
    let initial = AgentObservation {
        state: State::new([]),
        ..world.observation()
    };
    assert!(matches!(
        AutonomousCausalAgent::new(world.task(), initial, AgentBounds::default()),
        Err(AgentError::InvalidObservation(_))
    ));
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    let AgentDecision::Act(proposal) = agent.decide().unwrap() else {
        panic!()
    };
    let bad = AgentObservation {
        sequence: 1,
        state: State::new([Literal::new(world.fields[0], false)]),
        ..world.observation()
    };
    assert!(matches!(
        agent.observe(proposal.command, bad),
        Err(AgentError::InvalidObservation(_))
    ));
    assert_eq!(agent.evidence_count(), 0);
}

#[test]
fn unsupported_two_effect_action_must_abstain_not_fake_recovery() {
    let world = World::new(37);
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    let after = world
        .state
        .with(Literal::new(world.fields[0], true))
        .with(Literal::new(world.fields[1], true));
    let verdict = agent
        .learn_trace(TransitionExample::new(
            1,
            world.actions[0],
            world.state.clone(),
            after,
        ))
        .unwrap();
    assert_eq!(verdict, EvidenceVerdict::OutOfModelClass);
    assert_eq!(
        agent.decide().unwrap(),
        AgentDecision::Abstain(AbstainReason::OutOfModelClass)
    );
}

#[test]
fn opt_in_two_effect_grammar_learns_and_transfers_a_composed_action() {
    let bounds = AgentBounds {
        max_effects_per_action: 2,
        max_hypotheses_per_action: 1024,
        ..AgentBounds::default()
    };
    for seed in 0..128 {
        let mut source = World::new(10_000 + seed);
        source.laws[0].effects = vec![
            Literal::new(source.fields[0], true),
            Literal::new(source.fields[3], seed % 2 == 0),
        ];
        let mut learned =
            AutonomousCausalAgent::new(source.task(), source.observation(), bounds).unwrap();
        let (solved, steps, _) = run(&mut learned, &mut source);
        assert!(solved, "source seed={seed}, steps={steps}");
        assert!(learned
            .candidates(source.actions[0])
            .unwrap()
            .iter()
            .any(|rule| {
                rule.effects.len() == 2
                    && rule.effects.contains(&Literal::new(source.fields[0], true))
                    && rule
                        .effects
                        .contains(&Literal::new(source.fields[3], seed % 2 == 0))
            }));

        let mut target = World::new(20_000 + seed);
        target.laws[0].effects = vec![
            Literal::new(target.fields[0], true),
            Literal::new(target.fields[3], seed % 2 == 0),
        ];
        let mut transferred = learned
            .transfer_to(target.task(), target.observation())
            .unwrap();
        let (solved, steps, _) = run(&mut transferred, &mut target);
        assert!(solved, "target seed={seed}, steps={steps}");
    }
}

#[test]
fn effect_bound_rejects_unbounded_hypothesis_generation() {
    let world = World::new(91);
    let invalid = AgentBounds {
        max_effects_per_action: 3,
        ..AgentBounds::default()
    };
    assert!(matches!(
        AutonomousCausalAgent::new(world.task(), world.observation(), invalid),
        Err(AgentError::InvalidTask("invalid bounds"))
    ));
}

#[test]
fn multi_effect_transfer_falsifies_changed_target_law_without_mutating_source() {
    let bounds = AgentBounds {
        max_effects_per_action: 2,
        max_hypotheses_per_action: 1024,
        ..AgentBounds::default()
    };
    let mut source = World::new(30_001);
    source.laws[0].effects = vec![
        Literal::new(source.fields[0], true),
        Literal::new(source.fields[1], true),
    ];
    let mut learned =
        AutonomousCausalAgent::new(source.task(), source.observation(), bounds).unwrap();
    source.teach_table(&mut learned);
    let source_memory = learned.export_memory().unwrap();

    // The target preserves the visible graph shape but restores a different
    // law: the second field is no longer changed by the first action.
    let mut target = World::new(30_002);
    let mut transferred = learned
        .transfer_to(target.task(), target.observation())
        .unwrap();
    let (solved, _, _) = run(&mut transferred, &mut target);
    assert!(solved);
    assert!(transferred.telemetry().transfer_falsifications > 0);
    assert_eq!(learned.export_memory().unwrap(), source_memory);
}

#[test]
fn incompatible_historical_behavior_is_reported_not_erased_as_drift() {
    let world = World::new(41);
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    let action = world.actions[0];
    let before = world.state.clone();
    let after = before.with(Literal::new(world.fields[0], true));
    agent
        .learn_trace(TransitionExample::new(1, action, before.clone(), after))
        .unwrap();
    assert_eq!(
        agent
            .learn_trace(TransitionExample::new(2, action, before.clone(), before))
            .unwrap(),
        EvidenceVerdict::OutOfModelClass
    );
    assert_eq!(agent.evidence_count(), 2);
}

#[test]
fn memory_roundtrip_replays_evidence_and_preserves_next_decision() {
    let world = World::new(43);
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    world.teach_table(&mut agent);
    let memory = agent.export_memory().unwrap();
    let mut restored = AutonomousCausalAgent::restore(
        world.task(),
        world.observation(),
        AgentBounds::default(),
        &memory,
    )
    .unwrap();
    assert_eq!(restored.evidence_count(), agent.evidence_count());
    assert_eq!(restored.decide().unwrap(), agent.decide().unwrap());
}

#[test]
fn memory_rejects_a_different_effect_grammar() {
    let world = World::new(44);
    let a2 = AgentBounds {
        max_effects_per_action: 2,
        max_hypotheses_per_action: 1024,
        ..AgentBounds::default()
    };
    let agent = AutonomousCausalAgent::new(world.task(), world.observation(), a2).unwrap();
    let memory = agent.export_memory().unwrap();
    assert!(AutonomousCausalAgent::restore(
        world.task(),
        world.observation(),
        AgentBounds::default(),
        &memory,
    )
    .is_err());
    assert!(
        AutonomousCausalAgent::restore(world.task(), world.observation(), a2, &memory,).is_ok()
    );
}

#[test]
fn memory_refuses_wrong_domain_duplicates_unknown_fields_and_oversize() {
    let world = World::new(47);
    let agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    let memory = agent.export_memory().unwrap();
    let other = World::new(48);
    assert!(AutonomousCausalAgent::restore(
        other.task(),
        other.observation(),
        AgentBounds::default(),
        &memory
    )
    .is_err());
    let duplicate = memory.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert!(AutonomousCausalAgent::restore(
        world.task(),
        world.observation(),
        AgentBounds::default(),
        &duplicate
    )
    .is_err());
    let unknown = memory.replacen("{", "{\"success\":true,", 1);
    assert!(AutonomousCausalAgent::restore(
        world.task(),
        world.observation(),
        AgentBounds::default(),
        &unknown
    )
    .is_err());
    let small = AgentBounds {
        max_snapshot_bytes: 10,
        ..AgentBounds::default()
    };
    assert!(
        AutonomousCausalAgent::restore(world.task(), world.observation(), small, &memory).is_err()
    );
}

#[test]
fn work_evidence_step_and_event_budgets_are_real_stop_conditions() {
    let mut world = World::new(53);
    let tiny = AgentBounds {
        max_evaluations_per_call: 1,
        ..AgentBounds::default()
    };
    let mut agent = AutonomousCausalAgent::new(world.task(), world.observation(), tiny).unwrap();
    assert_eq!(
        agent.decide().unwrap(),
        AgentDecision::Abstain(AbstainReason::WorkBudget)
    );
    assert!(agent.telemetry().hypothesis_evaluations <= 1);
    let tiny = AgentBounds {
        max_evidence: 1,
        ..AgentBounds::default()
    };
    let mut agent = AutonomousCausalAgent::new(world.task(), world.observation(), tiny).unwrap();
    let noop = TransitionExample::new(
        1,
        world.actions[2],
        world.state.clone(),
        world.state.clone(),
    );
    agent.learn_trace(noop.clone()).unwrap();
    assert_eq!(
        agent.learn_trace(noop),
        Err(AgentError::Budget(AbstainReason::EvidenceBudget))
    );
    assert_eq!(agent.evidence_count(), 1);
    let tiny = AgentBounds {
        max_steps_per_episode: 1,
        ..AgentBounds::default()
    };
    let mut agent = AutonomousCausalAgent::new(world.task(), world.observation(), tiny).unwrap();
    let AgentDecision::Act(proposal) = agent.decide().unwrap() else {
        panic!()
    };
    agent
        .observe(proposal.command, world.step(proposal.action))
        .unwrap();
    assert_eq!(
        agent.decide().unwrap(),
        AgentDecision::Abstain(AbstainReason::StepBudget)
    );
    let tiny = AgentBounds {
        max_events: 1,
        ..AgentBounds::default()
    };
    let mut agent = AutonomousCausalAgent::new(world.task(), world.observation(), tiny).unwrap();
    let noop = TransitionExample::new(
        1,
        world.actions[2],
        world.state.clone(),
        world.state.clone(),
    );
    agent.learn_trace(noop.clone()).unwrap();
    assert_eq!(
        agent.learn_trace(noop),
        Err(AgentError::Budget(AbstainReason::EventBudget))
    );
}

#[test]
fn already_observed_goal_does_not_dispatch_a_gratuitous_action() {
    let mut world = World::new(59);
    world.state = world.state.with(Literal::new(world.fields[2], true));
    let mut agent =
        AutonomousCausalAgent::new(world.task(), world.observation(), AgentBounds::default())
            .unwrap();
    assert_eq!(agent.decide().unwrap(), AgentDecision::GoalObserved);
    assert_eq!(agent.telemetry().dispatched_proposals, 0);
}

#[test]
fn state_and_action_input_order_do_not_change_the_decision() {
    let world = World::new(61);
    let task = world.task();
    let mut reverse = task.clone();
    reverse.actions.reverse();
    let mut a =
        AutonomousCausalAgent::new(task, world.observation(), AgentBounds::default()).unwrap();
    let mut b =
        AutonomousCausalAgent::new(reverse, world.observation(), AgentBounds::default()).unwrap();
    assert_eq!(a.decide(), b.decide());
}

#[test]
fn graph_symmetry_requires_abstention_and_exhausted_uniqueness_is_not_proof() {
    let graph = CanonicalCausalGraph::new(
        vec![
            CausalNode {
                id: 1,
                kind: NodeKind::StateField,
                type_tag: 1,
            },
            CausalNode {
                id: 2,
                kind: NodeKind::StateField,
                type_tag: 1,
            },
        ],
        vec![],
    )
    .unwrap();
    assert!(matches!(
        graph.find_unique_mapping(&graph, MappingBudget::new(100).unwrap()),
        MappingVerdict::Ambiguous { .. }
    ));
    // DFS reaches its FIRST complete map at 3 visited states. The old mapper
    // incorrectly returned Unique here although the second map was unvisited.
    assert!(matches!(
        graph.find_unique_mapping(&graph, MappingBudget::new(3).unwrap()),
        MappingVerdict::ResourceLimit { .. }
    ));
}

#[test]
fn structural_transfer_resource_limit_cannot_be_silently_promoted() {
    let source = World::new(67);
    let bounds = AgentBounds {
        mapping_states: 1,
        ..AgentBounds::default()
    };
    let agent = AutonomousCausalAgent::new(source.task(), source.observation(), bounds).unwrap();
    let target = World::new(1067);
    assert!(agent
        .transfer_to(target.task(), target.observation())
        .is_err());
}

#[test]
fn forbidden_allocation_bounds_and_hypothesis_truncation_are_rejected() {
    let world = World::new(71);
    let bad = AgentBounds {
        max_fluents: usize::MAX,
        ..AgentBounds::default()
    };
    assert!(AutonomousCausalAgent::new(world.task(), world.observation(), bad).is_err());
    let bad = AgentBounds {
        max_hypotheses_per_action: 3,
        ..AgentBounds::default()
    };
    assert!(AutonomousCausalAgent::new(world.task(), world.observation(), bad).is_err());
}
