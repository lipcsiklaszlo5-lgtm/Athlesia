use athlesia_types::{Grid, Program, Action};

/// Munkamemória: az aktuális lépés kontextusa.
#[derive(Debug, Clone)]
pub struct WorkingContext {
    pub current_grid: Grid,
    pub active_hypothesis: Option<u64>,
}

/// Epizodikus memória: egy megoldott példa (megőrizve a korábbi API-hoz).
#[derive(Debug, Clone)]
pub struct Episode {
    pub input: Grid,
    pub target: Grid,
    pub program: Program,
}

/// Interakciós esemény: a teljes előzmény naplózásához.
#[derive(Debug, Clone)]
pub enum InteractionEvent {
    Observation(Grid),
    Action(Action),
    HypothesisConfirmed(u64),
    HypothesisFalsified(u64),
}

/// Hosszú távú memória: játékok közötti tartós tudás.
/// Jelenleg a megtanult programokat tárolja indexként (később Knowledge Base-re mutat).
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

    /// Pillanatkép-tömörítés: duplikátumok eltávolítása.
    pub fn compress_snapshot(&mut self) {
        let mut unique: Vec<Program> = Vec::new();
        let mut usage: Vec<u32> = Vec::new();
        for (i, prog) in self.known_programs.iter().enumerate() {
            if let Some(pos) = unique.iter().position(|p| p == prog) {
                usage[pos] += self.program_usage[i];
            } else {
                unique.push(prog.clone());
                usage.push(self.program_usage[i]);
            }
        }
        self.known_programs = unique;
        self.program_usage = usage;
    }
}

/// A Manhattan Kernel memória-architektúrája, három időskálával.
#[derive(Debug, Default)]
pub struct Memory {
    pub working: Option<WorkingContext>,
    pub episodic: Vec<Episode>,
    pub event_log: Vec<InteractionEvent>,
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
    pub fn append_episode(&mut self, input: Grid, target: Grid, program: Program) {
        self.episodic.push(Episode {
            input,
            target,
            program: program.clone(),
        });
        self.long_term.add_program(program);
    }

    /// Interakciós esemény hozzáfűzése a naplóhoz (O(1)).
    pub fn append_event(&mut self, event: InteractionEvent) {
        self.event_log.push(event);
    }

    /// Visszaadja az epizódokat (régi API).
    pub fn episode_history(&self) -> &[Episode] {
        &self.episodic
    }

    /// Visszaadja a teljes interakciós naplót.
    pub fn interaction_history(&self) -> &[InteractionEvent] {
        &self.event_log
    }

    /// Pillanatkép: a hosszú távú memóriában tárolt programok tömörített másolata.
    pub fn snapshot(&self) -> Vec<Program> {
        let mut seen = std::collections::HashSet::new();
        let mut snapshot = Vec::new();
        for prog in &self.long_term.known_programs {
            if seen.insert(prog.clone()) {
                snapshot.push(prog.clone());
            }
        }
        snapshot
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

    /// Visszaadja a hosszú távú memóriában tárolt összes ismert programot.
    pub fn get_known_programs(&self) -> &[Program] {
        self.long_term.get_known_programs()
    }

    /// Hosszú távú memóriába emeli az epizodikus memóriában lévő összes programot.
    pub fn consolidate_known_programs(&mut self) {
        for ep in &self.episodic {
            self.long_term.add_program(ep.program.clone());
        }
        self.long_term.compress_snapshot();
    }
}
