#!/usr/bin/env python3
import pathlib

files_to_print = [
    "crates/athlesia-kernel/src/cognitive.rs",
    "crates/athlesia-kernel/src/lib.rs",
    "crates/athlesia-verifier/src/lib.rs",
    "crates/athlesia-world-model/src/lib.rs",
]

for fpath in files_to_print:
    p = pathlib.Path(fpath)
    if p.exists():
        print("\n" + "=" * 60)
        print(f"FILE: {fpath}")
        print("=" * 60)
        try:
            print(p.read_text())
        except Exception as e:
            print(f"Error reading: {e}")
    else:
        print(f"\n[WARNING] {fpath} does not exist.")

print("\n[INFO] Print finished.")
