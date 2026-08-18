
use athlesia_core::openworld::OpenWorldCycle;
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

#[test]
fn openworld_cycle_creates_verified_concept_on_out_of_model() {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    // Prediction: eltérő dimenziójú, hogy a mismatch_score 1.0 legyen,
    // így a candidate confidence eléri a küszöböt.
    let prediction = Prediction {
        state: grid_3x3_zeros(),
        confidence: 0.5,
    };
    let observation = Observation { state: initial.clone() };

    let mut kb = KnowledgeBase::new();
    let verified = OpenWorldCycle::run(&wm, &action, &prediction, &observation, &mut kb);

    assert!(verified.is_some(), "A ciklusnak igazolt fogalmat kell létrehoznia");
    assert_eq!(kb.get_verified_concepts().len(), 1);
}

#[test]
fn openworld_cycle_no_verified_concept_when_not_out_of_model() {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let wm = WorldModel::new(initial.clone()); // nincs hipotézis

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: initial.clone() };

    let mut kb = KnowledgeBase::new();
    let verified = OpenWorldCycle::run(&wm, &action, &prediction, &observation, &mut kb);

    assert!(verified.is_none(), "Nem szabad fogalmat létrehozni, ha nem OutOfModel");
}
