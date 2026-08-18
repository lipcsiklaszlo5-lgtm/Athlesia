
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
fn agent_openworld_step_recalls_existing_concept_on_second_episode() {
    let initial = grid_5x5_with_pixel(0, 0, 1);
    let mut agent = Agent::new(initial.clone());

    // Irreleváns hipotézis a WorldModelben.
    agent.wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let action = Action {
        prim: PrimName::Translate,
        params: Params::Translate(0, 1),
    };

    // Első epizód: dimenzióeltérés → Verified
    let obs1 = Observation {
        state: Grid::new(3, 3),
    };
    let outcome1 = agent.openworld_step(&action, &obs1);

    match outcome1 {
        OpenWorldOutcome::Verified(_) | OpenWorldOutcome::Retrieved(_) => {}
        other => panic!("Első epizódban Verified/Retrieved várt, de {:?} kaptunk", other),
    }
    assert_eq!(agent.kb.get_verified_concepts().len(), 1);

    // Második epizód: új, eltérő dimenziójú megfigyelés
    let obs2 = Observation {
        state: Grid::new(4, 4),
    };
    let outcome2 = agent.openworld_step(&action, &obs2);

    match outcome2 {
        OpenWorldOutcome::Retrieved(_) => {}
        other => panic!("Második epizódban Retrieved várt, de {:?} kaptunk", other),
    }
    // Nem szabad új fogalmat létrehozni.
    assert_eq!(
        agent.kb.get_verified_concepts().len(),
        1,
        "A meglévő fogalmat kell visszakapni, nem újat létrehozni"
    );
}
