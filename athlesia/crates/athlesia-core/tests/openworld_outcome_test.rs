
use athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};
use athlesia_knowledge::KnowledgeBase;

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

fn grid_3x3_zeros() -> Grid {
    Grid { width: 3, height: 3, cells: vec![Color(0); 9] }
}

fn setup_wm() -> WorldModel {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);
    wm
}

#[test]
fn run_with_outcome_not_out_of_model() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let wm = WorldModel::new(grid_5x5_with_pixel(0, 0, 1)); // nincs hipotézis
    let prediction = wm.predict(&grid_5x5_with_pixel(0, 0, 1), &action);
    let observation = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let mut kb = KnowledgeBase::new();

    let outcome = OpenWorldCycle::run_with_outcome(&wm, &action, &prediction, &observation, &mut kb);
    assert_eq!(outcome, OpenWorldOutcome::NotOutOfModel);
}

#[test]
fn run_with_outcome_abstain_when_low_confidence() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: grid_3x3_zeros(), confidence: 0.5 };
    let observation = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let mut kb = KnowledgeBase::new();

    // Itt a mismatch_score 1.0 lesz, mert dimenzióeltérés, tehát confidence >= 0.5 -> Verified.
    // Ez NEM jó Abstain tesztnek. Hogy Abstain legyen, olyan reziduális kell, ahol a mismatches kisebb.
    // Ehelyett a tesztet inkább úgy módosítjuk, hogy a `discover_candidate_concept` visszatérési
    // feltételét kikerüljük? Nem, a confidence 0.5 alatt kell lennie.
    // Készítünk egy olyan predikciót, amely ugyanolyan méretű és 1 pixel eltérés = 0.04 -> confidence 0.04 < 0.5.
    let pred_low = Prediction { state: grid_5x5_with_pixel(1, 0, 0), confidence: 0.5 };
    let obs_low = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let outcome = OpenWorldCycle::run_with_outcome(&setup_wm(), &action, &pred_low, &obs_low, &mut kb);
    assert_eq!(outcome, OpenWorldOutcome::Abstain);
}

#[test]
fn run_with_outcome_verified_on_dimension_mismatch() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: grid_3x3_zeros(), confidence: 0.5 };
    let observation = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let mut kb = KnowledgeBase::new();

    let outcome = OpenWorldCycle::run_with_outcome(&setup_wm(), &action, &prediction, &observation, &mut kb);
    match outcome {
        OpenWorldOutcome::Verified(_) => {}
        other => panic!("Verified várt, de {:?} kaptunk", other),
    }
}
