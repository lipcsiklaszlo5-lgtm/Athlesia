#!/usr/bin/env python3
from pathlib import Path
import hashlib
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / "state/project_state.json"
MANIFEST = ROOT / "state/module_24_revision_freeze.json"

def digest(path):
    return hashlib.sha256(
        path.read_bytes()
    ).hexdigest()

def fail(message):
    print(
        "MODULE 24 REVISION VERIFY: FAIL"
    )
    print(message)
    return False

ok = True

if not STATE.exists():
    ok = fail("Missing project state.")

if not MANIFEST.exists():
    ok = fail("Missing Module 24 manifest.")

if ok:
    state = json.loads(
        STATE.read_text(encoding="utf-8")
    )

    manifest = json.loads(
        MANIFEST.read_text(encoding="utf-8")
    )

    if manifest.get("module") != 24:
        ok = fail("Wrong module number.")

if ok:
    frozen = manifest.get(
        "frozen_files",
        {},
    )

    for relative, expected in frozen.items():
        path = ROOT / relative

        if not path.is_file():
            ok = fail(
                f"Missing frozen file: {relative}"
            )
            break

        actual = digest(path)

        if actual != expected:
            ok = fail(
                f"Frozen file drift: {relative}"
            )
            break

if ok:
    validated = set(
        state.get(
            "validated_invariants",
            [],
        )
    )

    required = manifest.get(
        "required_invariants",
        [],
    )

    missing = [
        invariant
        for invariant in required
        if invariant not in validated
    ]

    if missing:
        ok = fail(
            "Missing invariants: "
            + ", ".join(missing)
        )

if ok:
    result = subprocess.run(
        [
            "cargo",
            "test",
            "--manifest-path",
            "crates/athlesia_revision/Cargo.toml",
            "--",
            "--list",
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )

    if result.returncode != 0:
        ok = fail(
            "Unable to enumerate revision tests."
        )
    else:
        count = sum(
            1
            for line in result.stdout.splitlines()
            if line.rstrip().endswith(": test")
        )

        expected = manifest.get(
            "expected_test_count"
        )

        if count != expected:
            ok = fail(
                f"Revision test count drift: "
                f"{count} != {expected}"
            )

if ok:
    print(
        "MODULE 24 REVISION VERIFY: PASS"
    )
    print(
        "Revision integrity gate: "
        f"{len(manifest['required_invariants'])}/"
        f"{len(manifest['required_invariants'])}"
    )
    print(
        "Frozen revision files:",
        len(manifest["frozen_files"]),
    )
    print(
        "Required invariants:",
        len(manifest["required_invariants"]),
    )
    print(
        "Revision tests:",
        manifest["expected_test_count"],
    )

sys.exit(0 if ok else 1)
