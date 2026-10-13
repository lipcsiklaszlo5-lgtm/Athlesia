# Athlesia E6 Day6a — paired active versus uniform-random ablation

Recorded UTC: 2026-10-13. Reviewed baseline commit: `b859932853d338e29c871d2b760c1a3940ad5c3c`.
Source subset SHA-256: `c45aaa2613a8a09d6d7b6edbbc47a829a2e1c7f7425af9a9a0f85696df38558d`.
This is a **same-author, deterministic Boolean fixture**, **not independent**, **not** an ARC-AGI-3 evaluation, and **not proof of general intelligence**.

## Reproduce

From `experiments/e6_online_revision_transfer`, Python 3.11+:

```bash
python3 -m unittest discover -s tests -v
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day6a_ablation.py --cases 100 --budget 12 --output results/day6a_ablation100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5c_active.py --cases 100 --output results/day6a_day5c_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5b_effects.py --cases 100 --output results/day6a_day5b_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5a_epochs.py --cases 100 --output results/day6a_day5a_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day4b_goals.py --cases 100 --output results/day6a_day4b_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day4_stress.py --cases 100 --output results/day6a_day4_regression100.json
python3 -m athlesia.benchmark --cases 100 --output results/day6a_broker_regression100.json
sha256sum -c SHA256SUMS.txt
```

Generated JSON/JSONL stay gitignored. The report below records observed measurements rather than bundled benchmark output. Unit tests: **68**.

## Paired evaluation — same hidden rules, evidence, budget

In this fixed-seed 100-case comparison, active solved 100/100, random solved 82/100. Measured total real actions: active 1125, random 2113. This is a bounded within-fixture effect size and cannot establish general performance or independence.

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
  "wall_ms_total": 20194.1381,
  "evaluation_type": "paired in-process deterministic fixture; same author-developed World",
  "independent_generator": false,
  "independent_scorer": false,
  "official_arc_agi_3": false,
  "autonomous_episode_reset": false,
  "notes": "All failed/refused/wrong-commit cases remain in denominator; active superiority is NOT asserted."
}
```

All 100 cases stay in the denominator. Identical input seeds and starting
memories were supplied to active and random strategies, with at most 12 new
probes per arm. A failed/uncertified model is not counted as a success, and
model truth is evaluated against the private fixture only **after** execution.
`probe_actions_all_cases` is **observed target action count**;
`calibration_actions_all_cases` counts observed public state-setting actions;
`total_real_actions_all_cases` is their sum.
`capped_probe_mean_all_cases` is an **explicitly penalized score**: failed
cases are scored at `budget + 1` (13), even though they executed at most 12
probe actions. It is NOT the mean physical probe count. Measured mean probes
per case are 4.33 (active) and 7.84 (random).
These metrics do not include the initial contradiction action, pretraining
cost, or fresh-episode reset costs; they must not be presented as full
end-to-end task cost.

Winning a fixed-seed author-developed sample demonstrates a bounded
within-generator comparative advantage, not cross-distribution generalization.
The random policy uses only public state, predefined feature roles and legal
SET_TRUE/SET_FALSE actions, but it is only one baseline. The paired test lacks
an independent environment generator, independent scorer, sealed unseen
seeds, repeated random baselines, uncertainty/noise, and official ARC-AGI-3
measurement. The hidden guard remains a one-positive-conjunction fixture
with externally supplied resets.

## Regression results (all 100 cases each)

### Day5c active guard pilot

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
  "wall_ms_total": 9981.401994
}
```

### Day5b new single effect pilot

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
  "wall_ms_total": 6592.977409
}
```

### Day5a guard reidentification

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
  "wall_ms_total": 8161.130408
}
```

### Day4b declared-goal reuse

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
  "wall_ms_total": 4349.745061
}
```

### Day4 topology extension

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
  "total_cpu_ms": 3552.922467,
  "total_wall_ms": 3651.105095,
  "wall_ms_per_case_p50": 29.902959,
  "wall_ms_per_case_p95": 59.558382,
  "process_peak_rss_bytes_upper": 26345472,
  "limitations": "Does not test novel goals, new rule grammars, missing observation, subprocess isolation, or unseen non-fixture mechanisms."
}
```

### Day1–3 separate-process broker

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
  "solver_total_cpu_ms_p50": 111.433224,
  "solver_total_cpu_ms_p95": 180.330257,
  "case_wall_ms_p50_including_startup": 1105.43507,
  "case_wall_ms_p95_including_startup": 1230.301557,
  "max_solver_peak_rss_bytes": 26128384,
  "max_sum_process_peak_rss_bytes": 78385152,
  "isolation": "trusted_code_subprocess_protocol_only",
  "novelty_claim": false,
  "robotics_or_cross_mechanism_transfer_tested": false,
  "scope": "3..5 guard channels, three mixed opaque action schemas, fixed positive-conjunction grammar, relabelled isomorphic worlds"
}
```

## Next falsification gate

A separately designed generator and independently implemented scoring,
locked test seeds before solver changes, multiple policy baselines with
repeated randomness, and complete CPU, memory, reset, preparation, and probe
budgets. A strong single-family result is a research checkpoint, not a
verified general solution or an ARC-AGI-3 score.
