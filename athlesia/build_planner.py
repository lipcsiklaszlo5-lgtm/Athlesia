#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
PLAN_DIR = os.path.join(PROJECT, "crates", "athlesia-planner")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-planner" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory", "crates/athlesia-knowledge", "crates/athlesia-abstraction", "crates/athlesia-hypothesis"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier", "crates/athlesia-synthesis", "crates/athlesia-core", "crates/athlesia-search", "crates/athlesia-memory", "crates/athlesia-knowledge", "crates/athlesia-abstraction", "crates/athlesia-hypothesis", "crates/athlesia-planner"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Planner crate létrehozása
os.makedirs(os.path.join(PLAN_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(PLAN_DIR, "tests"), exist_ok=True)

write_file(os.path.join(PLAN_DIR, "Cargo.toml"), '''[package]
name = "athlesia-planner"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-search = { path = "../athlesia-search" }
''')

write_file(os.path.join(PLAN_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Grid, PrimName, Params, Program};
use athlesia_search::search;

/// A tervező üzemmódja.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannerMode {
    GoalDirected,
    Exploration,
}

/// A Manhattan Kernel tervezője.
///
/// - Cél-irányított mód: egy ismert cél-grid eléréséhez készít programot.
///   Ez a legegyszerűbb, de hasznos proxy a valódi cél-irányított tervezésre.
/// - Feltáró mód: mivel még nincs cél, egy alapértelmezett, determinisztikus
///   akciót javasol, amely a későbbiekben információnyereség-alapúvá bővülhet.
#[derive(Debug)]
pub struct Planner {
    pub mode: PlannerMode,
}

impl Planner {
    pub fn new(mode: PlannerMode) -> Self {
        Planner { mode }
    }

    /// Terv készítése. Ha a cél ismert, és elérhető max_depth lépésen belül,
    /// a Search Engine segítségével visszaad egy programszekvenciát.
    pub fn plan(&self, current: &Grid, target: Option<&Grid>, max_depth: usize) -> Option<Program> {
        match self.mode {
            PlannerMode::GoalDirected => {
                // Ha nincs cél, nincs cél-irányított terv
                target?;
                search(current, target.unwrap(), max_depth)
            }
            PlannerMode::Exploration => {
                // Determinisztikus feltáró akció: egy lépés jobbra.
                // A későbbi változatban ez bizonytalanság-alapú lesz.
                Some(vec![(PrimName::Translate, Params::Translate(1, 0))])
            }
        }
    }
}
''')

# 3. Tesztek
write_file(os.path.join(PLAN_DIR, "tests", "planner_test.rs"), r'''
use athlesia_planner::{Planner, PlannerMode};
use athlesia_types::Grid;

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn goal_directed_planner_finds_reachable_target() {
    let planner = Planner::new(PlannerMode::GoalDirected);

    let current = build_grid([
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

    let plan = planner.plan(&current, Some(&target), 2);
    assert!(plan.is_some());
    assert!(!plan.unwrap().is_empty());
}

#[test]
fn goal_directed_planner_returns_none_when_no_solution() {
    let planner = Planner::new(PlannerMode::GoalDirected);

    let current = build_grid([[0; 5]; 5]);
    let target = build_grid([
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
        [9, 9, 9, 9, 9],
    ]);

    let plan = planner.plan(&current, Some(&target), 2);
    assert!(plan.is_none());
}

#[test]
fn exploration_planner_returns_default_action() {
    let planner = Planner::new(PlannerMode::Exploration);

    let current = build_grid([[0; 5]; 5]);
    let plan = planner.plan(&current, None, 1);
    assert!(plan.is_some());
    assert_eq!(plan.unwrap().len(), 1);
}
''')

print("[INFO] Planner crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-planner"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Planner tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Planner tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-planner module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
