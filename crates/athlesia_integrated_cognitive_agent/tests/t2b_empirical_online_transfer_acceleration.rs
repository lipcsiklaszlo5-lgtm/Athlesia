use athlesia_executive_agency::{
    EpistemicExecutableIntentionStep, EpistemicExecutiveControl, EpistemicExecutiveControlPolicy,
};
use athlesia_integrated_cognitive_agent::{
    EnvironmentInteractionBoundary, EnvironmentInteractionEvidence,
    EnvironmentInteractionObservation, OnlineGroundedEpisodicTransferMemory,
    OnlineGroundedEpisodicTransferPolicy, OnlineGroundedEpisodicTransferRuntime,
    OnlineGroundedEpisodicTransferStatus,
};
use athlesia_meta_learning_skill_memory::{
    GroundedEpisodicAnalogyPolicy, GroundedSkillEpisode, GroundedSkillStep,
};
use athlesia_mindstone_sparse_cognition::{
    CognitiveSignal, CognitiveStructure, ModelFreeEpistemicExploration, ModelFreeExplorationMemory,
    ModelFreeExplorationMemoryPolicy, ModelFreeExplorationPolicy,
};

fn s(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).expect("positive bounded cognitive signal")
}

fn a(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

#[derive(Clone, Debug)]
struct TwoStepWorld {
    initial: CognitiveStructure,
    first_action: CognitiveStructure,
    intermediate: CognitiveStructure,
    goal_action: CognitiveStructure,
    terminal: CognitiveStructure,
    initial_actions: Vec<CognitiveStructure>,
    intermediate_actions: Vec<CognitiveStructure>,
}

impl TwoStepWorld {
    fn new(
        initial: u64,
        first_action: u64,
        intermediate: u64,
        goal_action: u64,
        terminal: u64,
    ) -> Self {
        Self {
            initial: a(initial),
            first_action: a(first_action),
            intermediate: a(intermediate),
            goal_action: a(goal_action),
            terminal: a(terminal),
            initial_actions: vec![a(initial + 1), a(initial + 2), a(first_action)],
            intermediate_actions: vec![a(goal_action - 2), a(goal_action - 1), a(goal_action)],
        }
    }

    fn available_actions(&self, state: &CognitiveStructure) -> Vec<CognitiveStructure> {
        if state == &self.initial {
            self.initial_actions.clone()
        } else if state == &self.intermediate {
            self.intermediate_actions.clone()
        } else {
            Vec::new()
        }
    }

    fn execute(
        &self,
        state: &CognitiveStructure,
        action: &CognitiveStructure,
    ) -> CognitiveStructure {
        if state == &self.initial && action == &self.first_action {
            self.intermediate.clone()
        } else if state == &self.intermediate && action == &self.goal_action {
            self.terminal.clone()
        } else {
            state.clone()
        }
    }

    fn identities(&self) -> Vec<CognitiveStructure> {
        let mut values = vec![
            self.initial.clone(),
            self.first_action.clone(),
            self.intermediate.clone(),
            self.goal_action.clone(),
            self.terminal.clone(),
        ];

        values.extend(self.initial_actions.clone());
        values.extend(self.intermediate_actions.clone());
        values.sort();
        values.dedup();
        values
    }
}

fn exploration_policy() -> ModelFreeExplorationPolicy {
    ModelFreeExplorationPolicy::new(16).expect("positive exploration bound")
}

fn memory_policy() -> ModelFreeExplorationMemoryPolicy {
    ModelFreeExplorationMemoryPolicy::new(64).expect("positive memory bound")
}

fn executive_policy() -> EpistemicExecutiveControlPolicy {
    EpistemicExecutiveControlPolicy::new(s(1)).expect("positive executive threshold")
}

fn transfer_policy() -> OnlineGroundedEpisodicTransferPolicy {
    OnlineGroundedEpisodicTransferPolicy::new(
        GroundedEpisodicAnalogyPolicy::new(16, s(500)).expect("positive analogy policy"),
        EpistemicExecutiveControlPolicy::new(s(500))
            .expect("positive transfer authority threshold"),
    )
}

fn selected_model_free_action(
    state: &CognitiveStructure,
    world: &TwoStepWorld,
    memory: &ModelFreeExplorationMemory,
) -> CognitiveStructure {
    let decision = ModelFreeEpistemicExploration::select(
        state,
        &world.available_actions(state),
        memory,
        exploration_policy(),
    );

    assert!(decision.selected());

    decision
        .selected_action()
        .expect("non-terminal state must expose an action")
        .clone()
}

fn record_attempt(
    memory: ModelFreeExplorationMemory,
    state: &CognitiveStructure,
    action: &CognitiveStructure,
) -> ModelFreeExplorationMemory {
    memory
        .record_attempt(state.clone(), action.clone(), memory_policy())
        .expect("bounded microworld exploration memory must admit attempt")
}

fn dispatch_model_free_observation(
    goal_identity: &CognitiveStructure,
    state: &CognitiveStructure,
    action: &CognitiveStructure,
    observed_outcome: &CognitiveStructure,
    event_index: u64,
) -> EnvironmentInteractionEvidence {
    let step = EpistemicExecutableIntentionStep::new(state.clone(), action.clone(), None, s(1))
        .expect("grounded available action has minimal execution authority");

    let authorization =
        EpistemicExecutiveControl::authorize(goal_identity, state, step, executive_policy());

    assert!(authorization.authorized());

    let dispatch = EnvironmentInteractionBoundary::dispatch_epistemic(state, &authorization);

    assert!(dispatch.ready());

    let dispatch = dispatch
        .dispatch()
        .expect("authorized model-free action must dispatch");

    assert_eq!(dispatch.action(), action);
    assert_eq!(dispatch.predicted_outcome(), None);

    let observation =
        EnvironmentInteractionObservation::new(event_index, observed_outcome.clone(), s(900))
            .expect("actual environment observation has positive confidence");

    EnvironmentInteractionBoundary::bind_epistemic_observation(dispatch, &observation)
        .expect("online observation must bind into canonical environment evidence")
}

fn model_free_source_episode(world: &TwoStepWorld) -> (GroundedSkillEpisode, usize) {
    let mut current = world.initial.clone();
    let mut memory = ModelFreeExplorationMemory::empty();
    let mut interactions = 0usize;
    let mut successful_steps = Vec::new();

    while current != world.terminal {
        let before = current.clone();
        let action = selected_model_free_action(&before, world, &memory);

        memory = record_attempt(memory, &before, &action);

        let after = world.execute(&before, &action);
        interactions = interactions.saturating_add(1);

        if after != before {
            successful_steps.push(
                GroundedSkillStep::new(before, action, after.clone(), s(900))
                    .expect("actual state-changing transition is a grounded skill step"),
            );
        }

        current = after;

        assert!(interactions <= 16);
    }

    let episode = GroundedSkillEpisode::new(
        world.initial.clone(),
        world.goal_action.clone(),
        successful_steps,
        s(900),
    )
    .expect("actual successful trajectory forms one grounded source episode");

    (episode, interactions)
}

fn cold_solve(world: &TwoStepWorld) -> usize {
    let mut current = world.initial.clone();
    let mut memory = ModelFreeExplorationMemory::empty();
    let mut interactions = 0usize;

    while current != world.terminal {
        let action = selected_model_free_action(&current, world, &memory);

        memory = record_attempt(memory, &current, &action);

        current = world.execute(&current, &action);
        interactions = interactions.saturating_add(1);

        assert!(interactions <= 16);
    }

    interactions
}

fn transferred_online_solve(source_episode: GroundedSkillEpisode, world: &TwoStepWorld) -> usize {
    let mut transfer_memory =
        OnlineGroundedEpisodicTransferMemory::new(world.initial.clone(), world.goal_action.clone());

    assert!(transfer_memory.remember_source_episode(source_episode));

    let mut exploration_memory = ModelFreeExplorationMemory::empty();
    let mut current = world.initial.clone();
    let mut interactions = 0usize;

    while current == world.initial {
        let before = current.clone();

        let action = selected_model_free_action(&before, world, &exploration_memory);

        exploration_memory = record_attempt(exploration_memory, &before, &action);

        let after = world.execute(&before, &action);

        interactions = interactions.saturating_add(1);

        let evidence = dispatch_model_free_observation(
            &world.goal_action,
            &before,
            &action,
            &after,
            interactions as u64,
        );

        assert!(transfer_memory.record_environment_evidence(&evidence));

        current = after;

        let transfer = OnlineGroundedEpisodicTransferRuntime::evaluate(
            &transfer_memory,
            &current,
            transfer_policy(),
        );

        let analogy = transfer
            .analogy()
            .expect("source episode plus target evidence produces an epistemic diagnosis");

        println!(
            "T2B_PREFIX interaction={} progressed={} status={:?} candidates={} conflicting={} correspondence_conflict={} observations={}",
            interactions,
            current != world.initial,
            transfer.status(),
            analogy.candidate_count(),
            analogy.conflicting_evidence(),
            analogy.correspondence_conflict(),
            transfer_memory.observation_count(),
        );

        if current == world.initial {
            assert!(
                !transfer.authorized(),
                "non-progress self-loop evidence must not authorize transferred continuation"
            );
        } else {
            assert_eq!(current, world.intermediate);

            assert_eq!(
                transfer.status(),
                OnlineGroundedEpisodicTransferStatus::Authorized,
                "once an actual structurally matching prefix is observed, irrelevant failed exploration must not block one-shot continuation"
            );

            let selection = transfer
                .selection()
                .expect("grounded target prefix must authorize transferred continuation");

            assert_eq!(selection.required_state(), &world.intermediate);
            assert_eq!(selection.action(), &world.goal_action);
            assert_eq!(selection.predicted_outcome(), None);

            let authorization = transfer
                .authorization()
                .expect("authorized transfer retains executive authority");

            let dispatch =
                EnvironmentInteractionBoundary::dispatch_epistemic(&current, authorization);

            assert!(dispatch.ready());

            let dispatch = dispatch
                .dispatch()
                .expect("transferred action must cross environment boundary");

            assert_eq!(dispatch.action(), &world.goal_action);
            assert_eq!(dispatch.predicted_outcome(), None);

            let terminal = world.execute(&current, dispatch.action());

            interactions = interactions.saturating_add(1);
            current = terminal;
        }

        assert!(interactions <= 16);
    }

    assert_eq!(current, world.terminal);

    interactions
}

#[test]
fn source_and_cold_controls_reproduce_t1b_six_interaction_baseline() {
    let source_world = TwoStepWorld::new(100, 110, 210, 700, 999);
    let target_world = TwoStepWorld::new(900, 910, 1010, 20_000, 30_000);

    let (source_episode, source_interactions) = model_free_source_episode(&source_world);

    let cold_target_interactions = cold_solve(&target_world);

    assert_eq!(source_episode.steps().len(), 2);
    assert_eq!(source_interactions, 6);
    assert_eq!(cold_target_interactions, 6);
}

#[test]
fn production_online_bridge_must_preserve_one_shot_sample_efficiency_with_full_feedback_stream() {
    let source_world = TwoStepWorld::new(100, 110, 210, 700, 999);
    let target_world = TwoStepWorld::new(900, 910, 1010, 20_000, 30_000);

    for source_identity in source_world.identities() {
        assert!(
            !target_world.identities().contains(&source_identity),
            "source and target microworld identities must be fully disjoint"
        );
    }

    let (source_episode, source_interactions) = model_free_source_episode(&source_world);

    let cold_target_interactions = cold_solve(&target_world);

    let transferred_target_interactions = transferred_online_solve(source_episode, &target_world);

    let saved_vs_cold = cold_target_interactions.saturating_sub(transferred_target_interactions);

    println!(
        "T2B_METRIC source_episodes=1 source_world={} cold_world_b={} online_transferred_world_b={} saved_vs_cold={}",
        source_interactions,
        cold_target_interactions,
        transferred_target_interactions,
        saved_vs_cold,
    );

    assert_eq!(source_interactions, 6);
    assert_eq!(cold_target_interactions, 6);
    assert_eq!(transferred_target_interactions, 4);
    assert_eq!(saved_vs_cold, 2);
}
