
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

fn setup_wm() -> WorldModel {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);
    wm
}

#[test]
fn openworld_cycle_recalls_existing_concept_on_same_pattern() {
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let prediction = Prediction {
        state: grid_3x3_zeros(),
        confidence: 0.5,
    };
    let observation = Observation {
        state: grid_5x5_with_pixel(0, 0, 1),
    };

    let mut kb = KnowledgeBase::new();

    // Első alkalom: létrehoz egy igazolt fogalmat.
    let first = OpenWorldCycle::run(&setup_wm(), &action, &prediction, &observation, &mut kb);
    assert!(first.is_some());
    let first_count = kb.get_verified_concepts().len();
    assert_eq!(first_count, 1);

    // Második, új példány azonos reziduális mintával:
    // ugyanazt a fogalmat kell visszakapnia, nem szabad új elemet hozzáadnia.
    let second = OpenWorldCycle::run(&setup_wm(), &action, &prediction, &observation, &mut kb);
    assert!(second.is_some());
    assert_eq!(kb.get_verified_concepts().len(), first_count);
    assert_eq!(
        second.as_ref().unwrap().relation_pattern,
        first.as_ref().unwrap().relation_pattern
    );
}
