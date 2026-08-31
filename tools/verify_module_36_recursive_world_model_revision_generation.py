#!/usr/bin/env python3
from pathlib import Path
import hashlib
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

STATE = (
    ROOT
    / "state/project_state.json"
)

MANIFEST = (
    ROOT
    / "state/module_36_recursive_world_model_revision_generation_freeze.json"
)

CRATE_MANIFEST = (
    "crates/athlesia_recursive_world_model_revision_generation/Cargo.toml"
)

FREEZE_STAGE = (
    "module_36_recursive_world_model_revision_generation_freeze"
)

FREEZE_INVARIANT = (
    "recursive_world_revision_generation_freeze_integrity"
)


def fail(message):
    print(
        "MODULE 36 RECURSIVE WORLD MODEL "
        "REVISION GENERATION VERIFY: FAIL"
    )

    print(
        message
    )

    sys.exit(1)


def sha(path):
    return hashlib.sha256(
        path.read_bytes()
    ).hexdigest()


def run(cmd):
    return subprocess.run(
        cmd,
        cwd=ROOT,
        check=False,
    )


if not STATE.exists():
    fail(
        "Project state missing."
    )

if not MANIFEST.exists():
    fail(
        "Freeze manifest missing."
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

# Progression-safe:
# current stage may already be Module 37+.
# Module 36 only needs its freeze recorded
# and its frozen integrity preserved.

completed_layers = state[
    "rust_port"
]["completed_layers"]

if FREEZE_STAGE not in completed_layers:
    fail(
        "Module 36 freeze is not recorded "
        "as completed."
    )

if FREEZE_INVARIANT not in state[
    "validated_invariants"
]:
    fail(
        "Module 36 freeze integrity "
        "invariant missing."
    )

required = manifest[
    "required_invariants"
]

if len(required) != manifest[
    "invariant_count"
]:
    fail(
        "Freeze manifest invariant "
        "count mismatch."
    )

for invariant in required:
    if invariant not in state[
        "validated_invariants"
    ]:
        fail(
            "Missing frozen invariant: "
            + invariant
        )

actual_module_invariants = sorted([
    invariant
    for invariant in state[
        "validated_invariants"
    ]
    if invariant.startswith(
        "recursive_world_revision_generation_"
    )
    and invariant != FREEZE_INVARIANT
])

if actual_module_invariants != sorted(
    required
):
    fail(
        "Module 36 invariant set "
        "drift detected."
    )

frozen_hashes = manifest[
    "frozen_sha256"
]

if len(frozen_hashes) != manifest[
    "frozen_file_count"
]:
    fail(
        "Freeze manifest frozen file "
        "count mismatch."
    )

for relative, expected in frozen_hashes.items():
    path = ROOT / relative

    if not path.exists():
        fail(
            "Frozen file missing: "
            + relative
        )

    actual = sha(
        path
    )

    if actual != expected:
        fail(
            "Frozen file hash mismatch: "
            + relative
        )

test_list = subprocess.run(
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

try:
    actual_test_count = int(
        test_list.stdout.strip()
    )
except ValueError:
    fail(
        "Could not determine Module 36 "
        "test count."
    )

expected_test_count = manifest[
    "test_count"
]

if actual_test_count != expected_test_count:
    fail(
        "Module 36 test count drift: "
        + str(actual_test_count)
        + " != "
        + str(expected_test_count)
    )

tests = run([
    "cargo",
    "test",
    "--manifest-path",
    CRATE_MANIFEST,
])

if tests.returncode != 0:
    fail(
        "Module 36 tests failed."
    )

clippy = run([
    "cargo",
    "clippy",
    "--manifest-path",
    CRATE_MANIFEST,
    "--all-targets",
    "--",
    "-D",
    "warnings",
])

if clippy.returncode != 0:
    fail(
        "Module 36 clippy failed."
    )

print(
    "MODULE 36 RECURSIVE WORLD MODEL "
    "REVISION GENERATION VERIFY: PASS"
)

print(
    "Recursive world revision generation "
    "integrity gate: "
    + str(len(required))
    + "/"
    + str(len(required))
)

print(
    "Frozen revision generation files:",
    len(frozen_hashes),
)

print(
    "Required invariants:",
    len(required),
)

print(
    "Revision generation tests:",
    actual_test_count,
)
