
use athlesia_planner::{Planner, PlannerMode, ExperimentPlan};
use athlesia_hypothesis::{CandidateConcept, ConceptSketch};

#[test]
fn plan_experiment_returns_plan_with_target_hypothesis() {
    let planner = Planner::new(PlannerMode::Exploration);
    let candidate = CandidateConcept {
        sketch: ConceptSketch {
            name: "RepeatedInteraction".to_string(),
            relation_pattern: "interaction(A,B)".to_string(),
            objects_involved: vec![1, 2],
        },
        evidence: vec!["residual".to_string()],
        confidence: 0.5,
    };

    let plan: ExperimentPlan = planner.plan_experiment(&candidate);
    assert_eq!(plan.target_hypothesis, "RepeatedInteraction");
    assert_eq!(plan.expected_observation, "interaction(A,B)");
    assert!(plan.actions.is_empty());
}
