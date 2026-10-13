# Athlesia E6 — Day 4 topology-extension pilot checkpoint

Date (UTC): 2026-10-13. This is a bounded symbolic AML *pilot*, not general AGI,
not independent held-out environment generation and not official ARC-AGI-3.

## Reproduce

From `experiments/e6_online_revision_transfer` (Python 3.11+, Linux):

```bash
python3 -m unittest discover -s tests -v
PYTHONPATH="$PWD" python3 tests/test_day4_stress.py --cases 100 --output results/day4_stress100.json
python3 -m athlesia.benchmark --cases 100 --output results/day4_day3_regression100.json
sha256sum -c SHA256SUMS.txt
```

Generated benchmark `.json`/`.jsonl` files remain gitignored, so the
numbers below are the observed checkpoint, not output bundled with the commit.

## Day 4: expanded topology stress (in-process evaluator)

```json
{
  "benchmark": "Athlesia E6 Day 4 topology-extension fixture stress v0.1",
  "test_family": "instrumented Boolean fixture; 3..5 channels, 1..2 positive guards",
  "evaluator": "in-process deterministic fixture, not independently developed",
  "not_official_arc_agi_3": true,
  "solver_source_sha256_subset": "f441c2d511db8d14d68dadfd2552b994460ec78608c6b8d9d9d4edc99009f872",
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
  "total_cpu_ms": 3358.46528,
  "total_wall_ms": 3384.709305,
  "wall_ms_per_case_p50": 31.188144,
  "wall_ms_per_case_p95": 49.536626,
  "process_peak_rss_bytes_upper": 26324992,
  "limitations": "Does not test novel goals, new rule grammars, missing observation, subprocess isolation, or unseen non-fixture mechanisms."
}
```

The 400 targeted checks are 100 cases × four properties; they are not
400 independent, unrelated environment families. The test itself constructs
extra object/fluents/goals, but does not show novel-goal solving. Memory digest
remained unchanged in accepted and rejected transfers.

## Day 1–3 regression under modified Day-4 sources (separate-process broker)

```json
{
  "benchmark": "Athlesia bounded Day-3 online validation v0.3",
  "source_sha256": "95b7a480caed51d25dd02235b35f8360619133ba6bb894f47adf80a2817ca576",
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
  "solver_total_cpu_ms_p50": 109.572657,
  "solver_total_cpu_ms_p95": 171.676746,
  "case_wall_ms_p50_including_startup": 1082.415332,
  "case_wall_ms_p95_including_startup": 1204.736714,
  "max_solver_peak_rss_bytes": 25927680,
  "max_sum_process_peak_rss_bytes": 77783040,
  "isolation": "trusted_code_subprocess_protocol_only",
  "novelty_claim": false,
  "robotics_or_cross_mechanism_transfer_tested": false,
  "scope": "3..5 guard channels, three mixed opaque action schemas, fixed positive-conjunction grammar, relabelled isomorphic worlds"
}
```

This regression still uses isomorphic training/target graphs and does not
exercise Day-4 topology extension in the separate-process broker.

## Explicit experimental boundaries

- 3–5 instrumented Boolean channels, 1–2 positive conjunctive guards.
- Known graph relation types and known control affordances; learned mechanism
  still belongs to the declared finite hypothesis class.
- Full, deterministic observations; no noisy/missing observations, no invented
  predicates, no new goals, no new action grammar or true mechanism relearning.
- A unique induced mapping is required. Ambiguous/over-budget matches fail closed.
- Expanded-world stress is in-process and author-developed; not a second
  independent generator, not hostile-code sandboxing, not an ARC-AGI-3 score.
- Peak-RSS from the stress process is a process-lifetime maximum and is not
  interchangeable with combined peak-RSS in the subprocess regression.
