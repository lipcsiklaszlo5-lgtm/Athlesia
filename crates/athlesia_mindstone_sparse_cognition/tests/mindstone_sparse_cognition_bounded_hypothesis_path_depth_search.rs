use athlesia_mindstone_sparse_cognition::{
    BoundedHypothesisPathDepthSearch, BoundedHypothesisSearchNode, BoundedHypothesisSearchPolicy,
    CognitiveBudget, CognitiveFingerprint, CognitiveSignal, CognitiveStructure,
    CollisionSafeStructuralIdentity, MindstoneBoundedHypothesisPathDepthSearch,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn budget(units: u32) -> CognitiveBudget {
    CognitiveBudget::new(units).unwrap()
}

fn atom(value: u64) -> CognitiveStructure {
    CognitiveStructure::atom(value)
}

fn identity(value: u64) -> CollisionSafeStructuralIdentity {
    CollisionSafeStructuralIdentity::from_structure(atom(value))
}

fn hinted_identity(fingerprint: u64, value: u64) -> CollisionSafeStructuralIdentity {
    CollisionSafeStructuralIdentity::with_fingerprint_hint(
        CognitiveFingerprint::new(fingerprint),
        atom(value),
    )
}

fn node(
    value: u64,
    score: u16,
    cost: u32,
    depth: u16,
    path_length: usize,
) -> BoundedHypothesisSearchNode {
    BoundedHypothesisSearchNode::new(identity(value), signal(score), cost, depth, path_length)
        .unwrap()
}

fn policy(
    max_hypotheses: usize,
    max_path_length: usize,
    max_depth: u16,
) -> BoundedHypothesisSearchPolicy {
    BoundedHypothesisSearchPolicy::new(max_hypotheses, max_path_length, max_depth).unwrap()
}

#[test]
fn hypothesis_node_requires_positive_score_cost_path_and_consistent_depth() {
    assert_eq!(
        BoundedHypothesisSearchNode::new(identity(1), CognitiveSignal::zero(), 1, 0, 1,),
        None
    );

    assert_eq!(
        BoundedHypothesisSearchNode::new(identity(1), signal(500), 0, 0, 1,),
        None
    );

    assert_eq!(
        BoundedHypothesisSearchNode::new(identity(1), signal(500), 1, 0, 0,),
        None
    );

    assert_eq!(
        BoundedHypothesisSearchNode::new(identity(1), signal(500), 1, 2, 2,),
        None
    );

    let valid = node(1, 500, 2, 1, 2);

    assert_eq!(valid.depth(), 1);

    assert_eq!(valid.path_length(), 2);
}

#[test]
fn hypothesis_search_policy_requires_nonzero_frontier_and_path_bound() {
    assert_eq!(BoundedHypothesisSearchPolicy::new(0, 4, 2,), None);

    assert_eq!(BoundedHypothesisSearchPolicy::new(4, 0, 2,), None);

    let root_only = policy(4, 1, 0);

    assert_eq!(root_only.max_depth(), 0);

    assert_eq!(root_only.max_hypotheses(), 4);
}

#[test]
fn higher_scoring_hypothesis_is_ranked_first() {
    let candidates = vec![
        node(1, 500, 1, 0, 1),
        node(2, 900, 1, 0, 1),
        node(3, 700, 1, 0, 1),
    ];

    let result =
        BoundedHypothesisPathDepthSearch::search(&candidates, policy(8, 8, 8), budget(100));

    assert_eq!(result.selected_count(), 3);

    assert_eq!(result.selected()[0].structure(), &atom(2));

    assert_eq!(result.selected()[0].score().value(), 900);
}

#[test]
fn equal_score_prefers_shallower_hypothesis() {
    let candidates = vec![
        node(1, 800, 1, 2, 3),
        node(2, 800, 1, 0, 1),
        node(3, 800, 1, 1, 2),
    ];

    let result =
        BoundedHypothesisPathDepthSearch::search(&candidates, policy(8, 8, 8), budget(100));

    assert_eq!(result.selected()[0].depth(), 0);

    assert_eq!(result.selected()[1].depth(), 1);

    assert_eq!(result.selected()[2].depth(), 2);
}

#[test]
fn equal_score_and_depth_prefers_shorter_path_then_lower_cost() {
    let candidates = vec![
        node(1, 700, 1, 1, 4),
        node(2, 700, 5, 1, 2),
        node(3, 700, 2, 1, 2),
    ];

    let result =
        BoundedHypothesisPathDepthSearch::search(&candidates, policy(8, 8, 8), budget(100));

    assert_eq!(result.selected()[0].structure(), &atom(3));

    assert_eq!(result.selected()[0].estimated_cost(), 2);

    assert_eq!(result.selected()[1].structure(), &atom(2));

    assert_eq!(result.selected()[2].path_length(), 4);
}

#[test]
fn depth_and_path_limits_filter_hypotheses_before_frontier_admission() {
    let candidates = vec![
        node(1, 900, 1, 0, 1),
        node(2, 1000, 1, 3, 4),
        node(3, 950, 1, 1, 5),
    ];

    let result =
        BoundedHypothesisPathDepthSearch::search(&candidates, policy(8, 3, 2), budget(100));

    assert_eq!(result.input_count(), 3);

    assert_eq!(result.eligible_count(), 1);

    assert_eq!(result.rejected_by_depth_count(), 1);

    assert_eq!(result.rejected_by_path_count(), 1);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].structure(), &atom(1));
}

#[test]
fn hard_hypothesis_frontier_retains_only_best_bounded_set() {
    let candidates = (1_u64..=100)
        .map(|value| node(value, value as u16, 1, 0, 1))
        .collect::<Vec<_>>();

    let result =
        BoundedHypothesisPathDepthSearch::search(&candidates, policy(4, 4, 3), budget(100));

    assert_eq!(result.input_count(), 100);

    assert_eq!(result.retained_frontier_count(), 4);

    assert_eq!(result.selected_count(), 4);

    assert_eq!(result.selected()[0].score().value(), 100);

    assert_eq!(result.selected()[3].score().value(), 97);

    assert!(result.dropped_by_frontier_count() > 0);
}

#[test]
fn exact_duplicate_identity_keeps_only_best_ranked_variant() {
    let same_identity = identity(55);

    let candidates = vec![
        BoundedHypothesisSearchNode::new(same_identity.clone(), signal(500), 5, 2, 3).unwrap(),
        BoundedHypothesisSearchNode::new(same_identity, signal(900), 2, 0, 1).unwrap(),
    ];

    let result =
        BoundedHypothesisPathDepthSearch::search(&candidates, policy(8, 8, 8), budget(100));

    assert_eq!(result.eligible_count(), 2);

    assert_eq!(result.retained_frontier_count(), 1);

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].score().value(), 900);
}

#[test]
fn forced_same_fingerprint_different_structures_remain_distinct_hypotheses() {
    let shared = 777_u64;

    let first =
        BoundedHypothesisSearchNode::new(hinted_identity(shared, 1), signal(800), 1, 0, 1).unwrap();

    let second =
        BoundedHypothesisSearchNode::new(hinted_identity(shared, 2), signal(700), 1, 0, 1).unwrap();

    let result =
        BoundedHypothesisPathDepthSearch::search(&[first, second], policy(8, 8, 8), budget(100));

    assert_eq!(result.retained_frontier_count(), 2);

    assert_eq!(result.selected_count(), 2);

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(shared,)
    );

    assert_ne!(
        result.selected()[0].structure(),
        result.selected()[1].structure()
    );
}

#[test]
fn unaffordable_next_best_hypothesis_stops_without_cheaper_tail_substitution() {
    let candidates = vec![
        node(1, 1000, 2, 0, 1),
        node(2, 900, 5, 0, 1),
        node(3, 800, 1, 0, 1),
    ];

    let result = BoundedHypothesisPathDepthSearch::search(&candidates, policy(8, 8, 8), budget(4));

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].structure(), &atom(1));

    assert_eq!(result.total_selected_cost(), 2);

    assert!(result.truncated_by_compute_budget());

    assert!(!result
        .selected()
        .iter()
        .any(|candidate| { candidate.structure() == &atom(3) },));
}

#[test]
fn zero_depth_root_hypothesis_is_valid_under_root_only_policy() {
    let root = node(100, 600, 1, 0, 1);

    let deeper = node(200, 1000, 1, 1, 2);

    let root_policy = policy(4, 1, 0);

    assert!(root_policy.admits(&root,));

    assert!(!root_policy.admits(&deeper,));

    let result = BoundedHypothesisPathDepthSearch::search(&[root, deeper], root_policy, budget(10));

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.selected()[0].depth(), 0);
}

#[test]
fn bounded_hypothesis_search_is_deterministic_non_mutating_and_facade_equivalent() {
    let candidates = vec![
        node(1, 900, 2, 0, 1),
        node(2, 800, 2, 1, 2),
        node(3, 700, 2, 2, 3),
    ];

    let before = candidates.clone();

    let search_policy = policy(3, 3, 2);

    let compute = budget(6);

    let direct = BoundedHypothesisPathDepthSearch::search(&candidates, search_policy, compute);

    let facade =
        MindstoneBoundedHypothesisPathDepthSearch::evaluate(&candidates, search_policy, compute);

    let repeated =
        MindstoneBoundedHypothesisPathDepthSearch::evaluate(&candidates, search_policy, compute);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(candidates, before);

    assert_eq!(facade.selected_count(), 3);

    assert!(facade.total_selected_cost() <= compute.units());
}
