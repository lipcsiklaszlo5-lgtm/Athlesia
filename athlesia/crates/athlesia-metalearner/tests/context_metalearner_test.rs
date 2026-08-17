
use athlesia_metalearner::MetaLearner;
use athlesia_features::FeatureVector;

fn fv(object_count: u8, touching_pairs: u8) -> FeatureVector {
    FeatureVector {
        object_count,
        color_counts: [0; 4],
        touching_pairs,
        ..Default::default()
    }
}

#[test]
fn context_scores_change_ranking_only_when_enough_evidence() {
    let mut ml = MetaLearner::default();
    let ctx = fv(2, 1);
    let ids = vec![0, 1];

    // Globálisan a 0 legyen jobb
    ml.record_success(0);
    ml.record_success(0);
    ml.record_failure(1);

    // Kontextusban a 1 kapjon két sikert, a 0 kapjon két kudarcot
    for _ in 0..2 {
        ml.record_success_in_context(ctx, 1);
        ml.record_failure_in_context(ctx, 0);
    }

    let ranked = ml.rank_in_context(ctx, &ids);
    assert_eq!(ranked, vec![1, 0]);
}

#[test]
fn context_falls_back_to_global_when_insufficient_evidence() {
    let mut ml = MetaLearner::default();
    let ctx = fv(2, 1);
    let ids = vec![0, 1];

    ml.record_success(0);
    ml.record_failure(1);

    // Csak egy kontextus minta: nem elég, globális prioritás érvényesül
    ml.record_success_in_context(ctx, 0);

    let ranked = ml.rank_in_context(ctx, &ids);
    // globális: 0 jobb, mert 0>1
    assert_eq!(ranked, vec![0, 1]);
}
