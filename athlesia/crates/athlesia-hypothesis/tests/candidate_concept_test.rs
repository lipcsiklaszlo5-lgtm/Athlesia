
use athlesia_hypothesis::{ConceptSketch, CandidateConcept};

#[test]
fn candidate_concept_is_created_without_knowledge_base() {
    let sketch = ConceptSketch {
        name: "RepeatedInteraction".to_string(),
        relation_pattern: "interaction(A,B)".to_string(),
        objects_involved: vec![1, 2],
    };

    let candidate = CandidateConcept {
        sketch,
        evidence: vec!["residual: unexpected motion".to_string()],
        confidence: 0.3,
    };

    assert_eq!(candidate.sketch.name, "RepeatedInteraction");
    assert_eq!(candidate.sketch.relation_pattern, "interaction(A,B)");
    assert_eq!(candidate.sketch.objects_involved, vec![1, 2]);
    assert_eq!(candidate.evidence.len(), 1);
    assert!((candidate.confidence - 0.3).abs() < 1e-9);
}
