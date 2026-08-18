#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. cognitive_test.rs beolvasása
p = pathlib.Path("crates/athlesia-kernel/tests/cognitive_test.rs")
s = p.read_text()

# 2. A régi 3 argumentumos hívás cseréje 5 argumentumosra
old_call = "CognitiveController::decide(&fv, &meta, &programs);"
new_call = "CognitiveController::decide(&fv, &meta, &programs, &Grid::new(5,5), &Grid::new(5,5));"
count = s.count(old_call)
if count == 0:
    print("[WARNING] Nem található a régi hívás; lehet, hogy már frissítve van.")
else:
    s = s.replace(old_call, new_call)
    print(f"[1] {count} darab hívás frissítve.")

write_file(p, s)

# 3. Tesztek futtatása
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

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Update cognitive_test.rs to match new decide signature"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
