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
