#!/usr/bin/env python3
import pathlib

files = [
    "crates/athlesia-search/src/lib.rs",
    "crates/athlesia-world-model/src/lib.rs",
    "crates/athlesia-abstraction/src/lib.rs",
    "crates/athlesia-knowledge/src/lib.rs",
    "crates/athlesia-planner/src/lib.rs",
    "crates/athlesia-interactive/src/lib.rs",
]

for f in files:
    p = pathlib.Path(f)
    print(f"\n===== {f} =====")
    if not p.exists():
        print("[NEM LÉTEZIK]")
        continue
    print(p.read_text())
