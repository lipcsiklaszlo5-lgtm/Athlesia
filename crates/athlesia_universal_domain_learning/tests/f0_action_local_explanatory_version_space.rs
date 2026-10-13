use athlesia_mindstone_sparse_cognition::CognitiveStructure;
use athlesia_universal_domain_learning::{
    ContextPremiseSet, GroundedExplanatoryEpisodeAssessment, GroundedExplanatoryPredictionStatus,
    GroundedExplanatoryVersionSpace, GroundedExplanatoryVersionSpacePolicy,
    GroundedExplanatoryVersionSpaceSynthesis as Synthesis, GroundedStateSnapshot,
    GroundedTransformationEpisode, TransitionEffectKind,
};

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn state(facts: &[u64]) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts.iter().copied().map(atom).collect()).unwrap()
}

fn episode(
    before: &[u64],
    after: &[u64],
    action: &CognitiveStructure,
) -> GroundedTransformationEpisode {
    GroundedTransformationEpisode::new(state(before), state(after), action.clone())
}

fn experience(action: &CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
    vec![
        episode(&[1], &[1, 9], action),
        episode(&[2], &[2, 9], action),
    ]
}

fn unrelated_experience() -> Vec<GroundedTransformationEpisode> {
    (0..16)
        .map(|index| {
            episode(
                &[10 + index],
                &[10 + index, 1_000 + index],
                &atom(200 + index),
            )
        })
        .collect()
}

fn truncation(space: &GroundedExplanatoryVersionSpace) -> (bool, bool, bool) {
    (
        space.context_frontier_truncated(),
        space.evaluation_truncated(),
        space.frontier_truncated(),
    )
}

#[test]
fn unrelated_transformations_cannot_consume_any_local_synthesis_frontier() {
    let action = atom(100);
    let policy = GroundedExplanatoryVersionSpacePolicy::new(1, 2, 3, 3).unwrap();
    let local_episodes = experience(&action);
    let mut episodes = local_episodes.clone();
    episodes.extend(unrelated_experience());
    let global = Synthesis::synthesize(&episodes, policy);
    assert!(global.context_frontier_truncated());
    assert!(global.evaluation_truncated());

    let local = Synthesis::synthesize_for_transformation(&episodes, &action, policy);
    assert_eq!(truncation(&local), (false, false, false));
    assert_eq!(local, Synthesis::synthesize(&local_episodes, policy));
    assert_eq!(local.possible_context_count(), 2);
    assert_eq!(local.generated_context_count(), 2);
    assert_eq!(local.effect_target_count(), 1);
    assert_eq!(local.possible_hypothesis_evaluation_count(), 3);

    // One unconditional explanation and the two independently observed
    // contexts remain unresolved; counts follow the supplied episodes.
    assert_eq!(local.active_count(), 3);
    for (context, support) in [(None, 2), (Some(1), 1), (Some(2), 1)] {
        let context = context.map(|fact| ContextPremiseSet::new(vec![atom(fact)]).unwrap());
        let hypothesis = local
            .active()
            .iter()
            .find(|hypothesis| hypothesis.context() == context.as_ref())
            .unwrap();
        assert_eq!(hypothesis.transformation(), &action);
        assert_eq!(hypothesis.effect_kind(), TransitionEffectKind::Added);
        assert_eq!(hypothesis.effect_fact(), &atom(9));
        assert_eq!(hypothesis.support_count(), support);
        assert_eq!(hypothesis.opportunity_count(), support);
        assert_eq!(hypothesis.counterexample_count(), 0);
    }

    episodes.reverse();
    assert_eq!(
        local,
        Synthesis::synthesize_for_transformation(&episodes, &action, policy)
    );
}

#[test]
fn each_action_local_resource_limit_reports_its_own_incompleteness() {
    let action = atom(100);
    let local_episodes = experience(&action);
    let mut episodes = local_episodes.clone();
    episodes.extend(unrelated_experience());
    for (policy, expected) in [
        (
            GroundedExplanatoryVersionSpacePolicy::new(1, 1, 8, 8).unwrap(),
            (true, false, false),
        ),
        (
            GroundedExplanatoryVersionSpacePolicy::new(1, 8, 2, 8).unwrap(),
            (false, true, false),
        ),
        (
            GroundedExplanatoryVersionSpacePolicy::new(1, 8, 8, 2).unwrap(),
            (false, false, true),
        ),
    ] {
        let local = Synthesis::synthesize_for_transformation(&episodes, &action, policy);
        assert_eq!(truncation(&local), expected);
        assert_eq!(
            local,
            Synthesis::synthesize_for_transformation(&local_episodes, &action, policy),
            "removing unrelated actions cannot complete an incomplete local space"
        );
    }
}

#[test]
fn local_counterexamples_specialize_without_losing_abstention_or_effect_opportunity_semantics() {
    let action = atom(100);
    let policy = GroundedExplanatoryVersionSpacePolicy::new(1, 8, 2, 8).unwrap();
    let supporting = episode(&[1], &[1, 9], &action);
    let no_opportunity = episode(&[1, 9], &[1, 9], &action);
    let counterexample = episode(&[2], &[2], &action);
    let mut episodes = vec![supporting, no_opportunity.clone()];
    let initial = Synthesis::synthesize_for_transformation(&episodes, &action, policy);
    assert_eq!(initial.active_count(), 2);
    let broad = initial
        .active()
        .iter()
        .find(|hypothesis| hypothesis.context().is_none())
        .unwrap();
    assert_eq!(
        broad.assess(&counterexample),
        GroundedExplanatoryEpisodeAssessment::Counterexample
    );

    episodes.push(counterexample.clone());
    episodes.extend(unrelated_experience());
    let local = Synthesis::synthesize_for_transformation(&episodes, &action, policy);
    assert_eq!(truncation(&local), (false, false, false));
    assert_eq!(local.possible_context_count(), 3);
    assert_eq!(
        local.possible_hypothesis_evaluation_count(),
        2,
        "contexts never witnessed supporting this effect must not consume evaluations"
    );
    assert_eq!(local.active_count(), 1);
    let specialized = &local.active()[0];
    assert_eq!(
        specialized.context(),
        Some(&ContextPremiseSet::new(vec![atom(1)]).unwrap())
    );
    assert_eq!(specialized.support_count(), 1);
    assert_eq!(specialized.opportunity_count(), 1);
    assert_eq!(specialized.counterexample_count(), 0);
    assert_eq!(
        specialized.assess(&counterexample),
        GroundedExplanatoryEpisodeAssessment::NotApplicable
    );
    assert_eq!(
        specialized.assess(&no_opportunity),
        GroundedExplanatoryEpisodeAssessment::NoEffectOpportunity
    );
    assert_eq!(
        specialized.predict(&state(&[2]), &action).status(),
        GroundedExplanatoryPredictionStatus::ContextNotSatisfied
    );
    assert_eq!(
        specialized.predict(&state(&[1, 9]), &action).status(),
        GroundedExplanatoryPredictionStatus::NoEffectOpportunity
    );
}

#[test]
fn exact_structured_action_identity_separates_identical_state_transitions() {
    let first = CognitiveStructure::Ordered(vec![atom(10), atom(20)]);
    let second = CognitiveStructure::Ordered(vec![atom(20), atom(10)]);
    let policy = GroundedExplanatoryVersionSpacePolicy::new(1, 1, 2, 2).unwrap();
    let episodes = vec![
        episode(&[1], &[1, 9], &first),
        episode(&[1], &[1, 9], &second),
        episode(&[1], &[1, 9], &second),
    ];
    for (action, other, support) in [(&first, &second, 1), (&second, &first, 2)] {
        let local = Synthesis::synthesize_for_transformation(&episodes, action, policy);
        assert_eq!(truncation(&local), (false, false, false));
        assert_eq!(local.active_count(), 2);
        for hypothesis in local.active() {
            assert_eq!(hypothesis.transformation(), action);
            assert_eq!(hypothesis.support_count(), support);
            assert_eq!(
                hypothesis.predict(&state(&[1]), other).status(),
                GroundedExplanatoryPredictionStatus::IrrelevantTransformation
            );
        }
    }
    let unseen = CognitiveStructure::Unordered(vec![atom(10), atom(20)]);
    let absent = Synthesis::synthesize_for_transformation(&episodes, &unseen, policy);
    assert_eq!(absent, Synthesis::synthesize(&[], policy));
    assert_eq!(truncation(&absent), (false, false, false));
}
