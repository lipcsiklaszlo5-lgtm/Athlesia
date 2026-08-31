#!/usr/bin/env python3

from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

STATE = ROOT / "state/project_state.json"

MANIFEST = (
    ROOT
    / "state/module_41_recursive_world_model_revision_abstraction_induction_freeze.json"
)

FREEZE_STAGE = (
    "module_41_recursive_world_model_revision_abstraction_induction_freeze"
)

CRATE_MANIFEST = (
    "crates/athlesia_recursive_world_model_revision_abstraction_induction/Cargo.toml"
)

FORBIDDEN = [
    r"\bTODO\b",
    r"\btodo!\s*\(",
    r"\bunimplemented!\s*\(",
    r"\bFIXME\b",
    r"\bPLACEHOLDER\b",
    r"\.\.\.",
]


def fail(message):
    print(
        "MODULE 41 RECURSIVE WORLD MODEL "
        "REVISION ABSTRACTION INDUCTION VERIFY: FAIL"
    )
    print(
        message
    )
    sys.exit(1)


def sha(path):
    return hashlib.sha256(
        path.read_bytes()
    ).hexdigest()


if not STATE.exists():
    fail(
        "state/project_state.json missing"
    )

if not MANIFEST.exists():
    fail(
        "Module 41 freeze manifest missing"
    )

state = json.loads(
    STATE.read_text(
        encoding="utf-8"
    )
)

manifest = json.loads(
    MANIFEST.read_text(
        encoding="utf-8"
    )
)

if manifest.get(
    "module"
) != 41:
    fail(
        "freeze manifest module identity mismatch"
    )

if manifest.get(
    "name"
) != "recursive_world_model_revision_abstraction_induction":
    fail(
        "freeze manifest name mismatch"
    )

expected_tests = manifest.get(
    "expected_test_count"
)

expected_invariants = manifest.get(
    "expected_invariant_count"
)

expected_files = manifest.get(
    "expected_frozen_file_count"
)

if expected_tests != 132:
    fail(
        f"expected test count manifest mismatch: {expected_tests}"
    )

if expected_invariants != 132:
    fail(
        f"expected invariant count manifest mismatch: {expected_invariants}"
    )

if expected_files != 14:
    fail(
        f"expected frozen file count manifest mismatch: {expected_files}"
    )

freeze_invariant = manifest.get(
    "freeze_invariant"
)

if freeze_invariant != (
    "recursive_world_revision_abstraction_induction_freeze_integrity"
):
    fail(
        "freeze invariant identity mismatch"
    )

required_invariants = manifest.get(
    "required_invariants"
)

if not isinstance(
    required_invariants,
    list,
):
    fail(
        "required_invariants is not a list"
    )

if len(
    required_invariants
) != expected_invariants:
    fail(
        "required invariant manifest count mismatch"
    )

if len(
    set(
        required_invariants
    )
) != expected_invariants:
    fail(
        "duplicate required invariants"
    )

if required_invariants != sorted(
    required_invariants
):
    fail(
        "required invariants are not canonical"
    )

for invariant in required_invariants:
    if not invariant.startswith(
        "recursive_world_revision_abstraction_induction_"
    ):
        fail(
            f"foreign Module 41 invariant: {invariant}"
        )

    if invariant == freeze_invariant:
        fail(
            "freeze invariant must not be part of the 132 implementation invariants"
        )

validated = set(
    state.get(
        "validated_invariants",
        [],
    )
)

missing_invariants = [
    invariant
    for invariant in required_invariants
    if invariant not in validated
]

if missing_invariants:
    fail(
        "missing required invariants: "
        + ", ".join(
            missing_invariants
        )
    )

if freeze_invariant not in validated:
    fail(
        "freeze integrity invariant missing"
    )

rust_port = state.get(
    "rust_port",
    {},
)

completed_layers = set(
    rust_port.get(
        "completed_layers",
        [],
    )
)

current_stage = rust_port.get(
    "stage"
)

# Progression-safe:
# once the freeze layer is completed and the freeze
# invariant exists, later stages are allowed.
if (
    current_stage != FREEZE_STAGE
    and FREEZE_STAGE not in completed_layers
):
    fail(
        "Module 41 freeze stage has not been completed"
    )

frozen_files = manifest.get(
    "frozen_files"
)

if not isinstance(
    frozen_files,
    dict,
):
    fail(
        "frozen_files is not an object"
    )

if len(
    frozen_files
) != expected_files:
    fail(
        "frozen file manifest count mismatch"
    )

for relative, expected_sha in sorted(
    frozen_files.items()
):
    path = ROOT / relative

    if not path.exists():
        fail(
            f"frozen file missing: {relative}"
        )

    actual_sha = sha(
        path
    )

    if actual_sha != expected_sha:
        fail(
            f"frozen file hash mismatch: {relative}"
        )

    if path.suffix in {
        ".rs",
        ".toml",
    }:
        text = path.read_text(
            encoding="utf-8"
        )

        for pattern in FORBIDDEN:
            if re.search(
                pattern,
                text,
                re.IGNORECASE,
            ):
                fail(
                    f"lazy-code firewall violation in {relative}: {pattern}"
                )

result = subprocess.run(
    (
        "cargo test --manifest-path "
        + CRATE_MANIFEST
        + " -- --list 2>/dev/null "
        + "| grep ': test$' | wc -l"
    ),
    cwd=ROOT,
    shell=True,
    check=False,
    text=True,
    capture_output=True,
)

if result.returncode != 0:
    fail(
        "Module 41 test count command failed"
    )

try:
    test_count = int(
        result.stdout.strip()
    )
except ValueError:
    fail(
        "Module 41 test count was not numeric"
    )

if test_count != expected_tests:
    fail(
        f"Module 41 test count mismatch: {test_count}/{expected_tests}"
    )

test_result = subprocess.run(
    [
        "cargo",
        "test",
        "--manifest-path",
        CRATE_MANIFEST,
    ],
    cwd=ROOT,
    check=False,
)

if test_result.returncode != 0:
    fail(
        "Module 41 tests failed"
    )

print(
    "MODULE 41 RECURSIVE WORLD MODEL "
    "REVISION ABSTRACTION INDUCTION VERIFY: PASS"
)

print(
    "Recursive world revision abstraction induction integrity gate: "
    f"{expected_invariants}/{expected_invariants}"
)

print(
    "Frozen revision abstraction induction files:",
    expected_files,
)

print(
    "Required invariants:",
    expected_invariants,
)

print(
    "Revision abstraction induction tests:",
    test_count,
)
