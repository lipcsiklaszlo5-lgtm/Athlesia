
use athlesia_types::{Grid, Program};

/// Munkamemória: az aktuális lépés kontextusa.
#[derive(Debug, Clone)]
pub struct WorkingContext {
    pub current_grid: Grid,
    pub active_hypothesis: Option<u64>,
}

/// Epizodikus memória: egy megoldott példa.
#[derive(Debug, Clone)]
pub struct Episode {
    pub input: Grid,
    pub target: Grid,
    pub program: Program,
}

/// Hosszú távú memória: játékok közötti tartós tudás.
/// Jelenleg a megtanult programokat és használati számlálójukat tárolja.
#[derive(Debug, Default)]
pub struct LongTermMemory {
    pub known_programs: Vec<Program>,
    pub program_usage: Vec<u32>,
}

impl LongTermMemory {
    pub fn add_program(&mut self, program: Program) {
        if let Some(pos) = self.known_programs.iter().position(|p| p == &program) {
            self.program_usage[pos] += 1;
        } else {
            self.known_programs.push(program);
            self.program_usage.push(1);
        }
    }



    pub fn get_known_programs(&self) -> &[Program] {
        &self.known_programs
    }
}

/// A Manhattan Kernel memória-architektúrája, három időskálával.
#[derive(Debug, Default)]
pub struct Memory {
    pub working: Option<WorkingContext>,
    pub episodic: Vec<Episode>,
    pub long_term: LongTermMemory,
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Beállítja a pillanatnyi munkamemóriát.
    pub fn set_working_context(&mut self, grid: Grid, active_hypothesis: Option<u64>) {
        self.working = Some(WorkingContext {
            current_grid: grid,
            active_hypothesis,
        });
    }

    /// Törli a munkamemóriát (pl. a lépés végén).
    pub fn clear_working_context(&mut self) {
        self.working = None;
    }

    /// Hozzáad egy megoldott epizódot, és frissíti a hosszú távú memóriát.
    pub fn add_episode(&mut self, input: Grid, target: Grid, program: Program) {
        self.episodic.push(Episode {
            input,
            target,
            program: program.clone(),
        });
        self.long_term.add_program(program);
    }

    /// Pontos bemenetre megkeresi a már ismert programot.
    pub fn find_program_by_input(&self, input: &Grid) -> Option<Program> {
        for ep in &self.episodic {
            if ep.input == *input {
                return Some(ep.program.clone());
            }
        }
        None
    }

    /// Hosszú távú memóriába emeli az epizodikus memóriában lévő összes programot.
    /// Ez a játékok közötti tanulás alapja.
    pub fn consolidate_known_programs(&mut self) {
        for ep in &self.episodic {
            self.long_term.add_program(ep.program.clone());
        }
    }

    /// Visszaadja a hosszú távú memóriában tárolt összes ismert programot.
    pub fn get_known_programs(&self) -> &[Program] {
        self.long_term.get_known_programs()
    }
}
