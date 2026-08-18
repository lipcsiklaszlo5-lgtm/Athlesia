#!/usr/bin/env python3
import pathlib

files = [
    "crates/athlesia-core/tests/generalization_benchmark.rs",
    "crates/athlesia-core/tests/concept_transfer_test.rs",
    "crates/athlesia-core/tests/search_cost_learning_test.rs",
    "crates/athlesia-search/tests/budget_abort_test.rs",
    "crates/athlesia-world-model/tests/prediction_error_test.rs",
    "crates/athlesia-world-model/tests/belief_test.rs",
    "crates/athlesia-interactive/tests/info_gain_benchmark.rs",
]

for f in files:
    p = pathlib.Path(f)
    print(f"\n===== {f} =====")
    if not p.exists():
        print("[NEM LÉTEZIK]")
        continue
    print(p.read_text())
