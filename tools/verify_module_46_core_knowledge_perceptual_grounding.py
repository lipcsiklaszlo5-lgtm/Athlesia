#!/usr/bin/env python3

from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

MANIFEST = 'crates/athlesia_core_knowledge_perceptual_grounding/Cargo.toml'

SOURCE_COMMIT = 'd834ba46e7766e841e989aa8010b32adfce9a8df'
EXPECTED_TESTS = 72
FROZEN_STAGE = 'module_46_core_knowledge_perceptual_grounding_frozen'

FROZEN_FILES = ['crates/athlesia_core_knowledge_perceptual_grounding/Cargo.lock', 'crates/athlesia_core_knowledge_perceptual_grounding/Cargo.toml', 'crates/athlesia_core_knowledge_perceptual_grounding/src/lib.rs', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_action_consequences.rs', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_final_integration_freeze.rs', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_foundation.rs', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_motion_change.rs', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_persistence_tracking.rs', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_topology_relations.rs']
FROZEN_HASHES = {'crates/athlesia_core_knowledge_perceptual_grounding/Cargo.lock': 'fbefd559ec01ff3860d153fdf05e9a29f10ecff62eb63206d1b8e643c35ec7b4', 'crates/athlesia_core_knowledge_perceptual_grounding/Cargo.toml': '04ffcd03f7862aac5423b16466ca1ea5ad1ad65856a76a9f4a4265c6dca64ff8', 'crates/athlesia_core_knowledge_perceptual_grounding/src/lib.rs': 'f9e565a91959d756336ee0baf0f47592328e63a204bb91f7d8c6e367450a12ee', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_action_consequences.rs': '42a0979e1e762a8e5f0e1ac1369ae48aad6c0a28ddf7a30f68910ecde957c08b', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_final_integration_freeze.rs': 'e8f163cf46390eb88c9b23be47f95cc59c0e8af870a8a28b4c51edcf156e40ea', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_foundation.rs': 'a5229032192bdd0bbf0dd58ef55aeeec9410917a164a40ea80d73d63ff32d4cb', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_motion_change.rs': '0e192ae3a9e9eede4fc9ce40f79b9a0e93598b29ee956b292f289f2fe6a44371', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_persistence_tracking.rs': 'a16fbb00c99c8298055f7dd6421d30c173c0dae4e1f96ec96db45528994512cc', 'crates/athlesia_core_knowledge_perceptual_grounding/tests/core_knowledge_perceptual_grounding_topology_relations.rs': '2596aa80657ffdfaecb1681a5b62a8153f47de0d6d91c76771d76bc9e5bc8fe3'}
REQUIRED_INVARIANTS = ['perceptual_grounding_action_consequence_no_mandatory_evidence_axis', 'perceptual_grounding_action_consequence_transition_window_positive_support', 'perceptual_grounding_action_determinism_non_mutation_facade_equivalence', 'perceptual_grounding_action_duplicate_exact_best_evidence', 'perceptual_grounding_action_hard_global_frontier', 'perceptual_grounding_action_hard_per_action_bound', 'perceptual_grounding_action_hard_per_change_bound', 'perceptual_grounding_action_multiple_descriptors_same_action_change', 'perceptual_grounding_action_observation_opaque_source_descriptor', 'perceptual_grounding_action_one_action_multiple_consequences', 'perceptual_grounding_action_one_change_multiple_action_explanations', 'perceptual_grounding_action_self_external_source_capacity', 'perceptual_grounding_canonical_element_order_opaque_signature', 'perceptual_grounding_canonical_membership_and_frame_grounding', 'perceptual_grounding_change_common_symmetric_relative_directional', 'perceptual_grounding_change_comparative_same_window_distinct_transition', 'perceptual_grounding_change_determinism_non_mutation_facade_equivalence', 'perceptual_grounding_change_duplicate_exact_best_evidence', 'perceptual_grounding_change_hard_global_frontier', 'perceptual_grounding_change_hard_per_transition_bound', 'perceptual_grounding_change_kind_reference_capacity', 'perceptual_grounding_change_membership_signature_change_motion_support', 'perceptual_grounding_change_multiple_kinds_descriptors_same_transition', 'perceptual_grounding_change_no_mandatory_single_evidence_axis', 'perceptual_grounding_change_persistence_grounded_transition', 'perceptual_grounding_change_relative_common_coexistence_no_implication', 'perceptual_grounding_determinism_non_mutation_facade_equivalence', 'perceptual_grounding_duplicate_membership_scene_rejection', 'perceptual_grounding_final_action_change_dependency_closure', 'perceptual_grounding_final_association_without_causal_lift_remains_hypothesis', 'perceptual_grounding_final_change_persistence_dependency_closure', 'perceptual_grounding_final_complete_dependency_closed_chain', 'perceptual_grounding_final_determinism_non_mutation_facade_equivalence', 'perceptual_grounding_final_hard_bounds_active_all_frontiers', 'perceptual_grounding_final_persistence_scene_dependency_closure', 'perceptual_grounding_final_rejected_upstream_cannot_be_resurrected_downstream', 'perceptual_grounding_final_scene_ambiguity_can_preserve_downstream_alternatives', 'perceptual_grounding_final_scene_grouping_controls_downstream_identity', 'perceptual_grounding_final_strict_forward_frame_order', 'perceptual_grounding_final_topology_scene_dependency_closure', 'perceptual_grounding_hard_competing_scene_frontier', 'perceptual_grounding_invalid_scene_filtering', 'perceptual_grounding_no_mandatory_single_objecthood_axis', 'perceptual_grounding_nonempty_unique_observation_local_elements', 'perceptual_grounding_nonvacuous_hard_scene_object_bounds', 'perceptual_grounding_object_hypothesis_requires_evidence', 'perceptual_grounding_overlapping_object_hypotheses_supported', 'perceptual_grounding_persistence_determinism_non_mutation_facade_equivalence', 'perceptual_grounding_persistence_duplicate_transition_best_variant', 'perceptual_grounding_persistence_forward_time_supported_link', 'perceptual_grounding_persistence_grounded_object_observation', 'perceptual_grounding_persistence_hard_global_frontier', 'perceptual_grounding_persistence_merge_split_ambiguity_without_forced_assignment', 'perceptual_grounding_persistence_multiple_predecessor_hypotheses', 'perceptual_grounding_persistence_multiple_successor_hypotheses', 'perceptual_grounding_persistence_no_mandatory_single_evidence_axis', 'perceptual_grounding_persistence_nonvacuous_hard_tracking_bounds', 'perceptual_grounding_persistence_signature_membership_change_allowed', 'perceptual_grounding_persistence_temporal_gap_supported', 'perceptual_grounding_support_then_simplicity_deterministic_ranking', 'perceptual_grounding_topology_determinism_non_mutation_facade_equivalence', 'perceptual_grounding_topology_directional_containment_preservation', 'perceptual_grounding_topology_duplicate_exact_relation_best_variant', 'perceptual_grounding_topology_hard_global_relation_bound', 'perceptual_grounding_topology_hard_per_pair_relation_bound', 'perceptual_grounding_topology_multiple_relation_kinds_same_pair', 'perceptual_grounding_topology_no_forced_single_relation_kind', 'perceptual_grounding_topology_overlapping_object_relation_support', 'perceptual_grounding_topology_reverse_symmetric_identity_canonicalization', 'perceptual_grounding_topology_same_observation_distinct_supported_relation', 'perceptual_grounding_topology_symmetric_directional_relation_capacity', 'perceptual_grounding_topology_symmetric_endpoint_canonicalization']

FORBIDDEN = [
    r"\bTODO\b",
    r"\btodo!\s*\(",
    r"\bunimplemented!\s*\(",
    r"\bFIXME\b",
    r"\bPLACEHOLDER\b",
    r"\.\.\.",
]

DOMAIN_FIREWALL = [
    r"\bpixel\b",
    r"\bcolor\b",
    r"\bcolour\b",
    r"\bgravity\b",
    r"\bwall\b",
    r"connected\s+component",
    r"\bARCObjectDetector\b",
]


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


def fail(message):
    print(
        "MODULE 46 CORE KNOWLEDGE + PERCEPTUAL GROUNDING VERIFY: FAIL"
    )

    print(
        message
    )

    sys.exit(1)


if run(
    [
        "git",
        "cat-file",
        "-e",
        SOURCE_COMMIT + "^{commit}",
    ]
).returncode != 0:
    fail(
        "Frozen M46 source commit is unavailable"
    )

if run(
    [
        "git",
        "merge-base",
        "--is-ancestor",
        SOURCE_COMMIT,
        "HEAD",
    ]
).returncode != 0:
    fail(
        "Frozen M46 source commit is not an ancestor of HEAD"
    )

for relative in FROZEN_FILES:
    path = ROOT / relative

    if not path.exists():
        fail(
            "Missing frozen M46 file: "
            + relative
        )

    if sha(path) != FROZEN_HASHES[
        relative
    ]:
        fail(
            "Frozen M46 file hash mismatch: "
            + relative
        )

    if path.suffix == ".rs":
        text = path.read_text(
            encoding="utf-8",
            errors="ignore",
        )

        for pattern in FORBIDDEN:
            if re.search(
                pattern,
                text,
                re.IGNORECASE,
            ):
                fail(
                    "Lazy-code pattern in frozen M46 file: "
                    + relative
                )

        for pattern in DOMAIN_FIREWALL:
            if re.search(
                pattern,
                text,
                re.IGNORECASE,
            ):
                fail(
                    "Domain-specific perception assumption in frozen M46 file: "
                    + relative
                )

state = json.loads(
    (
        ROOT
        / "state/project_state.json"
    ).read_text(
        encoding="utf-8"
    )
)

completed_layers = set(
    state[
        "rust_port"
    ][
        "completed_layers"
    ]
)

if FROZEN_STAGE not in completed_layers:
    fail(
        "Frozen M46 completion marker is missing"
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
    fail(
        "Missing required M46 invariants: "
        + ", ".join(
            missing
        )
    )

test_count = module_test_count()

if test_count != EXPECTED_TESTS:
    fail(
        "Expected "
        + str(
            EXPECTED_TESTS
        )
        + " M46 tests, got "
        + str(
            test_count
        )
    )

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
        fail(
            "Frozen M46 quality gate failed"
        )

print(
    "MODULE 46 CORE KNOWLEDGE + PERCEPTUAL GROUNDING VERIFY: PASS"
)

print(
    "Perceptual grounding integrity gate: "
    + str(
        EXPECTED_TESTS
    )
    + "/"
    + str(
        EXPECTED_TESTS
    )
)

print(
    "Frozen perceptual grounding files:",
    len(
        FROZEN_FILES
    ),
)

print(
    "Required invariants:",
    len(
        REQUIRED_INVARIANTS
    ),
)

print(
    "Perceptual grounding tests:",
    test_count,
)

print(
    "Frozen source commit:",
    SOURCE_COMMIT,
)
