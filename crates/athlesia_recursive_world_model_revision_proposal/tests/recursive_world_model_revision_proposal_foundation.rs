use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::AbstractionUnit;

use athlesia_recursive::RecursiveUnit;

use athlesia_recursive_world_model::RecursiveWorldRule;

use athlesia_recursive_world_model_revision_proposal::{
    RecursiveWorldRevisionProposal, RecursiveWorldRevisionProposalSet,
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

#[test]
fn exact_noop_proposal_is_rejected() {
    let source = rule(&[1], &[2]);

    assert!(RecursiveWorldRevisionProposal::new(source.clone(), source,).is_none());
}

#[test]
fn distinct_replacement_is_accepted() {
    let proposal = RecursiveWorldRevisionProposal::new(rule(&[1], &[2]), rule(&[1], &[3]));

    assert!(proposal.is_some());
}

#[test]
fn proposal_preserves_target_identity() {
    let target = rule(&[1], &[2]);

    let proposal = RecursiveWorldRevisionProposal::new(target.clone(), rule(&[1], &[3])).unwrap();

    assert_eq!(proposal.target(), &target);
}

#[test]
fn proposal_preserves_replacement_identity() {
    let replacement = rule(&[1], &[3]);

    let proposal =
        RecursiveWorldRevisionProposal::new(rule(&[1], &[2]), replacement.clone()).unwrap();

    assert_eq!(proposal.replacement(), &replacement);
}

#[test]
fn premise_change_is_detected() {
    let proposal = RecursiveWorldRevisionProposal::new(rule(&[1], &[2]), rule(&[3], &[2])).unwrap();

    assert!(proposal.changes_premises());

    assert!(!proposal.changes_conclusions());
}

#[test]
fn conclusion_change_is_detected() {
    let proposal = RecursiveWorldRevisionProposal::new(rule(&[1], &[2]), rule(&[1], &[3])).unwrap();

    assert!(!proposal.changes_premises());

    assert!(proposal.changes_conclusions());
}

#[test]
fn empty_proposal_set_is_empty() {
    let set = RecursiveWorldRevisionProposalSet::new(Vec::new());

    assert!(set.is_empty());

    assert_eq!(set.len(), 0);
}

#[test]
fn proposal_set_preserves_distinct_proposals() {
    let first = RecursiveWorldRevisionProposal::new(rule(&[1], &[2]), rule(&[1], &[3])).unwrap();

    let second = RecursiveWorldRevisionProposal::new(rule(&[5], &[6]), rule(&[5], &[7])).unwrap();

    let set = RecursiveWorldRevisionProposalSet::new(vec![first.clone(), second.clone()]);

    assert_eq!(set.len(), 2);

    assert!(set.contains(&first,));

    assert!(set.contains(&second,));
}

#[test]
fn exact_duplicate_proposals_are_deduplicated() {
    let proposal = RecursiveWorldRevisionProposal::new(rule(&[1], &[2]), rule(&[1], &[3])).unwrap();

    let set = RecursiveWorldRevisionProposalSet::new(vec![proposal.clone(), proposal]);

    assert_eq!(set.len(), 1);
}

#[test]
fn target_scoped_query_is_exact() {
    let first_target = rule(&[1], &[2]);

    let second_target = rule(&[5], &[6]);

    let first =
        RecursiveWorldRevisionProposal::new(first_target.clone(), rule(&[1], &[3])).unwrap();

    let second =
        RecursiveWorldRevisionProposal::new(second_target.clone(), rule(&[5], &[7])).unwrap();

    let set = RecursiveWorldRevisionProposalSet::new(vec![second, first.clone()]);

    assert_eq!(set.proposals_for_target(&first_target,), vec![first,]);

    assert!(set.proposals_for_target(&rule(&[8], &[9],),).is_empty());
}

#[test]
fn proposal_set_is_deterministic_under_input_order() {
    let first = RecursiveWorldRevisionProposal::new(rule(&[1], &[2]), rule(&[1], &[3])).unwrap();

    let second = RecursiveWorldRevisionProposal::new(rule(&[5], &[6]), rule(&[5], &[7])).unwrap();

    assert_eq!(
        RecursiveWorldRevisionProposalSet::new(vec![first.clone(), second.clone(),],),
        RecursiveWorldRevisionProposalSet::new(vec![second, first,],)
    );
}

#[test]
fn proposal_set_construction_does_not_mutate_source_vector() {
    let source = vec![
        RecursiveWorldRevisionProposal::new(rule(&[1], &[2]), rule(&[1], &[3])).unwrap(),
        RecursiveWorldRevisionProposal::new(rule(&[5], &[6]), rule(&[5], &[7])).unwrap(),
    ];

    let before = source.clone();

    let _ = RecursiveWorldRevisionProposalSet::new(source.clone());

    assert_eq!(source, before);
}
