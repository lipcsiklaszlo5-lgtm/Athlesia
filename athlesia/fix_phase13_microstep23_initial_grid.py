#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-core/tests/openworld_interactive_transfer_test.rs")
s = p.read_text()

old_fn = '''fn create_world_model_with_reflect_only() -> WorldModel {
    let initial = Grid::new(5, 5);
    let mut wm = WorldModel::new(initial);
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);
    wm
}
'''
new_fn = '''fn create_world_model_with_reflect_only(initial: Grid) -> WorldModel {
    let mut wm = WorldModel::new(initial);
    wm.add_hypothesis(vec![(PrimName::ReflectH, Params::None)]);
    wm
}
'''

if old_fn not in s:
    print("[ERROR] A create_world_model függvény nem található.")
    sys.exit(1)
s = s.replace(old_fn, new_fn)

# Hívások módosítása: adjuk át a környezet kezdeti rácsát.
s = s.replace(
    "    let wm1 = create_world_model_with_reflect_only();",
    "    let wm1 = create_world_model_with_reflect_only(env1.grid.clone());",
)
s = s.replace(
    "    let wm2 = create_world_model_with_reflect_only();",
    "    let wm2 = create_world_model_with_reflect_only(env2.grid.clone());",
)

p.write_text(s)
print("[1] A teszt initial gridje mostantól a környezet kezdeti rácsát használja.")

# Core tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core", "--test", "openworld_interactive_transfer_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] A transzfer teszt továbbra is hibás.")
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
subprocess.run(["git", "commit", "-m", "Fix openworld_interactive_transfer_test: use environment initial grid in WorldModel"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
