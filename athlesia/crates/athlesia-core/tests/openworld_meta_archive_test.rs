
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
fn run_with_meta_records_failed_concept_and_abstains_on_second_try() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction_low = Prediction {
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

    // Első próba: alacsony confidence -> Abstain, kudarc rögzítése.
    let outcome1 = OpenWorldCycle::run_with_meta(
        &wm, &action, &prediction_low, &observation, &mut kb, &mut meta,
    );
    assert_eq!(outcome1, OpenWorldOutcome::Abstain);
    assert!(meta.is_known_failed_concept("pixel_mismatch"));

    // Második próba ugyanazzal a mintával: az archívum miatt Abstain.
    let outcome2 = OpenWorldCycle::run_with_meta(
        &wm, &action, &prediction_low, &observation, &mut kb, &mut meta,
    );
    assert_eq!(outcome2, OpenWorldOutcome::Abstain);
    assert_eq!(kb.get_verified_concepts().len(), 0);
}
