use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    GroundedInvariantHypothesis, GroundedStateSnapshot, GroundedTransformationEpisode,
    InvariantDiscovery, InvariantDiscoveryPolicy, TransitionEffectKind, TransitionSchemaInduction,
    TransitionSchemaPolicy, UniversalTransitionSchemaInduction,
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

fn policy(
    support: u64,
    precision: u16,
    lift: u16,
    max_candidates: usize,
    max_schemas: usize,
) -> TransitionSchemaPolicy {
    TransitionSchemaPolicy::new(
        support,
        signal(precision),
        signal(lift),
        max_candidates,
        max_schemas,
    )
    .unwrap()
}

fn invariant_seeds() -> Vec<GroundedInvariantHypothesis> {
    let history = vec![
        transition(&[1, 2, 9], &[1, 9], atom(10)),
        transition(&[1, 2, 9], &[1, 9], atom(20)),
    ];

    let discovery_policy = InvariantDiscoveryPolicy::new(2, signal(1000), 2, 16, 16).unwrap();

    InvariantDiscovery::discover(&history, &[], discovery_policy)
        .selected()
        .to_vec()
}

fn has_schema(
    result: &athlesia_universal_domain_learning::TransitionSchemaInductionResult,
    transformation: &CognitiveStructure,
    kind: TransitionEffectKind,
    fact: &CognitiveStructure,
) -> bool {
    result.selected().iter().any(|schema| {
        schema.transformation() == transformation
            && schema.effect_kind() == kind
            && schema.fact() == fact
    })
}

#[test]
fn transition_effect_semantics_distinguish_add_remove_and_preservation() {
    let episode = transition(&[1, 2], &[1, 3], atom(100));

    assert!(episode.effect_occurs(TransitionEffectKind::Added, &atom(3,),));

    assert!(episode.effect_occurs(TransitionEffectKind::Removed, &atom(2,),));

    assert!(!episode.effect_occurs(TransitionEffectKind::Added, &atom(1,),));

    assert!(!episode.effect_occurs(TransitionEffectKind::Removed, &atom(1,),));
}

#[test]
fn transition_schema_policy_requires_positive_support_lift_and_hard_bounds() {
    assert_eq!(
        TransitionSchemaPolicy::new(0, signal(900,), signal(100,), 10, 10,),
        None
    );

    assert_eq!(
        TransitionSchemaPolicy::new(1, signal(900,), signal(0,), 10, 10,),
        None
    );

    assert_eq!(
        TransitionSchemaPolicy::new(1, signal(900,), signal(100,), 0, 10,),
        None
    );

    assert!(TransitionSchemaPolicy::new(1, signal(900,), signal(100,), 10, 10,).is_some());
}

#[test]
fn repeated_transformation_specific_addition_is_discovered() {
    let transformation = atom(100);

    let episodes = vec![
        transition(&[9], &[9, 1], transformation.clone()),
        transition(&[9], &[9, 1], transformation.clone()),
        transition(&[9], &[9], atom(200)),
        transition(&[9], &[9], atom(200)),
    ];

    let result = TransitionSchemaInduction::induce(&episodes, &[], policy(2, 1000, 400, 64, 16));

    assert!(has_schema(
        &result,
        &transformation,
        TransitionEffectKind::Added,
        &atom(1,),
    ));

    let schema = result
        .selected()
        .iter()
        .find(|schema| {
            schema.transformation() == &transformation
                && schema.effect_kind() == TransitionEffectKind::Added
                && schema.fact() == &atom(1)
        })
        .unwrap();

    assert_eq!(schema.support_count(), 2);

    assert_eq!(schema.precision().value(), 1000);

    assert_eq!(schema.baseline_rate().value(), 500);

    assert_eq!(schema.association_lift().value(), 500);
}

#[test]
fn repeated_transformation_specific_removal_is_discovered() {
    let transformation = atom(100);

    let episodes = vec![
        transition(&[1, 9], &[9], transformation.clone()),
        transition(&[1, 9], &[9], transformation.clone()),
        transition(&[1, 9], &[1, 9], atom(200)),
        transition(&[1, 9], &[1, 9], atom(200)),
    ];

    let result = TransitionSchemaInduction::induce(&episodes, &[], policy(2, 1000, 400, 64, 16));

    assert!(has_schema(
        &result,
        &transformation,
        TransitionEffectKind::Removed,
        &atom(1,),
    ));
}

#[test]
fn preserved_fact_is_not_misclassified_as_added_or_removed_effect() {
    let episodes = vec![
        transition(&[1, 9], &[1, 2, 9], atom(100)),
        transition(&[1, 9], &[1, 2, 9], atom(100)),
        transition(&[1, 9], &[1, 9], atom(200)),
    ];

    let result = TransitionSchemaInduction::induce(&episodes, &[], policy(1, 1, 1, 64, 32));

    assert!(!has_schema(
        &result,
        &atom(100,),
        TransitionEffectKind::Added,
        &atom(1,),
    ));

    assert!(!has_schema(
        &result,
        &atom(100,),
        TransitionEffectKind::Removed,
        &atom(1,),
    ));
}

#[test]
fn failed_expected_effect_is_retained_as_explicit_schema_counterexample() {
    let transformation = atom(100);

    let episodes = vec![
        transition(&[9], &[9, 1], transformation.clone()),
        transition(&[9], &[9, 1], transformation.clone()),
        transition(&[9], &[9], transformation.clone()),
        transition(&[9], &[9], atom(200)),
        transition(&[9], &[9], atom(200)),
        transition(&[9], &[9], atom(200)),
    ];

    let result = TransitionSchemaInduction::induce(&episodes, &[], policy(2, 600, 300, 64, 16));

    let schema = result
        .selected()
        .iter()
        .find(|schema| {
            schema.transformation() == &transformation
                && schema.effect_kind() == TransitionEffectKind::Added
                && schema.fact() == &atom(1)
        })
        .unwrap();

    assert_eq!(schema.support_count(), 2);

    assert_eq!(schema.transformation_opportunity_count(), 3);

    assert_eq!(schema.counterexample_count(), 1);

    assert!(schema.is_counterexample(&episodes[2],));
}

#[test]
fn effect_common_to_every_transformation_has_zero_lift_and_is_not_schema() {
    let episodes = vec![
        transition(&[9], &[9, 1], atom(100)),
        transition(&[9], &[9, 1], atom(100)),
        transition(&[9], &[9, 1], atom(200)),
        transition(&[9], &[9, 1], atom(200)),
    ];

    let result = TransitionSchemaInduction::induce(&episodes, &[], policy(1, 1, 1, 64, 32));

    assert_eq!(result.selected_count(), 0);
}

#[test]
fn one_transformation_can_support_multiple_distinct_effect_schemas() {
    let transformation = atom(100);

    let episodes = vec![
        transition(&[1, 9], &[2, 9], transformation.clone()),
        transition(&[1, 9], &[2, 9], transformation.clone()),
        transition(&[1, 9], &[1, 9], atom(200)),
        transition(&[1, 9], &[1, 9], atom(200)),
    ];

    let result = TransitionSchemaInduction::induce(&episodes, &[], policy(2, 1000, 400, 64, 32));

    assert!(has_schema(
        &result,
        &transformation,
        TransitionEffectKind::Removed,
        &atom(1,),
    ));

    assert!(has_schema(
        &result,
        &transformation,
        TransitionEffectKind::Added,
        &atom(2,),
    ));
}

#[test]
fn exact_transformation_structure_identity_keeps_reordered_transformations_distinct() {
    let first = ordered(&[10, 20]);

    let second = ordered(&[20, 10]);

    let episodes = vec![
        transition(&[9], &[9, 1], first.clone()),
        transition(&[9], &[9, 1], first.clone()),
        transition(&[9], &[9], second.clone()),
        transition(&[9], &[9], second.clone()),
    ];

    let result = TransitionSchemaInduction::induce(&episodes, &[], policy(2, 1000, 400, 64, 16));

    assert_ne!(first, second);

    assert!(has_schema(
        &result,
        &first,
        TransitionEffectKind::Added,
        &atom(1,),
    ));

    assert!(!has_schema(
        &result,
        &second,
        TransitionEffectKind::Added,
        &atom(1,),
    ));
}

#[test]
fn invariant_evidence_deprioritizes_stable_facts_under_hard_candidate_budget_without_blocking_them()
{
    let invariants = invariant_seeds();

    assert!(invariants
        .iter()
        .any(|candidate| { candidate.fact() == &atom(1,) },));

    assert!(invariants
        .iter()
        .all(|candidate| { candidate.fact() != &atom(2,) },));

    let episodes = vec![
        transition(&[1, 2, 9], &[9], atom(500)),
        transition(&[1, 2, 9], &[9], atom(500)),
        transition(&[1, 2, 9], &[1, 2, 9], atom(600)),
        transition(&[1, 2, 9], &[1, 2, 9], atom(600)),
    ];

    let bounded =
        TransitionSchemaInduction::induce(&episodes, &invariants, policy(2, 1000, 400, 1, 16));

    assert_eq!(bounded.evaluated_candidate_count(), 1);

    assert!(bounded.candidate_generation_truncated());

    assert!(bounded.invariant_seeded_fact_count() >= 1);

    assert!(has_schema(
        &bounded,
        &atom(500,),
        TransitionEffectKind::Removed,
        &atom(2,),
    ));

    let unbounded =
        TransitionSchemaInduction::induce(&episodes, &invariants, policy(2, 1000, 400, 64, 16));

    assert!(has_schema(
        &unbounded,
        &atom(500,),
        TransitionEffectKind::Removed,
        &atom(1,),
    ));
}

#[test]
fn hard_schema_frontier_prefers_stronger_effect_and_is_episode_order_invariant() {
    let original = vec![
        transition(&[9], &[9, 1, 2], atom(100)),
        transition(&[9], &[9, 1, 2], atom(100)),
        transition(&[9], &[9, 1], atom(100)),
        transition(&[9], &[9], atom(200)),
        transition(&[9], &[9], atom(200)),
        transition(&[9], &[9], atom(200)),
    ];

    let mut reversed = original.clone();

    reversed.reverse();

    let induction_policy = policy(2, 600, 300, 128, 1);

    let first = TransitionSchemaInduction::induce(&original, &[], induction_policy);

    let second = TransitionSchemaInduction::induce(&reversed, &[], induction_policy);

    assert_eq!(first, second);

    assert_eq!(first.selected_count(), 1);

    assert!(first.admitted_before_frontier() > first.selected_count());

    assert_eq!(first.selected()[0].fact(), &atom(1,));

    assert_eq!(first.selected()[0].precision().value(), 1000);
}

#[test]
fn transition_schema_induction_is_deterministic_non_mutating_and_facade_equivalent() {
    let episodes = vec![
        transition(&[1, 9], &[2, 9], atom(100)),
        transition(&[1, 9], &[2, 9], atom(100)),
        transition(&[1, 9], &[1, 9], atom(200)),
        transition(&[1, 9], &[1, 9], atom(200)),
    ];

    let invariants = invariant_seeds();

    let episodes_before = episodes.clone();

    let invariants_before = invariants.clone();

    let induction_policy = policy(1, 1, 1, 128, 32);

    let direct = TransitionSchemaInduction::induce(&episodes, &invariants, induction_policy);

    let facade =
        UniversalTransitionSchemaInduction::evaluate(&episodes, &invariants, induction_policy);

    let repeated =
        UniversalTransitionSchemaInduction::evaluate(&episodes, &invariants, induction_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, episodes_before);

    assert_eq!(invariants, invariants_before);

    assert_eq!(facade.transformation_count(), 2);

    assert!(facade.vocabulary_fact_count() >= 3);

    assert!(facade.possible_effect_candidate_count() >= facade.evaluated_candidate_count());
}
