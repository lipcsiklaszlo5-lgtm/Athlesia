#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. AbstractionEngine módosítása: object_count_changed prioritás
p = pathlib.Path("crates/athlesia-abstraction/src/lib.rs")
s = p.read_text()

old_block = '''        // A leggyakoribb megmagyarázatlan jellemzőt emeljük ki.
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
'''

new_block = '''        // A leggyakoribb megmagyarázatlan jellemzőt emeljük ki.
        let mut freq: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for r in &positive {
            for feat in &r.unexplained_features {
                *freq.entry(feat.as_str()).or_insert(0) += 1;
            }
        }

        // Ha van objektum-szintű változás, azt részesítsük előnyben a nyers
        // pixel_mismatch-sel szemben.
        let relation_pattern = if freq.contains_key("object_count_changed") {
            "object_count_change(A,B)".to_string()
        } else {
            freq.into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(feature, _)| feature.to_string())
                .unwrap_or_else(|| "pixel_mismatch".to_string())
        };

        let count = freq
            .get(relation_pattern.as_str())
            .copied()
            .unwrap_or(positive.len());
'''

if old_block not in s:
    print("[ERROR] A discover_candidate_concept blokk nem található.")
    sys.exit(1)

s = s.replace(old_block, new_block)
p.write_text(s)
print("[1] AbstractionEngine frissítve: object_count_changed prioritás.")

# 2. Új teszt az object_count_changed fogalomgeneráláshoz
test_code = r'''
use athlesia_abstraction::AbstractionEngine;
use athlesia_world_model::{Observation, PredictionResidual};
use athlesia_types::{Grid, Color};

fn grid_3x3_with_objects(count: usize) -> Grid {
    let mut g = Grid::new(3, 3);
    for i in 0..count {
        g.set(i as i8, 0, Color(1));
    }
    g
}

#[test]
fn discover_candidate_concept_prefers_object_count_changed() {
    let residual = PredictionResidual {
        expected_observation: Observation { state: grid_3x3_with_objects(1) },
        observed_observation: Observation { state: grid_3x3_with_objects(2) },
        mismatch_score: 0.5,
        unexplained_features: vec!["pixel_mismatch".to_string(), "object_count_changed".to_string()],
    };

    let candidate = AbstractionEngine::discover_candidate_concept(&[residual])
        .expect("Candidate fogalom kell");
    assert_eq!(candidate.sketch.relation_pattern, "object_count_change(A,B)");
    assert!(candidate.sketch.name.contains("object_count_change"));
}
'''
write_file("crates/athlesia-abstraction/tests/object_count_concept_test.rs", test_code)
print("[2] object_count_concept_test.rs létrehozva.")

# 3. Abstraction tesztek futtatása
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

# 4. Core tesztek futtatása (ellenőrzés, hogy a fogalomgenerálás nem tört meg)
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Core tesztek zöldek.")

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
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 19: prefer object-level features in candidate concept generation"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
