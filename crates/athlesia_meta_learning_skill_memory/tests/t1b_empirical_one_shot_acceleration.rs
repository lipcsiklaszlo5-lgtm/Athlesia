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
        values.extend(self.available_actions(&self.initial));
        values.extend(self.available_actions(&self.intermediate));
        values.sort();
        values.dedup();
        values
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

fn model_free_episode(world: &TwoStepWorld) -> (GroundedSkillEpisode, usize) {
    let mut memory = ModelFreeExplorationMemory::empty();
    let mut current = world.initial.clone();
    let mut successful_steps = Vec::new();
    let mut interaction_count = 0usize;

    while current != world.terminal {
        assert!(
            interaction_count < 32,
            "bounded deterministic microworld must terminate"
        );

        let decision = ModelFreeEpistemicExploration::select(
            &current,
            &world.available_actions(&current),
            &memory,
            exploration_policy(),
        );

        let action = decision
            .selected_action()
            .cloned()
            .expect("unsolved world state must expose an action");

        let next = world.execute(&current, &action);

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

    assert_eq!(
        successful_steps.len(),
        2,
        "the source episode contains only actually successful transitions"
    );
    assert!(
        interaction_count > successful_steps.len(),
        "source evidence must arise after real exploratory failures, not an oracle trace"
    );

    (
        GroundedSkillEpisode::new(
            world.initial.clone(),
            world.goal_action.clone(),
            successful_steps,
            s(1000),
        )
        .expect("successful interaction trace forms a grounded skill episode"),
        interaction_count,
    )
}

fn cold_model_free_solve(world: &TwoStepWorld) -> usize {
    model_free_episode(world).1
}

fn one_shot_transfer_solve(world: &TwoStepWorld, source_episode: &GroundedSkillEpisode) -> usize {
    let mut memory = ModelFreeExplorationMemory::empty();
    let mut current = world.initial.clone();
    let mut interaction_count = 0usize;
    let prefix_observation;

    loop {
        let decision = ModelFreeEpistemicExploration::select(
            &current,
            &world.available_actions(&current),
            &memory,
            exploration_policy(),
        );

        let action = decision
            .selected_action()
            .cloned()
            .expect("initial unknown state must expose an action");

        let next = world.execute(&current, &action);

        memory = memory
            .record_attempt(current.clone(), action.clone(), exploration_memory_policy())
            .expect("benchmark exploration memory has sufficient capacity");

        interaction_count = interaction_count.saturating_add(1);

        if next != current {
            prefix_observation =
                SkillExecutionObservation::new(current.clone(), action, next.clone(), s(900))
                    .expect("observed target prefix is valid");
            current = next;
            break;
        }
    }

    assert_eq!(
        current, world.intermediate,
        "B must first ground one real successful transition through model-free exploration"
    );

    let analogy = GroundedEpisodicAnalogyTransfer::infer_next(
        source_episode,
        &world.initial,
        &world.goal_action,
        &[prefix_observation],
        analogy_policy(),
    );

    assert_eq!(
        analogy.candidate_count(),
        1,
        "one source episode plus one grounded B prefix must yield exactly one next-action candidate"
    );
    assert!(!analogy.observation_frontier_exceeded());
    assert!(!analogy.conflicting_evidence());
    assert!(!analogy.correspondence_conflict());

    let transferred = &analogy.candidates()[0];

    assert_eq!(transferred.source_step_index(), 1);
    assert_eq!(transferred.required_state(), &world.intermediate);
    assert_eq!(
        transferred.action(),
        &world.goal_action,
        "the novel B goal/action identity must be obtained by grounded relational correspondence"
    );
    assert_eq!(
        transferred.predicted_outcome(),
        None,
        "B terminal has never been observed or grounded and must remain explicitly unknown"
    );
    assert!(
        world
            .available_actions(&current)
            .contains(transferred.action()),
        "transferred action must be a real affordance in the target environment"
    );

    let next = world.execute(&current, transferred.action());
    interaction_count = interaction_count.saturating_add(1);

    assert_eq!(
        next, world.terminal,
        "the transferred action must actually solve B's second step"
    );

    interaction_count
}

#[test]
fn single_world_a_episode_is_generated_by_real_model_free_interaction() {
    let world_a = TwoStepWorld::new(100, 700, 999);
    let (episode, interactions) = model_free_episode(&world_a);

    assert_eq!(episode.steps().len(), 2);
    assert_eq!(
        interactions, 6,
        "one A experience must include real cold exploration at both unknown states"
    );
    assert_eq!(episode.initial_state(), &world_a.initial);
    assert_eq!(episode.goal_identity(), &world_a.goal_action);
}

#[test]
fn one_real_world_a_episode_measurably_accelerates_disjoint_isomorphic_world_b() {
    let world_a = TwoStepWorld::new(100, 700, 999);
    let world_b = TwoStepWorld::new(10_000, 20_000, 30_000);

    let (source_episode, world_a_interactions) = model_free_episode(&world_a);

    let world_a_identities = world_a.identities();
    let world_b_identities = world_b.identities();

    assert!(
        world_b_identities
            .iter()
            .all(|identity| !world_a_identities.contains(identity)),
        "World B must use entirely disjoint state/action/goal/outcome identities"
    );

    let cold_world_b_interactions = cold_model_free_solve(&world_b);
    let transferred_world_b_interactions = one_shot_transfer_solve(&world_b, &source_episode);

    println!(
        "T1B_METRIC source_episodes=1 world_a={} cold_world_b={} transferred_world_b={} saved_vs_cold={}",
        world_a_interactions,
        cold_world_b_interactions,
        transferred_world_b_interactions,
        cold_world_b_interactions.saturating_sub(transferred_world_b_interactions),
    );

    assert_eq!(world_a_interactions, 6);
    assert_eq!(
        cold_world_b_interactions, 6,
        "cold B must retain the full model-free exploration cost"
    );
    assert_eq!(
        transferred_world_b_interactions, 4,
        "one-shot analogy should still discover B's first transition but skip both second-state exploratory failures"
    );
    assert!(
        transferred_world_b_interactions < world_a_interactions,
        "the disjoint isomorphic B world must be solved faster than the single source experience"
    );
    assert!(
        transferred_world_b_interactions < cold_world_b_interactions,
        "the acceleration must disappear in the cold control"
    );
}
