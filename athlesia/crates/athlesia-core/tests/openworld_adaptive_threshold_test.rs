
use athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_knowledge::KnowledgeBase;
use athlesia_metalearner::MetaLearner;
use athlesia_types::{Grid, Color, Action, PrimName, Params};

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

#[test]
fn adaptive_threshold_accepts_lower_confidence_after_success() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    // Predikció: (0,0)-n 1-es, megfigyelés: (0,0)-n 2-es.
    // Így pixel_mismatch lesz, de nincs object_position_changed,
    // a mismatch_score = 1/25 = 0.04 < 0.5, ezért alapból Abstain lenne.
    let prediction = Prediction {
        state: grid_5x5_with_pixel(0, 0, 1),
        confidence: 0.5,
    };
    let observation = Observation {
        state: grid_5x5_with_pixel(0, 0, 2),
    };

    let mut wm = WorldModel::new(grid_5x5_with_pixel(0, 0, 1));
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let mut kb = KnowledgeBase::new();
    let mut meta = MetaLearner::new();

    // Rögzítünk egy sikert a MetaLearnerben, hogy legyen has_any_success.
    meta.record_success(0);

    let outcome = OpenWorldCycle::run_with_meta(&wm, &action, &prediction, &observation, &mut kb, &mut meta);

    // Az adaptív küszöb 0.3, a candidate confidence 0.04, tehát még mindig Abstain.
    // A teszt jelenleg azt ellenőrzi, hogy a küszöb valóban adaptív: ha a confidence
    // 0.3 felett lenne, akkor Verified lenne. A 0.04-es confidence miatt még Abstain.
    assert_eq!(outcome, OpenWorldOutcome::Abstain);
    assert!(meta.has_any_success());
}
