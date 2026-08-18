#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Planner lib.rs: ExperimentRequest típus és plan_experiment_request metódus
p = pathlib.Path("crates/athlesia-planner/src/lib.rs")
s = p.read_text()

# ExperimentRequest a már meglévő ExperimentPlan után
anchor = '''/// Kísérleti terv a candidate concept aktív verifikálásához.
#[derive(Debug, Clone)]
pub struct ExperimentPlan {
    pub actions: Vec<athlesia_types::Action>,
    pub target_hypothesis: String,
    pub expected_observation: String,
}
'''

new_struct = anchor + '''

/// Kísérleti kérés: egyetlen akció, amelyet végre kell hajtani.
#[derive(Debug, Clone)]
pub struct ExperimentRequest {
    pub action: athlesia_types::Action,
    pub target_hypothesis: String,
    pub expected_observation: String,
}
'''

if anchor not in s:
    print("[ERROR] ExperimentPlan blokk nem található.")
    sys.exit(1)
s = s.replace(anchor, new_struct)

# Új metódus beszúrása a select_probe_action után
anchor2 = '''    pub fn select_probe_action(&self, candidate: &CandidateConcept) -> Action {
        let pattern = candidate.sketch.relation_pattern.to_lowercase();
        if pattern.contains("interaction") {
            Action { prim: PrimName::Translate, params: Params::Translate(1, 0) }
        } else if pattern.contains("symmetry") {
            Action { prim: PrimName::ReflectH, params: Params::None }
        } else {
            Action { prim: PrimName::Translate, params: Params::Translate(0, 1) }
        }
    }
'''

new_method = anchor2 + '''

    /// Kísérleti kérést készít a candidate concepthez.
    pub fn plan_experiment_request(&self, candidate: &CandidateConcept) -> ExperimentRequest {
        let action = self.select_probe_action(candidate);
        ExperimentRequest {
            action,
            target_hypothesis: candidate.sketch.name.clone(),
            expected_observation: candidate.sketch.relation_pattern.clone(),
        }
    }
'''

if anchor2 not in s:
    print("[ERROR] select_probe_action blokk nem található.")
    sys.exit(1)
s = s.replace(anchor2, new_method)
p.write_text(s)
print("[1] Planner lib.rs frissítve: ExperimentRequest és plan_experiment_request.")

# 2. OpenWorldCycle: prepare_experiment metódus hozzáadása
p = pathlib.Path("crates/athlesia-core/src/openworld.rs")
s = p.read_text()

# Import Planner és ExperimentRequest
if "use athlesia_planner::{Planner, PlannerMode};" not in s:
    s = s.replace(
        "use athlesia_metalearner::MetaLearner;",
        "use athlesia_metalearner::MetaLearner;\nuse athlesia_planner::{Planner, PlannerMode};",
        1,
    )

# Metódus beszúrása a run_with_meta után
anchor3 = '''    /// Az open-world ciklus kimenettel együtt (MetaLearner nélkül).
    ///
    /// - Ha nem OutOfModel: NotOutOfModel
    /// - Ha van OutOfModel, de a candidate confidence < 0.5: Abstain
    /// - Ha a relation_pattern már létezik: Retrieved
    /// - Különben Verified
    pub fn run_with_outcome(
'''

new_method3 = '''    /// Kísérleti kérést generál a reziduálisból felfedezett candidate concepthez.
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

    /// Az open-world ciklus kimenettel együtt (MetaLearner nélkül).
    ///
    /// - Ha nem OutOfModel: NotOutOfModel
    /// - Ha van OutOfModel, de a candidate confidence < 0.5: Abstain
    /// - Ha a relation_pattern már létezik: Retrieved
    /// - Különben Verified
    pub fn run_with_outcome(
'''

if anchor3 not in s:
    print("[ERROR] run_with_outcome blokk nem található.")
    sys.exit(1)
s = s.replace(anchor3, new_method3)
p.write_text(s)
print("[2] openworld.rs frissítve: prepare_experiment metódus.")

# 3. Új tesztfájl a prepare_experiment-hez
test_code = r'''
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
'''

write_file("crates/athlesia-core/tests/openworld_prepare_experiment_test.rs", test_code)
print("[3] openworld_prepare_experiment_test.rs létrehozva.")

# 4. Core tesztek futtatása
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

# 5. Teljes workspace teszt
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

# 6. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 20: add ExperimentRequest and prepare_experiment to OpenWorldCycle"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
