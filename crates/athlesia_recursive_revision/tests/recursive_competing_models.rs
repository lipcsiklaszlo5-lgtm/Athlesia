use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_revision::{
    RecursiveCompetingModels, RecursiveRevisionMemory, RecursiveRevisionStatus,
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
fn empty_memory_produces_empty_competition() {
    let memory = RecursiveRevisionMemory::new();

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert!(models.is_empty());

    assert_eq!(models.len(), 0);

    assert!(models.best().is_none());

    assert!(models.runner_up().is_none());
}

#[test]
fn one_model_becomes_best() {
    let target = concept(1, 2);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(target.clone());

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.len(), 1);

    assert_eq!(models.best().unwrap().concept(), &target);
}

#[test]
fn supported_model_beats_contested_model() {
    let supported = concept(1, 2);

    let contested = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(supported.clone());

    memory.confirm(contested.clone());

    memory.violate(contested);

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.best().unwrap().concept(), &supported);
}

#[test]
fn contested_model_beats_weakened_model() {
    let contested = concept(1, 2);

    let weakened = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(contested.clone());

    memory.violate(contested.clone());

    memory.violate(weakened);

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.best().unwrap().concept(), &contested);
}

#[test]
fn supported_count_is_explicit() {
    let first = concept(1, 2);

    let second = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.supported_count(), 2);

    assert_eq!(models.contested_count(), 0);

    assert_eq!(models.weakened_count(), 0);
}

#[test]
fn contested_count_is_explicit() {
    let target = concept(1, 2);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(target.clone());

    memory.violate(target);

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.contested_count(), 1);
}

#[test]
fn weakened_count_is_explicit() {
    let target = concept(1, 2);

    let mut memory = RecursiveRevisionMemory::new();

    memory.violate(target);

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.weakened_count(), 1);
}

#[test]
fn runner_up_is_second_ranked_model() {
    let best = concept(1, 2);

    let second = concept(5, 6);

    let third = concept(9, 10);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(best.clone());

    memory.confirm(second.clone());

    memory.violate(second.clone());

    memory.violate(third);

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.best().unwrap().concept(), &best);

    assert_eq!(models.runner_up().unwrap().concept(), &second);
}

#[test]
fn stronger_supported_model_wins_within_status() {
    let stronger = concept(1, 2);

    let weaker = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(stronger.clone());

    memory.confirm(stronger.clone());

    memory.confirm(weaker);

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.best().unwrap().concept(), &stronger);

    assert_eq!(models.best().unwrap().confirmations(), 2);
}

#[test]
fn exact_competition_tie_uses_recursive_identity() {
    let first = concept(1, 2);

    let second = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(second.clone());

    memory.confirm(first.clone());

    let models = RecursiveCompetingModels::from_memory(&memory);

    let expected = if first < second { first } else { second };

    assert_eq!(models.best().unwrap().concept(), &expected);
}

#[test]
fn recursive_depth_remains_distinct_in_competition() {
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

    memory.confirm(deep.clone());

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.len(), 2);

    assert!(models
        .models()
        .iter()
        .any(|model| { model.concept() == &shallow }));

    assert!(models
        .models()
        .iter()
        .any(|model| { model.concept() == &deep }));
}

#[test]
fn model_assessments_preserve_status_and_evidence() {
    let target = concept(1, 2);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(target.clone());

    memory.confirm(target.clone());

    let models = RecursiveCompetingModels::from_memory(&memory);

    let assessment = models.best().unwrap();

    assert_eq!(assessment.status(), RecursiveRevisionStatus::Supported);

    assert_eq!(assessment.confirmations(), 2);

    assert_eq!(assessment.violations(), 0);
}

#[test]
fn construction_deduplicates_same_recursive_model() {
    let target = concept(1, 2);

    let memory = RecursiveRevisionMemory::new();

    let assessment = memory.assessment(&target);

    let models = RecursiveCompetingModels::new(vec![assessment.clone(), assessment]);

    assert_eq!(models.len(), 1);

    assert_eq!(models.unsupported_count(), 1);
}

#[test]
fn competition_is_deterministic_and_non_mutating() {
    let first = concept(1, 2);

    let second = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second.clone());

    memory.violate(second);

    let before = memory.clone();

    let first_run = RecursiveCompetingModels::from_memory(&memory);

    let second_run = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(first_run, second_run);

    assert_eq!(memory, before);
}
