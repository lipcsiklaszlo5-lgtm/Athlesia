#!/usr/bin/env python3
import pathlib

files = [
    "crates/athlesia-kernel/src/cognitive.rs",
    "crates/athlesia-kernel/tests/cognitive_test.rs",
    "crates/athlesia-kernel/tests/prior_test.rs",
    "crates/athlesia-kernel/tests/structural_analysis_test.rs",
    "crates/athlesia-kernel/tests/structured_solve_test.rs",
    "crates/athlesia-kernel/tests/abstraction_learning_test.rs",
    "crates/athlesia-kernel/tests/phase9_external_benchmark.rs",
    "crates/athlesia-kernel/src/lib.rs",
    "crates/athlesia-core/src/lib.rs",
    "crates/athlesia-core/tests/concept_transfer_test.rs",
    "crates/athlesia-core/tests/generalization_benchmark.rs",
    "crates/athlesia-core/tests/search_cost_learning_test.rs",
    "crates/athlesia-search/src/lib.rs",
    "crates/athlesia-search/tests/budget_abort_test.rs",
    "crates/athlesia-world-model/src/lib.rs",
    "crates/athlesia-world-model/tests/prediction_error_test.rs",
    "crates/athlesia-abstraction/src/lib.rs",
    "crates/athlesia-hypothesis/src/lib.rs",
    "crates/athlesia-knowledge/src/lib.rs",
    "crates/athlesia-planner/src/lib.rs",
    "crates/athlesia-planner/tests/action_value_test.rs",
    "crates/athlesia-interactive/src/lib.rs",
    "crates/athlesia-interactive/tests/info_gain_benchmark.rs",
    "crates/athlesia-memory/src/lib.rs",
    "crates/athlesia-metalearner/src/lib.rs",
]

for f in files:
    p = pathlib.Path(f)
    if not p.exists():
        print(f"\n===== {f} =====")
        print("[NEM LÉTEZIK]")
        continue
    print(f"\n===== {f} =====")
    try:
        print(p.read_text())
    except Exception as e:
        print(f"HIBA: {e}")
