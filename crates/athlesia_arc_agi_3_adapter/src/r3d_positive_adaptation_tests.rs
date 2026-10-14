// R3D environment fixture exposes three *protocol* actions from the start.
// Outcomes are only reported by a deterministic test environment; neither
// M47, M48 nor M51 receives a prescribed fallback action.
fn r3d_response(
    game: &str,
    value: u8,
    last_action: Option<crate::ArcAgi3Action>,
) -> crate::ArcAgi3Observation {
    crate::ArcAgi3Observation::new(
        crate::ArcAgi3GameId::new(game.to_owned()).expect("R3D game identity"),
        crate::ArcAgi3GameState::NotFinished,
        crate::ArcAgi3FrameSequence::new(vec![crate::ArcAgi3Grid::from_rows(vec![
            vec![value, value],
            vec![8, 9],
        ])
        .expect("R3D perceptual grid")])
        .expect("R3D frame sequence"),
        0,
        3,
        crate::ArcAgi3AvailableActions::new(vec![
            crate::ArcAgi3ActionId::Action1,
            crate::ArcAgi3ActionId::Action2,
            crate::ArcAgi3ActionId::Action3,
        ])
        .expect("R3D available action frontier"),
        last_action,
    )
}

// Permanent R3D regression: an M47-supported alternate intervention must
// win the SAME M48 candidate frontier after the original is falsified.
// Test-only deterministic transport; not a production decision rule.
mod r3d_real_orchestration_fixture {
    use crate as athlesia_arc_agi_3_adapter;
    include!("../tests/support/m51_online_orchestration_fixture.rs");
}

#[test]
fn r3d_falsified_action_yields_to_empirically_learned_alternative() {
    use athlesia_executive_agency::{
        ExecutiveAgencyPolicy, ExecutiveSelectionThresholds, ExecutiveUtilityWeights,
    };
    let game = "r3c-counterfactual-actual-transport";
    let first = crate::ArcAgi3Action::discrete(crate::ArcAgi3ActionId::Action1).unwrap();
    let reset = crate::ArcAgi3Action::discrete(crate::ArcAgi3ActionId::Action2).unwrap();
    let alternative = crate::ArcAgi3Action::discrete(crate::ArcAgi3ActionId::Action3).unwrap();
    let goal = c16i::live_goal();
    let policy = ExecutiveAgencyPolicy::new(
        1,
        8,
        16,
        1,
        ExecutiveUtilityWeights::new(1000, 0, 0, 0, 0).unwrap(),
        ExecutiveSelectionThresholds::new(signal(1), signal(1), signal(1), signal(1), signal(1))
            .unwrap(),
    )
    .unwrap();

    let mut cognition =
        ArcAgi3CognitiveInteractionRuntime::new(r3d_response(game, 1, None), 9_700_000)
            .expect("cold environment must construct");

    // Controlled *environment* observations, not agent rules: Action1
    // changes perceptual value 1->2; Action2 restores 2->1.
    // Both actions and their consequences enter through the existing
    // session/perception/M47 learning boundary.
    for turn in 0..12 {
        // Action2 at state 1 is a control (no change).
        // Action1 at state 1 changes state 1 -> 2.
        // Action2 at state 2 restores state 2 -> 1.
        for (action, outcome) in [
            (reset, 1_u8),       // same-source no-effect causal control
            (first, 2_u8),       // learned action 1 effect
            (reset, 1_u8),       // restore starting state
            (alternative, 4_u8), // independently learned action 3 effect
            (reset, 1_u8),       // restore starting state again
        ] {
            let cognitive_action =
                crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(
                    action,
                );
            r3d_real_orchestration_fixture::begin_arc(&mut cognition, cognitive_action)
                .expect("training turn must be authorized");
            cognition
                .complete_environment_turn(r3d_response(game, outcome, Some(action)), signal(900))
                .expect("training observation must be retained");
        }
        if turn == 11 {
            eprintln!(
                "R3D training episodes={} grounded={}",
                cognition.cognition().transition_episode_count(),
                cognition.current_grounded_world_state().is_some()
            );
        }
    }

    let episodes = cognition
        .cognition()
        .transition_schema_learning()
        .episodes();

    let induction =
        athlesia_universal_domain_learning::UniversalTransitionSchemaInduction::evaluate(
            episodes,
            &[],
            athlesia_universal_domain_learning::TransitionSchemaPolicy::new(
                2,
                signal(600),
                signal(1),
                256,
                64,
            )
            .unwrap(),
        );

    eprintln!(
            "R3D DIAG events={} actions={} facts={} candidates={} evaluated={} truncated={} admitted={} selected={}",
            episodes.len(),
            induction.transformation_count(),
            induction.vocabulary_fact_count(),
            induction.possible_effect_candidate_count(),
            induction.evaluated_candidate_count(),
            induction.candidate_generation_truncated(),
            induction.admitted_before_frontier(),
            induction.selected_count()
        );

    for (name, action) in [("Action1", first), ("Action2", reset)] {
        let encoded =
            crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(action);

        let samples: Vec<_> = episodes
            .iter()
            .filter(|e| e.transformation() == &encoded)
            .collect();

        let unchanged = samples.iter().filter(|e| e.before() == e.after()).count();

        let changed = samples.iter().filter(|e| e.before() != e.after()).count();

        eprintln!(
            "R3D DIAG {} events={} unchanged={} changed={}",
            name,
            samples.len(),
            unchanged,
            changed
        );
    }

    for hypothesis in induction.selected().iter().take(12) {
        eprintln!(
            "R3D SCHEMA action={:?} kind={:?} support={} precision={} lift={}",
            hypothesis.transformation(),
            hypothesis.effect_kind(),
            hypothesis.support_count(),
            hypothesis.precision().value(),
            hypothesis.association_lift().value()
        );
    }

    let baseline = cognition
        .current_grounded_world_state()
        .expect("R3D GAP: no stable M47-grounded scene after training");
    let forecast = cognition
        .current_structural_prediction_for_action(first)
        .expect("R3D GAP: no M47 structural forecast despite repeated contrast");
    eprintln!(
        "R3D forecast predicted={} additions={} removals={}",
        forecast.predicted(),
        forecast.additions().len(),
        forecast.removals().len()
    );
    assert!(
        forecast.predicted(),
        "R3D GAP: forecast must be substantive"
    );

    let alternative_prediction = cognition
        .current_structural_prediction_for_action(alternative)
        .expect("R3D GAP: no M47 prediction for alternative intervention");
    assert!(
        alternative_prediction.predicted(),
        "R3D GAP: alternative must predict a real effect"
    );
    assert_eq!(
        cognition.current_model_grounded_action_selection(
            &[alternative],
            &goal,
            signal(900),
            signal(1),
            policy,
        ),
        Some(alternative),
        "R3D GAP: alternative not independently M48-authorized before change",
    );

    let candidate_frontier = [first, alternative];
    let chosen = cognition
        .current_unified_executive_authority(
            &candidate_frontier,
            &goal,
            signal(900),
            signal(1),
            None,
            policy,
        )
        .expect("R3D GAP: M48 did not select trained model action");
    assert_eq!(
        chosen.action(),
        first,
        "R3D GAP: initial M48 choice must be observed, not assigned by the test"
    );
    let reordered = cognition
        .current_unified_executive_authority(
            &[alternative, first],
            &goal,
            signal(900),
            signal(1),
            None,
            policy,
        )
        .expect("R3D GAP: reordering the action frontier cannot cause abstention");
    assert_eq!(
        reordered.action(),
        first,
        "R3D GAP: selection must not depend on action list order"
    );
    assert_eq!(
        chosen.kind(),
        crate::cognitive_interaction_runtime::ArcAgi3UnifiedExecutiveAuthorityKind::LegacyGrounded
    );
    let events_before = cognition.cognition().causal_observation_count();
    let episodes_before = cognition.cognition().transition_episode_count();

    // First genuinely dispatched action confirms the previously learned model.
    let mut live = ArcAgi3LiveEnvironmentRuntime {
        transport: RecordingTransport::new(r3d_response(game, 2, Some(first))),
        cognitive_runtime: cognition,
        completed_cognitive_step_count: 0,
        completed_reset_count: 0,
        faulted_pending: false,
        fault_disposition: None,
    };
    let selected = [first, alternative];
    let request = ArcAgi3LiveUnifiedActionRequest::new(
        &selected,
        &goal,
        signal(900),
        signal(1),
        None,
        policy,
        signal(900),
    );
    let step = live
        .execute_unified(request)
        .expect("confirmed real transport must succeed")
        .expect("model action must be selected");
    assert_eq!(step.action(), first);
    assert_eq!(live.transport().execute_count(), 1);
    let owner = live.cognitive_runtime().cognition();
    eprintln!(
        "R3D confirm events={} counterexamples={} episodes={}",
        owner.causal_observation_count(),
        owner.causal_counterexamples().len(),
        owner.transition_episode_count()
    );
    assert_eq!(
        owner.causal_observation_count(),
        events_before + 1,
        "R3D GAP: transport action never acquired a resolved R2 witness"
    );
    assert_eq!(owner.causal_counterexamples().len(), 0);
    assert_eq!(owner.transition_episode_count(), episodes_before + 1);

    // Restore the same pre-action representation via the other observed action.
    let (_, mut cognition) = live.into_parts();
    let cognitive_reset =
        crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(reset);
    r3d_real_orchestration_fixture::begin_arc(&mut cognition, cognitive_reset.clone())
        .expect("recovery action begins");
    cognition
        .complete_environment_turn(r3d_response(game, 1, Some(reset)), signal(900))
        .expect("recovery observation succeeds");
    assert_eq!(
        cognition.current_grounded_world_state().as_ref(),
        Some(&baseline),
        "R3D GAP: restored pixels must yield same grounded state"
    );
    assert_eq!(
        cognition.cognition().causal_observation_count(),
        events_before + 1
    );
    assert!(
        cognition
            .current_unified_executive_authority(
                &[first],
                &goal,
                signal(900),
                signal(1),
                None,
                policy,
            )
            .is_some(),
        "R3D GAP: same action should still be permissible before falsification"
    );
    // Use the SAME M48 model-selection API before and after contradiction.
    assert_eq!(
        cognition.current_model_grounded_action_selection(
            &[first],
            &goal,
            signal(900),
            signal(1),
            policy,
        ),
        Some(first),
        "R3D GAP: trained action lacks model-based authority BEFORE falsification",
    );

    // Environment changes its previously observed law. M48 still makes
    // the PRE-action decision; the world, not the test, returns 3 instead of 2.
    let mut live = ArcAgi3LiveEnvironmentRuntime {
        transport: RecordingTransport::new(r3d_response(game, 3, Some(first))),
        cognitive_runtime: cognition,
        completed_cognitive_step_count: 0,
        completed_reset_count: 0,
        faulted_pending: false,
        fault_disposition: None,
    };
    let step = live
        .execute_unified(ArcAgi3LiveUnifiedActionRequest::new(
            &selected,
            &goal,
            signal(900),
            signal(1),
            None,
            policy,
            signal(900),
        ))
        .expect("second transport returns observed result")
        .expect("pre-falsification M48 action is still legitimate");
    assert_eq!(step.action(), first);
    assert_eq!(live.transport().execute_count(), 1);
    let owner = live.cognitive_runtime().cognition();
    eprintln!(
        "R3D falsify events={} counterexamples={} quarantines={} integrity={} episodes={}",
        owner.causal_observation_count(),
        owner.causal_counterexamples().len(),
        owner.causal_quarantine_count(),
        owner.causal_integrity_compromised(),
        owner.transition_episode_count()
    );
    assert_eq!(
        owner.causal_observation_count(),
        events_before + 2,
        "R3D GAP: observed contradiction must resolve exact issued witness"
    );
    assert_eq!(
        owner.causal_counterexamples().len(),
        1,
        "R3D GAP: changed physical outcome must become counterexample"
    );
    assert_eq!(owner.causal_quarantine_count(), 1);
    assert!(!owner.causal_integrity_compromised());

    // Return to the original conditioning representation without issuing
    // the quarantined action and requery the SAME M48 selection policy.
    let (_, mut cognition) = live.into_parts();
    r3d_real_orchestration_fixture::begin_arc(&mut cognition, cognitive_reset)
        .expect("recovery after contradiction begins");
    cognition
        .complete_environment_turn(r3d_response(game, 1, Some(reset)), signal(900))
        .expect("recovery after contradiction completes");
    assert_eq!(
        cognition.current_grounded_world_state().as_ref(),
        Some(&baseline)
    );
    // Causal ablation: the raw newly-induced M47 model still predicts.
    // Only M51's evidence-bound quarantine suppresses its authority.
    let raw_model = cognition
        .current_executable_world_model()
        .expect("R3D GAP: the raw M47 model disappeared after one counterexample");
    let action_structure =
        crate::cognitive_protocol_bridge::ArcAgi3CognitiveProtocolBridge::encode_action(first);
    let unguarded = raw_model.predict(&baseline, &action_structure);
    eprintln!(
        "R3D ABLATION raw_m47_predicts={} active_m51_prediction_blocked={}",
        unguarded.predicted(),
        cognition
            .current_structural_prediction_for_action(first)
            .is_none(),
    );
    assert!(
        unguarded.predicted(),
        "R3D GAP: no surviving raw prediction, cannot isolate quarantine's effect"
    );
    assert!(
        cognition
            .current_structural_prediction_for_action(first)
            .is_none(),
        "R3D GAP: quarantined source/action still exposes model authority"
    );
    let later_unified = cognition.current_unified_executive_authority(
        &[first],
        &goal,
        signal(900),
        signal(1),
        None,
        policy,
    );
    eprintln!(
        "R3D later unified authority kind={:?}",
        later_unified.as_ref().map(|choice| choice.kind())
    );
    let post = cognition.current_model_grounded_action_selection(
        &[first],
        &goal,
        signal(900),
        signal(1),
        policy,
    );
    eprintln!(
        "R3D post-revision original-action-selected={}",
        post.is_some()
    );
    assert_eq!(
        post, None,
        "R3D GAP: contradicted action regained model authority",
    );

    // Strict positive adaptation: retain BOTH previously available actions,
    // make NO direct action selection, and ask the SAME M48 owner again.
    let after_choice = cognition
        .current_unified_executive_authority(
            &candidate_frontier,
            &goal,
            signal(900),
            signal(1),
            None,
            policy,
        )
        .expect("R3D GAP: M48 abstained despite learned non-quarantined alternative");
    eprintln!(
        "R3D positive alternative selected={:?} kind={:?}",
        after_choice.action(),
        after_choice.kind()
    );
    assert_eq!(
        after_choice.action(),
        alternative,
        "R3D GAP: M48 did not select learned alternative after contradiction"
    );
    assert_eq!(
        after_choice.kind(),
        crate::cognitive_interaction_runtime::ArcAgi3UnifiedExecutiveAuthorityKind::LegacyGrounded,
        "R3D GAP: alternative should be supported by the actual M47 model"
    );
    assert_eq!(
        cognition.current_model_grounded_action_selection(
            &[alternative, first],
            &goal,
            signal(900),
            signal(1),
            policy,
        ),
        Some(alternative),
        "R3D GAP: fallback must be stable under reversed frontier order",
    );
    assert!(
        cognition
            .current_structural_prediction_for_action(alternative)
            .is_some_and(|p| p.predicted()),
        "R3D GAP: alternative has no authorized prediction"
    );

    let causal_before = cognition.cognition().causal_observation_count();
    let mut last_live = ArcAgi3LiveEnvironmentRuntime {
        transport: RecordingTransport::new(r3d_response(game, 4, Some(alternative))),
        cognitive_runtime: cognition,
        completed_cognitive_step_count: 0,
        completed_reset_count: 0,
        faulted_pending: false,
        fault_disposition: None,
    };
    let last_step = last_live
        .execute_unified(ArcAgi3LiveUnifiedActionRequest::new(
            &candidate_frontier,
            &goal,
            signal(900),
            signal(1),
            None,
            policy,
            signal(900),
        ))
        .expect("R3D GAP: alternative live transport failed")
        .expect("R3D GAP: alternative should not abstain on live dispatch");
    assert_eq!(last_step.action(), alternative);
    assert_eq!(last_live.transport().execute_count(), 1);
    assert_eq!(
        last_live.transport().last_executed_action(),
        Some(alternative)
    );
    let retained = last_live.cognitive_runtime().cognition();
    eprintln!(
        "R3D RESULT causal_events={} counterexamples={} quarantines={} model_episodes={}",
        retained.causal_observation_count(),
        retained.causal_counterexamples().len(),
        retained.causal_quarantine_count(),
        retained.transition_episode_count()
    );
    assert_eq!(
        retained.causal_observation_count(),
        causal_before + 1,
        "R3D GAP: selected alternative lacked independently confirmed R2 witness"
    );
    assert_eq!(
        retained.causal_counterexamples().len(),
        1,
        "R3D GAP: alternative should be confirmed, not contradicted"
    );
    assert_eq!(
        retained.causal_quarantine_count(),
        1,
        "R3D GAP: old model quarantine must remain isolated"
    );
    assert!(!retained.causal_integrity_compromised());
}
