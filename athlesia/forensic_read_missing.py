#!/usr/bin/env python3
import pathlib

files = [
    "crates/athlesia-kernel/src/cognitive.rs",
    "crates/athlesia-kernel/src/lib.rs",
    "crates/athlesia-kernel/tests/prior_test.rs",
    "crates/athlesia-kernel/tests/cognitive_test.rs",
    "crates/athlesia-kernel/tests/structural_analysis_test.rs",
    "crates/athlesia-kernel/tests/structured_solve_test.rs",
    "crates/athlesia-kernel/tests/abstraction_learning_test.rs",
    "crates/athlesia-kernel/tests/phase9_external_benchmark.rs",
    "crates/athlesia-core/src/lib.rs",
    "crates/athlesia-core/tests/generalization_benchmark.rs",
    "crates/athlesia-core/tests/concept_transfer_test.rs",
    "crates/athlesia-core/tests/search_cost_learning_test.rs",
    "crates/athlesia-search/src/lib.rs",
    "crates/athlesia-search/tests/budget_abort_test.rs",
    "crates/athlesia-world-model/src/lib.rs",
    "crates/athlesia-world-model/tests/prediction_error_test.rs",
    "crates/athlesia-world-model/tests/belief_test.rs",
    "crates/athlesia-abstraction/src/lib.rs",
    "crates/athlesia-hypothesis/src/lib.rs",
]

for f in files:
    p = pathlib.Path(f)
    print(f"\n===== {f} =====")
    if not p.exists():
        print("[NEM LÉTEZIK]")
        continue
    try:
        print(p.read_text())
    except Exception as e:
        print(f"HIBA: {e}")
