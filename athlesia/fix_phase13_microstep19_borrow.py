#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-abstraction/src/lib.rs")
s = p.read_text()

old = "            freq.into_iter()\n                .max_by_key(|(_, count)| *count)"
new = "            freq.iter()\n                .max_by_key(|(_, count)| *count)"

if old not in s:
    print("[ERROR] A mozgatásos iter sor nem található.")
    sys.exit(1)

s = s.replace(old, new)
p.write_text(s)
print("[1] freq.iter() használata a mozgatás elkerülésére.")

# Abstraction tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-abstraction"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Abstraction tesztek nem mentek át.")
    sys.exit(1)

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
subprocess.run(["git", "commit", "-m", "Fix borrow in abstraction: use iter() instead of into_iter() to avoid moving freq"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
