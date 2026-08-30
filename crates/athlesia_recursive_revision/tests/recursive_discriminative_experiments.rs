use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_revision::{
    RecursiveCompetingModels, RecursiveDiscriminativeExperimentSelector, RecursiveRevisionMemory,
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

fn concept(first: RecursiveUnit, second: RecursiveUnit) -> RecursiveConcept {
    RecursiveConcept::new(vec![first, second]).unwrap()
}

#[test]
fn empty_competition_produces_no_experiment() {
    let models = RecursiveCompetingModels::default();

    assert!(RecursiveDiscriminativeExperimentSelector::new()
        .select(&models,)
        .is_none());
}

#[test]
fn single_model_produces_no_experiment() {
    let target = concept(base(1), cross(2, &[3, 4]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(target);

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert!(RecursiveDiscriminativeExperimentSelector::new()
        .select(&models,)
        .is_none());
}

#[test]
fn shared_units_are_not_discriminative() {
    let shared = cross(3, &[4, 5]);

    let first = concept(base(1), shared.clone());

    let second = concept(base(2), shared);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiments = RecursiveDiscriminativeExperimentSelector::new().generate(&models);

    assert!(experiments
        .iter()
        .all(|experiment| { experiment.target() != &cross(3, &[4, 5],) }));
}

#[test]
fn unique_base_unit_can_discriminate_models() {
    let first = concept(base(1), cross(3, &[4, 5]));

    let second = concept(base(2), cross(3, &[4, 5]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiments = RecursiveDiscriminativeExperimentSelector::new().generate(&models);

    assert_eq!(experiments.len(), 2);

    assert!(experiments
        .iter()
        .any(|experiment| { experiment.target() == &base(1) }));

    assert!(experiments
        .iter()
        .any(|experiment| { experiment.target() == &base(2) }));
}

#[test]
fn unique_cross_level_unit_can_discriminate_models() {
    let first = concept(base(1), cross(3, &[4, 5]));

    let second = concept(base(1), cross(6, &[7, 8]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiments = RecursiveDiscriminativeExperimentSelector::new().generate(&models);

    assert_eq!(experiments.len(), 2);
}

#[test]
fn balanced_split_has_expected_score() {
    let shared = cross(9, &[10, 11]);

    let first = concept(base(1), shared.clone());

    let second = concept(base(1), cross(3, &[4, 5]));

    let third = concept(base(2), shared);

    let fourth = concept(base(2), cross(6, &[7, 8]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    memory.confirm(third);

    memory.confirm(fourth);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let selected = RecursiveDiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    assert_eq!(selected.discrimination_score(), 2);

    assert_eq!(selected.supporters(), 2);

    assert_eq!(selected.opponents(), 2);
}

#[test]
fn selector_prefers_more_balanced_partition() {
    let shared_three = cross(20, &[21, 22]);

    let shared_two = cross(30, &[31, 32]);

    let models_raw = vec![
        concept(base(1), shared_three.clone()),
        concept(base(2), shared_three.clone()),
        concept(base(3), shared_three),
        concept(base(4), shared_two.clone()),
        concept(base(5), shared_two),
    ];

    let mut memory = RecursiveRevisionMemory::new();

    for model in models_raw {
        memory.confirm(model);
    }

    let models = RecursiveCompetingModels::from_memory(&memory);

    let selected = RecursiveDiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    assert_eq!(selected.discrimination_score(), 2);
}

#[test]
fn exact_discrimination_tie_uses_support_then_identity() {
    let shared = cross(3, &[4, 5]);

    let first = concept(base(1), shared.clone());

    let second = concept(base(2), shared);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let selected = RecursiveDiscriminativeExperimentSelector::new()
        .select(&models)
        .unwrap();

    let expected = if base(1) < base(2) { base(1) } else { base(2) };

    assert_eq!(selected.target(), &expected);
}

#[test]
fn recursive_unit_can_be_discriminative_target() {
    let nested = concept(base(1), cross(2, &[3, 4]));

    let recursive = RecursiveUnit::Recursive(Box::new(nested));

    let first = concept(base(5), recursive.clone());

    let second = concept(base(5), cross(6, &[7, 8]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiments = RecursiveDiscriminativeExperimentSelector::new().generate(&models);

    assert!(experiments
        .iter()
        .any(|experiment| { experiment.target() == &recursive }));
}

#[test]
fn recursive_depth_identity_is_preserved() {
    let level_one = concept(base(1), cross(2, &[3, 4]));

    let level_two = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(level_one.clone())),
    ])
    .unwrap();

    let shallow = RecursiveUnit::Recursive(Box::new(level_one));

    let deep = RecursiveUnit::Recursive(Box::new(level_two));

    let first = concept(base(6), shallow.clone());

    let second = concept(base(6), deep.clone());

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiments = RecursiveDiscriminativeExperimentSelector::new().generate(&models);

    assert!(experiments
        .iter()
        .any(|experiment| { experiment.target() == &shallow }));

    assert!(experiments
        .iter()
        .any(|experiment| { experiment.target() == &deep }));
}

#[test]
fn experiment_generation_ignores_revision_status() {
    let first = concept(base(1), cross(3, &[4, 5]));

    let second = concept(base(2), cross(3, &[4, 5]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second.clone());

    memory.violate(second);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiments = RecursiveDiscriminativeExperimentSelector::new().generate(&models);

    assert_eq!(experiments.len(), 2);
}

#[test]
fn experiments_preserve_partition_counts() {
    let first = concept(base(1), cross(3, &[4, 5]));

    let second = concept(base(2), cross(3, &[4, 5]));

    let third = concept(base(1), cross(6, &[7, 8]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    memory.confirm(third);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiments = RecursiveDiscriminativeExperimentSelector::new().generate(&models);

    for experiment in experiments {
        assert_eq!(
            experiment.supporters() + experiment.opponents(),
            models.len()
        );
    }
}

#[test]
fn generation_is_deterministic_and_non_mutating() {
    let first = concept(base(1), cross(3, &[4, 5]));

    let second = concept(base(2), cross(3, &[4, 5]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first);

    memory.confirm(second);

    let before = memory.clone();

    let models = RecursiveCompetingModels::from_memory(&memory);

    let selector = RecursiveDiscriminativeExperimentSelector::new();

    let first_run = selector.generate(&models);

    let second_run = selector.generate(&models);

    assert_eq!(first_run, second_run);

    assert_eq!(memory, before);
}
