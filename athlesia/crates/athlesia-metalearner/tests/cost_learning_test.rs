
use athlesia_metalearner::MetaLearner;
use athlesia_features::FeatureVector;

#[test]
fn record_and_estimate_search_cost() {
    let mut meta = MetaLearner::new();
    let context = FeatureVector::default();
    let hyp_id = 0;

    // Kezdetben nincs becslés
    assert!(meta.estimated_cost(context, hyp_id).is_none());

    // Költségek rögzítése
    meta.record_search_cost_in_context(context, hyp_id, 10.0);
    meta.record_search_cost_in_context(context, hyp_id, 20.0);

    let estimated = meta.estimated_cost(context, hyp_id).expect("Léteznie kell becslésnek");
    assert!((estimated - 15.0).abs() < 0.001, "Átlag 15.0 kell, de {} volt", estimated);
}
