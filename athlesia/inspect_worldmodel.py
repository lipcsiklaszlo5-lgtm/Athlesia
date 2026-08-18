#!/usr/bin/env python3
import pathlib
import subprocess

def run(cmd):
    print("\n$ " + cmd)
    res = subprocess.run(cmd, shell=True, capture_output=True, text=True, check=False)
    print(res.stdout)
    if res.stderr:
        print("STDERR:", res.stderr)

# Alaphelyzet
run("pwd")

# WorldModel crate fájljainak listája
print("\n--- WORLD MODEL CRATE FILES ---")
run("find crates/athlesia-world-model -type f -name '*.rs' | sort")

# Releváns minták keresése a teljes crates/ alatt
print("\n--- RELEVÁNS STRUKTÚRÁK KERESÉSE ---")
patterns = [
    "struct WorldModel",
    "struct Prediction",
    "struct Observation",
    "struct PredictionError",
    "PredictionResidual",
    "KnowledgeState",
    "enum UpdateResult",
    "struct Query",
    "struct TransitionHypothesis",
]
for pat in patterns:
    run(f"grep -R --include=*.rs \"{pat}\" crates/athlesia-world-model crates/athlesia-kernel/src crates/athlesia-kernel/tests 2>/dev/null | head -40")

# WorldModel src/lib.rs teljes tartalma
print("\n--- WORLD MODEL src/lib.rs ---")
p = pathlib.Path("crates/athlesia-world-model/src/lib.rs")
if p.exists():
    print(p.read_text())
else:
    print("[NEM LÉTEZIK]")

# WorldModel tesztek listája és tartalma
print("\n--- WORLD MODEL TESTS ---")
test_dir = pathlib.Path("crates/athlesia-world-model/tests")
if test_dir.exists():
    for f in sorted(test_dir.glob("*.rs")):
        print(f"\n===== {f} =====")
        print(f.read_text())
else:
    print("[NINCS TESZT KÖNYVTÁR]")
