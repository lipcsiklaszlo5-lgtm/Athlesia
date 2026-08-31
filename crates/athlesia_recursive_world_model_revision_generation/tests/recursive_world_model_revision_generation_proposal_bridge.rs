use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;
use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_generation::{
    RecursiveWorldRevisionGenerationCandidate, RecursiveWorldRevisionGenerationCandidateSet,
    RecursiveWorldRevisionGenerationProposalBridge,
    RecursiveWorldRevisionGenerationProposalBridgeBuilder,
};

use athlesia_recursive_world_model_revision_proposal::RecursiveWorldRevisionProposal;

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

fn candidate(
    target: RecursiveWorldRule,
    replacement: RecursiveWorldRule,
    basis: &[usize],
) -> RecursiveWorldRevisionGenerationCandidate {
    RecursiveWorldRevisionGenerationCandidate::new(
        target,
        replacement,
        basis.iter().copied().map(unit).collect(),
    )
    .unwrap()
}

#[test]
fn empty_candidate_set_produces_empty_proposal_set() {
    let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(Vec::new()),
    );

    assert_eq!(bridge.candidate_count(), 0);

    assert_eq!(bridge.proposal_count(), 0);

    assert!(bridge.proposals().is_empty());
}

#[test]
fn single_candidate_materializes_single_proposal() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let generated = candidate(target.clone(), replacement.clone(), &[9]);

    let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated]),
    );

    assert_eq!(bridge.proposal_count(), 1);

    assert_eq!(bridge.proposals().proposals()[0].target(), &target);

    assert_eq!(
        bridge.proposals().proposals()[0].replacement(),
        &replacement
    );
}

#[test]
fn proposal_preserves_target_identity() {
    let target = rule(&[1], &[2]);

    let generated = candidate(target.clone(), rule(&[1], &[3]), &[9]);

    let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated]),
    );

    assert_eq!(bridge.proposals().proposals()[0].target(), &target);
}

#[test]
fn proposal_preserves_replacement_identity() {
    let replacement = rule(&[1], &[3]);

    let generated = candidate(rule(&[1], &[2]), replacement.clone(), &[9]);

    let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated]),
    );

    assert_eq!(
        bridge.proposals().proposals()[0].replacement(),
        &replacement
    );
}

#[test]
fn distinct_proposal_identities_are_preserved() {
    let first = candidate(rule(&[1], &[2]), rule(&[1], &[3]), &[9]);

    let second = candidate(rule(&[5], &[6]), rule(&[5], &[7]), &[10]);

    let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![first, second]),
    );

    assert_eq!(bridge.candidate_count(), 2);

    assert_eq!(bridge.proposal_count(), 2);
}

#[test]
fn distinct_basis_same_rules_collapse_to_one_proposal() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![
            candidate(target.clone(), replacement.clone(), &[9]),
            candidate(target, replacement, &[10]),
        ]),
    );

    assert_eq!(bridge.candidate_count(), 2);

    assert_eq!(bridge.proposal_count(), 1);
}

#[test]
fn proposal_provenance_returns_all_matching_candidates() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let first = candidate(target.clone(), replacement.clone(), &[9]);

    let second = candidate(target.clone(), replacement.clone(), &[10]);

    let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![first.clone(), second.clone()]),
    );

    let proposal = RecursiveWorldRevisionProposal::new(target, replacement).unwrap();

    assert_eq!(
        bridge.candidates_for_proposal(&proposal,),
        vec![first, second,]
    );
}

#[test]
fn unrelated_candidate_is_excluded_from_proposal_provenance() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let matching = candidate(target.clone(), replacement.clone(), &[9]);

    let unrelated = candidate(rule(&[5], &[6]), rule(&[5], &[7]), &[10]);

    let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![matching.clone(), unrelated]),
    );

    let proposal = RecursiveWorldRevisionProposal::new(target, replacement).unwrap();

    assert_eq!(bridge.candidates_for_proposal(&proposal,), vec![matching,]);
}

#[test]
fn candidate_maps_back_to_exact_proposal_identity() {
    let target = rule(&[1], &[2]);

    let replacement = rule(&[1], &[3]);

    let generated = candidate(target.clone(), replacement.clone(), &[9]);

    let bridge = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![generated.clone()]),
    );

    let proposal = bridge.proposal_for_candidate(&generated).unwrap();

    assert_eq!(proposal.target(), &target);

    assert_eq!(proposal.replacement(), &replacement);
}

#[test]
fn bridge_builder_matches_direct_construction() {
    let set = RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
        rule(&[1], &[2]),
        rule(&[1], &[3]),
        &[9],
    )]);

    assert_eq!(
        RecursiveWorldRevisionGenerationProposalBridgeBuilder::build(set.clone(),),
        RecursiveWorldRevisionGenerationProposalBridge::new(set,)
    );
}

#[test]
fn bridge_is_deterministic_under_candidate_order() {
    let first = candidate(rule(&[1], &[2]), rule(&[1], &[3]), &[9]);

    let second = candidate(rule(&[5], &[6]), rule(&[5], &[7]), &[10]);

    let left = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![first.clone(), second.clone()]),
    );

    let right = RecursiveWorldRevisionGenerationProposalBridge::new(
        RecursiveWorldRevisionGenerationCandidateSet::new(vec![second, first]),
    );

    assert_eq!(left, right);
}

#[test]
fn bridge_does_not_mutate_source_candidate_set() {
    let set = RecursiveWorldRevisionGenerationCandidateSet::new(vec![candidate(
        rule(&[1], &[2]),
        rule(&[1], &[3]),
        &[9],
    )]);

    let before = set.clone();

    let _ = RecursiveWorldRevisionGenerationProposalBridge::new(set.clone());

    assert_eq!(set, before);
}
