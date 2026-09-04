use athlesia_mindstone_sparse_cognition::{
    CognitiveStructure, ModelFreeEpistemicExploration, ModelFreeExplorationMemory,
    ModelFreeExplorationMemoryPolicy, ModelFreeExplorationPolicy, ModelFreeExplorationStatus,
    UniversalModelFreeEpistemicExploration,
};

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn exploration_policy() -> ModelFreeExplorationPolicy {
    ModelFreeExplorationPolicy::new(16).expect("positive exploration bound is valid")
}

fn memory_policy() -> ModelFreeExplorationMemoryPolicy {
    ModelFreeExplorationMemoryPolicy::new(64).expect("positive memory bound is valid")
}

#[test]
fn zero_knowledge_with_available_actions_produces_a_real_intervention() {
    let context = atom(1);

    let memory = ModelFreeExplorationMemory::empty();

    let result = ModelFreeEpistemicExploration::select(
        &context,
        &[atom(30), atom(10), atom(20)],
        &memory,
        exploration_policy(),
    );

    assert_eq!(result.status(), ModelFreeExplorationStatus::Selected);

    assert_eq!(result.selected_action(), Some(&atom(10)));

    assert_eq!(result.prior_attempt_count(), Some(0));

    assert!(result.selected_is_novel_in_context());
}

#[test]
fn untried_action_is_preferred_over_already_tried_action() {
    let context = atom(1);

    let memory = ModelFreeExplorationMemory::empty()
        .record_attempt(context.clone(), atom(10), memory_policy())
        .expect("first exact experience must fit memory");

    let result = ModelFreeEpistemicExploration::select(
        &context,
        &[atom(10), atom(20), atom(30)],
        &memory,
        exploration_policy(),
    );

    assert_eq!(result.selected_action(), Some(&atom(20)));

    assert_eq!(result.prior_attempt_count(), Some(0));
}

#[test]
fn repeated_selection_systematically_covers_untried_actions() {
    let context = atom(1);

    let actions = vec![atom(10), atom(20), atom(30)];

    let mut memory = ModelFreeExplorationMemory::empty();
    let mut selected = Vec::new();

    for _ in 0..3 {
        let decision = ModelFreeEpistemicExploration::select(
            &context,
            &actions,
            &memory,
            exploration_policy(),
        );

        let action = decision
            .selected_action()
            .cloned()
            .expect("an available action must be selected");

        assert_eq!(decision.prior_attempt_count(), Some(0));

        selected.push(action.clone());

        memory = memory
            .record_attempt(context.clone(), action, memory_policy())
            .expect("three experiences fit bounded memory");
    }

    selected.sort();
    selected.dedup();

    assert_eq!(selected.len(), 3);
}

#[test]
fn exploration_memory_is_exact_context_sensitive() {
    let first_context = atom(1);
    let second_context = atom(2);

    let memory = ModelFreeExplorationMemory::empty()
        .record_attempt(first_context.clone(), atom(10), memory_policy())
        .expect("experience fits memory");

    assert_eq!(memory.attempt_count(&first_context, &atom(10)), 1);

    assert_eq!(memory.attempt_count(&second_context, &atom(10)), 0);

    let result = ModelFreeEpistemicExploration::select(
        &second_context,
        &[atom(10)],
        &memory,
        exploration_policy(),
    );

    assert!(result.selected_is_novel_in_context());
}

#[test]
fn exact_structural_action_identity_remains_authoritative() {
    let context = atom(1);

    let ordered =
        CognitiveStructure::ordered(vec![atom(7), atom(8)]).expect("ordered structure is nonempty");

    let reversed =
        CognitiveStructure::ordered(vec![atom(8), atom(7)]).expect("ordered structure is nonempty");

    let memory = ModelFreeExplorationMemory::empty()
        .record_attempt(context.clone(), ordered.clone(), memory_policy())
        .expect("experience fits memory");

    let result = ModelFreeEpistemicExploration::select(
        &context,
        &[ordered.clone(), reversed.clone()],
        &memory,
        exploration_policy(),
    );

    assert_ne!(ordered, reversed);

    assert_eq!(result.selected_action(), Some(&reversed));

    assert!(result.selected_is_novel_in_context());
}

#[test]
fn action_input_order_and_exact_duplicates_cannot_change_selection() {
    let context = atom(1);

    let memory = ModelFreeExplorationMemory::empty();

    let left = ModelFreeEpistemicExploration::select(
        &context,
        &[atom(30), atom(10), atom(20), atom(10)],
        &memory,
        exploration_policy(),
    );

    let right = ModelFreeEpistemicExploration::select(
        &context,
        &[atom(20), atom(30), atom(10)],
        &memory,
        exploration_policy(),
    );

    assert_eq!(left, right);
    assert_eq!(left.admitted_action_count(), 3);
    assert_eq!(left.evaluated_action_count(), 3);
}

#[test]
fn candidate_evaluation_is_hard_bounded_and_deterministic() {
    let context = atom(1);

    let memory = ModelFreeExplorationMemory::empty();

    let policy = ModelFreeExplorationPolicy::new(2).expect("positive bound is valid");

    let result = ModelFreeEpistemicExploration::select(
        &context,
        &[atom(30), atom(20), atom(10)],
        &memory,
        policy,
    );

    assert_eq!(result.admitted_action_count(), 3);
    assert_eq!(result.evaluated_action_count(), 2);
    assert!(result.candidate_frontier_truncated());

    assert_eq!(result.selected_action(), Some(&atom(10)));
}

#[test]
fn bounded_memory_refuses_new_identity_without_silent_eviction() {
    let context = atom(1);

    let policy = ModelFreeExplorationMemoryPolicy::new(1).expect("positive memory bound is valid");

    let memory = ModelFreeExplorationMemory::empty()
        .record_attempt(context.clone(), atom(10), policy)
        .expect("first record fits memory");

    let rejected = memory.record_attempt(context.clone(), atom(20), policy);

    assert!(rejected.is_none());

    assert_eq!(memory.record_count(), 1);

    assert_eq!(memory.attempt_count(&context, &atom(10)), 1);

    assert_eq!(memory.attempt_count(&context, &atom(20)), 0);
}

#[test]
fn empty_affordance_set_does_not_fabricate_an_action() {
    let result = ModelFreeEpistemicExploration::select(
        &atom(1),
        &[],
        &ModelFreeExplorationMemory::empty(),
        exploration_policy(),
    );

    assert_eq!(
        result.status(),
        ModelFreeExplorationStatus::NoAvailableAction
    );

    assert!(result.selected_action().is_none());
    assert_eq!(result.prior_attempt_count(), None);
}

#[test]
fn universal_facade_matches_direct_model_free_exploration() {
    let context = atom(1);

    let actions = vec![atom(30), atom(10), atom(20)];

    let memory = ModelFreeExplorationMemory::empty();

    let direct =
        ModelFreeEpistemicExploration::select(&context, &actions, &memory, exploration_policy());

    let universal = UniversalModelFreeEpistemicExploration::select(
        &context,
        &actions,
        &memory,
        exploration_policy(),
    );

    assert_eq!(direct, universal);
}
