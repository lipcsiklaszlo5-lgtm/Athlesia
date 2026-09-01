use athlesia_mindstone_sparse_cognition::{CognitiveSignal, CognitiveStructure};
use athlesia_universal_domain_learning::{
    ContextPremiseSet, ContextualTransitionEvidenceThresholds, ContextualTransitionRuleInduction,
    ContextualTransitionRulePolicy, GroundedStateSnapshot, GroundedTransformationEpisode,
    GroundedTransitionSchemaHypothesis, TransitionEffectKind, TransitionSchemaInduction,
    TransitionSchemaPolicy, UniversalContextualTransitionRuleInduction, MAX_CONTEXT_PREMISES,
};

#[derive(Clone, Copy)]
struct SearchBounds {
    max_context_premises: usize,
    max_contexts: usize,
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

fn ordered(values: &[u64]) -> CognitiveStructure {
    CognitiveStructure::ordered(values.iter().copied().map(atom).collect()).unwrap()
}

fn snapshot(facts: &[u64]) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts.iter().copied().map(atom).collect()).unwrap()
}

fn structured_snapshot(facts: Vec<CognitiveStructure>) -> GroundedStateSnapshot {
    GroundedStateSnapshot::new(facts).unwrap()
}

fn transition(
    before: &[u64],
    after: &[u64],
    transformation: CognitiveStructure,
) -> GroundedTransformationEpisode {
    GroundedTransformationEpisode::new(snapshot(before), snapshot(after), transformation)
}

fn structured_transition(
    before: Vec<CognitiveStructure>,
    after: Vec<CognitiveStructure>,
    transformation: CognitiveStructure,
) -> GroundedTransformationEpisode {
    GroundedTransformationEpisode::new(
        structured_snapshot(before),
        structured_snapshot(after),
        transformation,
    )
}

fn search_bounds(
    max_context_premises: usize,
    max_contexts: usize,
    max_evaluations: usize,
    max_rules: usize,
) -> SearchBounds {
    SearchBounds {
        max_context_premises,
        max_contexts,
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

fn policy(bounds: SearchBounds, evidence: EvidenceSpec) -> ContextualTransitionRulePolicy {
    let thresholds = ContextualTransitionEvidenceThresholds::new(
        evidence.support,
        signal(evidence.precision),
        signal(evidence.lift),
        signal(evidence.incremental_gain),
    )
    .unwrap();

    ContextualTransitionRulePolicy::new(
        bounds.max_context_premises,
        bounds.max_contexts,
        bounds.max_evaluations,
        bounds.max_rules,
        thresholds,
    )
    .unwrap()
}

fn schema_seeds() -> Vec<GroundedTransitionSchemaHypothesis> {
    let episodes = vec![
        transition(&[9], &[5, 9], atom(100)),
        transition(&[9], &[5, 9], atom(100)),
        transition(&[9], &[9], atom(200)),
        transition(&[9], &[9], atom(200)),
    ];

    let schema_policy = TransitionSchemaPolicy::new(2, signal(1000), signal(400), 64, 16).unwrap();

    TransitionSchemaInduction::induce(&episodes, &[], schema_policy)
        .selected()
        .to_vec()
}

fn has_rule(
    result: &athlesia_universal_domain_learning::ContextualTransitionRuleResult,
    transformation: &CognitiveStructure,
    context: &ContextPremiseSet,
    kind: TransitionEffectKind,
    fact: &CognitiveStructure,
) -> bool {
    result.selected().iter().any(|rule| {
        rule.transformation() == transformation
            && rule.context() == context
            && rule.effect_kind() == kind
            && rule.effect_fact() == fact
    })
}

#[test]
fn context_premise_set_is_nonempty_bounded_and_canonical() {
    assert_eq!(ContextPremiseSet::new(Vec::new(),), None);

    assert_eq!(
        ContextPremiseSet::new(
            (0..=MAX_CONTEXT_PREMISES)
                .map(|value| { atom(value as u64,) },)
                .collect(),
        ),
        None
    );

    let first = ContextPremiseSet::new(vec![atom(2), atom(1), atom(2)]).unwrap();

    let second = ContextPremiseSet::new(vec![atom(1), atom(2)]).unwrap();

    assert_eq!(first, second);

    assert_eq!(first.premise_count(), 2);
}

#[test]
fn contextual_policy_requires_positive_evidence_and_hard_search_bounds() {
    assert_eq!(
        ContextualTransitionEvidenceThresholds::new(0, signal(900,), signal(100,), signal(100,),),
        None
    );

    assert_eq!(
        ContextualTransitionEvidenceThresholds::new(1, signal(900,), signal(0,), signal(100,),),
        None
    );

    assert_eq!(
        ContextualTransitionEvidenceThresholds::new(1, signal(900,), signal(100,), signal(0,),),
        None
    );

    let thresholds =
        ContextualTransitionEvidenceThresholds::new(1, signal(900), signal(100), signal(100))
            .unwrap();

    assert_eq!(
        ContextualTransitionRulePolicy::new(0, 10, 10, 10, thresholds,),
        None
    );

    assert_eq!(
        ContextualTransitionRulePolicy::new(MAX_CONTEXT_PREMISES + 1, 10, 10, 10, thresholds,),
        None
    );

    assert_eq!(
        ContextualTransitionRulePolicy::new(1, 0, 10, 10, thresholds,),
        None
    );

    assert!(ContextualTransitionRulePolicy::new(1, 10, 10, 10, thresholds,).is_some());
}

#[test]
fn context_can_explain_effect_when_transformation_alone_is_ambiguous() {
    let transformation = atom(100);

    let context = ContextPremiseSet::new(vec![atom(1)]).unwrap();

    let episodes = vec![
        transition(&[1, 9], &[1, 5, 9], transformation.clone()),
        transition(&[1, 9], &[1, 5, 9], transformation.clone()),
        transition(&[2, 9], &[2, 9], transformation.clone()),
        transition(&[2, 9], &[2, 9], transformation.clone()),
    ];

    let result = ContextualTransitionRuleInduction::induce(
        &episodes,
        &[],
        policy(
            search_bounds(1, 32, 256, 16),
            evidence_spec(2, 1000, 400, 400),
        ),
    );

    assert!(has_rule(
        &result,
        &transformation,
        &context,
        TransitionEffectKind::Added,
        &atom(5,),
    ));

    let rule = result
        .selected()
        .iter()
        .find(|rule| rule.context() == &context && rule.effect_fact() == &atom(5))
        .unwrap();

    assert_eq!(rule.precision().value(), 1000);

    assert_eq!(rule.transformation_precision().value(), 500);

    assert_eq!(rule.incremental_precision_gain().value(), 500);
}

#[test]
fn redundant_context_is_rejected_when_transformation_alone_is_equally_predictive() {
    let transformation = atom(100);

    let context = ContextPremiseSet::new(vec![atom(1)]).unwrap();

    let episodes = vec![
        transition(&[1, 9], &[1, 5, 9], transformation.clone()),
        transition(&[1, 9], &[1, 5, 9], transformation.clone()),
        transition(&[2, 9], &[2, 5, 9], transformation.clone()),
        transition(&[2, 9], &[2, 5, 9], transformation.clone()),
    ];

    let result = ContextualTransitionRuleInduction::induce(
        &episodes,
        &[],
        policy(search_bounds(1, 32, 256, 16), evidence_spec(2, 900, 1, 1)),
    );

    assert!(!has_rule(
        &result,
        &transformation,
        &context,
        TransitionEffectKind::Added,
        &atom(5,),
    ));
}

#[test]
fn failed_contextual_effect_is_retained_as_explicit_counterexample() {
    let transformation = atom(100);

    let context = ContextPremiseSet::new(vec![atom(1)]).unwrap();

    let episodes = vec![
        transition(&[1, 9], &[1, 5, 9], transformation.clone()),
        transition(&[1, 9], &[1, 5, 9], transformation.clone()),
        transition(&[1, 9], &[1, 9], transformation.clone()),
        transition(&[2, 9], &[2, 9], transformation.clone()),
    ];

    let result = ContextualTransitionRuleInduction::induce(
        &episodes,
        &[],
        policy(
            search_bounds(1, 32, 256, 16),
            evidence_spec(2, 600, 100, 100),
        ),
    );

    let rule = result
        .selected()
        .iter()
        .find(|rule| rule.context() == &context && rule.effect_fact() == &atom(5))
        .unwrap();

    assert_eq!(rule.support_count(), 2);

    assert_eq!(rule.context_opportunity_count(), 3);

    assert_eq!(rule.counterexample_count(), 1);

    assert!(rule.is_counterexample(&episodes[2],));
}

#[test]
fn conjunctive_context_can_explain_effect_when_single_context_facts_are_insufficient() {
    let transformation = atom(100);

    let conjunction = ContextPremiseSet::new(vec![atom(1), atom(2)]).unwrap();

    let episodes = vec![
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 9], transformation.clone()),
        transition(&[2, 4, 9], &[2, 4, 9], transformation.clone()),
        transition(&[8, 9], &[8, 9], transformation.clone()),
    ];

    let result = ContextualTransitionRuleInduction::induce(
        &episodes,
        &[],
        policy(
            search_bounds(2, 128, 2048, 32),
            evidence_spec(2, 900, 500, 500),
        ),
    );

    assert!(has_rule(
        &result,
        &transformation,
        &conjunction,
        TransitionEffectKind::Added,
        &atom(5,),
    ));

    let rule = result
        .selected()
        .iter()
        .find(|rule| rule.context() == &conjunction)
        .unwrap();

    assert_eq!(rule.precision().value(), 1000);

    assert_eq!(rule.transformation_precision().value(), 400);

    assert_eq!(rule.incremental_precision_gain().value(), 600);
}

#[test]
fn maximum_context_arity_prevents_hidden_conjunctive_context_search() {
    let transformation = atom(100);

    let conjunction = ContextPremiseSet::new(vec![atom(1), atom(2)]).unwrap();

    let episodes = vec![
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 2, 9], &[1, 2, 5, 9], transformation.clone()),
        transition(&[1, 3, 9], &[1, 3, 9], transformation.clone()),
        transition(&[2, 4, 9], &[2, 4, 9], transformation.clone()),
        transition(&[8, 9], &[8, 9], transformation.clone()),
    ];

    let result = ContextualTransitionRuleInduction::induce(
        &episodes,
        &[],
        policy(
            search_bounds(1, 128, 2048, 32),
            evidence_spec(2, 900, 500, 500),
        ),
    );

    assert!(!has_rule(
        &result,
        &transformation,
        &conjunction,
        TransitionEffectKind::Added,
        &atom(5,),
    ));

    assert!(result
        .selected()
        .iter()
        .all(|rule| { rule.context().premise_count() <= 1 },));
}

#[test]
fn exact_transformation_and_context_structure_identity_remain_semantic_authority() {
    let transformation = atom(100);

    let context_a = ordered(&[1, 2]);

    let context_b = ordered(&[2, 1]);

    let context_set_a = ContextPremiseSet::new(vec![context_a.clone()]).unwrap();

    let episodes = vec![
        structured_transition(
            vec![context_a.clone(), atom(9)],
            vec![context_a.clone(), atom(5), atom(9)],
            transformation.clone(),
        ),
        structured_transition(
            vec![context_a.clone(), atom(9)],
            vec![context_a.clone(), atom(5), atom(9)],
            transformation.clone(),
        ),
        structured_transition(
            vec![context_b.clone(), atom(9)],
            vec![context_b.clone(), atom(9)],
            transformation.clone(),
        ),
        structured_transition(
            vec![context_b.clone(), atom(9)],
            vec![context_b.clone(), atom(9)],
            transformation.clone(),
        ),
    ];

    let result = ContextualTransitionRuleInduction::induce(
        &episodes,
        &[],
        policy(
            search_bounds(1, 32, 256, 16),
            evidence_spec(2, 1000, 400, 400),
        ),
    );

    assert_ne!(context_a, context_b);

    assert!(has_rule(
        &result,
        &transformation,
        &context_set_a,
        TransitionEffectKind::Added,
        &atom(5,),
    ));
}

#[test]
fn transition_schema_seeds_prioritize_effect_targets_without_becoming_semantic_gate() {
    let schemas = schema_seeds();

    assert!(schemas
        .iter()
        .any(|schema| { schema.transformation() == &atom(100,) && schema.fact() == &atom(5,) },));

    let episodes = vec![
        transition(&[1, 9], &[1, 5, 6, 9], atom(100)),
        transition(&[1, 9], &[1, 5, 6, 9], atom(100)),
        transition(&[2, 9], &[2, 9], atom(100)),
        transition(&[2, 9], &[2, 9], atom(100)),
    ];

    let bounded = ContextualTransitionRuleInduction::induce(
        &episodes,
        &schemas,
        policy(search_bounds(1, 1, 1, 8), evidence_spec(2, 1000, 400, 400)),
    );

    assert_eq!(bounded.evaluated_rule_candidate_count(), 1);

    assert!(bounded.rule_evaluation_truncated());

    assert!(bounded.schema_seeded_effect_target_count() >= 1);

    assert!(bounded
        .selected()
        .iter()
        .any(|rule| { rule.effect_fact() == &atom(5,) },));

    let unseeded = ContextualTransitionRuleInduction::induce(
        &episodes,
        &[],
        policy(
            search_bounds(1, 32, 256, 16),
            evidence_spec(2, 1000, 400, 400),
        ),
    );

    assert!(unseeded
        .selected()
        .iter()
        .any(|rule| { rule.effect_fact() == &atom(6,) },));
}

#[test]
fn hard_contextual_rule_evaluation_budget_is_reported_and_never_exceeded() {
    let episodes = vec![
        transition(&[1, 2, 9], &[1, 2, 5, 6, 9], atom(100)),
        transition(&[1, 3, 9], &[1, 3, 7, 9], atom(200)),
    ];

    let result = ContextualTransitionRuleInduction::induce(
        &episodes,
        &[],
        policy(search_bounds(2, 64, 1, 32), evidence_spec(1, 1, 1, 1)),
    );

    assert_eq!(result.evaluated_rule_candidate_count(), 1);

    assert!(result.rule_evaluation_truncated());

    assert!(result.possible_rule_evaluation_count() >= result.evaluated_rule_candidate_count());
}

#[test]
fn hard_contextual_rule_frontier_prefers_stronger_condition_and_is_episode_order_invariant() {
    let original = vec![
        transition(&[1, 9], &[1, 5, 9], atom(100)),
        transition(&[1, 9], &[1, 5, 9], atom(100)),
        transition(&[1, 9], &[1, 5, 9], atom(100)),
        transition(&[2, 9], &[2, 6, 9], atom(100)),
        transition(&[2, 9], &[2, 6, 9], atom(100)),
        transition(&[2, 9], &[2, 9], atom(100)),
        transition(&[8, 9], &[8, 9], atom(100)),
    ];

    let mut reversed = original.clone();

    reversed.reverse();

    let induction_policy = policy(
        search_bounds(1, 32, 512, 1),
        evidence_spec(2, 600, 300, 300),
    );

    let first = ContextualTransitionRuleInduction::induce(&original, &[], induction_policy);

    let second = ContextualTransitionRuleInduction::induce(&reversed, &[], induction_policy);

    assert_eq!(first, second);

    assert_eq!(first.selected_count(), 1);

    assert!(first.admitted_before_frontier() > first.selected_count());

    assert_eq!(first.selected()[0].effect_fact(), &atom(5,));

    assert_eq!(first.selected()[0].precision().value(), 1000);
}

#[test]
fn contextual_transition_rule_induction_is_deterministic_non_mutating_and_facade_equivalent() {
    let episodes = vec![
        transition(&[1, 9], &[1, 5, 9], atom(100)),
        transition(&[1, 9], &[1, 5, 9], atom(100)),
        transition(&[2, 9], &[2, 9], atom(100)),
        transition(&[2, 9], &[2, 9], atom(100)),
    ];

    let schemas = schema_seeds();

    let episodes_before = episodes.clone();

    let schemas_before = schemas.clone();

    let induction_policy = policy(search_bounds(2, 64, 512, 32), evidence_spec(1, 1, 1, 1));

    let direct = ContextualTransitionRuleInduction::induce(&episodes, &schemas, induction_policy);

    let facade =
        UniversalContextualTransitionRuleInduction::evaluate(&episodes, &schemas, induction_policy);

    let repeated =
        UniversalContextualTransitionRuleInduction::evaluate(&episodes, &schemas, induction_policy);

    assert_eq!(direct, facade);

    assert_eq!(facade, repeated);

    assert_eq!(episodes, episodes_before);

    assert_eq!(schemas, schemas_before);

    assert!(facade.vocabulary_fact_count() >= 3);

    assert!(facade.effect_target_count() >= 1);

    assert!(facade.possible_context_count() >= facade.generated_context_count());
}
