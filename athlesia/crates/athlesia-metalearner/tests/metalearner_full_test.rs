
use athlesia_metalearner::MetaLearner;
use athlesia_features::FeatureVector;
use athlesia_types::{PrimName, Params, Program};

fn fv(object_count: u8, touching_pairs: u8) -> FeatureVector {
    FeatureVector {
        object_count,
        touching_pairs,
        ..Default::default()
    }
}

#[test]
fn initial_priority_is_neutral() {
    let ml = MetaLearner::new();
    assert_eq!(ml.priority(0), 0.5);
}

#[test]
fn success_increases_priority() {
    let mut ml = MetaLearner::new();
    ml.record_success(0);
    assert!(ml.priority(0) > 0.5);
}

#[test]
fn failure_decreases_priority() {
    let mut ml = MetaLearner::new();
    ml.record_failure(0);
    assert!(ml.priority(0) < 0.5);
}

#[test]
fn context_scores_change_ranking_only_when_enough_evidence() {
    let mut ml = MetaLearner::new();
    let ctx = fv(2, 1);
    let ids = vec![0, 1];

    ml.record_success(0);
    ml.record_success(0);
    ml.record_failure(1);

    for _ in 0..2 {
        ml.record_success_in_context(ctx, 1);
        ml.record_failure_in_context(ctx, 0);
    }

    let ranked = ml.rank_in_context(ctx, &ids);
    assert_eq!(ranked, vec![1, 0]);
}

#[test]
fn context_falls_back_to_global_when_insufficient_evidence() {
    let mut ml = MetaLearner::new();
    let ctx = fv(2, 1);
    let ids = vec![0, 1];

    ml.record_success(0);
    ml.record_failure(1);

    ml.record_success_in_context(ctx, 0);

    let ranked = ml.rank_in_context(ctx, &ids);
    assert_eq!(ranked, vec![0, 1]);
}

#[test]
fn failure_archive_records_and_checks() {
    let mut ml = MetaLearner::new();
    let ctx = fv(1, 0);
    let program: Program = vec![(PrimName::ReflectH, Params::None)];

    assert!(!ml.is_known_failure(ctx, &program));
    ml.record_failure_pattern(ctx, program.clone());
    assert!(ml.is_known_failure(ctx, &program));
}

#[test]
fn abstraction_score_is_deterministic_and_positive() {
    let ml = MetaLearner::new();
    let program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let score1 = ml.score_abstraction(&program);
    let score2 = ml.score_abstraction(&program);
    assert_eq!(score1, score2);
    assert!(score1 >= 0.0);
}
