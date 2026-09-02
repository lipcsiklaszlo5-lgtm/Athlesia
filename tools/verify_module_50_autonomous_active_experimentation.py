#!/usr/bin/env python3
from pathlib import Path
import hashlib
import re
import subprocess
import sys

R = Path(__file__).resolve().parents[1]

CRATE = 'crates/athlesia_autonomous_active_experimentation'
MANIFEST = 'crates/athlesia_autonomous_active_experimentation/Cargo.toml'
SOURCE_COMMIT = '3fa4e05975f2587d3dbb5943e950bce3619c2c65'
EXPECTED_TESTS = 120

FROZEN_HASHES = {'crates/athlesia_autonomous_active_experimentation/Cargo.lock': '1c12abed42af5b27b1bea85545134dfa5356790380225f867fa22410c5c6291a', 'crates/athlesia_autonomous_active_experimentation/Cargo.toml': '49498766817cc955e460692355ff4c0b30cbe6cb5c8add30139fd646632ce666', 'crates/athlesia_autonomous_active_experimentation/src/lib.rs': 'bd90df9dd9965b91901e4e964ba54d71d9690c025fc04d3399c8993897a74acb', 'crates/athlesia_autonomous_active_experimentation/tests/autonomous_active_experimentation_foundation.rs': '7c3a148fc604232d9e66f4f4fa9a023a920ad0b8bc36be0759e71d074731d62c'}

def run(cmd):
    return subprocess.run(
        cmd,
        cwd=R,
    ).returncode

def output(cmd):
    return subprocess.run(
        cmd,
        cwd=R,
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip()

def sha(rel):
    return hashlib.sha256(
        (R / rel).read_bytes()
    ).hexdigest()

def test_count():
    process = subprocess.run(
        [
            "cargo",
            "test",
            "--manifest-path",
            MANIFEST,
            "--",
            "--list",
        ],
        cwd=R,
        text=True,
        capture_output=True,
    )

    if process.returncode:
        return -1

    return sum(
        line.rstrip().endswith(": test")
        for line in process.stdout.splitlines()
    )

def fail(message):
    print(
        "MODULE 50 AUTONOMOUS ACTIVE EXPERIMENTATION VERIFY: FAIL"
    )
    print(message)
    raise SystemExit(1)

if run([
    "git",
    "cat-file",
    "-e",
    SOURCE_COMMIT + "^{commit}",
]):
    fail(
        "Frozen source commit is unavailable"
    )

tracked = sorted(
    filter(
        None,
        output([
            "git",
            "ls-files",
            CRATE,
        ]).splitlines(),
    )
)

expected_files = sorted(
    FROZEN_HASHES
)

if tracked != expected_files:
    fail(
        "Frozen file frontier changed"
    )

for rel, expected_sha in sorted(
    FROZEN_HASHES.items()
):
    path = R / rel

    if not path.is_file():
        fail(
            "Frozen file missing: "
            + rel
        )

    actual_sha = sha(rel)

    if actual_sha != expected_sha:
        fail(
            "Frozen file drift: "
            + rel
        )

source_diff = output([
    "git",
    "diff",
    "--name-only",
    SOURCE_COMMIT,
    "--",
    CRATE,
])

if source_diff:
    fail(
        "Current M50 source differs from frozen source commit"
    )

for rel in expected_files:
    if not rel.endswith(".rs"):
        continue

    text = (
        R / rel
    ).read_text()

    forbidden = [
        r"\bTODO\b",
        r"\btodo!\s*\(",
        r"\bunimplemented!\s*\(",
        r"\bFIXME\b",
        r"\bPLACEHOLDER\b",
        r"\.\.\.",
        r"#\s*\[\s*allow\s*\(",
    ]

    for pattern in forbidden:
        if re.search(
            pattern,
            text,
            re.I,
        ):
            fail(
                "Frozen lazy-code firewall violation in "
                + rel
            )

if run([
    "cargo",
    "fmt",
    "--manifest-path",
    MANIFEST,
    "--all",
    "--",
    "--check",
]):
    fail(
        "cargo fmt check failed"
    )

if run([
    "cargo",
    "test",
    "--manifest-path",
    MANIFEST,
]):
    fail(
        "cargo test failed"
    )

if test_count() != EXPECTED_TESTS:
    fail(
        "Frozen M50 test-count drift"
    )

if run([
    "cargo",
    "clippy",
    "--manifest-path",
    MANIFEST,
    "--all-targets",
    "--",
    "-D",
    "warnings",
]):
    fail(
        "cargo clippy failed"
    )

print(
    "MODULE 50 AUTONOMOUS ACTIVE EXPERIMENTATION VERIFY: PASS"
)
print(
    "Autonomous active experimentation integrity gate: "
    + str(EXPECTED_TESTS)
    + "/"
    + str(EXPECTED_TESTS)
)
print(
    "Frozen autonomous active experimentation files: "
    + str(len(FROZEN_HASHES))
)
print(
    "Required invariants: "
    + str(EXPECTED_TESTS)
)
print(
    "Autonomous active experimentation tests: "
    + str(EXPECTED_TESTS)
)
print(
    "Frozen source commit: "
    + SOURCE_COMMIT
)
