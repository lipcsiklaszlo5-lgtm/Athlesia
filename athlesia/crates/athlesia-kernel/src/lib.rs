use serde::Deserialize;

use athlesia_types::Action;
use athlesia_world_model::{WorldModel, HypothesisStatus};
use athlesia_planner::PlannerMode;

use athlesia_types::{Grid, Program, Color};
use athlesia_memory::Memory;
use athlesia_knowledge::KnowledgeBase;
use athlesia_planner::Planner;
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_core::CoreEngine;

/// Integrációs funkció: megold egy feladatot a teljes csővezetéken.
/// Visszaadja a megtalált programot, ha sikerült.
pub fn solve_with_kernel(
    input: &Grid,
    target: &Grid,
    kb: &mut KnowledgeBase,
    memory: &mut Memory,
    planner: &Planner,
    wm: &athlesia_world_model::WorldModel,
    core: &mut CoreEngine,
    max_depth: usize,
) -> Option<Program> {
    // 1. Próbáljuk a CoreEngine-t, amely a memóriát, a MetaLearnert,
    //    a Synthesis Engine-t és a Search Engine-t is használja.
    if let Some(program) = core.solve(input, target) {
        memory.add_episode(input.clone(), target.clone(), program.clone());
        if !kb.get_all_macros().iter().any(|m| m.program == program) {
            kb.add_macro(format!("solved_{}", kb.get_all_macros().len()), program.clone());
        }
        return Some(program);
    }

    // 2. Ha a CoreEngine nem járt sikerrel, próbáljuk a Plannert.
    //    (pl. ha a CoreEngine nem ismerte fel, de a Planner mégis talál megoldást)
    if let Some(program) = planner.plan(input, Some(target), wm, max_depth) {
        let verifier = Verifier;
        if verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
            memory.add_episode(input.clone(), target.clone(), program.clone());
            if !kb.get_all_macros().iter().any(|m| m.program == program) {
                kb.add_macro(format!("solved_{}", kb.get_all_macros().len()), program.clone());
            }
            return Some(program);
        }
    }

    None
}

/// Interaktív ágens, amely a Manhattan Kernel moduljait használja.
pub struct Agent {
    pub wm: WorldModel,
    pub planner: Planner,
}

impl Agent {
    pub fn new(initial_grid: Grid) -> Self {
        Agent {
            wm: WorldModel::new(initial_grid),
            planner: Planner::new(PlannerMode::Exploration),
        }
    }

    /// Lépés: ha adott cél, és a világmodell elég magabiztos, cél-irányított
    /// tervezést használ, egyébként feltár.
    /// A kiválasztott akciót minden esetben hozzáadjuk a hipotézisekhez.
    pub fn step(&mut self, current: &Grid, target: Option<&Grid>) -> Action {
        if let Some(goal) = target {
            // Próbáljunk cél-irányított tervet készíteni
            let goal_planner = Planner::new(PlannerMode::GoalDirected);
            if let Some(program) = goal_planner.plan(current, Some(goal), &self.wm, 3) {
                let (prim, params) = program[0].clone();
                let action = Action { prim, params };

                let prog = vec![(prim, params)];
                if !self.wm.hypotheses.iter().any(|h| h.program == prog) {
                    self.wm.add_hypothesis(prog);
                }
                return action;
            }
        }

        // Feltáró ág, ha nincs cél vagy a cél-irányított keresés kudarcot vall
        let program = self
            .planner
            .plan(current, None, &self.wm, 1)
            .expect("Feltáró módban mindig kell lennie akciónak");

        let (prim, params) = program[0].clone();
        let action = Action { prim, params };

        let prog = vec![(prim, params)];
        if !self.wm.hypotheses.iter().any(|h| h.program == prog) {
            self.wm.add_hypothesis(prog);
        }

        action
    }


    /// A WorldModel megerősített hipotéziseit makróként átemeli a Knowledge Base-be,
    /// és a memóriába is. Ez a játékon belüli tanulás lezárása.
    pub fn consolidate_learned_macros(&mut self, kb: &mut KnowledgeBase, memory: &mut Memory) {
        for hyp in &self.wm.hypotheses {
            if hyp.status == HypothesisStatus::Confirmed {
                kb.add_macro(format!("learned_{}", kb.get_all_macros().len()), hyp.program.clone());
                memory.long_term.add_program(hyp.program.clone());
            }
        }
    }
    /// A környezet megfigyelése után frissíti a WorldModel-t.
    pub fn update(&mut self, previous: &Grid, observed: &Grid) {
        self.wm.update(previous, observed);
    }
}

#[derive(Debug, Deserialize)]
pub struct ArcExample {
    pub input: Vec<Vec<u8>>,
    pub output: Vec<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct ArcTask {
    pub train: Vec<ArcExample>,
    pub test: Vec<ArcExample>,
}

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

/// Betölt egy ARC feladatot JSON-ből, tanít a train példákon,
/// majd megpróbálja megoldani az első test példát.
/// Visszaadja a prediktált gridet (ha sikerült) és az elvárt gridet.
pub fn solve_arc_json(task_json: &str) -> (Option<Grid>, Grid) {
    let task: ArcTask = serde_json::from_str(task_json).expect("Hibás ARC JSON");

    let mut core = CoreEngine::new();
    let mut mem = Memory::new();
    let mut kb = KnowledgeBase::new();
    let planner = Planner::new(PlannerMode::GoalDirected);
    let wm = WorldModel::new(Grid::new(0, 0));

    // Tanulás a train példákon
    for example in &task.train {
        let input_grid = grid_from_rows(&example.input);
        let output_grid = grid_from_rows(&example.output);
        let _ = solve_with_kernel(
            &input_grid,
            &output_grid,
            &mut kb,
            &mut mem,
            &planner,
            &wm,
            &mut core,
            5,
        );
    }

    let test_input = grid_from_rows(&task.test[0].input);
    let test_expected = grid_from_rows(&task.test[0].output);

    // A legjobbnak ítélt programot a core known_programs-ből próbáljuk
    let mut predicted = None;
    for program in core.known_programs.iter().rev() {
        let mut budget = athlesia_types::Budget { max_steps: 10 };
        if let Ok(output) = athlesia_executor::run_program(program, &test_input, &mut budget) {
            predicted = Some(output);
            break;
        }
    }

    (predicted, test_expected)
}
