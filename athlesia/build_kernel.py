#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
KERNEL_DIR = os.path.join(PROJECT, "crates", "athlesia-kernel")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-kernel" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory", "crates/athlesia-knowledge", "crates/athlesia-abstraction", "crates/athlesia-hypothesis", "crates/athlesia-planner"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory", "crates/athlesia-knowledge", "crates/athlesia-abstraction", "crates/athlesia-hypothesis", "crates/athlesia-planner", "crates/athlesia-kernel"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Kernel crate létrehozása (bináris)
os.makedirs(os.path.join(KERNEL_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(KERNEL_DIR, "tests"), exist_ok=True)

write_file(os.path.join(KERNEL_DIR, "Cargo.toml"), '''[package]
name = "athlesia-kernel"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-executor = { path = "../athlesia-executor" }
athlesia-perception = { path = "../athlesia-perception" }
athlesia-world-model = { path = "../athlesia-world-model" }
athlesia-features = { path = "../athlesia-features" }
athlesia-metalearner = { path = "../athlesia-metalearner" }
athlesia-verifier = { path = "../athlesia-verifier" }
athlesia-synthesis = { path = "../athlesia-synthesis" }
athlesia-core = { path = "../athlesia-core" }
athlesia-search = { path = "../athlesia-search" }
athlesia-memory = { path = "../athlesia-memory" }
athlesia-knowledge = { path = "../athlesia-knowledge" }
athlesia-abstraction = { path = "../athlesia-abstraction" }
athlesia-hypothesis = { path = "../athlesia-hypothesis" }
athlesia-planner = { path = "../athlesia-planner" }
''')

write_file(os.path.join(KERNEL_DIR, "src", "main.rs"), r'''
use athlesia_types::{Grid, PrimName, Params, Program};
use athlesia_features::extract_features;
use athlesia_memory::Memory;
use athlesia_knowledge::KnowledgeBase;
use athlesia_abstraction::AbstractionEngine;
use athlesia_hypothesis::{HypothesisProposer, StaticProposer};
use athlesia_planner::{Planner, PlannerMode};
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_executor::run_program;

fn main() {
    println!("=== Athlesia Kernel ===");
    println!("Manhattan Kernel core inicializálva.");
    println!("Futtasd a teszteket: cargo test -p athlesia-kernel");
}

/// Integrációs funkció: megold egy feladatot a teljes csővezetéken.
/// Visszaadja a megtalált programot, ha sikerült.
pub fn solve_with_kernel(
    input: &Grid,
    target: &Grid,
    kb: &mut KnowledgeBase,
    memory: &mut Memory,
    planner: &Planner,
    max_depth: usize,
) -> Option<Program> {
    // 1. Próbáljuk a memóriában lévő programokat
    if let Some(program) = memory.find_program_by_input(input) {
        let mut budget = athlesia_types::Budget { max_steps: program.len() as u64 };
        if let Ok(output) = run_program(&program, input, &mut budget) {
            if output == *target {
                return Some(program);
            }
        }
    }

    // 2. Próbáljuk a tudásbázisból a makrókat
    for m in kb.get_all_macros() {
        let mut budget = athlesia_types::Budget { max_steps: m.program.len() as u64 };
        if let Ok(output) = run_program(&m.program, input, &mut budget) {
            if output == *target {
                memory.add_episode(input.clone(), target.clone(), m.program.clone());
                return Some(m.program.clone());
            }
        }
    }

    // 3. Tervezés a keresőmotorral
    if let Some(program) = planner.plan(input, Some(target), max_depth) {
        // Verifikáljuk
        let verifier = Verifier;
        if verifier.verify(&program, &[(input.clone(), target.clone())]) == VerificationResult::Accept {
            memory.add_episode(input.clone(), target.clone(), program.clone());
            kb.add_macro(format!("solved_{}", kb.get_all_macros().len()), program.clone());
            return Some(program);
        }
    }

    None
}
''')

# 3. Teszt
write_file(os.path.join(KERNEL_DIR, "tests", "kernel_test.rs"), r'''
use athlesia_kernel::solve_with_kernel;
use athlesia_memory::Memory;
use athlesia_knowledge::KnowledgeBase;
use athlesia_planner::{Planner, PlannerMode};
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn kernel_solves_simple_translate() {
    let mut kb = KnowledgeBase::new();
    let mut mem = Memory::new();
    let planner = Planner::new(PlannerMode::GoalDirected);

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

    let program = solve_with_kernel(
        &input,
        &target,
        &mut kb,
        &mut mem,
        &planner,
        2,
    );

    assert!(program.is_some());
    // A megoldásnak Translate(1,0)-nek kell lennie
    assert_eq!(
        program.unwrap(),
        vec![(PrimName::Translate, Params::Translate(1, 0))]
    );
}
''')

print("[INFO] Kernel crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-kernel"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Kernel tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Kernel tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-kernel binary integrating all modules"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
