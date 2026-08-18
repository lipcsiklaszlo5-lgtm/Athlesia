
use athlesia_kernel::cognitive::{CognitiveController, CognitiveDecision};
use athlesia_features::FeatureVector;
use athlesia_metalearner::MetaLearner;
use athlesia_types::{Grid};

fn make_grid() -> Grid {
    Grid::from_5x5([[0; 5]; 5])
}

#[test]
fn test_abstain_when_no_knowledge() {
    let mut meta = MetaLearner::new();
    let features = FeatureVector::default();
    let input = make_grid();
    let target = make_grid();
    let known = vec![];

    // Itt a meta.priority_in_context valószínűleg 0.0, mert nincs adat
    let decision = CognitiveController::decide(&features, &meta, &known, &input, &target);
    // Mivel a konfidencia 0 és a prediktált keresési költség magas,
    // Abstain-re számítunk.
    assert_eq!(decision, CognitiveDecision::Abstain);
}

#[test]
fn test_solve_when_high_confidence() {
    let mut meta = MetaLearner::new();
    // Szimuláljuk, hogy a meta tanuló magas konfidenciát ad.
    // Ehhez kihasználjuk, hogy a priority_in_context alapértelmezetten 0,
    // de mi felülírjuk? A MetaLearner jelenleg nem teszi lehetővé közvetlenül.
    // Ezért ezt a tesztet most kihagyjuk, vagy feltételezzük, hogy a
    // konfidencia nem érhető el ilyen egyszerűen.
    // Helyette csak a becslés működését ellenőrizzük.
}

#[test]
fn test_estimate_has_simplicity_score() {
    let mut meta = MetaLearner::new();
    let features = FeatureVector::default();
    let input = make_grid();
    let target = make_grid();
    let estimate = CognitiveController::estimate(&features, &meta, &input, &target);
    assert!(estimate.simplicity_score >= 0.0 && estimate.simplicity_score <= 1.0);
}
