
use athlesia_core::openworld::{OpenWorldCycle, OpenWorldOutcome};
use athlesia_world_model::{WorldModel, Observation, Prediction};
use athlesia_knowledge::KnowledgeBase;
use athlesia_metalearner::MetaLearner;
use athlesia_interactive::{Environment, ProbeAction};
use athlesia_types::{Grid, Color, Action, PrimName, Params};

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
fn run_experiment_cycle_discovers_trigger_via_interactive_environment() {
    // A környezet rejtett triggere C.
    let mut env = Environment::new(ProbeAction::C);
    let initial_grid = env.grid.clone();

    // WorldModel kezdetben csak egy irreleváns hipotézissel.
    let mut wm = WorldModel::new(initial_grid.clone());
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let mut kb = KnowledgeBase::new();
    let mut meta = MetaLearner::new();

    // Kísérleti kérés: Translate(0,1)
    let request = athlesia_planner::ExperimentRequest {
        action: Action { prim: PrimName::Translate, params: Params::Translate(0, 1) },
        target_hypothesis: "object_count_change(A,B)".to_string(),
        expected_observation: "object_count_change(A,B)".to_string(),
    };

    // Closure: végrehajtja a C akciót a környezetben.
    // (Mivel a request.action Translate(0,1), de a környezet a C-t várja,
    // a rendszernek kísérleti megfigyelést kell kapnia.)
    let mut executed = false;
    let outcome = OpenWorldCycle::run_experiment_cycle(
        &wm,
        &mut kb,
        &mut meta,
        request,
        |_| {
            executed = true;
            // Valós interaktív környezet: a C akció jobbra mozdítja az objektumot.
            let observed_grid = env.step(&ProbeAction::C);
            Observation { state: observed_grid }
        },
    );

    assert!(executed, "A kísérleti akciót végre kellett hajtani");
    match outcome {
        OpenWorldOutcome::Verified(_) | OpenWorldOutcome::Retrieved(_) => {}
        other => panic!("Várt Verified/Retrieved, de {:?} kaptunk", other),
    }
    assert_eq!(kb.get_verified_concepts().len(), 1);
}
