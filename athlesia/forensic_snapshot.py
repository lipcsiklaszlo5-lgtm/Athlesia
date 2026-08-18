#!/usr/bin/env python3
import subprocess, pathlib, os

def run(cmd, timeout=20):
    print("\n$ " + cmd)
    try:
        res = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout, check=False)
        print(res.stdout)
        if res.stderr:
            print("STDERR:", res.stderr)
    except subprocess.TimeoutExpired:
        print("TIMEOUT")
    except Exception as e:
        print("ERROR:", e)

print("========================================")
print("FORENSIC SNAPSHOT — READ ONLY")
print("========================================")

run("pwd")
run("git status --short")
run("git rev-parse HEAD")
run("git branch --show-current")

print("\n--- WORKSPACE Cargo.toml ---")
p = pathlib.Path("Cargo.toml")
if p.exists():
    print(p.read_text())
else:
    print("Nincs Cargo.toml")

print("\n--- CRATE LISTA ---")
run("find crates -maxdepth 2 -name Cargo.toml | sort")

print("\n--- PHASE 1-13 RELEVÁNS FÁJLOK (source/test) ---")
patterns = [
    "crates/athlesia-kernel/src/cognitive.rs",
    "crates/athlesia-kernel/src/lib.rs",
    "crates/athlesia-kernel/src/openworld.rs",
    "crates/athlesia-kernel/tests/prior_test.rs",
    "crates/athlesia-kernel/tests/cognitive_test.rs",
    "crates/athlesia-kernel/tests/structural_analysis_test.rs",
    "crates/athlesia-kernel/tests/structured_solve_test.rs",
    "crates/athlesia-kernel/tests/abstraction_learning_test.rs",
    "crates/athlesia-kernel/tests/phase9_external_benchmark.rs",
    "crates/athlesia-kernel/tests/phase13_scaffold.rs",
    "crates/athlesia-core/src/lib.rs",
    "crates/athlesia-core/tests/generalization_benchmark.rs",
    "crates/athlesia-core/tests/concept_transfer_test.rs",
    "crates/athlesia-core/tests/search_cost_learning_test.rs",
    "crates/athlesia-search/src/lib.rs",
    "crates/athlesia-search/tests/budget_abort_test.rs",
    "crates/athlesia-world-model/src/lib.rs",
    "crates/athlesia-world-model/tests/prediction_error_test.rs",
    "crates/athlesia-memory/src/lib.rs",
    "crates/athlesia-knowledge/src/lib.rs",
    "crates/athlesia-abstraction/src/lib.rs",
    "crates/athlesia-hypothesis/src/lib.rs",
    "crates/athlesia-planner/src/lib.rs",
    "crates/athlesia-planner/tests/action_value_test.rs",
    "crates/athlesia-interactive/src/lib.rs",
    "crates/athlesia-interactive/tests/info_gain_benchmark.rs",
    "crates/athlesia-openworld/src/lib.rs",
    "crates/athlesia-openworld/tests/open_world_discovery.rs",
]
existing = []
missing = []
for path in patterns:
    if pathlib.Path(path).exists():
        existing.append(path)
    else:
        missing.append(path)
print("LEtezo fajlok:")
for f in existing:
    print(" ", f)
if missing:
    print("Hianyzo fajlok (a repositoryban nem talalhatok):")
    for f in missing:
        print(" ", f)

print("\n--- PHASE 13 RELEVANS FAJL TARTALOM (ha letezik) ---")
for f in ["crates/athlesia-kernel/src/openworld.rs", "crates/athlesia-kernel/tests/phase13_scaffold.rs", "crates/athlesia-openworld/src/lib.rs", "crates/athlesia-openworld/tests/open_world_discovery.rs"]:
    p = pathlib.Path(f)
    if p.exists():
        print(f"\n--- {f} ---")
        print(p.read_text()[:5000])
    else:
        print(f"\n--- {f} --- NEM LETEZIK")

print("\n--- GIT DIFF STAT ---")
run("git diff --stat")

print("\n--- GIT DIFF NAME-ONLY ---")
run("git diff --name-only")

print("\n--- GIT STATUS BRANCH ---")
run("git status --short --branch")

print("\n--- FORENSIC SNAPSHOT KESZ ---")
