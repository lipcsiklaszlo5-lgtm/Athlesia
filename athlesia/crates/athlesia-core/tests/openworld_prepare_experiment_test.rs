
use athlesia_core::openworld::OpenWorldCycle;
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::{Grid, Color, Action, PrimName, Params};

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

fn grid_3x3_zeros() -> Grid {
    Grid { width: 3, height: 3, cells: vec![Color(0); 9] }
}

#[test]
fn prepare_experiment_returns_request_for_out_of_model_with_high_confidence() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(0, 1) };
    let prediction = Prediction { state: grid_3x3_zeros(), confidence: 0.5 };
    let observation = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let mut wm = WorldModel::new(grid_5x5_with_pixel(0, 0, 1));
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);
    let kb = KnowledgeBase::new();

    let request = OpenWorldCycle::prepare_experiment(&wm, &action, &prediction, &observation, &kb)
        .expect("Kísérleti kérést kell kapni");

    assert!(!request.target_hypothesis.is_empty());
    assert!(!request.expected_observation.is_empty());
    assert_eq!(request.action.prim, PrimName::Translate); // a select_probe_action heurisztika szerint
}

#[test]
fn prepare_experiment_returns_none_when_not_out_of_model() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(0, 1) };
    let prediction = Prediction { state: grid_5x5_with_pixel(0, 0, 1), confidence: 0.5 };
    let observation = Observation { state: grid_5x5_with_pixel(0, 0, 1) };
    let wm = WorldModel::new(grid_5x5_with_pixel(0, 0, 1));
    let kb = KnowledgeBase::new();

    let request = OpenWorldCycle::prepare_experiment(&wm, &action, &prediction, &observation, &kb);
    assert!(request.is_none());
}
