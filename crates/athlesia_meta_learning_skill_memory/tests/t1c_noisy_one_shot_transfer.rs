use athlesia_meta_learning_skill_memory::{
    GroundedEpisodicAnalogyPolicy, GroundedEpisodicAnalogyTransfer, GroundedSkillEpisode,
    GroundedSkillStep, SkillExecutionObservation,
};
use athlesia_mindstone_sparse_cognition::{
    CognitiveSignal, CognitiveStructure, ModelFreeEpistemicExploration, ModelFreeExplorationMemory,
    ModelFreeExplorationMemoryPolicy, ModelFreeExplorationPolicy,
};

#[derive(Clone, Debug)]
struct TwoStepWorld {
    initial: CognitiveStructure,
    first_action: CognitiveStructure,
    intermediate: CognitiveStructure,
    goal_action: CognitiveStructure,
    terminal: CognitiveStructure,
}

impl TwoStepWorld {
    fn new(base: u64, goal_action: u64, terminal: u64) -> Self {
        Self {
            initial: a(base),
            first_action: a(base + 10),
            intermediate: a(base + 110),
            goal_action: a(goal_action),
            terminal: a(terminal),
        }
    }

    fn available_actions(&self, state: &CognitiveStructure) -> Vec<CognitiveStructure> {
        if state == &self.initial {
            vec![
                a(atom_id(&self.initial) + 1),
                a(atom_id(&self.initial) + 2),
                self.first_action.clone(),
            ]
        } else if state == &self.intermediate {
            let goal = atom_id(&self.goal_action);
            vec![a(goal - 2), a(goal - 1), self.goal_action.clone()]
        } else {
            Vec::new()
        }
    }

    fn execute_clean(
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
        values.extend(self.available_actions(&self.initial));
        values.extend(self.available_actions(&self.intermediate));
        values.sort();
        values.dedup();
        values
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct TransientNoise {
    first_correct_attempts: usize,
}

impl TransientNoise {
    fn execute(
        &mut self,
        world: &TwoStepWorld,
        state: &CognitiveStructure,
        action: &CognitiveStructure,
    ) -> CognitiveStructure {
        if state == &world.initial && action == &world.first_action {
            self.first_correct_attempts = self.first_correct_attempts.saturating_add(1);
            if self.first_correct_attempts == 1 {
                return state.clone();
            }
        }
        world.execute_clean(state, action)
    }
}

fn s(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).expect("positive bounded signal")
}

fn a(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn atom_id(value: &CognitiveStructure) -> u64 {
    match value {
        CognitiveStructure::Atom(id) => *id,
        other => panic!("benchmark worlds use atomic identities only: {other:?}"),
    }
}

fn exploration_policy() -> ModelFreeExplorationPolicy {
    ModelFreeExplorationPolicy::new(16).expect("positive exploration frontier")
}

fn exploration_memory_policy() -> ModelFreeExplorationMemoryPolicy {
    ModelFreeExplorationMemoryPolicy::new(64).expect("positive exploration memory frontier")
}

fn analogy_policy() -> GroundedEpisodicAnalogyPolicy {
    GroundedEpisodicAnalogyPolicy::new(16, s(500)).expect("valid one-shot analogy policy")
}

fn clean_model_free_episode(world: &TwoStepWorld) -> (GroundedSkillEpisode, usize) {
    let mut memory = ModelFreeExplorationMemory::empty();
    let mut current = world.initial.clone();
    let mut successful_steps = Vec::new();
    let mut interaction_count = 0usize;

    while current != world.terminal {
        assert!(interaction_count < 32, "clean microworld must terminate");

        let decision = ModelFreeEpistemicExploration::select(
            &current,
            &world.available_actions(&current),
            &memory,
            exploration_policy(),
        );
        let action = decision
            .selected_action()
            .cloned()
            .expect("unsolved clean state must expose an action");
        let next = world.execute_clean(&current, &action);

        memory = memory
            .record_attempt(current.clone(), action.clone(), exploration_memory_policy())
            .expect("benchmark exploration memory has sufficient capacity");
        interaction_count = interaction_count.saturating_add(1);

        if next != current {
            successful_steps.push(
                GroundedSkillStep::new(current.clone(), action, next.clone(), s(1000))
                    .expect("observed successful transition is valid"),
            );
        }

        current = next;
    }

    (
        GroundedSkillEpisode::new(
            world.initial.clone(),
            world.goal_action.clone(),
            successful_steps,
            s(1000),
        )
        .expect("clean interaction trace forms a grounded skill episode"),
        interaction_count,
    )
}

fn noisy_cold_solve(world: &TwoStepWorld) -> usize {
    let mut memory = ModelFreeExplorationMemory::empty();
    let mut current = world.initial.clone();
    let mut noise = TransientNoise::default();
    let mut interactions = 0usize;

    while current != world.terminal {
        assert!(interactions < 48, "noisy cold microworld must terminate");

        let decision = ModelFreeEpistemicExploration::select(
            &current,
            &world.available_actions(&current),
            &memory,
            exploration_policy(),
        );
        let action = decision
            .selected_action()
            .cloned()
            .expect("unsolved noisy state must expose an action");
        let next = noise.execute(world, &current, &action);

        memory = memory
            .record_attempt(current.clone(), action, exploration_memory_policy())
            .expect("benchmark exploration memory has sufficient capacity");

        interactions = interactions.saturating_add(1);
        current = next;
    }

    interactions
}

fn noisy_one_shot_transfer_solve(
    world: &TwoStepWorld,
    source_episode: &GroundedSkillEpisode,
) -> usize {
    let mut memory = ModelFreeExplorationMemory::empty();
    let mut current = world.initial.clone();
    let mut noise = TransientNoise::default();
    let mut interactions = 0usize;
    let prefix_observation;

    loop {
        assert!(interactions < 32, "noisy prefix discovery must terminate");

        let decision = ModelFreeEpistemicExploration::select(
            &current,
            &world.available_actions(&current),
            &memory,
            exploration_policy(),
        );
        let action = decision
            .selected_action()
            .cloned()
            .expect("noisy initial state must expose an action");
        let next = noise.execute(world, &current, &action);

        memory = memory
            .record_attempt(current.clone(), action.clone(), exploration_memory_policy())
            .expect("benchmark exploration memory has sufficient capacity");

        interactions = interactions.saturating_add(1);

        if next != current {
            prefix_observation =
                SkillExecutionObservation::new(current.clone(), action, next.clone(), s(900))
                    .expect("successful noisy-world prefix observation is valid");
            current = next;
            break;
        }
    }

    assert_eq!(current, world.intermediate);

    let analogy = GroundedEpisodicAnalogyTransfer::infer_next(
        source_episode,
        &world.initial,
        &world.goal_action,
        &[prefix_observation],
        analogy_policy(),
    );

    assert_eq!(analogy.candidate_count(), 1);
    assert!(!analogy.observation_frontier_exceeded());
    assert!(!analogy.conflicting_evidence());
    assert!(!analogy.correspondence_conflict());

    let transferred = &analogy.candidates()[0];
    assert_eq!(transferred.required_state(), &world.intermediate);
    assert_eq!(transferred.action(), &world.goal_action);
    assert_eq!(transferred.predicted_outcome(), None);
    assert!(
        world
            .available_actions(&current)
            .contains(transferred.action()),
        "transferred action must remain a real noisy-world affordance"
    );

    let next = noise.execute(world, &current, transferred.action());
    interactions = interactions.saturating_add(1);

    assert_eq!(
        next, world.terminal,
        "transferred second action must solve the noisy target after the prefix is grounded"
    );

    interactions
}

#[test]
fn transient_environment_noise_increases_cold_exploration_without_fabricating_source_learning() {
    let source_world = TwoStepWorld::new(100, 700, 999);
    let noisy_target = TwoStepWorld::new(10_000, 20_000, 30_000);

    let (source_episode, source_interactions) = clean_model_free_episode(&source_world);
    let noisy_cold_interactions = noisy_cold_solve(&noisy_target);

    assert_eq!(source_episode.steps().len(), 2);
    assert_eq!(source_interactions, 6);
    assert_eq!(
        noisy_cold_interactions, 9,
        "one transient failure of the correct first action must increase cold exploration cost"
    );
}

#[test]
fn one_real_source_episode_still_accelerates_disjoint_isomorphic_world_under_transient_noise() {
    let source_world = TwoStepWorld::new(100, 700, 999);
    let noisy_target = TwoStepWorld::new(10_000, 20_000, 30_000);

    let (source_episode, source_interactions) = clean_model_free_episode(&source_world);

    let source_identities = source_world.identities();
    let target_identities = noisy_target.identities();
    assert!(
        target_identities
            .iter()
            .all(|identity| !source_identities.contains(identity)),
        "noisy target must use entirely disjoint state/action/goal/outcome identities"
    );

    let noisy_cold_interactions = noisy_cold_solve(&noisy_target);
    let noisy_transfer_interactions = noisy_one_shot_transfer_solve(&noisy_target, &source_episode);

    println!(
        "T1C_METRIC source_episodes=1 source_world={} noisy_cold_world_b={} noisy_transferred_world_b={} saved_vs_noisy_cold={}",
        source_interactions,
        noisy_cold_interactions,
        noisy_transfer_interactions,
        noisy_cold_interactions.saturating_sub(noisy_transfer_interactions),
    );

    assert_eq!(source_interactions, 6);
    assert_eq!(noisy_cold_interactions, 9);
    assert_eq!(
        noisy_transfer_interactions, 7,
        "transfer must tolerate the same transient prefix failure and still skip the two second-state cold probes"
    );
    assert!(noisy_transfer_interactions < noisy_cold_interactions);
}
