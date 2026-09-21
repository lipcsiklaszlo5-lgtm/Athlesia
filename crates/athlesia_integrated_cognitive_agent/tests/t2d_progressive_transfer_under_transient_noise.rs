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
    CognitiveSignal::new(value).expect("positive bounded signal")
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

#[derive(Clone, Debug)]
struct TransientFailure {
    state: CognitiveStructure,
    action: CognitiveStructure,
    failures_remaining: usize,
}

impl TransientFailure {
    fn once(state: CognitiveStructure, action: CognitiveStructure) -> Self {
        Self {
            state,
            action,
            failures_remaining: 1,
        }
    }

    fn execute(
        &mut self,
        world: &ThreeStepWorld,
        state: &CognitiveStructure,
        action: &CognitiveStructure,
    ) -> CognitiveStructure {
        if self.failures_remaining > 0 && state == &self.state && action == &self.action {
            self.failures_remaining = self.failures_remaining.saturating_sub(1);
            return state.clone();
        }

        world.execute(state, action)
    }

    fn exhausted(&self) -> bool {
        self.failures_remaining == 0
    }
}

fn exploration_policy() -> ModelFreeExplorationPolicy {
    ModelFreeExplorationPolicy::new(16).expect("positive exploration bound")
}

fn exploration_memory_policy() -> ModelFreeExplorationMemoryPolicy {
    ModelFreeExplorationMemoryPolicy::new(128).expect("positive memory bound")
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
    let result = ModelFreeEpistemicExploration::select(
        state,
        &world.available_actions(state),
        memory,
        exploration_policy(),
    );

    assert!(result.selected());

    result
        .selected_action()
        .expect("non-terminal state exposes an action")
        .clone()
}

fn record_attempt(
    memory: ModelFreeExplorationMemory,
    state: &CognitiveStructure,
    action: &CognitiveStructure,
) -> ModelFreeExplorationMemory {
    memory
        .record_attempt(state.clone(), action.clone(), exploration_memory_policy())
        .expect("bounded memory admits attempt")
}

fn bind_dispatch_observation(
    state: &CognitiveStructure,
    authorization: &EpistemicExecutiveAuthorizationResult,
    outcome: &CognitiveStructure,
    event_index: u64,
) -> EnvironmentInteractionEvidence {
    let dispatch = EnvironmentInteractionBoundary::dispatch_epistemic(state, authorization);

    assert!(dispatch.ready());

    let dispatch = dispatch
        .dispatch()
        .expect("authorized action crosses environment boundary");

    let observation = EnvironmentInteractionObservation::new(event_index, outcome.clone(), s(900))
        .expect("positive environment observation");

    EnvironmentInteractionBoundary::bind_epistemic_observation(dispatch, &observation)
        .expect("environment observation binds into canonical evidence")
}

fn model_free_evidence(
    goal: &CognitiveStructure,
    state: &CognitiveStructure,
    action: &CognitiveStructure,
    outcome: &CognitiveStructure,
    event_index: u64,
) -> EnvironmentInteractionEvidence {
    let step = EpistemicExecutableIntentionStep::new(state.clone(), action.clone(), None, s(1))
        .expect("grounded available action is executable");

    let authorization =
        EpistemicExecutiveControl::authorize(goal, state, step, minimal_executive_policy());

    assert!(authorization.authorized());

    bind_dispatch_observation(state, &authorization, outcome, event_index)
}

fn source_episode(world: &ThreeStepWorld) -> (GroundedSkillEpisode, usize) {
    let mut current = world.initial.clone();
    let mut exploration = ModelFreeExplorationMemory::empty();
    let mut interactions = 0usize;
    let mut steps = Vec::new();

    while current != world.terminal {
        let before = current.clone();
        let action = selected_model_free_action(world, &before, &exploration);

        exploration = record_attempt(exploration, &before, &action);

        let after = world.execute(&before, &action);
        interactions = interactions.saturating_add(1);

        if after != before {
            steps.push(
                GroundedSkillStep::new(before, action, after.clone(), s(900))
                    .expect("state-changing source transition is grounded"),
            );
        }

        current = after;
        assert!(interactions <= 24);
    }

    (
        GroundedSkillEpisode::new(
            world.initial.clone(),
            world.goal_action.clone(),
            steps,
            s(900),
        )
        .expect("successful source trajectory forms an episode"),
        interactions,
    )
}

fn noisy_cold_solve(world: &ThreeStepWorld) -> usize {
    let mut noise =
        TransientFailure::once(world.intermediate_one.clone(), world.goal_action.clone());

    let mut current = world.initial.clone();
    let mut exploration = ModelFreeExplorationMemory::empty();
    let mut interactions = 0usize;

    while current != world.terminal {
        let action = selected_model_free_action(world, &current, &exploration);

        exploration = record_attempt(exploration, &current, &action);

        current = noise.execute(world, &current, &action);
        interactions = interactions.saturating_add(1);

        assert!(interactions <= 24);
    }

    assert!(noise.exhausted());

    interactions
}

fn noisy_progressive_transfer(episode: GroundedSkillEpisode, world: &ThreeStepWorld) -> usize {
    let mut noise =
        TransientFailure::once(world.intermediate_one.clone(), world.goal_action.clone());

    let mut transfer_memory =
        OnlineGroundedEpisodicTransferMemory::new(world.initial.clone(), world.goal_action.clone());

    assert!(transfer_memory.remember_source_episode(episode));

    let mut exploration = ModelFreeExplorationMemory::empty();
    let mut current = world.initial.clone();
    let mut interactions = 0usize;

    while current == world.initial {
        let before = current.clone();
        let action = selected_model_free_action(world, &before, &exploration);

        exploration = record_attempt(exploration, &before, &action);

        let after = noise.execute(world, &before, &action);
        interactions = interactions.saturating_add(1);

        let evidence = model_free_evidence(
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

    assert_eq!(
        first_transfer.status(),
        OnlineGroundedEpisodicTransferStatus::Authorized
    );

    let first_selection = first_transfer
        .selection()
        .expect("first target prefix authorizes transferred action");

    assert_eq!(first_selection.required_state(), &world.intermediate_one);
    assert_eq!(first_selection.action(), &world.goal_action);
    assert_eq!(first_selection.predicted_outcome(), None);

    let first_authorization = first_transfer
        .authorization()
        .expect("first transfer retains executive authority");

    let failed_outcome = noise.execute(world, &current, first_selection.action());

    interactions = interactions.saturating_add(1);

    assert_eq!(
        failed_outcome, current,
        "transient failure must produce exactly one self-loop"
    );
    assert!(noise.exhausted());

    let failed_evidence = bind_dispatch_observation(
        &current,
        first_authorization,
        &failed_outcome,
        interactions as u64,
    );

    assert!(transfer_memory.record_environment_evidence(&failed_evidence));

    let retry_transfer = OnlineGroundedEpisodicTransferRuntime::evaluate(
        &transfer_memory,
        &current,
        transfer_policy(),
    );

    println!(
        "T2D_STAGE after_transient_failure status={:?} candidates={} conflicting={} correspondence_conflict={} observations={}",
        retry_transfer.status(),
        retry_transfer
            .analogy()
            .map_or(0, |analogy| analogy.candidate_count()),
        retry_transfer
            .analogy()
            .is_some_and(|analogy| analogy.conflicting_evidence()),
        retry_transfer
            .analogy()
            .is_some_and(|analogy| analogy.correspondence_conflict()),
        transfer_memory.observation_count(),
    );

    assert_eq!(
        retry_transfer.status(),
        OnlineGroundedEpisodicTransferStatus::Authorized,
        "one transient failure must not erase an otherwise grounded progressive analogy"
    );

    let retry_selection = retry_transfer
        .selection()
        .expect("grounded analogy must retain retry authority");

    assert_eq!(retry_selection.required_state(), &world.intermediate_one);
    assert_eq!(retry_selection.action(), &world.goal_action);
    assert_eq!(retry_selection.predicted_outcome(), None);

    let retry_authorization = retry_transfer
        .authorization()
        .expect("retry retains executive authorization");

    let progressed = noise.execute(world, &current, retry_selection.action());

    interactions = interactions.saturating_add(1);

    assert_eq!(progressed, world.intermediate_two);

    let progressed_evidence = bind_dispatch_observation(
        &current,
        retry_authorization,
        &progressed,
        interactions as u64,
    );

    assert!(transfer_memory.record_environment_evidence(&progressed_evidence));
    current = progressed;

    let final_transfer = OnlineGroundedEpisodicTransferRuntime::evaluate(
        &transfer_memory,
        &current,
        transfer_policy(),
    );

    println!(
        "T2D_STAGE after_retry_success status={:?} candidates={} conflicting={} correspondence_conflict={} observations={}",
        final_transfer.status(),
        final_transfer
            .analogy()
            .map_or(0, |analogy| analogy.candidate_count()),
        final_transfer
            .analogy()
            .is_some_and(|analogy| analogy.conflicting_evidence()),
        final_transfer
            .analogy()
            .is_some_and(|analogy| analogy.correspondence_conflict()),
        transfer_memory.observation_count(),
    );

    assert_eq!(
        final_transfer.status(),
        OnlineGroundedEpisodicTransferStatus::Authorized
    );

    let final_selection = final_transfer
        .selection()
        .expect("successful retry progressively re-anchors final transfer");

    assert_eq!(final_selection.required_state(), &world.intermediate_two);
    assert_eq!(final_selection.action(), &world.goal_action);
    assert_eq!(final_selection.predicted_outcome(), None);

    let final_authorization = final_transfer
        .authorization()
        .expect("final transfer retains executive authority");

    let terminal = noise.execute(world, &current, final_selection.action());

    interactions = interactions.saturating_add(1);

    assert_eq!(terminal, world.terminal);

    let terminal_evidence = bind_dispatch_observation(
        &current,
        final_authorization,
        &terminal,
        interactions as u64,
    );

    assert!(transfer_memory.record_environment_evidence(&terminal_evidence));

    interactions
}

#[test]
fn transient_failure_increases_three_step_cold_cost_from_nine_to_twelve() {
    let world = ThreeStepWorld::new(900, 910, 1010, 1110, 20_000, 30_000);

    assert_eq!(noisy_cold_solve(&world), 12);
}

#[test]
fn progressive_one_shot_transfer_survives_transient_failure_of_transferred_action() {
    let source = ThreeStepWorld::new(100, 110, 210, 310, 700, 999);
    let target = ThreeStepWorld::new(900, 910, 1010, 1110, 20_000, 30_000);

    for source_identity in source.identities() {
        assert!(
            !target.identities().contains(&source_identity),
            "source and target identities must remain disjoint"
        );
    }

    let (episode, source_interactions) = source_episode(&source);
    let noisy_cold = noisy_cold_solve(&target);
    let noisy_transfer = noisy_progressive_transfer(episode, &target);

    let saved = noisy_cold.saturating_sub(noisy_transfer);

    println!(
        "T2D_METRIC source_episodes=1 source_world={} noisy_cold_world_b={} noisy_online_transferred_world_b={} saved_vs_noisy_cold={}",
        source_interactions,
        noisy_cold,
        noisy_transfer,
        saved,
    );

    assert_eq!(source_interactions, 9);
    assert_eq!(noisy_cold, 12);
    assert_eq!(noisy_transfer, 6);
    assert_eq!(saved, 6);
}
