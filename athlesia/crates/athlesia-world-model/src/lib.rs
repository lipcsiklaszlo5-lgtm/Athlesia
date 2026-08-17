
use athlesia_types::{Action, Grid, Program};
use athlesia_executor::run_program;
use athlesia_types::Budget;

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct WorldModel {
    pub current_grid: Grid,
    pub hypotheses: Vec<TransitionHypothesis>,
    pub tick: u64,
}

impl WorldModel {
    pub fn new(initial_grid: Grid) -> Self {
        WorldModel {
            current_grid: initial_grid,
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

    pub fn predict(&self, state: &Grid, action: &Action) -> (Grid, f64) {
        let program = vec![(action.prim, action.params)];
        let mut budget = Budget { max_steps: 1 };
        let predicted_grid = run_program(&program, state, &mut budget)
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
        (predicted_grid, confidence)
    }


    /// Bizonytalanság: 1 - konfidencia. Determinisztikus, mert a konfidencia
    /// az evidencia-számlálókból jön, nem valószínűségi mintavételből.
    pub fn uncertainty(&self, state: &Grid, action: &Action) -> f64 {
        let (_, confidence) = self.predict(state, action);
        1.0 - confidence
    }
    pub fn update(&mut self, previous_grid: &Grid, observed_grid: &Grid) {
        for hyp in &mut self.hypotheses {
            let mut budget = Budget { max_steps: 1000 };
            let predicted = run_program(&hyp.program, previous_grid, &mut budget).ok();
            if let Some(predicted) = predicted {
                if predicted == *observed_grid {
                    hyp.evidence_for += 1;
                    if hyp.evidence_against == 0 && hyp.evidence_for >= 3 {
                        hyp.status = HypothesisStatus::Confirmed;
                    }
                } else {
                    hyp.evidence_against += 1;
                    hyp.status = HypothesisStatus::Falsified;
                }
            }
        }
        self.tick += 1;
    }
}
