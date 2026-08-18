
use athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_knowledge::KnowledgeBase;
use athlesia_interactive::{Environment, ProbeAction, InteractiveAgent};
use athlesia_types::{Grid, Action, PrimName, Params};

fn object_position(grid: &Grid) -> (i8, i8) {
    for y in 0..grid.height as i8 {
        for x in 0..grid.width as i8 {
            if let Some(c) = grid.get(x, y) {
                if c.0 != 0 {
                    return (x, y);
                }
            }
        }
    }
    (-1, -1)
}

#[test]
fn active_loop_discovers_hidden_trigger_via_openworld_cycle() {
    // A környezet rejtett triggere C.
    let mut env = Environment::new(ProbeAction::C);
    let initial_grid = env.grid.clone();

    // WorldModel kezdetben csak egy irreleváns hipotézissel (ReflectH),
    // hogy a Translate akcióra OutOfModel-t adjon.
    let mut wm = WorldModel::new(initial_grid.clone());
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let mut kb = KnowledgeBase::new();
    let mut steps = 0;
    let mut verified_concepts_count = 0;

    // Próbáljuk ki a Translate(1,0) akciót.
    let action = Action { prim: PrimName::Translate, params: Params::Translate(0, 1) };

    // Predikció: a jelenlegi modell szerint a Translate nem változtat
    // (nincs rá hipotézis), ezért a predict eredménye maga az input.
    let prediction = Prediction { state: Grid::new(3, 3), confidence: 0.5 }; // szándékos dimenzióeltérés

    // Megfigyelés a környezetből: valójában az objektum jobbra mozdul.
    let old_pos = object_position(&env.grid);
    let observed_grid = env.step(&ProbeAction::C); // A C akció a rejtett trigger
    let new_pos = object_position(&observed_grid);
    assert_ne!(old_pos, new_pos, "A C akciónak mozgást kell kiváltania.");

    let observation = Observation { state: observed_grid.clone() };

    // Open-world ciklus futtatása.
    let outcome = OpenWorldCycle::run_with_outcome(
        &wm,
        &action,
        &prediction,
        &observation,
        &mut kb,
    );

    // Az eltérés miatt OutOfModel-nek kell lennie, és fogalmat kell létrehoznia/igazolnia.
    match outcome {
        OpenWorldOutcome::Verified(_) | OpenWorldOutcome::Retrieved(_) => {
            verified_concepts_count = kb.get_verified_concepts().len();
        }
        other => panic!("Várt Verified/Retrieved, de {:?} kaptunk", other),
    }

    assert_eq!(verified_concepts_count, 1, "Egy igazolt fogalomnak kell keletkeznie.");
    steps += 1;
    assert!(steps < 5, "A felfedezés túl sok lépést igényelt.");
}
