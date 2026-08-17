
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
