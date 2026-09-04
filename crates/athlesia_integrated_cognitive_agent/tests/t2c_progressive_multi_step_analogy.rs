use athlesia_executive_agency::{
    EpistemicExecutableIntentionStep, EpistemicExecutiveAuthorizationResult,
    EpistemicExecutiveControl, EpistemicExecutiveControlPolicy,
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
struct ThreeStepWorld {
    initial: CognitiveStructure,
    first_action: CognitiveStructure,
    intermediate_one: CognitiveStructure,
    intermediate_two: CognitiveStructure,
    goal_action: CognitiveStructure,
    terminal: CognitiveStructure,
    initial_actions: Vec<CognitiveStructure>,
    intermediate_one_actions: Vec<CognitiveStructure>,
    intermediate_two_actions: Vec<CognitiveStructure>,
}

impl ThreeStepWorld {
    fn new(
        initial: u64,
        first_action: u64,
        intermediate_one: u64,
        intermediate_two: u64,
        goal_action: u64,
        terminal: u64,
    ) -> Self {
        Self {
            initial: a(initial),
            first_action: a(first_action),
            intermediate_one: a(intermediate_one),
            intermediate_two: a(intermediate_two),
            goal_action: a(goal_action),
            terminal: a(terminal),
            initial_actions: vec![a(initial + 1), a(initial + 2), a(first_action)],
            intermediate_one_actions: vec![a(goal_action - 2), a(goal_action - 1), a(goal_action)],
            intermediate_two_actions: vec![a(goal_action - 2), a(goal_action - 1), a(goal_action)],
        }
    }

    fn available_actions(&self, state: &CognitiveStructure) -> Vec<CognitiveStructure> {
        if state == &self.initial {
            self.initial_actions.clone()
        } else if state == &self.intermediate_one {
            self.intermediate_one_actions.clone()
        } else if state == &self.intermediate_two {
            self.intermediate_two_actions.clone()
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
            self.intermediate_one.clone()
        } else if state == &self.intermediate_one && action == &self.goal_action {
            self.intermediate_two.clone()
        } else if state == &self.intermediate_two && action == &self.goal_action {
            self.terminal.clone()
        } else {
            state.clone()
        }
    }

    fn identities(&self) -> Vec<CognitiveStructure> {
        let mut values = vec![
            self.initial.clone(),
            self.first_action.clone(),
            self.intermediate_one.clone(),
            self.intermediate_two.clone(),
            self.goal_action.clone(),
            self.terminal.clone(),
        ];

        values.extend(self.initial_actions.clone());
        values.extend(self.intermediate_one_actions.clone());
        values.extend(self.intermediate_two_actions.clone());
        values.sort();
        values.dedup();
        values
    }
}

fn exploration_policy() -> ModelFreeExplorationPolicy {
    ModelFreeExplorationPolicy::new(16).expect("positive exploration bound")
}

fn exploration_memory_policy() -> ModelFreeExplorationMemoryPolicy {
    ModelFreeExplorationMemoryPolicy::new(96).expect("positive memory bound")
}

fn minimal_executive_policy() -> EpistemicExecutiveControlPolicy {
    EpistemicExecutiveControlPolicy::new(s(1)).expect("positive executive threshold")
}

fn transfer_policy() -> OnlineGroundedEpisodicTransferPolicy {
    OnlineGroundedEpisodicTransferPolicy::new(
        GroundedEpisodicAnalogyPolicy::new(32, s(500)).expect("positive analogy policy"),
        EpistemicExecutiveControlPolicy::new(s(500))
            .expect("positive transfer authority threshold"),
    )
}

fn selected_model_free_action(
    world: &ThreeStepWorld,
    state: &CognitiveStructure,
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
        .expect("non-terminal state has available actions")
        .clone()
}

fn record_attempt(
    memory: ModelFreeExplorationMemory,
    state: &CognitiveStructure,
    action: &CognitiveStructure,
) -> ModelFreeExplorationMemory {
    memory
        .record_attempt(state.clone(), action.clone(), exploration_memory_policy())
        .expect("bounded exploration memory admits attempt")
}

fn model_free_environment_evidence(
    goal_identity: &CognitiveStructure,
    state: &CognitiveStructure,
    action: &CognitiveStructure,
    outcome: &CognitiveStructure,
    event_index: u64,
) -> EnvironmentInteractionEvidence {
    let step = EpistemicExecutableIntentionStep::new(state.clone(), action.clone(), None, s(1))
        .expect("grounded available action is executable");

    let authorization = EpistemicExecutiveControl::authorize(
        goal_identity,
        state,
        step,
        minimal_executive_policy(),
    );

    assert!(authorization.authorized());

    bind_authorized_environment_evidence(state, &authorization, outcome, event_index)
}

fn bind_authorized_environment_evidence(
    state: &CognitiveStructure,
    authorization: &EpistemicExecutiveAuthorizationResult,
    outcome: &CognitiveStructure,
    event_index: u64,
) -> EnvironmentInteractionEvidence {
    let dispatch = EnvironmentInteractionBoundary::dispatch_epistemic(state, authorization);

    assert!(dispatch.ready());

    let dispatch = dispatch
        .dispatch()
        .expect("authorized epistemic action must dispatch");

    let observation = EnvironmentInteractionObservation::new(event_index, outcome.clone(), s(900))
        .expect("positive environment observation");

    EnvironmentInteractionBoundary::bind_epistemic_observation(dispatch, &observation)
        .expect("environment feedback must bind into canonical evidence")
}

fn model_free_source_episode(world: &ThreeStepWorld) -> (GroundedSkillEpisode, usize) {
    let mut current = world.initial.clone();
    let mut memory = ModelFreeExplorationMemory::empty();
    let mut interactions = 0usize;
    let mut successful_steps = Vec::new();

    while current != world.terminal {
        let before = current.clone();
        let action = selected_model_free_action(world, &before, &memory);

        memory = record_attempt(memory, &before, &action);

        let after = world.execute(&before, &action);
        interactions = interactions.saturating_add(1);

        if after != before {
            successful_steps.push(
                GroundedSkillStep::new(before, action, after.clone(), s(900))
                    .expect("actual state-changing transition is grounded"),
            );
        }

        current = after;

        assert!(interactions <= 24);
    }

    let episode = GroundedSkillEpisode::new(
        world.initial.clone(),
        world.goal_action.clone(),
        successful_steps,
        s(900),
    )
    .expect("successful three-step trajectory forms grounded episode");

    (episode, interactions)
}

fn cold_solve(world: &ThreeStepWorld) -> usize {
    let mut current = world.initial.clone();
    let mut memory = ModelFreeExplorationMemory::empty();
    let mut interactions = 0usize;

    while current != world.terminal {
        let action = selected_model_free_action(world, &current, &memory);

        memory = record_attempt(memory, &current, &action);

        current = world.execute(&current, &action);
        interactions = interactions.saturating_add(1);

        assert!(interactions <= 24);
    }

    interactions
}

fn progressive_online_transfer(
    source_episode: GroundedSkillEpisode,
    world: &ThreeStepWorld,
) -> usize {
    let mut transfer_memory =
        OnlineGroundedEpisodicTransferMemory::new(world.initial.clone(), world.goal_action.clone());

    assert!(transfer_memory.remember_source_episode(source_episode));

    let mut exploration_memory = ModelFreeExplorationMemory::empty();
    let mut current = world.initial.clone();
    let mut interactions = 0usize;

    while current == world.initial {
        let before = current.clone();

        let action = selected_model_free_action(world, &before, &exploration_memory);

        exploration_memory = record_attempt(exploration_memory, &before, &action);

        let after = world.execute(&before, &action);
        interactions = interactions.saturating_add(1);

        let evidence = model_free_environment_evidence(
            &world.goal_action,
            &before,
            &action,
            &after,
            interactions as u64,
        );

        assert!(transfer_memory.record_environment_evidence(&evidence));

        current = after;
    }

    assert_eq!(current, world.intermediate_one);
    assert_eq!(interactions, 3);

    let first_transfer = OnlineGroundedEpisodicTransferRuntime::evaluate(
        &transfer_memory,
        &current,
        transfer_policy(),
    );

    println!(
        "T2C_STAGE first_transfer status={:?} candidates={} observations={}",
        first_transfer.status(),
        first_transfer
            .analogy()
            .map_or(0, |analogy| analogy.candidate_count()),
        transfer_memory.observation_count(),
    );

    assert_eq!(
        first_transfer.status(),
        OnlineGroundedEpisodicTransferStatus::Authorized
    );

    let first_selection = first_transfer
        .selection()
        .expect("first grounded prefix authorizes second action");

    assert_eq!(first_selection.required_state(), &world.intermediate_one);
    assert_eq!(first_selection.action(), &world.goal_action);
    assert_eq!(first_selection.predicted_outcome(), None);

    let first_authorization = first_transfer
        .authorization()
        .expect("first transfer retains executive authority");

    let after_first_transfer = world.execute(&current, first_selection.action());

    interactions = interactions.saturating_add(1);

    let transferred_evidence = bind_authorized_environment_evidence(
        &current,
        first_authorization,
        &after_first_transfer,
        interactions as u64,
    );

    assert!(transfer_memory.record_environment_evidence(&transferred_evidence));

    current = after_first_transfer;

    assert_eq!(current, world.intermediate_two);
    assert_eq!(interactions, 4);

    let second_transfer = OnlineGroundedEpisodicTransferRuntime::evaluate(
        &transfer_memory,
        &current,
        transfer_policy(),
    );

    println!(
        "T2C_STAGE second_transfer status={:?} candidates={} conflicting={} correspondence_conflict={} observations={}",
        second_transfer.status(),
        second_transfer
            .analogy()
            .map_or(0, |analogy| analogy.candidate_count()),
        second_transfer
            .analogy()
            .is_some_and(|analogy| analogy.conflicting_evidence()),
        second_transfer
            .analogy()
            .is_some_and(|analogy| analogy.correspondence_conflict()),
        transfer_memory.observation_count(),
    );

    assert_eq!(
        second_transfer.status(),
        OnlineGroundedEpisodicTransferStatus::Authorized,
        "after observing the transferred second step, episodic analogy must progressively re-anchor and authorize the third step instead of replaying the previous continuation"
    );

    let second_selection = second_transfer
        .selection()
        .expect("progressively re-anchored analogy authorizes final action");

    assert_eq!(second_selection.required_state(), &world.intermediate_two);
    assert_eq!(second_selection.action(), &world.goal_action);
    assert_eq!(second_selection.predicted_outcome(), None);

    let terminal = world.execute(&current, second_selection.action());

    interactions = interactions.saturating_add(1);
    current = terminal;

    assert_eq!(current, world.terminal);

    interactions
}

#[test]
fn three_step_source_and_cold_target_require_nine_model_free_interactions() {
    let source = ThreeStepWorld::new(100, 110, 210, 310, 700, 999);
    let target = ThreeStepWorld::new(900, 910, 1010, 1110, 20_000, 30_000);

    let (episode, source_interactions) = model_free_source_episode(&source);
    let cold_interactions = cold_solve(&target);

    assert_eq!(episode.steps().len(), 3);
    assert_eq!(source_interactions, 9);
    assert_eq!(cold_interactions, 9);
}

#[test]
fn one_source_episode_must_support_progressive_online_transfer_beyond_one_continuation() {
    let source = ThreeStepWorld::new(100, 110, 210, 310, 700, 999);
    let target = ThreeStepWorld::new(900, 910, 1010, 1110, 20_000, 30_000);

    for source_identity in source.identities() {
        assert!(
            !target.identities().contains(&source_identity),
            "source and target worlds must remain identity-disjoint"
        );
    }

    let (episode, source_interactions) = model_free_source_episode(&source);
    let cold_interactions = cold_solve(&target);

    let transferred_interactions = progressive_online_transfer(episode, &target);

    let saved_vs_cold = cold_interactions.saturating_sub(transferred_interactions);

    println!(
        "T2C_METRIC source_episodes=1 source_world={} cold_world_b={} online_transferred_world_b={} saved_vs_cold={}",
        source_interactions,
        cold_interactions,
        transferred_interactions,
        saved_vs_cold,
    );

    assert_eq!(source_interactions, 9);
    assert_eq!(cold_interactions, 9);
    assert_eq!(transferred_interactions, 5);
    assert_eq!(saved_vs_cold, 4);
}
