#!/usr/bin/env python3
from pathlib import Path
import hashlib
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / "state/project_state.json"
MANIFEST = ROOT / "state/module_30_recursive_control_freeze.json"

def sha(path):
    return hashlib.sha256(
        path.read_bytes()
    ).hexdigest()

def fail(message):
    print(
        "MODULE 30 RECURSIVE CONTROL VERIFY: FAIL"
    )
    print(message)
    sys.exit(1)

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

completed_layers = state["rust_port"]["completed_layers"]

if "module_30_recursive_control_freeze" not in completed_layers:
    fail(
        "Module 30 freeze is not recorded as completed."
    )

required = manifest[
    "required_invariants"
]

expected_invariant_count = manifest[
    "expected_invariant_count"
]

if len(required) != expected_invariant_count:
    fail(
        "Manifest invariant count mismatch."
    )

missing = [
    invariant
    for invariant in required
    if invariant not in state[
        "validated_invariants"
    ]
]

if missing:
    fail(
        "Missing frozen invariants: "
        + ", ".join(missing)
    )

actual_module_invariants = sorted([
    invariant
    for invariant in state[
        "validated_invariants"
    ]
    if invariant.startswith(
        "recursive_control_"
    )
    and invariant != (
        "recursive_control_freeze_integrity"
    )
])

if actual_module_invariants != sorted(required):
    fail(
        "Module 30 invariant set drift detected."
    )

frozen = manifest["files"]

for relative, expected_hash in frozen.items():
    path = ROOT / relative

    if not path.exists():
        fail(
            "Frozen file missing: "
            + relative
        )

    actual_hash = sha(path)

    if actual_hash != expected_hash:
        fail(
            "Frozen file hash mismatch: "
            + relative
        )

test_list = subprocess.run(
    "cargo test --manifest-path "
    "crates/athlesia_recursive_control/Cargo.toml "
    "-- --list 2>/dev/null | "
    "grep ': test$' | wc -l",
    cwd=ROOT,
    shell=True,
    check=False,
    text=True,
    capture_output=True,
)

if test_list.returncode != 0:
    fail(
        "Unable to enumerate Module 30 tests."
    )

try:
    test_count = int(
        test_list.stdout.strip()
    )
except ValueError:
    fail(
        "Invalid Module 30 test count."
    )

expected_test_count = manifest[
    "expected_test_count"
]

if test_count != expected_test_count:
    fail(
        "Unexpected Module 30 test count: "
        + str(test_count)
        + " expected "
        + str(expected_test_count)
    )

tests = subprocess.run(
    [
        "cargo",
        "test",
        "--manifest-path",
        "crates/athlesia_recursive_control/Cargo.toml",
    ],
    cwd=ROOT,
    check=False,
)

if tests.returncode != 0:
    fail(
        "Module 30 tests failed."
    )

clippy = subprocess.run(
    [
        "cargo",
        "clippy",
        "--manifest-path",
        "crates/athlesia_recursive_control/Cargo.toml",
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ],
    cwd=ROOT,
    check=False,
)

if clippy.returncode != 0:
    fail(
        "Module 30 Clippy gate failed."
    )

print(
    "MODULE 30 RECURSIVE CONTROL VERIFY: PASS"
)
print(
    "Recursive control integrity gate: "
    + str(len(required))
    + "/"
    + str(len(required))
)
print(
    "Frozen recursive control files:",
    len(frozen),
)
print(
    "Required invariants:",
    len(required),
)
print(
    "Recursive control tests:",
    test_count,
)
