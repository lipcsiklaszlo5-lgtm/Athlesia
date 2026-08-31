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
    / "state/module_35_recursive_world_model_revision_proposal_freeze.json"
)

CRATE_MANIFEST = (
    "crates/athlesia_recursive_world_model_revision_proposal/Cargo.toml"
)

FREEZE_STAGE = (
    "module_35_recursive_world_model_revision_proposal_freeze"
)

FREEZE_INVARIANT = (
    "recursive_world_revision_proposal_freeze_integrity"
)


def fail(message):
    print(
        "MODULE 35 RECURSIVE WORLD MODEL "
        "REVISION PROPOSAL VERIFY: FAIL"
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
# the current stage may already be Module 36+.
# The Module 35 freeze only needs to remain
# recorded in completed_layers.

completed_layers = state[
    "rust_port"
]["completed_layers"]

if FREEZE_STAGE not in completed_layers:
    fail(
        "Module 35 freeze is not recorded "
        "as completed."
    )

if FREEZE_INVARIANT not in state[
    "validated_invariants"
]:
    fail(
        "Module 35 freeze integrity "
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
        "recursive_world_revision_proposal_"
    )
    and invariant
    != FREEZE_INVARIANT
])

if actual_module_invariants != sorted(
    required
):
    fail(
        "Module 35 invariant set "
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
        "Could not determine Module 35 "
        "test count."
    )

expected_test_count = manifest[
    "test_count"
]

if actual_test_count != expected_test_count:
    fail(
        "Module 35 test count drift: "
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
        "Module 35 tests failed."
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
        "Module 35 clippy failed."
    )

print(
    "MODULE 35 RECURSIVE WORLD MODEL "
    "REVISION PROPOSAL VERIFY: PASS"
)

print(
    "Recursive world revision proposal "
    "integrity gate: "
    + str(len(required))
    + "/"
    + str(len(required))
)

print(
    "Frozen revision proposal files:",
    len(frozen_hashes),
)

print(
    "Required invariants:",
    len(required),
)

print(
    "Revision proposal tests:",
    actual_test_count,
)
