use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_revision::{
    RecursiveCompetingModels, RecursiveDiscriminativeExperimentSelector, RecursiveEvidenceLoop,
    RecursiveExperimentObservation, RecursiveRevisionMemory,
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

fn competing_pair() -> (RecursiveRevisionMemory, RecursiveConcept, RecursiveConcept) {
    let shared = cross(3, &[4, 5]);

    let first = concept(base(1), shared.clone());

    let second = concept(base(2), shared);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first.clone());

    memory.confirm(second.clone());

    (memory, first, second)
}

#[test]
fn present_target_confirms_supporting_model() {
    let (mut memory, first, _) = competing_pair();

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &base(1))
        .unwrap();

    RecursiveEvidenceLoop::new().apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Present,
    );

    assert_eq!(memory.evidence(&first,).unwrap().confirmations(), 2);

    assert_eq!(memory.evidence(&first,).unwrap().violations(), 0);
}

#[test]
fn absent_target_violates_supporting_model() {
    let (mut memory, first, _) = competing_pair();

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &base(1))
        .unwrap();

    RecursiveEvidenceLoop::new().apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Absent,
    );

    assert_eq!(memory.evidence(&first,).unwrap().confirmations(), 1);

    assert_eq!(memory.evidence(&first,).unwrap().violations(), 1);
}

#[test]
fn non_supporting_model_remains_neutral_on_presence() {
    let (mut memory, _, second) = competing_pair();

    let before = *memory.evidence(&second).unwrap();

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &base(1))
        .unwrap();

    RecursiveEvidenceLoop::new().apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Present,
    );

    assert_eq!(memory.evidence(&second,).unwrap(), &before);
}

#[test]
fn non_supporting_model_remains_neutral_on_absence() {
    let (mut memory, _, second) = competing_pair();

    let before = *memory.evidence(&second).unwrap();

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &base(1))
        .unwrap();

    RecursiveEvidenceLoop::new().apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Absent,
    );

    assert_eq!(memory.evidence(&second,).unwrap(), &before);
}

#[test]
fn all_supporters_are_updated_once() {
    let target = cross(9, &[10, 11]);

    let first = concept(base(1), target.clone());

    let second = concept(base(2), target.clone());

    let third = concept(base(3), cross(12, &[13, 14]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first.clone());

    memory.confirm(second.clone());

    memory.confirm(third);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &target)
        .unwrap();

    let result = RecursiveEvidenceLoop::new().apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Present,
    );

    assert_eq!(result.len(), 2);

    assert_eq!(memory.evidence(&first,).unwrap().confirmations(), 2);

    assert_eq!(memory.evidence(&second,).unwrap().confirmations(), 2);
}

#[test]
fn update_records_before_and_after_evidence() {
    let (mut memory, first, _) = competing_pair();

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &base(1))
        .unwrap();

    let result = RecursiveEvidenceLoop::new().apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Present,
    );

    assert_eq!(result.len(), 1);

    let update = &result.updates()[0];

    assert_eq!(update.concept(), &first);

    assert_eq!(update.before().confirmations(), 1);

    assert_eq!(update.after().confirmations(), 2);
}

#[test]
fn recursive_target_presence_confirms_supporter() {
    let nested = concept(base(1), cross(2, &[3, 4]));

    let recursive_target = RecursiveUnit::Recursive(Box::new(nested));

    let supporter = concept(base(5), recursive_target.clone());

    let opponent = concept(base(5), cross(6, &[7, 8]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(supporter.clone());

    memory.confirm(opponent);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &recursive_target)
        .unwrap();

    RecursiveEvidenceLoop::new().apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Present,
    );

    assert_eq!(memory.evidence(&supporter,).unwrap().confirmations(), 2);
}

#[test]
fn recursive_target_absence_violates_supporter() {
    let nested = concept(base(1), cross(2, &[3, 4]));

    let recursive_target = RecursiveUnit::Recursive(Box::new(nested));

    let supporter = concept(base(5), recursive_target.clone());

    let opponent = concept(base(5), cross(6, &[7, 8]));

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(supporter.clone());

    memory.confirm(opponent);

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &recursive_target)
        .unwrap();

    RecursiveEvidenceLoop::new().apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Absent,
    );

    assert_eq!(memory.evidence(&supporter,).unwrap().violations(), 1);
}

#[test]
fn recursive_depth_identity_is_not_flattened() {
    let level_one = concept(base(1), cross(2, &[3, 4]));

    let level_two = RecursiveConcept::new(vec![
        base(5),
        RecursiveUnit::Recursive(Box::new(level_one.clone())),
    ])
    .unwrap();

    let shallow = RecursiveUnit::Recursive(Box::new(level_one));

    let deep = RecursiveUnit::Recursive(Box::new(level_two));

    let shallow_model = concept(base(6), shallow.clone());

    let deep_model = concept(base(6), deep.clone());

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(shallow_model.clone());

    memory.confirm(deep_model.clone());

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &deep)
        .unwrap();

    RecursiveEvidenceLoop::new().apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Present,
    );

    assert_eq!(memory.evidence(&deep_model,).unwrap().confirmations(), 2);

    assert_eq!(memory.evidence(&shallow_model,).unwrap().confirmations(), 1);
}

#[test]
fn repeated_present_observations_accumulate() {
    let (mut memory, first, _) = competing_pair();

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &base(1))
        .unwrap();

    let loop_engine = RecursiveEvidenceLoop::new();

    loop_engine.apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Present,
    );

    loop_engine.apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Present,
    );

    assert_eq!(memory.evidence(&first,).unwrap().confirmations(), 3);
}

#[test]
fn repeated_absent_observations_accumulate() {
    let (mut memory, first, _) = competing_pair();

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &base(1))
        .unwrap();

    let loop_engine = RecursiveEvidenceLoop::new();

    loop_engine.apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Absent,
    );

    loop_engine.apply(
        &mut memory,
        &models,
        &experiment,
        RecursiveExperimentObservation::Absent,
    );

    assert_eq!(memory.evidence(&first,).unwrap().violations(), 2);
}

#[test]
fn evidence_loop_is_deterministic() {
    let (memory, _, _) = competing_pair();

    let models = RecursiveCompetingModels::from_memory(&memory);

    let experiment = RecursiveDiscriminativeExperimentSelector::new()
        .generate(&models)
        .into_iter()
        .find(|experiment| experiment.target() == &base(1))
        .unwrap();

    let mut left = memory.clone();

    let mut right = memory.clone();

    let left_result = RecursiveEvidenceLoop::new().apply(
        &mut left,
        &models,
        &experiment,
        RecursiveExperimentObservation::Absent,
    );

    let right_result = RecursiveEvidenceLoop::new().apply(
        &mut right,
        &models,
        &experiment,
        RecursiveExperimentObservation::Absent,
    );

    assert_eq!(left_result, right_result);

    assert_eq!(left, right);
}
