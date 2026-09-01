use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    ContextPremiseSet, ContextualTransitionEvidenceThresholds, ContextualTransitionRuleInduction,
    ContextualTransitionRulePolicy, CrossContextGeneralization, CrossContextGeneralizationPolicy,
    CrossContextGeneralizationThresholds, ExceptionRefinement, ExceptionRefinementPolicy,
    ExceptionRefinementThresholds, GroundedCrossContextGeneralizationHypothesis,
    GroundedStateSnapshot, GroundedTransformationEpisode, TransitionEffectKind,
    UniversalExceptionRefinement, MAX_CONTEXT_PREMISES,
};

#[derive(Clone, Copy)]
struct ExceptionBounds {
    max_seeds: usize,
    max_premises: usize,
    max_candidates: usize,
    max_evaluations: usize,
    max_refinements: usize,
}

#[derive(Clone, Copy)]
struct ExceptionEvidence {
    support: u64,
    failure_rate: u16,
    failure_lift: u16,
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

fn exception_bounds(
    max_seeds: usize,
    max_premises: usize,
    max_candidates: usize,
    max_evaluations: usize,
    max_refinements: usize,
) -> ExceptionBounds {
    ExceptionBounds {
        max_seeds,
        max_premises,
        max_candidates,
        max_evaluations,
        max_refinements,
    }
}

fn exception_evidence(support: u64, failure_rate: u16, failure_lift: u16) -> ExceptionEvidence {
    ExceptionEvidence {
        support,
        failure_rate,
        failure_lift,
    }
}

fn exception_policy(
    bounds: ExceptionBounds,
    evidence: ExceptionEvidence,
) -> ExceptionRefinementPolicy {
    let thresholds = ExceptionRefinementThresholds::new(
        evidence.support,
        signal(evidence.failure_rate),
        signal(evidence.failure_lift),
    )
    .unwrap();

    ExceptionRefinementPolicy::new(
        bounds.max_seeds,
        bounds.max_premises,
        bounds.max_candidates,
        bounds.max_evaluations,
        bounds.max_refinements,
        thresholds,
    )
    .unwrap()
}

fn default_exception_policy() -> ExceptionRefinementPolicy {
    exception_policy(
        exception_bounds(128, 2, 128, 128, 32),
        exception_evidence(2, 900, 400),
    )
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

fn conjunctive_exception_history(
    transformation: CognitiveStructure,
) -> Vec<GroundedTransformationEpisode> {
    vec![
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[1, 6, 8, 9], &[1, 5, 6, 8, 9], transformation.clone()),
        transition(&[1, 6, 8, 9], &[1, 5, 6, 8, 9], transformation.clone()),
        transition(&[1, 7, 8, 9], &[1, 5, 7, 8, 9], transformation.clone()),
        transition(&[1, 7, 8, 9], &[1, 5, 7, 8, 9], transformation.clone()),
        transition(&[1, 6, 7, 9], &[1, 6, 7, 9], transformation.clone()),
        transition(&[1, 6, 7, 9], &[1, 6, 7, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation),
    ]
}

fn dual_marker_history(transformation: CognitiveStructure) -> Vec<GroundedTransformationEpisode> {
    vec![
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 5, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 2, 9], &[4, 2, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[4, 3, 9], &[4, 3, 9], transformation.clone()),
        transition(&[1, 6, 7, 9], &[1, 6, 7, 9], transformation.clone()),
        transition(&[1, 6, 7, 9], &[1, 6, 7, 9], transformation),
    ]
}

fn seed_generalizations_from(
    episodes: &[GroundedTransformationEpisode],
) -> Vec<GroundedCrossContextGeneralizationHypothesis> {
    let contextual_thresholds =
        ContextualTransitionEvidenceThresholds::new(2, signal(900), signal(300), signal(300))
            .unwrap();

    let contextual_policy =
        ContextualTransitionRulePolicy::new(2, 256, 8192, 256, contextual_thresholds).unwrap();

    let contextual = ContextualTransitionRuleInduction::induce(episodes, &[], contextual_policy);

    let generalization_thresholds =
        CrossContextGeneralizationThresholds::new(2, 2, signal(600), signal(200)).unwrap();

    let generalization_policy =
        CrossContextGeneralizationPolicy::new(256, 2, 256, 128, generalization_thresholds).unwrap();

    CrossContextGeneralization::generalize(episodes, contextual.selected(), generalization_policy)
        .selected()
        .to_vec()
}

fn context(facts: &[u64]) -> ContextPremiseSet {
    ContextPremiseSet::new(facts.iter().copied().map(atom).collect()).unwrap()
}

fn has_exception(
    result: &athlesia_universal_domain_learning::ExceptionRefinementResult,
    transformation: &CognitiveStructure,
    base: &ContextPremiseSet,
    exception: &ContextPremiseSet,
    kind: TransitionEffectKind,
    fact: &CognitiveStructure,
) -> bool {
    result.selected().iter().any(|hypothesis| {
        hypothesis.transformation() == transformation
            && hypothesis.base_context() == base
            && hypothesis.exception_context() == exception
            && hypothesis.effect_kind() == kind
            && hypothesis.effect_fact() == fact
    })
}

#[test]
fn exception_policy_requires_positive_failure_evidence_and_hard_bounds() {
    assert_eq!(
        ExceptionRefinementThresholds::new(0, signal(900,), signal(100,),),
        None
    );

    assert_eq!(
        ExceptionRefinementThresholds::new(1, signal(0,), signal(100,),),
        None
    );

    assert_eq!(
        ExceptionRefinementThresholds::new(1, signal(900,), signal(0,),),
        None
    );

    let thresholds = ExceptionRefinementThresholds::new(1, signal(900), signal(100)).unwrap();

    assert_eq!(
        ExceptionRefinementPolicy::new(0, 1, 10, 10, 10, thresholds,),
        None
    );

    assert_eq!(
        ExceptionRefinementPolicy::new(10, MAX_CONTEXT_PREMISES + 1, 10, 10, 10, thresholds,),
        None
    );

    assert!(ExceptionRefinementPolicy::new(10, 1, 10, 10, 10, thresholds,).is_some());
}

#[test]
fn generalization_without_observed_counterexamples_produces_no_exception() {
    let transformation = atom(100);

    let training = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&training);

    assert!(!seeds.is_empty());

    let evaluation = vec![
        transition(&[1, 2], &[1, 2, 5], transformation.clone()),
        transition(&[1, 3], &[1, 3, 5], transformation),
    ];

    let result = ExceptionRefinement::refine(&evaluation, &seeds, default_exception_policy());

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn repeated_failure_specific_grounded_fact_is_discovered_as_exception() {
    let transformation = atom(100);

    let episodes = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&episodes);

    let result = ExceptionRefinement::refine(&episodes, &seeds, default_exception_policy());

    assert!(has_exception(
        &result,
        &transformation,
        &context(&[1,],),
        &context(&[6,],),
        TransitionEffectKind::Added,
        &atom(5,),
    ));
}

#[test]
fn exception_retains_base_failure_lift_coverage_and_leakage_evidence() {
    let transformation = atom(100);

    let episodes = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&episodes);

    let result = ExceptionRefinement::refine(&episodes, &seeds, default_exception_policy());

    let hypothesis = result
        .selected()
        .iter()
        .find(|candidate| {
            candidate.transformation() == &transformation
                && candidate.exception_context() == &context(&[6])
                && candidate.effect_fact() == &atom(5)
        })
        .unwrap();

    assert_eq!(hypothesis.base_opportunity_count(), 6);

    assert_eq!(hypothesis.base_failure_count(), 2);

    assert_eq!(hypothesis.exception_opportunity_count(), 2);

    assert_eq!(hypothesis.exception_failure_count(), 2);

    assert_eq!(hypothesis.exception_success_count(), 0);

    assert_eq!(hypothesis.base_failure_rate().value(), 333);

    assert_eq!(hypothesis.exception_failure_rate().value(), 1000);

    assert_eq!(hypothesis.failure_lift().value(), 667);

    assert_eq!(hypothesis.failure_coverage().value(), 1000);

    assert!(hypothesis.explains_counterexample(&episodes[8],));

    assert!(!hypothesis.leaks_on_support(&episodes[0],));
}

#[test]
fn base_context_premise_is_not_reused_as_exception_condition() {
    let episodes = base_history(atom(100));

    let seeds = seed_generalizations_from(&episodes);

    let result = ExceptionRefinement::refine(&episodes, &seeds, default_exception_policy());

    assert!(result.selected().iter().all(|hypothesis| {
        !hypothesis
            .exception_context()
            .premises()
            .iter()
            .any(|premise| hypothesis.base_context().premises().contains(premise))
    },));
}

#[test]
fn one_off_failure_marker_is_rejected_by_minimum_failure_support() {
    let transformation = atom(100);

    let training = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&training);

    let evaluation = vec![
        transition(&[1, 2], &[1, 2, 5], transformation.clone()),
        transition(&[1, 3], &[1, 3, 5], transformation.clone()),
        transition(&[1, 6], &[1, 6], transformation.clone()),
        transition(&[1, 7], &[1, 7], transformation),
    ];

    let result = ExceptionRefinement::refine(
        &evaluation,
        &seeds,
        exception_policy(
            exception_bounds(128, 1, 128, 128, 32),
            exception_evidence(2, 900, 100),
        ),
    );

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn candidate_with_no_failure_rate_lift_over_base_rule_is_rejected() {
    let transformation = atom(100);

    let training = base_history(transformation.clone());

    let seeds = seed_generalizations_from(&training);

    let evaluation = vec![
        transition(&[1, 6], &[1, 5, 6], transformation.clone()),
        transition(&[1, 6], &[1, 5, 6], transformation.clone()),
        transition(&[1, 6], &[1, 6], transformation.clone()),
        transition(&[1, 6], &[1, 6], transformation.clone()),
        transition(&[1, 7], &[1, 5, 7], transformation.clone()),
        transition(&[1, 7], &[1, 5, 7], transformation.clone()),
        transition(&[1, 7], &[1, 7], transformation.clone()),
        transition(&[1, 7], &[1, 7], transformation),
    ];

    let result = ExceptionRefinement::refine(
        &evaluation,
        &seeds,
        exception_policy(
            exception_bounds(128, 1, 128, 128, 32),
            exception_evidence(2, 400, 1),
        ),
    );

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn conjunctive_exception_is_discovered_when_single_failure_markers_are_insufficient() {
    let transformation = atom(100);

    let episodes = conjunctive_exception_history(transformation.clone());

    let seeds = seed_generalizations_from(&episodes);

    let result = ExceptionRefinement::refine(
        &episodes,
        &seeds,
        exception_policy(
            exception_bounds(128, 2, 256, 256, 64),
            exception_evidence(2, 900, 500),
        ),
    );

    assert!(has_exception(
        &result,
        &transformation,
        &context(&[1,],),
        &context(&[6, 7,],),
        TransitionEffectKind::Added,
        &atom(5,),
    ));

    assert!(!has_exception(
        &result,
        &transformation,
        &context(&[1,],),
        &context(&[6,],),
        TransitionEffectKind::Added,
        &atom(5,),
    ));
}

#[test]
fn maximum_exception_arity_prevents_hidden_conjunctive_exception_search() {
    let transformation = atom(100);

    let episodes = conjunctive_exception_history(transformation.clone());

    let seeds = seed_generalizations_from(&episodes);

    let result = ExceptionRefinement::refine(
        &episodes,
        &seeds,
        exception_policy(
            exception_bounds(128, 1, 256, 256, 64),
            exception_evidence(2, 900, 500),
        ),
    );

    assert!(!has_exception(
        &result,
        &transformation,
        &context(&[1,],),
        &context(&[6, 7,],),
        TransitionEffectKind::Added,
        &atom(5,),
    ));

    assert!(result
        .selected()
        .iter()
        .all(|hypothesis| { hypothesis.exception_context().premise_count() <= 1 },));
}

#[test]
fn exact_transformation_structure_identity_keeps_exception_groups_distinct() {
    let first = ordered(&[10, 20]);

    let second = ordered(&[20, 10]);

    let mut episodes = base_history(first.clone());

    episodes.extend(base_history(second.clone()));

    let seeds = seed_generalizations_from(&episodes);

    let result = ExceptionRefinement::refine(&episodes, &seeds, default_exception_policy());

    assert_ne!(first, second);

    assert!(has_exception(
        &result,
        &first,
        &context(&[1,],),
        &context(&[6,],),
        TransitionEffectKind::Added,
        &atom(5,),
    ));

    assert!(has_exception(
        &result,
        &second,
        &context(&[1,],),
        &context(&[6,],),
        TransitionEffectKind::Added,
        &atom(5,),
    ));
}

#[test]
fn hard_exception_search_and_final_frontiers_are_bounded_and_input_order_invariant() {
    let transformation = atom(100);

    let original = dual_marker_history(transformation);

    let seeds = seed_generalizations_from(&original);

    let mut reversed_episodes = original.clone();

    reversed_episodes.reverse();

    let mut reversed_seeds = seeds.clone();

    reversed_seeds.reverse();

    let refinement_policy = exception_policy(
        exception_bounds(128, 1, 128, 128, 1),
        exception_evidence(2, 900, 400),
    );

    let first = ExceptionRefinement::refine(&original, &seeds, refinement_policy);

    let second =
        ExceptionRefinement::refine(&reversed_episodes, &reversed_seeds, refinement_policy);

    assert_eq!(first, second);

    assert_eq!(first.selected_count(), 1);

    assert!(first.admitted_before_frontier() > first.selected_count());

    assert_eq!(first.selected()[0].exception_context(), &context(&[6,],));

    assert!(first.possible_candidate_context_count() >= first.generated_candidate_context_count());

    assert!(first.generated_candidate_context_count() >= first.evaluated_candidate_count());
}

#[test]
fn exception_refinement_is_deterministic_non_mutating_and_facade_equivalent() {
    let episodes = base_history(atom(100));

    let seeds = seed_generalizations_from(&episodes);

    let episodes_before = episodes.clone();

    let seeds_before = seeds.clone();

    let refinement_policy = default_exception_policy();

    let direct = ExceptionRefinement::refine(&episodes, &seeds, refinement_policy);

    let facade = UniversalExceptionRefinement::evaluate(&episodes, &seeds, refinement_policy);

    let repeated = UniversalExceptionRefinement::evaluate(&episodes, &seeds, refinement_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, episodes_before);

    assert_eq!(seeds, seeds_before);

    assert_eq!(facade.input_seed_count(), seeds.len());

    assert!(facade.considered_seed_count() <= facade.input_seed_count());
}
