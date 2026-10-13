# Athlesia E8 — bounded pixel-to-causal bridge checkpoint

Recorded UTC: 2026-10-13T22:10:24Z. Baseline commit: `b1c437131c1308cf1a51eb94d340e3e0ce6de11b`.
E8 bridge SHA-256: `3400fa0db1c7969fc66a950fd40484e2f9101508b1bda94b0959e0c4a27f9e23`; E8 private evaluator SHA-256: `111236cd9da2a6f3e2219e1d98b7255aef5cde029593e16a95e09cbeefe0c4fe`; test SHA-256: `5804af7f15e865cdb0a646fd5234623d255795d985d4c0e508e3573564ba5584`.
Frozen 100-case schedule SHA-256: `563ab0061a022a31a8fae552e40440f93a941911c11a1d8e05a391a61f63136b`.

## Claim boundary

This experiment imports the **real, unchanged E7 visual `Agent`** and the
**real, unchanged E6 causal `Agent`**. It uses five external demonstrations to
ground four opaque cardinal movement actions. Two public on/off controls are
calibrated against a fully visible 9×9 integer-colour grid; the bridge maps two
instrumented indicator pixels and actor position into E6's Boolean symbolic
scene, and E6 chooses and performs active guard probes. One initial
contradiction and all fresh episodes are supplied by the evaluator.

Neither `bridge.py` nor the E7/E6 learner directly imports the E8 private
simulator. **This is still an in-process same-author experiment.** It is not an
independently authored benchmark, sandboxed interactive evaluation, general
visual-object discovery, unsupervised grounding of affordances, autonomous
episode-reset detection, new action-grammar learning, arbitrary long-horizon
planning, or an official ARC-AGI-3 result. The bridge has hardcoded two
indicator roles, a declared gate cell, known set-true/set-false controls, full
visibility, and a fixed Boolean conjunction grammar (at most 2 literals).
The environment specifies initial old-law training examples and selects the
first contradictory example; the active policy only chooses subsequent probes.
**Action counts include only post-quarantine intervention and target-probe
actions.** They explicitly exclude the five E7 tutorials, four E6 historical
training traces per arm, the externally selected first contradictory action
per arm, and the shared four initial control-calibration actions per case.
Consequently 550/1082 are comparable post-quarantine costs, not total
end-to-end training costs. The fresh episode resets are likewise external.

## Reproduction

From Athlesia repo root:

```bash
E8=experiments/e8_pixel_causal_bridge
E7=experiments/e7_raw_grid_transfer
E6=experiments/e6_online_revision_transfer
export PYTHONPATH="$PWD/$E8:$PWD/$E7:$PWD/$E6"
python3 -m unittest discover -s "$E8" -p 'test_e8.py' -v
python3 "$E8/test_e8.py" --cases 100 --budget 12 --output "$E8/results/e8_pixel_causal100.json"
(cd "$E7" && python3 -m unittest -v test_e7 && python3 test_e7.py --cases 100 --budget 60 --output results/e8_finalize_e7_100.json)
(cd "$E6" && python3 -m unittest discover -s tests -q && PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day6b_external.py --cases 100 --budget 12 --output results/e8_finalize_day6b100.json)
(cd "$E8" && sha256sum -c SHA256SUMS.txt)
(cd "$E7" && sha256sum -c SHA256SUMS.txt)
(cd "$E6" && sha256sum -c SHA256SUMS.txt)
```

The JSON files under `results/` are intentionally ignored in Git; the
following reproduces the actual measured checkpoint results. Wall times are
not claimed as single-CPU constrained or official benchmark latencies.

## E8 paired visual-causal learning — 100 cases

```json
{
  "benchmark": "Athlesia E8 E7-pixels to E6-guard-learning integration pilot v0.1",
  "cases": 100,
  "budget_probes_per_arm": 12,
  "fixed_schedule_sha256": "563ab0061a022a31a8fae552e40440f93a941911c11a1d8e05a391a61f63136b",
  "active": {
    "successes": 100,
    "failures": 0,
    "probe_actions": 275,
    "calibration_actions": 275,
    "real_actions": 550,
    "status_counts": {
      "committed": 100
    }
  },
  "uniform_random": {
    "successes": 93,
    "failures": 7,
    "probe_actions": 542,
    "calibration_actions": 540,
    "real_actions": 1082,
    "status_counts": {
      "budget_exhausted": 7,
      "committed": 93
    }
  },
  "paired": {
    "both_success": 93,
    "active_only": 7,
    "random_only": 0,
    "both_failed": 0
  },
  "by_law": [
    {
      "name": "((1, True),)",
      "cases": 25,
      "active_successes": 25,
      "random_successes": 24
    },
    {
      "name": "((1, False),)",
      "cases": 25,
      "active_successes": 25,
      "random_successes": 25
    },
    {
      "name": "((0, True), (1, True))",
      "cases": 25,
      "active_successes": 25,
      "random_successes": 21
    },
    {
      "name": "((0, False), (1, True))",
      "cases": 25,
      "active_successes": 25,
      "random_successes": 23
    }
  ],
  "sample_cases": [
    {
      "index": 0,
      "law_family": 0,
      "active": {
        "success": true,
        "status": "committed",
        "probes": 3,
        "calibration_actions": 3,
        "real_actions": 6,
        "error": null,
        "archived": true
      },
      "random": {
        "success": true,
        "status": "committed",
        "probes": 5,
        "calibration_actions": 5,
        "real_actions": 10,
        "error": null,
        "archived": true
      }
    },
    {
      "index": 1,
      "law_family": 1,
      "active": {
        "success": true,
        "status": "committed",
        "probes": 2,
        "calibration_actions": 2,
        "real_actions": 4,
        "error": null,
        "archived": true
      },
      "random": {
        "success": true,
        "status": "committed",
        "probes": 6,
        "calibration_actions": 7,
        "real_actions": 13,
        "error": null,
        "archived": true
      }
    },
    {
      "index": 2,
      "law_family": 2,
      "active": {
        "success": true,
        "status": "committed",
        "probes": 3,
        "calibration_actions": 3,
        "real_actions": 6,
        "error": null,
        "archived": true
      },
      "random": {
        "success": true,
        "status": "committed",
        "probes": 9,
        "calibration_actions": 7,
        "real_actions": 16,
        "error": null,
        "archived": true
      }
    },
    {
      "index": 3,
      "law_family": 3,
      "active": {
        "success": true,
        "status": "committed",
        "probes": 3,
        "calibration_actions": 3,
        "real_actions": 6,
        "error": null,
        "archived": true
      },
      "random": {
        "success": true,
        "status": "committed",
        "probes": 3,
        "calibration_actions": 3,
        "real_actions": 6,
        "error": null,
        "archived": true
      }
    }
  ],
  "simulator_source_imported_by_bridge": false,
  "pixel_controls_preinstrumented": true,
  "fully_observable": true,
  "externally_supplied_first_contradiction_and_resets": true,
  "same_author_test": true,
  "official_arc_agi_3": false,
  "wall_ms_total": 4381.434
}
```

## E7 raw-grid regression — 100 cases

```json
{
  "benchmark": "Athlesia E7 raw pixel grid transition-grounding pilot v0.1",
  "cases": 100,
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
  "official_arc_agi_3": false
}
```

## E6 Day6b separate Boolean evaluator regression — 100 cases

```json
{
  "benchmark": "Athlesia E6 Day6b separately implemented Boolean evaluator v0.1",
  "cases": 100,
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
  "official_arc_agi_3": false
}
```

## Next falsification targets

1. A version where gate/controls are not pre-labeled and their affordances
   must be found through visible interaction under strict exploration costs.
2. Different visual morphology and unpredictable distractors, occlusions,
   switching laws and nonlinear action consequences: measure refusal rates.
3. Process-isolate the bridge solver and environment, and verify byte-level
   exchanges, exact action budgets, failures and memory limits.
4. Measure generalization on externally authored environments and eventually
   official ARC-AGI-3 using the actual official protocol and scoring.
