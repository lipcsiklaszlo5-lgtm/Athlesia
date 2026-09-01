use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    GroundedLearningEpisode, PredicateDiscovery, PredicateDiscoveryPolicy, RuleEvidenceThresholds,
    RuleInduction, RuleInductionPolicy, RulePremiseSet, UniversalRuleInduction, MAX_RULE_PREMISES,
};

#[derive(Clone, Copy)]
struct SearchBounds {
    max_premises: usize,
    max_premise_sets: usize,
    max_evaluations: usize,
    max_rules: usize,
}

#[derive(Clone, Copy)]
struct EvidenceSpec {
    support: u64,
    precision: u16,
    lift: u16,
    incremental_gain: u16,
}

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn episode(facts: &[u64], outcome: u64) -> GroundedLearningEpisode {
    GroundedLearningEpisode::new(facts.iter().copied().map(atom).collect(), atom(outcome)).unwrap()
}

fn search_bounds(
    max_premises: usize,
    max_premise_sets: usize,
    max_evaluations: usize,
    max_rules: usize,
) -> SearchBounds {
    SearchBounds {
        max_premises,
        max_premise_sets,
        max_evaluations,
        max_rules,
    }
}

fn evidence_spec(support: u64, precision: u16, lift: u16, incremental_gain: u16) -> EvidenceSpec {
    EvidenceSpec {
        support,
        precision,
        lift,
        incremental_gain,
    }
}

fn thresholds(spec: EvidenceSpec) -> RuleEvidenceThresholds {
    RuleEvidenceThresholds::new(
        spec.support,
        signal(spec.precision),
        signal(spec.lift),
        signal(spec.incremental_gain),
    )
    .unwrap()
}

fn policy(bounds: SearchBounds, evidence: EvidenceSpec) -> RuleInductionPolicy {
    RuleInductionPolicy::new(
        bounds.max_premises,
        bounds.max_premise_sets,
        bounds.max_evaluations,
        bounds.max_rules,
        thresholds(evidence),
    )
    .unwrap()
}

fn empty_seeds() -> Vec<athlesia_universal_domain_learning::GroundedPredicateHypothesis> {
    Vec::new()
}

fn predicate_seeds(
    episodes: &[GroundedLearningEpisode],
    minimum_support: u64,
) -> Vec<athlesia_universal_domain_learning::GroundedPredicateHypothesis> {
    let discovery_policy =
        PredicateDiscoveryPolicy::new(minimum_support, signal(1), signal(1), 64).unwrap();

    PredicateDiscovery::discover(episodes, discovery_policy)
        .selected()
        .to_vec()
}

fn has_rule(
    result: &athlesia_universal_domain_learning::RuleInductionResult,
    premises: &[u64],
    consequent: u64,
) -> bool {
    let expected = RulePremiseSet::new(premises.iter().copied().map(atom).collect()).unwrap();

    result
        .selected()
        .iter()
        .any(|rule| rule.premises() == &expected && rule.consequent() == &atom(consequent))
}

#[test]
fn rule_premise_set_requires_two_unique_facts_and_canonicalizes_exact_identity() {
    assert_eq!(RulePremiseSet::new(vec![atom(1,),],), None);

    assert_eq!(RulePremiseSet::new(vec![atom(1,), atom(1,),],), None);

    let first = RulePremiseSet::new(vec![atom(2), atom(1), atom(2)]).unwrap();

    let second = RulePremiseSet::new(vec![atom(1), atom(2)]).unwrap();

    assert_eq!(first, second);

    assert_eq!(first.premise_count(), 2);
}

#[test]
fn rule_policy_enforces_positive_evidence_bounds_and_bounded_premise_arity() {
    assert_eq!(
        RuleEvidenceThresholds::new(0, signal(500,), signal(100,), signal(100,),),
        None
    );

    assert_eq!(
        RuleEvidenceThresholds::new(1, signal(500,), signal(0,), signal(100,),),
        None
    );

    assert_eq!(
        RuleEvidenceThresholds::new(1, signal(500,), signal(100,), signal(0,),),
        None
    );

    let valid_thresholds = thresholds(evidence_spec(1, 500, 100, 100));

    assert_eq!(
        RuleInductionPolicy::new(1, 10, 10, 10, valid_thresholds,),
        None
    );

    assert_eq!(
        RuleInductionPolicy::new(MAX_RULE_PREMISES + 1, 10, 10, 10, valid_thresholds,),
        None
    );

    assert_eq!(
        RuleInductionPolicy::new(2, 0, 10, 10, valid_thresholds,),
        None
    );

    assert!(RuleInductionPolicy::new(2, 10, 10, 10, valid_thresholds,).is_some());
}

#[test]
fn conjunction_can_be_discovered_when_joint_precision_exceeds_every_single_premise() {
    let episodes = vec![
        episode(&[1, 2], 100),
        episode(&[1, 2], 100),
        episode(&[1], 200),
        episode(&[2], 300),
        episode(&[9], 100),
    ];

    let result = RuleInduction::induce(
        &episodes,
        &empty_seeds(),
        policy(
            search_bounds(2, 64, 256, 16),
            evidence_spec(2, 900, 300, 300),
        ),
    );

    assert!(has_rule(&result, &[1, 2,], 100,));

    let expected = RulePremiseSet::new(vec![atom(1), atom(2)]).unwrap();

    let rule = result
        .selected()
        .iter()
        .find(|rule| rule.premises() == &expected && rule.consequent() == &atom(100))
        .unwrap();

    assert_eq!(rule.precision().value(), 1000);

    assert_eq!(rule.best_proper_subset_precision().value(), 666);

    assert_eq!(rule.incremental_precision_gain().value(), 334);
}

#[test]
fn redundant_conjunction_is_rejected_when_a_proper_subset_is_equally_predictive() {
    let episodes = vec![
        episode(&[1, 2], 100),
        episode(&[1, 2], 100),
        episode(&[1, 3], 100),
        episode(&[9], 200),
    ];

    let result = RuleInduction::induce(
        &episodes,
        &empty_seeds(),
        policy(search_bounds(2, 64, 256, 16), evidence_spec(2, 900, 1, 1)),
    );

    assert!(!has_rule(&result, &[1, 2,], 100,));
}

#[test]
fn falsifying_episodes_are_retained_as_explicit_rule_counterexamples() {
    let episodes = vec![
        episode(&[1, 2], 100),
        episode(&[1, 2], 100),
        episode(&[1, 2], 200),
        episode(&[1], 300),
        episode(&[2], 400),
        episode(&[9], 100),
    ];

    let result = RuleInduction::induce(
        &episodes,
        &empty_seeds(),
        policy(
            search_bounds(2, 64, 256, 16),
            evidence_spec(2, 600, 100, 100),
        ),
    );

    let expected = RulePremiseSet::new(vec![atom(1), atom(2)]).unwrap();

    let rule = result
        .selected()
        .iter()
        .find(|rule| rule.premises() == &expected && rule.consequent() == &atom(100))
        .unwrap();

    assert_eq!(rule.support_count(), 2);

    assert_eq!(rule.premise_opportunity_count(), 3);

    assert_eq!(rule.counterexample_count(), 1);

    assert_eq!(rule.precision().value(), 666);

    assert!(rule.is_counterexample(&episodes[2],));
}

#[test]
fn ubiquitous_outcome_has_zero_association_lift_and_cannot_become_rule() {
    let episodes = vec![
        episode(&[1, 2], 100),
        episode(&[1, 2], 100),
        episode(&[1, 3], 100),
        episode(&[2, 4], 100),
    ];

    let result = RuleInduction::induce(
        &episodes,
        &empty_seeds(),
        policy(search_bounds(2, 64, 256, 32), evidence_spec(1, 1, 1, 1)),
    );

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn three_way_rule_can_outperform_every_proper_subset() {
    let episodes = vec![
        episode(&[1, 2, 3], 100),
        episode(&[1, 2, 3], 100),
        episode(&[1, 2], 200),
        episode(&[1, 3], 300),
        episode(&[2, 3], 400),
        episode(&[9], 500),
    ];

    let result = RuleInduction::induce(
        &episodes,
        &empty_seeds(),
        policy(
            search_bounds(3, 128, 1024, 32),
            evidence_spec(2, 900, 300, 300),
        ),
    );

    assert!(has_rule(&result, &[1, 2, 3,], 100,));

    let rule = result
        .selected()
        .iter()
        .find(|rule| rule.premises().premise_count() == 3 && rule.consequent() == &atom(100))
        .unwrap();

    assert_eq!(rule.best_proper_subset_precision().value(), 666);

    assert_eq!(rule.incremental_precision_gain().value(), 334);
}

#[test]
fn maximum_premise_arity_prevents_hidden_three_way_rule_search() {
    let episodes = vec![
        episode(&[1, 2, 3], 100),
        episode(&[1, 2, 3], 100),
        episode(&[1, 2], 200),
        episode(&[1, 3], 300),
        episode(&[2, 3], 400),
        episode(&[9], 500),
    ];

    let result = RuleInduction::induce(
        &episodes,
        &empty_seeds(),
        policy(
            search_bounds(2, 128, 1024, 32),
            evidence_spec(2, 900, 300, 300),
        ),
    );

    assert!(!has_rule(&result, &[1, 2, 3,], 100,));

    assert!(result
        .selected()
        .iter()
        .all(|rule| { rule.premises().premise_count() <= 2 },));
}

#[test]
fn predicate_seeds_prioritize_candidate_generation_under_hard_premise_budget() {
    let episodes = vec![
        episode(&[1, 2], 100),
        episode(&[1, 2], 100),
        episode(&[1], 200),
        episode(&[2], 300),
        episode(&[7], 400),
        episode(&[8], 500),
    ];

    let seeds = predicate_seeds(&episodes, 2);

    let result = RuleInduction::induce(
        &episodes,
        &seeds,
        policy(search_bounds(2, 1, 16, 8), evidence_spec(2, 900, 300, 300)),
    );

    assert_eq!(result.generated_premise_set_count(), 1);

    assert!(result.candidate_generation_truncated());

    assert!(result.seeded_vocabulary_fact_count() >= 2);

    assert!(has_rule(&result, &[1, 2,], 100,));
}

#[test]
fn hard_rule_evaluation_budget_is_reported_and_never_exceeded() {
    let episodes = vec![
        episode(&[1, 2], 100),
        episode(&[1, 3], 200),
        episode(&[2, 3], 300),
        episode(&[4, 5], 400),
    ];

    let result = RuleInduction::induce(
        &episodes,
        &empty_seeds(),
        policy(search_bounds(2, 64, 1, 32), evidence_spec(1, 1, 1, 1)),
    );

    assert_eq!(result.evaluated_rule_candidate_count(), 1);

    assert!(result.rule_evaluation_truncated());

    assert!(result.possible_premise_set_count() >= result.generated_premise_set_count());
}

#[test]
fn rule_ranking_prefers_incremental_gain_and_is_episode_order_invariant() {
    let original = vec![
        episode(&[1, 2], 100),
        episode(&[1, 2], 100),
        episode(&[1], 200),
        episode(&[2], 300),
        episode(&[3, 4], 400),
        episode(&[3, 4], 400),
        episode(&[3, 4], 500),
        episode(&[3], 600),
        episode(&[4], 700),
    ];

    let mut reversed = original.clone();

    reversed.reverse();

    let induction_policy = policy(search_bounds(2, 128, 2048, 64), evidence_spec(2, 500, 1, 1));

    let first = RuleInduction::induce(&original, &empty_seeds(), induction_policy);

    let second = RuleInduction::induce(&reversed, &empty_seeds(), induction_policy);

    assert_eq!(first, second);

    for pair in first.selected().windows(2) {
        assert!(
            pair[0].incremental_precision_gain().value()
                >= pair[1].incremental_precision_gain().value()
        );
    }
}

#[test]
fn rule_induction_is_deterministic_non_mutating_and_facade_equivalent() {
    let episodes = vec![
        episode(&[1, 2], 100),
        episode(&[1, 2], 100),
        episode(&[1], 200),
        episode(&[2], 300),
        episode(&[9], 100),
    ];

    let seeds = predicate_seeds(&episodes, 1);

    let episodes_before = episodes.clone();

    let seeds_before = seeds.clone();

    let induction_policy = policy(search_bounds(3, 128, 2048, 64), evidence_spec(1, 1, 1, 1));

    let direct = RuleInduction::induce(&episodes, &seeds, induction_policy);

    let facade = UniversalRuleInduction::evaluate(&episodes, &seeds, induction_policy);

    let repeated = UniversalRuleInduction::evaluate(&episodes, &seeds, induction_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, episodes_before);

    assert_eq!(seeds, seeds_before);

    assert!(facade.vocabulary_fact_count() >= 3);
}
