#!/usr/bin/env python3
from pathlib import Path
import hashlib
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]

MANIFEST='crates/athlesia_meta_learning_skill_memory/Cargo.toml'
SOURCE_COMMIT='d022a0bd11877985cb4f600d1fcdd30873d418d3'
EXPECTED_TESTS=120

FROZEN_FILES={
    "crates/athlesia_meta_learning_skill_memory/Cargo.lock": "641aac08044b700a765d2e6a93508989365f97801ae817d73c69c3c99c00506d",
    "crates/athlesia_meta_learning_skill_memory/Cargo.toml": "3ecb2ef81d5a7d2e6eb902981ff7fe2a1446969521a0544a22bb0ba360954265",
    "crates/athlesia_meta_learning_skill_memory/src/lib.rs": "7d90563d57c3ae9196eb5ee4bc01f43991f01aef7ce0f0cf8cb96fea3ba3aa04",
    "crates/athlesia_meta_learning_skill_memory/tests/meta_learning_cross_context_skill_generalization.rs": "1d2c3337a2c8f1cd3a296cd5b8d43d9f1cb9d4c87c7f8cce10ef02131e66b72d",
    "crates/athlesia_meta_learning_skill_memory/tests/meta_learning_repeated_skill_candidate_discovery.rs": "94003729f2ffd64425216b6d947f3cd678af993fd982c26bcaaaa5577c39a808",
    "crates/athlesia_meta_learning_skill_memory/tests/meta_learning_skill_compression.rs": "d7482d0221a10b7d7d95dd0fb49fdd4c897af01292d1121e5519091b694c9c02",
    "crates/athlesia_meta_learning_skill_memory/tests/meta_learning_skill_memory_foundation.rs": "a3bd6e23372dc7615e8d0cd24bd103a8127ac796abf9e9d26fde79ee73d3d2c0",
    "crates/athlesia_meta_learning_skill_memory/tests/meta_learning_skill_retrieval_and_reuse.rs": "81918a4b3634fd4bdfaee8fc59650795e66e88d665cab5bbf64fc25956aad16b",
    "crates/athlesia_meta_learning_skill_memory/tests/meta_learning_skill_structural_abstraction_induction.rs": "d76a172b432ad297eebb5ce98d24a0c50834477e84caa56aa136ec6299131309"
}

def sha(path):
    return hashlib.sha256(
        (ROOT/path).read_bytes()
    ).hexdigest()

def run(command):
    return subprocess.run(
        command,
        cwd=ROOT
    ).returncode

def output(command):
    return subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True
    ).stdout.strip()

def test_count():
    process=subprocess.run(
        [
            "cargo","test",
            "--manifest-path",MANIFEST,
            "--","--list"
        ],
        cwd=ROOT,
        text=True,
        capture_output=True
    )

    if process.returncode:
        return -1

    return sum(
        line.rstrip().endswith(": test")
        for line in process.stdout.splitlines()
    )

def fail(message):
    print(
        "MODULE 49 META-LEARNING SKILL MEMORY VERIFY: FAIL"
    )
    print(message)
    sys.exit(1)

for relative,expected in FROZEN_FILES.items():
    path=ROOT/relative

    if not path.is_file():
        fail(
            "MISSING FROZEN FILE: "
            +relative
        )

    actual=sha(relative)

    if actual!=expected:
        fail(
            "FROZEN FILE DRIFT: "
            +relative
        )

if run([
    "git","cat-file",
    "-e",
    SOURCE_COMMIT+"^{commit}"
]):
    fail(
        "FROZEN SOURCE COMMIT MISSING"
    )

tests=test_count()

if tests!=EXPECTED_TESTS:
    fail(
        "M49 TEST COUNT DRIFT: "
        +str(tests)
        +"/"
        +str(EXPECTED_TESTS)
    )

if run([
    "cargo","clippy",
    "--manifest-path",MANIFEST,
    "--all-targets",
    "--","-D","warnings"
]):
    fail(
        "M49 CLIPPY REGRESSION"
    )

print(
    "MODULE 49 META-LEARNING SKILL MEMORY VERIFY: PASS"
)
print(
    "Meta-learning skill memory integrity gate: "
    +str(EXPECTED_TESTS)
    +"/"
    +str(EXPECTED_TESTS)
)
print(
    "Frozen meta-learning skill memory files:",
    len(FROZEN_FILES)
)
print(
    "Meta-learning skill memory tests:",
    EXPECTED_TESTS
)
print(
    "Frozen source commit:",
    SOURCE_COMMIT
)
