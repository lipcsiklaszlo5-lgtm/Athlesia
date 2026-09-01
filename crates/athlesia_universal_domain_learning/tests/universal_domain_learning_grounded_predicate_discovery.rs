use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    GroundedLearningEpisode, PredicateDiscovery, PredicateDiscoveryPolicy,
    UniversalPredicateDiscovery,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn episode(facts: Vec<CognitiveStructure>, outcome: CognitiveStructure) -> GroundedLearningEpisode {
    GroundedLearningEpisode::new(facts, outcome).unwrap()
}

fn policy(
    support: u64,
    precision: u16,
    lift: u16,
    max_predicates: usize,
) -> PredicateDiscoveryPolicy {
    PredicateDiscoveryPolicy::new(support, signal(precision), signal(lift), max_predicates).unwrap()
}

#[test]
fn grounded_episode_requires_at_least_one_fact() {
    assert_eq!(GroundedLearningEpisode::new(Vec::new(), atom(100,),), None);

    assert!(GroundedLearningEpisode::new(vec![atom(1,),], atom(100,),).is_some());
}

#[test]
fn grounded_episode_canonicalizes_fact_order_and_deduplicates_exact_repetition() {
    let first = episode(vec![atom(2), atom(1), atom(2)], atom(100));

    let second = episode(vec![atom(1), atom(2)], atom(100));

    assert_eq!(first, second);

    assert_eq!(first.fact_count(), 2);
}

#[test]
fn exact_structural_kind_and_order_remain_part_of_predicate_identity() {
    let ordered = CognitiveStructure::ordered(vec![atom(1), atom(2)]).unwrap();

    let reversed = CognitiveStructure::ordered(vec![atom(2), atom(1)]).unwrap();

    let unordered = CognitiveStructure::unordered(vec![atom(1), atom(2)]).unwrap();

    let episodes = vec![
        episode(vec![ordered.clone()], atom(100)),
        episode(vec![ordered.clone()], atom(100)),
        episode(vec![reversed.clone()], atom(200)),
        episode(vec![unordered.clone()], atom(300)),
        episode(vec![atom(999)], atom(400)),
    ];

    let result = PredicateDiscovery::discover(&episodes, policy(1, 1, 1, 16));

    assert!(result.selected().iter().any(|candidate| {
        candidate.antecedent() == &ordered && candidate.consequent() == &atom(100)
    },));

    assert!(result.selected().iter().any(|candidate| {
        candidate.antecedent() == &reversed && candidate.consequent() == &atom(200)
    },));

    assert!(result.selected().iter().any(|candidate| {
        candidate.antecedent() == &unordered && candidate.consequent() == &atom(300)
    },));
}

#[test]
fn repeated_fact_inside_one_episode_does_not_inflate_support_or_opportunity_count() {
    let episodes = vec![
        episode(vec![atom(1), atom(1), atom(1)], atom(100)),
        episode(vec![atom(1)], atom(100)),
        episode(vec![atom(2)], atom(200)),
    ];

    let result = PredicateDiscovery::discover(&episodes, policy(1, 1, 1, 8));

    let discovered = result
        .selected()
        .iter()
        .find(|candidate| {
            candidate.antecedent() == &atom(1) && candidate.consequent() == &atom(100)
        })
        .unwrap();

    assert_eq!(discovered.support_count(), 2);

    assert_eq!(discovered.antecedent_count(), 2);

    assert_eq!(discovered.precision().value(), 1000);
}

#[test]
fn repeated_grounded_association_discovers_a_positive_predicate_hypothesis() {
    let episodes = vec![
        episode(vec![atom(10)], atom(100)),
        episode(vec![atom(10)], atom(100)),
        episode(vec![atom(20)], atom(200)),
        episode(vec![atom(30)], atom(300)),
    ];

    let result = PredicateDiscovery::discover(&episodes, policy(2, 900, 1, 8));

    assert_eq!(result.selected_count(), 1);

    let discovered = &result.selected()[0];

    assert_eq!(discovered.antecedent(), &atom(10,));

    assert_eq!(discovered.consequent(), &atom(100,));

    assert_eq!(discovered.support_count(), 2);

    assert_eq!(discovered.precision().value(), 1000);

    assert_eq!(discovered.baseline_rate().value(), 500);

    assert_eq!(discovered.association_lift().value(), 500);
}

#[test]
fn ubiquitous_outcome_produces_zero_lift_and_is_not_discovered_as_informative_predicate() {
    let episodes = vec![
        episode(vec![atom(1)], atom(100)),
        episode(vec![atom(1)], atom(100)),
        episode(vec![atom(2)], atom(100)),
        episode(vec![atom(3)], atom(100)),
    ];

    let result = PredicateDiscovery::discover(&episodes, policy(1, 1, 1, 8));

    assert_eq!(result.selected_count(), 0);

    assert_eq!(result.discovered_before_policy(), 0);
}

#[test]
fn one_fact_can_support_multiple_lifted_outcomes_without_forcing_single_meaning() {
    let feature = atom(50);

    let episodes = vec![
        episode(vec![feature.clone()], atom(100)),
        episode(vec![feature.clone()], atom(100)),
        episode(vec![feature.clone()], atom(200)),
        episode(vec![feature.clone()], atom(200)),
        episode(vec![atom(60)], atom(300)),
        episode(vec![atom(70)], atom(300)),
    ];

    let result = PredicateDiscovery::discover(&episodes, policy(2, 400, 1, 8));

    let feature_candidates = result
        .selected()
        .iter()
        .filter(|candidate| candidate.antecedent() == &feature)
        .collect::<Vec<_>>();

    assert_eq!(feature_candidates.len(), 2);

    assert!(feature_candidates
        .iter()
        .any(|candidate| { candidate.consequent() == &atom(100,) },));

    assert!(feature_candidates
        .iter()
        .any(|candidate| { candidate.consequent() == &atom(200,) },));
}

#[test]
fn minimum_support_filters_single_episode_coincidences() {
    let episodes = vec![
        episode(vec![atom(1)], atom(100)),
        episode(vec![atom(2)], atom(200)),
        episode(vec![atom(3)], atom(300)),
    ];

    let result = PredicateDiscovery::discover(&episodes, policy(2, 1, 1, 8));

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn minimum_precision_rejects_weakly_predictive_antecedents_even_with_support() {
    let feature = atom(5);

    let episodes = vec![
        episode(vec![feature.clone()], atom(100)),
        episode(vec![feature.clone()], atom(100)),
        episode(vec![feature.clone()], atom(200)),
        episode(vec![feature.clone()], atom(200)),
        episode(vec![feature.clone()], atom(200)),
        episode(vec![atom(6)], atom(300)),
    ];

    let result = PredicateDiscovery::discover(&episodes, policy(2, 600, 1, 8));

    assert!(result.selected().iter().all(|candidate| {
        !(candidate.antecedent() == &feature && candidate.consequent() == &atom(100))
    },));
}

#[test]
fn hard_predicate_frontier_retains_only_best_ranked_hypotheses() {
    let episodes = vec![
        episode(vec![atom(1)], atom(100)),
        episode(vec![atom(1)], atom(100)),
        episode(vec![atom(2)], atom(200)),
        episode(vec![atom(2)], atom(200)),
        episode(vec![atom(3)], atom(300)),
        episode(vec![atom(4)], atom(400)),
    ];

    let result = PredicateDiscovery::discover(&episodes, policy(1, 1, 1, 2));

    assert_eq!(result.selected_count(), 2);

    assert!(result.truncated_by_frontier());

    assert!(result.discovered_before_policy() > result.selected_count());
}

#[test]
fn ranking_prefers_lift_then_precision_then_support_and_is_input_order_invariant() {
    let original = vec![
        episode(vec![atom(1)], atom(100)),
        episode(vec![atom(1)], atom(100)),
        episode(vec![atom(2)], atom(200)),
        episode(vec![atom(2)], atom(200)),
        episode(vec![atom(2)], atom(300)),
        episode(vec![atom(3)], atom(300)),
        episode(vec![atom(4)], atom(400)),
    ];

    let mut reversed = original.clone();

    reversed.reverse();

    let discovery_policy = policy(1, 1, 1, 16);

    let first = PredicateDiscovery::discover(&original, discovery_policy);

    let second = PredicateDiscovery::discover(&reversed, discovery_policy);

    assert_eq!(first, second);

    for pair in first.selected().windows(2) {
        let left = &pair[0];

        let right = &pair[1];

        assert!(left.association_lift().value() >= right.association_lift().value());
    }
}

#[test]
fn predicate_discovery_is_deterministic_non_mutating_and_facade_equivalent() {
    let episodes = vec![
        episode(vec![atom(1), atom(2)], atom(100)),
        episode(vec![atom(1), atom(3)], atom(100)),
        episode(vec![atom(4)], atom(200)),
        episode(vec![atom(5)], atom(300)),
    ];

    let before = episodes.clone();

    let discovery_policy = policy(1, 1, 1, 16);

    let direct = PredicateDiscovery::discover(&episodes, discovery_policy);

    let facade = UniversalPredicateDiscovery::evaluate(&episodes, discovery_policy);

    let repeated = UniversalPredicateDiscovery::evaluate(&episodes, discovery_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, before);

    assert_eq!(facade.episode_count(), 4);
}
