#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Planner lib.rs frissítése: plan_experiment használja a select_probe_action-t
p = pathlib.Path("crates/athlesia-planner/src/lib.rs")
s = p.read_text()

old_method = '''    pub fn plan_experiment(&self, candidate: &CandidateConcept) -> ExperimentPlan {
        ExperimentPlan {
            actions: Vec::new(),
            target_hypothesis: candidate.sketch.name.clone(),
            expected_observation: candidate.sketch.relation_pattern.clone(),
        }
    }
'''
new_method = '''    pub fn plan_experiment(&self, candidate: &CandidateConcept) -> ExperimentPlan {
        // A kísérleti akciót a jelenlegi heurisztika alapján választjuk.
        let probe_action = self.select_probe_action(candidate);
        ExperimentPlan {
            actions: vec![probe_action],
            target_hypothesis: candidate.sketch.name.clone(),
            expected_observation: candidate.sketch.relation_pattern.clone(),
        }
    }
'''

if old_method not in s:
    print("[ERROR] plan_experiment blokk nem található.")
    sys.exit(1)

s = s.replace(old_method, new_method)
p.write_text(s)
print("[1] Planner lib.rs frissítve: plan_experiment tartalmazza a kísérleti akciót.")

# 2. Teszt módosítása: a plan_experiment most már nem üres
p = pathlib.Path("crates/athlesia-planner/tests/experiment_plan_test.rs")
s = p.read_text()

old_assert = "    assert!(plan.actions.is_empty());"
new_assert = "    assert_eq!(plan.actions.len(), 1);\n    assert_eq!(plan.actions[0].prim, athlesia_types::PrimName::Translate);"

if old_assert not in s:
    print("[ERROR] A régi assert nem található.")
    sys.exit(1)

s = s.replace(old_assert, new_assert)
p.write_text(s)
print("[2] experiment_plan_test.rs frissítve: az akciósor nem üres.")

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
subprocess.run(["git", "commit", "-m", "Phase 13 microstep 13: plan_experiment now includes a selected probe action"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
