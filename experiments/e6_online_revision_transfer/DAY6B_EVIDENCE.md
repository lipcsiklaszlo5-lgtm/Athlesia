# Athlesia E6 — Day6b separate implementation paired ablation

Recorded UTC: 2026-10-13. Baseline commit: `dd118caa9e7f3c8652b053ddc2163defbe488667`.
Source subset SHA-256: `1d58b5ee7fbc7456d03cd7814b6ba9dad4a0ec86999987b49333890ba677116f`. Frozen schedule: `88f5c8db7b2374f0814a08809e23389291314d8a5a94051cb3a140d530a3c753`.

**Scope:** A second *same-author* Boolean evaluator implementation, not importing
`athlesia.environment.World`, uses a shared public symbolic ontology with
3–5 Boolean channels, 0–2 conjunctive literals in the learned guard grammar,
and four shifted-law families including negative literals. This is **not**
an independently authored or externally audited test, novel perception
benchmark, autonomous reset or official ARC-AGI-3 measurement.

## Reproduction

Run from `experiments/e6_online_revision_transfer` on Python 3.11+:

```bash
python3 -m unittest discover -s tests -v
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day6b_external.py --cases 100 --budget 12 --output results/day6b_external100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day6a_ablation.py --cases 100 --budget 12 --output results/day6b_day6a_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5c_active.py --cases 100 --output results/day6b_day5c_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5b_effects.py --cases 100 --output results/day6b_day5b_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5a_epochs.py --cases 100 --output results/day6b_day5a_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day4b_goals.py --cases 100 --output results/day6b_day4b_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day4_stress.py --cases 100 --output results/day6b_day4_regression100.json
python3 -m athlesia.benchmark --cases 100 --output results/day6b_broker_regression100.json
sha256sum -c SHA256SUMS.txt
```

JSON/JSONL results are deliberately not tracked; data below are measured on
the checkpoint machine. Unit test count: **75**.

## Day6b results — 100 paired cases (25 in each guard family)

Day6b separately implemented Boolean engine: 100/100 active vs 81/100 uniform random with 12 probes per arm. Actual calibration+target actions: 1243 active and 2527 random. This does not demonstrate cross-ontology transfer or third-party independence.

```json
{
  "benchmark": "Athlesia E6 Day6b separately implemented Boolean evaluator v0.1",
  "cases": 100,
  "budget_probes_per_arm": 12,
  "frozen_schedule_sha256": "88f5c8db7b2374f0814a08809e23389291314d8a5a94051cb3a140d530a3c753",
  "active": {
    "successes": 100,
    "failures": 0,
    "probe_p50_success_only": 5,
    "probe_p95_success_only": 6,
    "probes_executed_all": 480,
    "calibrations_executed_all": 763,
    "real_actions_executed_all": 1243,
    "failure_penalized_probe_mean": 4.8,
    "failure_reasons": {}
  },
  "uniform_random": {
    "successes": 81,
    "failures": 19,
    "probe_p50_success_only": 7,
    "probe_p95_success_only": 12,
    "probes_executed_all": 831,
    "calibrations_executed_all": 1696,
    "real_actions_executed_all": 2527,
    "failure_penalized_probe_mean": 8.5,
    "failure_reasons": {
      "budget_exhausted": 19
    }
  },
  "pair_counts": {
    "both_success": 81,
    "active_only": 19,
    "random_only": 0,
    "both_failed": 0
  },
  "paired_success_probe_comparison": {
    "active": 63,
    "tie": 11,
    "random": 7
  },
  "paired_success_total_action_comparison": {
    "active": 74,
    "tie": 2,
    "random": 5
  },
  "by_guard_family": {
    "0": {
      "law_shape": "single_positive",
      "cases": 25,
      "active_successes": 25,
      "random_successes": 21
    },
    "1": {
      "law_shape": "single_negative",
      "cases": 25,
      "active_successes": 25,
      "random_successes": 19
    },
    "2": {
      "law_shape": "two_mixed_positive_negative",
      "cases": 25,
      "active_successes": 25,
      "random_successes": 24
    },
    "3": {
      "law_shape": "two_mixed_negative_positive",
      "cases": 25,
      "active_successes": 25,
      "random_successes": 17
    }
  },
  "sample_cases": [
    {
      "family": 0,
      "channels": 3,
      "active": {
        "status": "committed",
        "correct": true,
        "probes": 4,
        "calibration_actions": 6,
        "real_actions": 10,
        "error": null
      },
      "random": {
        "status": "committed",
        "correct": true,
        "probes": 7,
        "calibration_actions": 10,
        "real_actions": 17,
        "error": null
      }
    },
    {
      "family": 1,
      "channels": 3,
      "active": {
        "status": "committed",
        "correct": true,
        "probes": 4,
        "calibration_actions": 3,
        "real_actions": 7,
        "error": null
      },
      "random": {
        "status": "budget_exhausted",
        "correct": false,
        "probes": 12,
        "calibration_actions": 17,
        "real_actions": 29,
        "error": null
      }
    },
    {
      "family": 2,
      "channels": 3,
      "active": {
        "status": "committed",
        "correct": true,
        "probes": 4,
        "calibration_actions": 5,
        "real_actions": 9,
        "error": null
      },
      "random": {
        "status": "committed",
        "correct": true,
        "probes": 10,
        "calibration_actions": 17,
        "real_actions": 27,
        "error": null
      }
    }
  ],
  "wall_ms_total": 34167.434,
  "environment_imports_original_private_fixture": false,
  "environment_implementation_separate": true,
  "independent_authorship_or_external_audit": false,
  "public_ontology_shared_with_training": true,
  "hidden_laws_new_polarities_and_conjunctions": true,
  "official_arc_agi_3": false,
  "autonomous_episode_reset": false,
  "note": "All failures and incorrect commits remain in the denominator; no success threshold enforced."
}
```

**Accounting:** Both arms receive identical initial episodes/observations and
the same 12-probe budget. The scoring function checks the actual identified
full guard with the private evaluator after execution; a reported commit by
itself does not count. Refusals, wrong commits and budget exhaustion remain
in the denominator. `probes_executed_all` counts actual target actions,
`calibrations_executed_all` counts real advertised feature-setting actions,
and `real_actions_executed_all` is their sum. `failure_penalized_probe_mean`
is a distinct reporting score: it assigns 13 to unsuccessful cases, **not**
13 physical actions. The accounting omits demo acquisition, initial anomaly,
fresh-episode reset and external orchestration costs.

The environment is independently *implemented* relative to the original
`World` module, but **not independently authored**; the Boolean variables,
relation/action ontology and calibration affordances are still shared.
There is no evidence of generality to ARC-AGI-3 pixels, partial observations,
continuous actions, delayed effects, noisy laws or autonomous episode resets.
Both policies only compete against one frozen uniform-random baseline.

## Day6a regression — 100 paired cases

```json
{
  "benchmark": "Athlesia E6 Day6a paired active-vs-random ablation v0.1",
  "cases": 100,
  "probe_budget_per_arm": 12,
  "fixed_seed_schedule_sha256": "b3c0aea849e353be594de6a5ef4e2ff468e6409b479094b20d39f6a82759dca5",
  "active": {
    "successes": 100,
    "failures": 0,
    "probe_p50_success_only": 4,
    "probe_p95_success_only": 5,
    "actions_p50_success_only": 10,
    "actions_p95_success_only": 14,
    "capped_probe_mean_all_cases": 4.33,
    "probe_actions_all_cases": 433,
    "calibration_actions_all_cases": 692,
    "total_real_actions_all_cases": 1125,
    "failure_status_counts": {}
  },
  "uniform_random": {
    "successes": 82,
    "failures": 18,
    "probe_p50_success_only": 7,
    "probe_p95_success_only": 12,
    "actions_p50_success_only": 18,
    "actions_p95_success_only": 33,
    "capped_probe_mean_all_cases": 8.02,
    "probe_actions_all_cases": 784,
    "calibration_actions_all_cases": 1329,
    "total_real_actions_all_cases": 2113,
    "failure_status_counts": {
      "budget_exhausted": 18
    }
  },
  "paired_comparison": {
    "both_success": 82,
    "active_only_success": 18,
    "random_only_success": 0,
    "both_failed": 0,
    "paired_success_probe_wins_active": 56,
    "paired_success_probe_ties": 11,
    "paired_success_probe_wins_random": 15,
    "paired_success_total_action_wins_active": 63,
    "paired_success_total_action_ties": 3,
    "paired_success_total_action_wins_random": 16
  },
  "sample_cases": [
    {
      "index": 0,
      "channels": 3,
      "active": {
        "status": "committed",
        "correct": true,
        "probes": 4,
        "calibration_actions": 6,
        "total_real_actions": 10,
        "error": null
      },
      "random": {
        "status": "committed",
        "correct": true,
        "probes": 6,
        "calibration_actions": 10,
        "total_real_actions": 16,
        "error": null
      }
    },
    {
      "index": 1,
      "channels": 4,
      "active": {
        "status": "committed",
        "correct": true,
        "probes": 4,
        "calibration_actions": 8,
        "total_real_actions": 12,
        "error": null
      },
      "random": {
        "status": "committed",
        "correct": true,
        "probes": 3,
        "calibration_actions": 7,
        "total_real_actions": 10,
        "error": null
      }
    },
    {
      "index": 2,
      "channels": 3,
      "active": {
        "status": "committed",
        "correct": true,
        "probes": 4,
        "calibration_actions": 5,
        "total_real_actions": 9,
        "error": null
      },
      "random": {
        "status": "committed",
        "correct": true,
        "probes": 11,
        "calibration_actions": 16,
        "total_real_actions": 27,
        "error": null
      }
    }
  ],
  "wall_ms_total": 20335.0052,
  "evaluation_type": "paired in-process deterministic fixture; same author-developed World",
  "independent_generator": false,
  "independent_scorer": false,
  "official_arc_agi_3": false,
  "autonomous_episode_reset": false,
  "notes": "All failed/refused/wrong-commit cases remain in denominator; active superiority is NOT asserted."
}
```

## Day5c active experiment selection — 100 cases

```json
{
  "benchmark": "Athlesia E6 Day5c active bounded guard probe v0.1",
  "cases": 100,
  "checks": {
    "quarantine": 100,
    "active_reidentified": 100,
    "archive_preserved": 100,
    "unrelated_actions_preserved": 100
  },
  "probe_actions_total": 433,
  "probe_actions_p50": 4,
  "probe_actions_max": 5,
  "calibration_actions_total": 717,
  "fixture": "author-developed, Boolean guard shift; externally fresh episodes",
  "independent_generator": false,
  "autonomous_environment_reset": false,
  "official_arc_agi_3": false,
  "wall_ms_total": 9804.276954
}
```

## Day5b changed Boolean effects — 100 cases

```json
{
  "benchmark": "Athlesia E6 Day5b observed-single-effect drift pilot v0.1",
  "cases": 100,
  "checks": {
    "quarantined_first": 100,
    "effect_relearned": 100,
    "archive_preserved": 100,
    "unrelated_action_preserved": 100
  },
  "official_arc_agi_3": false,
  "scope": "existing identified unguarded action, one changed Boolean effect",
  "independent_generator": false,
  "autonomous_interventions": false,
  "new_features_discovered": false,
  "wall_ms_total": 6664.679939
}
```

## Day5a mechanism shift — 100 cases

```json
{
  "benchmark": "Athlesia E6 Day5a mechanism epoch pilot v0.1",
  "cases": 100,
  "checks": {
    "old_epoch_identified": 100,
    "single_trace_quarantined": 100,
    "new_guard_reidentified": 100,
    "old_epoch_archived": 100
  },
  "test_family": "same author-developed Boolean generator, unchanged effect signature",
  "drift_identification": "requires independent episode observations; assumes stationary new law",
  "autonomous_intervention_selection": false,
  "new_effect_signature_learning": false,
  "noisy_mechanism_distinction": false,
  "official_arc_agi_3": false,
  "wall_ms_total": 8141.777565
}
```

## Day4b externally declared goal — 100 cases

```json
{
  "benchmark": "Athlesia E6 Day4b declared-goal learned-action pilot v0.1",
  "cases": 100,
  "checks": {
    "goal_up": 25,
    "goal_down": 25,
    "expanded_goal": 25,
    "uncertain_refused": 25
  },
  "test_family": "author-developed in-process Boolean fixture",
  "official_arc_agi_3": false,
  "autonomous_goal_invention": false,
  "mechanism_relearning": false,
  "wall_ms_total": 4384.334534
}
```

## Day4 topology stress — 100 cases

```json
{
  "benchmark": "Athlesia E6 Day 4 topology-extension fixture stress v0.1",
  "test_family": "instrumented Boolean fixture; 3..5 channels, 1..2 positive guards",
  "evaluator": "in-process deterministic fixture, not independently developed",
  "not_official_arc_agi_3": true,
  "solver_source_sha256_subset": "2c6dcb92716e74b10502407504f51716a9c630d6b1d99b96bd2f3ed6872a782a",
  "cases": 100,
  "checks": {
    "expanded_transfer": 100,
    "changed_mechanism_rejected": 100,
    "unexpected_effect_rejected": 100,
    "ambiguous_embedding_rejected": 100
  },
  "sample_cases": [
    {
      "case": 0,
      "channels": 3,
      "guards": [
        0,
        2
      ],
      "template_nodes": 17,
      "expanded_nodes": 21,
      "embedding_search_states": 18,
      "preparation_actions": 2
    },
    {
      "case": 1,
      "channels": 4,
      "guards": [
        1
      ],
      "template_nodes": 21,
      "expanded_nodes": 25,
      "embedding_search_states": 22,
      "preparation_actions": 1
    },
    {
      "case": 2,
      "channels": 5,
      "guards": [
        2
      ],
      "template_nodes": 25,
      "expanded_nodes": 29,
      "embedding_search_states": 26,
      "preparation_actions": 1
    },
    {
      "case": 3,
      "channels": 3,
      "guards": [
        0
      ],
      "template_nodes": 17,
      "expanded_nodes": 21,
      "embedding_search_states": 18,
      "preparation_actions": 1
    },
    {
      "case": 4,
      "channels": 4,
      "guards": [
        0,
        3
      ],
      "template_nodes": 21,
      "expanded_nodes": 25,
      "embedding_search_states": 22,
      "preparation_actions": 2
    }
  ],
  "total_cpu_ms": 3427.623757,
  "total_wall_ms": 3497.559522,
  "wall_ms_per_case_p50": 28.72328,
  "wall_ms_per_case_p95": 60.860572,
  "process_peak_rss_bytes_upper": 26472448,
  "limitations": "Does not test novel goals, new rule grammars, missing observation, subprocess isolation, or unseen non-fixture mechanisms."
}
```

## Day1–3 separate-process broker — 100 cases

```json
{
  "benchmark": "Athlesia bounded Day-3 online validation v0.3",
  "source_sha256": "e39aaf5057e096f603b79182160d4f2d4f313d3081cc125c271ed198eb85f3b2",
  "python": "3.14.2",
  "platform": "Linux-6.8.0-1064-azure-x86_64-with-glibc2.39",
  "processor": "x86_64",
  "one_cpu_affinity": true,
  "address_space_limit_enforced": true,
  "address_space_limit_bytes": 67108864,
  "cases": 100,
  "successes": 100,
  "failures": 0,
  "held_out_action_schema_predictions": 400,
  "exact_held_out_action_schema_predictions": 400,
  "online_transition_validations": 400,
  "online_transition_validation_attempts": 400,
  "solver_total_cpu_ms_p50": 112.1032,
  "solver_total_cpu_ms_p95": 167.580359,
  "case_wall_ms_p50_including_startup": 1096.042264,
  "case_wall_ms_p95_including_startup": 1212.544924,
  "max_solver_peak_rss_bytes": 26128384,
  "max_sum_process_peak_rss_bytes": 78385152,
  "isolation": "trusted_code_subprocess_protocol_only",
  "novelty_claim": false,
  "robotics_or_cross_mechanism_transfer_tested": false,
  "scope": "3..5 guard channels, three mixed opaque action schemas, fixed positive-conjunction grammar, relabelled isomorphic worlds"
}
```

## Next falsification targets

Seal unknown test cases before development, involve an evaluator written by
an independent author, change the public ontology/affordances, introduce
partial/noisy/delayed observations, and evaluate CPU and all action/reset
costs. Do not generalize beyond this controlled Boolean research checkpoint.
