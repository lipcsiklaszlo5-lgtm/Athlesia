# Athlesia E6 — Day 4b declared-goal pilot checkpoint

UTC date: 2026-10-13. All results below are measured against the current local
working tree before this checkpoint commit. Day4b pilot source subset SHA-256:
`35300fd26e52167671b8ffc09497d350e6c5cebedf32742998c2429dc7547370` (`athlesia/learner.py`, `athlesia/worker.py`, `tests/test_day4b_goals.py`).

## Reproduce (Linux, Python 3.11+)

From `experiments/e6_online_revision_transfer`:

```bash
python3 -m unittest discover -s tests -v
PYTHONPATH="$PWD" python3 tests/test_day4b_goals.py --cases 100 --output results/day4b_goals100.json
PYTHONPATH="$PWD" python3 tests/test_day4_stress.py --cases 100 --output results/day4b_topology_regression100.json
python3 -m athlesia.benchmark --cases 100 --output results/day4b_broker_regression100.json
sha256sum -c SHA256SUMS.txt
```

Generated `.json` and `.jsonl` results are intentionally ignored by Git. The
reported numbers are the observed checkpoint and can be regenerated.

## Declared-goal pilot: 100 in-process cases

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
  "wall_ms_total": 4184.881539
}
```

The new `goal` worker operation receives a Boolean target **from outside**. It
chooses at most one already-learned action where all surviving hypotheses agree
on the full predicted successor; then it checks the actual complete observed
state. Uncertainty, unknown targets and unexpected effects are not counted as
successful execution. In this fixture, the four 25-case categories are
positive target, negative target, expanded topology and uncertainty rejection.
These are **not** 100 independently developed worlds.

## Day 4 topology regression: 100 in-process cases / 400 checks

```json
{
  "benchmark": "Athlesia E6 Day 4 topology-extension fixture stress v0.1",
  "test_family": "instrumented Boolean fixture; 3..5 channels, 1..2 positive guards",
  "evaluator": "in-process deterministic fixture, not independently developed",
  "not_official_arc_agi_3": true,
  "solver_source_sha256_subset": "c5cd9334b2935590cdaa318380850b2941d87275c86ebd8ed08935cfd56134d4",
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
  "total_cpu_ms": 3534.624173,
  "total_wall_ms": 3640.357945,
  "wall_ms_per_case_p50": 37.204709,
  "wall_ms_per_case_p95": 57.124257,
  "process_peak_rss_bytes_upper": 26451968,
  "limitations": "Does not test novel goals, new rule grammars, missing observation, subprocess isolation, or unseen non-fixture mechanisms."
}
```

## Day 1–3 separate-process broker regression: 100 cases

```json
{
  "benchmark": "Athlesia bounded Day-3 online validation v0.3",
  "source_sha256": "0f019924bb2cc703bf1592226e4acbf87225f8ed99bd128d089bed9f7fb5b4e4",
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
  "solver_total_cpu_ms_p50": 115.058335,
  "solver_total_cpu_ms_p95": 169.949065,
  "case_wall_ms_p50_including_startup": 1114.20278,
  "case_wall_ms_p95_including_startup": 1239.844011,
  "max_solver_peak_rss_bytes": 25866240,
  "max_sum_process_peak_rss_bytes": 77598720,
  "isolation": "trusted_code_subprocess_protocol_only",
  "novelty_claim": false,
  "robotics_or_cross_mechanism_transfer_tested": false,
  "scope": "3..5 guard channels, three mixed opaque action schemas, fixed positive-conjunction grammar, relabelled isomorphic worlds"
}
```

The separate-process broker regression does **not** exercise the new Day4b
`goal` worker command; that command is covered by unit and in-process pilot
tests. This is trusted-code subprocess protocol separation, not a hostile-code
sandbox. This checkpoint does not establish official ARC-AGI-3 performance.

## Limits and next falsification target

- Does not invent goals, new predicates, new actions, or unknown causal laws.
- Assumes complete deterministic Boolean observations, typed graph relations,
  unique correspondence, known interventions and a bounded action hypothesis class.
- Plans at most one already learned action; no multi-step planning or exploration.
- A changed mechanism is detected when the action disagrees with prediction,
  but the mechanism is **not** relearned in this pilot.
- The Day4b case-family evaluator is in-process and developed alongside solver.
  Independent frozen cases and separate-process goal evaluation remain open.
- Case-family timings do not include remote model calls or official evaluation.
