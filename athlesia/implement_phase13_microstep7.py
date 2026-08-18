#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Planner Cargo.toml frissítése (ellenőrzött hozzáfűzés)
p = pathlib.Path("crates/athlesia-planner/Cargo.toml")
s = p.read_text()

if "athlesia-hypothesis" not in s:
    # Ha van [dependencies] szekció, oda fűzzük
    if "[dependencies]" in s:
        s = s.replace(
            "[dependencies]",
            "[dependencies]\nathlesia-hypothesis = { path = \"../athlesia-hypothesis\" }",
            1,
        )
    else:
        # ha nincs, hozzunk létre egyet
        s += "\n[dependencies]\nathlesia-hypothesis = { path = \"../athlesia-hypothesis\" }\n"
    p.write_text(s)
    print("[1] Planner Cargo.toml frissítve: athlesia-hypothesis függőség hozzáadva.")
else:
    print("[1] Planner Cargo.toml már tartalmazza a függőséget.")

# 2. Planner lib.rs bővítése
p = pathlib.Path("crates/athlesia-planner/src/lib.rs")
s = p.read_text()

# Import módosítása
old_import = "use athlesia_world_model::{WorldModel, Query};"
new_import = "use athlesia_world_model::{WorldModel, Query};\nuse athlesia_hypothesis::CandidateConcept;"
if old_import not in s:
    print("[ERROR] A Planner import blokk nem található.")
    sys.exit(1)
s = s.replace(old_import, new_import)

# ExperimentPlan struct és metódus hozzáadása
# Az ActionValue után, a Planner struct elé szúrjuk be az ExperimentPlan-ot
anchor = "/// A Manhattan Kernel tervezője.\n"
new_struct = '''/// Kísérleti terv a candidate concept aktív verifikálásához.
#[derive(Debug, Clone)]
pub struct ExperimentPlan {
    pub actions: Vec<athlesia_types::Action>,
    pub target_hypothesis: String,
    pub expected_observation: String,
}

/// A Manhattan Kernel tervezője.
'''
if anchor not in s:
    print("[ERROR] A Planner struct előtti komment nem található.")
    sys.exit(1)
s = s.replace(anchor, new_struct)

# plan_experiment metódus beszúrása a Planner impl blokk végére
method_anchor = '''    /// Kiválasztja a legjobb akciót a megadott súlyokkal.
    ///
    /// `value = α * info_gain + β * progress - γ * cost - δ * risk`
'''
new_method = '''    /// Kísérleti tervet készít egy candidate concept alapján.
    /// Jelenleg egyszerű placeholder: az akciósor üres, de a célhipotézis
    /// neve rögzítve van. A következő mikrolépésekben lesz diszkriminatív.
    pub fn plan_experiment(&self, candidate: &CandidateConcept) -> ExperimentPlan {
        ExperimentPlan {
            actions: Vec::new(),
            target_hypothesis: candidate.sketch.name.clone(),
            expected_observation: candidate.sketch.relation_pattern.clone(),
        }
    }

    /// Kiválasztja a legjobb akciót a megadott súlyokkal.
    ///
    /// `value = α * info_gain + β * progress - γ * cost - δ * risk`
'''
if method_anchor not in s:
    print("[ERROR] A select_action előtti komment nem található.")
    sys.exit(1)
s = s.replace(method_anchor, new_method)

p.write_text(s)
print("[2] Planner lib.rs bővítve: ExperimentPlan és plan_experiment.")

# 3. Új tesztfájl
test_code = r'''
use athlesia_planner::{Planner, PlannerMode, ExperimentPlan};
use athlesia_hypothesis::{CandidateConcept, ConceptSketch};

#[test]
fn plan_experiment_returns_plan_with_target_hypothesis() {
    let planner = Planner::new(PlannerMode::Exploration);
    let candidate = CandidateConcept {
        sketch: ConceptSketch {
            name: "RepeatedInteraction".to_string(),
            relation_pattern: "interaction(A,B)".to_string(),
            objects_involved: vec![1, 2],
        },
        evidence: vec!["residual".to_string()],
        confidence: 0.5,
    };

    let plan: ExperimentPlan = planner.plan_experiment(&candidate);
    assert_eq!(plan.target_hypothesis, "RepeatedInteraction");
    assert_eq!(plan.expected_observation, "interaction(A,B)");
    assert!(plan.actions.is_empty());
}
'''
write_file("crates/athlesia-planner/tests/experiment_plan_test.rs", test_code)
print("[3] experiment_plan_test.rs létrehozva.")

# 4. Planner tesztek futtatása
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
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 7: add ExperimentPlan and plan_experiment to Planner"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
