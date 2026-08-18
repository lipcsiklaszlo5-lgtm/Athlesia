
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

#[test]
fn residual_includes_object_count_changed_when_segmentation_differs() {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let mut observation = grid_5x5_with_pixel(0, 0, 1);
    // Adjunk egy extra objektumot a megfigyeléshez, hogy a szegmensek száma eltérjen.
    observation.set(2, 2, Color(1));

    let wm = WorldModel::new(initial.clone());
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction { state: initial.clone(), confidence: 0.5 };
    let observation = Observation { state: observation };

    let residual = wm.compute_prediction_residual(&action, &prediction, &observation);
    assert!(residual.unexplained_features.contains(&"object_count_changed".to_string()));
}
