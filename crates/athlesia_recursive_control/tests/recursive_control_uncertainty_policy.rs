use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_control::{RecursiveControlObjective, RecursiveControlUncertaintyPolicy};

use athlesia_recursive_planning::{
    RecursivePlanningGoal, RecursivePlanningMemory, RecursivePlanningState,
    RecursivePlanningTransition,
};

use athlesia_recursive_revision::{RecursiveCompetingModels, RecursiveRevisionMemory};

fn structural(span: usize) -> StructuralConcept {
    StructuralConcept::with_sequence_length(
        vec![PrimitiveSignature::new(RelationKind::Equal, span)],
        8,
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

fn state(units: Vec<RecursiveUnit>) -> RecursivePlanningState {
    RecursivePlanningState::new(units)
}

fn goal(units: Vec<RecursiveUnit>) -> RecursivePlanningGoal {
    RecursivePlanningGoal::new(units).unwrap()
}

fn objective(model: RecursiveConcept, target_span: usize) -> RecursiveControlObjective {
    RecursiveControlObjective::new(model, goal(vec![base(target_span)]))
}

fn insert(
    memory: &mut RecursivePlanningMemory,
    from: RecursivePlanningState,
    to: RecursivePlanningState,
) {
    memory.insert(RecursivePlanningTransition::new(from, to).unwrap());
}

fn contested_models() -> (RecursiveCompetingModels, RecursiveConcept, RecursiveConcept) {
    let first = concept(1, 2);
    let second = concept(1, 5);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(first.clone());
    revision.violate(first.clone());

    revision.confirm(second.clone());
    revision.violate(second.clone());

    (
        RecursiveCompetingModels::from_memory(&revision),
        first,
        second,
    )
}

#[test]
fn empty_models_produce_no_decision() {
    let models = RecursiveCompetingModels::from_memory(&RecursiveRevisionMemory::new());

    let result = RecursiveControlUncertaintyPolicy::new().decide(
        &RecursivePlanningMemory::new(),
        &models,
        &state(vec![base(1)]),
        &[],
    );

    assert!(result.is_no_decision());
}

#[test]
fn supported_best_model_allows_action() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(model.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let result = RecursiveControlUncertaintyPolicy::new().decide(
        &planning,
        &models,
        &start,
        &[objective(model, 20)],
    );

    assert!(result.is_act());

    assert!(result.control().is_some());
}

#[test]
fn supported_model_without_reachable_objective_has_no_decision() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(model.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let result = RecursiveControlUncertaintyPolicy::new().decide(
        &RecursivePlanningMemory::new(),
        &models,
        &state(vec![base(10)]),
        &[objective(model, 99)],
    );

    assert!(result.is_no_decision());
}

#[test]
fn contested_best_model_requests_experiment() {
    let (models, _, _) = contested_models();

    let result = RecursiveControlUncertaintyPolicy::new().decide(
        &RecursivePlanningMemory::new(),
        &models,
        &state(vec![base(10)]),
        &[],
    );

    assert!(result.is_experiment());

    assert!(result.experiment().is_some());
}

#[test]
fn weakened_best_model_does_not_act() {
    let first = concept(1, 2);
    let second = concept(1, 5);

    let mut revision = RecursiveRevisionMemory::new();

    revision.violate(first);
    revision.violate(second);

    let models = RecursiveCompetingModels::from_memory(&revision);

    let result = RecursiveControlUncertaintyPolicy::new().decide(
        &RecursivePlanningMemory::new(),
        &models,
        &state(vec![base(10)]),
        &[],
    );

    assert!(!result.is_act());
}

#[test]
fn contested_model_does_not_act_even_when_goal_is_reachable() {
    let (models, first, _) = contested_models();

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let result = RecursiveControlUncertaintyPolicy::new().decide(
        &planning,
        &models,
        &start,
        &[objective(first, 20)],
    );

    assert!(!result.is_act());

    assert!(result.is_experiment());
}

#[test]
fn uncertain_models_without_discriminator_return_no_decision() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.violate(model);

    let models = RecursiveCompetingModels::from_memory(&revision);

    let result = RecursiveControlUncertaintyPolicy::new().decide(
        &RecursivePlanningMemory::new(),
        &models,
        &state(vec![base(10)]),
        &[],
    );

    assert!(result.is_no_decision());
}

#[test]
fn action_preserves_selected_control_objective() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(model.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let target = objective(model, 20);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let result = RecursiveControlUncertaintyPolicy::new().decide(
        &planning,
        &models,
        &start,
        std::slice::from_ref(&target),
    );

    assert_eq!(result.control().unwrap().objective(), &target);
}

#[test]
fn uncertainty_policy_reacts_to_revision_change() {
    let first = concept(1, 2);
    let second = concept(1, 5);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(first.clone());
    revision.confirm(first.clone());
    revision.confirm(second.clone());

    let before_models = RecursiveCompetingModels::from_memory(&revision);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let before = RecursiveControlUncertaintyPolicy::new().decide(
        &planning,
        &before_models,
        &start,
        &[objective(first.clone(), 20), objective(second, 30)],
    );

    assert!(before.is_act());

    revision.violate(first);

    let after_models = RecursiveCompetingModels::from_memory(&revision);

    let after =
        RecursiveControlUncertaintyPolicy::new().decide(&planning, &after_models, &start, &[]);

    assert!(!after.is_act());
}

#[test]
fn experiment_selection_is_deterministic() {
    let (models, _, _) = contested_models();

    let policy = RecursiveControlUncertaintyPolicy::new();

    let start = state(vec![base(10)]);

    let planning = RecursivePlanningMemory::new();

    let left = policy.decide(&planning, &models, &start, &[]);

    let right = policy.decide(&planning, &models, &start, &[]);

    assert_eq!(left, right);
}

#[test]
fn uncertainty_policy_does_not_mutate_inputs() {
    let (models, _, _) = contested_models();

    let planning = RecursivePlanningMemory::new();

    let start = state(vec![base(10)]);

    let models_before = models.clone();

    let planning_before = planning.clone();

    let start_before = start.clone();

    let _ = RecursiveControlUncertaintyPolicy::new().decide(&planning, &models, &start, &[]);

    assert_eq!(models, models_before);

    assert_eq!(planning, planning_before);

    assert_eq!(start, start_before);
}

#[test]
fn uncertainty_policy_preserves_recursive_identity() {
    let shallow = concept(1, 2);

    let deep = RecursiveConcept::new(vec![
        base(8),
        RecursiveUnit::Recursive(Box::new(shallow.clone())),
    ])
    .unwrap();

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(deep.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let result = RecursiveControlUncertaintyPolicy::new().decide(
        &planning,
        &models,
        &start,
        &[objective(shallow, 30), objective(deep.clone(), 20)],
    );

    assert_eq!(result.control().unwrap().objective().model(), &deep);
}
