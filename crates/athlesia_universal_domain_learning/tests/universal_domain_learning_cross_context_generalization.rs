use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    ContextPremiseSet, ContextualTransitionEvidenceThresholds, ContextualTransitionRuleInduction,
    ContextualTransitionRulePolicy, CrossContextGeneralization, CrossContextGeneralizationPolicy,
    CrossContextGeneralizationThresholds, GroundedContextualTransitionRuleHypothesis,
    GroundedStateSnapshot, GroundedTransformationEpisode, TransitionEffectKind,
    UniversalCrossContextGeneralization, MAX_CONTEXT_PREMISES,
};

#[derive(Clone, Copy)]
struct GeneralizationBounds {
    max_seed_rules: usize,
    max_generalized_premises: usize,
    max_candidates: usize,
    max_generalizations: usize,
}

#[derive(Clone, Copy)]
struct GeneralizationEvidence {
    minimum_seed_contexts: usize,
    support: u64,
    precision: u16,
    incremental_gain: u16,
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(values.iter().copied().map(atom).collect()).unwrap()
}

fn snapshot(facts: &[u64]) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts.iter().copied().map(atom).collect()).unwrap()
}

fn transition(
    before: &[u64],
    after: &[u64],
    transformation: CognitiveStructure,
) -> GroundedTransformationEpisode {
    GroundedTransformationEpisode::new(snapshot(before), snapshot(after), transformation)
}

fn bounds(
    max_seed_rules: usize,
    max_generalized_premises: usize,
    max_candidates: usize,
    max_generalizations: usize,
) -> GeneralizationBounds {
    GeneralizationBounds {
        max_seed_rules,
        max_generalized_premises,
        max_candidates,
        max_generalizations,
    }
}

fn evidence(
    minimum_seed_contexts: usize,
    support: u64,
    precision: u16,
    incremental_gain: u16,
) -> GeneralizationEvidence {
    GeneralizationEvidence {
        minimum_seed_contexts,
        support,
        precision,
        incremental_gain,
    }
}

fn policy(
    search: GeneralizationBounds,
    evidence: GeneralizationEvidence,
) -> CrossContextGeneralizationPolicy {
    let thresholds = CrossContextGeneralizationThresholds::new(
        evidence.minimum_seed_contexts,
        evidence.support,
        signal(evidence.precision),
        signal(evidence.incremental_gain),
    )
    .unwrap();

    CrossContextGeneralizationPolicy::new(
        search.max_seed_rules,
        search.max_generalized_premises,
        search.max_candidates,
        search.max_generalizations,
        thresholds,
    )
    .unwrap()
}

fn base_history(transformation: CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
    vec![
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[1, 6, 9], &[1, 6, 9], transformation.clone()),
        transition(&[1, 6, 9], &[1, 6, 9], transformation),
    ]
}

fn dual_effect_history(transformation: CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
    vec![
        transition(&[1, 2, 9], &[1, 2, 5, 6, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 6, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 6, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 6, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[1, 6, 9], &[1, 6, 9], transformation.clone()),
        transition(&[1, 6, 9], &[1, 6, 9], transformation),
    ]
}

fn strength_history(transformation: CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
    vec![
        transition(&[1, 2, 9], &[1, 2, 5, 6, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 6, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 6, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 6, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[1, 7, 9], &[1, 5, 7, 9], transformation.clone()),
        transition(&[1, 7, 9], &[1, 5, 7, 9], transformation),
    ]
}

fn seed_rules_from(
    episodes: &[GroundedTransformationEpisode],
) -> Vec<GroundedContextualTransitionRuleHypothesis> {
    let thresholds =
        ContextualTransitionEvidenceThresholds::new(2, signal(900), signal(300), signal(300))
            .unwrap();

    let induction_policy =
        ContextualTransitionRulePolicy::new(2, 128, 4096, 128, thresholds).unwrap();

    ContextualTransitionRuleInduction::induce(episodes, &[], induction_policy)
        .selected()
        .to_vec()
}

fn default_policy() -> CrossContextGeneralizationPolicy {
    policy(bounds(128, 2, 128, 32), evidence(2, 3, 600, 200))
}

fn context(facts: &[u64]) -> ContextPremiseSet {
    ContextPremiseSet::new(facts.iter().copied().map(atom).collect()).unwrap()
}

fn has_generalization(
    result: &athlesia_universal_domain_learning::CrossContextGeneralizationResult,
    transformation: &CognitiveStructure,
    generalized_context: &ContextPremiseSet,
    kind: TransitionEffectKind,
    fact: &CognitiveStructure,
) -> bool {
    result.selected().iter().any(|hypothesis| {
        hypothesis.transformation() == transformation
            && hypothesis.generalized_context() == generalized_context
            && hypothesis.effect_kind() == kind
            && hypothesis.effect_fact() == fact
    })
}

#[test]
fn cross_context_policy_requires_multiple_seed_contexts_positive_evidence_and_hard_bounds() {
    assert_eq!(
        CrossContextGeneralizationThresholds::new(1, 1, signal(600,), signal(100,),),
        None
    );

    assert_eq!(
        CrossContextGeneralizationThresholds::new(2, 0, signal(600,), signal(100,),),
        None
    );

    assert_eq!(
        CrossContextGeneralizationThresholds::new(2, 1, signal(600,), signal(0,),),
        None
    );

    let thresholds =
        CrossContextGeneralizationThresholds::new(2, 1, signal(600), signal(100)).unwrap();

    assert_eq!(
        CrossContextGeneralizationPolicy::new(0, 1, 10, 10, thresholds,),
        None
    );

    assert_eq!(
        CrossContextGeneralizationPolicy::new(10, MAX_CONTEXT_PREMISES + 1, 10, 10, thresholds,),
        None
    );

    assert!(CrossContextGeneralizationPolicy::new(10, 1, 10, 10, thresholds,).is_some());
}

#[test]
fn one_seed_context_cannot_be_promoted_to_cross_context_generalization() {
    let episodes = base_history(atom(100));

    let seeds = seed_rules_from(&episodes);

    assert!(seeds.len() >= 2);

    let single_seed = vec![seeds[0].clone()];

    let result = CrossContextGeneralization::generalize(&episodes, &single_seed, default_policy());

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn shared_proper_subset_is_discovered_across_distinct_predictive_seed_contexts() {
    let transformation = atom(100);

    let episodes = base_history(transformation.clone());

    let seeds = seed_rules_from(&episodes);

    let result = CrossContextGeneralization::generalize(&episodes, &seeds, default_policy());

    let shared = context(&[1]);

    assert!(has_generalization(
        &result,
        &transformation,
        &shared,
        TransitionEffectKind::Added,
        &atom(5,),
    ));

    let hypothesis = result
        .selected()
        .iter()
        .find(|hypothesis| {
            hypothesis.generalized_context() == &shared && hypothesis.effect_fact() == &atom(5)
        })
        .unwrap();

    assert_eq!(hypothesis.covered_seed_context_count(), 2);

    assert_eq!(hypothesis.minimum_premise_reduction(), 1);

    assert_eq!(hypothesis.support_count(), 4);

    assert_eq!(hypothesis.context_opportunity_count(), 6);

    assert_eq!(hypothesis.precision().value(), 666);

    assert_eq!(hypothesis.transformation_precision().value(), 400);

    assert_eq!(hypothesis.incremental_precision_gain().value(), 266);
}

#[test]
fn full_seed_context_is_not_re_emitted_as_a_generalization() {
    let episodes = base_history(atom(100));

    let seeds = seed_rules_from(&episodes);

    let result = CrossContextGeneralization::generalize(&episodes, &seeds, default_policy());

    for hypothesis in result.selected() {
        assert!(seeds
            .iter()
            .all(|seed| { hypothesis.generalized_context() != seed.context() },));
    }
}

#[test]
fn shared_seed_subset_without_episode_level_predictive_gain_is_rejected() {
    let transformation = atom(100);

    let training = base_history(transformation.clone());

    let seeds = seed_rules_from(&training);

    let evaluation = vec![
        transition(&[1, 2], &[1, 2, 5], transformation.clone()),
        transition(&[1, 3], &[1, 3, 5], transformation.clone()),
        transition(&[4, 2], &[4, 2, 5], transformation.clone()),
        transition(&[4, 3], &[4, 3, 5], transformation),
    ];

    let result = CrossContextGeneralization::generalize(
        &evaluation,
        &seeds,
        policy(bounds(128, 2, 128, 32), evidence(2, 1, 1, 1)),
    );

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn generalized_rule_retains_explicit_counterexamples_from_broader_context() {
    let transformation = atom(100);

    let episodes = base_history(transformation.clone());

    let seeds = seed_rules_from(&episodes);

    let result = CrossContextGeneralization::generalize(&episodes, &seeds, default_policy());

    let hypothesis = result
        .selected()
        .iter()
        .find(|hypothesis| {
            hypothesis.transformation() == &transformation
                && hypothesis.generalized_context() == &context(&[1])
                && hypothesis.effect_fact() == &atom(5)
        })
        .unwrap();

    assert_eq!(hypothesis.counterexample_count(), 2);

    assert!(hypothesis.is_counterexample(&episodes[8],));

    assert!(hypothesis.is_supported_by(&episodes[0],));
}

#[test]
fn exact_transformation_structure_identity_keeps_generalization_groups_distinct() {
    let first = ordered(&[10, 20]);

    let second = ordered(&[20, 10]);

    let mut episodes = base_history(first.clone());

    episodes.extend(base_history(second.clone()));

    let seeds = seed_rules_from(&episodes);

    let result = CrossContextGeneralization::generalize(&episodes, &seeds, default_policy());

    let shared = context(&[1]);

    assert_ne!(first, second);

    assert!(has_generalization(
        &result,
        &first,
        &shared,
        TransitionEffectKind::Added,
        &atom(5,),
    ));

    assert!(has_generalization(
        &result,
        &second,
        &shared,
        TransitionEffectKind::Added,
        &atom(5,),
    ));
}

#[test]
fn different_effect_targets_remain_distinct_during_cross_context_generalization() {
    let transformation = atom(100);

    let episodes = dual_effect_history(transformation.clone());

    let seeds = seed_rules_from(&episodes);

    let result = CrossContextGeneralization::generalize(&episodes, &seeds, default_policy());

    let shared = context(&[1]);

    assert!(has_generalization(
        &result,
        &transformation,
        &shared,
        TransitionEffectKind::Added,
        &atom(5,),
    ));

    assert!(has_generalization(
        &result,
        &transformation,
        &shared,
        TransitionEffectKind::Added,
        &atom(6,),
    ));
}

#[test]
fn hard_seed_rule_budget_prevents_unbounded_generalization_search() {
    let episodes = base_history(atom(100));

    let seeds = seed_rules_from(&episodes);

    assert!(seeds.len() > 1);

    let result = CrossContextGeneralization::generalize(
        &episodes,
        &seeds,
        policy(bounds(1, 2, 128, 32), evidence(2, 1, 1, 1)),
    );

    assert_eq!(result.considered_seed_rule_count(), 1);

    assert!(result.seed_rule_truncated());

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn hard_candidate_generalization_budget_is_reported_and_never_exceeded() {
    let episodes = dual_effect_history(atom(100));

    let seeds = seed_rules_from(&episodes);

    let result = CrossContextGeneralization::generalize(
        &episodes,
        &seeds,
        policy(bounds(128, 2, 1, 32), evidence(2, 1, 1, 1)),
    );

    assert!(result.possible_candidate_count() >= 2);

    assert_eq!(result.evaluated_candidate_count(), 1);

    assert!(result.candidate_generation_truncated());
}

#[test]
fn hard_generalization_frontier_prefers_stronger_transfer_and_is_input_order_invariant() {
    let transformation = atom(100);

    let original = strength_history(transformation);

    let seeds = seed_rules_from(&original);

    let mut reversed_episodes = original.clone();

    reversed_episodes.reverse();

    let mut reversed_seeds = seeds.clone();

    reversed_seeds.reverse();

    let generalization_policy = policy(bounds(128, 2, 128, 1), evidence(2, 3, 600, 200));

    let first = CrossContextGeneralization::generalize(&original, &seeds, generalization_policy);

    let second = CrossContextGeneralization::generalize(
        &reversed_episodes,
        &reversed_seeds,
        generalization_policy,
    );

    assert_eq!(first, second);

    assert_eq!(first.selected_count(), 1);

    assert!(first.admitted_before_frontier() > first.selected_count());

    assert_eq!(first.selected()[0].effect_fact(), &atom(5,));

    assert_eq!(first.selected()[0].precision().value(), 1000);
}

#[test]
fn cross_context_generalization_is_deterministic_non_mutating_and_facade_equivalent() {
    let episodes = base_history(atom(100));

    let seeds = seed_rules_from(&episodes);

    let episodes_before = episodes.clone();

    let seeds_before = seeds.clone();

    let generalization_policy = default_policy();

    let direct = CrossContextGeneralization::generalize(&episodes, &seeds, generalization_policy);

    let facade =
        UniversalCrossContextGeneralization::evaluate(&episodes, &seeds, generalization_policy);

    let repeated =
        UniversalCrossContextGeneralization::evaluate(&episodes, &seeds, generalization_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, episodes_before);

    assert_eq!(seeds, seeds_before);

    assert_eq!(facade.input_seed_rule_count(), seeds.len());

    assert!(facade.possible_candidate_count() >= facade.evaluated_candidate_count());
}
