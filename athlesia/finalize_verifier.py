#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Verifier lib.rs teljes újraírása a dokumentum 5. fejezete szerint
write_file("crates/athlesia-verifier/src/lib.rs", r'''
use athlesia_types::{Grid, Program, Budget};
use athlesia_executor::run_program;

/// Elfogadási/elutasítási eredmény.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationResult {
    Accept,
    Reject,
    Inconclusive,
}

/// Bizonyíték: múltbeli megfigyelések (bemenet-kimenet párok).
pub type Evidence = Vec<(Grid, Grid)>;

/// A Manhattan Kernel Verifier modulja.
#[derive(Debug, Clone, Default)]
pub struct Verifier {
    // Ha később kell állapotot tartani, itt lehet.
}

impl Verifier {
    pub fn new() -> Self {
        Self::default()
    }

    /// Programhelyesség-ellenőrzés tanulópéldák ellen.
    pub fn verify(&self, program: &Program, examples: &Evidence) -> VerificationResult {
        if examples.is_empty() {
            return VerificationResult::Inconclusive;
        }

        for (input, expected) in examples {
            let mut budget = Budget { max_steps: 1000, max_depth: 100 };
            match run_program(program, input, &mut budget) {
                Ok(output) if output == *expected => continue,
                _ => return VerificationResult::Reject,
            }
        }

        VerificationResult::Accept
    }

    /// Hipotézis-ellenőrzés a teljes megfigyelt előzmény ellen.
    /// Ugyanaz, mint a `verify`, de kifejezettebb nevet adunk neki,
    /// hogy a hívó szándéka világos legyen.
    pub fn verify_hypothesis(&self, hypothesis: &Program, history: &Evidence) -> VerificationResult {
        self.verify(hypothesis, history)
    }

    /// Két program szemantikai ekvivalenciája a megadott példákon.
    /// Csak akkor `Accept`, ha minden példán ugyanazt a kimenetet adják.
    pub fn verify_equivalence(&self, program_a: &Program, program_b: &Program, examples: &Evidence) -> VerificationResult {
        if examples.is_empty() {
            return VerificationResult::Inconclusive;
        }

        for (input, _) in examples {
            let mut budget_a = Budget { max_steps: 1000, max_depth: 100 };
            let mut budget_b = Budget { max_steps: 1000, max_depth: 100 };

            let output_a = run_program(program_a, input, &mut budget_a);
            let output_b = run_program(program_b, input, &mut budget_b);

            match (output_a, output_b) {
                (Ok(a), Ok(b)) if a == b => continue,
                _ => return VerificationResult::Reject,
            }
        }

        VerificationResult::Accept
    }
}
''')
print("[1] Verifier lib.rs teljesen újraírva.")

# 2. Tesztek frissítése
write_file("crates/athlesia-verifier/tests/verifier_full_test.rs", r'''
use athlesia_verifier::{Verifier, VerificationResult};
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn accepts_correct_program() {
    let v = Verifier::new();
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
    let v = Verifier::new();
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
    let v = Verifier::new();
    let program: Program = vec![(PrimName::Recolor, Params::Recolor([
        Color(1), Color(0), Color(2), Color(3), Color(4), Color(5), Color(6), Color(7), Color(8), Color(9)
    ]))];
    let result = v.verify(&program, &[]);
    assert_eq!(result, VerificationResult::Inconclusive);
}

#[test]
fn verify_hypothesis_checks_full_history() {
    let v = Verifier::new();
    let program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    let history = vec![
        (
            build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
            build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
        ),
        (
            build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
            build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
        ),
    ];

    let result = v.verify_hypothesis(&program, &history);
    assert_eq!(result, VerificationResult::Accept);
}

#[test]
fn verify_equivalence_detects_same_program() {
    let v = Verifier::new();
    let p1: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let p2: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];

    let examples = vec![
        (build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
         build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]])),
    ];

    let result = v.verify_equivalence(&p1, &p2, &examples);
    assert_eq!(result, VerificationResult::Accept);
}

#[test]
fn verify_equivalence_rejects_different_programs() {
    let v = Verifier::new();
    let p1: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let p2: Program = vec![(PrimName::ReflectH, Params::None)];

    let examples = vec![
        (build_grid([[1,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]]),
         build_grid([[0,1,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0],[0,0,0,0,0]])),
    ];

    let result = v.verify_equivalence(&p1, &p2, &examples);
    assert_eq!(result, VerificationResult::Reject);
}
''')
print("[2] Verifier tesztek hozzáadva.")

# 3. Tesztek futtatása
result = subprocess.run(["cargo", "test", "-p", "athlesia-verifier", "--test", "verifier_full_test"], capture_output=True, text=True, check=False)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Verifier tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Verifier tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Finalize Verifier with hypothesis and equivalence checking"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
