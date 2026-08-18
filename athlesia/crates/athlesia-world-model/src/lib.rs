
use athlesia_types::{Grid, Program, Budget, Action};
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

#[derive(Debug)]
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
        let program = vec![(action.prim, action.params.clone())];
        let mut budget = Budget { max_steps: 1, max_depth: 100 };
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
                let mut budget = Budget { max_steps: program.len() as u64, max_depth: 100 };
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
