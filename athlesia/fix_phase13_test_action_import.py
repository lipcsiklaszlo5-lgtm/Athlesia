#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-world-model/tests/knowledge_state_test.rs")
s = p.read_text()

old_use = """use athlesia_world_model::{
    WorldModel, KnowledgeState, Prediction, Observation, Action,
};
use athlesia_types::{Grid, PrimName, Params};
"""
new_use = """use athlesia_world_model::{WorldModel, KnowledgeState, Observation};
use athlesia_types::{Grid, PrimName, Params, Action};
"""

if old_use not in s:
    print("[ERROR] A régi import blokk nem található.")
    sys.exit(1)

s = s.replace(old_use, new_use)
p.write_text(s)
print("[1] Teszt import javítva: Action az athlesia_types-ből.")

# Teszt futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-world-model", "--test", "knowledge_state_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] WorldModel tesztek még mindig hibásak.")
    sys.exit(1)

# Teljes world-model teszt futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-world-model"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] WorldModel tesztek nem mentek át.")
    sys.exit(1)

print("\n[SUCCESS] WorldModel tesztek zöldek.")

subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Fix knowledge_state_test import Action from athlesia_types"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
