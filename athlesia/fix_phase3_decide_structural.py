#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. cognitive.rs beolvasása és decide logika módosítása
p = pathlib.Path("crates/athlesia-kernel/src/cognitive.rs")
s = p.read_text()

old_block = '''        if estimate.hypothesis_confidence > 0.8 {
            CognitiveDecision::Solve
        } else if estimate.hypothesis_confidence > 0.5 {
            CognitiveDecision::Explore
        } else if predicted_search_cost < 50.0 {
            CognitiveDecision::Guess
        } else {
            CognitiveDecision::Abstain
        }'''

new_block = '''        if estimate.structural_match > 0.9 {
            CognitiveDecision::Solve
        } else if estimate.hypothesis_confidence > 0.8 {
            CognitiveDecision::Solve
        } else if estimate.hypothesis_confidence > 0.5 {
            CognitiveDecision::Explore
        } else if predicted_search_cost < 50.0 {
            CognitiveDecision::Guess
        } else {
            CognitiveDecision::Abstain
        }'''

if old_block not in s:
    print("[ERROR] A decide blokk nem található a várt formában.")
    sys.exit(1)

s = s.replace(old_block, new_block)
write_file(p, s)
print("[1] cognitive.rs frissítve: magas strukturális egyezés esetén Solve döntés.")

# 2. Tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-kernel"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] A kernel tesztek még mindig hibásak.")
    sys.exit(1)
print("\n[SUCCESS] Kernel tesztek zöldek.")

# 3. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Fix decide: high structural match triggers Solve"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
