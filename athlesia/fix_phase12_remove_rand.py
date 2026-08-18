#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-interactive/src/lib.rs")
s = p.read_text()

# Eltávolítjuk a rand import sort
lines = s.splitlines()
filtered = [line for line in lines if "use rand::Rng;" not in line]
new_s = "\n".join(filtered) + "\n"

p.write_text(new_s)
print("[1] rand import eltávolítva.")

# Teszt futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-interactive"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Phase 12 interaktív tesztek nem mentek át.")
    sys.exit(1)

print("\n[SUCCESS] Phase 12 interaktív tesztek zöldek.")

subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Fix Phase 12: remove unused rand import"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
