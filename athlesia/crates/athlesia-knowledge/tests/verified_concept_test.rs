
use athlesia_knowledge::KnowledgeBase;

#[test]
fn add_and_retrieve_verified_concepts() {
    let mut kb = KnowledgeBase::new();
    kb.add_verified_concept("RepeatedInteraction".to_string(), "interaction(A,B)".to_string(), 3);

    let verified = kb.get_verified_concepts();
    assert_eq!(verified.len(), 1);
    assert_eq!(verified[0].name, "RepeatedInteraction");
    assert_eq!(verified[0].relation_pattern, "interaction(A,B)");
    assert_eq!(verified[0].evidence_count, 3);
}
