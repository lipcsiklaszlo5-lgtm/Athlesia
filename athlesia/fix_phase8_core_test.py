#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

p = pathlib.Path("crates/athlesia-core/tests/search_cost_learning_test.rs")
s = p.read_text()

# A FeatureVector::default() helyett extract_features(&input)-ot használunk
old_line = "use athlesia_features::FeatureVector;"
if old_line in s:
    s = s.replace(old_line, "use athlesia_features::extract_features;")

old_use = "let fv = FeatureVector::default();"
new_use = "let fv = extract_features(&input);"
if old_use not in s:
    print("[ERROR] A FeatureVector::default() sor nem található.")
    sys.exit(1)
s = s.replace(old_use, new_use)

write_file(p, s)
print("[1] search_cost_learning_test.rs frissítve: extract_features(&input) használata.")

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
    print("\n[FAILURE] Core tesztek még mindig hibásak.")
    sys.exit(1)

print("\n[SUCCESS] Core tesztek zöldek.")

# Kernel tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Kernel tesztek nem mentek át.")
    sys.exit(1)

print("\n[SUCCESS] Kernel tesztek zöldek.")

# Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Fix Phase 8 core test to use extract_features for context"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
