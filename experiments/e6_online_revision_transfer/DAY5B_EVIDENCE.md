# Athlesia E6 — Day5b single-Boolean-effect epoch checkpoint

Recorded UTC: 2026-10-13. Source subset SHA-256: `6e92d1be821671344065aef47bd08274e950e614ed5a36b3ff7cf9812c83cd5b`.
This is an **author-developed bounded deterministic fixture**, not an official
ARC-AGI-3 evaluation or a demonstration of general intelligence.

## Reproduction

From `experiments/e6_online_revision_transfer` on Python 3.11+:

```bash
python3 -m unittest discover -s tests -v
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5b_effects.py --cases 100 --output results/day5b_effects100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day5a_epochs.py --cases 100 --output results/day5b_day5a_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day4b_goals.py --cases 100 --output results/day5b_day4b_regression100.json
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day4_stress.py --cases 100 --output results/day5b_day4_regression100.json
python3 -m athlesia.benchmark --cases 100 --output results/day5b_broker_regression100.json
sha256sum -c SHA256SUMS.txt
```

The results are gitignored: JSON below reproduces measured checkpoint values,
not committed result artifacts. Tests collected: **59**. A separate-process
solver worker integration smoke additionally verified the `effect_shift` wire
command: quarantine, quarantine, commit and later exact prediction. Its
synthetic traces were supplied by a private **in-process** fixture; it is not
an independent cross-mechanism benchmark or a separate-process environment.

## Day5b: changed single effect — 100 cases

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
  "wall_ms_total": 6741.674001
}
```

The four 100-count checks concern the **same 100 cases**. They do not constitute
400 unrelated test worlds. On the first observed contradiction, an already
identified, unguarded action is quarantined; two changed-effect examples and one
idempotent/no-op example from distinct episodes allow replacement. Old models
are archived, unrelated action models preserved. This is within a very small
Boolean hypothesis class and assumes stationary dynamics between observations.

## Day5a guard shift regression — 100 cases

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
  "wall_ms_total": 8106.195564
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
  "wall_ms_total": 4454.387723
}
```

## Day4 topology regression — 100 cases

```json
{
  "benchmark": "Athlesia E6 Day 4 topology-extension fixture stress v0.1",
  "test_family": "instrumented Boolean fixture; 3..5 channels, 1..2 positive guards",
  "evaluator": "in-process deterministic fixture, not independently developed",
  "not_official_arc_agi_3": true,
  "solver_source_sha256_subset": "9024c0c52fa71ab0dc11e56dbceafc3d46a742338f1e64ee4817464da60b0fa7",
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
  "total_cpu_ms": 3487.137494,
  "total_wall_ms": 3549.326701,
  "wall_ms_per_case_p50": 30.327248,
  "wall_ms_per_case_p95": 58.460304,
  "process_peak_rss_bytes_upper": 26210304,
  "limitations": "Does not test novel goals, new rule grammars, missing observation, subprocess isolation, or unseen non-fixture mechanisms."
}
```

## Day1–3 broker regression — 100 cases

```json
{
  "benchmark": "Athlesia bounded Day-3 online validation v0.3",
  "source_sha256": "004f7265065bfcb56e4a92a5dedb659a55f225c0b0f762eeee0ce69433c7c21b",
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
  "solver_total_cpu_ms_p50": 113.460665,
  "solver_total_cpu_ms_p95": 170.860321,
  "case_wall_ms_p50_including_startup": 1105.065273,
  "case_wall_ms_p95_including_startup": 1230.668247,
  "max_solver_peak_rss_bytes": 25993216,
  "max_sum_process_peak_rss_bytes": 77979648,
  "isolation": "trusted_code_subprocess_protocol_only",
  "novelty_claim": false,
  "robotics_or_cross_mechanism_transfer_tested": false,
  "scope": "3..5 guard channels, three mixed opaque action schemas, fixed positive-conjunction grammar, relabelled isomorphic worlds"
}
```

## Known limitations and falsification priorities

- Author-developed generator; no independent hidden task families or official
  ARC-AGI-3 score. Changing RNG seeds alone is not distributional novelty.
- Observations are complete Boolean snapshots, and the learner is supplied
  externally recorded transitions, not free autonomous exploration.
- One changed Boolean effect role/value on an **identified, unguarded action**;
  no simultaneous effect/guard changes, multi-effect discovery, partial states,
  noisy observations or unknown predicates.
- Two positive transitions and one no-op are not a statistical proof of
  causality. The no-op can be idempotent and is not a discriminating witness
  against alternate conditional models.
- Full effect_shift worker command smoke verified the JSON protocol, but the
  100-case effect pilot runs in-process and the 100-case broker regression does
  not measure effect_shift independently through solver/environment processes.
- The exact held-out results and memory audits do not establish general
  transfer across truly unrelated mechanisms. Report rejection and adaptation
  rates independently on future locked tests, and count all probing actions.
