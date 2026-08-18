#!/usr/bin/env python3
import os, subprocess, sys, pathlib

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Verifier lib.rs teljes újraírása strukturált riporttal
write_file("crates/athlesia-verifier/src/lib.rs", r'''
use athlesia_types::{Grid, Program, Budget};
use athlesia_executor::run_program;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationResult {
    Accept,
    Reject,
    Inconclusive,
}

pub type Evidence = Vec<(Grid, Grid)>;

/// Strukturált hiba-struktúra.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResidualStructure {
    None,
    Translation,
    Reflection,
    Rotation,
    Recolor,
    MissingObjects,
    ExtraObjects,
    WrongBlockTransform,
    WrongBlockPlacement,
    Unknown,
}

/// Kudarc-aláírás: a hibák számszerűsített jellege.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FailureSignature {
    pub pixel_mismatch: usize,
    pub dimension_mismatch: bool,
    pub summary: String,
}

/// Részletes verifikációs riport.
#[derive(Debug, Clone, PartialEq)]
pub struct VerificationReport {
    pub exact: bool,
    pub pixel_accuracy: f32,
    pub shape_score: f32,
    pub object_score: f32,
    pub spatial_score: f32,
    pub color_score: f32,
    pub symmetry_score: f32,
    pub matched_objects: usize,
    pub matched_blocks: usize,
    pub residual: ResidualStructure,
    pub failure_signature: FailureSignature,
}

#[derive(Debug, Clone, Default)]
pub struct Verifier;

impl Verifier {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn verify_hypothesis(&self, hypothesis: &Program, history: &Evidence) -> VerificationResult {
        self.verify(hypothesis, history)
    }

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

    /// Strukturált riport készítése egyetlen bemenet-cél párra.
    pub fn report(&self, program: &Program, input: &Grid, target: &Grid) -> VerificationReport {
        let mut budget = Budget { max_steps: 1000, max_depth: 100 };
        let output = run_program(program, input, &mut budget).unwrap_or_else(|_| input.clone());

        let exact = output == *target;

        // Pixel pontosság
        let pixel_accuracy = if exact {
            1.0
        } else if output.width == target.width && output.height == target.height {
            let total = (output.width as usize) * (output.height as usize);
            let matching = output
                .cells
                .iter()
                .zip(target.cells.iter())
                .filter(|(a, b)| a == b)
                .count();
            matching as f32 / total as f32
        } else {
            0.0
        };

        // Alak- és színpontszám
        let shape_score = if output.width == target.width && output.height == target.height { 1.0 } else { 0.0 };
        let mut color_mismatch = 0;
        if output.width == target.width && output.height == target.height {
            for (a, b) in output.cells.iter().zip(target.cells.iter()) {
                if a != b {
                    color_mismatch += 1;
                }
            }
        }
        let color_score = if exact { 1.0 } else if output.width == target.width && output.height == target.height {
            1.0 - (color_mismatch as f32 / (output.width as usize * output.height as usize) as f32)
        } else { 0.0 };

        // Objektum- és térbeli egyezés most egyszerű közelítés
        let object_score = if exact { 1.0 } else { 0.0 };
        let spatial_score = shape_score;
        let symmetry_score = 0.0;

        let failure_signature = FailureSignature {
            pixel_mismatch: if output.width == target.width && output.height == target.height {
                output
                    .cells
                    .iter()
                    .zip(target.cells.iter())
                    .filter(|(a, b)| a != b)
                    .count()
            } else {
                usize::MAX
            },
            dimension_mismatch: output.width != target.width || output.height != target.height,
            summary: if exact { "exact".to_string() } else { "mismatch".to_string() },
        };

        let residual = if exact {
            ResidualStructure::None
        } else if failure_signature.dimension_mismatch {
            ResidualStructure::Unknown
        } else {
            ResidualStructure::Unknown
        };

        VerificationReport {
            exact,
            pixel_accuracy,
            shape_score,
            object_score,
            spatial_score,
            color_score,
            symmetry_score,
            matched_objects: 0,
            matched_blocks: 0,
            residual,
            failure_signature,
        }
    }
}
''')

print("[1] Verifier lib.rs strukturált riporttal frissítve.")

# 2. Teszt hozzáadása
write_file("crates/athlesia-verifier/tests/structural_report_test.rs", r'''
use athlesia_verifier::{Verifier, ResidualStructure};
use athlesia_types::{Grid, PrimName, Params, Program};

fn build_grid(rows: [[u8; 5]; 5]) -> Grid {
    Grid::from_5x5(rows)
}

#[test]
fn report_exact_match() {
    let v = Verifier::new();
    let program: Program = vec![(PrimName::Translate, Params::Translate(1, 0))];
    let input = build_grid([
        [1,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ]);
    let target = build_grid([
        [0,1,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ]);

    let report = v.report(&program, &input, &target);
    assert!(report.exact);
    assert_eq!(report.residual, ResidualStructure::None);
    assert_eq!(report.pixel_accuracy, 1.0);
}

#[test]
fn report_rejects_mismatch() {
    let v = Verifier::new();
    let program: Program = vec![(PrimName::ReflectH, Params::None)];
    let input = build_grid([
        [1,2,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ]);
    let target = build_grid([
        [2,1,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
        [0,0,0,0,0],
    ]);

    let report = v.report(&program, &input, &target);
    assert!(!report.exact);
    assert!(report.pixel_accuracy < 1.0);
    assert!(report.pixel_accuracy > 0.0);
    assert_eq!(report.residual, ResidualStructure::Unknown);
}
''')

print("[2] Strukturált verifier teszt hozzáadva.")

# 3. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-verifier"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Structural verifier tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Structural verifier tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Add structural verifier with rich report"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
