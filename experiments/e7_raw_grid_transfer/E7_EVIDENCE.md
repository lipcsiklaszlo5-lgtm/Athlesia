# Athlesia E7 — raw-grid visual grounding pilot checkpoint

Recorded UTC: 2026-10-13T21:50:48Z.
Baseline Git commit: `790aeb5d99c4787c0935d719910280e36c04e850`.
Learner SHA-256: `f8ece20d7fc09f1773ddfae86428cb6b336bca4e248c90e9eaecd49417af1e87`; evaluator SHA-256: `f3aab54504741340ae8506748ff88c8d6f6c503913c0cb17af51d3938deb4752`.
Frozen 100-case seed schedule SHA-256: `6965de38a483775d3d6d8a15ef6a95f04961025a8d7b01cd6f3679504f842b53`.

## Claim boundary

This is **not** a claim of AGI, general-purpose visual reasoning, independent
research validation, or official ARC-AGI-3 performance. It is a separately
located, same-author, in-process deterministic visual-grid prototype. The
solver source does not import the evaluator; an additional smoke test copied
only `agent.py` into a fresh Python subprocess and verified its planned opaque
actions against the evaluator in the parent process. That smoke test is not a
fully isolated interactive evaluation and is not an adversarial sandbox.

The inputs are 9×9 integer-colour pixel arrays with exactly one moving actor,
complete maze visibility, four cardinal unit moves (unknown action IDs), one
known model class, an external target coordinate, and **five evaluator-supplied
demonstrations per case**. The training arena and held-out maze share the same
per-case action bindings and colour palette. One blocked movement grounds the
wall colour. Classical shortest-path search is then run on the full visible
map. No autonomous tutorial acquisition, unknown goal induction, partial
observation, visual object persistence, stochastic action learning, or
cross-domain transfer is demonstrated.

## Reproduction

From `experiments/e7_raw_grid_transfer`:

```bash
python3 -m unittest -v test_e7
python3 test_e7.py --cases 100 --budget 60 --output results/e7_raw_grid100.json
sha256sum -c SHA256SUMS.txt
```

E6 regression (from `experiments/e6_online_revision_transfer`):

```bash
python3 -m unittest discover -s tests -q
PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day6b_external.py --cases 100 --budget 12 --output results/e7_finalize_day6b100.json
sha256sum -c SHA256SUMS.txt
```

Generated JSON reports are ignored by Git. Results below were measured on the
checkpoint machine; elapsed wall time is not a CPU-constrained benchmark.

## Measured E7 results

```json
{
  "benchmark": "Athlesia E7 raw pixel grid transition-grounding pilot v0.1",
  "cases": 100,
  "action_budget_per_arm": 60,
  "fixed_seed_schedule_sha256": "6965de38a483775d3d6d8a15ef6a95f04961025a8d7b01cd6f3679504f842b53",
  "visual_planner": {
    "successes": 100,
    "failures": 0,
    "real_actions_all_cases": 709,
    "real_actions_median_success_only": 7,
    "status_counts": {
      "reached": 100
    }
  },
  "greedy_local": {
    "successes": 84,
    "failures": 16,
    "real_actions_all_cases": 1610,
    "real_actions_median_success_only": 7,
    "status_counts": {
      "budget_exhausted": 16,
      "reached": 84
    }
  },
  "uniform_random": {
    "successes": 21,
    "failures": 79,
    "real_actions_all_cases": 5522,
    "real_actions_median_success_only": 44,
    "status_counts": {
      "budget_exhausted": 79,
      "reached": 21
    }
  },
  "shortest_path_optimal_successes": 100,
  "training_demonstrations_per_case": 5,
  "external_goal_coordinate_provided": true,
  "action_identifiers_per_case_remapped": true,
  "pixel_palette_per_case_remapped": true,
  "public_visual_ontology": "opaque integer grids only; tutorial move effects and wall collision",
  "solver_imports_simulator": false,
  "same_author_evaluator": true,
  "official_arc_agi_3": false,
  "autonomous_goal_invention": false,
  "novel_action_grammar_learned": false,
  "partial_observation_solved": false,
  "wall_ms_total": 503.245,
  "sample_cases": [
    {
      "case": 0,
      "visual_planner": {
        "success": true,
        "status": "reached",
        "actions": 9,
        "optimal_actions": 9
      },
      "greedy_local": {
        "status": "reached",
        "success": true,
        "actions": 11
      },
      "uniform_random": {
        "status": "budget_exhausted",
        "success": false,
        "actions": 60
      }
    },
    {
      "case": 1,
      "visual_planner": {
        "success": true,
        "status": "reached",
        "actions": 7,
        "optimal_actions": 7
      },
      "greedy_local": {
        "status": "reached",
        "success": true,
        "actions": 7
      },
      "uniform_random": {
        "status": "budget_exhausted",
        "success": false,
        "actions": 60
      }
    },
    {
      "case": 2,
      "visual_planner": {
        "success": true,
        "status": "reached",
        "actions": 9,
        "optimal_actions": 9
      },
      "greedy_local": {
        "status": "reached",
        "success": true,
        "actions": 9
      },
      "uniform_random": {
        "status": "reached",
        "success": true,
        "actions": 56
      }
    }
  ]
}
```

## E6 regression summary

E6 unit tests: 75/75; Day6b separate-implementation 100 paired cases.
The E6 solver and all E6 tracked files remained unchanged.

```json
{
  "benchmark": "Athlesia E6 Day6b separately implemented Boolean evaluator v0.1",
  "cases": 100,
  "budget_probes_per_arm": 12,
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
  "pair_counts": {
    "both_success": 81,
    "active_only": 19,
    "random_only": 0,
    "both_failed": 0
  },
  "official_arc_agi_3": false
}
```

## Next falsification targets

1. Pretrain a visual representation on different backgrounds and evaluate with
   a distinct palette, object placement and maze morphology at test time.
2. Introduce unseen movement physics, occlusion and visual changes that do not
   satisfy the four fixed translation primitives; measure refusals and failures.
3. Run an interactive process-isolated evaluator without providing five
   demonstrations in advance; count data-acquisition actions and resets.
4. Test whether E6 causal hypothesis refinement can use E7 perceptual outputs
   without the E7 evaluator's latent state or goal-coordinate metadata.
