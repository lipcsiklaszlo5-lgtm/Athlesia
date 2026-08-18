#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-planner/src/lib.rs")
s = p.read_text()

old = "\nuse athlesia_types::{PrimName, Params, Action};"
new = ""
if old not in s:
    print("[ERROR] A duplikált import sor nem található.")
    sys.exit(1)

s = s.replace(old, new)
p.write_text(s)
print("[1] Duplikált import eltávolítva.")

# Planner tesztek futtatása
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
subprocess.run(["git", "commit", "-m", "Fix duplicate athlesia_types import in Planner"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
