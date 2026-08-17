#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
MEMORY_DIR = os.path.join(PROJECT, "crates", "athlesia-memory")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-memory" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Memory crate létrehozása
os.makedirs(os.path.join(MEMORY_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(MEMORY_DIR, "tests"), exist_ok=True)

write_file(os.path.join(MEMORY_DIR, "Cargo.toml"), '''[package]
name = "athlesia-memory"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
''')

write_file(os.path.join(MEMORY_DIR, "src", "lib.rs"), r'''
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
''')

# 3. Tesztek
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
fn does_not_duplicate_known_program() {
    let mut mem = Memory::new();
    let input = build_grid([[0; 5]; 5]);
    let target = build_grid([[0; 5]; 5]);
    let program = vec![(PrimName::ReflectH, Params::None)];

    mem.add_episode(input.clone(), target.clone(), program.clone());
    mem.add_episode(input.clone(), target, program.clone());

    assert_eq!(mem.get_known_programs().len(), 1);
}
''')

print("[INFO] Memory crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-memory"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Memory tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Memory tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-memory module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
