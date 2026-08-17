
use athlesia_metalearner::MetaLearner;

#[test]
fn initial_priority_is_neutral() {
    let ml = MetaLearner::default();
    assert_eq!(ml.priority(0), 0.5);
}

#[test]
fn success_increases_priority() {
    let mut ml = MetaLearner::default();
    ml.record_success(0);
    assert!(ml.priority(0) > 0.5);
}

#[test]
fn failure_decreases_priority() {
    let mut ml = MetaLearner::default();
    ml.record_failure(0);
    assert!(ml.priority(0) < 0.5);
}

#[test]
fn rank_orders_by_priority() {
    let mut ml = MetaLearner::default();
    ml.record_success(0);
    ml.record_success(0);
    ml.record_failure(1);
    ml.record_failure(1);
    // 0: (3)/(4) = 0.75, 1: (1)/(4) = 0.25
    let ranked = ml.rank(&[1, 0]);
    assert_eq!(ranked, vec![0, 1]);
}
