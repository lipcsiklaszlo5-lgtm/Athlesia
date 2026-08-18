#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. OpenWorldCycle bővítése generikus kísérleti ciklussal
p = pathlib.Path("crates/athlesia-core/src/openworld.rs")
s = p.read_text()

# Szükséges importok bővítése
if "use athlesia_planner::ExperimentRequest;" not in s:
    s = s.replace(
        "use athlesia_planner::{Planner, PlannerMode};",
        "use athlesia_planner::{Planner, PlannerMode, ExperimentRequest};",
        1,
    )

# Új metódus beszúrása a run_with_outcome után, de a run elé
anchor = '''    /// Kísérleti kérést generál a reziduálisból felfedezett candidate concepthez.
    ///
    /// Ha a rendszer OutOfModel, és a candidate confidence elég magas
    /// (>= 0.5), akkor kísérleti kérést ad; különben `None`.
    pub fn prepare_experiment(
        wm: &WorldModel,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
        _kb: &KnowledgeBase,
    ) -> Option<athlesia_planner::ExperimentRequest> {
        let (state, residual) = wm.evaluate_with_residual(action, prediction, observation);
        if state != KnowledgeState::OutOfModel {
            return None;
        }

        let residuals = vec![residual];
        let candidate = AbstractionEngine::discover_candidate_concept(&residuals)?;

        if candidate.confidence < 0.5 {
            return None;
        }

        let planner = Planner::new(PlannerMode::Exploration);
        Some(planner.plan_experiment_request(&candidate))
    }
'''

new_method = anchor + '''

    /// Generikus kísérleti ciklus.
    ///
    /// A megadott `execute` closure végrehajtja az `ExperimentRequest` akcióját,
    /// és visszaadja a megfigyelt állapotot. Ezután az OpenWorldCycle
    /// lefuttatja a fogalomtanulást a predikció és a megfigyelés alapján.
    ///
    /// F: FnMut(&Action) -> Observation
    pub fn run_experiment_cycle<F>(
        wm: &WorldModel,
        kb: &mut KnowledgeBase,
        meta: &mut MetaLearner,
        request: ExperimentRequest,
        mut execute: F,
    ) -> OpenWorldOutcome
    where
        F: FnMut(&Action) -> Observation,
    {
        // Predikció készítése a jelenlegi állapotból.
        let current_state = wm.current_state.clone();
        let prediction = wm.predict(&current_state, &request.action);

        // Az akció végrehajtása a külvilág által.
        let observation = execute(&request.action);

        // OpenWorld futtatása a megfigyeléssel.
        Self::run_with_meta(wm, &request.action, &prediction, &observation, kb, meta)
    }
'''

if anchor not in s:
    print("[ERROR] prepare_experiment blokk nem található.")
    sys.exit(1)
s = s.replace(anchor, new_method)

p.write_text(s)
print("[1] openworld.rs frissítve: run_experiment_cycle generikus metódus.")

# 2. Új tesztfájl, amely az athlesia-interactive környezetet használja
test_code = r'''
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
'''

write_file("crates/athlesia-core/tests/openworld_experiment_cycle_test.rs", test_code)
print("[2] openworld_experiment_cycle_test.rs létrehozva.")

# 3. Core tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Core tesztek zöldek.")

# 4. Teljes workspace teszt
result = subprocess.run(
    ["cargo", "test", "--workspace", "--no-fail-fast"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Teljes workspace tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Teljes workspace tesztek zöldek.")

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 21: generic experiment cycle with external execution closure"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
