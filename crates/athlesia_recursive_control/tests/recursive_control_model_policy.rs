use athlesia::{PrimitiveSignature, RelationKind, StructuralConcept};

use athlesia_cross_level::{AbstractionUnit, CrossLevelConcept};

use athlesia_hierarchy::HierarchicalConcept;

use athlesia_recursive::{RecursiveConcept, RecursiveUnit};

use athlesia_recursive_control::{RecursiveControlModelPolicy, RecursiveControlObjective};

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

fn cross_level(span: usize, hierarchy_spans: &[usize]) -> CrossLevelConcept {
    CrossLevelConcept::new(vec![
        structural_unit(span),
        hierarchical_unit(hierarchy_spans),
    ])
    .unwrap()
}

fn base(span: usize) -> RecursiveUnit {
    RecursiveUnit::Base(structural_unit(span))
}

fn cross(span: usize, hierarchy_spans: &[usize]) -> RecursiveUnit {
    RecursiveUnit::CrossLevel(cross_level(span, hierarchy_spans))
}

fn concept(base_span: usize, cross_span: usize) -> RecursiveConcept {
    RecursiveConcept::new(vec![
        base(base_span),
        cross(cross_span, &[cross_span + 1, cross_span + 2]),
    ])
    .unwrap()
}

fn deep_concept() -> RecursiveConcept {
    RecursiveConcept::new(vec![
        base(8),
        RecursiveUnit::Recursive(Box::new(concept(1, 2))),
    ])
    .unwrap()
}

fn state(units: Vec<RecursiveUnit>) -> RecursivePlanningState {
    RecursivePlanningState::new(units)
}

fn goal(units: Vec<RecursiveUnit>) -> RecursivePlanningGoal {
    RecursivePlanningGoal::new(units).unwrap()
}

fn insert(
    memory: &mut RecursivePlanningMemory,
    from: RecursivePlanningState,
    to: RecursivePlanningState,
) {
    memory.insert(RecursivePlanningTransition::new(from, to).unwrap());
}

fn ranked(first: &RecursiveConcept, second: &RecursiveConcept) -> RecursiveCompetingModels {
    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first.clone());
    memory.confirm(first.clone());
    memory.confirm(second.clone());

    RecursiveCompetingModels::from_memory(&memory)
}

#[test]
fn empty_model_set_selects_no_objective() {
    let models = RecursiveCompetingModels::from_memory(&RecursiveRevisionMemory::new());

    let objective = RecursiveControlObjective::new(concept(1, 2), goal(vec![base(9)]));

    assert!(RecursiveControlModelPolicy::new()
        .select_objective(&models, &[objective],)
        .is_none());
}

#[test]
fn best_ranked_model_selects_its_objective() {
    let best = concept(1, 2);
    let weaker = concept(5, 6);

    let models = ranked(&best, &weaker);

    let expected = RecursiveControlObjective::new(best.clone(), goal(vec![base(20)]));

    let other = RecursiveControlObjective::new(weaker, goal(vec![base(30)]));

    let selected = RecursiveControlModelPolicy::new()
        .select_objective(&models, &[other, expected.clone()])
        .unwrap();

    assert_eq!(selected, expected);
    assert_eq!(selected.model(), &best);
}

#[test]
fn stronger_evidence_controls_model_priority() {
    let stronger = concept(1, 2);
    let weaker = concept(5, 6);

    let models = ranked(&stronger, &weaker);

    assert_eq!(models.best().unwrap().concept(), &stronger);
}

#[test]
fn violation_can_change_selected_objective() {
    let first = concept(1, 2);
    let second = concept(5, 6);

    let mut memory = RecursiveRevisionMemory::new();

    memory.confirm(first.clone());
    memory.confirm(second.clone());
    memory.confirm(second.clone());

    assert_eq!(
        RecursiveCompetingModels::from_memory(&memory)
            .best()
            .unwrap()
            .concept(),
        &second
    );

    memory.violate(second.clone());

    let models = RecursiveCompetingModels::from_memory(&memory);

    assert_eq!(models.best().unwrap().concept(), &first);

    let expected = RecursiveControlObjective::new(first, goal(vec![base(20)]));

    let other = RecursiveControlObjective::new(second, goal(vec![base(30)]));

    assert_eq!(
        RecursiveControlModelPolicy::new()
            .select_objective(&models, &[other, expected.clone(),],)
            .unwrap(),
        expected
    );
}

#[test]
fn missing_objective_for_best_model_returns_none() {
    let best = concept(1, 2);
    let weaker = concept(5, 6);

    let models = ranked(&best, &weaker);

    let only_weaker = RecursiveControlObjective::new(weaker, goal(vec![base(30)]));

    assert!(RecursiveControlModelPolicy::new()
        .select_objective(&models, &[only_weaker],)
        .is_none());
}

#[test]
fn equal_model_objectives_use_goal_identity_tie_break() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(model.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let first_goal = goal(vec![base(20)]);
    let second_goal = goal(vec![base(30)]);

    let first = RecursiveControlObjective::new(model.clone(), first_goal.clone());

    let second = RecursiveControlObjective::new(model, second_goal.clone());

    let selected = RecursiveControlModelPolicy::new()
        .select_objective(&models, &[second, first])
        .unwrap();

    let expected = if first_goal < second_goal {
        first_goal
    } else {
        second_goal
    };

    assert_eq!(selected.goal(), &expected);
}

#[test]
fn selected_objective_builds_control_execution() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(model.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish.clone());

    let result = RecursiveControlModelPolicy::new()
        .prepare(
            &planning,
            &models,
            &start,
            &[RecursiveControlObjective::new(model, goal(vec![base(20)]))],
        )
        .unwrap();

    assert_eq!(result.control().plan().final_state(), &finish);

    assert_eq!(result.execution().current_state(), &start);
}

#[test]
fn unreachable_best_model_objective_is_rejected() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(model.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let start = state(vec![base(10)]);
    let planning = RecursivePlanningMemory::new();

    assert!(RecursiveControlModelPolicy::new()
        .prepare(
            &planning,
            &models,
            &start,
            &[RecursiveControlObjective::new(model, goal(vec![base(99)]),),],
        )
        .is_none());
}

#[test]
fn policy_decision_preserves_selected_objective() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(model.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let start = state(vec![base(10)]);

    let finish = state(vec![base(10), base(20)]);

    let target = RecursiveControlObjective::new(model, goal(vec![base(20)]));

    let mut planning = RecursivePlanningMemory::new();

    insert(&mut planning, start.clone(), finish);

    let result = RecursiveControlModelPolicy::new()
        .prepare(&planning, &models, &start, std::slice::from_ref(&target))
        .unwrap();

    assert_eq!(result.objective(), &target);
}

#[test]
fn policy_preserves_recursive_model_depth_identity() {
    let shallow = concept(1, 2);
    let deep = deep_concept();

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(deep.clone());
    revision.confirm(deep.clone());
    revision.confirm(shallow.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let expected = RecursiveControlObjective::new(deep.clone(), goal(vec![base(40)]));

    let other = RecursiveControlObjective::new(shallow, goal(vec![base(50)]));

    let selected = RecursiveControlModelPolicy::new()
        .select_objective(&models, &[other, expected.clone()])
        .unwrap();

    assert_eq!(selected, expected);
    assert_eq!(selected.model(), &deep);
}

#[test]
fn model_policy_does_not_mutate_inputs() {
    let model = concept(1, 2);

    let mut revision = RecursiveRevisionMemory::new();

    revision.confirm(model.clone());

    let models = RecursiveCompetingModels::from_memory(&revision);

    let objectives = vec![RecursiveControlObjective::new(model, goal(vec![base(20)]))];

    let models_before = models.clone();
    let objectives_before = objectives.clone();

    let _ = RecursiveControlModelPolicy::new().select_objective(&models, &objectives);

    assert_eq!(models, models_before);

    assert_eq!(objectives, objectives_before);
}

#[test]
fn model_policy_is_deterministic() {
    let best = concept(1, 2);
    let weaker = concept(5, 6);

    let models = ranked(&best, &weaker);

    let objectives = vec![
        RecursiveControlObjective::new(weaker, goal(vec![base(30)])),
        RecursiveControlObjective::new(best, goal(vec![base(20)])),
    ];

    let left = RecursiveControlModelPolicy::new().select_objective(&models, &objectives);

    let right = RecursiveControlModelPolicy::new().select_objective(&models, &objectives);

    assert_eq!(left, right);
}
