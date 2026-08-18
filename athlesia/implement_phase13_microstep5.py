#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. athlesia-abstraction Cargo.toml frissítése függőségekkel
p = pathlib.Path("crates/athlesia-abstraction/Cargo.toml")
s = p.read_text()

if "athlesia-world-model" not in s:
    s += """
[dependencies]
athlesia-types = { path = "../athlesia-types" }
athlesia-knowledge = { path = "../athlesia-knowledge" }
athlesia-world-model = { path = "../athlesia-world-model" }
athlesia-hypothesis = { path = "../athlesia-hypothesis" }
"""
p.write_text(s)
print("[1] abstraction Cargo.toml frissítve.")

# 2. AbstractionEngine bővítése a discover_candidate_concept metódussal
p = pathlib.Path("crates/athlesia-abstraction/src/lib.rs")
s = p.read_text()

# A fájl végére szúrjuk be az új impl blokkot
new_impl = r'''

impl AbstractionEngine {
    /// CandidateConcept generálása predikciós reziduálisokból.
    ///
    /// Ez a Phase 13 minimális mechanizmusa: még nem valódi relációindukció,
    /// de a reziduálisok alapján általános fogalomjelöltet hoz létre.
    /// A későbbi mikrolépések ezt fogják finomítani.
    pub fn discover_candidate_concept(
        residuals: &[athlesia_world_model::PredictionResidual],
    ) -> Option<athlesia_hypothesis::CandidateConcept> {
        // Csak azokat vesszük figyelembe, ahol van eltérés.
        let positive: Vec<_> = residuals
            .iter()
            .filter(|r| r.mismatch_score > 0.0)
            .collect();

        if positive.is_empty() {
            return None;
        }

        // A leggyakoribb megmagyarázatlan jellemzőt emeljük ki.
        let mut freq: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for r in &positive {
            for feat in &r.unexplained_features {
                *freq.entry(feat.as_str()).or_insert(0) += 1;
            }
        }

        let (relation_pattern, count) = freq
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .unwrap_or(("pixel_mismatch", positive.len()));

        // Átlagos mismatch_score mint kezdeti confidence.
        let avg_mismatch = positive
            .iter()
            .map(|r| r.mismatch_score)
            .sum::<f64>()
            / positive.len() as f64;

        let sketch = athlesia_hypothesis::ConceptSketch {
            name: format!("candidate_{}", relation_pattern),
            relation_pattern: relation_pattern.to_string(),
            objects_involved: Vec::new(),
        };

        let evidence = positive
            .iter()
            .map(|r| format!("residual: {} features, mismatch={:.2}", r.unexplained_features.join(","), r.mismatch_score))
            .collect();

        Some(athlesia_hypothesis::CandidateConcept {
            sketch,
            evidence,
            confidence: avg_mismatch.min(1.0),
        })
    }
}
'''

s = s.rstrip() + "\n" + new_impl
p.write_text(s)
print("[2] abstraction lib.rs bővítve: discover_candidate_concept metódus.")

# 3. Új tesztfájl
test_code = r'''
use athlesia_abstraction::AbstractionEngine;
use athlesia_world_model::{Observation, PredictionResidual};
use athlesia_types::{Grid, Color};

fn grid_3x3_with_pixel(x: usize, y: usize, val: u8) -> Grid {
    let mut cells = vec![Color(0); 9];
    cells[y * 3 + x] = Color(val);
    Grid { width: 3, height: 3, cells }
}

fn residual_with_mismatch() -> PredictionResidual {
    PredictionResidual {
        expected_observation: Observation { state: grid_3x3_with_pixel(0, 0, 1) },
        observed_observation: Observation { state: grid_3x3_with_pixel(1, 0, 2) },
        mismatch_score: 0.5,
        unexplained_features: vec!["pixel_mismatch".to_string()],
    }
}

#[test]
fn discover_candidate_concept_returns_none_for_empty() {
    let result = AbstractionEngine::discover_candidate_concept(&[]);
    assert!(result.is_none());
}

#[test]
fn discover_candidate_concept_returns_candidate_for_mismatch() {
    let residuals = vec![residual_with_mismatch()];
    let candidate = AbstractionEngine::discover_candidate_concept(&residuals)
        .expect("Candidate conceptet kell generálni");
    assert!(!candidate.sketch.name.is_empty());
    assert!(!candidate.sketch.relation_pattern.is_empty());
    assert!(candidate.confidence > 0.0);
    assert!(!candidate.evidence.is_empty());
}
'''

write_file("crates/athlesia-abstraction/tests/candidate_concept_discovery_test.rs", test_code)
print("[3] candidate_concept_discovery_test.rs létrehozva.")

# 4. Abstraction tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-abstraction"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Abstraction tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Abstraction tesztek zöldek.")

# 5. Teljes workspace teszt
result = subprocess.run(
    ["cargo", "test", "--workspace", "--no-fail-fast"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Teljes workspace tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Teljes workspace tesztek zöldek.")

# 6. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 5: discover_candidate_concept from prediction residuals"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
