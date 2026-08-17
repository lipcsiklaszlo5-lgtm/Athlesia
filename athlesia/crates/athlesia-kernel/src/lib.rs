
use athlesia_types::Action;
use athlesia_world_model::{WorldModel, HypothesisStatus};
use athlesia_planner::PlannerMode;

use athlesia_types::{Grid, Program, Budget};
use athlesia_memory::Memory;
use athlesia_knowledge::KnowledgeBase;
use athlesia_planner::Planner;
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_executor::run_program;

/// Integrációs funkció: megold egy feladatot a teljes csővezetéken.
/// Visszaadja a megtalált programot, ha sikerült.
pub fn solve_with_kernel(
    input: &Grid,
    target: &Grid,
    kb: &mut KnowledgeBase,
    memory: &mut Memory,
    planner: &Planner, wm: &athlesia_world_model::WorldModel,
    max_depth: usize,
) -> Option<Program> {
    // 1. Próbáljuk a memóriában lévő programokat
    if let Some(program) = memory.find_program_by_input(input) {
        let mut budget = Budget { max_steps: program.len() as u64 };
        if let Ok(output) = run_program(&program, input, &mut budget) {
            if output == *target {
                return Some(program);
            }
        }
    }

    // 2. Próbáljuk a tudásbázisból a makrókat
    for m in kb.get_all_macros() {
        let mut budget = Budget { max_steps: m.program.len() as u64 };
        if let Ok(output) = run_program(&m.program, input, &mut budget) {
            if output == *target {
                memory.add_episode(input.clone(), target.clone(), m.program.clone());
                return Some(m.program.clone());
            }
        }
    }

    // 3. Tervezés a keresőmotorral
    if let Some(program) = planner.plan(input, Some(target), wm, max_depth) {
        let verifier = Verifier;
        if verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
            memory.add_episode(input.clone(), target.clone(), program.clone());
            kb.add_macro(format!("solved_{}", kb.get_all_macros().len()), program.clone());
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
