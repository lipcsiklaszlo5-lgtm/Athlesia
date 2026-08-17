#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
WORLD_DIR = os.path.join(PROJECT, "crates", "athlesia-world-model")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. .gitignore létrehozása, hogy a target mappa ne kerüljön be többé
write_file(".gitignore", "target/\n*.log\n")
print("[INFO] .gitignore létrehozva.")

# 2. Workspace Cargo.toml frissítése az új crate-tel
workspace_content = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-world-model" not in workspace_content:
    workspace_content = workspace_content.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model"]'
    )
    write_file(WORKSPACE_TOML, workspace_content)
    print("[INFO] Workspace frissítve a world-model crate-tel.")

# 3. World Model crate létrehozása
os.makedirs(os.path.join(WORLD_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(WORLD_DIR, "tests"), exist_ok=True)

write_file(os.path.join(WORLD_DIR, "Cargo.toml"), '''[package]
name = "athlesia-world-model"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-executor = { path = "../athlesia-executor" }
''')

write_file(os.path.join(WORLD_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Grid, Program};
use athlesia_executor::{run_program, Budget};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HypothesisStatus {
    Active,
    Confirmed,
    Falsified,
}

#[derive(Debug, Clone)]
pub struct TransitionHypothesis {
    pub id: u64,
    pub program: Program,
    pub evidence_for: u32,
    pub evidence_against: u32,
    pub status: HypothesisStatus,
}

#[derive(Debug, Clone)]
pub struct WorldModel {
    pub current_grid: Grid,
    pub hypotheses: Vec<TransitionHypothesis>,
    pub tick: u64,
}

impl WorldModel {
    pub fn new(initial_grid: Grid) -> Self {
        WorldModel {
            current_grid: initial_grid,
            hypotheses: Vec::new(),
            tick: 0,
        }
    }

    pub fn add_hypothesis(&mut self, program: Program) -> u64 {
        let id = self.hypotheses.len() as u64;
        self.hypotheses.push(TransitionHypothesis {
            id,
            program,
            evidence_for: 0,
            evidence_against: 0,
            status: HypothesisStatus::Active,
        });
        id
    }

    pub fn predict(&self, program: &Program, grid: &Grid) -> Option<Grid> {
        let mut budget = Budget { max_steps: 1000 };
        run_program(program, grid, &mut budget).ok()
    }

    pub fn update(&mut self, previous_grid: &Grid, observed_grid: &Grid) {
        for hyp in &mut self.hypotheses {
            if let Some(predicted) = self.predict(&hyp.program, previous_grid) {
                if predicted == *observed_grid {
                    hyp.evidence_for += 1;
                    if hyp.evidence_against == 0 && hyp.evidence_for >= 3 {
                        hyp.status = HypothesisStatus::Confirmed;
                    }
                } else {
                    hyp.evidence_against += 1;
                    hyp.status = HypothesisStatus::Falsified;
                }
            }
        }
        self.tick += 1;
    }
}
''')

# 4. World Model tesztek
write_file(os.path.join(WORLD_DIR, "tests", "world_model_test.rs"), r'''
use athlesia_world_model::{WorldModel, HypothesisStatus};
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn hypothesis_gets_evidence_for_correct_prediction() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    // Program: Translate(1,0) - jobbra tolás
    let program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    wm.add_hypothesis(program.clone());

    // Előző rács: [1,0,0,0,0] egy sorban
    let prev = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    // Megfigyelt rács: [0,1,0,0,0] - pontosan a Translate(1,0) eredménye
    let obs = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    wm.update(&prev, &obs);
    assert_eq!(wm.hypotheses[0].evidence_for, 1);
    assert_eq!(wm.hypotheses[0].evidence_against, 0);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Active); // még nem elég bizonyíték
}

#[test]
fn hypothesis_gets_evidence_against_wrong_prediction() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    let program: Program = vec![(PrimName::ReflectH, Params::None)];
    wm.add_hypothesis(program.clone());

    let prev = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    // Megfigyelt rács nem tükrözés, hanem ugyanaz (rossz predikció)
    let obs = prev.clone();

    wm.update(&prev, &obs);
    assert_eq!(wm.hypotheses[0].evidence_against, 1);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Falsified);
}

#[test]
fn multiple_updates_confirm_hypothesis() {
    let mut wm = WorldModel::new(build_grid([[0; 5]; 5]));

    let program: Program = vec![(PrimName::Recolor, Params::Recolor([1, 0, 3, 2]))];
    wm.add_hypothesis(program.clone());

    let prev = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let obs = build_grid([
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    // Három helyes predikcióval megerősítjük a hipotézist
    wm.update(&prev, &obs);
    wm.update(&prev, &obs);
    wm.update(&prev, &obs);
    assert_eq!(wm.hypotheses[0].evidence_for, 3);
    assert_eq!(wm.hypotheses[0].status, HypothesisStatus::Confirmed);
}
''')

print("[INFO] World Model crate létrehozva.")

# 5. Cargo test futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-world-model"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] World Model tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] World Model tesztek zöldek.")

# 6. Git takarítás: target mappa eltávolítása a verziókövetésből, .gitignore commitolása
try:
    subprocess.run(["git", "rm", "-r", "--cached", "athlesia/target"], check=False, capture_output=True)
    subprocess.run(["git", "add", "-A"], check=True)
    subprocess.run(["git", "commit", "-m", "Add world model module and ignore target dir"], check=True)
    subprocess.run(["git", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre (valószínűleg nincs git repó).")
