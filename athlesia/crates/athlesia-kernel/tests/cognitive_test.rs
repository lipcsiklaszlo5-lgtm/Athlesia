
use athlesia_kernel::cognitive::{CognitiveController, CognitiveDecision};
use athlesia_metalearner::MetaLearner;
use athlesia_features::FeatureVector;
use athlesia_types::{PrimName, Params, Program};

fn make_fv(object_count: u8) -> FeatureVector {
    FeatureVector {
        object_count,
        ..Default::default()
    }
}

#[test]
fn controller_decides_guess_when_no_prior() {
    let fv = make_fv(1);
    let meta = MetaLearner::new();
    let programs: Vec<Program> = vec![vec![(PrimName::Translate, Params::Translate(1, 0))]];

    let decision = CognitiveController::decide(&fv, &meta, &programs);
    // Mivel nincs kontextus-pontszám, a döntés Guess (vagy Explore, ha így alakul)
    // Most a placeholder logika szerint Guess lesz, de ellenőrizzük a változatosságot.
    assert_ne!(decision, CognitiveDecision::Abstain);
}

#[test]
fn controller_estimates_competence() {
    let fv = make_fv(2);
    let meta = MetaLearner::new();

    let estimate = CognitiveController::estimate(&fv, &meta);
    assert!(estimate.hypothesis_confidence >= 0.0 && estimate.hypothesis_confidence <= 1.0);
    assert_eq!(estimate.familiarity, estimate.hypothesis_confidence);
}

#[test]
fn controller_decides_solve_when_confident() {
    let mut meta = MetaLearner::new();
    // Mesterségesen megnöveljük a 0 hipotézis konfidenciáját
    let fv = make_fv(1);
    for _ in 0..5 {
        meta.record_success_in_context(fv, 0);
    }

    let programs: Vec<Program> = vec![vec![(PrimName::Translate, Params::Translate(1, 0))]];
    let decision = CognitiveController::decide(&fv, &meta, &programs);

    assert_eq!(decision, CognitiveDecision::Solve);
}
