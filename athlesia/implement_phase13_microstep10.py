#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Planner lib.rs bővítése select_probe_action metódussal
p = pathlib.Path("crates/athlesia-planner/src/lib.rs")
s = p.read_text()

# Importok bővítése
old_import = "use athlesia_hypothesis::CandidateConcept;"
new_import = "use athlesia_hypothesis::CandidateConcept;\nuse athlesia_types::{PrimName, Params, Action};"
if old_import not in s:
    print("[ERROR] CandidateConcept import nem található.")
    sys.exit(1)
s = s.replace(old_import, new_import)

# Új metódus beszúrása a plan_experiment után
anchor = '''    /// Kísérleti tervet készít egy candidate concept alapján.
    /// Jelenleg egyszerű placeholder: az akciósor üres, de a célhipotézis
    /// neve rögzítve van. A következő mikrolépésekben lesz diszkriminatív.
    pub fn plan_experiment(&self, candidate: &CandidateConcept) -> ExperimentPlan {
        ExperimentPlan {
            actions: Vec::new(),
            target_hypothesis: candidate.sketch.name.clone(),
            expected_observation: candidate.sketch.relation_pattern.clone(),
        }
    }
'''

new_method = anchor + '''

    /// Egyetlen egyszerű kísérleti akciót választ a candidate concept alapján.
    ///
    /// Jelenleg csak a relation_pattern stringjére hagyatkozik:
    /// - ha tartalmazza az "interaction" szót, Translate(1,0)
    /// - ha tartalmazza a "symmetry" szót, ReflectH
    /// - különben Translate(0,1)
    ///
    /// Ez egy placeholder heurisztika, amit később információnyerés-alapú
    /// diszkriminatív akcióválasztással váltunk ki.
    pub fn select_probe_action(&self, candidate: &CandidateConcept) -> Action {
        let pattern = candidate.sketch.relation_pattern.to_lowercase();
        if pattern.contains("interaction") {
            Action { prim: PrimName::Translate, params: Params::Translate(1, 0) }
        } else if pattern.contains("symmetry") {
            Action { prim: PrimName::ReflectH, params: Params::None }
        } else {
            Action { prim: PrimName::Translate, params: Params::Translate(0, 1) }
        }
    }
'''

if anchor not in s:
    print("[ERROR] plan_experiment blokk nem található.")
    sys.exit(1)
s = s.replace(anchor, new_method)

p.write_text(s)
print("[1] Planner lib.rs frissítve: select_probe_action metódus hozzáadva.")

# 2. Új tesztfájl
test_code = r'''
use athlesia_planner::{Planner, PlannerMode};
use athlesia_hypothesis::{CandidateConcept, ConceptSketch};
use athlesia_types::{PrimName, Params};

#[test]
fn select_probe_action_returns_translate_for_interaction() {
    let planner = Planner::new(PlannerMode::Exploration);
    let candidate = CandidateConcept {
        sketch: ConceptSketch {
            name: "candidate".to_string(),
            relation_pattern: "interaction(A,B)".to_string(),
            objects_involved: vec![1, 2],
        },
        evidence: vec!["residual".to_string()],
        confidence: 0.5,
    };
    let action = planner.select_probe_action(&candidate);
    assert_eq!(action.prim, PrimName::Translate);
    assert_eq!(action.params, Params::Translate(1, 0));
}

#[test]
fn select_probe_action_returns_reflect_for_symmetry() {
    let planner = Planner::new(PlannerMode::Exploration);
    let candidate = CandidateConcept {
        sketch: ConceptSketch {
            name: "candidate".to_string(),
            relation_pattern: "symmetry(A,B)".to_string(),
            objects_involved: vec![1, 2],
        },
        evidence: vec!["residual".to_string()],
        confidence: 0.5,
    };
    let action = planner.select_probe_action(&candidate);
    assert_eq!(action.prim, PrimName::ReflectH);
    assert_eq!(action.params, Params::None);
}
'''

write_file("crates/athlesia-planner/tests/select_probe_action_test.rs", test_code)
print("[2] select_probe_action_test.rs létrehozva.")

# 3. Planner tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-planner"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Planner tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Planner tesztek zöldek.")

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
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 10: add select_probe_action heuristic to Planner"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
