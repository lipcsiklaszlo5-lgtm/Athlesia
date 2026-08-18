#!/usr/bin/env python3
import pathlib

files = [
    "crates/athlesia-kernel/tests/prior_test.rs",
    "crates/athlesia-kernel/tests/cognitive_test.rs",
    "crates/athlesia-kernel/tests/structural_analysis_test.rs",
    "crates/athlesia-kernel/tests/structured_solve_test.rs",
    "crates/athlesia-kernel/tests/abstraction_learning_test.rs",
    "crates/athlesia-kernel/tests/phase9_external_benchmark.rs",
]

for f in files:
    p = pathlib.Path(f)
    print(f"\n===== {f} =====")
    if not p.exists():
        print("[NEM LÉTEZIK]")
        continue
    print(p.read_text())
