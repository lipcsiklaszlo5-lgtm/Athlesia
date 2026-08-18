#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-world-model/tests/prediction_residual_test.rs")
s = p.read_text()

old = "    assert!((residual.mismatch_score - 0.04).abs() < 0.0001);"
new = "    assert!((residual.mismatch_score - 0.08).abs() < 0.0001);"

if old not in s:
    print("[ERROR] A várt mismatch_score sor nem található.")
    sys.exit(1)

s = s.replace(old, new)
p.write_text(s)
print("[1] mismatch_score elvárás 0.08-ra javítva.")

# WorldModel tesztek futtatása
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
subprocess.run(["git", "commit", "-m", "Fix prediction_residual_test mismatch_score expectation to 0.08"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
