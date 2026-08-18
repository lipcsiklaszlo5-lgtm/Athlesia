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

    /// Strukturált predikciós reziduális előállítása.
    /// - mismatch_score: 0.0 = nincs eltérés, 1.0 = teljes eltérés vagy dimenzióeltérés.
    /// - unexplained_features: jelenleg alapvető "pixel_mismatch" jelzés, ha eltérés van.
    pub fn compute_prediction_residual(
        &self,
        _action: &Action,
        prediction: &Prediction,
        observation: &Observation,
    ) -> PredictionResidual {
        let mismatch_score = if prediction.state.width != observation.state.width
            || prediction.state.height != observation.state.height
        {
            1.0
        } else {
            let total = (prediction.state.width as usize) * (prediction.state.height as usize);
            if total == 0 {
                0.0
            } else {
                let mismatches = prediction
                    .state
                    .cells
                    .iter()
                    .zip(observation.state.cells.iter())
                    .filter(|(a, b)| a != b)
                    .count();
                mismatches as f64 / total as f64
            }
        };

        let mut unexplained_features = Vec::new();
        if mismatch_score > 0.0 {
            unexplained_features.push("pixel_mismatch".to_string());
        }

        PredictionResidual {
            expected_observation: Observation { state: prediction.state.clone() },
            observed_observation: observation.clone(),
            mismatch_score,
            unexplained_features,
        }
    }

    /// A predikciós hiba tárolása későbbi absztrakcióhoz.
    pub fn record_prediction_error(&mut self, error: PredictionError) {
        self.recent_errors.push(error);
    }
}
