#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

p = pathlib.Path("crates/athlesia-kernel/src/cognitive.rs")
s = p.read_text()

old_line = '''        let predicted_search_cost = meta
            .estimated_cost(*features, 0)
            .unwrap_or_else(|| 100.0 * (1.0 - conf));'''
new_line = '''        let predicted_search_cost = meta
            .estimated_cost(*features, 0)
            .map(|c| c as f32)
            .unwrap_or_else(|| 100.0 * (1.0 - conf));'''

if old_line not in s:
    print("[ERROR] A predicted_search_cost blokk nem található.")
    sys.exit(1)

s = s.replace(old_line, new_line)
write_file(p, s)
print("[1] cognitive.rs típusjavítás: estimated_cost -> f32 cast.")

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
print("\n[SUCCESS] Phase 7 kernel tesztek zöldek.")

subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Fix Phase 7 type mismatch in predicted_search_cost"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
