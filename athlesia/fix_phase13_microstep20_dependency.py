#!/usr/bin/env python3
import pathlib, subprocess, sys

def write_file(path, content):
    pathlib.Path(path).parent.mkdir(parents=True, exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)

# 1. Cargo.toml frissítése
p = pathlib.Path("crates/athlesia-core/Cargo.toml")
s = p.read_text()

if "athlesia-planner" not in s:
    # Beszúrjuk a [dependencies] szekcióba, a többi path függőség közé.
    # Egyszerűen a [dependencies] sor után szúrjuk be.
    if "[dependencies]" in s:
        s = s.replace(
            "[dependencies]\n",
            "[dependencies]\nathlesia-planner = { path = \"../athlesia-planner\" }\n",
            1,
        )
    else:
        # Nem várható, de ha nincs, létrehozzuk.
        s += "\n[dependencies]\nathlesia-planner = { path = \"../athlesia-planner\" }\n"
    p.write_text(s)
    print("[1] athlesia-core Cargo.toml: athlesia-planner függőség hozzáadva.")
else:
    print("[1] athlesia-planner már szerepel a függőségek között.")

# 2. Core tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core tesztek nem mentek át.")
    sys.exit(1)
print("\n[SUCCESS] Core tesztek zöldek.")

# 3. Teljes workspace teszt
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
print("\n[SUCCESS] Teljes workspace tesztek zöldek.")

# 4. Git commit és push
subprocess.run(["git", "add", "-A"], check=True)
subprocess.run(["git", "commit", "-m", "Add athlesia-planner dependency to athlesia-core for ExperimentRequest"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
