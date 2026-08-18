
pub mod cognitive;
use serde::Deserialize;
use athlesia_types::{Grid, Color, Action, PrimName, Params, Program, Budget};
use athlesia_perception::perceive;
use athlesia_world_model::WorldModel;
use athlesia_memory::{Memory, InteractionEvent};
use athlesia_knowledge::KnowledgeBase;
use athlesia_verifier::{Verifier, VerificationResult};

use athlesia_planner::{Planner, PlannerMode};
use athlesia_core::CoreEngine;


/// ARC példa struktúra JSON-ből.
#[derive(Debug, Deserialize)]
pub struct ArcExample {
    pub input: Vec<Vec<u8>>,
    pub output: Vec<Vec<u8>>,
}

/// ARC feladat struktúra.
#[derive(Debug, Deserialize)]
pub struct ArcTask {
    pub train: Vec<ArcExample>,
    pub test: Vec<ArcExample>,
}

/// Grid létrehozása sorvektorokból.
pub fn grid_from_rows(rows: &[Vec<u8>]) -> Grid {
    let height = rows.len() as u8;
    let width = if height > 0 { rows[0].len() as u8 } else { 0 };
    let mut cells = Vec::with_capacity((width as usize) * (height as usize));
    for row in rows {
        for &c in row {
            cells.push(Color(c));
        }
    }
    Grid { width, height, cells }
}

/// Interaktív ágens, amely a Manhattan Kernel moduljait használja.
pub struct Agent {
    pub wm: WorldModel,
    pub planner: Planner,
    pub memory: Memory,
    pub kb: KnowledgeBase,
    pub core: CoreEngine,
}

impl Agent {
    pub fn new(initial_grid: Grid) -> Self {
        Agent {
            wm: WorldModel::new(initial_grid.clone()),
            planner: Planner::new(PlannerMode::Exploration),
            memory: Memory::new(),
            kb: KnowledgeBase::new(),
            core: CoreEngine::new(),
        }
    }

    /// Feltáró lépés: a Planner kiválaszt egy akciót, hozzáadjuk a hipotézisekhez.
    pub fn step(&mut self, current: &Grid, target: Option<&Grid>) -> Action {
        let program = if let Some(goal) = target {
            let goal_planner = Planner::new(PlannerMode::GoalDirected);
            goal_planner.plan(current, Some(goal), &self.wm, 3)
        } else {
            self.planner.plan(current, None, &self.wm, 1)
        };

        if let Some(program) = program {
            let (prim, params) = program[0].clone();
            let action = Action { prim, params };

            let prog = vec![(prim, params)];
            if !self.wm.hypotheses.iter().any(|h| h.program == prog) {
                self.wm.add_hypothesis(prog);
            }
            self.memory.append_event(InteractionEvent::Action(action));
            return action;
        }

        // Biztonsági akció
        Action { prim: PrimName::Translate, params: Params::Translate(0, 0) }
    }

    /// A környezet megfigyelése után frissíti a világmodellt.
    pub fn update(&mut self, observation: &Grid) {
        let obs = athlesia_world_model::Observation { state: observation.clone() };
        self.wm.update(&obs);
        self.memory.append_event(InteractionEvent::Observation(observation.clone()));
    }

    /// Megerősített hipotézisek makrósítása.
    pub fn consolidate_learned_macros(&mut self) {
        for hyp in &self.wm.hypotheses {
            if hyp.status == athlesia_world_model::HypothesisStatus::Confirmed {
                self.kb.add_macro(
                    format!("learned_{}", self.kb.get_all_macros().len()),
                    hyp.program.clone(),
                );
                self.memory.long_term.add_program(hyp.program.clone());
            }
        }
    }
}

/// Teljes megoldási pipeline egy ARC feladat JSON-re.
pub fn solve_arc_json(task_json: &str) -> (Option<Grid>, Grid) {
    let task: ArcTask = serde_json::from_str(task_json).expect("Hibás ARC JSON");

    let mut agent = Agent::new(grid_from_rows(&task.train[0].input));

    // Tanulás a train példákon
    for example in &task.train {
        let input_grid = grid_from_rows(&example.input);
        let output_grid = grid_from_rows(&example.output);

        // Percepció
        let _perception = perceive(Some(&input_grid), &output_grid);

        // Cél-irányított lépés
        let action = agent.step(&input_grid, Some(&output_grid));
        let program = vec![(action.prim, action.params)];

        // Verifikáció
        let verifier = Verifier::new();
        if verifier.verify(&program, &vec![(input_grid.clone(), output_grid.clone())]) == VerificationResult::Accept {
            let id = agent.core.known_programs.len() as u64;
            agent.core.known_programs.push(program.clone());
            agent.core.meta.record_success_in_context(
                athlesia_features::extract_features(&input_grid),
                id,
            );
            agent.memory.append_episode(input_grid.clone(), output_grid.clone(), program);
        }
    }

    // Test predikció
    let test_input = grid_from_rows(&task.test[0].input);
    let test_expected = grid_from_rows(&task.test[0].output);

    let mut predicted = None;
    for program in agent.core.known_programs.iter().rev() {
        let mut budget = Budget { max_steps: 10, max_depth: 100 };
        if let Ok(output) = athlesia_executor::run_program(program, &test_input, &mut budget) {
            predicted = Some(output);
            break;
        }
    }

    (predicted, test_expected)
}

/// Egyszerű kernel szintű megoldás adott bemenet-cél párra.
pub fn solve_with_kernel(
    input: &Grid,
    target: &Grid,
    kb: &mut KnowledgeBase,
    memory: &mut Memory,
    planner: &Planner,
    wm: &WorldModel,
    core: &mut CoreEngine,
    max_depth: usize,
) -> Option<Program> {
    // 1. CoreEngine próbálkozás
    if let Some(program) = core.solve(input, target) {
        memory.append_episode(input.clone(), target.clone(), program.clone());
        if !kb.get_all_macros().iter().any(|m| m.program == program) {
            kb.add_macro(format!("solved_{}", kb.get_all_macros().len()), program.clone());
        }
        return Some(program);
    }

    // 2. Planner a Search Engine-nel
    if let Some(program) = planner.plan(input, Some(target), wm, max_depth) {
        let verifier = Verifier::new();
        if verifier.verify(&program, &vec![(input.clone(), target.clone())]) == VerificationResult::Accept {
            memory.append_episode(input.clone(), target.clone(), program.clone());
            if !kb.get_all_macros().iter().any(|m| m.program == program) {
                kb.add_macro(format!("solved_{}", kb.get_all_macros().len()), program.clone());
            }
            return Some(program);
        }
    }

    None
}
