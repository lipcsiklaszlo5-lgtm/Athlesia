use athlesia_meta_learning_skill_memory::{
    CompressedSkillRecord, CrossContextSkillGeneralization, CrossContextSkillGeneralizationPolicy,
    GroundedPartialSkillTransfer, GroundedSkillCorrespondenceInference,
    GroundedSkillCorrespondencePolicy, GroundedSkillEpisode, GroundedSkillStep,
    LossControlledSkillCompression, RepeatedSkillCandidate, RepeatedSkillCandidateDiscovery,
    RepeatedSkillCandidatePolicy, SkillCompressionBounds, SkillCompressionPolicy,
    SkillCompressionThresholds, SkillExecutionObservation, SkillMemoryFoundation,
    SkillMemoryPolicy, SkillReuseBounds, SkillReusePolicy, SkillReuseThresholds,
    StructuralSkillAbstractionEvidence, StructuralSkillAbstractionInduction,
    StructuralSkillAbstractionPolicy,
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

fn memory_policy() -> SkillMemoryPolicy {
    SkillMemoryPolicy::new(64, 16, 64, 64, s(1), s(1)).expect("valid skill-memory policy")
}

fn candidate_policy() -> RepeatedSkillCandidatePolicy {
    RepeatedSkillCandidatePolicy::new(64, 64, 16, 64, 2, s(1), s(1))
        .expect("valid repeated-skill policy")
}

fn abstraction_policy() -> StructuralSkillAbstractionPolicy {
    StructuralSkillAbstractionPolicy::new(16, 16, 16, 16, 2, s(1), s(1))
        .expect("valid structural abstraction policy")
}

fn generalization_policy() -> CrossContextSkillGeneralizationPolicy {
    CrossContextSkillGeneralizationPolicy::new(16, 16, 16, 16, 1, s(1), s(1))
        .expect("valid cross-context generalization policy")
}

fn compression_policy() -> SkillCompressionPolicy {
    SkillCompressionPolicy::new(
        SkillCompressionBounds::new(16, 16, 16, 16).expect("valid compression bounds"),
        SkillCompressionThresholds::new(1, s(1), s(1), 0).expect("valid compression thresholds"),
    )
}

fn correspondence_policy() -> GroundedSkillCorrespondencePolicy {
    GroundedSkillCorrespondencePolicy::new(32, 32, 32, s(500)).expect("valid correspondence policy")
}

fn reuse_policy() -> SkillReusePolicy {
    SkillReusePolicy::new(
        SkillReuseBounds::new(32, 32, 16, 16, 32).expect("valid reuse bounds"),
        SkillReuseThresholds::new(1, 1, s(500), s(500), s(500)).expect("valid reuse thresholds"),
    )
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

        let available = world.available_actions(&current);

        let decision = ModelFreeEpistemicExploration::select(
            &current,
            &available,
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
        "the learned skill trace contains the two actually successful transitions"
    );

    assert!(
        interaction_count > successful_steps.len(),
        "World A evidence must arise after real exploratory failures, not an oracle trace"
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

fn repeated_candidate_from_interaction(
    world: &TwoStepWorld,
    support: usize,
) -> (RepeatedSkillCandidate, Vec<usize>) {
    let mut episodes = Vec::new();
    let mut interactions = Vec::new();

    for _ in 0..support {
        let (episode, count) = model_free_episode(world);
        episodes.push(episode);
        interactions.push(count);
    }

    let memory = SkillMemoryFoundation::build(&episodes, memory_policy());

    let discovered =
        RepeatedSkillCandidateDiscovery::discover(memory.entries(), candidate_policy());

    assert!(
        !discovered.candidates().is_empty(),
        "repeated interaction evidence must yield a skill candidate"
    );

    (discovered.candidates()[0].clone(), interactions)
}

fn abstraction_from_interaction(
    left: &TwoStepWorld,
    right: &TwoStepWorld,
    support: usize,
) -> (StructuralSkillAbstractionEvidence, Vec<usize>) {
    let (left_candidate, mut left_interactions) =
        repeated_candidate_from_interaction(left, support);

    let (right_candidate, right_interactions) = repeated_candidate_from_interaction(right, support);

    left_interactions.extend(right_interactions);

    let induced = StructuralSkillAbstractionInduction::induce(
        &[left_candidate, right_candidate],
        abstraction_policy(),
    );

    assert!(
        !induced.abstractions().is_empty(),
        "cross-instance interaction evidence must yield structural abstraction"
    );

    (induced.abstractions()[0].clone(), left_interactions)
}

fn learned_record_from_world_a_interactions() -> (CompressedSkillRecord, Vec<usize>) {
    let a1 = TwoStepWorld::new(100, 7, 70);
    let a2 = TwoStepWorld::new(200, 7, 70);

    let a3 = TwoStepWorld::new(300, 8, 80);
    let a4 = TwoStepWorld::new(400, 8, 80);

    let (left_abstraction, mut interactions) = abstraction_from_interaction(&a1, &a2, 2);

    let (right_abstraction, right_interactions) = abstraction_from_interaction(&a3, &a4, 2);

    interactions.extend(right_interactions);

    let generalized = CrossContextSkillGeneralization::generalize(
        &[left_abstraction, right_abstraction],
        generalization_policy(),
    );

    assert!(
        !generalized.generalizations().is_empty(),
        "two interaction-derived abstractions must yield a cross-context skill"
    );

    let compressed = LossControlledSkillCompression::compress_all(
        std::slice::from_ref(&generalized.generalizations()[0]),
        compression_policy(),
    );

    assert_eq!(
        compressed.records().len(),
        1,
        "the learned cross-context skill must compress to one reusable record"
    );

    (compressed.records()[0].clone(), interactions)
}

fn cold_model_free_solve(world: &TwoStepWorld) -> usize {
    model_free_episode(world).1
}

fn transfer_solve(world: &TwoStepWorld, record: &CompressedSkillRecord) -> usize {
    let mut memory = ModelFreeExplorationMemory::empty();
    let source_state = world.initial.clone();
    let mut current = source_state.clone();
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
                    .expect("observed prefix transition is valid");

            current = next;
            break;
        }
    }

    assert_eq!(
        current, world.intermediate,
        "model-free exploration must ground the first successful novel-world transition"
    );

    let correspondence = GroundedSkillCorrespondenceInference::infer(
        std::slice::from_ref(record),
        &source_state,
        &world.goal_action,
        &[prefix_observation],
        correspondence_policy(),
    );

    assert_eq!(
        correspondence.request_count(),
        1,
        "World B correspondence must be inferred without manually supplied slot bindings"
    );

    let partial = GroundedPartialSkillTransfer::retrieve_next(
        std::slice::from_ref(record),
        &correspondence.requests()[0],
        &current,
        reuse_policy(),
    );

    assert_eq!(
        partial.candidate_count(),
        1,
        "learned structural correspondence must expose exactly one next action"
    );

    let transferred = &partial.candidates()[0];

    assert_eq!(transferred.required_state(), &world.intermediate);

    assert_eq!(
        transferred.action(),
        &world.goal_action,
        "the novel goal/action identity must be transferred structurally"
    );

    assert_eq!(
        transferred.predicted_outcome(),
        None,
        "novel terminal outcome was never observed and must remain unknown"
    );

    assert!(
        world
            .available_actions(&current)
            .contains(transferred.action()),
        "transferred action must be a real affordance of the current environment"
    );

    let next = world.execute(&current, transferred.action());

    interaction_count = interaction_count.saturating_add(1);

    assert_eq!(
        next, world.terminal,
        "transferred action must actually solve the novel-world second step"
    );

    interaction_count
}

#[test]
fn world_a_learning_evidence_is_generated_by_real_model_free_interaction() {
    let (record, interactions) = learned_record_from_world_a_interactions();

    assert_eq!(record.step_count(), 2);
    assert_eq!(interactions.len(), 8);

    assert!(
        interactions.iter().all(|count| *count > 2),
        "training evidence must not be fabricated from direct two-step oracle traces"
    );

    assert!(
        interactions.iter().all(|count| *count == 6),
        "the deterministic F0-A baseline should require exploration at both unknown states"
    );
}

#[test]
fn learned_world_a_skill_measurably_accelerates_disjoint_isomorphic_world_b() {
    let (record, training_interactions) = learned_record_from_world_a_interactions();

    let world_b = TwoStepWorld::new(10_000, 20_000, 30_000);

    let training_worlds = [
        TwoStepWorld::new(100, 7, 70),
        TwoStepWorld::new(200, 7, 70),
        TwoStepWorld::new(300, 8, 80),
        TwoStepWorld::new(400, 8, 80),
    ];

    let mut training_identities = Vec::new();

    for world in &training_worlds {
        training_identities.extend(world.identities());
    }

    training_identities.sort();
    training_identities.dedup();

    let world_b_identities = world_b.identities();

    assert!(
        world_b_identities
            .iter()
            .all(|identity| !training_identities.contains(identity)),
        "World B must use entirely new state/action/goal/outcome atom identities"
    );

    let first_world_a_interactions = training_interactions[0];

    let cold_world_b_interactions = cold_model_free_solve(&world_b);

    let transferred_world_b_interactions = transfer_solve(&world_b, &record);

    println!(
        "T0C_METRIC first_world_a={} cold_world_b={} transferred_world_b={} saved_vs_cold={}",
        first_world_a_interactions,
        cold_world_b_interactions,
        transferred_world_b_interactions,
        cold_world_b_interactions.saturating_sub(transferred_world_b_interactions),
    );

    assert_eq!(
        first_world_a_interactions, 6,
        "the first A instance must require genuine cold exploration"
    );

    assert_eq!(
        cold_world_b_interactions, 6,
        "the same novel B world with no learned skill transfer must retain cold exploration cost"
    );

    assert_eq!(
        transferred_world_b_interactions,
        4,
        "after learning, B should still discover its first correspondence-grounding transition but skip two second-state exploratory failures"
    );

    assert!(
        transferred_world_b_interactions < first_world_a_interactions,
        "learning must make the structurally similar novel world faster than the first training world"
    );

    assert!(
        transferred_world_b_interactions < cold_world_b_interactions,
        "the acceleration must disappear in the cold-memory control"
    );
}
