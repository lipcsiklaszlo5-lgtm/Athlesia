
use athlesia_types::{Grid, Program};

/// Egy epizód: bemenet, cél, és a hozzá megtalált program.
#[derive(Debug, Clone)]
pub struct Episode {
    pub input: Grid,
    pub target: Grid,
    pub program: Program,
}

/// Hosszú távú memória a megtanult programok és epizódok tárolására.
#[derive(Debug, Default)]
pub struct Memory {
    pub episodes: Vec<Episode>,
    pub known_programs: Vec<Program>,
}

impl Memory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_episode(&mut self, input: Grid, target: Grid, program: Program) {
        self.episodes.push(Episode {
            input,
            target,
            program: program.clone(),
        });
        if !self.known_programs.contains(&program) {
            self.known_programs.push(program);
        }
    }

    pub fn find_program_by_input(&self, input: &Grid) -> Option<Program> {
        for ep in &self.episodes {
            if ep.input == *input {
                return Some(ep.program.clone());
            }
        }
        None
    }

    pub fn get_known_programs(&self) -> &[Program] {
        &self.known_programs
    }
}
