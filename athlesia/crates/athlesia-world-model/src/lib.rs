use athlesia_types::{Grid, PrimName, Params, Program, Budget, Action};
use athlesia_executor::run_program;

pub type State = Grid;

#[derive(Debug, Clone, PartialEq)]
pub struct Prediction {
    pub state: State,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub state: State,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateResult {
    NoChange,
    Updated,
    Falsified,
}

#[derive(Debug, Clone)]
pub struct Query {
    pub state: State,
    pub action: Action,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HypothesisStatus {
    Active,
    Confirmed,
    Falsified,
}

#[derive(Debug, Clone)]
pub struct TransitionHypothesis {
    pub id: u64,
    pub program: Program,
    pub evidence_for: u32,
    pub evidence_against: u32,
    pub status: HypothesisStatus,
}

/// Explicit prior/belief egy fogalom vagy szabály megbízhatóságáról.
#[derive(Debug, Clone)]
pub struct Belief {
    pub concept_id: u64,
    pub confidence: f32,
    pub evidence_for: usize,
    pub evidence_against: usize,
}

/// Predikciós hiba: miért nem egyezett a predikció a megfigyeléssel.
#[derive(Debug, Clone)]
pub struct PredictionError {
    pub expected: Grid,
    pub observed: Grid,
    pub summary: String,
    pub feature_mismatch: usize,
}

#[derive(Debug)]
pub struct WorldModel {
    pub current_state: State,
    pub hypotheses: Vec<TransitionHypothesis>,
    pub tick: u64,
    pub recent_errors: Vec<PredictionError>,
}

impl WorldModel {
    pub fn new(initial_grid: Grid) -> Self {
        WorldModel {
            current_state: initial_grid,
            hypotheses: Vec::new(),
            tick: 0,
            recent_errors: Vec::new(),
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

    pub fn uncertainty(&self, query: &Query) -> f64 {
        let pred = self.predict(&query.state, &query.action);
        1.0 - pred.confidence
    }

    /// A modell frissítése predikciós hiba alapján.
    /// A hibát egy hipotézishez kötjük, és csökkentjük annak konfidenciáját.
    pub fn learn_from_error(&mut self, hypothesis_id: u64, _error: &PredictionError) {
        if let Some(hyp) = self.hypotheses.iter_mut().find(|h| h.id == hypothesis_id) {
            hyp.evidence_against += 1;
            hyp.status = HypothesisStatus::Falsified;
        }
    }

    /// A predikciós hiba tárolása későbbi absztrakcióhoz.
    pub fn record_prediction_error(&mut self, error: PredictionError) {
        self.recent_errors.push(error);
    }
}
