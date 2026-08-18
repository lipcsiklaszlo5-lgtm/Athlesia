
use athlesia_kernel::Agent;
use athlesia_world_model::Observation;
use athlesia_types::{Grid, Color, Action, PrimName, Params};
use athlesia_core::openworld::{OpenWorldOutcome};

fn grid_5x5_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 25];
    cells[y * 5 + x] = Color(val);
    Grid { width: 5, height: 5, cells }
}

#[test]
fn agent_openworld_step_creates_verified_concept_on_out_of_model() {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let mut agent = Agent::new(initial.clone());

    // Irreleváns hipotézis, hogy a Translate akcióra OutOfModel legyen.
    agent.wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let action = Action {
        prim: PrimName::Translate,
        params: Params::Translate(0, 1),
    };
    // Szándékosan eltérő dimenziójú megfigyelés -> magas mismatch_score -> Verified
    let observation = Observation {
        state: Grid::new(3, 3),
    };

    let outcome = agent.openworld_step(&action, &observation);

    match outcome {
        OpenWorldOutcome::Verified(_) | OpenWorldOutcome::Retrieved(_) => {}
        other => panic!("Várt Verified/Retrieved, de {:?} kaptunk", other),
    }

    assert_eq!(
        agent.kb.get_verified_concepts().len(),
        1,
        "Egy igazolt fogalomnak kell keletkeznie"
    );
}
