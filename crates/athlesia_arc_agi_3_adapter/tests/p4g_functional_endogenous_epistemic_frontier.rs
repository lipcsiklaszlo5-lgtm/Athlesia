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

#[test]
fn live_current_state_without_matching_empirical_history_has_zero_epistemic_priority_and_zero_transport()
 {
    let game = "p4gc3f-live-no-matching-history";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 400_000);

    /*
     * Use the already-proven grounding trajectory rather than assuming
     * that one environment turn is sufficient for perceptual grounding.
     *
     * mature_runtime has never visited state 7.
     */
    mature_runtime(&mut runtime, game);

    /*
     * Enter genuinely unseen state 7 through a REAL Action2 turn.
     *
     * Any C3E progress generated by this turn has the PREVIOUS state as
     * source identity. Therefore it cannot be exact matching history for
     * the newly current state 7.
     */
    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let current = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("proven mature + holdout trajectory must ground current state 7")
        .clone();

    let cognitive_action_one = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let cognitive_action_two = ArcAgi3CognitiveProtocolBridge::encode_action(action_two);

    let version_policy =
        athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
            1, 64, 512, 256,
        )
        .expect("positive explanatory bounds");

    let discrimination_policy =
        athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy::new(
            512, 512,
        )
        .expect("positive discrimination bounds");

    let expectation_policy =
        athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy::new(
            256, 256, 1,
        )
        .expect("positive empirical expectation bounds");

    let priority_policy =
        athlesia_autonomous_active_experimentation::EmpiricalEpistemicActionPriorityPolicy::new(8)
            .expect("positive priority bound");

    /*
     * Critical anti-vacuity oracle:
     *
     * Action1 MUST actually be an informative epistemic possibility here.
     * Otherwise "no priority" could pass simply because no epistemic
     * candidate existed at all.
     */
    let possibility = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(&current, &cognitive_action_one, version_policy)
        .expect("holdout state 7 must expose the proven informative Action1 possibility");

    let discrimination =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::
                evaluate(
                    &possibility,
                    discrimination_policy,
                );

    assert!(
        discrimination.informative(),
        "fixture must contain a real epistemic disagreement before testing history gating",
    );

    assert!(discrimination.pairwise_separation_score() > 0,);

    let history_before_query = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_progress_event_count();

    let transport_before_query = runtime.transport().execute_count();

    /*
     * Exact C3E check:
     *
     * The owner may contain empirical events from earlier source states,
     * but NONE matches the exact current state-7 epistemic pattern.
     */
    let expectation = runtime
        .cognitive_runtime()
        .cognition()
        .current_empirical_expected_epistemic_progress(
            &possibility,
            discrimination_policy,
            expectation_policy,
        );

    assert_eq!(
        expectation.status(),
        athlesia_autonomous_active_experimentation::
            EmpiricalExpectedEpistemicProgressStatus::
                NoMatchingEvidence,
        "informative current Action1 without exact retained state-7 history must abstain at C3E",
    );

    assert!(expectation.estimate().is_none(),);

    let actions = [cognitive_action_one.clone(), cognitive_action_two];

    let frontier = runtime
        .cognitive_runtime()
        .cognition()
        .current_empirical_epistemic_action_priority_frontier(
            &current,
            &actions,
            version_policy,
            discrimination_policy,
            expectation_policy,
            priority_policy,
        );

    assert_eq!(
        frontier.status(),
        athlesia_autonomous_active_experimentation::
            EmpiricalEpistemicActionPriorityStatus::
                NoPositiveEmpiricalPriority,
        "real epistemic disagreement without matching empirical progress history must not receive priority",
    );

    assert!(frontier.best().is_none());

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_progress_event_count(),
        history_before_query,
        "expectation and priority queries must not manufacture retained history",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        transport_before_query,
        "C3E/C3F epistemic queries must have zero transport authority",
    );
}

#[test]
fn live_progress_history_from_other_source_state_cannot_prioritize_new_current_state() {
    let game = "p4gc3f-live-source-freshness";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 410_000);

    mature_runtime(&mut runtime, game);

    /*
     * Enter the proven C3D holdout.
     */
    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let history_before_progress = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_progress_event_count();

    /*
     * This is the real C3D progress event:
     *     state 7 --ACTION1--> state 6
     */
    real_training_turn(&mut runtime, game, action_one, 6_u8);

    let history_after_progress = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_progress_event_count();

    assert!(
        history_after_progress > history_before_progress,
        "fixture must contain a genuinely retained live progress event",
    );

    /*
     * Enter a NEW state 8.  Existing retained progress evidence has a
     * different exact source-state identity and therefore must not transfer.
     */
    real_training_turn(&mut runtime, game, action_two, 8_u8);

    let current = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("new current state 8 must be grounded")
        .clone();

    let actions = [
        ArcAgi3CognitiveProtocolBridge::encode_action(action_one),
        ArcAgi3CognitiveProtocolBridge::encode_action(action_two),
    ];

    let history_before_query = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_progress_event_count();

    let transport_before_query = runtime.transport().execute_count();

    let frontier = runtime
        .cognitive_runtime()
        .cognition()
        .current_empirical_epistemic_action_priority_frontier(
        &current,
        &actions,
        athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
            1, 64, 512, 256,
        )
        .unwrap(),
        athlesia_autonomous_active_experimentation::EpistemicForecastDiscriminationPolicy::new(
            512, 512,
        )
        .unwrap(),
        athlesia_autonomous_active_experimentation::EmpiricalExpectedEpistemicProgressPolicy::new(
            256, 256, 1,
        )
        .unwrap(),
        athlesia_autonomous_active_experimentation::EmpiricalEpistemicActionPriorityPolicy::new(8)
            .unwrap(),
    );

    assert_eq!(
        frontier.status(),
        athlesia_autonomous_active_experimentation::
            EmpiricalEpistemicActionPriorityStatus::
                NoPositiveEmpiricalPriority,
        "real progress from another source state must not become fake current-state priority",
    );

    assert!(frontier.best().is_none());

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_progress_event_count(),
        history_before_query,
        "priority query must not manufacture empirical history",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        transport_before_query,
        "epistemic priority remains non-authoritative for transport",
    );
}

#[test]
fn live_real_c3d_event_retains_pre_learning_structural_transfer_identity_with_same_event_provenance()
 {
    let game = "p4gc3g-live-transfer-retention";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 430_000);

    mature_runtime(&mut runtime, game);

    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let current = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("holdout state 7 must be grounded")
        .clone();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let pre_learning = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(
            &current,
            &cognitive_action,
            athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
                1, 64, 512, 256,
            )
            .unwrap(),
        )
        .expect("state7 Action1 must expose pre-learning epistemic possibility");

    let discrimination =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::
                evaluate(
                    &pre_learning,
                    athlesia_autonomous_active_experimentation::
                        EpistemicForecastDiscriminationPolicy::
                            new(
                                512,
                                512,
                            )
                            .unwrap(),
                );

    assert!(discrimination.informative());

    let expected_identity =
        athlesia_autonomous_active_experimentation::
            AutonomousEmpiricalEpistemicTransferIdentity::
                derive(
                    &pre_learning,
                    athlesia_autonomous_active_experimentation::
                        EmpiricalEpistemicTransferIdentityPolicy::
                            new(
                                512,
                            )
                            .unwrap(),
                )
                .identity()
                .expect(
                    "pre-learning transfer identity must derive",
                )
                .clone();

    let exact_before = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_progress_event_count();

    let transfer_before = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_transfer_progress_event_count();

    real_training_turn(&mut runtime, game, action_one, 6_u8);

    let cognition = runtime.cognitive_runtime().cognition();

    assert_eq!(cognition.epistemic_progress_event_count(), exact_before + 1,);

    assert_eq!(
        cognition.epistemic_transfer_progress_event_count(),
        transfer_before + 1,
    );

    let exact = cognition.epistemic_progress_history().last().unwrap();

    let transfer = cognition
        .epistemic_transfer_progress_history()
        .last()
        .unwrap();

    assert_eq!(
        transfer.event_index(),
        exact.event_index(),
        "both histories must share exact completed-turn provenance",
    );

    assert_eq!(
        transfer.sample(),
        exact.sample(),
        "transfer sidecar must retain the exact same C3D sample",
    );

    assert_eq!(
        transfer.transfer_identity(),
        &expected_identity,
        "identity must come from the live PRE-learning possibility",
    );

    assert_eq!(
        transfer.sample().source_state(),
        pre_learning.source_state(),
    );

    assert_eq!(transfer.sample().action(), pre_learning.action(),);
}

#[test]
fn live_noninformative_turn_cannot_manufacture_transfer_progress_history() {
    let game = "p4gc3g-live-no-fake-transfer";

    let action_one = action(ArcAgi3ActionId::Action1);

    let mut runtime = live_runtime(game, 440_000);

    mature_runtime(&mut runtime, game);

    let current = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .expect("mature current state must be grounded")
        .clone();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let possibility = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(
            &current,
            &cognitive_action,
            athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
                1, 64, 512, 256,
            )
            .unwrap(),
        )
        .expect("known action may remain modeled");

    let discrimination =
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
                            .unwrap(),
                );

    assert!(
        !discrimination.informative(),
        "negative control must begin epistemically resolved",
    );

    let exact_before = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_progress_event_count();

    let transfer_before = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_transfer_progress_event_count();

    real_training_turn(&mut runtime, game, action_one, 6_u8);

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_progress_event_count(),
        exact_before,
    );

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_transfer_progress_event_count(),
        transfer_before,
        "epistemically resolved real turn must not fabricate transfer progress",
    );
}

#[test]
fn live_real_transfer_history_can_estimate_structurally_identical_other_source_query_without_transport()
 {
    let game = "p4gc3g-live-transfer-estimate";

    let action_one = action(ArcAgi3ActionId::Action1);

    let action_two = action(ArcAgi3ActionId::Action2);

    let mut runtime = live_runtime(game, 450_000);

    mature_runtime(&mut runtime, game);

    real_training_turn(&mut runtime, game, action_two, 7_u8);

    let current = runtime
        .cognitive_runtime()
        .current_grounded_world_state()
        .unwrap()
        .clone();

    let cognitive_action = ArcAgi3CognitiveProtocolBridge::encode_action(action_one);

    let pre_learning = runtime
        .cognitive_runtime()
        .cognition()
        .current_m50_epistemic_possibility(
            &current,
            &cognitive_action,
            athlesia_universal_domain_learning::GroundedExplanatoryVersionSpacePolicy::new(
                1, 64, 512, 256,
            )
            .unwrap(),
        )
        .expect("state7 Action1 must expose real pre-learning possibility");

    let transfer_before = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_transfer_progress_event_count();

    real_training_turn(&mut runtime, game, action_one, 6_u8);

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_transfer_progress_event_count(),
        transfer_before + 1,
        "fixture must contain one genuinely retained live C3D transfer event",
    );

    /*
     * C3G-C functional transfer contract:
     *
     * Keep exact action + complete forecasts, change ONLY the concrete
     * source-state identity.
     *
     * This is deliberately a counterfactual transfer query, not yet a
     * claim that the live environment has endogenously entered such a
     * second state. C3G-D will test that stronger condition.
     */
    let counterfactual_source = pre_learning.action().clone();

    assert_ne!(&counterfactual_source, pre_learning.source_state(),);

    let other_source =
        athlesia_autonomous_active_experimentation::GroundedEpistemicExperimentPossibility::new(
            counterfactual_source,
            pre_learning.action().clone(),
            pre_learning.forecasts().to_vec(),
        )
        .expect("structurally identical counterfactual possibility");

    let expected_identity =
        athlesia_autonomous_active_experimentation::
            AutonomousEmpiricalEpistemicTransferIdentity::
                derive(
                    &pre_learning,
                    athlesia_autonomous_active_experimentation::
                        EmpiricalEpistemicTransferIdentityPolicy::
                            new(
                                512,
                            )
                            .unwrap(),
                )
                .identity()
                .unwrap()
                .clone();

    let other_identity =
        athlesia_autonomous_active_experimentation::
            AutonomousEmpiricalEpistemicTransferIdentity::
                derive(
                    &other_source,
                    athlesia_autonomous_active_experimentation::
                        EmpiricalEpistemicTransferIdentityPolicy::
                            new(
                                512,
                            )
                            .unwrap(),
                )
                .identity()
                .unwrap()
                .clone();

    assert_eq!(
        expected_identity, other_identity,
        "only concrete source identity may differ in the transfer-positive fixture",
    );

    let transport_before = runtime.transport().execute_count();

    let history_before = runtime
        .cognitive_runtime()
        .cognition()
        .epistemic_transfer_progress_event_count();

    let result = runtime
        .cognitive_runtime()
        .cognition()
        .current_empirical_expected_epistemic_transfer_progress(
            &other_source,
            athlesia_autonomous_active_experimentation::
                EmpiricalEpistemicTransferIdentityPolicy::
                    new(
                        512,
                    )
                    .unwrap(),
            athlesia_autonomous_active_experimentation::
                EpistemicForecastDiscriminationPolicy::
                    new(
                        512,
                        512,
                    )
                    .unwrap(),
            athlesia_autonomous_active_experimentation::
                EmpiricalExpectedEpistemicTransferProgressPolicy::
                    new(
                        256,
                        256,
                        1,
                    )
                    .unwrap(),
        );

    assert!(result.estimated());

    assert_eq!(result.cross_state_matching_evidence_count(), 1,);

    assert_eq!(result.estimate().unwrap().qualifying_sample_count(), 1,);

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_transfer_progress_event_count(),
        history_before,
        "transfer estimation query must not mutate retained history",
    );

    assert_eq!(
        runtime.transport().execute_count(),
        transport_before,
        "transfer estimator has zero transport authority",
    );
}

fn c3h_distinct_transfer_targets(
    identity:
        &athlesia_autonomous_active_experimentation::
            EmpiricalEpistemicTransferIdentity,
) -> Vec<CognitiveStructure> {
    let mut result =
        Vec::new();

    for forecast in
        identity.forecasts()
    {
        let target =
            forecast.target().clone();

        if !result.contains(
            &target,
        ) {
            result.push(target);
        }
    }

    result
}


fn c3h_target_difference(
    left: &[CognitiveStructure],
    right: &[CognitiveStructure],
) -> Vec<CognitiveStructure> {
    left.iter()
        .filter(|candidate| {
            !right.contains(candidate)
        })
        .cloned()
        .collect()
}


fn c3h_binding_signature(
    result:
        &athlesia_autonomous_active_experimentation::
            RolePreservingTargetSchemaResult,
) -> Vec<(u64, u64)> {
    let mut signature =
        result.bindings()
            .iter()
            .map(|binding| {
                (
                    binding.historical_atom(),
                    binding.current_atom(),
                )
            })
            .collect::<Vec<_>>();

    signature.sort();

    signature
}


#[test]
fn live_c3h_role_preserving_schema_recovers_d6_target_substitution() {
    let mut cross_world_binding_signature:
        Option<Vec<(u64, u64)>> =
        None;

    /*
     * Two separated holdout response values from the D6 family.
     * The schema must emerge from live cognition, not from manually
     * constructed target fixtures.
     */
    for (
        world_index,
        candidate_value,
    ) in [
        2_u8,
        14_u8,
    ]
    .into_iter()
    .enumerate()
    {
        let game = format!(
            "p4gc3h-live-role-schema-{candidate_value}"
        );

        let mut runtime =
            live_runtime(
                &game,
                700_000
                    + world_index as u64
                        * 10_000,
            );

        mature_runtime(
            &mut runtime,
            &game,
        );

        real_training_turn(
            &mut runtime,
            &game,
            action(
                ArcAgi3ActionId::Action2,
            ),
            7_u8,
        );

        let state7 = runtime
            .cognitive_runtime()
            .current_grounded_world_state()
            .unwrap()
            .clone();

        let action_one =
            ArcAgi3CognitiveProtocolBridge::
                encode_action(
                    action(
                        ArcAgi3ActionId::Action1,
                    ),
                );

        let pre_learning =
            runtime
                .cognitive_runtime()
                .cognition()
                .current_m50_epistemic_possibility(
                    &state7,
                    &action_one,
                    athlesia_universal_domain_learning::
                        GroundedExplanatoryVersionSpacePolicy::
                            new(
                                1,
                                64,
                                512,
                                256,
                            )
                            .unwrap(),
                )
                .expect(
                    "state7 Action1 must be informative before the real progress event",
                );

        let pre_discrimination =
            athlesia_autonomous_active_experimentation::
                AutonomousEpistemicForecastDiscrimination::
                    evaluate(
                        &pre_learning,
                        athlesia_autonomous_active_experimentation::
                            EpistemicForecastDiscriminationPolicy::
                                new(
                                    512,
                                    512,
                                )
                                .unwrap(),
                    );

        assert!(
            pre_discrimination.informative(),
        );

        let history_before =
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count();

        real_training_turn(
            &mut runtime,
            &game,
            action(
                ArcAgi3ActionId::Action1,
            ),
            6_u8,
        );

        assert_eq!(
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count(),
            history_before + 1,
        );

        let historical_identity =
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_history()
                .last()
                .unwrap()
                .transfer_identity()
                .clone();

        /*
         * The second real Action1 transition creates the different
         * live source-state target frontier observed in D6.
         */
        real_training_turn(
            &mut runtime,
            &game,
            action(
                ArcAgi3ActionId::Action1,
            ),
            candidate_value,
        );

        let current_state =
            runtime
                .cognitive_runtime()
                .current_grounded_world_state()
                .unwrap()
                .clone();

        let current_possibility =
            runtime
                .cognitive_runtime()
                .cognition()
                .current_m50_epistemic_possibility(
                    &current_state,
                    &action_one,
                    athlesia_universal_domain_learning::
                        GroundedExplanatoryVersionSpacePolicy::
                            new(
                                1,
                                64,
                                512,
                                256,
                            )
                            .unwrap(),
                )
                .expect(
                    "holdout live current state must expose Action1 possibility",
                );

        let current_discrimination =
            athlesia_autonomous_active_experimentation::
                AutonomousEpistemicForecastDiscrimination::
                    evaluate(
                        &current_possibility,
                        athlesia_autonomous_active_experimentation::
                            EpistemicForecastDiscriminationPolicy::
                                new(
                                    512,
                                    512,
                                )
                                .unwrap(),
                    );

        assert!(
            current_discrimination.informative(),
        );

        let current_identity =
            athlesia_autonomous_active_experimentation::
                AutonomousEmpiricalEpistemicTransferIdentity::
                    derive(
                        &current_possibility,
                        athlesia_autonomous_active_experimentation::
                            EmpiricalEpistemicTransferIdentityPolicy::
                                new(
                                    512,
                                )
                                .unwrap(),
                    )
                    .identity()
                    .unwrap()
                    .clone();

        let historical_targets =
            c3h_distinct_transfer_targets(
                &historical_identity,
            );

        let current_targets =
            c3h_distinct_transfer_targets(
                &current_identity,
            );

        let lost =
            c3h_target_difference(
                &historical_targets,
                &current_targets,
            );

        let gained =
            c3h_target_difference(
                &current_targets,
                &historical_targets,
            );

        assert_eq!(
            lost.len(),
            4,
            "D6 live family must expose four historical targets replaced in the new source state",
        );

        assert_eq!(
            gained.len(),
            4,
        );

        let transport_before_schema_queries =
            runtime.transport().execute_count();

        let history_before_schema_queries =
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count();

        let mut world_signature:
            Option<Vec<(u64, u64)>> =
            None;

        let mut mapped =
            0_usize;

        for historical_target in
            &lost
        {
            let mut best:
                Option<
                    athlesia_autonomous_active_experimentation::
                        RolePreservingTargetSchemaResult,
                > =
                None;

            for current_target in
                &gained
            {
                let candidate =
                    athlesia_autonomous_active_experimentation::
                        AutonomousRolePreservingTargetSchema::
                            derive(
                                historical_target,
                                current_target,
                                athlesia_autonomous_active_experimentation::
                                    RolePreservingTargetSchemaPolicy::
                                        new(
                                            256,
                                            32,
                                        )
                                        .unwrap(),
                            );

                if !candidate.derived()
                    || candidate.role_count() == 0
                {
                    continue;
                }

                let replace =
                    best.as_ref()
                        .map(|existing| {
                            (
                                candidate.role_count(),
                                candidate
                                    .substitution_occurrence_count(),
                            )
                            <
                            (
                                existing.role_count(),
                                existing
                                    .substitution_occurrence_count(),
                            )
                        })
                        .unwrap_or(true);

                if replace {
                    best =
                        Some(candidate);
                }
            }

            let best =
                best.expect(
                    "every D6 lost target must have a role-preserving live replacement",
                );

            /*
             * D6 showed two UNIQUE bindings even when one binding is
             * repeated at multiple structural positions.
             */
            assert_eq!(
                best.role_count(),
                2,
            );

            let signature =
                c3h_binding_signature(
                    &best,
                );

            assert_eq!(
                signature.len(),
                2,
            );

            if let Some(expected) =
                &world_signature
            {
                assert_eq!(
                    &signature,
                    expected,
                    "all four target replacements in one live world must share the same role binding relation",
                );
            } else {
                world_signature =
                    Some(signature);
            }

            mapped += 1;
        }

        assert_eq!(
            mapped,
            4,
        );

        let world_signature =
            world_signature.unwrap();

        if let Some(expected) =
            &cross_world_binding_signature
        {
            assert_eq!(
                &world_signature,
                expected,
                "separated live holdout response values must recover the same structural substitution relation",
            );
        } else {
            cross_world_binding_signature =
                Some(world_signature);
        }

        assert_eq!(
            runtime.transport().execute_count(),
            transport_before_schema_queries,
            "C3H-A schema derivation has zero transport authority",
        );

        assert_eq!(
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count(),
            history_before_schema_queries,
            "C3H-A schema derivation cannot mutate retained empirical history",
        );
    }

    assert!(
        cross_world_binding_signature.is_some(),
    );
}

fn c3hb_live_historical_and_current_identity(
    game: &str,
    first_index: u64,
    route_action:
        ArcAgi3ActionId,
    candidate_value: u8,
) -> (
    ArcAgi3LiveEnvironmentRuntime<
        RecordingTransport,
    >,
    athlesia_autonomous_active_experimentation::
        EmpiricalEpistemicTransferIdentity,
    athlesia_autonomous_active_experimentation::
        EmpiricalEpistemicTransferIdentity,
) {
    let mut runtime =
        live_runtime(
            game,
            first_index,
        );

    mature_runtime(
        &mut runtime,
        game,
    );

    real_training_turn(
        &mut runtime,
        game,
        action(
            ArcAgi3ActionId::Action2,
        ),
        7_u8,
    );

    /*
     * Real Action1 consequence creates the genuine historical C3D
     * transfer event retained by C3G-B.
     */
    real_training_turn(
        &mut runtime,
        game,
        action(
            ArcAgi3ActionId::Action1,
        ),
        6_u8,
    );

    let historical =
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_transfer_progress_history()
            .last()
            .expect(
                "real historical C3D transfer event must exist",
            )
            .transfer_identity()
            .clone();

    real_training_turn(
        &mut runtime,
        game,
        action(route_action),
        candidate_value,
    );

    let current_state =
        runtime
            .cognitive_runtime()
            .current_grounded_world_state()
            .expect(
                "candidate must be a genuinely reached grounded state",
            )
            .clone();

    let action_one =
        ArcAgi3CognitiveProtocolBridge::
            encode_action(
                action(
                    ArcAgi3ActionId::Action1,
                ),
            );

    let current_possibility =
        runtime
            .cognitive_runtime()
            .cognition()
            .current_m50_epistemic_possibility(
                &current_state,
                &action_one,
                athlesia_universal_domain_learning::
                    GroundedExplanatoryVersionSpacePolicy::
                        new(
                            1,
                            64,
                            512,
                            256,
                        )
                        .unwrap(),
            )
            .expect(
                "current live state must expose Action1 possibility",
            );

    let discrimination =
        athlesia_autonomous_active_experimentation::
            AutonomousEpistemicForecastDiscrimination::
                evaluate(
                    &current_possibility,
                    athlesia_autonomous_active_experimentation::
                        EpistemicForecastDiscriminationPolicy::
                            new(
                                512,
                                512,
                            )
                            .unwrap(),
                );

    assert!(
        discrimination.informative(),
        "C3H-B live comparison remains restricted to a genuinely unresolved current epistemic problem",
    );

    let current =
        athlesia_autonomous_active_experimentation::
            AutonomousEmpiricalEpistemicTransferIdentity::
                derive(
                    &current_possibility,
                    athlesia_autonomous_active_experimentation::
                        EmpiricalEpistemicTransferIdentityPolicy::
                            new(
                                512,
                            )
                            .unwrap(),
                )
                .identity()
                .expect(
                    "bounded current possibility must derive exact C3G identity",
                )
                .clone();

    (
        runtime,
        historical,
        current,
    )
}


fn c3hb_policy(
) -> athlesia_autonomous_active_experimentation::
    SchemaLevelTargetTransferIdentityPolicy {
    athlesia_autonomous_active_experimentation::
        SchemaLevelTargetTransferIdentityPolicy::
            new(
                64,
                64,
                4096,
                athlesia_autonomous_active_experimentation::
                    RolePreservingTargetSchemaPolicy::
                        new(
                            512,
                            32,
                        )
                        .unwrap(),
            )
            .unwrap()
}


#[test]
fn live_c3hb_schema_level_identity_recovers_role_substitution_across_holdouts() {
    let mut shared_identity:
        Option<
            athlesia_autonomous_active_experimentation::
                SchemaLevelTargetTransferIdentity,
        > =
        None;

    for (
        world_index,
        candidate_value,
    ) in [
        2_u8,
        14_u8,
    ]
    .into_iter()
    .enumerate()
    {
        let game = format!(
            "p4gc3hb-role-holdout-{candidate_value}"
        );

        let (
            runtime,
            historical,
            current,
        ) =
            c3hb_live_historical_and_current_identity(
                &game,
                760_000
                    + world_index as u64
                        * 10_000,
                ArcAgi3ActionId::Action1,
                candidate_value,
            );

        let transport_before =
            runtime.transport().execute_count();

        let exact_history_before =
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_progress_event_count();

        let transfer_history_before =
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count();

        let relation =
            athlesia_autonomous_active_experimentation::
                AutonomousSchemaLevelTargetTransferIdentity::
                    derive(
                        &historical,
                        &current,
                        c3hb_policy(),
                    );

        assert!(
            relation.derived(),
            "D6-proven live role substitutions must form one bounded schema-level historical-to-current relation",
        );

        assert_eq!(
            relation.historical_target_count(),
            16,
        );

        assert_eq!(
            relation.current_target_count(),
            16,
        );

        assert_eq!(
            relation.exact_match_count(),
            12,
        );

        assert_eq!(
            relation.role_preserving_match_count(),
            4,
        );

        assert_eq!(
            relation.ignored_current_target_count(),
            0,
        );

        assert_eq!(
            relation.global_binding_count(),
            2,
            "four replaced live targets must share the same two concrete changed-atom bindings",
        );

        let identity =
            relation
                .identity()
                .unwrap()
                .clone();

        if let Some(expected) =
            &shared_identity
        {
            assert_eq!(
                &identity,
                expected,
                "separated real holdout worlds must recover the same abstract target-transfer identity",
            );
        } else {
            shared_identity =
                Some(identity);
        }

        assert_eq!(
            runtime.transport().execute_count(),
            transport_before,
            "C3H-B derivation has zero transport authority",
        );

        assert_eq!(
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_progress_event_count(),
            exact_history_before,
            "C3H-B cannot mutate frozen exact progress history",
        );

        assert_eq!(
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count(),
            transfer_history_before,
            "C3H-B cannot mutate retained transfer history",
        );
    }

    assert!(
        shared_identity.is_some(),
    );
}


#[test]
fn live_c3hb_current_only_target_refinement_preserves_all_historical_targets_exactly() {
    let (
        runtime,
        historical,
        current,
    ) =
        c3hb_live_historical_and_current_identity(
            "p4gc3hb-exact-refinement-control",
            790_000,
            ArcAgi3ActionId::Action2,
            2_u8,
        );

    let transport_before =
        runtime.transport().execute_count();

    let relation =
        athlesia_autonomous_active_experimentation::
            UniversalAutonomousSchemaLevelTargetTransferIdentity::
                derive(
                    &historical,
                    &current,
                    c3hb_policy(),
                );

    assert!(
        relation.derived(),
    );

    assert_eq!(
        relation.historical_target_count(),
        16,
    );

    assert_eq!(
        relation.current_target_count(),
        20,
    );

    assert_eq!(
        relation.exact_match_count(),
        16,
        "D5/D6 Action2 control preserves every historical target exactly",
    );

    assert_eq!(
        relation.role_preserving_match_count(),
        0,
        "current-only refinement must not manufacture role substitution",
    );

    assert_eq!(
        relation.ignored_current_target_count(),
        4,
        "four genuinely new current targets are explicit refinement rather than historical mismatch",
    );

    assert_eq!(
        relation.global_binding_count(),
        0,
    );

    assert_eq!(
        runtime.transport().execute_count(),
        transport_before,
    );
}

#[test]
fn live_c3h_c8_target_anchored_context_transformation_recovers_unique_hypothesis_correspondence() {
    use athlesia_autonomous_active_experimentation::{
        AutonomousTargetAnchoredContextTransformation,
        GroundedTargetAnchoredContextTransformationIdentity,
        TargetAnchoredContextTopologyClass,
        TargetAnchoredContextTransformationPolicy,
    };

    let policy =
        TargetAnchoredContextTransformationPolicy::
            new(
                32,
                4096,
                2048,
            )
            .unwrap();

    let mut shared_identity:
        Option<
            GroundedTargetAnchoredContextTransformationIdentity
        > =
        None;

    let mut direct_count =
        0_usize;

    let mut recursive_count =
        0_usize;

    let mut total_derived_pairs =
        0_usize;


    for (
        world_index,
        candidate_value,
    ) in [
        2_u8,
        14_u8,
    ]
    .into_iter()
    .enumerate()
    {
        let game =
            format!(
                "p4gc3hc8-role-{candidate_value}"
            );

        let (
            runtime,
            historical,
            current,
        ) =
            c3hb_live_historical_and_current_identity(
                &game,
                1_240_000
                    + world_index as u64
                        * 20_000,
                ArcAgi3ActionId::Action1,
                candidate_value,
            );


        let transport_before =
            runtime
                .transport()
                .execute_count();

        let exact_history_before =
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_progress_event_count();

        let transfer_history_before =
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count();


        let relation =
            athlesia_autonomous_active_experimentation::
                AutonomousSchemaLevelTargetTransferIdentity::
                    derive(
                        &historical,
                        &current,
                        c3hb_policy(),
                    );

        assert!(relation.derived());

        assert_eq!(
            relation
                .role_preserving_match_count(),
            4,
        );


        let mut role_target_count =
            0_usize;

        for correspondence in
            relation.correspondences()
        {
            if correspondence.match_kind()
                != athlesia_autonomous_active_experimentation::
                    SchemaLevelTargetMatchKind::
                        RolePreserving
            {
                continue;
            }

            role_target_count += 1;


            let historical_context =
                historical
                    .forecasts()
                    .iter()
                    .filter(|forecast| {
                        forecast.target()
                            == correspondence
                                .historical_target()
                            && format!(
                                "{:?}",
                                forecast.status(),
                            ) == "ContextAbstained"
                    })
                    .collect::<Vec<_>>();

            let current_context =
                current
                    .forecasts()
                    .iter()
                    .filter(|forecast| {
                        forecast.target()
                            == correspondence
                                .current_target()
                            && format!(
                                "{:?}",
                                forecast.status(),
                            ) == "ContextAbstained"
                    })
                    .collect::<Vec<_>>();

            assert_eq!(
                historical_context.len(),
                4,
            );

            assert_eq!(
                current_context.len(),
                4,
            );


            let mut left_degree =
                vec![
                    0_usize;
                    historical_context.len()
                ];

            let mut right_degree =
                vec![
                    0_usize;
                    current_context.len()
                ];

            let mut target_derived_count =
                0_usize;


            for (
                historical_index,
                historical_forecast,
            ) in historical_context
                .iter()
                .enumerate()
            {
                for (
                    current_index,
                    current_forecast,
                ) in current_context
                    .iter()
                    .enumerate()
                {
                    let derived =
                        AutonomousTargetAnchoredContextTransformation::
                            derive(
                                historical_forecast
                                    .hypothesis(),
                                current_forecast
                                    .hypothesis(),
                                correspondence,
                                policy,
                            );

                    if !derived.derived() {
                        continue;
                    }

                    target_derived_count += 1;
                    total_derived_pairs += 1;

                    left_degree[
                        historical_index
                    ] += 1;

                    right_degree[
                        current_index
                    ] += 1;


                    let transformation =
                        derived
                            .transformation()
                            .unwrap();

                    assert_eq!(
                        transformation
                            .target_anchor_topology(),
                        transformation
                            .context_merge_topology(),
                        "production relation must reproduce frozen C3H-C7 target-conditioned realization",
                    );

                    match transformation
                        .target_anchor_topology()
                    {
                        TargetAnchoredContextTopologyClass::
                            Direct =>
                        {
                            direct_count += 1;
                        }

                        TargetAnchoredContextTopologyClass::
                            Recursive =>
                        {
                            recursive_count += 1;
                        }
                    }


                    let identity =
                        transformation
                            .identity()
                            .clone();

                    if let Some(
                        expected,
                    ) = &shared_identity
                    {
                        assert_eq!(
                            &identity,
                            expected,
                            "all direct/recursive and cross-world realizations must share one abstract context transformation identity",
                        );
                    } else {
                        shared_identity =
                            Some(identity);
                    }
                }
            }


            assert_eq!(
                target_derived_count,
                4,
                "production representation must recover exactly four of sixteen candidate hypothesis pairs",
            );

            assert!(
                left_degree
                    .iter()
                    .all(|degree| {
                        *degree == 1
                    }),
                "every historical context hypothesis must have exactly one grounded current continuation",
            );

            assert!(
                right_degree
                    .iter()
                    .all(|degree| {
                        *degree == 1
                    }),
                "every current context hypothesis must be consumed by exactly one grounded historical continuation",
            );
        }


        assert_eq!(
            role_target_count,
            4,
        );


        assert_eq!(
            runtime
                .transport()
                .execute_count(),
            transport_before,
            "C3H-C8 derivation has zero transport authority",
        );

        assert_eq!(
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_progress_event_count(),
            exact_history_before,
            "C3H-C8 cannot mutate exact empirical progress history",
        );

        assert_eq!(
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count(),
            transfer_history_before,
            "C3H-C8 cannot mutate frozen structural transfer history",
        );
    }


    assert_eq!(
        total_derived_pairs,
        32,
    );

    assert_eq!(
        direct_count,
        16,
    );

    assert_eq!(
        recursive_count,
        16,
    );

    assert!(
        shared_identity.is_some(),
    );
}

#[test]
fn live_c3h_c10_grounded_forecast_correspondence_closes_full_frontier_before_status_analysis() {
    use athlesia_autonomous_active_experimentation::{
        AutonomousGroundedForecastCorrespondence,
        EpistemicHypothesisForecastStatus,
        GroundedForecastCorrespondencePolicy,
        GroundedForecastCorrespondenceRelation,
    };

    let policy =
        GroundedForecastCorrespondencePolicy::
            new(
                32,
                10_000,

                32,
                4096,
                2048,

                4096,
                256,
            )
            .unwrap();

    let mut total_forecasts =
        0_usize;

    let mut total_context =
        0_usize;

    let mut total_base =
        0_usize;

    let mut context_status_preserved =
        0_usize;

    let mut base_predicted_to_no_opportunity =
        0_usize;

    let mut context_none_to_none =
        0_usize;

    let mut base_some_to_none =
        0_usize;

    let mut evidence_maturity_changed =
        0_usize;

    let mut shared_context_identity = None;


    for (
        world_index,
        candidate_value,
    ) in [
        2_u8,
        14_u8,
    ]
    .into_iter()
    .enumerate()
    {
        let game =
            format!(
                "p4gc3hc10-role-{candidate_value}"
            );

        let (
            runtime,
            historical,
            current,
        ) =
            c3hb_live_historical_and_current_identity(
                &game,
                1_340_000
                    + world_index as u64
                        * 20_000,
                ArcAgi3ActionId::Action1,
                candidate_value,
            );


        let transport_before =
            runtime
                .transport()
                .execute_count();

        let exact_history_before =
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_progress_event_count();

        let transfer_history_before =
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count();


        let target_relation =
            athlesia_autonomous_active_experimentation::
                AutonomousSchemaLevelTargetTransferIdentity::
                    derive(
                        &historical,
                        &current,
                        c3hb_policy(),
                    );

        assert!(
            target_relation.derived(),
        );

        assert_eq!(
            target_relation
                .role_preserving_match_count(),
            4,
        );


        let mut role_target_count =
            0_usize;


        for target_correspondence in
            target_relation.correspondences()
        {
            if target_correspondence
                .match_kind()
                != athlesia_autonomous_active_experimentation::
                    SchemaLevelTargetMatchKind::
                        RolePreserving
            {
                continue;
            }

            role_target_count += 1;


            let result =
                AutonomousGroundedForecastCorrespondence::
                    derive(
                        &historical,
                        &current,
                        target_correspondence,
                        policy,
                    );

            assert!(
                result.derived(),
                "frozen C3H-C9 structural graph must derive one grounded correspondence",
            );

            assert_eq!(
                result
                    .matching_solution_count(),
                1,
            );

            let correspondence =
                result
                    .correspondence()
                    .unwrap();

            assert_eq!(
                correspondence
                    .forecast_count(),
                5,
            );

            assert_eq!(
                correspondence
                    .context_transformation_count(),
                4,
            );

            assert_eq!(
                correspondence
                    .target_bound_continuation_count(),
                1,
            );

            assert_eq!(
                result
                    .candidate_edge_count(),
                5,
                "C3H-C9 established exactly five non-overlapping structural candidate edges per role target",
            );


            let mut seen_current =
                std::collections::BTreeSet::<
                    usize
                >::new();

            for entry in
                correspondence.entries()
            {
                total_forecasts += 1;

                assert!(
                    seen_current.insert(
                        entry
                            .current_forecast_index(),
                    ),
                    "grounded correspondence must be one-to-one",
                );


                let historical_forecast =
                    &historical
                        .forecasts()[
                            entry
                                .historical_forecast_index()
                        ];

                let current_forecast =
                    &current
                        .forecasts()[
                            entry
                                .current_forecast_index()
                        ];


                /*
                 * These checks are intentionally AFTER structural
                 * correspondence derivation.
                 *
                 * Status/outcome is observed provenance here, never
                 * matching authority.
                 */
                match entry.relation() {
                    GroundedForecastCorrespondenceRelation::
                        TargetAnchoredContextTransformation(
                            identity,
                        ) =>
                    {
                        total_context += 1;

                        assert_eq!(
                            historical_forecast
                                .status(),
                            EpistemicHypothesisForecastStatus::
                                ContextAbstained,
                        );

                        assert_eq!(
                            current_forecast
                                .status(),
                            EpistemicHypothesisForecastStatus::
                                ContextAbstained,
                        );

                        context_status_preserved +=
                            1;

                        assert!(
                            historical_forecast
                                .predicted_outcome()
                                .is_none(),
                        );

                        assert!(
                            current_forecast
                                .predicted_outcome()
                                .is_none(),
                        );

                        context_none_to_none +=
                            1;


                        if let Some(
                            expected,
                        ) =
                            &shared_context_identity
                        {
                            assert_eq!(
                                identity,
                                expected,
                            );
                        } else {
                            shared_context_identity =
                                Some(
                                    identity.clone(),
                                );
                        }
                    }

                    GroundedForecastCorrespondenceRelation::
                        TargetBoundHypothesisContinuation =>
                    {
                        total_base += 1;

                        assert_eq!(
                            historical_forecast
                                .status(),
                            EpistemicHypothesisForecastStatus::
                                Predicted,
                        );

                        assert_eq!(
                            current_forecast
                                .status(),
                            EpistemicHypothesisForecastStatus::
                                NoEffectOpportunity,
                        );

                        base_predicted_to_no_opportunity +=
                            1;

                        assert!(
                            historical_forecast
                                .predicted_outcome()
                                .is_some(),
                        );

                        assert!(
                            current_forecast
                                .predicted_outcome()
                                .is_none(),
                        );

                        base_some_to_none +=
                            1;
                    }
                }


                /*
                 * C3H-C9 measured (-1,-1,0) on every matched edge.
                 *
                 * This remains source evidence and is deliberately
                 * absent from GroundedForecastCorrespondenceRelation.
                 */
                assert_eq!(
                    current_forecast
                        .support_count()
                        .checked_add(1),
                    Some(
                        historical_forecast
                            .support_count(),
                    ),
                );

                assert_eq!(
                    current_forecast
                        .opportunity_count()
                        .checked_add(1),
                    Some(
                        historical_forecast
                            .opportunity_count(),
                    ),
                );

                assert_eq!(
                    current_forecast
                        .counterexample_count(),
                    historical_forecast
                        .counterexample_count(),
                );

                evidence_maturity_changed +=
                    1;
            }

            assert_eq!(
                seen_current.len(),
                5,
            );
        }


        assert_eq!(
            role_target_count,
            4,
        );


        assert_eq!(
            runtime
                .transport()
                .execute_count(),
            transport_before,
            "C3H-C10 correspondence has zero transport authority",
        );

        assert_eq!(
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_progress_event_count(),
            exact_history_before,
            "C3H-C10 cannot mutate exact progress history",
        );

        assert_eq!(
            runtime
                .cognitive_runtime()
                .cognition()
                .epistemic_transfer_progress_event_count(),
            transfer_history_before,
            "C3H-C10 cannot mutate transfer progress history",
        );
    }


    assert_eq!(
        total_forecasts,
        40,
    );

    assert_eq!(
        total_context,
        32,
    );

    assert_eq!(
        total_base,
        8,
    );

    assert_eq!(
        context_status_preserved,
        32,
    );

    assert_eq!(
        base_predicted_to_no_opportunity,
        8,
    );

    assert_eq!(
        context_none_to_none,
        32,
    );

    assert_eq!(
        base_some_to_none,
        8,
    );

    assert_eq!(
        evidence_maturity_changed,
        40,
    );

    assert!(
        shared_context_identity
            .is_some(),
    );
}


#[test]
fn live_c3h_c10_exact_target_correspondence_cannot_be_reinterpreted_as_role_forecast_transfer() {
    use athlesia_autonomous_active_experimentation::{
        AutonomousGroundedForecastCorrespondence,
        GroundedForecastCorrespondencePolicy,
        GroundedForecastCorrespondenceStatus,
    };

    let (
        runtime,
        historical,
        current,
    ) =
        c3hb_live_historical_and_current_identity(
            "p4gc3hc10-exact-target-control",
            1_390_000,
            ArcAgi3ActionId::Action2,
            2_u8,
        );


    let transport_before =
        runtime
            .transport()
            .execute_count();

    let exact_history_before =
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_progress_event_count();

    let transfer_history_before =
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_transfer_progress_event_count();


    let target_relation =
        athlesia_autonomous_active_experimentation::
            AutonomousSchemaLevelTargetTransferIdentity::
                derive(
                    &historical,
                    &current,
                    c3hb_policy(),
                );

    assert!(
        target_relation.derived(),
    );

    let exact =
        target_relation
            .correspondences()
            .iter()
            .find(|correspondence| {
                correspondence.match_kind()
                    == athlesia_autonomous_active_experimentation::
                        SchemaLevelTargetMatchKind::
                            Exact
            })
            .expect(
                "C3H-B refinement control contains exact historical target correspondence",
            );


    let result =
        AutonomousGroundedForecastCorrespondence::
            derive(
                &historical,
                &current,
                exact,
                GroundedForecastCorrespondencePolicy::
                    new(
                        32,
                        10_000,
                        32,
                        4096,
                        2048,
                        4096,
                        256,
                    )
                    .unwrap(),
            );


    assert_eq!(
        result.status(),
        GroundedForecastCorrespondenceStatus::
            TargetCorrespondenceNotRolePreserving,
    );

    assert!(
        result
            .correspondence()
            .is_none(),
    );


    assert_eq!(
        runtime
            .transport()
            .execute_count(),
        transport_before,
    );

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_progress_event_count(),
        exact_history_before,
    );

    assert_eq!(
        runtime
            .cognitive_runtime()
            .cognition()
            .epistemic_transfer_progress_event_count(),
        transfer_history_before,
    );
}


#[test]
fn live_c3h_c15b_semantic_provenance_preserves_c10_and_dynamic_state_causality() {
    let (
        runtime2,
        historical2,
        current2,
    ) =
        c3hb_live_historical_and_current_identity(
            "p4gc3hc15b-state2",
            6_000_000,
            ArcAgi3ActionId::Action1,
            2,
        );

    let (
        runtime7,
        historical7,
        current7,
    ) =
        c3hb_live_historical_and_current_identity(
            "p4gc3hc15b-state7",
            6_100_000,
            ArcAgi3ActionId::Action1,
            7,
        );

    assert_eq!(
        historical2,
        historical7,
        "C14B authority: historical transfer identity is equal",
    );

    let state2 =
        runtime2
            .cognitive_runtime()
            .current_grounded_world_state()
            .expect(
                "state2 grounded state",
            )
            .clone();

    let state7 =
        runtime7
            .cognitive_runtime()
            .current_grounded_world_state()
            .expect(
                "state7 grounded state",
            )
            .clone();

    let action_one =
        ArcAgi3CognitiveProtocolBridge::
            encode_action(
                action(
                    ArcAgi3ActionId::Action1,
                ),
            );

    let policy =
        || {
            athlesia_universal_domain_learning::
                GroundedExplanatoryVersionSpacePolicy::
                    new(
                        1,
                        64,
                        512,
                        256,
                    )
                    .unwrap()
        };

    let frozen2 =
        runtime2
            .cognitive_runtime()
            .cognition()
            .current_m50_epistemic_possibility(
                &state2,
                &action_one,
                policy(),
            )
            .expect(
                "frozen state2 M50 possibility",
            );

    let semantic2 =
        runtime2
            .cognitive_runtime()
            .cognition()
            .current_semantic_m50_epistemic_possibility(
                &state2,
                &action_one,
                policy(),
            )
            .expect(
                "state2 typed semantic sidecar",
            );

    assert_eq!(
        semantic2
            .m50_possibility(),
        &frozen2,
        "semantic sidecar cannot alter M50 possibility",
    );

    let frozen7 =
        runtime7
            .cognitive_runtime()
            .cognition()
            .current_m50_epistemic_possibility(
                &state7,
                &action_one,
                policy(),
            )
            .expect(
                "frozen state7 M50 possibility",
            );

    let semantic7 =
        runtime7
            .cognitive_runtime()
            .cognition()
            .current_semantic_m50_epistemic_possibility(
                &state7,
                &action_one,
                policy(),
            )
            .expect(
                "state7 typed semantic sidecar",
            );

    assert_eq!(
        semantic7
            .m50_possibility(),
        &frozen7,
        "semantic sidecar cannot alter M50 possibility",
    );

    let derived2 =
        athlesia_autonomous_active_experimentation::
            AutonomousEmpiricalEpistemicTransferIdentity::
                derive(
                    semantic2
                        .m50_possibility(),
                    athlesia_autonomous_active_experimentation::
                        EmpiricalEpistemicTransferIdentityPolicy::
                            new(
                                512,
                            )
                            .unwrap(),
                )
                .identity()
                .expect(
                    "state2 transfer identity",
                )
                .clone();

    let derived7 =
        athlesia_autonomous_active_experimentation::
            AutonomousEmpiricalEpistemicTransferIdentity::
                derive(
                    semantic7
                        .m50_possibility(),
                    athlesia_autonomous_active_experimentation::
                        EmpiricalEpistemicTransferIdentityPolicy::
                            new(
                                512,
                            )
                            .unwrap(),
                )
                .identity()
                .expect(
                    "state7 transfer identity",
                )
                .clone();

    assert_eq!(
        derived2,
        current2,
        "semantic bridge preserves exact frozen state2 transfer identity",
    );

    assert_eq!(
        derived7,
        current7,
        "semantic bridge preserves exact frozen state7 transfer identity",
    );

    let relation2 =
        athlesia_autonomous_active_experimentation::
            AutonomousSchemaLevelTargetTransferIdentity::
                derive(
                    &historical2,
                    &derived2,
                    c3hb_policy(),
                );

    let relation7 =
        athlesia_autonomous_active_experimentation::
            AutonomousSchemaLevelTargetTransferIdentity::
                derive(
                    &historical7,
                    &derived7,
                    c3hb_policy(),
                );

    assert!(
        relation2.derived()
            && relation7.derived(),
    );

    let realization22 =
        semantic2
            .realize_at_state(
                &state2,
            )
            .expect(
                "history2/state2 semantic realization",
            );

    let realization27 =
        semantic2
            .realize_at_state(
                &state7,
            )
            .expect(
                "history2/state7 semantic realization",
            );

    let realization77 =
        semantic7
            .realize_at_state(
                &state7,
            )
            .expect(
                "history7/state7 semantic realization",
            );

    let realization72 =
        semantic7
            .realize_at_state(
                &state2,
            )
            .expect(
                "history7/state2 semantic realization",
            );

    let mut state2_context =
        0_usize;
    let mut state2_base =
        0_usize;
    let mut state7_context =
        0_usize;
    let mut state7_base =
        0_usize;

    for (
        target2,
        target7,
    ) in relation2
        .correspondences()
        .iter()
        .filter(
            |target| {
                target.match_kind()
                    == athlesia_autonomous_active_experimentation::
                        SchemaLevelTargetMatchKind::
                            RolePreserving
            },
        )
        .zip(
            relation7
                .correspondences()
                .iter()
                .filter(
                    |target| {
                        target.match_kind()
                            == athlesia_autonomous_active_experimentation::
                                SchemaLevelTargetMatchKind::
                                    RolePreserving
                    },
                ),
        )
    {
        let c10_2 =
            athlesia_autonomous_active_experimentation::
                AutonomousGroundedForecastCorrespondence::
                    derive(
                        &historical2,
                        &derived2,
                        target2,
                        athlesia_autonomous_active_experimentation::
                            GroundedForecastCorrespondencePolicy::
                                new(
                                    32,
                                    10_000,
                                    32,
                                    4096,
                                    2048,
                                    4096,
                                    256,
                                )
                                .unwrap(),
                    );

        let c10_7 =
            athlesia_autonomous_active_experimentation::
                AutonomousGroundedForecastCorrespondence::
                    derive(
                        &historical7,
                        &derived7,
                        target7,
                        athlesia_autonomous_active_experimentation::
                            GroundedForecastCorrespondencePolicy::
                                new(
                                    32,
                                    10_000,
                                    32,
                                    4096,
                                    2048,
                                    4096,
                                    256,
                                )
                                .unwrap(),
                    );

        assert!(
            c10_2.derived()
                && c10_7.derived(),
        );

        let correspondence2 =
            c10_2
                .correspondence()
                .unwrap();

        let correspondence7 =
            c10_7
                .correspondence()
                .unwrap();

        assert_eq!(
            correspondence2
                .forecast_count(),
            5,
        );

        assert_eq!(
            correspondence7
                .forecast_count(),
            5,
        );

        for entry in
            correspondence2.entries()
        {
            let empirical_forecast =
                &derived2
                    .forecasts()[
                        entry
                            .current_forecast_index()
                    ];

            let hypothesis_identity =
                empirical_forecast
                    .hypothesis();

            let mut matching22 =
                realization22
                    .forecasts()
                    .iter()
                    .filter(
                        |realization| {
                            realization
                                .hypothesis_identity()
                                == hypothesis_identity
                        },
                    );

            let realized22 =
                matching22
                    .next()
                    .expect(
                        "every C10 state2 forecast must retain exact semantic provenance",
                    );

            assert!(
                matching22
                    .next()
                    .is_none(),
                "C10 state2 forecast identity must map to exactly one semantic realization",
            );

            let mut matching27 =
                realization27
                    .forecasts()
                    .iter()
                    .filter(
                        |realization| {
                            realization
                                .hypothesis_identity()
                                == hypothesis_identity
                        },
                    );

            let realized27 =
                matching27
                    .next()
                    .expect(
                        "state2 semantic provenance must remain identifiable under state7 realization",
                    );

            assert!(
                matching27
                    .next()
                    .is_none(),
                "counterfactual state7 realization must retain unique state2 hypothesis identity",
            );

            let status22 =
                realized22
                    .prediction()
                    .status();

            let status27 =
                realized27
                    .prediction()
                    .status();

            match entry.relation() {
                athlesia_autonomous_active_experimentation::
                    GroundedForecastCorrespondenceRelation::
                        TargetAnchoredContextTransformation(
                            _,
                        ) =>
                {
                    state2_context += 1;

                    assert_eq!(
                        status22,
                        athlesia_universal_domain_learning::
                            GroundedExplanatoryPredictionStatus::
                                ContextNotSatisfied,
                    );

                    assert_eq!(
                        status27,
                        athlesia_universal_domain_learning::
                            GroundedExplanatoryPredictionStatus::
                                Predicted,
                    );
                }

                athlesia_autonomous_active_experimentation::
                    GroundedForecastCorrespondenceRelation::
                        TargetBoundHypothesisContinuation =>
                {
                    state2_base += 1;

                    assert_eq!(
                        status22,
                        athlesia_universal_domain_learning::
                            GroundedExplanatoryPredictionStatus::
                                NoEffectOpportunity,
                    );

                    assert_eq!(
                        status27,
                        athlesia_universal_domain_learning::
                            GroundedExplanatoryPredictionStatus::
                                Predicted,
                    );
                }
            }
        }

        for entry in
            correspondence7.entries()
        {
            let empirical_forecast =
                &derived7
                    .forecasts()[
                        entry
                            .current_forecast_index()
                    ];

            let hypothesis_identity =
                empirical_forecast
                    .hypothesis();

            let mut matching77 =
                realization77
                    .forecasts()
                    .iter()
                    .filter(
                        |realization| {
                            realization
                                .hypothesis_identity()
                                == hypothesis_identity
                        },
                    );

            let realized77 =
                matching77
                    .next()
                    .expect(
                        "every C10 state7 forecast must retain exact semantic provenance",
                    );

            assert!(
                matching77
                    .next()
                    .is_none(),
                "C10 state7 forecast identity must map to exactly one semantic realization",
            );

            let mut matching72 =
                realization72
                    .forecasts()
                    .iter()
                    .filter(
                        |realization| {
                            realization
                                .hypothesis_identity()
                                == hypothesis_identity
                        },
                    );

            let realized72 =
                matching72
                    .next()
                    .expect(
                        "state7 semantic provenance must remain identifiable under state2 realization",
                    );

            assert!(
                matching72
                    .next()
                    .is_none(),
                "counterfactual state2 realization must retain unique state7 hypothesis identity",
            );

            let status77 =
                realized77
                    .prediction()
                    .status();

            let status72 =
                realized72
                    .prediction()
                    .status();

            match entry.relation() {
                athlesia_autonomous_active_experimentation::
                    GroundedForecastCorrespondenceRelation::
                        TargetAnchoredContextTransformation(
                            _,
                        ) =>
                {
                    state7_context += 1;

                    assert_eq!(
                        status77,
                        athlesia_universal_domain_learning::
                            GroundedExplanatoryPredictionStatus::
                                Predicted,
                    );

                    assert_eq!(
                        status72,
                        athlesia_universal_domain_learning::
                            GroundedExplanatoryPredictionStatus::
                                ContextNotSatisfied,
                    );
                }

                athlesia_autonomous_active_experimentation::
                    GroundedForecastCorrespondenceRelation::
                        TargetBoundHypothesisContinuation =>
                {
                    state7_base += 1;

                    assert_eq!(
                        status77,
                        athlesia_universal_domain_learning::
                            GroundedExplanatoryPredictionStatus::
                                Predicted,
                    );

                    assert_eq!(
                        status72,
                        athlesia_universal_domain_learning::
                            GroundedExplanatoryPredictionStatus::
                                NoEffectOpportunity,
                    );
                }
            }
        }
    }

    assert_eq!(
        state2_context,
        16,
    );

    assert_eq!(
        state2_base,
        4,
    );

    assert_eq!(
        state7_context,
        16,
    );

    assert_eq!(
        state7_base,
        4,
    );

    println!(
        "C3HC15B_LIVE \
         M50_EXACT_PRESERVATION=1 \
         C10_STRUCTURAL_PRESERVATION=1 \
         STATE2_CONTEXT_NOT_SATISFIED=16 \
         STATE2_NO_EFFECT_OPPORTUNITY=4 \
         STATE7_PREDICTED_CONTEXT=16 \
         STATE7_PREDICTED_BASE=4 \
         COUNTERFACTUAL_CROSSOVER=1"
    );
}
