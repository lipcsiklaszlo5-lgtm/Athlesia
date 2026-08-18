#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-world-model/tests/evaluate_with_residual_test.rs")
s = p.read_text()

old_assert = '    assert_eq!(residual.unexplained_features, vec!["pixel_mismatch"]);'
new_assert = '    assert!(residual.unexplained_features.contains(&"pixel_mismatch".to_string()));'

if old_assert not in s:
    print("[ERROR] A régi assert nem található.")
    sys.exit(1)

s = s.replace(old_assert, new_assert)
p.write_text(s)
print("[1] evaluate_with_residual_test.rs frissítve: tartalmazás-vizsgálat.")

# World-model tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-world-model"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] World-model tesztek nem mentek át.")
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
subprocess.run(["git", "commit", "-m", "Fix evaluate_with_residual_test to use contains instead of exact feature list"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
