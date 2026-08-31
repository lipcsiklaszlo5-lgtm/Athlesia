use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_discovery::{
    RecursiveWorldRevisionDiscoveryGenerationBridge,
    RecursiveWorldRevisionDiscoveryGenerationBridgeBuilder,
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
fn empty_hypothesis_set_produces_empty_candidate_set() {
    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(Vec::new()),
    );

    assert_eq!(bridge.hypothesis_count(), 0);

    assert_eq!(bridge.candidate_count(), 0);

    assert!(bridge.candidates().is_empty());
}

#[test]
fn single_hypothesis_materializes_single_generation_candidate() {
    let target = rule(&[1], &[2]);

    let discovered = hypothesis(target.clone(), &[1], &[3]);

    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered]),
    );

    assert_eq!(bridge.candidate_count(), 1);

    assert_eq!(bridge.candidates().candidates()[0].target(), &target);

    assert_eq!(
        bridge.candidates().candidates()[0].replacement(),
        &rule(&[1], &[3],)
    );
}

#[test]
fn bridge_preserves_discovered_target_identity() {
    let target = rule(&[1], &[2]);

    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
            target.clone(),
            &[1],
            &[3],
        )]),
    );

    assert_eq!(bridge.candidates().candidates()[0].target(), &target);
}

#[test]
fn bridge_preserves_discovered_replacement_identity() {
    let replacement = rule(&[1], &[3]);

    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
            rule(&[1], &[2]),
            &[1],
            &[3],
        )]),
    );

    assert_eq!(
        bridge.candidates().candidates()[0].replacement(),
        &replacement
    );
}

#[test]
fn observation_structure_becomes_generation_basis() {
    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
            rule(&[1], &[2]),
            &[3, 1],
            &[4, 2],
        )]),
    );

    assert_eq!(
        bridge.candidates().candidates()[0].basis(),
        &[unit(1,), unit(2,), unit(3,), unit(4,),]
    );
}

#[test]
fn overlapping_observation_units_are_deduplicated_in_basis() {
    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
            rule(&[1], &[2]),
            &[1, 2],
            &[2, 3],
        )]),
    );

    assert_eq!(bridge.candidates().candidates()[0].basis_count(), 3);

    assert_eq!(
        bridge.candidates().candidates()[0].basis(),
        &[unit(1,), unit(2,), unit(3,),]
    );
}

#[test]
fn multiple_hypotheses_materialize_distinct_candidates() {
    let first = hypothesis(rule(&[1], &[2]), &[1], &[3]);

    let second = hypothesis(rule(&[5], &[6]), &[5], &[7]);

    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![first, second]),
    );

    assert_eq!(bridge.hypothesis_count(), 2);

    assert_eq!(bridge.candidate_count(), 2);
}

#[test]
fn direct_hypothesis_to_candidate_mapping_is_exact() {
    let discovered = hypothesis(rule(&[1], &[2]), &[1], &[3]);

    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    let candidate = bridge.candidate_for_hypothesis(&discovered).unwrap();

    assert_eq!(candidate, bridge.candidates().candidates()[0]);
}

#[test]
fn reverse_candidate_provenance_is_preserved() {
    let discovered = hypothesis(rule(&[1], &[2]), &[1], &[3]);

    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![discovered.clone()]),
    );

    let candidate = bridge.candidates().candidates()[0].clone();

    assert_eq!(
        bridge.hypotheses_for_candidate(&candidate,),
        vec![discovered,]
    );
}

#[test]
fn unrelated_hypothesis_is_excluded_from_candidate_provenance() {
    let first = hypothesis(rule(&[1], &[2]), &[1], &[3]);

    let second = hypothesis(rule(&[5], &[6]), &[5], &[7]);

    let bridge = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![first.clone(), second]),
    );

    let candidate = bridge.candidate_for_hypothesis(&first).unwrap();

    assert_eq!(bridge.hypotheses_for_candidate(&candidate,), vec![first,]);
}

#[test]
fn bridge_builder_matches_direct_construction() {
    let set = RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![hypothesis(
        rule(&[1], &[2]),
        &[1],
        &[3],
    )]);

    assert_eq!(
        RecursiveWorldRevisionDiscoveryGenerationBridgeBuilder::build(set.clone(),),
        RecursiveWorldRevisionDiscoveryGenerationBridge::new(set,)
    );
}

#[test]
fn bridge_is_deterministic_and_non_mutating() {
    let first = hypothesis(rule(&[1], &[2]), &[1], &[3]);

    let second = hypothesis(rule(&[5], &[6]), &[5], &[7]);

    let source =
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![second.clone(), first.clone()]);

    let before = source.clone();

    let left = RecursiveWorldRevisionDiscoveryGenerationBridge::new(source.clone());

    let right = RecursiveWorldRevisionDiscoveryGenerationBridge::new(
        RecursiveWorldRevisionDiscoveryHypothesisSet::new(vec![first, second]),
    );

    assert_eq!(left, right);

    assert_eq!(source, before);
}
