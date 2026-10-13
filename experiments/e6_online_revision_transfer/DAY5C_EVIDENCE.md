# Athlesia E6 — Day5c active guarded-mechanism pilot checkpoint

Recorded UTC: 2026-10-13. Solver/pilot source subset SHA-256: `a3b06d8ed84bb869dc83f8cb2be9302bf6fbc8a8de40e299d9767b3b071d657f`.
The evidence concerns **author-developed deterministic Boolean fixtures**, not
official ARC-AGI-3, general intelligence or independent held-out generators.

## Reproduction

From `experiments/e6_online_revision_transfer`, Python 3.11+:

```bash
python3 -m unittest discover -s tests -v
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5c_active.py --cases 100 --output results/day5c_active100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5b_effects.py --cases 100 --output results/day5c_day5b_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5a_epochs.py --cases 100 --output results/day5c_day5a_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day4b_goals.py --cases 100 --output results/day5c_day4b_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day4_stress.py --cases 100 --output results/day5c_day4_regression100.json
python3 -m athlesia.benchmark --cases 100 --output results/day5c_broker_regression100.json
sha256sum -c SHA256SUMS.txt
```

Results JSON/JSONL stay gitignored. Tests collected: **64**.
A separate-process integration smoke additionally verified the new
`active_guard_probe` command over a live solver/environment protocol, counted
all calibration and target actions, and verified a fresh successor prediction.
That is a single fixture-derived smoke, not a 100-case independent active broker.

## Day5c active guard reidentification — 100 cases

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
  "wall_ms_total": 9778.672749
}
```

The four check categories each refer to **the same 100 cases**; they do not
mean 400 independent test environments. Of the reported actions,
`probe_actions_total` counts target experiments, and
`calibration_actions_total` counts advertised setting interventions. These
are distinct costs. The first quarantining transition and external episode
creation are additional work, not accounted for by those two counters.
`probe_actions_p50` and `probe_actions_max` describe target probe counts per
case, not all environment actions or end-to-end action cost.

A quarantined guard epoch is probed by minimizing worst-case surviving models
within a bounded Boolean assignment language. The solver chooses advertised
calibration actions and executes the target action in each provided episode.
It never reads the private hidden-law parameter and does not create or reset
fresh episodes itself. Commitment is delegated to Day5a evidence validation.

## Day5b changed-effect regression — 100 cases

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
  "wall_ms_total": 6735.462783
}
```

## Day5a guard-epoch regression — 100 cases

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
  "wall_ms_total": 8069.168919
}
```

## Day4b goal regression — 100 cases

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
  "wall_ms_total": 4367.549581
}
```

## Day4 topology regression — 100 cases

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
  "total_cpu_ms": 3445.514244,
  "total_wall_ms": 3491.47475,
  "wall_ms_per_case_p50": 35.859983,
  "wall_ms_per_case_p95": 54.951566,
  "process_peak_rss_bytes_upper": 26451968,
  "limitations": "Does not test novel goals, new rule grammars, missing observation, subprocess isolation, or unseen non-fixture mechanisms."
}
```

## Day1–3 separate-process regression — 100 cases

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
  "solver_total_cpu_ms_p50": 110.033488,
  "solver_total_cpu_ms_p95": 164.491539,
  "case_wall_ms_p50_including_startup": 1102.25498,
  "case_wall_ms_p95_including_startup": 1227.365538,
  "max_solver_peak_rss_bytes": 25858048,
  "max_sum_process_peak_rss_bytes": 77574144,
  "isolation": "trusted_code_subprocess_protocol_only",
  "novelty_claim": false,
  "robotics_or_cross_mechanism_transfer_tested": false,
  "scope": "3..5 guard channels, three mixed opaque action schemas, fixed positive-conjunction grammar, relabelled isomorphic worlds"
}
```

## Explicit limitations and falsification priorities

- An externally supplied **fresh episode** is needed for each new probe.
  The solver does not autonomously reset the world or decide whether resets
  are desirable. Calibrated SET_TRUE/SET_FALSE actions are pre-advertised.
- Experiments assume complete Boolean observations, static known fluents,
  and at most two positive/negative conjunctive guard literals after a drift.
  No newly invented observables, multi-step action planning, or causal graphs
  discovered from raw pixels are demonstrated.
- A bounded minimax split among existing hypotheses is not open-world active
  causal discovery. No independent test generator or sealed blind seeds exist.
- `active_guard_probe` currently targets the guard-drift path, not the Day5b
  changed-effect path. The live worker integration smoke uses a private
  author-developed fixture and a trusted broker; it is not a malicious-code
  sandbox verification or an ARC-AGI-3 score.
- Next: independent generators, locked negative tests, action/episode totals,
  reset costs, p50/p95/p99 on the *same* process isolation regime, and
  comparison to random and exhaustive interventions on equal budgets.
