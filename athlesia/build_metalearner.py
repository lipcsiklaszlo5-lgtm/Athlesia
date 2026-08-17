#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
ML_DIR = os.path.join(PROJECT, "crates", "athlesia-metalearner")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-metalearner" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. MetaLearner crate létrehozása
os.makedirs(os.path.join(ML_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(ML_DIR, "tests"), exist_ok=True)

write_file(os.path.join(ML_DIR, "Cargo.toml"), '''[package]
name = "athlesia-metalearner"
version = "0.1.0"
edition = "2021"

[dependencies]
''')

write_file(os.path.join(ML_DIR, "src", "lib.rs"), r'''
use std::collections::HashMap;

/// Hipotézis-pontszámok.
#[derive(Debug, Clone, Copy, Default)]
pub struct HypothesisScore {
    pub successes: u32,
    pub failures: u32,
}

/// MetaLearner: hipotézisek siker/kudarc alapú rangsorolása.
/// Jelenleg Laplace-simított prioritást használ:
/// (successes + 1) / (successes + failures + 2)
#[derive(Debug, Clone, Default)]
pub struct MetaLearner {
    pub scores: HashMap<u64, HypothesisScore>,
}

impl MetaLearner {
    pub fn record_success(&mut self, hyp_id: u64) {
        let entry = self.scores.entry(hyp_id).or_default();
        entry.successes += 1;
    }

    pub fn record_failure(&mut self, hyp_id: u64) {
        let entry = self.scores.entry(hyp_id).or_default();
        entry.failures += 1;
    }

    /// Prioritás: magasabb = jobb.
    pub fn priority(&self, hyp_id: u64) -> f64 {
        match self.scores.get(&hyp_id) {
            Some(s) => (s.successes as f64 + 1.0) / (s.successes as f64 + s.failures as f64 + 2.0),
            None => 0.5, // ismeretlen hipotézis semleges
        }
    }

    /// Hipotézis-azonosítókat sorba rendez prioritás szerint csökkenő sorrendben.
    /// Azonos prioritás esetén az id szerint növekvő sorrend (determinizmus).
    pub fn rank(&self, ids: &[u64]) -> Vec<u64> {
        let mut sorted = ids.to_vec();
        sorted.sort_by(|a, b| {
            let pa = self.priority(*a);
            let pb = self.priority(*b);
            pb.partial_cmp(&pa).unwrap().then(a.cmp(b))
        });
        sorted
    }
}
''')

# 3. Tesztek
write_file(os.path.join(ML_DIR, "tests", "metalearner_test.rs"), r'''
use athlesia_metalearner::MetaLearner;

#[test]
fn initial_priority_is_neutral() {
    let ml = MetaLearner::default();
    assert_eq!(ml.priority(0), 0.5);
}

#[test]
fn success_increases_priority() {
    let mut ml = MetaLearner::default();
    ml.record_success(0);
    assert!(ml.priority(0) > 0.5);
}

#[test]
fn failure_decreases_priority() {
    let mut ml = MetaLearner::default();
    ml.record_failure(0);
    assert!(ml.priority(0) < 0.5);
}

#[test]
fn rank_orders_by_priority() {
    let mut ml = MetaLearner::default();
    ml.record_success(0);
    ml.record_success(0);
    ml.record_failure(1);
    ml.record_failure(1);
    // 0: (3)/(4) = 0.75, 1: (1)/(4) = 0.25
    let ranked = ml.rank(&[1, 0]);
    assert_eq!(ranked, vec![0, 1]);
}
''')

print("[INFO] MetaLearner crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-metalearner"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] MetaLearner tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] MetaLearner tesztek zöldek.")

# 5. Git commit és push (a felső könyvtárban próbáljuk, ha van .git)
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-metalearner module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
