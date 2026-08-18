#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-knowledge/src/lib.rs")
s = p.read_text()

old = """/// Igazolt fogalom: olyan fogalom, amelyet kísérletekkel megerősítettünk.
#[derive(Debug, Clone)]
pub struct VerifiedConcept {
"""
new = """/// Igazolt fogalom: olyan fogalom, amelyet kísérletekkel megerősítettünk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedConcept {
"""

if old not in s:
    print("[ERROR] VerifiedConcept struct nem található a várt formában.")
    sys.exit(1)

s = s.replace(old, new)
p.write_text(s)
print("[1] VerifiedConcept PartialEq/Eq derive hozzáadva.")

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
subprocess.run(["git", "commit", "-m", "Add PartialEq and Eq to VerifiedConcept for OpenWorldOutcome"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
