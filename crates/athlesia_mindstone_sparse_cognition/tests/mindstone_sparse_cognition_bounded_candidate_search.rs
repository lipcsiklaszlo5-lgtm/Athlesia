use athlesia_mindstone_sparse_cognition::{
    BoundedCandidateSearch, BoundedCandidateSearchPolicy, CognitiveBudget, CognitiveCandidate,
    CognitiveFingerprint, CognitiveSignal, CognitiveStructure, MindstoneBoundedCandidateSearch,
    MindstoneSignalProfile, MindstoneStreamingAggregator, StreamingAggregationState,
};

fn signal(value: u16) -> CognitiveSignal {
    CognitiveSignal::new(value).unwrap()
}

fn salience(value: u16) -> athlesia_mindstone_sparse_cognition::CognitiveSalience {
    MindstoneSignalProfile::new(
        signal(value),
        signal(value),
        signal(value),
        signal(value),
        signal(value),
    )
    .salience()
}

fn candidate(id: u64, salience_value: u16, support: u64, cost: u32) -> CognitiveCandidate {
    CognitiveCandidate::new(
        CognitiveFingerprint::new(id),
        salience(salience_value),
        support,
        cost,
    )
    .unwrap()
}

fn policy(max_candidates: usize) -> BoundedCandidateSearchPolicy {
    BoundedCandidateSearchPolicy::new(max_candidates).unwrap()
}

fn budget(units: u32) -> CognitiveBudget {
    CognitiveBudget::new(units).unwrap()
}

#[test]
fn candidate_and_search_policy_require_positive_bounds() {
    assert_eq!(
        CognitiveCandidate::new(CognitiveFingerprint::new(1,), salience(500,), 0, 1,),
        None
    );

    assert_eq!(
        CognitiveCandidate::new(CognitiveFingerprint::new(1,), salience(500,), 1, 0,),
        None
    );

    assert_eq!(BoundedCandidateSearchPolicy::new(0,), None);

    assert_eq!(policy(3,).max_candidates(), 3);
}

#[test]
fn higher_salience_candidate_ranks_first() {
    let result = BoundedCandidateSearch::select(
        vec![candidate(1, 300, 10, 1), candidate(2, 900, 1, 1)],
        policy(2),
        budget(100),
    );

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(2,)
    );
}

#[test]
fn equal_salience_prefers_higher_support() {
    let result = BoundedCandidateSearch::select(
        vec![candidate(1, 700, 2, 1), candidate(2, 700, 20, 1)],
        policy(2),
        budget(100),
    );

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(2,)
    );
}

#[test]
fn equal_salience_and_support_prefers_lower_compute_cost() {
    let result = BoundedCandidateSearch::select(
        vec![candidate(1, 700, 10, 8), candidate(2, 700, 10, 2)],
        policy(2),
        budget(100),
    );

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(2,)
    );
}

#[test]
fn complete_rank_tie_uses_fingerprint_identity_deterministically() {
    let result = BoundedCandidateSearch::select(
        vec![candidate(20, 700, 10, 2), candidate(10, 700, 10, 2)],
        policy(2),
        budget(100),
    );

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(10,)
    );

    assert_eq!(
        result.selected()[1].fingerprint(),
        CognitiveFingerprint::new(20,)
    );
}

#[test]
fn duplicate_fingerprint_is_canonicalized_to_best_candidate() {
    let result = BoundedCandidateSearch::select(
        vec![
            candidate(7, 300, 100, 1),
            candidate(7, 900, 1, 9),
            candidate(8, 500, 1, 1),
        ],
        policy(3),
        budget(100),
    );

    assert_eq!(result.input_candidate_count(), 3);

    assert_eq!(result.unique_candidate_count(), 2);

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(7,)
    );

    assert_eq!(result.selected()[0].salience(), salience(900,));
}

#[test]
fn candidate_count_limit_is_a_hard_upper_bound() {
    let result = BoundedCandidateSearch::select(
        vec![
            candidate(1, 900, 1, 1),
            candidate(2, 800, 1, 1),
            candidate(3, 700, 1, 1),
            candidate(4, 600, 1, 1),
        ],
        policy(2),
        budget(100),
    );

    assert_eq!(result.selected_count(), 2);

    assert!(result.truncated_by_candidate_limit());

    assert!(result.was_truncated());

    assert_eq!(result.total_selected_cost(), 2);
}

#[test]
fn compute_budget_is_a_hard_total_cost_bound() {
    let result = BoundedCandidateSearch::select(
        vec![
            candidate(1, 900, 1, 3),
            candidate(2, 800, 1, 3),
            candidate(3, 700, 1, 3),
        ],
        policy(10),
        budget(5),
    );

    assert_eq!(result.selected_count(), 1);

    assert_eq!(result.total_selected_cost(), 3);

    assert!(result.truncated_by_compute_budget());

    assert!(result.total_selected_cost() <= 5);
}

#[test]
fn unaffordable_best_remaining_candidate_is_not_skipped_for_cheaper_tail() {
    let result = BoundedCandidateSearch::select(
        vec![
            candidate(1, 1000, 1, 3),
            candidate(2, 900, 1, 4),
            candidate(3, 800, 1, 1),
        ],
        policy(10),
        budget(5),
    );

    assert_eq!(result.selected_count(), 1);

    assert_eq!(
        result.selected()[0].fingerprint(),
        CognitiveFingerprint::new(1,)
    );

    assert!(result.truncated_by_compute_budget());

    assert!(!result
        .selected()
        .iter()
        .any(|item| { item.fingerprint() == CognitiveFingerprint::new(3,) },));
}

#[test]
fn very_large_candidate_pool_has_strictly_bounded_selected_frontier() {
    let candidates = (0_u64..10_000)
        .map(|id| candidate(id, 500, id + 1, 1))
        .collect::<Vec<_>>();

    let result = BoundedCandidateSearch::select(candidates, policy(8), budget(100));

    assert_eq!(result.input_candidate_count(), 10_000);

    assert_eq!(result.unique_candidate_count(), 10_000);

    assert_eq!(result.selected_count(), 8);

    assert!(result.truncated_by_candidate_limit());

    assert_eq!(result.total_selected_cost(), 8);
}

#[test]
fn streaming_sufficient_statistics_can_become_ranked_candidates() {
    let initial = StreamingAggregationState::new(4).unwrap();

    let profile = MindstoneSignalProfile::new(
        signal(800),
        signal(800),
        signal(800),
        signal(800),
        signal(800),
    );

    let first =
        MindstoneStreamingAggregator::observe(initial, 1, CognitiveStructure::atom(11), profile);

    let second = MindstoneStreamingAggregator::observe(
        first.aggregation().state_after().clone(),
        2,
        CognitiveStructure::atom(11),
        profile,
    );

    let aggregate = second.aggregation().aggregate().unwrap();

    let derived = CognitiveCandidate::from_streaming_aggregate(aggregate, 3).unwrap();

    assert_eq!(derived.fingerprint(), second.fingerprint());

    assert_eq!(derived.support(), 2);

    assert_eq!(derived.salience(), profile.salience());

    assert_eq!(derived.estimated_cost(), 3);
}

#[test]
fn bounded_search_is_deterministic_non_mutating_and_facade_equivalent() {
    let candidates = vec![
        candidate(3, 700, 5, 2),
        candidate(1, 900, 2, 3),
        candidate(2, 800, 9, 1),
        candidate(4, 600, 20, 1),
    ];

    let before = candidates.clone();

    let search_policy = policy(3);

    let compute_budget = budget(6);

    let direct = BoundedCandidateSearch::select(candidates.clone(), search_policy, compute_budget);

    let facade = MindstoneBoundedCandidateSearch::evaluate(
        candidates.clone(),
        search_policy,
        compute_budget,
    );

    let reversed = MindstoneBoundedCandidateSearch::evaluate(
        candidates.iter().copied().rev().collect(),
        search_policy,
        compute_budget,
    );

    assert_eq!(direct, facade);

    assert_eq!(facade, reversed);

    assert_eq!(candidates, before);

    assert_eq!(facade.selected_count(), 3);

    assert_eq!(facade.total_selected_cost(), 6);
}
