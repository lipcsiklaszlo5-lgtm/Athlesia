#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-core/src/openworld.rs")
s = p.read_text()

old_imports = """use athlesia_world_model::{WorldModel, KnowledgeState, Prediction, Observation, Action, PredictionResidual};
use athlesia_abstraction::AbstractionEngine;
use athlesia_hypothesis::{CandidateConcept, ConceptSketch};
use athlesia_knowledge::KnowledgeBase;
"""
new_imports = """use athlesia_world_model::{WorldModel, KnowledgeState, Prediction, Observation};
use athlesia_abstraction::AbstractionEngine;
use athlesia_knowledge::KnowledgeBase;
use athlesia_types::Action;
"""

if old_imports not in s:
    print("[ERROR] A régi import blokk nem található.")
    sys.exit(1)

s = s.replace(old_imports, new_imports)
p.write_text(s)
print("[1] openworld.rs importok javítva: Action az athlesia_types-ből.")

# Core tesztek futtatása
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

# Teljes workspace teszt
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

print("\n[SUCCESS] Minden teszt zöld.")

subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Fix openworld.rs imports: Action from athlesia_types"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
