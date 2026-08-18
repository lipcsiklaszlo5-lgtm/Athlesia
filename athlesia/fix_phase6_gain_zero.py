#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

p = pathlib.Path("crates/athlesia-abstraction/src/lib.rs")
s = p.read_text()

old_line = "            let gain = (pattern.len() as i64) * (count as i64 - 1) - 1;\n            if gain <= 0 {\n                continue;\n            }"
new_line = "            let gain = (pattern.len() as i64) * (count as i64 - 1) - 1;\n            if gain < 0 {\n                continue;\n            }"

if old_line not in s:
    print("[ERROR] A gain számítás nem található a várt formában.")
    sys.exit(1)

s = s.replace(old_line, new_line)
write_file(p, s)
print("[1] AbstractionEngine gain feltétel módosítva: gain >= 0 elfogadva.")

# Teszt futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel", "--test", "abstraction_learning_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Abstraction teszt nem ment át.")
    sys.exit(1)

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

print("\n[SUCCESS] Phase 6 tesztek zöldek.")

# Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Allow zero-gain macros in abstraction to enable simple patterns"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
