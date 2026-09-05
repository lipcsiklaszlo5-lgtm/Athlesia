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

#[test]
fn live_holdout_m47_m50_bridge_preserves_exact_epistemic_separation_without_transport() {
    let game = "p4gc3b-live-holdout";

    let action_one = action(ArcAgi3ActionId::Action1);
    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 320_000);

    /*
     * Retain the frozen C2 learning history, then enter the same genuinely
     * unseen context used by C3A through a REAL Action2 environment turn.
     *
     * No Action1 outcome is observed in context 7 before the epistemic query.
     */
    mature_runtime(&mut runtime, game);
    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let before_query_transport = runtime.transport().execute_count();

    let cognitive_runtime = runtime.cognitive_runtime();

    let current_state = cognitive_runtime
        .current_grounded_world_state()
        .expect("live holdout context must retain a grounded current M47 state");

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let version_policy =
        athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
            1, 64, 512, 256,
        )
        .expect("C3B live explanatory bounds are positive");

    let m47 = cognitive_runtime
        .cognition()
        .current_factorized_action_discrimination(
            current_state,
            std::slice::from_ref(&cognitive_action),
            version_policy,
            athlesia_universal_domain_learning::GroundedFactorizedDiscriminationPolicy::new(8)
                .expect("C3B live M47 action bound is positive"),
        );

    let m47_action = m47
        .best_informative()
        .expect("live holdout Action1 must remain informative in M47");

    assert_eq!(
        m47_action.transformation(),
        &cognitive_action,
        "M47 epistemic authority must remain bound to exact Action1 identity",
    );

    let m50_possibility = cognitive_runtime
        .cognition()
        .current_m50_epistemic_possibility(current_state, &cognitive_action, version_policy)
        .expect("live retained M47 evidence must materialize an M50 epistemic possibility");

    assert_eq!(
        m50_possibility.action(),
        &cognitive_action,
        "M50 possibility must preserve exact live Action1 identity",
    );

    let m50 =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::evaluate(
                &m50_possibility,
                athlesia_autonomous_active_experimentation::
                    EpistemicForecastDiscriminationPolicy::new(
                        512,
                        512,
                    )
                    .expect("C3B live M50 bounds are positive"),
            );

    assert!(
        m50.informative(),
        "the same live holdout disagreement must remain informative after M47 -> M50 mapping",
    );

    assert_eq!(
        m50.pairwise_separation_score(),
        m47_action.pairwise_separation_score(),
        "live M47 -> M50 bridge must preserve the exact factorized separation score",
    );

    let predicted_count = m50_possibility
        .forecasts()
        .iter()
        .filter(|forecast| {
            forecast.status()
                == athlesia_autonomous_active_experimentation::
                    EpistemicHypothesisForecastStatus::Predicted
        })
        .count();

    let abstained = m50_possibility
        .forecasts()
        .iter()
        .filter(|forecast| {
            forecast.status()
                == athlesia_autonomous_active_experimentation::
                    EpistemicHypothesisForecastStatus::ContextAbstained
        })
        .collect::<Vec<_>>();

    assert!(
        predicted_count > 0,
        "live M50 epistemic possibility must contain at least one real grounded prediction",
    );

    assert!(
        !abstained.is_empty(),
        "live M50 epistemic possibility must retain contextual abstention",
    );

    assert!(
        abstained
            .iter()
            .all(|forecast| forecast.predicted_outcome().is_none()),
        "live contextual abstention must never acquire a fabricated predicted outcome",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        before_query_transport,
        "M47 and M50 epistemic queries must have zero hidden transport effect",
    );
}

#[test]
fn live_resolved_context_remains_noninformative_after_m47_m50_bridge() {
    let game = "p4gc3b-live-resolved";

    let action_one = action(ArcAgi3ActionId::Action1);

    let mut runtime = live_runtime(game, 330_000);

    /*
     * The frozen mature trajectory ends in an already learned context.
     * C3A proved that this context carries no actionable epistemic split.
     * C3B must preserve that negative fact across the M47 -> M50 boundary.
     */
    mature_runtime(&mut runtime, game);

    let before_query_transport = runtime.transport().execute_count();

    let cognitive_runtime = runtime.cognitive_runtime();

    let current_state = cognitive_runtime
        .current_grounded_world_state()
        .expect("matured live runtime must retain grounded M47 state");

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let version_policy =
        athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
            1, 64, 512, 256,
        )
        .expect("C3B resolved explanatory bounds are positive");

    let m47 = cognitive_runtime
        .cognition()
        .current_factorized_action_discrimination(
            current_state,
            std::slice::from_ref(&cognitive_action),
            version_policy,
            athlesia_universal_domain_learning::GroundedFactorizedDiscriminationPolicy::new(8)
                .expect("C3B resolved M47 bound is positive"),
        );

    assert_eq!(
        m47.best_informative(),
        None,
        "resolved live context must remain noninformative at M47",
    );

    let m50_possibility = cognitive_runtime
        .cognition()
        .current_m50_epistemic_possibility(current_state, &cognitive_action, version_policy)
        .expect("resolved context may retain grounded explanatory forecasts");

    let m50 =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::evaluate(
                &m50_possibility,
                athlesia_autonomous_active_experimentation::
                    EpistemicForecastDiscriminationPolicy::new(
                        512,
                        512,
                    )
                    .expect("C3B resolved M50 bounds are positive"),
            );

    assert!(
        !m50.informative(),
        "M50 must not manufacture epistemic disagreement absent from live M47",
    );

    assert_eq!(
        m50.pairwise_separation_score(),
        0,
        "resolved live M47 evidence must remain zero-separation after M50 mapping",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        before_query_transport,
        "resolved-context epistemic queries must not execute hidden transport",
    );
}

#[test]
fn live_real_action_consequence_resolves_pre_action_m50_epistemic_forecasts() {
    let game = "p4gc3c-live-resolution";

    let action_one = action(ArcAgi3ActionId::Action1);
    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 340_000);

    /*
     * Frozen C2 learning trajectory.
     *
     * Then enter the C3A/C3B holdout context via ACTION2 so ACTION1 has
     * never yet been executed from this context.
     */
    mature_runtime(&mut runtime, game);
    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let before_state = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("holdout context must be grounded")
        .clone();

    let version_policy =
        athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
            1, 64, 512, 256,
        )
        .expect("positive explanatory bounds");

    /*
     * This forecast is created BEFORE the real Action1 consequence.
     */
    let possibility = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(&before_state, &cognitive_action, version_policy)
        .expect("holdout Action1 must expose grounded pre-action forecasts");

    let pre_action_discrimination =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::
                evaluate(
                    &possibility,
                    athlesia_autonomous_active_experimentation::
                        EpistemicForecastDiscriminationPolicy::
                            new(
                                512,
                                512,
                            )
                            .expect("positive discrimination bounds"),
                );

    assert!(
        pre_action_discrimination.informative(),
        "the action must be epistemically unresolved before execution",
    );

    let before_transport = runtime.transport().execute_count();

    /*
     * REAL environment consequence:
     *
     * unseen context 7 --ACTION1--> 6
     *
     * Existing unconditional Add(6)-type predictions can now be
     * supported, while incompatible predicted effects are falsified.
     */
    real_training_turn(&mut runtime, game, action_one, 6_u8);

    assert_eq!(
        runtime.transport().execute_count(),
        before_transport + 1,
        "exactly one real Action1 environment transaction must occur",
    );

    assert_eq!(runtime.transport().last_executed_action(), Some(action_one),);

    let after_state = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("real Action1 consequence must leave grounded after-state")
        .clone();

    let before_resolution_transport = runtime.transport().execute_count();

    let resolution = runtime
        .cognitive_runtime()
        .cognition()
        .resolve_m50_epistemic_possibility_against_transition(
            &possibility,
            &before_state,
            &after_state,
            &cognitive_action,
            athlesia_autonomous_active_experimentation::EpistemicOutcomeResolutionPolicy::new(
                512, 512,
            )
            .expect("positive outcome-resolution bounds"),
        )
        .expect("frozen C3B targets must decode exactly");

    assert!(
        resolution.resolved(),
        "real environment consequence must resolve the pre-action epistemic record",
    );

    assert!(
        resolution.supported_prediction_count() > 0,
        "real consequence must support at least one pre-action prediction",
    );

    assert!(
        resolution.counterexample_prediction_count() > 0,
        "the same real consequence must falsify at least one incompatible pre-action prediction",
    );

    assert!(
        resolution.context_uninformative_count() > 0,
        "contextual abstentions must remain explicitly uninformative after the real consequence",
    );

    assert_eq!(
        resolution.falsified_hypothesis_count(),
        resolution.counterexample_prediction_count(),
        "falsification count must be exact rather than inferred from raw disagreement",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        before_resolution_transport,
        "post-action epistemic resolution must have zero hidden transport effect",
    );
}

#[test]
fn live_real_consequence_with_wrong_action_identity_is_rejected_without_hidden_transport() {
    let game = "p4gc3c-live-provenance-negative";

    let action_one = action(ArcAgi3ActionId::Action1);
    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 350_000);

    mature_runtime(&mut runtime, game);

    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let cognitive_action_one = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let cognitive_action_two = ArcAgi3CognitiveProtocolBridge::encode_action(action_two);

    let before_state = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .unwrap()
        .clone();

    let possibility = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(
            &before_state,
            &cognitive_action_one,
            athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
                1, 64, 512, 256,
            )
            .unwrap(),
        )
        .expect("pre-action Action1 forecast");

    real_training_turn(&mut runtime, game, action_one, 6_u8);

    let after_state = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .unwrap()
        .clone();

    let before_resolution_transport = runtime.transport().execute_count();

    let result = runtime
        .cognitive_runtime()
        .cognition()
        .resolve_m50_epistemic_possibility_against_transition(
            &possibility,
            &before_state,
            &after_state,
            &cognitive_action_two,
            athlesia_autonomous_active_experimentation::EpistemicOutcomeResolutionPolicy::new(
                512, 512,
            )
            .unwrap(),
        )
        .expect("target decoding remains valid");

    assert_eq!(
        result.status(),
        athlesia_autonomous_active_experimentation::
            EpistemicOutcomeResolutionStatus::ActionMismatch,
        "a real consequence cannot be reassigned to the wrong action identity",
    );

    assert!(result.assessments().is_empty());

    assert_eq!(
        runtime.transport().execute_count(),
        before_resolution_transport,
        "rejected provenance must not cause hidden transport",
    );
}

#[test]
fn live_real_learning_changes_same_holdout_epistemic_frontier_and_c3d_measures_exact_change() {
    let game = "p4gc3d-live-progress";

    let action_one = action(ArcAgi3ActionId::Action1);
    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 360_000);

    /*
     * Establish the frozen C2/C3 experience, then enter a genuinely unseen
     * current context through real ACTION2.
     *
     * ACTION1 has not yet been executed from context 7.
     */
    mature_runtime(&mut runtime, game);

    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    /*
     * Preserve the exact PRE-action state.
     *
     * After the real consequence, the retained cognitive owner will contain
     * new learning, but this same state is intentionally queried again so
     * C3D compares the same epistemic question:
     *
     *     "What would ACTION1 do from this exact state?"
     */
    let before_state = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("holdout context must retain a grounded state")
        .clone();

    let version_policy =
        athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
            1, 64, 512, 256,
        )
        .expect("positive explanatory bounds");

    let pre_learning = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(&before_state, &cognitive_action, version_policy)
        .expect("unseen context must expose a grounded pre-learning Action1 possibility");

    let discrimination_policy =
        athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy::new(
            512, 512,
        )
        .expect("positive discrimination bounds");

    let pre_discrimination =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::
                evaluate(
                    &pre_learning,
                    discrimination_policy,
                );

    assert!(
        pre_discrimination.informative(),
        "pre-learning holdout frontier must contain real unresolved epistemic separation",
    );

    assert!(pre_discrimination.pairwise_separation_score() > 0,);

    let before_action_transport = runtime.transport().execute_count();

    /*
     * REAL learning event.
     *
     * The same retained owner observes:
     *
     *     context 7 --ACTION1--> context 6
     */
    real_training_turn(&mut runtime, game, action_one, 6_u8);

    assert_eq!(
        runtime.transport().execute_count(),
        before_action_transport + 1,
        "exactly one real intervention must separate pre- and post-learning cognition",
    );

    assert_eq!(runtime.transport().last_executed_action(), Some(action_one),);

    let after_state = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("real consequence must leave a grounded after-state")
        .clone();

    /*
     * Resolve the PRE-action possibility against the real consequence.
     * This is the frozen C3C bridge.
     */
    let realized_outcome = runtime
        .cognitive_runtime()
        .cognition()
        .resolve_m50_epistemic_possibility_against_transition(
            &pre_learning,
            &before_state,
            &after_state,
            &cognitive_action,
            athlesia_autonomous_active_experimentation::EpistemicOutcomeResolutionPolicy::new(
                512, 512,
            )
            .expect("positive outcome-resolution bounds"),
        )
        .expect("frozen C3B effect targets must decode");

    assert!(realized_outcome.resolved());

    assert!(
        realized_outcome.empirically_tested_prediction_count() > 0,
        "real environment consequence must test at least one concrete prediction",
    );

    /*
     * IMPORTANT:
     *
     * Re-evaluate the SAME OLD state 7 after the retained owner has learned
     * from ACTION1 7 -> 6.
     *
     * This is not the new current state. It is a counterfactual re-query of
     * the exact same pre-action epistemic problem using the UPDATED retained
     * model.
     */
    let post_learning = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(&before_state, &cognitive_action, version_policy)
        .expect("updated retained cognition must still answer the same grounded Action1 query");

    assert_eq!(
        pre_learning.source_state(),
        post_learning.source_state(),
        "C3D live progress must compare the exact same source-state identity",
    );

    assert_eq!(
        pre_learning.action(),
        post_learning.action(),
        "C3D live progress must compare the exact same action identity",
    );

    let post_discrimination =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::
                evaluate(
                    &post_learning,
                    discrimination_policy,
                );

    assert!(
        !post_discrimination.forecast_frontier_truncated()
            && !post_discrimination.target_frontier_truncated(),
        "post-learning comparison must not depend on a partial frontier",
    );

    let before_resolution_transport = runtime.transport().execute_count();

    let progress =
        athlesia_autonomous_active_experimentation::AutonomousEpistemicResolutionProgress::measure(
            &pre_learning,
            &realized_outcome,
            &post_learning,
            discrimination_policy,
        );

    assert!(
        progress.measured(),
        "a real consequence followed by retained model update must yield measurable C3D progress",
    );

    let sample = progress
        .sample()
        .expect("measured C3D result must contain one exact sample");

    assert_eq!(sample.source_state(), pre_learning.source_state(),);

    assert_eq!(sample.action(), pre_learning.action(),);

    assert_eq!(
        sample.separation_before(),
        pre_discrimination.pairwise_separation_score(),
        "C3D must preserve exact pre-learning separation",
    );

    assert_eq!(
        sample.separation_after(),
        post_discrimination.pairwise_separation_score(),
        "C3D must preserve exact post-learning separation",
    );

    assert_ne!(
        sample.separation_before(),
        sample.separation_after(),
        "the real ACTION1 consequence must actually change the retained answer to the same holdout epistemic question",
    );

    assert!(
        sample.realized_separation_reduction() > 0 || sample.realized_separation_increase() > 0,
        "realized model change must remain explicit rather than being clipped to zero",
    );

    assert!(
        !(sample.realized_separation_reduction() > 0 && sample.realized_separation_increase() > 0),
        "one scalar separation change cannot simultaneously be classified as both increase and reduction",
    );

    assert_eq!(
        sample.supported_prediction_count(),
        realized_outcome.supported_prediction_count(),
    );

    assert_eq!(
        sample.counterexample_prediction_count(),
        realized_outcome.counterexample_prediction_count(),
    );

    assert_eq!(
        runtime.transport().execute_count(),
        before_resolution_transport,
        "post-learning re-query and C3D measurement must have zero hidden transport effect",
    );
}

#[test]
fn live_epistemic_requery_without_new_environment_evidence_cannot_manufacture_progress() {
    let game = "p4gc3d-live-no-new-evidence";

    let action_one = action(ArcAgi3ActionId::Action1);
    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 370_000);

    mature_runtime(&mut runtime, game);

    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let state = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("holdout context must be grounded")
        .clone();

    let version_policy =
        athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
            1, 64, 512, 256,
        )
        .expect("positive explanatory bounds");

    let before_transport = runtime.transport().execute_count();

    let first = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(&state, &cognitive_action, version_policy)
        .expect("first epistemic query");

    let second = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(&state, &cognitive_action, version_policy)
        .expect("second epistemic query");

    assert_eq!(
        first, second,
        "without intervening environment evidence, repeated epistemic queries must be identical",
    );

    let policy =
        athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy::new(
            512, 512,
        )
        .expect("positive discrimination bounds");

    let first_frontier =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::
                evaluate(
                    &first,
                    policy,
                );

    let second_frontier =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::
                evaluate(
                    &second,
                    policy,
                );

    assert_eq!(
        first_frontier, second_frontier,
        "epistemic separation cannot change merely because cognition was queried twice",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        before_transport,
        "epistemic re-query must have zero hidden transport effect",
    );
}

#[test]
fn live_real_c3d_progress_is_retained_with_exact_turn_event_provenance() {
    let game = "p4gc3e-live-history";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 380_000);

    mature_runtime(&mut runtime, game);

    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let before_state = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("holdout state must be grounded")
        .clone();

    let pre_learning = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(
            &before_state,
            &cognitive_action,
            athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
                1, 64, 512, 256,
            )
            .unwrap(),
        )
        .expect("holdout Action1 must be epistemically grounded");

    let history_before = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_progress_event_count();

    let transport_before = runtime.transport().execute_count();

    runtime
        .transport()
        .push(Ok(normal_observation(game, 6_u8, Some(action_one))));

    let step = runtime
        .execute_with(signal(900), |cognitive| {
            m51_fixture::begin_arc(cognitive, cognitive_action.clone())
        })
        .expect("real Action1 progress turn must execute");

    let event_index = step.completion().turn().event_index();

    assert_eq!(runtime.transport().execute_count(), transport_before + 1,);

    let cognition = runtime.cognitive_runtime().cognition();

    assert_eq!(
        cognition.epistemic_progress_event_count(),
        history_before + 1,
        "one real C3D progress event must be retained exactly once",
    );

    let retained = cognition
        .epistemic_progress_history()
        .last()
        .expect("new real progress event must be retained");

    assert_eq!(
        retained.event_index(),
        event_index,
        "retained provenance must use the exact completed session event index",
    );

    assert_eq!(
        retained.sample().source_state(),
        pre_learning.source_state(),
    );

    assert_eq!(retained.sample().action(), pre_learning.action(),);

    let before_query_transport = runtime.transport().execute_count();

    let estimate = cognition.current_empirical_expected_epistemic_progress(
        &pre_learning,
        athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy::new(
            512, 512,
        )
        .unwrap(),
        athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy::new(
            256, 256, 1,
        )
        .unwrap(),
    );

    assert!(
        estimate.estimated(),
        "the retained live event must be consumable by C3E-A without caller-supplied history",
    );

    assert_eq!(
        estimate.estimate().unwrap().qualifying_sample_count(),
        1,
        "the unseen source-state has exactly one qualifying retained event",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        before_query_transport,
        "history query must have zero hidden transport effect",
    );
}

#[test]
fn live_empirical_history_queries_never_create_new_progress_events() {
    let game = "p4gc3e-live-query-only";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 390_000);

    mature_runtime(&mut runtime, game);

    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let state = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .unwrap()
        .clone();

    let possibility = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(
            &state,
            &cognitive_action,
            athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
                1, 64, 512, 256,
            )
            .unwrap(),
        )
        .unwrap();

    let history_before = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_progress_event_count();

    let transport_before = runtime.transport().execute_count();

    for _ in 0..2 {
        let _ = runtime
            .cognitive_runtime()
            .cognition()
            .current_empirical_expected_epistemic_progress(
                &possibility,
                athlesia_autonomous_active_experimentation::
                    EpistemicForecastDiscriminationPolicy::
                        new(
                            512,
                            512,
                        )
                        .unwrap(),
                athlesia_autonomous_active_experimentation::
                    EmpiricalExpectedEpistemicProgressPolicy::
                        new(
                            256,
                            256,
                            1,
                        )
                        .unwrap(),
            );
    }

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_progress_event_count(),
        history_before,
        "pure expectation queries cannot manufacture empirical history",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        transport_before,
        "pure history queries cannot execute transport",
    );
}
