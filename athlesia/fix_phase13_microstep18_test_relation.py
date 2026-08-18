#!/usr/bin/env python3
import pathlib, subprocess, sys

p = pathlib.Path("crates/athlesia-core/tests/openworld_meta_archive_test.rs")
s = p.read_text()

# A predikció legyen egypixelű objektum a (0,0)-n, a megfigyelés egy másik színű a (1,0)-n.
old_pred = "    let prediction_low = Prediction {\n        state: grid_5x5_with_pixel(1, 0, 0),\n        confidence: 0.5,\n    };"
new_pred = "    let prediction_low = Prediction {\n        state: grid_5x5_with_pixel(0, 0, 1),\n        confidence: 0.5,\n    };"

if old_pred not in s:
    print("[ERROR] A régi prediction blokk nem található.")
    sys.exit(1)
s = s.replace(old_pred, new_pred)

old_obs = "    let observation = Observation {\n        state: grid_5x5_with_pixel(0, 0, 1),\n    };"
new_obs = "    let observation = Observation {\n        state: grid_5x5_with_pixel(1, 0, 2),\n    };"

if old_obs not in s:
    print("[ERROR] A régi observation blokk nem található.")
    sys.exit(1)
s = s.replace(old_obs, new_obs)

p.write_text(s)
print("[1] A teszt predikció/megfigyelés páros módosítva: csak pixel_mismatch feature legyen.")

# Core tesztek futtatása
result = subprocess.run(
    ["cargo", "test", "-p", "athlesia-core", "--test", "openworld_meta_archive_test"],
    capture_output=True,
    text=True,
    check=False,
)
print(result.stdout)
print(result.stderr)
if result.returncode != 0:
    print("\n[FAILURE] Core teszt nem ment át.")
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
subprocess.run(["git", "commit", "-m", "Fix openworld_meta_archive_test: align object counts to produce only pixel_mismatch feature"], check=True)
subprocess.run(["git", "push"], check=True)
print("[INFO] Git commit és push sikeres.")
