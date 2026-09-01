#!/usr/bin/env python3
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

STATE = ROOT / "state/project_state.json"

CRATE_REL = 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization'
MANIFEST = CRATE_REL + "/Cargo.toml"

EXPECTED_TESTS = 96
EXPECTED_INVARIANTS = 96
EXPECTED_FROZEN_FILES = 11

FROZEN_HASHES = {'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/Cargo.toml': '74775a7eb9f0bb6ab393c37f18fd504e1167f5e4ec74779c6e3664fb80c114c0', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/Cargo.lock': '341242b446fc4f152bcd445370507036216c73093ab9fde57d80cca56ed03349', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/src/lib.rs': 'efdd3e639fd377f05a20405e938b93ed4f2152335792319c04cc3062ba1cb01d', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/tests/recursive_world_model_revision_abstraction_composition_generalization_discovery_bridge.rs': '89e89f9a31900e3e5bd98ad407fe9739ed70ced0ee760c279836a4c214e0369b', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/tests/recursive_world_model_revision_abstraction_composition_generalization_evidence_scope.rs': 'c7a904d23507240e242519b871e2b23747b6ca9762d9122a731c49d62d08e1e5', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/tests/recursive_world_model_revision_abstraction_composition_generalization_foundation.rs': '4e50cd3681a98bf49af0ad5584f545f27f3bdbe8ed201d1245eb8dda36ba57fa', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/tests/recursive_world_model_revision_abstraction_composition_generalization_motif_projection.rs': '2e07049123df8f5009fae2a5ee2edf9d94a5d1cf5e7762f751da6eb654f81bc7', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/tests/recursive_world_model_revision_abstraction_composition_generalization_motif_realization.rs': 'd197514e9c53e7f189b919836945f3753e3d14c61f20a8fffcf3cb5c325458a9', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/tests/recursive_world_model_revision_abstraction_composition_generalization_motif_resolution.rs': '53da316ec509d4ff6c47b71d80b6e17f05050d1e25f33de4c4cded096cdfd26c', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/tests/recursive_world_model_revision_abstraction_composition_generalization_revision_cycle.rs': '167bde3963eaeda785be9a252cc5bbec019e5f82121d2aa63e2b409e211e09db', 'crates/athlesia_recursive_world_model_revision_abstraction_composition_generalization/tests/recursive_world_model_revision_abstraction_composition_generalization_validation.rs': '61760abdd6fd44cdbc4a787d3ac81c9e1a6b93302cf56b840a39f5280605675e'}

REQUIRED_INVARIANTS = ['recursive_world_revision_abstraction_composition_generalization_context_local_dedup', 'recursive_world_revision_abstraction_composition_generalization_context_provenance', 'recursive_world_revision_abstraction_composition_generalization_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_generalization_discovery_ambiguity_guard', 'recursive_world_revision_abstraction_composition_generalization_discovery_application_provenance', 'recursive_world_revision_abstraction_composition_generalization_discovery_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_generalization_discovery_deterministic_bridge', 'recursive_world_revision_abstraction_composition_generalization_discovery_facade_equivalence', 'recursive_world_revision_abstraction_composition_generalization_discovery_noop_guard', 'recursive_world_revision_abstraction_composition_generalization_discovery_observation_identity', 'recursive_world_revision_abstraction_composition_generalization_discovery_projection_identity', 'recursive_world_revision_abstraction_composition_generalization_discovery_realization_guard', 'recursive_world_revision_abstraction_composition_generalization_discovery_replacement_identity', 'recursive_world_revision_abstraction_composition_generalization_discovery_selection_provenance', 'recursive_world_revision_abstraction_composition_generalization_discovery_target_identity', 'recursive_world_revision_abstraction_composition_generalization_distinct_context_guard', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_confirming_inactive', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_discovery_guard', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_empty_inactive', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_evidence_provenance', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_exact_violation_active', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_facade_equivalence', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_hypothesis_identity', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_other_target_inactive', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_projection_application_provenance', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_rejection_guard', 'recursive_world_revision_abstraction_composition_generalization_evidence_scope_target_replacement_observation_identity', 'recursive_world_revision_abstraction_composition_generalization_exact_three_class_motif', 'recursive_world_revision_abstraction_composition_generalization_facade_equivalence', 'recursive_world_revision_abstraction_composition_generalization_integer_threshold', 'recursive_world_revision_abstraction_composition_generalization_order_direction_identity', 'recursive_world_revision_abstraction_composition_generalization_projection_application_identity', 'recursive_world_revision_abstraction_composition_generalization_projection_conflict_exclusion', 'recursive_world_revision_abstraction_composition_generalization_projection_contiguity_guard', 'recursive_world_revision_abstraction_composition_generalization_projection_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_generalization_projection_direction_guard', 'recursive_world_revision_abstraction_composition_generalization_projection_exact_match', 'recursive_world_revision_abstraction_composition_generalization_projection_facade_equivalence', 'recursive_world_revision_abstraction_composition_generalization_projection_missing_match', 'recursive_world_revision_abstraction_composition_generalization_projection_resolution_identity', 'recursive_world_revision_abstraction_composition_generalization_projection_selection_dedup', 'recursive_world_revision_abstraction_composition_generalization_projection_selection_identity', 'recursive_world_revision_abstraction_composition_generalization_projection_window_support', 'recursive_world_revision_abstraction_composition_generalization_realization_class_identity', 'recursive_world_revision_abstraction_composition_generalization_realization_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_generalization_realization_end_ambiguity', 'recursive_world_revision_abstraction_composition_generalization_realization_end_witness_guard', 'recursive_world_revision_abstraction_composition_generalization_realization_facade_equivalence', 'recursive_world_revision_abstraction_composition_generalization_realization_no_middle_arbitrary_witness', 'recursive_world_revision_abstraction_composition_generalization_realization_observation_dedup', 'recursive_world_revision_abstraction_composition_generalization_realization_projection_provenance', 'recursive_world_revision_abstraction_composition_generalization_realization_start_ambiguity', 'recursive_world_revision_abstraction_composition_generalization_realization_start_witness_guard', 'recursive_world_revision_abstraction_composition_generalization_realization_uncovered_noise_ignored', 'recursive_world_revision_abstraction_composition_generalization_realization_unique_endpoint_determinism', 'recursive_world_revision_abstraction_composition_generalization_repeated_class_guard', 'recursive_world_revision_abstraction_composition_generalization_repeated_exact_motif', 'recursive_world_revision_abstraction_composition_generalization_resolution_chain_survival', 'recursive_world_revision_abstraction_composition_generalization_resolution_cross_context_conflict', 'recursive_world_revision_abstraction_composition_generalization_resolution_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_generalization_resolution_disjoint_survival', 'recursive_world_revision_abstraction_composition_generalization_resolution_endpoint_conflict', 'recursive_world_revision_abstraction_composition_generalization_resolution_facade_equivalence', 'recursive_world_revision_abstraction_composition_generalization_resolution_identity_non_conflict', 'recursive_world_revision_abstraction_composition_generalization_resolution_no_conflict_winner', 'recursive_world_revision_abstraction_composition_generalization_resolution_prefix_conflict', 'recursive_world_revision_abstraction_composition_generalization_resolution_shifted_overlap_compatibility', 'recursive_world_revision_abstraction_composition_generalization_resolution_single_overlap_compatibility', 'recursive_world_revision_abstraction_composition_generalization_resolution_suffix_conflict', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_confirming_inactive', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_discovery_unavailable_guard', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_facade_equivalence', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_full_provenance', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_inactive_guard', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_insufficient_budget_no_revision', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_model_non_mutation', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_other_target_inactive', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_rejection_guard', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_replacement_identity', 'recursive_world_revision_abstraction_composition_generalization_revision_cycle_sufficient_budget_revision', 'recursive_world_revision_abstraction_composition_generalization_single_context_rejection', 'recursive_world_revision_abstraction_composition_generalization_threshold_minimum_two', 'recursive_world_revision_abstraction_composition_generalization_validation_acceptance', 'recursive_world_revision_abstraction_composition_generalization_validation_accepted_hypothesis_identity', 'recursive_world_revision_abstraction_composition_generalization_validation_collision_rejection', 'recursive_world_revision_abstraction_composition_generalization_validation_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_generalization_validation_discovery_guard', 'recursive_world_revision_abstraction_composition_generalization_validation_facade_equivalence', 'recursive_world_revision_abstraction_composition_generalization_validation_missing_target_rejection', 'recursive_world_revision_abstraction_composition_generalization_validation_model_application_provenance', 'recursive_world_revision_abstraction_composition_generalization_validation_noop_guard', 'recursive_world_revision_abstraction_composition_generalization_validation_projection_provenance', 'recursive_world_revision_abstraction_composition_generalization_validation_rejected_hypothesis_identity', 'recursive_world_revision_abstraction_composition_generalization_validation_target_replacement_identity']

FORBIDDEN = ['\\bTODO\\b', '\\btodo!\\s*\\(', '\\bunimplemented!\\s*\\(', '\\bFIXME\\b', '\\bPLACEHOLDER\\b', '\\.\\.\\.']


def run(cmd):
    print("$ " + " ".join(cmd))

    return subprocess.run(
        cmd,
        cwd=ROOT,
        check=False,
    )


def sha(path):
    return hashlib.sha256(
        path.read_bytes()
    ).hexdigest()


def module_test_count():
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
        return int(
            result.stdout.strip()
        )

    except ValueError:
        return -1


# ============================================================
# REGEX SELF-CHECK
# ============================================================

for pattern in FORBIDDEN:
    try:
        re.compile(
            pattern,
            re.IGNORECASE,
        )

    except re.error as exc:
        print(
            "MODULE 44 VERIFY: FAIL"
        )

        print(
            "Invalid firewall regex:",
            repr(
                pattern
            ),
        )

        print(
            "Regex error:",
            exc,
        )

        sys.exit(1)

# ============================================================
# HASH INTEGRITY
# ============================================================

if (
    len(
        FROZEN_HASHES
    )
    != EXPECTED_FROZEN_FILES
):
    print(
        "MODULE 44 VERIFY: FAIL"
    )

    print(
        "Frozen file table count mismatch"
    )

    sys.exit(1)

for relative, expected in sorted(
    FROZEN_HASHES.items()
):
    path = ROOT / relative

    if not path.exists():
        print(
            "MODULE 44 VERIFY: FAIL"
        )

        print(
            "Missing frozen file:",
            relative,
        )

        sys.exit(1)

    actual = sha(
        path
    )

    if actual != expected:
        print(
            "MODULE 44 VERIFY: FAIL"
        )

        print(
            "Frozen hash mismatch:",
            relative,
        )

        print(
            "expected:",
            expected,
        )

        print(
            "actual  :",
            actual,
        )

        sys.exit(1)

# ============================================================
# LAZY CODE FIREWALL
# ============================================================

for relative in sorted(
    FROZEN_HASHES
):
    path = ROOT / relative

    if path.suffix != ".rs":
        continue

    source = path.read_text(
        encoding="utf-8"
    )

    for pattern in FORBIDDEN:
        if re.search(
            pattern,
            source,
            re.IGNORECASE,
        ):
            print(
                "MODULE 44 VERIFY: FAIL"
            )

            print(
                "Lazy-code pattern:",
                relative,
                pattern,
            )

            sys.exit(1)

# ============================================================
# STATE INVARIANT INTEGRITY
# ============================================================

state = json.loads(
    STATE.read_text(
        encoding="utf-8"
    )
)

validated = set(
    state[
        "validated_invariants"
    ]
)

missing = [
    invariant
    for invariant in REQUIRED_INVARIANTS
    if invariant not in validated
]

if missing:
    print(
        "MODULE 44 VERIFY: FAIL"
    )

    print(
        "Missing required invariants:"
    )

    for invariant in missing:
        print(
            " -",
            invariant,
        )

    sys.exit(1)

if (
    len(
        REQUIRED_INVARIANTS
    )
    != EXPECTED_INVARIANTS
):
    print(
        "MODULE 44 VERIFY: FAIL"
    )

    print(
        "Required invariant table count mismatch"
    )

    sys.exit(1)

# ============================================================
# QUALITY
# ============================================================

commands = [
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
]

for command in commands:
    if run(
        command
    ).returncode != 0:
        print(
            "MODULE 44 VERIFY: FAIL"
        )

        sys.exit(1)

test_count = (
    module_test_count()
)

if test_count != EXPECTED_TESTS:
    print(
        "MODULE 44 VERIFY: FAIL"
    )

    print(
        "Expected tests:",
        EXPECTED_TESTS,
    )

    print(
        "Actual tests:",
        test_count,
    )

    sys.exit(1)

print(
    "MODULE 44 RECURSIVE WORLD MODEL REVISION "
    "ABSTRACTION COMPOSITION GENERALIZATION VERIFY: PASS"
)

print(
    "Recursive world revision abstraction composition "
    "generalization integrity gate: "
    f"{EXPECTED_INVARIANTS}/{EXPECTED_INVARIANTS}"
)

print(
    "Frozen revision abstraction composition generalization files:",
    EXPECTED_FROZEN_FILES,
)

print(
    "Required invariants:",
    EXPECTED_INVARIANTS,
)

print(
    "Revision abstraction composition generalization tests:",
    test_count,
)
