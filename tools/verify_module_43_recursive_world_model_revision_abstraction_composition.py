#!/usr/bin/env python3

from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

STATE = ROOT / "state/project_state.json"

CRATE_REL = 'crates/athlesia_recursive_world_model_revision_abstraction_composition'

MANIFEST = CRATE_REL + "/Cargo.toml"

FREEZE_STAGE = 'module_43_recursive_world_model_revision_abstraction_composition_freeze'

FREEZE_INVARIANT = 'module_43_recursive_world_model_revision_abstraction_composition_frozen'

EXPECTED_TEST_COUNT = 108

EXPECTED_TEST_FILE_COUNT = 9

EXPECTED_FROZEN_FILE_COUNT = 12

REQUIRED_INVARIANTS = ['recursive_world_revision_abstraction_composition_conclusion_coverage_guard', 'recursive_world_revision_abstraction_composition_cycle_affordable_revision', 'recursive_world_revision_abstraction_composition_cycle_application_evidence_provenance', 'recursive_world_revision_abstraction_composition_cycle_budget_guard', 'recursive_world_revision_abstraction_composition_cycle_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_cycle_discovery_guard', 'recursive_world_revision_abstraction_composition_cycle_exact_target_replacement', 'recursive_world_revision_abstraction_composition_cycle_facade_equivalence', 'recursive_world_revision_abstraction_composition_cycle_inactive_guard', 'recursive_world_revision_abstraction_composition_cycle_realization_path_provenance', 'recursive_world_revision_abstraction_composition_cycle_rejection_guard', 'recursive_world_revision_abstraction_composition_cycle_rule_count_preservation', 'recursive_world_revision_abstraction_composition_cycle_rule_hypothesis_identity', 'recursive_world_revision_abstraction_composition_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_directionality', 'recursive_world_revision_abstraction_composition_discovery_ambiguity_guard', 'recursive_world_revision_abstraction_composition_discovery_application_provenance', 'recursive_world_revision_abstraction_composition_discovery_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_discovery_deterministic_bridge', 'recursive_world_revision_abstraction_composition_discovery_facade_equivalence', 'recursive_world_revision_abstraction_composition_discovery_hypothesis_identity', 'recursive_world_revision_abstraction_composition_discovery_noop_guard', 'recursive_world_revision_abstraction_composition_discovery_observation_identity', 'recursive_world_revision_abstraction_composition_discovery_path_identity', 'recursive_world_revision_abstraction_composition_discovery_realization_guard', 'recursive_world_revision_abstraction_composition_discovery_replacement_identity', 'recursive_world_revision_abstraction_composition_discovery_target_identity', 'recursive_world_revision_abstraction_composition_distinct_observation_support', 'recursive_world_revision_abstraction_composition_evidence_scope_confirming_inactive', 'recursive_world_revision_abstraction_composition_evidence_scope_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_evidence_scope_discovery_guard', 'recursive_world_revision_abstraction_composition_evidence_scope_exact_target_pressure', 'recursive_world_revision_abstraction_composition_evidence_scope_facade_equivalence', 'recursive_world_revision_abstraction_composition_evidence_scope_hypothesis_identity', 'recursive_world_revision_abstraction_composition_evidence_scope_no_evidence_inactive', 'recursive_world_revision_abstraction_composition_evidence_scope_path_provenance', 'recursive_world_revision_abstraction_composition_evidence_scope_rejection_guard', 'recursive_world_revision_abstraction_composition_evidence_scope_rule_observation_identity', 'recursive_world_revision_abstraction_composition_evidence_scope_state_identity', 'recursive_world_revision_abstraction_composition_evidence_scope_violating_active', 'recursive_world_revision_abstraction_composition_facade_equivalence', 'recursive_world_revision_abstraction_composition_observation_provenance', 'recursive_world_revision_abstraction_composition_path_branching', 'recursive_world_revision_abstraction_composition_path_cycle_guard', 'recursive_world_revision_abstraction_composition_path_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_path_disconnected_guard', 'recursive_world_revision_abstraction_composition_path_edge_class_identity', 'recursive_world_revision_abstraction_composition_path_exact_adjacency', 'recursive_world_revision_abstraction_composition_path_facade_equivalence', 'recursive_world_revision_abstraction_composition_path_minimum_length', 'recursive_world_revision_abstraction_composition_path_no_transitive_edge_materialization', 'recursive_world_revision_abstraction_composition_path_prefix_preservation', 'recursive_world_revision_abstraction_composition_path_realization_application_identity', 'recursive_world_revision_abstraction_composition_path_realization_conclusion_ambiguity_guard', 'recursive_world_revision_abstraction_composition_path_realization_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_path_realization_deterministic_endpoints', 'recursive_world_revision_abstraction_composition_path_realization_duplicate_observation_dedup', 'recursive_world_revision_abstraction_composition_path_realization_end_witness_guard', 'recursive_world_revision_abstraction_composition_path_realization_endpoint_identity', 'recursive_world_revision_abstraction_composition_path_realization_facade_equivalence', 'recursive_world_revision_abstraction_composition_path_realization_noise_isolation', 'recursive_world_revision_abstraction_composition_path_realization_premise_ambiguity_guard', 'recursive_world_revision_abstraction_composition_path_realization_selection_identity', 'recursive_world_revision_abstraction_composition_path_realization_start_witness_guard', 'recursive_world_revision_abstraction_composition_path_selection_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_path_selection_endpoint_isolation', 'recursive_world_revision_abstraction_composition_path_selection_facade_equivalence', 'recursive_world_revision_abstraction_composition_path_selection_identity_tiebreak', 'recursive_world_revision_abstraction_composition_path_selection_length_tiebreak', 'recursive_world_revision_abstraction_composition_path_selection_no_edge_materialization', 'recursive_world_revision_abstraction_composition_path_selection_single_candidate', 'recursive_world_revision_abstraction_composition_path_selection_source_identity', 'recursive_world_revision_abstraction_composition_path_selection_support_before_length', 'recursive_world_revision_abstraction_composition_path_selection_support_identity', 'recursive_world_revision_abstraction_composition_path_selection_support_priority', 'recursive_world_revision_abstraction_composition_path_selection_weakest_link_priority', 'recursive_world_revision_abstraction_composition_path_source_identity', 'recursive_world_revision_abstraction_composition_path_support_bottleneck_minimum', 'recursive_world_revision_abstraction_composition_path_support_complete_path_coverage', 'recursive_world_revision_abstraction_composition_path_support_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_path_support_edge_support_preservation', 'recursive_world_revision_abstraction_composition_path_support_endpoint_identity', 'recursive_world_revision_abstraction_composition_path_support_facade_equivalence', 'recursive_world_revision_abstraction_composition_path_support_global_bottleneck', 'recursive_world_revision_abstraction_composition_path_support_no_averaging', 'recursive_world_revision_abstraction_composition_path_support_observation_provenance', 'recursive_world_revision_abstraction_composition_path_support_path_identity', 'recursive_world_revision_abstraction_composition_path_support_source_identity', 'recursive_world_revision_abstraction_composition_path_support_weakest_link_guard', 'recursive_world_revision_abstraction_composition_path_two_edge_induction', 'recursive_world_revision_abstraction_composition_premise_coverage_guard', 'recursive_world_revision_abstraction_composition_repeated_support', 'recursive_world_revision_abstraction_composition_self_loop_guard', 'recursive_world_revision_abstraction_composition_threshold_minimum_two', 'recursive_world_revision_abstraction_composition_threshold_nonzero_guard', 'recursive_world_revision_abstraction_composition_threshold_rejection', 'recursive_world_revision_abstraction_composition_validation_acceptance', 'recursive_world_revision_abstraction_composition_validation_accepted_hypothesis_identity', 'recursive_world_revision_abstraction_composition_validation_collision_rejection', 'recursive_world_revision_abstraction_composition_validation_determinism_non_mutation', 'recursive_world_revision_abstraction_composition_validation_discovery_guard', 'recursive_world_revision_abstraction_composition_validation_facade_equivalence', 'recursive_world_revision_abstraction_composition_validation_missing_target_rejection', 'recursive_world_revision_abstraction_composition_validation_model_application_provenance', 'recursive_world_revision_abstraction_composition_validation_noop_guard', 'recursive_world_revision_abstraction_composition_validation_path_provenance', 'recursive_world_revision_abstraction_composition_validation_rejected_hypothesis_identity', 'recursive_world_revision_abstraction_composition_validation_rule_identity']

FROZEN_HASHES = {'crates/athlesia_recursive_world_model_revision_abstraction_composition/Cargo.lock': 'aa98714741afcd855a3dbceb5479123a21f5b6e604192a0a4017ac6f8b0dc50f', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/Cargo.toml': '6a07e964258050fdc308a761856d89ab23e88472fbd95b1aafae9af8498adf13', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/src/lib.rs': '601c1dc7eedbff475ca897977c2f4f8331d0cffb07874b2b0f5801c375b52c82', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/tests/recursive_world_model_revision_abstraction_composition_discovery_bridge.rs': '44a495430734387e39a928a952afa67a4aff82414f061ff4d31774fa9a225831', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/tests/recursive_world_model_revision_abstraction_composition_evidence_scope.rs': '249c5e78ff2e49db3458c6f244679a4ed26bd078f1fd839b6ef787b5bb793bb3', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/tests/recursive_world_model_revision_abstraction_composition_foundation.rs': 'fc396cb0c77254ccafb8197e9e86aef9ad9fd16396bdda83bfd32f34644a0084', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/tests/recursive_world_model_revision_abstraction_composition_path_induction.rs': '75501cc0bf7fccfdf2bc46d75bdb133c7bc2fb9db1e3716032a3c20404c133b9', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/tests/recursive_world_model_revision_abstraction_composition_path_realization.rs': 'ed847fbfb48bf4fdaf56f5db96f27c7495669acfae2e3da9b15011316d509477', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/tests/recursive_world_model_revision_abstraction_composition_path_selection.rs': 'a23458bccf512cc191d5a35a835ba21ea175c9df4b857aa8dcb196d9b4edf8cd', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/tests/recursive_world_model_revision_abstraction_composition_path_support.rs': '48820dbbb784c146055acfcd7f03b9d349a1cb9089c5be18ae1aeee6033e94f7', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/tests/recursive_world_model_revision_abstraction_composition_revision_cycle.rs': 'dc8397dd133afaef3857bb8544397705ef2e1b9090370764a092556308c1220a', 'crates/athlesia_recursive_world_model_revision_abstraction_composition/tests/recursive_world_model_revision_abstraction_composition_validation.rs': 'fd7737b3de7f8885ae44ce87d4d441e373bca4716812c91f5c8fde1f2aaec7fb'}

FORBIDDEN = ['\\bTODO\\b', '\\btodo!\\s*\\(', '\\bunimplemented!\\s*\\(', '\\bFIXME\\b', '\\bPLACEHOLDER\\b', '\\.\\.\\.']


def run(cmd):
    print(
        "$ " + " ".join(cmd)
    )

    return subprocess.run(
        cmd,
        cwd=ROOT,
        check=False,
    )


def sha(path):
    return hashlib.sha256(
        path.read_bytes()
    ).hexdigest()


def runtime_test_count():
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


state = json.loads(
    STATE.read_text(
        encoding="utf-8"
    )
)

completed = state[
    "rust_port"
]["completed_layers"]

if FREEZE_STAGE not in completed:
    print(
        "MODULE 43 VERIFY: FAIL"
    )
    print(
        "freeze stage missing from completed_layers"
    )
    sys.exit(1)

validated = set(
    state[
        "validated_invariants"
    ]
)

if FREEZE_INVARIANT not in validated:
    print(
        "MODULE 43 VERIFY: FAIL"
    )
    print(
        "freeze invariant missing"
    )
    sys.exit(1)

missing = [
    invariant
    for invariant in REQUIRED_INVARIANTS
    if invariant not in validated
]

if missing:
    print(
        "MODULE 43 VERIFY: FAIL"
    )
    print(
        "required invariants missing:"
    )

    for invariant in missing:
        print(
            invariant
        )

    sys.exit(1)

if len(
    REQUIRED_INVARIANTS
) != 108:
    print(
        "MODULE 43 VERIFY: FAIL"
    )
    print(
        "embedded invariant count mismatch"
    )
    sys.exit(1)

if len(
    FROZEN_HASHES
) != EXPECTED_FROZEN_FILE_COUNT:
    print(
        "MODULE 43 VERIFY: FAIL"
    )
    print(
        "embedded frozen file count mismatch"
    )
    sys.exit(1)

for relative, expected_hash in FROZEN_HASHES.items():
    path = ROOT / relative

    if not path.exists():
        print(
            "MODULE 43 VERIFY: FAIL"
        )
        print(
            "frozen file missing:",
            relative,
        )
        sys.exit(1)

    actual_hash = sha(
        path
    )

    if actual_hash != expected_hash:
        print(
            "MODULE 43 VERIFY: FAIL"
        )
        print(
            "frozen hash mismatch:",
            relative,
        )
        print(
            "expected:",
            expected_hash,
        )
        print(
            "actual  :",
            actual_hash,
        )
        sys.exit(1)

    if path.suffix in (
        ".rs",
        ".toml",
        ".lock",
    ):
        text = path.read_text(
            encoding="utf-8",
            errors="replace",
        )

        for pattern in FORBIDDEN:
            if re.search(
                pattern,
                text,
                re.IGNORECASE,
            ):
                print(
                    "MODULE 43 VERIFY: FAIL"
                )
                print(
                    "lazy-code pattern:",
                    relative,
                    pattern,
                )
                sys.exit(1)

test_dir = ROOT / CRATE_REL / "tests"

test_files = sorted(
    test_dir.glob(
        "*.rs"
    )
)

if len(
    test_files
) != EXPECTED_TEST_FILE_COUNT:
    print(
        "MODULE 43 VERIFY: FAIL"
    )
    print(
        "test file count:",
        len(
            test_files
        ),
    )
    sys.exit(1)

declared = 0

for path in test_files:
    count = len(
        re.findall(
            r"(?m)^\s*#\[test\]\s*$",
            path.read_text(
                encoding="utf-8"
            ),
        )
    )

    if count != 12:
        print(
            "MODULE 43 VERIFY: FAIL"
        )
        print(
            "test declaration mismatch:",
            path.name,
            count,
        )
        sys.exit(1)

    declared += count

if declared != EXPECTED_TEST_COUNT:
    print(
        "MODULE 43 VERIFY: FAIL"
    )
    print(
        "declared tests:",
        declared,
    )
    sys.exit(1)

quality_commands = [
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

for command in quality_commands:
    if run(
        command
    ).returncode != 0:
        print(
            "MODULE 43 VERIFY: FAIL"
        )
        print(
            "quality gate failed"
        )
        sys.exit(1)

actual_tests = runtime_test_count()

if actual_tests != EXPECTED_TEST_COUNT:
    print(
        "MODULE 43 VERIFY: FAIL"
    )
    print(
        "runtime tests:",
        actual_tests,
    )
    sys.exit(1)

print(
    "MODULE 43 RECURSIVE WORLD MODEL REVISION "
    "ABSTRACTION COMPOSITION VERIFY: PASS"
)

print(
    "Recursive world revision abstraction composition integrity gate: "
    + str(
        len(
            REQUIRED_INVARIANTS
        )
    )
    + "/"
    + str(
        len(
            REQUIRED_INVARIANTS
        )
    )
)

print(
    "Frozen revision abstraction composition files:",
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
    "Revision abstraction composition tests:",
    actual_tests,
)
