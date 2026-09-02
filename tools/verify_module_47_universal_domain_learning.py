#!/usr/bin/env python3
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

MANIFEST = 'crates/athlesia_universal_domain_learning/Cargo.toml'
STATE = ROOT / "state/project_state.json"

FROZEN_SOURCE_COMMIT = '4ebb995c61bbf9267042acdcceb03bbaff5d4454'
FROZEN_LAYER = 'module_47_universal_domain_learning_frozen'

EXPECTED_TESTS = 156
EXPECTED_FROZEN_FILES = 16
EXPECTED_INVARIANTS = 156

FROZEN_FILES = {'crates/athlesia_universal_domain_learning/Cargo.lock': 'dd15fa520383ab416934233abdaa2337ae0eadfa1483437d7d8392646e2b9de6', 'crates/athlesia_universal_domain_learning/Cargo.toml': '608f66355321005946091cb7bce202f353496f0b61edc6e457f31728a7de540b', 'crates/athlesia_universal_domain_learning/src/lib.rs': '7727f89ff2e014ac4089aae85af3e0716f7345751e10b3099c708c10810ba176', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_causal_contrast_induction.rs': '0e6e6429fb4fce39e52669fab25df498d3a5a9204e5705aede2ae400ca7285c2', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_contextual_transition_rules.rs': '8c70371cb2e2c7482294f4dc33cb6fbc7f9451a36f24241950e11c5a2b8ab314', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_cross_context_generalization.rs': 'b3dc16eace122f65215d918912ec5f5e6c0558fdbb1551c4e5929323a095f6dd', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_cross_domain_transfer.rs': '4c1201bcc564c9ef7d64637b3b640eff9ba228e91fa583924e8f9c2348a721fd', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_domain_model_compression.rs': '9a5355a8ed9761ae34474e96ce7c1b590540872c3077e58ce2ad95daf75fcfd7', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_exception_refinement.rs': '8d59b0ace0f74c67fa6ebee3f0b26f893e180d74ca8cd491dac697eb1617f77f', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_grounded_predicate_discovery.rs': '373d49d4010c873cb98d7a830c0dd31d2fd49179beea3a0a7b3dc3c5f0c1b363', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_integrated_domain_model.rs': '6c3e35906ee99a121356f4401bb9ea4ff5d553f8ce6027af95a8270e698c6a77', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_interventional_causal_validation.rs': '1baacc59dfb51266bfaab7a0a55871e318088141c487d8fbbaaf4bbc6b19a145', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_invariant_discovery.rs': 'ca578477f286abd57567a00efc7679776181d85a9aa3fb2dde4421fde6ed913c', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_rule_confidence_calibration.rs': '810a274b8e077826ea1d194dd1fbed7457595cdd79bc1f5f7491db3f69245e28', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_rule_induction.rs': '82aa2f1e82ca04dfabaa1251e65f0b58cd37f00970409e8dcb01187e3f86869c', 'crates/athlesia_universal_domain_learning/tests/universal_domain_learning_transition_schema_induction.rs': 'fe625f232a9cf500b8ba9d753411e2ed5231ae7b49369c2c3b6b14a5b09581c8'}

REQUIRED_INVARIANTS = ['universal_learning_causal_contrast_common_effect_zero_lift_rejection', 'universal_learning_causal_contrast_deterministic_non_mutating_facade_equivalence', 'universal_learning_causal_contrast_distinct_matched_state_support', 'universal_learning_causal_contrast_exact_transformation_identity', 'universal_learning_causal_contrast_explicit_target_and_control_counterevidence', 'universal_learning_causal_contrast_hard_evaluation_and_final_frontiers', 'universal_learning_causal_contrast_hard_per_seed_frontier', 'universal_learning_causal_contrast_multiple_alternatives_remain_distinct', 'universal_learning_causal_contrast_positive_evidence_and_hard_bounds', 'universal_learning_causal_contrast_requires_exact_pre_state_matching', 'universal_learning_causal_contrast_self_control_rejection', 'universal_learning_causal_contrast_transformation_specific_effect_lift', 'universal_learning_compression_deterministic_non_mutating_facade_equivalence', 'universal_learning_compression_exact_duplicate_collapse', 'universal_learning_compression_exact_target_context_separation', 'universal_learning_compression_exact_target_transformation_separation', 'universal_learning_compression_hard_group_generation_frontier', 'universal_learning_compression_hard_input_frontier_strength_priority', 'universal_learning_compression_hard_output_frontier_redundancy_priority', 'universal_learning_compression_multi_source_provenance_without_confidence_inflation', 'universal_learning_compression_passive_evidence_lossless_separation', 'universal_learning_compression_positive_hard_bounds', 'universal_learning_compression_reordered_structure_exact_identity', 'universal_learning_compression_target_evidence_statistics_lossless_separation', 'universal_learning_confidence_deterministic_non_mutating_facade_equivalence', 'universal_learning_confidence_exact_transformation_identity', 'universal_learning_confidence_exception_abstention_preserves_raw_precision', 'universal_learning_confidence_exception_leakage_retained', 'universal_learning_confidence_full_support_matches_effective_precision', 'universal_learning_confidence_hard_exception_check_budget', 'universal_learning_confidence_hard_final_frontier_input_order_invariance', 'universal_learning_confidence_hard_seed_rule_budget', 'universal_learning_confidence_positive_support_and_hard_bounds', 'universal_learning_confidence_small_sample_support_discount', 'universal_learning_confidence_threshold_rejects_under_supported_precision', 'universal_learning_confidence_unrelated_exception_identity_isolation', 'universal_learning_context_conjunctive_synergy_discovery', 'universal_learning_context_deterministic_non_mutating_facade_equivalence', 'universal_learning_context_exact_structure_identity_semantic_authority', 'universal_learning_context_explains_ambiguous_transformation_effect', 'universal_learning_context_explicit_effect_counterexamples', 'universal_learning_context_hard_frontier_input_order_invariance', 'universal_learning_context_hard_maximum_context_arity', 'universal_learning_context_hard_rule_evaluation_budget', 'universal_learning_context_nonempty_bounded_canonical_premises', 'universal_learning_context_positive_evidence_hard_search_bounds', 'universal_learning_context_redundancy_rejected_against_transformation_baseline', 'universal_learning_context_schema_seed_priority_not_semantic_gate', 'universal_learning_cross_context_deterministic_non_mutating_facade_equivalence', 'universal_learning_cross_context_effect_target_identity_separation', 'universal_learning_cross_context_episode_validation_over_seed_authority', 'universal_learning_cross_context_exact_transformation_identity', 'universal_learning_cross_context_explicit_broader_context_counterexamples', 'universal_learning_cross_context_full_seed_not_reemitted', 'universal_learning_cross_context_hard_candidate_budget', 'universal_learning_cross_context_hard_seed_rule_budget', 'universal_learning_cross_context_positive_multi_seed_evidence_and_hard_bounds', 'universal_learning_cross_context_shared_proper_subset_discovery', 'universal_learning_cross_context_single_seed_cannot_generalize', 'universal_learning_cross_context_stronger_transfer_frontier_and_input_order_invariance', 'universal_learning_exception_base_context_not_reused_as_exception', 'universal_learning_exception_base_failure_lift_coverage_and_leakage_evidence', 'universal_learning_exception_conjunctive_refinement', 'universal_learning_exception_deterministic_non_mutating_facade_equivalence', 'universal_learning_exception_exact_transformation_identity', 'universal_learning_exception_failure_lift_over_base_guard', 'universal_learning_exception_hard_maximum_exception_arity', 'universal_learning_exception_hard_search_and_final_frontiers', 'universal_learning_exception_minimum_failure_support_guard', 'universal_learning_exception_positive_failure_evidence_and_hard_bounds', 'universal_learning_exception_repeated_failure_specific_fact_discovery', 'universal_learning_exception_requires_observed_base_counterexamples', 'universal_learning_integrated_model_deterministic_non_mutating_facade_equivalence', 'universal_learning_integrated_model_exact_query_identity', 'universal_learning_integrated_model_exact_reordered_structure_identity', 'universal_learning_integrated_model_hard_final_frontier_local_authority', 'universal_learning_integrated_model_hard_local_frontier_strength_priority', 'universal_learning_integrated_model_hard_transfer_frontier_strength_priority', 'universal_learning_integrated_model_local_authority_over_exact_transfer', 'universal_learning_integrated_model_local_evidence_lossless_normalization', 'universal_learning_integrated_model_positive_hard_bounds', 'universal_learning_integrated_model_target_domain_isolation', 'universal_learning_integrated_model_transfer_evidence_variants_retained', 'universal_learning_integrated_model_transfer_provenance_confidence_normalization', 'universal_learning_intervention_common_effect_zero_lift_rejection', 'universal_learning_intervention_deterministic_non_mutating_facade_equivalence', 'universal_learning_intervention_distinct_matched_state_support', 'universal_learning_intervention_exact_transformation_identity', 'universal_learning_intervention_explicit_failure_and_control_success_counterevidence', 'universal_learning_intervention_hard_seed_evaluation_and_final_frontiers', 'universal_learning_intervention_matched_assignment_effect_validation', 'universal_learning_intervention_passive_evidence_retained_without_score_inflation', 'universal_learning_intervention_passive_observation_cannot_validate', 'universal_learning_intervention_positive_evidence_support_and_hard_bounds', 'universal_learning_intervention_requires_controlled_assignment_on_both_sides', 'universal_learning_intervention_small_sample_support_discount', 'universal_learning_invariant_after_only_emergence_not_preservation', 'universal_learning_invariant_context_independent_preservation_discovery', 'universal_learning_invariant_deterministic_non_mutating_facade_equivalence', 'universal_learning_invariant_distinct_transformation_requirement', 'universal_learning_invariant_exact_structural_identity', 'universal_learning_invariant_explicit_disruption_counterexamples', 'universal_learning_invariant_grounded_snapshot_exact_canonicalization', 'universal_learning_invariant_hard_frontier_and_input_order_invariance', 'universal_learning_invariant_opaque_exact_transformation_identity', 'universal_learning_invariant_positive_evidence_and_hard_policy_bounds', 'universal_learning_invariant_rule_seed_priority_not_semantic_gate', 'universal_learning_invariant_transformation_stability_separate_from_episode_support', 'universal_learning_predicate_baseline_corrected_association_lift', 'universal_learning_predicate_deterministic_rank_order', 'universal_learning_predicate_episode_duplicate_no_support_inflation', 'universal_learning_predicate_episode_exact_fact_canonicalization', 'universal_learning_predicate_episode_requires_grounded_fact', 'universal_learning_predicate_exact_structural_identity', 'universal_learning_predicate_hard_frontier_bound', 'universal_learning_predicate_minimum_precision_filter', 'universal_learning_predicate_minimum_support_filter', 'universal_learning_predicate_multiple_outcomes_same_fact', 'universal_learning_predicate_non_mutating_facade_equivalence', 'universal_learning_predicate_repeated_association_discovery', 'universal_learning_rule_baseline_corrected_association_lift', 'universal_learning_rule_conjunctive_synergy_discovery', 'universal_learning_rule_deterministic_non_mutating_facade_equivalence', 'universal_learning_rule_explicit_falsifying_counterexamples', 'universal_learning_rule_hard_evaluation_budget', 'universal_learning_rule_hard_maximum_premise_arity', 'universal_learning_rule_incremental_gain_rank_input_order_invariance', 'universal_learning_rule_policy_bounded_arity_and_positive_evidence', 'universal_learning_rule_premise_set_exact_canonical_identity', 'universal_learning_rule_redundant_conjunction_rejection', 'universal_learning_rule_seed_prioritized_bounded_candidate_generation', 'universal_learning_rule_three_way_incremental_discovery', 'universal_learning_transfer_complete_mapping_required', 'universal_learning_transfer_deterministic_non_mutating_facade_equivalence', 'universal_learning_transfer_distinct_domains_nonempty_bijective_mapping', 'universal_learning_transfer_exact_canonical_correspondence_identity', 'universal_learning_transfer_exact_source_target_structure_identity', 'universal_learning_transfer_explicit_target_counterevidence', 'universal_learning_transfer_hard_source_evaluation_and_final_frontiers', 'universal_learning_transfer_passive_evidence_retained_without_score_inflation', 'universal_learning_transfer_passive_target_evidence_cannot_validate', 'universal_learning_transfer_small_target_sample_support_discount', 'universal_learning_transfer_target_common_effect_zero_lift_rejection', 'universal_learning_transfer_target_interventional_revalidation', 'universal_learning_transition_deterministic_non_mutating_facade_equivalence', 'universal_learning_transition_effect_add_remove_preservation_distinction', 'universal_learning_transition_exact_opaque_transformation_identity', 'universal_learning_transition_explicit_effect_counterexamples', 'universal_learning_transition_global_baseline_lift_guard', 'universal_learning_transition_hard_frontier_input_order_invariance', 'universal_learning_transition_invariant_deprioritization_not_semantic_gate', 'universal_learning_transition_multiple_effects_per_transformation', 'universal_learning_transition_preservation_not_effect', 'universal_learning_transition_schema_positive_evidence_and_hard_bounds', 'universal_learning_transition_specific_addition_discovery', 'universal_learning_transition_specific_removal_discovery']

FORBIDDEN = ['\\bTODO\\b', '\\btodo!\\s*\\(', '\\bunimplemented!\\s*\\(', '\\bFIXME\\b', '\\bPLACEHOLDER\\b', '\\.\\.\\.']

DOMAIN_FIREWALL = ['\\bpixel\\b', '\\bcolor\\b', '\\bcolour\\b', '\\bgravity\\b', '\\bwall\\b', 'connected\\s+component', '\\bARCObjectDetector\\b']


def sha256(path):
    return hashlib.sha256(
        path.read_bytes()
    ).hexdigest()


def run(cmd):
    print("$ " + " ".join(cmd))

    return subprocess.run(
        cmd,
        cwd=ROOT,
        check=False,
    )


def test_count():
    result = subprocess.run(
        [
            "cargo",
            "test",
            "--manifest-path",
            MANIFEST,
            "--",
            "--list",
        ],
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )

    if result.returncode != 0:
        return -1

    return sum(
        1
        for line in result.stdout.splitlines()
        if line.rstrip().endswith(
            ": test"
        )
    )


def fail(message):
    print(
        "MODULE 47 UNIVERSAL DOMAIN LEARNING VERIFY: FAIL"
    )

    print(
        message
    )

    sys.exit(1)


# ============================================================
# VERIFIER SELF-CHECK
# ============================================================

for pattern in (
    FORBIDDEN
    + DOMAIN_FIREWALL
):
    try:
        re.compile(
            pattern
        )
    except re.error as exc:
        fail(
            "Invalid frozen verifier regex "
            + repr(
                pattern
            )
            + ": "
            + str(
                exc
            )
        )


if len(
    FROZEN_FILES
) != EXPECTED_FROZEN_FILES:
    fail(
        "Frozen file manifest count changed"
    )


if len(
    REQUIRED_INVARIANTS
) != EXPECTED_INVARIANTS:
    fail(
        "Frozen invariant manifest count changed"
    )


# ============================================================
# SOURCE COMMIT ANCESTRY
# ============================================================

ancestor = subprocess.run(
    [
        "git",
        "merge-base",
        "--is-ancestor",
        FROZEN_SOURCE_COMMIT,
        "HEAD",
    ],
    cwd=ROOT,
    check=False,
)

if ancestor.returncode != 0:
    fail(
        "Frozen Module 47 source commit "
        "is not an ancestor of HEAD"
    )


# ============================================================
# BYTE-IDENTICAL FROZEN FILES
# ============================================================

for (
    relative,
    expected_hash,
) in FROZEN_FILES.items():
    path = ROOT / relative

    if not path.is_file():
        fail(
            "Missing frozen Module 47 file: "
            + relative
        )

    actual_hash = sha256(
        path
    )

    if actual_hash != expected_hash:
        fail(
            "Hash mismatch for frozen Module 47 file: "
            + relative
        )


# ============================================================
# FORWARD-COMPATIBLE STATE CONTRACT
#
# Current stage may advance to M48/M49/etc.
# M47 freeze marker + invariants + hashes must remain.
# ============================================================

state = json.loads(
    STATE.read_text(
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

if FROZEN_LAYER not in completed_layers:
    fail(
        "Module 47 frozen layer marker "
        "missing from completed_layers"
    )


validated_invariants = set(
    state[
        "validated_invariants"
    ]
)

missing_invariants = [
    invariant
    for invariant in REQUIRED_INVARIANTS
    if invariant not in validated_invariants
]

if missing_invariants:
    fail(
        "Missing frozen Module 47 invariants: "
        + repr(
            missing_invariants
        )
    )


implementation_hashes = state[
    "implementation_sha256"
]

for (
    relative,
    expected_hash,
) in FROZEN_FILES.items():
    state_hash = implementation_hashes.get(
        relative
    )

    if state_hash != expected_hash:
        fail(
            "State implementation hash mismatch "
            "for frozen Module 47 file: "
            + relative
        )


# ============================================================
# ARCHITECTURAL FIREWALL REGRESSION
# ============================================================

rust_text = "\n".join(
    (
        ROOT / relative
    ).read_text(
        encoding="utf-8"
    )
    for relative in FROZEN_FILES
    if relative.endswith(
        ".rs"
    )
)

for pattern in FORBIDDEN:
    if re.search(
        pattern,
        rust_text,
        re.IGNORECASE,
    ):
        fail(
            "Lazy-code firewall regression: "
            + pattern
        )


for pattern in DOMAIN_FIREWALL:
    if re.search(
        pattern,
        rust_text,
        re.IGNORECASE,
    ):
        fail(
            "Domain firewall regression: "
            + pattern
        )


# ============================================================
# FROZEN QUALITY GATES
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
        fail(
            "Frozen Module 47 quality command failed"
        )


actual_tests = test_count()

if actual_tests != EXPECTED_TESTS:
    fail(
        "Frozen Module 47 test count changed: expected "
        + str(
            EXPECTED_TESTS
        )
        + ", got "
        + str(
            actual_tests
        )
    )


print(
    "MODULE 47 UNIVERSAL DOMAIN LEARNING VERIFY: PASS"
)

print(
    "Universal domain learning integrity gate:",
    str(
        EXPECTED_TESTS
    )
    + "/"
    + str(
        EXPECTED_TESTS
    ),
)

print(
    "Frozen universal domain learning files:",
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
    "Universal domain learning tests:",
    actual_tests,
)

print(
    "Frozen source commit:",
    FROZEN_SOURCE_COMMIT,
)
