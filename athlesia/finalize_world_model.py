#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. World Model lib.rs teljes újraírása a dokumentum szerint
write_file("crates/athlesia-world-model/src/lib.rs", r'''
use athlesia_types::{Grid, PrimName, Params, Program, Budget, Action};
use athlesia_executor::run_program;

/// A környezet állapotának típusa (a dokumentum szerint `State`).
pub type State = Grid;

/// Egy előrejelzés a jósolt állapottal és a konfidenciával.
#[derive(Debug, Clone, PartialEq)]
pub struct Prediction {
    pub state: State,
    pub confidence: f64,
}

/// Egy megfigyelt állapot.
#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub state: State,
}

/// A World Model frissítésének eredménye.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateResult {
    NoChange,
    Updated,
    Falsified,
}

/// Egy bizonytalansági lekérdezés.
#[derive(Debug, Clone)]
pub struct Query {
    pub state: State,
    pub action: Action,
}

/// Hipotézis-státusz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HypothesisStatus {
    Active,
    Confirmed,
    Falsified,
}

/// Átmeneti szabály-hipotézis evidencia-számlálókkal.
#[derive(Debug, Clone)]
pub struct TransitionHypothesis {
    pub id: u64,
    pub program: Program,
    pub evidence_for: u32,
    pub evidence_against: u32,
    pub status: HypothesisStatus,
}

#[derive(Debug, Default)]
pub struct WorldModel {
    pub current_state: State,
    pub hypotheses: Vec<TransitionHypothesis>,
    pub tick: u64,
}

impl WorldModel {
    pub fn new(initial_grid: Grid) -> Self {
        WorldModel {
            current_state: initial_grid,
            hypotheses: Vec::new(),
            tick: 0,
        }
    }

    pub fn add_hypothesis(&mut self, program: Program) -> u64 {
        let id = self.hypotheses.len() as u64;
        self.hypotheses.push(TransitionHypothesis {
            id,
            program,
            evidence_for: 0,
            evidence_against: 0,
            status: HypothesisStatus::Active,
        });
        id
    }

    /// A dokumentum szerinti `predict(state, action) -> Prediction`.
    pub fn predict(&self, state: &State, action: &Action) -> Prediction {
        let program = vec![(action.prim, action.params)];
        let mut budget = Budget { max_steps: 1 };
        let predicted_state = run_program(&program, state, &mut budget)
            .unwrap_or_else(|_| state.clone());

        let mut confidence = 0.5;
        for hyp in &self.hypotheses {
            if hyp.program == program {
                if hyp.evidence_for + hyp.evidence_against > 0 {
                    confidence = (hyp.evidence_for as f64 + 1.0)
                        / (hyp.evidence_for as f64 + hyp.evidence_against as f64 + 2.0);
                }
                break;
            }
        }

        Prediction {
            state: predicted_state,
            confidence,
        }
    }

    /// A dokumentum szerinti `update(observation) -> UpdateResult`.
    pub fn update(&mut self, observation: &Observation) -> UpdateResult {
        let previous_state = self.current_state.clone();
        let observed_state = observation.state.clone();

        let mut changed = false;
        let mut any_falsified = false;

        for hyp in &mut self.hypotheses {
            let predicted = {
                let program = &hyp.program;
                let mut budget = Budget { max_steps: program.len() as u64 };
                run_program(program, &previous_state, &mut budget).ok()
            };

            match predicted {
                Some(pred) if pred == observed_state => {
                    hyp.evidence_for += 1;
                    if hyp.evidence_against == 0 && hyp.evidence_for >= 3 {
                        hyp.status = HypothesisStatus::Confirmed;
                    }
                    changed = true;
                }
                Some(_) => {
                    hyp.evidence_against += 1;
                    hyp.status = HypothesisStatus::Falsified;
                    any_falsified = true;
                    changed = true;
                }
                None => {}
            }
        }

        self.current_state = observed_state;
        self.tick += 1;

        if any_falsified {
            UpdateResult::Falsified
        } else if changed {
            UpdateResult::Updated
        } else {
            UpdateResult::NoChange
        }
    }

    /// A dokumentum szerinti `uncertainty(query) -> f64`.
    pub fn uncertainty(&self, query: &Query) -> f64 {
        let pred = self.predict(&query.state, &query.action);
        1.0 - pred.confidence
    }
}
''')
print("[1] World Model lib.rs teljesen újraírva.")

# 2. Planner frissítése: az uncertainty hívása mostantól Query-t vár
p = pathlib.Path("crates/athlesia-planner/src/lib.rs")
s = p.read_text()
old = "let uncertainty = wm.uncertainty(current, &action);"
new = "let query = athlesia_world_model::Query { state: current.clone(), action };\n                let uncertainty = wm.uncertainty(&query);"
if old in s:
    s = s.replace(old, new)
    p.write_text(s)
    print("[2] Planner uncertainty hívása frissítve.")
else:
    print("[WARN] Planner uncertainty hívás nem található, kézi ellenőrzés kell.")

# 3. World Model tesztek teljes újraírása
write_file("crates/athlesia-world-model/tests/world_model_full_test.rs", r'''
use athlesia_world_model::{WorldModel, HypothesisStatus, Prediction, Observation, UpdateResult, Query};
use athlesia_types::{Grid, PrimName, Params, Action};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn predict_returns_prediction_with_confidence() {
    let wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let expected = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let pred = wm.predict(&input, &action);
    assert_eq!(pred.state, expected);
    assert_eq!(pred.confidence, 0.5); // nincs hipotézis
}

#[test]
fn update_confirms_hypothesis_after_three_successes() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    wm.add_hypothesis(program.clone());

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    wm.update(&Observation { state: obs.clone() });
    wm.update(&Observation { state: obs.clone() });
    wm.update(&Observation { state: obs.clone() });

    assert_eq!(wm.hypotheses[0].evidence_for, 3);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Confirmed);
}

#[test]
fn update_falsifies_wrong_hypothesis() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let program = vec![(PrimName::ReflectH, Params::None)];
    wm.add_hypothesis(program.clone());

    let input = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = input.clone(); // nem tükrözés

    let result = wm.update(&Observation { state: obs });
    assert_eq!(result, UpdateResult::Falsified);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Falsified);
}

#[test]
fn uncertainty_decreases_with_confidence() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));
    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };
    let program = vec![(action.prim, action.params)];
    wm.add_hypothesis(program.clone());

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    wm.update(&Observation { state: obs.clone() });
    wm.update(&Observation { state: obs.clone() });
    wm.update(&Observation { state: obs.clone() });

    let query = Query { state: input.clone(), action };
    let uncertainty = wm.uncertainty(&query);
    assert!(uncertainty < 0.5);
}
''')
print("[3] World Model tesztek hozzáadva.")

# 4. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-world-model"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] World Model tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] World Model tesztek zöldek.")

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Finalize World Model module with Prediction, Observation, UpdateResult, Query"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
