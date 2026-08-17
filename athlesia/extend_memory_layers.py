#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
MEMORY_DIR = os.path.join(PROJECT, "crates", "athlesia-memory")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# --- lib.rs: háromrétegű memória ---
write_file(os.path.join(MEMORY_DIR, "src", "lib.rs"), r'''
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

    /// Visszaadja a hosszú távú memóriában tárolt összes ismert programot.
    pub fn get_known_programs(&self) -> &[Program] {
        self.long_term.get_known_programs()
    }
}
''')

print("[INFO] Háromrétegű memória implementálva.")

# --- Tesztek: meglévők frissítése + új réteg tesztek ---
write_file(os.path.join(MEMORY_DIR, "tests", "memory_test.rs"), r'''
use athlesia_memory::Memory;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn stores_and_retrieves_program_by_exact_input() {
    let mut mem = Memory::new();
    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let target = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    mem.add_episode(input.clone(), target, program.clone());

    let retrieved = mem.find_program_by_input(&input);
    assert_eq!(retrieved, Some(program));

    assert_eq!(mem.get_known_programs().len(), 1);
}

#[test]
fn does_not_duplicate_known_program_but_increments_usage() {
    let mut mem = Memory::new();
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([[0; 5]; 5]);
    let program = vec![(PrimName::ReflectH, Params::None)];

    mem.add_episode(input.clone(), target.clone(), program.clone());
    mem.add_episode(input.clone(), target, program.clone());

    assert_eq!(mem.get_known_programs().len(), 1);
    assert_eq!(mem.long_term.program_usage[0], 2);
}

#[test]
fn working_context_is_set_and_cleared() {
    let mut mem = Memory::new();
    let grid = build_grid([[0; 5]; 5]);

    mem.set_working_context(grid.clone(), Some(42));
    assert!(mem.working.is_some());
    assert_eq!(mem.working.as_ref().unwrap().active_hypothesis, Some(42));

    mem.clear_working_context();
    assert!(mem.working.is_none());
}
''')

print("[INFO] Háromrétegű memória tesztek frissítve.")

# --- Teszt futtatása ---
result = subprocess.run(["cargo", "test", "-p", "athlesia-memory"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Memória tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Memória tesztek zöldek.")

# --- Git commit és push ---
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Extend memory to three-layer architecture"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
