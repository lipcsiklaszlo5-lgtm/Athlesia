#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. WorldModel lib.rs módosítása
p = pathlib.Path("crates/athlesia-world-model/src/lib.rs")
s = p.read_text()

# 1.a. KnowledgeState és PredictionResidual típusok beszúrása a PredictionError után
anchor = '''/// Predikciós hiba: miért nem egyezett a predikció a megfigyeléssel.
#[derive(Debug, Clone)]
pub struct PredictionError {
    pub expected: Grid,
    pub observed: Grid,
    pub summary: String,
    pub feature_mismatch: usize,
}
'''

new_types = anchor + '''

/// A tudás állapota a jelenlegi hipotézistér és a megfigyelés viszonyában.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeState {
    Explained,
    Uncertain,
    Contradicted,
    OutOfModel,
}

/// Predikciós reziduális: strukturált különbség a várt és megfigyelt állapot között.
#[derive(Debug, Clone)]
pub struct PredictionResidual {
    pub expected_observation: Observation,
    pub observed_observation: Observation,
    pub mismatch_score: f64,
    pub unexplained_features: Vec<String>,
}
'''

if anchor not in s:
    print("[ERROR] A PredictionError blokk nem található.")
    sys.exit(1)
s = s.replace(anchor, new_types)

# 1.b. evaluate_prediction metódus hozzáadása a WorldModel impl-en belül
# A learn_from_error metódus után szúrjuk be.
method_anchor = '''    /// A modell frissítése predikciós hiba alapján.
    /// A hibát egy hipotézishez kötjük, és csökkentjük annak konfidenciáját.
    pub fn learn_from_error(&mut self, hypothesis_id: u64, _error: &PredictionError) {
        if let Some(hyp) = self.hypotheses.iter_mut().find(|h| h.id == hypothesis_id) {
            hyp.evidence_against += 1;
            hyp.status = HypothesisStatus::Falsified;
        }
    }
'''

new_method = method_anchor + '''

    /// Kiértékeli a predikciót a megfigyeléshez képest.
    ///
    /// - Ha a predikció állapota egyezik a megfigyelttel -> Explained
    /// - Ha van illeszkedő hipotézis, de a predikció rossz -> Contradicted
    /// - Ha nincs hipotézis egyáltalán -> Uncertain
    /// - Különben (vannak hipotézisek, de egyik sem illik az akcióra) -> OutOfModel
    pub fn evaluate_prediction(
        &self,
        action: &Action,
        prediction: &Prediction,
        observation: &Observation,
    ) -> KnowledgeState {
        if prediction.state == observation.state {
            return KnowledgeState::Explained;
        }

        let action_program = vec![(action.prim, action.params.clone())];
        let matching_hypothesis = self.hypotheses.iter().any(|h| h.program == action_program);

        if matching_hypothesis {
            KnowledgeState::Contradicted
        } else if self.hypotheses.is_empty() {
            KnowledgeState::Uncertain
        } else {
            KnowledgeState::OutOfModel
        }
    }
'''

if method_anchor not in s:
    print("[ERROR] learn_from_error blokk nem található.")
    sys.exit(1)
s = s.replace(method_anchor, new_method)

p.write_text(s)
print("[1] WorldModel lib.rs frissítve: KnowledgeState, PredictionResidual, evaluate_prediction hozzáadva.")

# 2. Új tesztfájl
test_code = r'''
use athlesia_world_model::{
    WorldModel, KnowledgeState, Prediction, Observation, Action,
};
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

fn make_grid(x: usize, y: usize) -> Grid {
    let mut rows = [[0u8; 5]; 5];
    rows[y][x] = 1;
    build_grid(rows)
}

#[test]
fn evaluate_prediction_explained_when_state_matches() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(action.prim, action.params.clone())]);

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: prediction.state.clone() };

    let state = wm.evaluate_prediction(&action, &prediction, &observation);
    assert_eq!(state, KnowledgeState::Explained);
}

#[test]
fn evaluate_prediction_contradicted_when_known_but_wrong() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    wm.add_hypothesis(vec![(action.prim, action.params.clone())]);

    // Predikció jobbra, de megfigyelés balra (vagy ugyanaz a kezdő)
    let prediction = wm.predict(&initial, &action);
    let wrong_observation = Observation { state: initial.clone() }; // nem a predikció

    let state = wm.evaluate_prediction(&action, &prediction, &wrong_observation);
    assert_eq!(state, KnowledgeState::Contradicted);
}

#[test]
fn evaluate_prediction_uncertain_when_no_hypotheses() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let wm = WorldModel::new(initial.clone());

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: initial.clone() }; // rossz

    let state = wm.evaluate_prediction(&action, &prediction, &observation);
    assert_eq!(state, KnowledgeState::Uncertain);
}

#[test]
fn evaluate_prediction_out_of_model_when_no_matching_hypothesis() {
    let initial = make_grid(0, 0);
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let mut wm = WorldModel::new(initial.clone());
    // Van egy másik hipotézis (ReflectH), de nem az akcióra vonatkozik
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);

    let prediction = wm.predict(&initial, &action);
    let observation = Observation { state: initial.clone() }; // rossz

    let state = wm.evaluate_prediction(&action, &prediction, &observation);
    assert_eq!(state, KnowledgeState::OutOfModel);
}
'''

write_file("crates/athlesia-world-model/tests/knowledge_state_test.rs", test_code)
print("[2] knowledge_state_test.rs létrehozva.")

# 3. WorldModel tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-world-model"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] WorldModel tesztek nem mentek át.")
    sys.exit(1)

print("\n[SUCCESS] WorldModel tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 1: add KnowledgeState and evaluate_prediction to WorldModel"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
