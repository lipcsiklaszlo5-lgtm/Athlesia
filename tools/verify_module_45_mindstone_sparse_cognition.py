#!/usr/bin/env python3

from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

CRATE_REL = 'crates/athlesia_mindstone_sparse_cognition'
MANIFEST = 'crates/athlesia_mindstone_sparse_cognition/Cargo.toml'

SOURCE_COMMIT = '1c7def4064289188bc931f95b2ebbe048904c870'
EXPECTED_TESTS = 216

EXPECTED_STAGE = 'module_45_mindstone_sparse_cognition_frozen'
EXPECTED_STATUS = "validated"
EXPECTED_NEXT_LAYER = 'module_46_core_knowledge_perceptual_grounding'

FROZEN_FILES = ['crates/athlesia_mindstone_sparse_cognition/Cargo.lock', 'crates/athlesia_mindstone_sparse_cognition/Cargo.toml', 'crates/athlesia_mindstone_sparse_cognition/src/lib.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_adaptive_compute_allocation.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_bounded_candidate_search.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_bounded_hypothesis_path_depth_search.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_causal_controllability_baseline.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_collision_safe_structural_identity.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_compression_controllability.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_epistemic_self_model.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_expected_information_gain.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_final_integration_freeze.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_forgetting_cold_storage.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_foundation.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_hierarchical_memory_admission.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_information_gain_goal_ranking.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_integrated_sparse_cycle.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_novelty_gate.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_self_generated_goals.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_streaming_aggregation.rs', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_structural_hashing.rs']

FROZEN_HASHES = {'crates/athlesia_mindstone_sparse_cognition/Cargo.lock': 'd67e0a5b9f7be917468be052c0febda5861b9c25ce92228c287ab0181d3fb201', 'crates/athlesia_mindstone_sparse_cognition/Cargo.toml': '5cf3f615255e7419fa2240e54b4f49e9a4cc45d5b142a545fb856a66fc43f401', 'crates/athlesia_mindstone_sparse_cognition/src/lib.rs': '027850bcdc6c1d65084efac618ce58f69c1c35da0376c81807e7b9c19e272a54', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_adaptive_compute_allocation.rs': 'ea05b7e478b1c9ddaa06a205df0dfb5151fa526eab4143cd9d0beda960bc0678', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_bounded_candidate_search.rs': '03ac4de9a403aec3772c9e75b79375f128da0cc36920afcb84e3baaa895668b0', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_bounded_hypothesis_path_depth_search.rs': '4eb8bffbb63903045f200c66b6422a97c92e6e07ba091792e598b929ca39ad87', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_causal_controllability_baseline.rs': '7bd4d3c4d994a6e2f6eecc1d520ee5ce1195156ac3b71a42cc2200a0286f1c58', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_collision_safe_structural_identity.rs': '415227f613f33dab64a0e351dd3a9938cc97948ad4ca02035e72b31feff1eaa6', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_compression_controllability.rs': '093f549fe24c1356eece54bb01539db02a9cee5866453a2ac0ba55546f94183d', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_epistemic_self_model.rs': '11b6b13bc109f105a0013a615575dd4fff42e233caa10a64741b34dae7fb63a3', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_expected_information_gain.rs': '3eb7653ba90acf3fe95e78f6604a72c9b468a8f15072e33ba289d8fedc54ca23', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_final_integration_freeze.rs': '776ca8946e3ab876738f263f285dbf0c362bfc9312e468f2ed78c264c770fa95', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_forgetting_cold_storage.rs': 'e5dd436a438d00aee879708e88e865c41014898f5064dbd3260ad6d991d2a7f4', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_foundation.rs': '7c6e0ed54b601d97954680f311e19f780717dbb0746d30ff385c0ebc2b07f10b', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_hierarchical_memory_admission.rs': '9384367520f10afcdc08538b9de41406543842a1470844436678379111ef9ba0', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_information_gain_goal_ranking.rs': '3b18827096d21513f6afdbd2d01b10cbacac7080bdf9d02944a268e4a8d922fe', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_integrated_sparse_cycle.rs': '40214589315e3c829d07a5cb92cc48d012b7114ed21406651771885031d6544e', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_novelty_gate.rs': 'f07e1d5924abc353bd68b2920b4447f3cc9f99907fbbb8ddaa793197f2a804dc', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_self_generated_goals.rs': '5f3553179d0993ce8997b0525f1f431abafc644f466c3c7d4afa5919253722d2', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_streaming_aggregation.rs': 'b6daaaccc07d86160f6741db49aae508b1df9df4d422e6f224bf21aa44ec0694', 'crates/athlesia_mindstone_sparse_cognition/tests/mindstone_sparse_cognition_structural_hashing.rs': '682beffce3e7be63e2bccd26a7388b9a36619952e111f3a97e983062a475be0c'}

REQUIRED_INVARIANTS = ['mindstone_adaptive_compute_determinism_non_mutation_facade_equivalence', 'mindstone_adaptive_compute_four_epistemic_pressure_axes', 'mindstone_adaptive_compute_full_goal_pressure_assignment', 'mindstone_adaptive_compute_full_hypothesis_pressure_assignment', 'mindstone_adaptive_compute_information_gain_goal_shift', 'mindstone_adaptive_compute_invalid_overreservation_rejection', 'mindstone_adaptive_compute_pressure_scaled_activation', 'mindstone_adaptive_compute_proportional_branch_split_conservation', 'mindstone_adaptive_compute_reserved_admission_hard_cap', 'mindstone_adaptive_compute_surprise_learning_search_shift', 'mindstone_adaptive_compute_u32_overflow_safety', 'mindstone_adaptive_compute_zero_pressure_idle_retention', 'mindstone_candidate_duplicate_identity_canonicalization', 'mindstone_candidate_hard_frontier_count_bound', 'mindstone_candidate_hard_total_compute_cost_bound', 'mindstone_candidate_large_pool_bounded_selected_frontier', 'mindstone_candidate_no_skip_unaffordable_best_remaining', 'mindstone_candidate_positive_support_cost_and_search_bound', 'mindstone_candidate_ranking_cost_tertiary', 'mindstone_candidate_ranking_fingerprint_deterministic_tiebreak', 'mindstone_candidate_ranking_salience_priority', 'mindstone_candidate_ranking_support_secondary', 'mindstone_candidate_search_determinism_non_mutation_facade_equivalence', 'mindstone_candidate_streaming_statistic_composability', 'mindstone_causal_controllability_adaptive_compute_spurious_control_suppression', 'mindstone_causal_controllability_equal_baseline_zero_lift', 'mindstone_causal_controllability_facade_determinism_composability', 'mindstone_causal_controllability_fixed_point_determinism', 'mindstone_causal_controllability_negative_lift_clamped_zero', 'mindstone_causal_controllability_positive_intervention_lift', 'mindstone_causal_controllability_profile_replacement_non_mutation', 'mindstone_causal_controllability_proportional_evidence_scale_invariance', 'mindstone_causal_controllability_signal_convenience_equivalence', 'mindstone_causal_controllability_two_valid_comparison_groups', 'mindstone_causal_controllability_u64_overflow_safety', 'mindstone_causal_controllability_zero_passive_recovers_intervention_rate', 'mindstone_cognitive_signal_fixed_point_bounds', 'mindstone_compression_controllability_determinism_non_mutation_composability', 'mindstone_compression_fractional_reduction_fixed_scale', 'mindstone_compression_gain_nonzero_source_requirement', 'mindstone_compression_non_reduction_zero_gain', 'mindstone_controllability_empirical_success_ratio', 'mindstone_controllability_valid_empirical_evidence_shape', 'mindstone_epistemic_self_compression_control_axis_independence', 'mindstone_epistemic_self_determinism_non_mutation_facade_equivalence', 'mindstone_epistemic_self_first_record_creation', 'mindstone_epistemic_self_hard_capacity_recency_eviction', 'mindstone_epistemic_self_latest_state_update_without_identity_growth', 'mindstone_epistemic_self_learning_progress_classification', 'mindstone_epistemic_self_meta_claims_require_support', 'mindstone_epistemic_self_monotonic_update_guard', 'mindstone_epistemic_self_nonvacuous_policy_and_state_bounds', 'mindstone_epistemic_self_stability_requires_low_uncertainty_and_support', 'mindstone_epistemic_self_structural_identity_composability', 'mindstone_epistemic_self_uncertain_classification', 'mindstone_extended_admission_hard_compute_budget', 'mindstone_extended_complete_meta_maximum_salience', 'mindstone_extended_compression_controllability_signal_separation', 'mindstone_extended_meta_salience_domain_neutrality', 'mindstone_extended_meta_signal_priority_augmentation', 'mindstone_extended_zero_meta_preserves_base_salience', 'mindstone_final_causal_evidence_baseline_corrects_control', 'mindstone_final_collision_safe_identity_end_to_end', 'mindstone_final_context_preserves_cycle_prediction_hypothesis_bounds', 'mindstone_final_determinism_non_mutation_hard_cap_facade_equivalence', 'mindstone_final_eig_ranked_goal_adaptive_budget', 'mindstone_final_hypothesis_conservative_unaffordable_stop', 'mindstone_final_hypothesis_search_adaptive_budget', 'mindstone_final_ignored_repeat_expensive_path_suppression', 'mindstone_final_missing_causal_evidence_suppresses_raw_control', 'mindstone_final_out_of_order_atomic_expensive_path_rejection', 'mindstone_final_spurious_equal_baseline_control_no_expensive_compute', 'mindstone_final_zero_hypothesis_pressure_search_skip', 'mindstone_forgetting_active_to_consolidated_cooling', 'mindstone_forgetting_consolidated_to_cold_cooling', 'mindstone_forgetting_determinism_non_mutation_facade_equivalence', 'mindstone_forgetting_explicit_cold_removal', 'mindstone_forgetting_full_lifecycle_active_to_forgotten', 'mindstone_forgetting_high_salience_temporal_protection', 'mindstone_forgetting_monotonic_maintenance_guard', 'mindstone_forgetting_positive_ordered_temporal_policy', 'mindstone_forgetting_repeated_use_temporal_protection', 'mindstone_forgetting_single_step_per_maintenance_cycle', 'mindstone_forgetting_target_tier_capacity_bound', 'mindstone_forgetting_young_memory_retention', 'mindstone_foundation_determinism_non_mutation_facade_equivalence', 'mindstone_goal_information_determinism_non_mutation_facade_equivalence', 'mindstone_goal_information_equal_gain_base_priority_fallback', 'mindstone_goal_information_equal_gain_priority_cost_fallback', 'mindstone_goal_information_gain_derived_from_self_uncertainty', 'mindstone_goal_information_gain_primary_ranking', 'mindstone_goal_information_hard_frontier_count_bound', 'mindstone_goal_information_irrelevant_prediction_no_effect', 'mindstone_goal_information_missing_prediction_zero_gain', 'mindstone_goal_information_no_skip_unaffordable_next_goal', 'mindstone_goal_information_prediction_nonempty_outcomes', 'mindstone_goal_information_prediction_unique_identity', 'mindstone_goal_information_worsening_prediction_zero_gain', 'mindstone_hypothesis_search_collision_safe_identity_separation', 'mindstone_hypothesis_search_conservative_compute_no_skip', 'mindstone_hypothesis_search_determinism_non_mutation_facade_equivalence', 'mindstone_hypothesis_search_exact_identity_best_variant_canonicalization', 'mindstone_hypothesis_search_hard_depth_path_filtering', 'mindstone_hypothesis_search_hard_frontier_retention_bound', 'mindstone_hypothesis_search_nonvacuous_frontier_path_policy', 'mindstone_hypothesis_search_root_depth_zero_support', 'mindstone_hypothesis_search_score_primary_ranking', 'mindstone_hypothesis_search_shallow_depth_tiebreak', 'mindstone_hypothesis_search_short_path_low_cost_tiebreak', 'mindstone_hypothesis_search_valid_node_shape', 'mindstone_information_gain_conservative_expected_uncertainty_rounding', 'mindstone_information_gain_determinism_non_mutation_facade_composability', 'mindstone_information_gain_deterministic_uncertainty_reduction', 'mindstone_information_gain_hard_compute_budget', 'mindstone_information_gain_nonempty_outcome_requirement', 'mindstone_information_gain_nonnegative_under_expected_worsening', 'mindstone_information_gain_positive_outcome_weight', 'mindstone_information_gain_profile_replacement_non_mutation', 'mindstone_information_gain_sparse_admission_priority', 'mindstone_information_gain_weight_scale_invariance', 'mindstone_information_gain_weighted_expected_uncertainty', 'mindstone_information_gain_zero_when_uncertainty_unchanged', 'mindstone_integrated_sparse_admission_reserves_compute_before_goals', 'mindstone_integrated_sparse_collision_separation_end_to_end', 'mindstone_integrated_sparse_conservative_goal_budget_no_skip', 'mindstone_integrated_sparse_context_api_determinism_non_mutation', 'mindstone_integrated_sparse_control_goal_precedence', 'mindstone_integrated_sparse_exact_prediction_no_collision_leakage', 'mindstone_integrated_sparse_exact_repeat_stream_and_self_update', 'mindstone_integrated_sparse_expected_information_gain_goal_ranking', 'mindstone_integrated_sparse_ignore_structural_only_update', 'mindstone_integrated_sparse_monotonic_event_atomic_rejection', 'mindstone_integrated_sparse_novelty_to_admission_to_exact_epistemic', 'mindstone_integrated_sparse_valid_bounded_state_prediction_shape', 'mindstone_known_event_preserves_independent_epistemic_pressure', 'mindstone_known_low_signal_event_suppression', 'mindstone_learning_progress_uncertainty_reduction_only', 'mindstone_memory_determinism_non_mutation_facade_equivalence', 'mindstone_memory_high_salience_active_admission', 'mindstone_memory_identity_demotion_without_duplication', 'mindstone_memory_identity_promotion_without_duplication', 'mindstone_memory_independent_tier_bounds_total_bound', 'mindstone_memory_low_salience_supported_cold_admission', 'mindstone_memory_monotonic_event_order_guard', 'mindstone_memory_policy_hierarchical_threshold_capacity_validation', 'mindstone_memory_refresh_recency_eviction_protection', 'mindstone_memory_supported_medium_salience_consolidation', 'mindstone_memory_tier_lru_hard_capacity', 'mindstone_memory_weak_singleton_discard', 'mindstone_novel_event_cheap_update_admission', 'mindstone_novelty_admission_hard_compute_budget', 'mindstone_novelty_distinct_fingerprint_detection', 'mindstone_novelty_evicted_identity_reenters_as_novel', 'mindstone_novelty_exact_duplicate_zero_signal_non_mutation', 'mindstone_novelty_fingerprint_exact_identity', 'mindstone_novelty_first_observation_maximum_signal', 'mindstone_novelty_gate_determinism_composability_non_mutation', 'mindstone_novelty_memory_fifo_hard_bound', 'mindstone_novelty_memory_positive_bounded_capacity', 'mindstone_peak_signal_survives_mean_dilution', 'mindstone_salience_domain_neutral_signal_aggregation', 'mindstone_self_goal_controllable_nonstable_test_generation', 'mindstone_self_goal_cost_and_identity_deterministic_tiebreak', 'mindstone_self_goal_determinism_non_mutation_facade_equivalence', 'mindstone_self_goal_hard_frontier_count_bound', 'mindstone_self_goal_learning_progress_continuation_generation', 'mindstone_self_goal_no_skip_unaffordable_next_goal', 'mindstone_self_goal_positive_priority_cost_frontier_policy', 'mindstone_self_goal_priority_signal_ranking', 'mindstone_self_goal_single_primary_goal_per_identity_per_cycle', 'mindstone_self_goal_stable_compressible_representation_generation', 'mindstone_self_goal_stable_noncompressible_quiescence', 'mindstone_self_goal_uncertainty_resolution_generation', 'mindstone_streaming_determinism_non_mutation_facade_equivalence', 'mindstone_streaming_distinct_capacity_retention', 'mindstone_streaming_evicted_identity_fresh_reentry', 'mindstone_streaming_first_observation_statistic_creation', 'mindstone_streaming_large_repeat_sufficient_statistic_compression', 'mindstone_streaming_lru_hard_bound', 'mindstone_streaming_monotonic_event_order_guard', 'mindstone_streaming_positive_capacity', 'mindstone_streaming_recent_update_eviction_protection', 'mindstone_streaming_repeat_constant_distinct_state', 'mindstone_streaming_structural_canonical_aggregation', 'mindstone_streaming_total_mean_peak_salience', 'mindstone_structural_compound_nonempty_invariant', 'mindstone_structural_equal_forms_equal_fingerprint', 'mindstone_structural_first_observation_novel_without_manual_fingerprint', 'mindstone_structural_hash_determinism_non_mutation', 'mindstone_structural_hash_kind_separation', 'mindstone_structural_hash_nesting_boundary_preservation', 'mindstone_structural_hash_order_invariant_unordered_form', 'mindstone_structural_hash_order_sensitive_ordered_form', 'mindstone_structural_hash_unordered_multiplicity_preservation', 'mindstone_structural_identity_canonical_unordered_equivalence', 'mindstone_structural_identity_collision_statistics_independence', 'mindstone_structural_identity_computed_fingerprint_novelty_path', 'mindstone_structural_identity_determinism_non_mutation_facade_equivalence', 'mindstone_structural_identity_exact_repeat_update_without_duplication', 'mindstone_structural_identity_exact_structure_authority', 'mindstone_structural_identity_forced_hash_collision_separation', 'mindstone_structural_identity_global_recency_capacity_eviction', 'mindstone_structural_identity_large_repeat_bounded_retention', 'mindstone_structural_identity_monotonic_event_guard', 'mindstone_structural_identity_positive_hard_capacity', 'mindstone_structural_identity_repeat_zero_novelty', 'mindstone_structural_novelty_facade_budget_composability', 'mindstone_structural_ordered_reordering_distinct_identity', 'mindstone_structural_unordered_reordering_known_identity', 'mindstone_zero_signal_zero_salience']

FORBIDDEN = [
    r"\bTODO\b",
    r"\btodo!\s*\(",
    r"\bunimplemented!\s*\(",
    r"\bFIXME\b",
    r"\bPLACEHOLDER\b",
    r"\.\.\.",
]


def run(cmd):
    print("$ " + " ".join(cmd))

    return subprocess.run(
        cmd,
        cwd=ROOT,
        check=False,
    )


def capture(cmd):
    return subprocess.run(
        cmd,
        cwd=ROOT,
        check=True,
        text=True,
        capture_output=True,
    ).stdout.strip()


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
        "MODULE 45 MINDSTONE + SPARSE COGNITION VERIFY: FAIL"
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
        "Frozen source commit is unavailable"
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
        "Frozen M45 source commit is not an ancestor of HEAD"
    )

for relative in FROZEN_FILES:
    path = ROOT / relative

    if not path.exists():
        fail(
            "Missing frozen file: " + relative
        )

    actual = sha(
        path
    )

    expected = FROZEN_HASHES[
        relative
    ]

    if actual != expected:
        fail(
            "Frozen file hash mismatch: "
            + relative
        )

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
                "Lazy-code pattern in frozen file: "
                + relative
            )

state_path = ROOT / "state/project_state.json"

state = json.loads(
    state_path.read_text(
        encoding="utf-8"
    )
)

rust_port = state[
    "rust_port"
]

if rust_port[
    "stage"
] != EXPECTED_STAGE:
    fail(
        "Unexpected frozen stage: "
        + str(
            rust_port[
                "stage"
            ]
        )
    )

if rust_port[
    "status"
] != EXPECTED_STATUS:
    fail(
        "Unexpected frozen status: "
        + str(
            rust_port[
                "status"
            ]
        )
    )

if rust_port[
    "next_layer"
] != EXPECTED_NEXT_LAYER:
    fail(
        "Unexpected next layer: "
        + str(
            rust_port[
                "next_layer"
            ]
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
    fail(
        "Missing required M45 invariants: "
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
        + " M45 tests, got "
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
            "Frozen M45 quality gate failed"
        )

print(
    "MODULE 45 MINDSTONE + SPARSE COGNITION VERIFY: PASS"
)

print(
    "Mindstone sparse cognition integrity gate: "
    + str(
        EXPECTED_TESTS
    )
    + "/"
    + str(
        EXPECTED_TESTS
    )
)

print(
    "Frozen mindstone sparse cognition files:",
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
    "Mindstone sparse cognition tests:",
    test_count,
)

print(
    "Frozen source commit:",
    SOURCE_COMMIT,
)
