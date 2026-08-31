use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryHypothesis, RecursiveWorldRevisionDiscoveryHypothesisSet,
    RecursiveWorldRevisionDiscoveryObservation,
};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
    )
}

fn unit(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(AbstractionUnit::Structural(structural(span)))
}

fn rule(premises: &[usize], conclusions: &[usize]) -> RecursiveWorldRule {
    RecursiveWorldRule::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn observation(
    premises: &[usize],
    conclusions: &[usize],
) -> RecursiveWorldRevisionDiscoveryObservation {
    RecursiveWorldRevisionDiscoveryObservation::new(
        premises.iter().copied().map(unit).collect(),
        conclusions.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

fn hypothesis(
    target: RecursiveWorldRule,
    premises: &[usize],
    conclusions: &[usize],
) -> RecursiveWorldRevisionDiscoveryHypothesis {
    RecursiveWorldRevisionDiscoveryHypothesis::discover(target, observation(premises, conclusions))
        .unwrap()
}

#[test]
fn observation_rejects_empty_premises() {
    assert!(
        RecursiveWorldRevisionDiscoveryObservation::new(Vec::new(), vec![unit(2,),],).is_none()
    );
}

#[test]
fn observation_rejects_empty_conclusions() {
    assert!(
        RecursiveWorldRevisionDiscoveryObservation::new(vec![unit(1,),], Vec::new(),).is_none()
    );
}

#[test]
fn observation_canonicalizes_and_deduplicates_structure() {
    let observed = RecursiveWorldRevisionDiscoveryObservation::new(
        vec![unit(2), unit(1), unit(2)],
        vec![unit(4), unit(3), unit(4)],
    )
    .unwrap();

    assert_eq!(observed.premises(), &[unit(1,), unit(2,),]);

    assert_eq!(observed.conclusions(), &[unit(3,), unit(4,),]);

    assert_eq!(observed.premise_count(), 2);

    assert_eq!(observed.conclusion_count(), 2);
}

#[test]
fn observation_materializes_exact_world_rule() {
    let observed = observation(&[1, 2], &[3, 4]);

    assert_eq!(observed.materialize_rule(), rule(&[1, 2], &[3, 4],));
}

#[test]
fn discovery_rejects_observation_identical_to_target() {
    let target = rule(&[1], &[2]);

    assert!(
        RecursiveWorldRevisionDiscoveryHypothesis::discover(target, observation(&[1], &[2],),)
            .is_none()
    );
}

#[test]
fn discovery_materializes_replacement_without_replacement_input() {
    let target = rule(&[1], &[2]);

    let discovered =
        RecursiveWorldRevisionDiscoveryHypothesis::discover(target, observation(&[1], &[3]))
            .unwrap();

    assert_eq!(discovered.replacement(), &rule(&[1], &[3],));
}

#[test]
fn discovery_preserves_target_identity() {
    let target = rule(&[1], &[2]);

    let discovered = hypothesis(target.clone(), &[1], &[3]);

    assert_eq!(discovered.target(), &target);
}

#[test]
fn discovery_preserves_observation_identity() {
    let target = rule(&[1], &[2]);

    let observed = observation(&[1], &[3]);

    let discovered =
        RecursiveWorldRevisionDiscoveryHypothesis::discover(target, observed.clone()).unwrap();

    assert_eq!(discovered.observation(), &observed);
}

#[test]
fn discovery_reports_structural_change_identity() {
    let premise_change = hypothesis(rule(&[1], &[2]), &[3], &[2]);

    assert!(premise_change.changes_premises());

    assert!(!premise_change.changes_conclusions());

    let conclusion_change = hypothesis(rule(&[1], &[2]), &[1], &[3]);

    assert!(!conclusion_change.changes_premises());

    assert!(conclusion_change.changes_conclusions());
}

#[test]
fn exact_duplicate_hypotheses_are_deduplicated() {
    let discovered = hypothesis(rule(&[1], &[2]), &[1], &[3]);

    let set =
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone(), discovered]);

    assert_eq!(set.len(), 1);
}

#[test]
fn target_scoped_hypothesis_query_is_exact() {
    let first_target = rule(&[1], &[2]);

    let second_target = rule(&[5], &[6]);

    let first = hypothesis(first_target.clone(), &[1], &[3]);

    let second = hypothesis(second_target.clone(), &[5], &[7]);

    let set = RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![second, first.clone()]);

    assert_eq!(set.hypotheses_for_target(&first_target,), vec![first,]);

    assert!(set.hypotheses_for_target(&rule(&[8], &[9],),).is_empty());
}

#[test]
fn hypothesis_set_is_deterministic_and_non_mutating() {
    let first = hypothesis(rule(&[1], &[2]), &[1], &[3]);

    let second = hypothesis(rule(&[5], &[6]), &[5], &[7]);

    let source = vec![second.clone(), first.clone()];

    let before = source.clone();

    let left = RecursiveWorldRevisionDiscoveryHypothesisSet::new(source.clone());

    let right = RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![first, second]);

    assert_eq!(left, right);

    assert_eq!(source, before);
}
