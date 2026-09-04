use athlesia_mindstone_sparse_cognition::CognitiveStructure;
use athlesia_universal_domain_learning::{
    GroundedExplanatoryVersionSpacePolicy, GroundedExplanatoryVersionSpaceSynthesis,
    GroundedRelevanceSpecialization, GroundedRelevanceSpecializationPolicy, GroundedStateSnapshot,
    GroundedTransformationEpisode, TransitionEffectKind,
};

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn state(facts: &[u64]) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts.iter().copied().map(atom).collect())
        .expect("grounded state must be nonempty")
}

fn episode(before: &[u64], after: &[u64]) -> GroundedTransformationEpisode {
    GroundedTransformationEpisode::new(state(before), state(after), atom(10))
}

#[test]
fn prediction_error_discovers_minimal_relevant_distinction_under_hard_budgets() {
    let effect = atom(900);
    let relevant = atom(99);

    // Hidden rule:
    //
    //     fact 99 AND action 10 -> add fact 900
    //
    // Facts 1..7 are distractors.
    // The learner is never given the hidden rule.

    let seed = episode(&[1, 2, 3, 99], &[1, 2, 3, 99, 900]);

    let seed_space = GroundedExplanatoryVersionSpaceSynthesis::synthesize(
        std::slice::from_ref(&seed),
        GroundedExplanatoryVersionSpacePolicy::new(1, 16, 64, 32)
            .expect("valid explanatory policy"),
    );

    let general = seed_space
        .active()
        .iter()
        .find(|hypothesis| {
            hypothesis.context().is_none()
                && hypothesis.transformation() == &atom(10)
                && hypothesis.effect_kind() == TransitionEffectKind::Added
                && hypothesis.effect_fact() == &effect
        })
        .expect("initial experience must permit a general explanation")
        .clone();

    let evidence = vec![
        seed,
        episode(&[1, 4, 5, 99], &[1, 4, 5, 99, 900]),
        episode(&[2, 5, 6, 99], &[2, 5, 6, 99, 900]),
        episode(&[1, 2, 3], &[1, 2, 3]),
        episode(&[1, 4, 6], &[1, 4, 6]),
        episode(&[2, 5, 7], &[2, 5, 7]),
    ];

    let policy = GroundedRelevanceSpecializationPolicy::new(
        4, // less than full evidence history
        2, // only two candidate facts may be fully evaluated
        1, // only one specialization may survive
    )
    .expect("valid relevance policy");

    let forward = GroundedRelevanceSpecialization::discover(&general, &evidence, policy);

    let mut reversed = evidence.clone();
    reversed.reverse();

    let backward = GroundedRelevanceSpecialization::discover(&general, &reversed, policy);

    assert_eq!(
        forward, backward,
        "relevance discovery must be invariant to evidence input order",
    );

    assert_eq!(
        forward.considered_evidence_count(),
        4,
        "hard evidence budget must be respected",
    );

    assert!(
        forward.candidate_fact_count() > 2,
        "the world must contain more possible facts than the evaluation budget",
    );

    assert_eq!(
        forward.evaluated_candidate_fact_count(),
        2,
        "candidate evaluation frontier must respect its hard bound",
    );

    assert!(
        forward.candidate_generation_truncated(),
        "the challenge must genuinely exercise bounded relevance selection",
    );

    assert_eq!(
        forward.selected_count(),
        1,
        "only one minimal specialization may survive this policy",
    );

    let chosen = &forward.selected()[0];

    assert_eq!(
        chosen.added_premise(),
        &relevant,
        "prediction-error contrast must recover fact 99 despite lower-ID distractors",
    );

    assert_eq!(
        chosen.specialized_context().premise_count(),
        1,
        "one prediction error may add exactly one premise, not enumerate conjunctions",
    );

    assert_eq!(
        chosen.specialized_context().premises(),
        &[relevant],
        "the resulting representation must contain only the grounded relevant distinction",
    );

    assert!(
        chosen.retained_support_count() > 0,
        "specialization must preserve real successful evidence",
    );

    assert!(
        chosen.excluded_counterexample_count() > 0,
        "specialization must explain away real prediction failures",
    );

    assert_eq!(
        chosen.retained_counterexample_count(),
        0,
        "the selected distinction should exclude every considered counterexample",
    );
}

#[test]
fn resolved_relevance_specialization_becomes_the_same_executable_hypothesis_language() {
    use athlesia_universal_domain_learning::{
        GroundedExplanatoryPredictionStatus, GroundedRelevanceSpecialization,
    };

    let action = atom(10);
    let effect = atom(900);

    let seed = episode(&[1, 2, 99], &[1, 2, 99, 900]);

    let seed_space = GroundedExplanatoryVersionSpaceSynthesis::synthesize(
        std::slice::from_ref(&seed),
        GroundedExplanatoryVersionSpacePolicy::new(1, 16, 64, 32)
            .expect("valid explanatory policy"),
    );

    let general = seed_space
        .active()
        .iter()
        .find(|hypothesis| {
            hypothesis.context().is_none()
                && hypothesis.transformation() == &action
                && hypothesis.effect_fact() == &effect
        })
        .expect("general explanation")
        .clone();

    let wrong_source = seed_space
        .active()
        .iter()
        .find(|hypothesis| {
            hypothesis.context().is_some()
                && hypothesis.transformation() == &action
                && hypothesis.effect_fact() == &effect
        })
        .expect("distinct contextual explanation")
        .clone();

    let evidence = vec![
        seed,
        episode(&[3, 4, 99], &[3, 4, 99, 900]),
        episode(&[1, 2], &[1, 2]),
        episode(&[3, 4], &[3, 4]),
    ];

    let result = GroundedRelevanceSpecialization::discover(
        &general,
        &evidence,
        GroundedRelevanceSpecializationPolicy::new(4, 2, 1).expect("valid relevance policy"),
    );

    let candidate = result.selected().first().expect("relevance specialization");

    assert_eq!(candidate.added_premise(), &atom(99));
    assert!(candidate.resolved());
    assert!(candidate.matches_source(&general));

    assert!(
        candidate.materialize_resolved(&wrong_source).is_none(),
        "specialization evidence must never be applied to another hypothesis",
    );

    let specialized = candidate
        .materialize_resolved(&general)
        .expect("resolved specialization must materialize");

    assert!(specialized.active());

    assert_eq!(
        specialized
            .context()
            .expect("specialized context")
            .premises(),
        &[atom(99)],
    );

    let applicable = specialized.predict(&state(&[7, 99]), &action);

    assert_eq!(
        applicable.status(),
        GroundedExplanatoryPredictionStatus::Predicted,
    );
    assert!(applicable.predicts_addition(&effect));

    let absent_context = specialized.predict(&state(&[7]), &action);

    assert_eq!(
        absent_context.status(),
        GroundedExplanatoryPredictionStatus::ContextNotSatisfied,
    );
}

#[test]
fn progressive_specialization_builds_conjunction_one_prediction_error_at_a_time() {
    use athlesia_universal_domain_learning::GroundedExplanatoryPredictionStatus;

    let action = atom(10);
    let effect = atom(900);
    let first_relevant = atom(50);
    let second_relevant = atom(99);

    // Hidden rule:
    //
    //     fact 50 AND fact 99 AND action 10 -> add fact 900
    //
    // Neither conjunction nor either premise is supplied to the learner.

    let evidence = vec![
        episode(&[7, 50, 99], &[7, 50, 99, 900]),
        episode(&[8, 50, 99], &[8, 50, 99, 900]),
        episode(&[7, 99], &[7, 99]),
        episode(&[8, 99], &[8, 99]),
        episode(&[7, 8, 50], &[7, 8, 50]),
    ];

    let seed_space = GroundedExplanatoryVersionSpaceSynthesis::synthesize(
        std::slice::from_ref(&evidence[0]),
        GroundedExplanatoryVersionSpacePolicy::new(1, 16, 64, 32)
            .expect("valid explanatory policy"),
    );

    let general = seed_space
        .active()
        .iter()
        .find(|hypothesis| {
            hypothesis.context().is_none()
                && hypothesis.transformation() == &action
                && hypothesis.effect_kind() == TransitionEffectKind::Added
                && hypothesis.effect_fact() == &effect
        })
        .expect("initial general explanation")
        .clone();

    let relevance_policy =
        GroundedRelevanceSpecializationPolicy::new(5, 4, 1).expect("valid relevance policy");

    let first_step =
        GroundedRelevanceSpecialization::discover(&general, &evidence, relevance_policy);

    assert_eq!(first_step.selected_count(), 1);

    let first = &first_step.selected()[0];

    assert_eq!(
        first.added_premise(),
        &first_relevant,
        "first error contrast must prefer fact 50",
    );

    assert_eq!(
        first.specialized_context().premises(),
        std::slice::from_ref(&first_relevant),
    );

    assert!(
        first.retained_counterexample_count() > 0,
        "first specialization must remain explicitly falsified",
    );

    assert!(!first.resolved());

    assert!(
        first.materialize_resolved(&general).is_none(),
        "unresolved refinement must not become an active world model",
    );

    let provisional = first
        .materialize_refinement_seed(&general)
        .expect("unresolved but informative refinement must remain explorable");

    assert!(!provisional.active());

    let second_step =
        GroundedRelevanceSpecialization::discover(&provisional, &evidence, relevance_policy);

    assert_eq!(second_step.selected_count(), 1);

    let second = &second_step.selected()[0];

    assert_eq!(
        second.added_premise(),
        &second_relevant,
        "remaining prediction error must expose fact 99",
    );

    assert_eq!(
        second.specialized_context().premises(),
        &[first_relevant.clone(), second_relevant.clone()],
        "conjunction must emerge progressively rather than by subset enumeration",
    );

    assert!(second.resolved());

    let resolved = second
        .materialize_resolved(&provisional)
        .expect("second refinement must produce an executable hypothesis");

    assert!(resolved.active());

    assert_eq!(
        resolved.context().expect("resolved context").premises(),
        &[first_relevant, second_relevant],
    );

    let success_prediction = resolved.predict(&state(&[50, 99]), &action);

    assert_eq!(
        success_prediction.status(),
        GroundedExplanatoryPredictionStatus::Predicted,
    );
    assert!(success_prediction.predicts_addition(&effect));

    let missing_second = resolved.predict(&state(&[50]), &action);

    assert_eq!(
        missing_second.status(),
        GroundedExplanatoryPredictionStatus::ContextNotSatisfied,
    );

    let missing_first = resolved.predict(&state(&[99]), &action);

    assert_eq!(
        missing_first.status(),
        GroundedExplanatoryPredictionStatus::ContextNotSatisfied,
    );
}

#[test]
fn successful_prediction_without_error_does_not_invent_specialization() {
    let action = atom(10);
    let effect = atom(900);

    let evidence = vec![
        episode(&[1, 2, 3], &[1, 2, 3, 900]),
        episode(&[4, 5, 6], &[4, 5, 6, 900]),
        episode(&[7, 8, 9], &[7, 8, 9, 900]),
    ];

    let seed_space = GroundedExplanatoryVersionSpaceSynthesis::synthesize(
        std::slice::from_ref(&evidence[0]),
        GroundedExplanatoryVersionSpacePolicy::new(1, 16, 64, 32)
            .expect("valid explanatory policy"),
    );

    let general = seed_space
        .active()
        .iter()
        .find(|hypothesis| {
            hypothesis.context().is_none()
                && hypothesis.transformation() == &action
                && hypothesis.effect_kind() == TransitionEffectKind::Added
                && hypothesis.effect_fact() == &effect
        })
        .expect("general explanation")
        .clone();

    let result = GroundedRelevanceSpecialization::discover(
        &general,
        &evidence,
        GroundedRelevanceSpecializationPolicy::new(3, 2, 1).expect("valid relevance policy"),
    );

    assert_eq!(
        result.selected_count(),
        0,
        "successful model must not become more complex without prediction error",
    );

    assert_eq!(
        result.candidate_fact_count(),
        0,
        "absence of counterevidence must suppress relevance search itself",
    );

    assert_eq!(
        result.admitted_before_frontier(),
        0,
        "no fake specialization may enter the frontier",
    );
}
