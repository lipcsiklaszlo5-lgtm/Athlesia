#!/usr/bin/env python3
from pathlib import Path
import hashlib
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]

MANIFEST = "crates/athlesia_executive_agency/Cargo.toml"
CRATE_REL = "crates/athlesia_executive_agency"

SOURCE_COMMIT = 'abc4bf36d86bfa6490884d380d5b6fbe72ae4655'
FROZEN_STAGE = 'module_48_executive_agency_frozen'
NEXT_LAYER = 'module_49_meta_learning_skill_invention_compression_memory_foundation'

EXPECTED_TESTS = 108
FROZEN_FILES = ['crates/athlesia_executive_agency/Cargo.lock', 'crates/athlesia_executive_agency/Cargo.toml', 'crates/athlesia_executive_agency/src/lib.rs', 'crates/athlesia_executive_agency/tests/executive_agency_deviation_replanning.rs', 'crates/athlesia_executive_agency/tests/executive_agency_exploration_exploitation_control.rs', 'crates/athlesia_executive_agency/tests/executive_agency_foundation.rs', 'crates/athlesia_executive_agency/tests/executive_agency_goal_conflict_arbitration.rs', 'crates/athlesia_executive_agency/tests/executive_agency_goal_persistence.rs', 'crates/athlesia_executive_agency/tests/executive_agency_integrated_control_cycle.rs', 'crates/athlesia_executive_agency/tests/executive_agency_intention_execution_monitoring.rs', 'crates/athlesia_executive_agency/tests/executive_agency_multi_step_intention.rs', 'crates/athlesia_executive_agency/tests/executive_agency_stop_and_reconsideration.rs']
FROZEN_HASHES = {'crates/athlesia_executive_agency/Cargo.lock': 'da178d1345c1ad8d43f73b08ee997d5a093f9ae6f7bc713f6338c4e9e23207aa', 'crates/athlesia_executive_agency/Cargo.toml': '7fa2a02601b37038f11c2406b289c51af8338988d73fea784ae13a0dccd49cc1', 'crates/athlesia_executive_agency/src/lib.rs': '26fd7660e8349481c7da7fd8fbc550afbe42bbee7f1542f34b732bca21859ed1', 'crates/athlesia_executive_agency/tests/executive_agency_deviation_replanning.rs': '47d78aaa24946978fe4bfa4f4aeb419627b1656f576c5299bf7ee251121e884b', 'crates/athlesia_executive_agency/tests/executive_agency_exploration_exploitation_control.rs': '648d4e5b40d785dc81925ef14e263c191adbdb1609e64335411e08ec978429c7', 'crates/athlesia_executive_agency/tests/executive_agency_foundation.rs': '763fcb4d72809c9f2314b45f15e3593de68ea8f18388b2550a7ee5277d5d0920', 'crates/athlesia_executive_agency/tests/executive_agency_goal_conflict_arbitration.rs': '14582d55e38f6985849aaef41a4fffe0a1c15f5d2bd246b67fef193e131afa63', 'crates/athlesia_executive_agency/tests/executive_agency_goal_persistence.rs': '3ba2700464c9ab6a8a75ea9654ec97d51067e4ff5a35d5e3ffb879866a409a0f', 'crates/athlesia_executive_agency/tests/executive_agency_integrated_control_cycle.rs': '67a68b6ded4d5bab5b0a4bca3d66d48cc5ef77d9fe71cc955aea7436b1ab7c0d', 'crates/athlesia_executive_agency/tests/executive_agency_intention_execution_monitoring.rs': '5fb7c3f57e5529ed54dae707cace2c9974127e445697b7814cbe012f7c25ec00', 'crates/athlesia_executive_agency/tests/executive_agency_multi_step_intention.rs': 'd12e96689fb501314687e125a22a961c80a894acee2d80babe1c1c55e740c246', 'crates/athlesia_executive_agency/tests/executive_agency_stop_and_reconsideration.rs': 'dd7ecec8cfd9d9c85ad487bc2655a26cc7c0fad8b2de62bda0e32c028063081c'}
FROZEN_TESTS = ['absent_conflict_evidence_preserves_multiple_viable_intents', 'absent_execution_observation_leaves_intention_pending_at_first_step', 'action_candidate_must_match_goal_by_exact_opaque_identity', 'advanced_intention_executes_next_unconfirmed_step_not_first_step_again', 'challenger_switches_only_when_required_margin_is_reached', 'completed_intention_with_unsatisfied_goal_reconsiders_instead_of_claiming_success', 'confident_action_mismatch_creates_explicit_deviation', 'confident_deviation_selects_replacement_anchored_to_observed_outcome', 'confident_outcome_mismatch_creates_explicit_prediction_error', 'confident_state_mismatch_creates_explicit_deviation_without_advancing', 'conflict_matching_uses_exact_opaque_goal_identity', 'conflict_policy_requires_positive_bounds_thresholds_and_distinct_goal_pair', 'continuation_goal_matching_uses_exact_opaque_structure_identity', 'continuation_requires_exact_previous_predicted_outcome_as_next_required_state', 'continuation_stops_when_cost_reduces_expected_net_value_below_threshold', 'continuity_bonus_cannot_override_substantially_stronger_conflicting_challenger', 'continuity_bonus_preserves_incumbent_against_small_conflicting_advantage', 'decisive_learning_advantage_selects_exact_grounded_exploration_action', 'deviation_observation_confidence_discounts_replan_score', 'deviation_replanning_is_order_invariant_non_mutating_and_facade_equivalent', 'deviation_replanning_policy_requires_positive_bounds_and_thresholds', 'evidence_thresholds_reject_weak_action_without_forcing_execution', 'exact_confident_first_step_observation_advances_to_next_step', 'exact_opaque_execution_identity_rejects_reordered_structures', 'exact_opaque_goal_and_action_identity_are_not_lossily_collapsed', 'exact_ordered_observations_complete_entire_multi_step_intention', 'exact_symmetric_conflict_suppresses_weaker_intent', 'execution_bound_reconsideration_is_deterministic_non_mutating_and_facade_equivalent', 'execution_cost_can_reverse_action_preference', 'execution_cost_can_reverse_otherwise_equal_multi_step_preference', 'execution_monitoring_is_deterministic_non_mutating_and_facade_equivalent', 'execution_monitoring_policy_requires_positive_bounds_and_confidence', 'executive_agency_is_input_order_invariant_non_mutating_and_facade_equivalent', 'executive_policy_requires_positive_bounds_benefit_weights_and_thresholds', 'executive_stop_or_reconsideration_blocks_exploration_before_candidate_evaluation', 'exhausted_replanning_reconsiders_until_hard_cycle_limit_then_stops', 'exploration_below_advantage_margin_preserves_current_exploitation', 'exploration_execution_cost_can_reverse_information_gain_preference', 'exploration_exploitation_is_order_invariant_non_mutating_and_facade_equivalent', 'exploration_exploitation_policy_requires_positive_bounds_and_thresholds', 'exploration_wins_when_grounded_learning_value_exceeds_exploit_by_explicit_margin', 'first_confident_deviation_halts_monitoring_and_later_match_cannot_rescue_plan', 'first_step_must_bind_exactly_to_arbitrated_source_action_and_outcome', 'fully_satisfied_goal_produces_explicit_abstention', 'goal_conflict_arbitration_is_order_invariant_non_mutating_and_facade_equivalent', 'goal_persistence_is_deterministic_non_mutating_and_facade_equivalent', 'goal_persistence_policy_requires_positive_stall_margin_and_challenger_bounds', 'goal_pressure_combines_priority_with_remaining_unsatisfied_need', 'greater_goal_alignment_wins_when_other_evidence_is_equal', 'hard_candidate_and_evaluation_frontiers_are_enforced_deterministically', 'hard_candidate_evaluation_and_final_intent_frontiers_are_enforced', 'hard_candidate_evaluation_step_and_final_replan_frontiers_are_enforced', 'hard_challenger_frontier_uses_best_deterministic_challenger', 'hard_conflict_frontier_prefers_strongest_evidence_deterministically', 'hard_goal_frontier_prefers_highest_pressure_goal', 'hard_intent_pair_evaluation_and_final_selection_frontiers_are_enforced', 'hard_source_candidate_step_evaluation_and_final_frontiers_are_enforced', 'hard_step_and_observation_frontiers_are_enforced_without_hidden_progress', 'inconclusive_execution_evidence_reconsiders_before_any_continuation_assessment', 'inconclusive_execution_reconsiders_before_exploration_or_action_selection', 'information_gain_can_drive_exploration_when_policy_values_learning', 'information_gain_without_learning_progress_does_not_trigger_exploration', 'integrated_control_is_deterministic_non_mutating_and_facade_equivalent', 'integrated_policy_preserves_bounded_stop_and_exploration_subpolicies', 'integrated_selection_preserves_exact_opaque_action_identity', 'low_confidence_observation_is_retained_as_inconclusive_without_advancing', 'low_net_continuation_value_stops_before_exploration_can_override_stop_gate', 'missing_current_intention_reconsiders_without_manufacturing_action', 'multi_step_intention_is_order_invariant_non_mutating_and_facade_equivalent', 'multi_step_policy_requires_positive_bounds_thresholds_and_at_least_two_steps', 'no_prior_commitment_establishes_best_current_intent', 'non_conflicting_goal_survives_alongside_winner_of_conflicting_pair', 'observed_goal_progress_resets_stall_counter_and_preserves_incumbent', 'old_expected_suffix_cannot_continue_after_observed_outcome_deviation', 'overlong_candidate_is_rejected_before_step_evaluation', 'pending_viable_intention_executes_exact_first_step_without_exploration', 'recovery_anchor_uses_exact_opaque_structure_identity', 'reordered_opaque_action_identity_never_impersonates_existing_incumbent', 'reordered_opaque_action_structures_remain_distinct_and_deterministic', 'reordered_opaque_step_action_identity_does_not_preserve_source_binding', 'repeated_non_progress_reaches_stall_limit_and_forces_replanning', 'replacement_path_confidence_remains_threshold_gated', 'replanning_does_not_trigger_without_explicit_execution_deviation', 'replanning_preserves_exact_original_goal_identity', 'satisfied_goal_stops_before_exploration_is_considered', 'satisfied_goal_stops_immediately_even_when_continuation_looks_strong', 'satisfied_incumbent_goal_is_released_and_next_goal_can_take_control', 'small_challenger_advantage_does_not_break_existing_commitment', 'small_exploration_advantage_does_not_break_exploitation_hysteresis', 'stop_reconsideration_policy_requires_positive_bound_and_thresholds', 'stricter_replanning_confidence_can_refuse_an_otherwise_detected_deviation', 'strong_grounded_continuation_preserves_current_intention_and_resets_reconsideration', 'stronger_viable_replacement_outranks_weaker_recovery_plan', 'subthreshold_conflict_evidence_cannot_suppress_viable_intent', 'successful_deviation_replanning_continues_exact_selected_replacement', 'suppression_retains_exact_winner_loser_and_conflict_evidence', 'terminal_goal_alignment_remains_part_of_plan_confidence', 'unavailable_incumbent_action_triggers_switch_to_viable_alternative', 'unavailable_incumbent_without_alternative_releases_commitment_and_abstains', 'valid_ordered_multi_step_intention_is_admitted_with_exact_sequence', 'validated_deviation_replan_executes_replacement_from_recovery_state', 'viable_exploitation_continues_when_no_exploration_candidate_exists', 'weak_continuation_evidence_causes_reconsideration_without_blind_execution', 'weak_controllability_causes_reconsideration_even_with_high_expected_progress', 'weak_exploration_controllability_cannot_displace_exploitation', 'weak_exploration_evidence_cannot_displace_grounded_exploitation', 'weakest_step_controllability_is_explicit_and_threshold_gated', 'weakest_step_evidence_controls_path_confidence_and_can_reject_plan']


def run(cmd):
    print("$ " + " ".join(cmd))

    return subprocess.run(
        cmd,
        cwd=ROOT,
        check=False,
    )


def sha256_bytes(data):
    return hashlib.sha256(
        data
    ).hexdigest()


def sha256_file(path):
    return sha256_bytes(
        path.read_bytes()
    )


def tracked_files():
    result = subprocess.run(
        [
            "git",
            "ls-files",
            "-z",
            "--",
            CRATE_REL,
        ],
        cwd=ROOT,
        check=True,
        capture_output=True,
    )

    return sorted(
        item.decode(
            "utf-8",
            errors="strict",
        )
        for item in result.stdout.split(
            b"\0"
        )
        if item
    )


def test_names():
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
        return None

    names = []

    for line in result.stdout.splitlines():
        stripped = line.strip()

        if stripped.endswith(
            ": test"
        ):
            names.append(
                stripped[
                    :-len(
                        ": test"
                    )
                ]
            )

    return sorted(
        names
    )


def fail(message):
    print(
        "MODULE 48 EXECUTIVE AGENCY VERIFY: FAIL"
    )

    print(
        message
    )

    sys.exit(
        1
    )


# ------------------------------------------------------------
# Source commit must remain resolvable.
# ------------------------------------------------------------

source_exists = subprocess.run(
    [
        "git",
        "cat-file",
        "-e",
        SOURCE_COMMIT + "^{commit}",
    ],
    cwd=ROOT,
    check=False,
    stdout=subprocess.DEVNULL,
    stderr=subprocess.DEVNULL,
)

if source_exists.returncode != 0:
    fail(
        "Frozen source commit is not resolvable"
    )

# ------------------------------------------------------------
# Exact crate file frontier.
# ------------------------------------------------------------

current_files = tracked_files()

if current_files != FROZEN_FILES:
    fail(
        "Frozen Module 48 tracked-file frontier changed"
    )

# ------------------------------------------------------------
# Exact byte identity + source snapshot identity.
# ------------------------------------------------------------

for relative in FROZEN_FILES:
    path = ROOT / relative

    if not path.is_file():
        fail(
            "Missing frozen file: " + relative
        )

    expected_hash = FROZEN_HASHES[
        relative
    ]

    current_hash = sha256_file(
        path
    )

    if current_hash != expected_hash:
        fail(
            "Frozen file hash mismatch: " + relative
        )

    source_blob = subprocess.run(
        [
            "git",
            "show",
            SOURCE_COMMIT + ":" + relative,
        ],
        cwd=ROOT,
        check=False,
        capture_output=True,
    )

    if source_blob.returncode != 0:
        fail(
            "Frozen source snapshot missing file: " + relative
        )

    source_hash = sha256_bytes(
        source_blob.stdout
    )

    if source_hash != expected_hash:
        fail(
            "Frozen source commit does not match recorded file: "
            + relative
        )

# ------------------------------------------------------------
# Exact executable invariant/test identity.
# ------------------------------------------------------------

current_tests = test_names()

if current_tests is None:
    fail(
        "Unable to enumerate Module 48 tests"
    )

if len(
    current_tests
) != EXPECTED_TESTS:
    fail(
        "Module 48 test count mismatch"
    )

if current_tests != FROZEN_TESTS:
    fail(
        "Module 48 frozen test/invariant identity changed"
    )

# ------------------------------------------------------------
# Forward-compatible state contract.
# ------------------------------------------------------------

state_path = ROOT / "state/project_state.json"

if not state_path.is_file():
    fail(
        "Missing project state"
    )

state = json.loads(
    state_path.read_text(
        encoding="utf-8"
    )
)

rust_port = state.get(
    "rust_port",
    {},
)

completed = rust_port.get(
    "completed_layers",
    [],
)

if FROZEN_STAGE not in completed:
    fail(
        "Module 48 frozen stage missing from completed_layers"
    )

current_stage = rust_port.get(
    "stage"
)

if current_stage == FROZEN_STAGE:
    if rust_port.get(
        "status"
    ) != "validated":
        fail(
            "Module 48 frozen stage is not validated"
        )

    if rust_port.get(
        "next_layer"
    ) != NEXT_LAYER:
        fail(
            "Module 48 frozen next_layer mismatch"
        )

# ------------------------------------------------------------
# Quality gates.
# ------------------------------------------------------------

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
            "Module 48 frozen quality gate failed"
        )

print(
    "MODULE 48 EXECUTIVE AGENCY VERIFY: PASS"
)

print(
    "Executive agency integrity gate:",
    f"{EXPECTED_TESTS}/{EXPECTED_TESTS}",
)

print(
    "Frozen executive agency files:",
    len(
        FROZEN_FILES
    ),
)

print(
    "Required invariants:",
    len(
        FROZEN_TESTS
    ),
)

print(
    "Executive agency tests:",
    len(
        current_tests
    ),
)

print(
    "Frozen source commit:",
    SOURCE_COMMIT,
)
