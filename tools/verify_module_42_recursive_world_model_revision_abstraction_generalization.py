#!/usr/bin/env python3
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

STATE = ROOT / "state/project_state.json"

MANIFEST = 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/Cargo.toml'

FREEZE_STAGE = 'module_42_recursive_world_model_revision_abstraction_generalization_freeze'
FREEZE_INVARIANT = 'module_42_recursive_world_model_revision_abstraction_generalization_frozen'

REQUIRED_INVARIANTS = ['recursive_world_revision_abstraction_generalization_complete_clique', 'recursive_world_revision_abstraction_generalization_consensus_application_provenance', 'recursive_world_revision_abstraction_generalization_consensus_conclusion_identity', 'recursive_world_revision_abstraction_generalization_consensus_determinism_non_mutation', 'recursive_world_revision_abstraction_generalization_consensus_facade_equivalence', 'recursive_world_revision_abstraction_generalization_consensus_premise_identity', 'recursive_world_revision_abstraction_generalization_consensus_projection_guard', 'recursive_world_revision_abstraction_generalization_consensus_projection_identity', 'recursive_world_revision_abstraction_generalization_consensus_source_identity', 'recursive_world_revision_abstraction_generalization_consensus_success', 'recursive_world_revision_abstraction_generalization_consensus_support_count', 'recursive_world_revision_abstraction_generalization_consensus_unavailable_guard', 'recursive_world_revision_abstraction_generalization_consensus_vocabulary_identity', 'recursive_world_revision_abstraction_generalization_context_provenance', 'recursive_world_revision_abstraction_generalization_cycle_affordable_revision', 'recursive_world_revision_abstraction_generalization_cycle_budget_guard', 'recursive_world_revision_abstraction_generalization_cycle_determinism_non_mutation', 'recursive_world_revision_abstraction_generalization_cycle_discovery_guard', 'recursive_world_revision_abstraction_generalization_cycle_exact_target_replacement', 'recursive_world_revision_abstraction_generalization_cycle_facade_equivalence', 'recursive_world_revision_abstraction_generalization_cycle_hypothesis_identity', 'recursive_world_revision_abstraction_generalization_cycle_inactive', 'recursive_world_revision_abstraction_generalization_cycle_provenance', 'recursive_world_revision_abstraction_generalization_cycle_rejection_guard', 'recursive_world_revision_abstraction_generalization_cycle_replacement_identity', 'recursive_world_revision_abstraction_generalization_cycle_rule_count_preservation', 'recursive_world_revision_abstraction_generalization_determinism_non_mutation', 'recursive_world_revision_abstraction_generalization_discovery_abstraction_identity', 'recursive_world_revision_abstraction_generalization_discovery_ambiguity_guard', 'recursive_world_revision_abstraction_generalization_discovery_application_provenance', 'recursive_world_revision_abstraction_generalization_discovery_determinism_non_mutation', 'recursive_world_revision_abstraction_generalization_discovery_facade_equivalence', 'recursive_world_revision_abstraction_generalization_discovery_noop_guard', 'recursive_world_revision_abstraction_generalization_discovery_observation_identity', 'recursive_world_revision_abstraction_generalization_discovery_realization_guard', 'recursive_world_revision_abstraction_generalization_discovery_replacement_identity', 'recursive_world_revision_abstraction_generalization_discovery_source_provenance', 'recursive_world_revision_abstraction_generalization_discovery_success', 'recursive_world_revision_abstraction_generalization_discovery_target_identity', 'recursive_world_revision_abstraction_generalization_evidence_scope_confirming_inactive', 'recursive_world_revision_abstraction_generalization_evidence_scope_determinism_non_mutation', 'recursive_world_revision_abstraction_generalization_evidence_scope_discovery_guard', 'recursive_world_revision_abstraction_generalization_evidence_scope_exact_target_pressure', 'recursive_world_revision_abstraction_generalization_evidence_scope_facade_equivalence', 'recursive_world_revision_abstraction_generalization_evidence_scope_hypothesis_identity', 'recursive_world_revision_abstraction_generalization_evidence_scope_no_evidence_inactive', 'recursive_world_revision_abstraction_generalization_evidence_scope_provenance', 'recursive_world_revision_abstraction_generalization_evidence_scope_rejection_guard', 'recursive_world_revision_abstraction_generalization_evidence_scope_rule_observation_identity', 'recursive_world_revision_abstraction_generalization_evidence_scope_state_identity', 'recursive_world_revision_abstraction_generalization_evidence_scope_violating_active', 'recursive_world_revision_abstraction_generalization_facade_equivalence', 'recursive_world_revision_abstraction_generalization_pair_support_count', 'recursive_world_revision_abstraction_generalization_projection_application_provenance', 'recursive_world_revision_abstraction_generalization_projection_conclusion_class_identity', 'recursive_world_revision_abstraction_generalization_projection_conclusion_coverage_guard', 'recursive_world_revision_abstraction_generalization_projection_conflict_guard', 'recursive_world_revision_abstraction_generalization_projection_determinism_non_mutation', 'recursive_world_revision_abstraction_generalization_projection_facade_equivalence', 'recursive_world_revision_abstraction_generalization_projection_observation_count', 'recursive_world_revision_abstraction_generalization_projection_premise_class_identity', 'recursive_world_revision_abstraction_generalization_projection_premise_coverage_guard', 'recursive_world_revision_abstraction_generalization_projection_source_identity', 'recursive_world_revision_abstraction_generalization_projection_success', 'recursive_world_revision_abstraction_generalization_projection_vocabulary_identity', 'recursive_world_revision_abstraction_generalization_realization_ambiguity_provenance', 'recursive_world_revision_abstraction_generalization_realization_conclusion_ambiguity', 'recursive_world_revision_abstraction_generalization_realization_conclusion_witness_identity', 'recursive_world_revision_abstraction_generalization_realization_consensus_guard', 'recursive_world_revision_abstraction_generalization_realization_determinism_non_mutation', 'recursive_world_revision_abstraction_generalization_realization_deterministic_status', 'recursive_world_revision_abstraction_generalization_realization_facade_equivalence', 'recursive_world_revision_abstraction_generalization_realization_materialization', 'recursive_world_revision_abstraction_generalization_realization_premise_ambiguity', 'recursive_world_revision_abstraction_generalization_realization_premise_witness_identity', 'recursive_world_revision_abstraction_generalization_realization_source_provenance', 'recursive_world_revision_abstraction_generalization_realization_vocabulary_identity', 'recursive_world_revision_abstraction_generalization_repeated_context_support', 'recursive_world_revision_abstraction_generalization_resolution_context_provenance', 'recursive_world_revision_abstraction_generalization_resolution_determinism_non_mutation', 'recursive_world_revision_abstraction_generalization_resolution_disjoint_survival', 'recursive_world_revision_abstraction_generalization_resolution_facade_equivalence', 'recursive_world_revision_abstraction_generalization_resolution_identical_merge', 'recursive_world_revision_abstraction_generalization_resolution_identical_vocabulary_identity', 'recursive_world_revision_abstraction_generalization_resolution_no_arbitrary_winner', 'recursive_world_revision_abstraction_generalization_resolution_overlap_conflict', 'recursive_world_revision_abstraction_generalization_resolution_overlap_identity', 'recursive_world_revision_abstraction_generalization_resolution_source_identity', 'recursive_world_revision_abstraction_generalization_resolution_source_provenance', 'recursive_world_revision_abstraction_generalization_resolution_vocabulary_materialization', 'recursive_world_revision_abstraction_generalization_side_separation', 'recursive_world_revision_abstraction_generalization_source_identity', 'recursive_world_revision_abstraction_generalization_threshold_minimum_two', 'recursive_world_revision_abstraction_generalization_threshold_nonzero_guard', 'recursive_world_revision_abstraction_generalization_threshold_rejection', 'recursive_world_revision_abstraction_generalization_transitive_closure_guard', 'recursive_world_revision_abstraction_generalization_validation_acceptance', 'recursive_world_revision_abstraction_generalization_validation_accepted_hypothesis_identity', 'recursive_world_revision_abstraction_generalization_validation_ambiguity_guard', 'recursive_world_revision_abstraction_generalization_validation_collision_rejection', 'recursive_world_revision_abstraction_generalization_validation_determinism_non_mutation', 'recursive_world_revision_abstraction_generalization_validation_facade_equivalence', 'recursive_world_revision_abstraction_generalization_validation_missing_target_rejection', 'recursive_world_revision_abstraction_generalization_validation_model_identity', 'recursive_world_revision_abstraction_generalization_validation_noop_guard', 'recursive_world_revision_abstraction_generalization_validation_provenance', 'recursive_world_revision_abstraction_generalization_validation_rejected_hypothesis_identity', 'recursive_world_revision_abstraction_generalization_validation_rule_identity']

FROZEN_HASHES = {'crates/athlesia_recursive_world_model_revision_abstraction_generalization/Cargo.toml': 'ed1a1f6b7c983e328c84686926bb091c096e1b22d789b6157f5ebc7e66e3df56', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/Cargo.lock': '4daa6b1e3bc97f08ac971266ce5af4ac9f1451e26dc2250355506f4ae33cbef6', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/src/lib.rs': 'd0a3b5074bde03b7cdcee298ab78c64c7ba2875c32bf89e4ec482ec1c6d3ea6c', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests/recursive_world_model_revision_abstraction_generalization_consensus_bridge.rs': '5a32343cafd2dac040560d4eb796c38ab016ddf5618d3b2674f601b7d03bfc74', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests/recursive_world_model_revision_abstraction_generalization_discovery_bridge.rs': 'c2ae1e7dd9ff6d63f4c615b60458b3235d1b149ef5f91c457849c1ca15b19241', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests/recursive_world_model_revision_abstraction_generalization_evidence_scope.rs': 'dd470a05ad88ee659e61059793fc8eeaaa6fc4b37eefae411d904a4d65e0f4ec', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests/recursive_world_model_revision_abstraction_generalization_foundation.rs': '4756f9cd3c40443c3527e1a380d4de92745320eaf0a1bcac0aa2edadfb1cdeab', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests/recursive_world_model_revision_abstraction_generalization_projection_bridge.rs': '0811e09f33fb4cb53a85154b6d345cf8b4e0ba3636e357ebb7daada3dfbaa9ea', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests/recursive_world_model_revision_abstraction_generalization_realization_bridge.rs': 'e08f04b1060dbaa75d00718b9af917af92ec98e33fea8520b4ea4f84eb4c353e', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests/recursive_world_model_revision_abstraction_generalization_resolution.rs': '0fbb7a42d358977d5286e9013c0110f044a2a8daf3d8fa5312623fa58f5499a5', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests/recursive_world_model_revision_abstraction_generalization_revision_cycle.rs': 'bc88cb52f95490fa8ac514f382f57edaa0a5ea0ce50d5b94b0bcfb3b0857f3ee', 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests/recursive_world_model_revision_abstraction_generalization_validation.rs': '2d6d92f8cb0223a86c13da929d413a871ab2ba5b3657cc3c9d789ada18c84ada'}

EXPECTED_TEST_COUNT = 108
EXPECTED_FROZEN_FILE_COUNT = 12
EXPECTED_INVARIANT_COUNT = 108

FORBIDDEN = ['\\bTODO\\b', '\\btodo!\\s*\\(', '\\bunimplemented!\\s*\\(', '\\bFIXME\\b', '\\bPLACEHOLDER\\b', '\\.\\.\\.']


def fail(message):
    print(
        "MODULE 42 RECURSIVE WORLD MODEL REVISION "
        "ABSTRACTION GENERALIZATION VERIFY: FAIL"
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

state = json.loads(
    STATE.read_text(
        encoding="utf-8"
    )
)

completed = state.get(
    "rust_port",
    {},
).get(
    "completed_layers",
    [],
)

validated = state.get(
    "validated_invariants",
    [],
)

if FREEZE_STAGE not in completed:
    fail(
        "Module 42 freeze stage missing from completed_layers"
    )

if FREEZE_INVARIANT not in validated:
    fail(
        "Module 42 freeze invariant missing"
    )

if len(
    REQUIRED_INVARIANTS
) != EXPECTED_INVARIANT_COUNT:
    fail(
        "Embedded invariant count is not 108"
    )

missing_invariants = [
    invariant
    for invariant in REQUIRED_INVARIANTS
    if invariant not in validated
]

if missing_invariants:
    fail(
        "Missing frozen invariants: "
        + ", ".join(
            missing_invariants
        )
    )

if len(
    FROZEN_HASHES
) != EXPECTED_FROZEN_FILE_COUNT:
    fail(
        "Embedded frozen file count is not 12"
    )

for relative, expected_hash in FROZEN_HASHES.items():
    path = ROOT / relative

    if not path.exists():
        fail(
            "Frozen file missing: "
            + relative
        )

    actual_hash = sha(
        path
    )

    if actual_hash != expected_hash:
        fail(
            "Frozen hash mismatch: "
            + relative
        )

    if path.suffix in (
        ".rs",
        ".toml",
    ):
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
                    "Lazy-code firewall violation in "
                    + relative
                )

test_files = sorted(
    (
        ROOT
        / 'crates/athlesia_recursive_world_model_revision_abstraction_generalization/tests'
    ).glob(
        "*.rs"
    )
)

if len(
    test_files
) != 9:
    fail(
        "Expected exactly 9 frozen Module 42 test files"
    )

declared_tests = 0

for test_file in test_files:
    declared_tests += len(
        re.findall(
            r"(?m)^\s*#\[test\]\s*$",
            test_file.read_text(
                encoding="utf-8"
            ),
        )
    )

if declared_tests != EXPECTED_TEST_COUNT:
    fail(
        "Declared Module 42 test count mismatch"
    )

for command in [
    [
        "cargo",
        "fmt",
        "--manifest-path",
        MANIFEST,
        "--all",
        "--",
        "--check",
    ],
    [
        "cargo",
        "check",
        "--manifest-path",
        MANIFEST,
    ],
    [
        "cargo",
        "test",
        "--manifest-path",
        MANIFEST,
    ],
    [
        "cargo",
        "clippy",
        "--manifest-path",
        MANIFEST,
        "--all-targets",
        "--",
        "-D",
        "warnings",
    ],
]:
    result = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
    )

    if result.returncode != 0:
        fail(
            "Frozen Module 42 quality gate failed: "
            + " ".join(
                command
            )
        )

result = subprocess.run(
    (
        "cargo test --manifest-path "
        + MANIFEST
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
    test_count = int(
        result.stdout.strip()
    )
except ValueError:
    fail(
        "Unable to parse Module 42 test count"
    )

if test_count != EXPECTED_TEST_COUNT:
    fail(
        "Runtime Module 42 test count mismatch: "
        + str(
            test_count
        )
    )

print(
    "MODULE 42 RECURSIVE WORLD MODEL REVISION "
    "ABSTRACTION GENERALIZATION VERIFY: PASS"
)

print(
    "Recursive world revision abstraction generalization "
    "integrity gate: 108/108"
)

print(
    "Frozen revision abstraction generalization files:",
    len(
        FROZEN_HASHES
    ),
)

print(
    "Required invariants:",
    len(
        REQUIRED_INVARIANTS
    ),
)

print(
    "Revision abstraction generalization tests:",
    test_count
)
