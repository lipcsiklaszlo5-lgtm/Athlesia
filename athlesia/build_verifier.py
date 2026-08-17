#!/usr/bin/env python3
import os, subprocess, sys, pathlib

PROJECT = "."
VERIFIER_DIR = os.path.join(PROJECT, "crates", "athlesia-verifier")
WORKSPACE_TOML = os.path.join(PROJECT, "Cargo.toml")

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Workspace frissítése
ws = pathlib.Path(WORKSPACE_TOML).read_text()
if "athlesia-verifier" not in ws:
    ws = ws.replace(
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner"]',
        'members = ["crates/athlesia-types", "crates/athlesia-executor", "crates/athlesia-perception", "crates/athlesia-world-model", "crates/athlesia-features", "crates/athlesia-metalearner", "crates/athlesia-verifier"]'
    )
    write_file(WORKSPACE_TOML, ws)
    print("[INFO] Workspace frissítve.")

# 2. Verifier crate létrehozása
os.makedirs(os.path.join(VERIFIER_DIR, "src"), exist_ok=True)
os.makedirs(os.path.join(VERIFIER_DIR, "tests"), exist_ok=True)

write_file(os.path.join(VERIFIER_DIR, "Cargo.toml"), '''[package]
name = "athlesia-verifier"
version = "0.1.0"
edition = "2021"

[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-executor = { path = "../athlesia-executor" }
''')

write_file(os.path.join(VERIFIER_DIR, "src", "lib.rs"), r'''
use athlesia_types::{Grid, Program, Budget};
use athlesia_executor::run_program;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationResult {
    Accept,
    Reject,
    Inconclusive,
}

#[derive(Debug, Clone, Default)]
pub struct Verifier;

impl Verifier {
    pub fn verify(&self, program: &Program, examples: &[(Grid, Grid)]) -> VerificationResult {
        if examples.is_empty() {
            return VerificationResult::Inconclusive;
        }

        for (input, expected) in examples {
            let mut budget = Budget { max_steps: 1000 };
            match run_program(program, input, &mut budget) {
                Ok(output) if output == *expected => continue,
                _ => return VerificationResult::Reject,
            }
        }

        VerificationResult::Accept
    }
}
''')

# 3. Tesztek
write_file(os.path.join(VERIFIER_DIR, "tests", "verifier_test.rs"), r'''
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid { cells: rows }
}

#[test]
fn accepts_correct_program() {
    let v = Verifier;
    let program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    let input = build_grid([
        [1, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let expected = build_grid([
        [0, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let result = v.verify(&program, &[(input, expected)]);
    assert_eq!(result, VerificationResult::Accept);
}

#[test]
fn rejects_wrong_program() {
    let v = Verifier;
    let program: Program = vec![(PrimName::ReflectH, Params::None)];

    let input = build_grid([
        [1, 2, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);
    let expected = build_grid([
        [2, 1, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ]);

    let result = v.verify(&program, &[(input, expected)]);
    assert_eq!(result, VerificationResult::Reject);
}

#[test]
fn returns_inconclusive_for_empty_examples() {
    let v = Verifier;
    let program: Program = vec![(PrimName::Recolor, Params::Recolor([1, 0, 2, 3]))];
    let result = v.verify(&program, &[]);
    assert_eq!(result, VerificationResult::Inconclusive);
}
''')

print("[INFO] Verifier crate létrehozva.")

# 4. Teszt futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-verifier"], capture_output=True, text=True, check=False)
sys.stdout.write(result.stdout)
sys.stderr.write(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Verifier tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Verifier tesztek zöldek.")

# 5. Git commit és push
try:
    subprocess.run(["git", "-C", "..", "add", "-A"], check=True)
    subprocess.run(["git", "-C", "..", "commit", "-m", "Add athlesia-verifier module"], check=True)
    subprocess.run(["git", "-C", "..", "push"], check=True)
    print("[INFO] Git commit és push sikeres.")
except subprocess.CalledProcessError:
    print("[WARN] Git művelet nem hajtható végre.")
