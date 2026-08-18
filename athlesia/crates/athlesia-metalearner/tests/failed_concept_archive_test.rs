
use athlesia_metalearner::MetaLearner;

#[test]
fn record_and_check_failed_concept() {
    let mut meta = MetaLearner::new();
    assert!(!meta.is_known_failed_concept("interaction(A,B)"));

    meta.record_failed_concept("interaction(A,B)".to_string());
    assert!(meta.is_known_failed_concept("interaction(A,B)"));
    assert!(!meta.is_known_failed_concept("other"));
}
