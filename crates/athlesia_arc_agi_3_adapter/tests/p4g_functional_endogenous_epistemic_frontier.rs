/*
 * P4G-C3A functional/live gate.
 *
 * The frozen C2 test surface is included unchanged so this target reuses the
 * exact already-proven real transport/session/cognitive fixture rather than
 * constructing a parallel fake interaction path.
 *
 * C3A adds no M50 belief synthesis and does not choose or dispatch an
 * epistemic action. It proves only:
 *
 * real live consequences
 *   -> retained M47 episodes
 *   -> endogenous explanatory competition
 *   -> informative action discrimination frontier.
 */

include!("p4g_functional_live_unified_dispatch.rs");

#[test]
fn real_live_holdout_context_endogenously_creates_epistemic_action_frontier() {
    let game = "p4gc3a-live-holdout";

    let action_one = action(ArcAgi3ActionId::Action1);
    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 300_000);

    /*
     * This is the frozen C2 training trajectory:
     *
     * ACTION1 repeatedly changes the grounded object while ACTION2
     * is contrasted as a self-loop.
     *
     * It ends in the already-observed value-5 context.
     */
    mature_runtime(&mut runtime, game);

    let before_holdout_transport = runtime.transport().execute_count();

    /*
     * Enter a genuinely unseen current context through a REAL live
     * environment transaction.
     *
     * Crucially this is ACTION2, not ACTION1:
     * the runtime therefore receives no new Action1 outcome evidence.
     *
     * The retained Action1 explanations learned from the 5/6 contexts
     * must now disagree about what Action1 would do in context 7.
     */
    real_training_turn(&mut runtime, game, action_two, 7_u8);

    assert_eq!(
        runtime.transport().execute_count(),
        before_holdout_transport + 1,
        "holdout context must be reached by exactly one real transport action",
    );

    assert_eq!(
        runtime.transport().last_executed_action(),
        Some(action_two),
        "holdout transition must be caused by Action2 rather than by probing Action1",
    );

    let cognitive = runtime.cognitive_runtime();

    let current_state = cognitive
        .current_grounded_world_state()
        .expect("real holdout consequence must leave a grounded retained M47 state");

    let candidate_actions = [
        ArcAgi3CognitiveProtocolBridge::encode_action(action_one),
        ArcAgi3CognitiveProtocolBridge::encode_action(action_two),
    ];

    let frontier = cognitive
        .cognition()
        .current_factorized_action_discrimination(
            current_state,
            &candidate_actions,
            athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
                1, 64, 512, 256,
            )
            .expect("C3A explanatory frontier bounds are positive"),
            athlesia_universal_domain_learning::GroundedFactorizedDiscriminationPolicy::new(8)
                .expect("C3A action frontier bound is positive"),
        );

    let action_one_candidate = frontier
        .ranked()
        .iter()
        .find(|candidate| candidate.transformation() == &candidate_actions[0])
        .expect("Action1 must remain represented in the evaluated live frontier");

    assert!(
        action_one_candidate.informative(),
        "unseen live context must make Action1 epistemically informative",
    );

    assert!(
        action_one_candidate.pairwise_separation_score() > 0,
        "Action1 requires real positive separation between retained explanations",
    );

    let best = frontier
        .best_informative()
        .expect("holdout live context must expose at least one informative action");

    assert!(
        candidate_actions
            .iter()
            .any(|action| action == best.transformation()),
        "epistemic frontier must not invent an action outside real ARC affordances",
    );

    /*
     * Frontier evaluation itself is cognition-only.
     * It must not silently execute the proposed experiment.
     */
    assert_eq!(
        runtime.transport().execute_count(),
        before_holdout_transport + 1,
        "reading the epistemic frontier must have zero hidden transport effect",
    );
}

#[test]
fn matured_known_context_does_not_fabricate_epistemic_disagreement() {
    let game = "p4gc3a-known-context";

    let action_one = action(ArcAgi3ActionId::Action1);
    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 310_000);

    mature_runtime(&mut runtime, game);

    let cognitive = runtime.cognitive_runtime();

    let current_state = cognitive
        .current_grounded_world_state()
        .expect("matured live runtime must retain a grounded current state");

    let candidate_actions = [
        ArcAgi3CognitiveProtocolBridge::encode_action(action_one),
        ArcAgi3CognitiveProtocolBridge::encode_action(action_two),
    ];

    let frontier = cognitive
        .cognition()
        .current_factorized_action_discrimination(
            current_state,
            &candidate_actions,
            athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
                1, 64, 512, 256,
            )
            .expect("C3A explanatory frontier bounds are positive"),
            athlesia_universal_domain_learning::GroundedFactorizedDiscriminationPolicy::new(8)
                .expect("C3A action frontier bound is positive"),
        );

    assert_eq!(
        frontier.best_informative(),
        None,
        "already-resolved live context must not fabricate epistemic disagreement",
    );
}

#[test]
fn fresh_live_runtime_cannot_fabricate_epistemic_frontier_without_experience() {
    let game = "p4gc2-live-abstain";

    let mut runtime = live_runtime(game, 310_000);

    let candidates = [
        action(ArcAgi3ActionId::Action1),
        action(ArcAgi3ActionId::Action2),
    ];

    let goal = goal();

    let before_transport = runtime.transport().execute_count();

    let before_steps = runtime.completed_cognitive_step_count();

    let request = ArcAgi3LiveUnifiedActionRequest::new(
        &candidates,
        &goal,
        signal(900),
        CognitiveSignal::zero(),
        None,
        evidence_policy(),
        signal(900),
    );

    assert!(
        runtime
            .cognitive_runtime()
            .current_grounded_world_state()
            .is_none(),
        "a fresh live runtime must not fabricate grounded epistemic state before any accepted self-generated environment consequence",
    );

    let result = runtime
        .execute_unified(request)
        .expect("abstention is not a live execution error");

    assert!(
        result.is_none(),
        "fresh runtime must not fabricate action authority",
    );

    assert_eq!(runtime.transport().execute_count(), before_transport,);

    assert_eq!(runtime.completed_cognitive_step_count(), before_steps,);

    assert!(
        !runtime.cognitive_runtime().session().has_pending_command(),
        "abstention must not create hidden pending work",
    );
}
