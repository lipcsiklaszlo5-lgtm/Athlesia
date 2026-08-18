#!/usr/bin/env python3
import pathlib

files = [
    "crates/athlesia-kernel/src/cognitive.rs",
    "crates/athlesia-kernel/src/lib.rs",
    "crates/athlesia-core/src/lib.rs",
]

for f in files:
    p = pathlib.Path(f)
    print(f"\n===== {f} =====")
    if not p.exists():
        print("[NEM LÉTEZIK]")
        continue
    print(p.read_text())
