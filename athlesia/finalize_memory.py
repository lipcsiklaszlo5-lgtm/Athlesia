#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Memory lib.rs teljes újraírása a dokumentum 6. fejezete szerint
write_file("crates/athlesia-memory/src/lib.rs", r'''
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
    pub fn append(&mut self, input: Grid, target: Grid, program: Program) {
        self.episodic.push(Episode {
            input,
            target,
            program: program.clone(),
        });
        self.long_term.add_program(program);
    }

    /// Visszaadja az összes epizódot (előzmény).
    pub fn episode_history(&self) -> &[Episode] {
        &self.episodic
    }

    /// Snapshot: az összes ismert program másolata.
    pub fn snapshot(&self) -> Vec<Program> {
        self.long_term.known_programs.clone()
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
    }
}
''')
print("[1] Memory lib.rs teljesen újraírva.")

# 2. Tesztek hozzáadása
write_file("crates/athlesia-memory/tests/memory_full_test.rs", r'''
use athlesia_memory::Memory;
use athlesia_types::{Grid, PrimName, Params};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn append_stores_episode_and_known_program() {
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

    mem.append(input, target, program.clone());

    assert_eq!(mem.episode_history().len(), 1);
    assert_eq!(mem.get_known_programs().len(), 1);
    assert_eq!(mem.snapshot(), vec![program]);
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

#[test]
fn consolidate_known_programs_moves_all() {
    let mut mem = Memory::new();
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([[0; 5]; 5]);
    let program = vec![(PrimName::ReflectH, Params::None)];

    mem.append(input.clone(), target.clone(), program.clone());
    mem.append(input, target, program.clone());

    assert_eq!(mem.snapshot().len(), 1, "Duplikáció miatt eggyel kell lennie");

    mem.consolidate_known_programs();
    assert_eq!(mem.snapshot().len(), 1);
}
''')
print("[2] Memory tesztek hozzáadva.")

# 3. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-memory", "--test", "memory_full_test"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Memory tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Memory tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Finalize Memory module with append, episode_history, snapshot"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
