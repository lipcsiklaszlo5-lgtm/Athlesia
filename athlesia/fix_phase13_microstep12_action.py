#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-core/tests/openworld_interactive_integration.rs")
s = p.read_text()

old_action = "    let action = Action { prim: PrimName::Translate, params: Params::Translate(1, 0) };"
new_action = "    let action = Action { prim: PrimName::Translate, params: Params::Translate(0, 1) };"

if old_action not in s:
    print("[ERROR] A régi akció sor nem található.")
    sys.exit(1)

s = s.replace(old_action, new_action)
p.write_text(s)
print("[1] A teszt akciója Translate(0,1)-re módosítva.")

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
subprocess.run(["git", "commit", "-m", "Fix openworld_interactive_integration: use Translate(0,1) to produce genuine OutOfModel"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
