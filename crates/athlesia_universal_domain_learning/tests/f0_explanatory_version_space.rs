use athlesia_mindstone_sparse_cognition::CognitiveStructure;
use athlesia_universal_domain_learning::{
    ContextPremiseSet, GroundedExplanatoryEpisodeAssessment, GroundedExplanatoryVersionSpacePolicy,
    GroundedExplanatoryVersionSpaceSynthesis, GroundedStateSnapshot, GroundedTransformationEpisode,
    TransitionEffectKind,
};

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn state(facts: &[u64]) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts.iter().copied().map(atom).collect())
        .expect("state must be nonempty")
}

fn policy() -> GroundedExplanatoryVersionSpacePolicy {
    GroundedExplanatoryVersionSpacePolicy::new(1, 16, 64, 32).expect("valid version-space policy")
}

#[test]
fn ambiguity_is_preserved_then_discriminating_evidence_resolves_it() {
    let action = atom(10);
    let effect = atom(9);
    let x = atom(1);

    let first = GroundedTransformationEpisode::new(state(&[1]), state(&[1, 9]), action.clone());

    let initial_space = GroundedExplanatoryVersionSpaceSynthesis::synthesize(
        std::slice::from_ref(&first),
        policy(),
    );

    let x_context = ContextPremiseSet::new(vec![x]).expect("grounded context");

    let unconditional = initial_space
        .active()
        .iter()
        .find(|hypothesis| {
            hypothesis.context().is_none()
                && hypothesis.transformation() == &action
                && hypothesis.effect_kind() == TransitionEffectKind::Added
                && hypothesis.effect_fact() == &effect
        })
        .expect("general explanation must remain live");

    let contextual = initial_space
        .active()
        .iter()
        .find(|hypothesis| {
            hypothesis.context() == Some(&x_context)
                && hypothesis.transformation() == &action
                && hypothesis.effect_kind() == TransitionEffectKind::Added
                && hypothesis.effect_fact() == &effect
        })
        .expect("contextual explanation must remain live");

    assert!(
        initial_space.active_count() >= 2,
        "one observation must not collapse explanation uncertainty",
    );

    let discriminating =
        GroundedTransformationEpisode::new(state(&[2]), state(&[2]), action.clone());

    assert_eq!(
        unconditional.assess(&discriminating),
        GroundedExplanatoryEpisodeAssessment::Counterexample,
    );

    assert_eq!(
        contextual.assess(&discriminating),
        GroundedExplanatoryEpisodeAssessment::NotApplicable,
    );

    let resolved_space =
        GroundedExplanatoryVersionSpaceSynthesis::synthesize(&[first, discriminating], policy());

    assert!(
        !resolved_space.active().iter().any(|hypothesis| {
            hypothesis.context().is_none()
                && hypothesis.transformation() == &action
                && hypothesis.effect_kind() == TransitionEffectKind::Added
                && hypothesis.effect_fact() == &effect
        }),
        "directly contradicted general explanation must leave active version space",
    );

    assert!(
        resolved_space.active().iter().any(|hypothesis| {
            hypothesis.context() == Some(&x_context)
                && hypothesis.transformation() == &action
                && hypothesis.effect_kind() == TransitionEffectKind::Added
                && hypothesis.effect_fact() == &effect
        }),
        "non-applicable evidence must not erase the contextual explanation",
    );
}

#[test]
fn same_explanatory_hypothesis_is_directly_executable() {
    use athlesia_universal_domain_learning::GroundedExplanatoryPredictionStatus;

    let action = atom(10);
    let effect = atom(9);

    let episode = GroundedTransformationEpisode::new(state(&[1]), state(&[1, 9]), action.clone());

    let version_space = GroundedExplanatoryVersionSpaceSynthesis::synthesize(&[episode], policy());

    let x_context = ContextPremiseSet::new(vec![atom(1)]).expect("grounded context");

    let unconditional = version_space
        .active()
        .iter()
        .find(|hypothesis| hypothesis.context().is_none() && hypothesis.effect_fact() == &effect)
        .expect("unconditional explanation");

    let contextual = version_space
        .active()
        .iter()
        .find(|hypothesis| {
            hypothesis.context() == Some(&x_context) && hypothesis.effect_fact() == &effect
        })
        .expect("contextual explanation");

    let contextual_here = contextual.predict(&state(&[1]), &action);

    assert_eq!(
        contextual_here.status(),
        GroundedExplanatoryPredictionStatus::Predicted,
    );
    assert!(contextual_here.predicts_addition(&effect));

    let contextual_elsewhere = contextual.predict(&state(&[2]), &action);

    assert_eq!(
        contextual_elsewhere.status(),
        GroundedExplanatoryPredictionStatus::ContextNotSatisfied,
    );

    let general_elsewhere = unconditional.predict(&state(&[2]), &action);

    assert_eq!(
        general_elsewhere.status(),
        GroundedExplanatoryPredictionStatus::Predicted,
    );
    assert!(general_elsewhere.predicts_addition(&effect));

    let already_present = contextual.predict(&state(&[1, 9]), &action);

    assert_eq!(
        already_present.status(),
        GroundedExplanatoryPredictionStatus::NoEffectOpportunity,
    );

    let wrong_action = contextual.predict(&state(&[1]), &atom(20));

    assert_eq!(
        wrong_action.status(),
        GroundedExplanatoryPredictionStatus::IrrelevantTransformation,
    );
}

#[test]
fn factorized_uncertainty_selects_the_action_that_can_distinguish_live_explanations() {
    use athlesia_universal_domain_learning::{
        GroundedFactorizedActionDiscriminationEngine, GroundedFactorizedDiscriminationPolicy,
    };

    let action_a = atom(10);
    let action_b = atom(20);
    let effect = atom(9);

    let experience =
        GroundedTransformationEpisode::new(state(&[1]), state(&[1, 9]), action_a.clone());

    let version_space =
        GroundedExplanatoryVersionSpaceSynthesis::synthesize(&[experience], policy());

    let current = state(&[2]);

    let candidates = vec![action_b.clone(), action_a.clone()];

    let discrimination = GroundedFactorizedActionDiscriminationEngine::evaluate(
        &current,
        &candidates,
        &version_space,
        GroundedFactorizedDiscriminationPolicy::new(8).expect("valid action bound"),
    );

    let best = discrimination
        .best_informative()
        .expect("one action must discriminate the live explanations");

    assert_eq!(
        best.transformation(),
        &action_a,
        "the known action must be selected because its live explanations diverge here",
    );

    assert!(
        best.pairwise_separation_score() > 0,
        "selected action must have real predictive separation",
    );

    assert_eq!(best.disputed_effect_count(), 1);

    let disagreement = &best.disagreements()[0];

    assert_eq!(disagreement.effect_kind(), TransitionEffectKind::Added,);

    assert_eq!(disagreement.effect_fact(), &effect);

    assert!(
        disagreement.predicted_count() > 0,
        "at least one live explanation must predict the effect",
    );

    assert!(
        disagreement.context_abstention_count() > 0,
        "at least one competing explanation must remain silent because its context is absent",
    );

    let noninformative = discrimination
        .ranked()
        .iter()
        .find(|candidate| candidate.transformation() == &action_b)
        .expect("all supplied bounded actions remain represented");

    assert!(!noninformative.informative());
}

#[test]
fn autonomous_discriminating_experiment_reduces_explanatory_uncertainty() {
    use athlesia_universal_domain_learning::{
        GroundedFactorizedActionDiscriminationEngine, GroundedFactorizedDiscriminationPolicy,
    };

    let action_a = atom(10);
    let action_b = atom(20);
    let effect = atom(9);

    // Hidden world:
    //
    //     context atom(1) AND action_a -> add effect
    //
    // The learner is never given this rule.

    let initial_experience =
        GroundedTransformationEpisode::new(state(&[1]), state(&[1, 9]), action_a.clone());

    let mut evidence = vec![initial_experience];

    let initial_space = GroundedExplanatoryVersionSpaceSynthesis::synthesize(&evidence, policy());

    let x_context = ContextPremiseSet::new(vec![atom(1)]).expect("grounded context");

    assert!(
        initial_space.active().iter().any(|hypothesis| {
            hypothesis.context().is_none()
                && hypothesis.transformation() == &action_a
                && hypothesis.effect_fact() == &effect
        }),
        "general explanation must initially remain possible",
    );

    assert!(
        initial_space.active().iter().any(|hypothesis| {
            hypothesis.context() == Some(&x_context)
                && hypothesis.transformation() == &action_a
                && hypothesis.effect_fact() == &effect
        }),
        "contextual explanation must initially remain possible",
    );

    let current_state = state(&[2]);

    let discrimination = GroundedFactorizedActionDiscriminationEngine::evaluate(
        &current_state,
        &[action_b.clone(), action_a.clone()],
        &initial_space,
        GroundedFactorizedDiscriminationPolicy::new(8).expect("valid action bound"),
    );

    let chosen_action = discrimination
        .best_informative()
        .expect("learner must find a discriminating experiment")
        .transformation()
        .clone();

    assert_eq!(
        chosen_action, action_a,
        "epistemic disagreement must autonomously select action_a",
    );

    // Environment execution.
    // The hidden contextual condition is absent, so no effect occurs.
    let observed_after = if current_state.contains_fact(&atom(1)) && chosen_action == action_a {
        state(&[2, 9])
    } else {
        state(&[2])
    };

    evidence.push(GroundedTransformationEpisode::new(
        current_state.clone(),
        observed_after,
        chosen_action,
    ));

    let revised_space = GroundedExplanatoryVersionSpaceSynthesis::synthesize(&evidence, policy());

    assert!(
        !revised_space.active().iter().any(|hypothesis| {
            hypothesis.context().is_none()
                && hypothesis.transformation() == &action_a
                && hypothesis.effect_fact() == &effect
        }),
        "direct experimental counterevidence must eliminate the general explanation",
    );

    assert!(
        revised_space.active().iter().any(|hypothesis| {
            hypothesis.context() == Some(&x_context)
                && hypothesis.transformation() == &action_a
                && hypothesis.effect_fact() == &effect
        }),
        "the contextual explanation must survive because its premise was absent",
    );

    let after_revision = GroundedFactorizedActionDiscriminationEngine::evaluate(
        &current_state,
        &[action_a.clone(), action_b],
        &revised_space,
        GroundedFactorizedDiscriminationPolicy::new(8).expect("valid action bound"),
    );

    assert!(
        after_revision.best_informative().is_none(),
        "resolved local ambiguity must no longer demand another discriminating experiment",
    );
}
