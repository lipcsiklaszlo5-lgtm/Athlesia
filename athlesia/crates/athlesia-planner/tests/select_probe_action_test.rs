
use athlesia_planner::{Planner, PlannerMode};
use athlesia_hypothesis::{CandidateConcept, ConceptSketch};
use athlesia_types::{PrimName, Params};

#[test]
fn select_probe_action_returns_translate_for_interaction() {
    let planner = Planner::new(PlannerMode::Exploration);
    let candidate = CandidateConcept {
        sketch: ConceptSketch {
            name: "candidate".to_string(),
            relation_pattern: "interaction(A,B)".to_string(),
            objects_involved: vec![1, 2],
        },
        evidence: vec!["residual".to_string()],
        confidence: 0.5,
    };
    let action = planner.select_probe_action(&candidate);
    assert_eq!(action.prim, PrimName::Translate);
    assert_eq!(action.params, Params::Translate(1, 0));
}

#[test]
fn select_probe_action_returns_reflect_for_symmetry() {
    let planner = Planner::new(PlannerMode::Exploration);
    let candidate = CandidateConcept {
        sketch: ConceptSketch {
            name: "candidate".to_string(),
            relation_pattern: "symmetry(A,B)".to_string(),
            objects_involved: vec![1, 2],
        },
        evidence: vec!["residual".to_string()],
        confidence: 0.5,
    };
    let action = planner.select_probe_action(&candidate);
    assert_eq!(action.prim, PrimName::ReflectH);
    assert_eq!(action.params, Params::None);
}
