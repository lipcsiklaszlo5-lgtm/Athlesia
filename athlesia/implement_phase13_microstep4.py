#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. hypothesis lib.rs bővítése
p = pathlib.Path("crates/athlesia-hypothesis/src/lib.rs")
s = p.read_text()

# A CandidateHypothesis struct után szúrjuk be az új típusokat.
anchor = '''/// Jelölt hipotézis, amelynek a forrása és a programja is megvan.
#[derive(Debug, Clone)]
pub struct CandidateHypothesis {
    pub source: String,
    pub program: Program,
}
'''

new_types = anchor + '''

/// Absztrakt fogalomvázlat: relációs mintát ír le anélkül,
/// hogy konkrét primitívre vagy programra hivatkozna.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptSketch {
    pub name: String,
    pub relation_pattern: String,
    pub objects_involved: Vec<u64>,
}

/// Jelölt fogalom, amelyet még nem igazoltak.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateConcept {
    pub sketch: ConceptSketch,
    pub evidence: Vec<String>,
    pub confidence: f64,
}
'''

if anchor not in s:
    print("[ERROR] CandidateHypothesis blokk nem található.")
    sys.exit(1)

s = s.replace(anchor, new_types)
p.write_text(s)
print("[1] hypothesis lib.rs frissítve: ConceptSketch és CandidateConcept hozzáadva.")

# 2. Új tesztfájl
test_code = r'''
use athlesia_hypothesis::{ConceptSketch, CandidateConcept};

#[test]
fn candidate_concept_is_created_without_knowledge_base() {
    let sketch = ConceptSketch {
        name: "RepeatedInteraction".to_string(),
        relation_pattern: "interaction(A,B)".to_string(),
        objects_involved: vec![1, 2],
    };

    let candidate = CandidateConcept {
        sketch,
        evidence: vec!["residual: unexpected motion".to_string()],
        confidence: 0.3,
    };

    assert_eq!(candidate.sketch.name, "RepeatedInteraction");
    assert_eq!(candidate.sketch.relation_pattern, "interaction(A,B)");
    assert_eq!(candidate.sketch.objects_involved, vec![1, 2]);
    assert_eq!(candidate.evidence.len(), 1);
    assert!((candidate.confidence - 0.3).abs() < 1e-9);
}
'''
write_file("crates/athlesia-hypothesis/tests/candidate_concept_test.rs", test_code)
print("[2] candidate_concept_test.rs létrehozva.")

# 3. Hypothesis tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-hypothesis"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Hypothesis tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Hypothesis tesztek zöldek.")

# 4. Teljes workspace teszt
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

# 5. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 4: add ConceptSketch and CandidateConcept types"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
