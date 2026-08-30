use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_revision::{
    RecursiveEvidenceState, RecursiveRevisionMemory, RecursiveRevisionPolicy,
    RecursiveRevisionStatus,
};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        6,
    )
}

fn hierarchy(spans: &[usize]) -> HierarchicalConcept {
    HierarchicalConcept::new(spans.iter().copied().map(structural).collect()).unwrap()
}

fn structural_unit(span: usize) -> AbstractionUnit {
    AbstractionUnit::Structural(structural(span))
}

fn hierarchical_unit(spans: &[usize]) -> AbstractionUnit {
    AbstractionUnit::Hierarchical(hierarchy(spans))
}

fn cross_level(structural_span: usize, hierarchy_spans: &[usize]) -> CrossLevelConcept {
    CrossLevelConcept::new(vec![
        structural_unit(structural_span),
        hierarchical_unit(hierarchy_spans),
    ])
    .unwrap()
}

fn base(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(structural_unit(span))
}

fn cross(structural_span: usize, hierarchy_spans: &[usize]) -> RecursiveUnit {
    RecursiveUnit::CrossLevel(cross_level(structural_span, hierarchy_spans))
}

fn concept(base_span: usize, cross_span: usize) -> RecursiveConcept {
    RecursiveConcept::new(vec![
        base(base_span),
        cross(cross_span, &[cross_span + 1, cross_span + 2]),
    ])
    .unwrap()
}

#[test]
fn unobserved_evidence_is_unsupported() {
    let evidence = RecursiveEvidenceState::new();

    assert_eq!(
        RecursiveRevisionPolicy::new().classify(&evidence,),
        RecursiveRevisionStatus::Unsupported
    );
}

#[test]
fn violation_only_evidence_is_weakened() {
    let mut evidence = RecursiveEvidenceState::new();

    evidence.violate();

    assert_eq!(
        RecursiveRevisionPolicy::new().classify(&evidence,),
        RecursiveRevisionStatus::Weakened
    );
}

#[test]
fn mixed_evidence_is_contested() {
    let mut evidence = RecursiveEvidenceState::new();

    evidence.confirm();
    evidence.violate();

    assert_eq!(
        RecursiveRevisionPolicy::new().classify(&evidence,),
        RecursiveRevisionStatus::Contested
    );
}

#[test]
fn confirmation_only_evidence_is_supported() {
    let mut evidence = RecursiveEvidenceState::new();

    evidence.confirm();

    assert_eq!(
        RecursiveRevisionPolicy::new().classify(&evidence,),
        RecursiveRevisionStatus::Supported
    );
}

#[test]
fn repeated_confirmations_remain_supported() {
    let mut evidence = RecursiveEvidenceState::new();

    evidence.confirm();
    evidence.confirm();
    evidence.confirm();

    assert_eq!(
        RecursiveRevisionPolicy::new().classify(&evidence,),
        RecursiveRevisionStatus::Supported
    );
}

#[test]
fn repeated_violations_remain_weakened() {
    let mut evidence = RecursiveEvidenceState::new();

    evidence.violate();
    evidence.violate();
    evidence.violate();

    assert_eq!(
        RecursiveRevisionPolicy::new().classify(&evidence,),
        RecursiveRevisionStatus::Weakened
    );
}

#[test]
fn any_mixed_history_is_contested() {
    let mut evidence = RecursiveEvidenceState::new();

    evidence.confirm();
    evidence.confirm();
    evidence.confirm();
    evidence.violate();

    assert_eq!(
        RecursiveRevisionPolicy::new().classify(&evidence,),
        RecursiveRevisionStatus::Contested
    );
}

#[test]
fn absent_concept_assessment_is_unsupported() {
    let target = concept(1, 2);

    let memory = RecursiveRevisionMemory::new();

    let assessment = memory.assessment(&target);

    assert_eq!(assessment.concept(), &target);

    assert_eq!(assessment.status(), RecursiveRevisionStatus::Unsupported);

    assert_eq!(assessment.observations(), 0);
}

#[test]
fn assessment_preserves_recursive_identity() {
    let target = concept(1, 2);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(target.clone());

    let assessment = memory.assessment(&target);

    assert_eq!(assessment.concept(), &target);

    assert_eq!(assessment.confirmations(), 1);

    assert_eq!(assessment.violations(), 0);
}

#[test]
fn supported_models_rank_above_contested_models() {
    let supported = concept(1, 2);

    let contested = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(supported.clone());

    memory.confirm(contested.clone());

    memory.violate(contested);

    let ranked = memory.ranked_assessments();

    assert_eq!(ranked[0].concept(), &supported);

    assert_eq!(ranked[0].status(), RecursiveRevisionStatus::Supported);
}

#[test]
fn contested_models_rank_above_weakened_models() {
    let contested = concept(1, 2);

    let weakened = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(contested.clone());

    memory.violate(contested.clone());

    memory.violate(weakened);

    let ranked = memory.ranked_assessments();

    assert_eq!(ranked[0].concept(), &contested);

    assert_eq!(ranked[0].status(), RecursiveRevisionStatus::Contested);
}

#[test]
fn same_status_uses_balance_then_confirmation_evidence() {
    let stronger = concept(1, 2);

    let weaker = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(stronger.clone());

    memory.confirm(stronger.clone());

    memory.confirm(weaker.clone());

    let ranked = memory.ranked_assessments();

    assert_eq!(ranked[0].concept(), &stronger);

    assert_eq!(ranked[0].balance(), 2);
}

#[test]
fn exact_policy_tie_uses_recursive_identity() {
    let first = concept(1, 2);

    let second = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(second.clone());

    memory.confirm(first.clone());

    let ranked = memory.ranked_assessments();

    let expected = if first < second { first } else { second };

    assert_eq!(ranked[0].concept(), &expected);
}

#[test]
fn recursive_depth_identity_remains_separate_in_policy() {
    let child = concept(1, 2);

    let shallow = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(child.clone())),
    ])
    .unwrap();

    let middle =
        RecursiveConcept::new(vec![base(6), RecursiveUnit::Recursive(Box::new(child))]).unwrap();

    let deep =
        RecursiveConcept::new(vec![base(5), RecursiveUnit::Recursive(Box::new(middle))]).unwrap();

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(shallow.clone());

    memory.violate(deep.clone());

    assert_eq!(
        memory.assessment(&shallow,).status(),
        RecursiveRevisionStatus::Supported
    );

    assert_eq!(
        memory.assessment(&deep,).status(),
        RecursiveRevisionStatus::Weakened
    );
}

#[test]
fn ranking_is_deterministic_and_non_mutating() {
    let first = concept(1, 2);

    let second = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second.clone());

    memory.violate(second);

    let before = memory.clone();

    let first_run = memory.ranked_assessments();

    let second_run = memory.ranked_assessments();

    assert_eq!(first_run, second_run);

    assert_eq!(memory, before);
}
