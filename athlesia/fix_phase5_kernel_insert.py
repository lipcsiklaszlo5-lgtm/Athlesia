#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

p = pathlib.Path("crates/athlesia-kernel/src/lib.rs")
s = p.read_text()

# Pontos minta a learn_from_error utáni blokkra
old_text = '''                agent.wm.learn_from_error(id, &error);
            }
            agent.wm.record_prediction_error(error);'''

new_text = '''                agent.wm.learn_from_error(id, &error);
                agent.core.meta.record_failure_in_context(
                    athlesia_features::extract_features(&input_grid),
                    id,
                );
            }
            agent.wm.record_prediction_error(error);'''

if old_text not in s:
    print("[ERROR] Nem található a várt szövegrész a lib.rs-ben.")
    print("Keresett szöveg:\n", old_text)
    sys.exit(1)

s = s.replace(old_text, new_text)
write_file(p, s)
print("[1] Kernel lib.rs frissítve: meta.record_failure_in_context hívása a blokkon belül.")

# Futtassuk a metalearner tesztet is, mert a Phase 5 első fele már lefutott
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-metalearner"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Metalearner tesztek nem mentek át.")
    sys.exit(1)

# Kernel teszt
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

print("\n[SUCCESS] Phase 5 tesztek zöldek.")

subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Fix Phase 5: insert record_failure_in_context inside hypothesis error block"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
