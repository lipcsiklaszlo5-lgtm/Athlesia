use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    GroundedLearningEpisode, GroundedRuleHypothesis, GroundedStateSnapshot,
    GroundedTransformationEpisode, InvariantDiscovery, InvariantDiscoveryPolicy,
    RuleEvidenceThresholds, RuleInduction, RuleInductionPolicy, UniversalInvariantDiscovery,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn ordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(values.iter().copied().map(atom).collect()).unwrap()
}

fn unordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::unordered(values.iter().copied().map(atom).collect()).unwrap()
}

fn snapshot(facts: Vec<CognitiveStructure>) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts).unwrap()
}

fn transition(
    before: Vec<CognitiveStructure>,
    after: Vec<CognitiveStructure>,
    transformation: CognitiveStructure,
) -> GroundedTransformationEpisode {
    GroundedTransformationEpisode::new(snapshot(before), snapshot(after), transformation)
}

fn policy(
    support: u64,
    preservation: u16,
    transformations: usize,
    max_candidates: usize,
    max_invariants: usize,
) -> InvariantDiscoveryPolicy {
    InvariantDiscoveryPolicy::new(
        support,
        signal(preservation),
        transformations,
        max_candidates,
        max_invariants,
    )
    .unwrap()
}

fn empty_rules() -> Vec<GroundedRuleHypothesis> {
    Vec::new()
}

fn learning_episode(facts: &[u64], outcome: u64) -> GroundedLearningEpisode {
    GroundedLearningEpisode::new(facts.iter().copied().map(atom).collect(), atom(outcome)).unwrap()
}

fn rule_seeds() -> Vec<GroundedRuleHypothesis> {
    let episodes = vec![
        learning_episode(&[8, 9], 100),
        learning_episode(&[8, 9], 100),
        learning_episode(&[8], 200),
        learning_episode(&[9], 300),
        learning_episode(&[7], 100),
    ];

    let thresholds = RuleEvidenceThresholds::new(2, signal(900), signal(300), signal(300)).unwrap();

    let induction_policy = RuleInductionPolicy::new(2, 64, 256, 16, thresholds).unwrap();

    RuleInduction::induce(&episodes, &[], induction_policy)
        .selected()
        .to_vec()
}

fn has_invariant(
    result: &athlesia_universal_domain_learning::InvariantDiscoveryResult,
    fact: &CognitiveStructure,
) -> bool {
    result
        .selected()
        .iter()
        .any(|candidate| candidate.fact() == fact)
}

#[test]
fn grounded_state_snapshot_requires_nonempty_facts_and_canonicalizes_exact_duplicates() {
    assert_eq!(GroundedStateSnapshot::new(Vec::new(),), None);

    let first = snapshot(vec![atom(2), atom(1), atom(2)]);

    let second = snapshot(vec![atom(1), atom(2)]);

    assert_eq!(first, second);

    assert_eq!(first.fact_count(), 2);
}

#[test]
fn transformation_identity_remains_opaque_and_exact() {
    let first = transition(vec![atom(1)], vec![atom(1), atom(2)], ordered(&[10, 20]));

    let second = transition(vec![atom(1)], vec![atom(1), atom(2)], ordered(&[20, 10]));

    assert_ne!(first.transformation(), second.transformation());

    assert!(first.preserves(&atom(1,),));
}

#[test]
fn invariant_policy_requires_positive_evidence_and_hard_bounds() {
    assert_eq!(
        InvariantDiscoveryPolicy::new(0, signal(900,), 1, 10, 10,),
        None
    );

    assert_eq!(
        InvariantDiscoveryPolicy::new(1, signal(0,), 1, 10, 10,),
        None
    );

    assert_eq!(
        InvariantDiscoveryPolicy::new(1, signal(900,), 0, 10, 10,),
        None
    );

    assert_eq!(
        InvariantDiscoveryPolicy::new(1, signal(900,), 1, 0, 10,),
        None
    );

    assert!(InvariantDiscoveryPolicy::new(1, signal(900,), 1, 10, 10,).is_some());
}

#[test]
fn exact_fact_preserved_while_surrounding_context_changes_is_discovered_as_invariant() {
    let episodes = vec![
        transition(vec![atom(1), atom(2)], vec![atom(1), atom(20)], atom(100)),
        transition(vec![atom(1), atom(3)], vec![atom(1), atom(30)], atom(200)),
    ];

    let result =
        InvariantDiscovery::discover(&episodes, &empty_rules(), policy(2, 1000, 2, 16, 16));

    assert!(has_invariant(&result, &atom(1,),));

    let invariant = &result.selected()[0];

    assert_eq!(invariant.stable_support_count(), 2);

    assert_eq!(invariant.preservation_rate().value(), 1000);

    assert_eq!(invariant.distinct_stable_transformations(), 2);
}

#[test]
fn disrupted_preservation_opportunity_is_retained_as_explicit_counterexample() {
    let episodes = vec![
        transition(vec![atom(1)], vec![atom(1)], atom(100)),
        transition(vec![atom(1)], vec![atom(1)], atom(200)),
        transition(vec![atom(1)], vec![atom(9)], atom(300)),
    ];

    let result = InvariantDiscovery::discover(&episodes, &empty_rules(), policy(2, 600, 2, 16, 16));

    let invariant = result
        .selected()
        .iter()
        .find(|candidate| candidate.fact() == &atom(1))
        .unwrap();

    assert_eq!(invariant.stable_support_count(), 2);

    assert_eq!(invariant.opportunity_count(), 3);

    assert_eq!(invariant.disruption_count(), 1);

    assert_eq!(invariant.preservation_rate().value(), 666);

    assert!(invariant.is_counterexample(&episodes[2],));
}

#[test]
fn after_only_emergence_is_not_misclassified_as_preserved_invariant() {
    let episodes = vec![
        transition(vec![atom(1)], vec![atom(1), atom(9)], atom(100)),
        transition(vec![atom(2)], vec![atom(2), atom(9)], atom(200)),
    ];

    let result = InvariantDiscovery::discover(&episodes, &empty_rules(), policy(1, 1, 1, 16, 16));

    assert!(!has_invariant(&result, &atom(9,),));
}

#[test]
fn repeated_same_transformation_does_not_fake_cross_transformation_invariance() {
    let episodes = vec![
        transition(vec![atom(1)], vec![atom(1)], atom(100)),
        transition(vec![atom(1)], vec![atom(1)], atom(100)),
        transition(vec![atom(1)], vec![atom(1)], atom(100)),
    ];

    let result =
        InvariantDiscovery::discover(&episodes, &empty_rules(), policy(3, 1000, 2, 16, 16));

    assert!(!has_invariant(&result, &atom(1,),));
}

#[test]
fn stability_across_distinct_transformations_is_recorded_separately_from_episode_support() {
    let episodes = vec![
        transition(vec![atom(1)], vec![atom(1)], atom(100)),
        transition(vec![atom(1)], vec![atom(1)], atom(100)),
        transition(vec![atom(1)], vec![atom(1)], atom(200)),
    ];

    let result =
        InvariantDiscovery::discover(&episodes, &empty_rules(), policy(3, 1000, 2, 16, 16));

    let invariant = result
        .selected()
        .iter()
        .find(|candidate| candidate.fact() == &atom(1))
        .unwrap();

    assert_eq!(invariant.stable_support_count(), 3);

    assert_eq!(invariant.distinct_stable_transformations(), 2);

    assert_eq!(invariant.distinct_opportunity_transformations(), 2);

    assert_eq!(invariant.transformation_stability().value(), 1000);
}

#[test]
fn ordered_unordered_and_reordered_structures_remain_distinct_invariant_candidates() {
    let ordered_a = ordered(&[1, 2]);

    let ordered_b = ordered(&[2, 1]);

    let unordered_value = unordered(&[1, 2]);

    let episodes = vec![
        transition(
            vec![
                ordered_a.clone(),
                ordered_b.clone(),
                unordered_value.clone(),
            ],
            vec![
                ordered_a.clone(),
                ordered_b.clone(),
                unordered_value.clone(),
            ],
            atom(100),
        ),
        transition(
            vec![
                ordered_a.clone(),
                ordered_b.clone(),
                unordered_value.clone(),
            ],
            vec![
                ordered_a.clone(),
                ordered_b.clone(),
                unordered_value.clone(),
            ],
            atom(200),
        ),
    ];

    let result =
        InvariantDiscovery::discover(&episodes, &empty_rules(), policy(2, 1000, 2, 16, 16));

    assert!(has_invariant(&result, &ordered_a,));

    assert!(has_invariant(&result, &ordered_b,));

    assert!(has_invariant(&result, &unordered_value,));

    assert_ne!(ordered_a, ordered_b);

    assert_ne!(ordered_a, unordered_value);
}

#[test]
fn rule_seeds_prioritize_invariant_candidate_generation_without_becoming_semantic_gate() {
    let rules = rule_seeds();

    assert!(!rules.is_empty());

    let episodes = vec![
        transition(vec![atom(1), atom(9)], vec![atom(1), atom(9)], atom(100)),
        transition(vec![atom(1), atom(9)], vec![atom(1), atom(9)], atom(200)),
    ];

    let bounded = InvariantDiscovery::discover(&episodes, &rules, policy(2, 1000, 2, 1, 16));

    assert_eq!(bounded.evaluated_candidate_count(), 1);

    assert!(bounded.candidate_generation_truncated());

    assert!(bounded.seeded_vocabulary_fact_count() >= 1);

    assert!(has_invariant(&bounded, &atom(9,),));

    let unseeded = InvariantDiscovery::discover(&episodes, &[], policy(2, 1000, 2, 16, 16));

    assert!(has_invariant(&unseeded, &atom(1,),));

    assert!(has_invariant(&unseeded, &atom(9,),));
}

#[test]
fn hard_invariant_frontier_prefers_stronger_stability_and_is_episode_order_invariant() {
    let original = vec![
        transition(vec![atom(1), atom(2)], vec![atom(1), atom(2)], atom(100)),
        transition(vec![atom(1), atom(2)], vec![atom(1)], atom(200)),
        transition(vec![atom(1), atom(2)], vec![atom(1), atom(2)], atom(300)),
    ];

    let mut reversed = original.clone();

    reversed.reverse();

    let discovery_policy = policy(2, 600, 2, 16, 1);

    let first = InvariantDiscovery::discover(&original, &[], discovery_policy);

    let second = InvariantDiscovery::discover(&reversed, &[], discovery_policy);

    assert_eq!(first, second);

    assert_eq!(first.selected_count(), 1);

    assert!(first.admitted_before_frontier() > first.selected_count());

    assert_eq!(first.selected()[0].fact(), &atom(1,));

    assert_eq!(first.selected()[0].preservation_rate().value(), 1000);
}

#[test]
fn invariant_discovery_is_deterministic_non_mutating_and_facade_equivalent() {
    let episodes = vec![
        transition(vec![atom(1), atom(2)], vec![atom(1), atom(20)], atom(100)),
        transition(vec![atom(1), atom(3)], vec![atom(1), atom(30)], atom(200)),
    ];

    let rules = rule_seeds();

    let episodes_before = episodes.clone();

    let rules_before = rules.clone();

    let discovery_policy = policy(1, 500, 1, 32, 16);

    let direct = InvariantDiscovery::discover(&episodes, &rules, discovery_policy);

    let facade = UniversalInvariantDiscovery::evaluate(&episodes, &rules, discovery_policy);

    let repeated = UniversalInvariantDiscovery::evaluate(&episodes, &rules, discovery_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, episodes_before);

    assert_eq!(rules, rules_before);

    assert_eq!(facade.episode_count(), 2);
}
