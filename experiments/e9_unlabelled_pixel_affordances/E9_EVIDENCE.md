# Athlesia E9 — unlabelled pixel-affordance discovery checkpoint

Recorded UTC: 2026-10-13T22:25:03Z. Baseline E8 commit: `47235744a9f868e1264e0a0353acc68c99dde669`.

Source SHA-256: discovery `1a81439ba3aa8f52638f886e60e5b0dd7bc2060a45152d9a0a231d230278714f`, environment `00cd0e76459f7e642e079513abd8221e9740c5073fc3eac8ec08ad2374337b04`, test `f3c677d5a89af8b98e9e0592c2bccd76bcba52358998c19f8c7190bcc992c652`.
Frozen E9 schedule SHA-256: `f77b451be789496a39eb2c807cfc1e9ae03119f403c675446393e7f134aba918`.

## What was actually evaluated

The E9 experiment reuses **unchanged E7** opaque movement-token grounding and
**unchanged E6** symbolic guard hypotheses, quarantine and active experiments.
Unlike E8, the solver is not told the gate coordinates, the switch positions,
or which on/off control tokens form pairs. It receives 9×9 opaque-colour frames,
an eight-action public inventory and five **external** E7 demonstrations. It
classifies four known movement tokens, then executes a fixed two-pass action
exploration to identify two single-pixel, reversible Boolean controls and a
unique adjacent visually distinguished gate. It creates the E6 symbolic scene
from public pixel transitions, detects a mismatch from publicly enumerated
switch settings (requiring fresh externally reset episodes), and then runs
E6 active selection versus a uniform-random control with at most 12 later
probe actions per arm. A test-side scorer verifies the four complete visible
Boolean successor transitions on fresh episodes rather than trusting a commit
flag. These scoring actions are **not charged as learning actions**.

**Limits:** Same-author fully observed, deterministic, in-process 9×9 world.
Known five movement tutorials, fixed action inventory, exactly two reversible
single-pixel switches, identifiable adjacent gate marker, fixed E6 conjunctive
Boolean grammar of at most two literals, four historical traces *per arm*, and
externally supplied episode reset. It does not discover an open-ended action
ontology, perform partial-observation reasoning, autonomously reset episodes,
invent its goal, or solve an official ARC-AGI-3 task. Its detection phase uses
exhaustive enumeration over only two discovered visual switches. A separate
implementation or external scorer has not independently validated E9.

## Cost accounting

The measured shared discovery cost is **800 actions**
(8 per case), performed once before
both strategies. A separately measured contradiction-search cost is
**346 actions per 100-case arm**.
After quarantine, active/random cost respectively
**491 / 1100**
real target-plus-control actions; including contradiction search this becomes
**837 / 1446**.
The shared discovery is **not** included in those arm totals, and the
additional 5 tutorial transitions per case, 4 old-rule training traces per
arm, and scoring-only held-out validation actions are excluded. Thus neither
the post-quarantine sums nor the discovery-plus-detection sums are total
end-to-end costs. Failures stay in the 100-case denominator.

## Reproduce (Athlesia repo root; Python 3.11+)

```bash
E9=experiments/e9_unlabelled_pixel_affordances
E8=experiments/e8_pixel_causal_bridge
E7=experiments/e7_raw_grid_transfer
E6=experiments/e6_online_revision_transfer
PYTHONPATH="$PWD/$E9:$PWD/$E8:$PWD/$E7:$PWD/$E6" python3 -m unittest discover -s "$E9" -p test_e9.py -v
PYTHONPATH="$PWD/$E9:$PWD/$E8:$PWD/$E7:$PWD/$E6" python3 "$E9/test_e9.py" --cases 100 --budget 12 --output "$E9/results/e9_100.json"
PYTHONPATH="$PWD/$E8:$PWD/$E7:$PWD/$E6" python3 "$E8/test_e8.py" --cases 100 --budget 12 --output "$E8/results/e9_e8_100.json"
(cd "$E7" && python3 test_e7.py --cases 100 --budget 60 --output results/e9_e7_100.json)
(cd "$E6" && PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day6b_external.py --cases 100 --budget 12 --output results/e9_e6_100.json)
for sub in "$E9" "$E8" "$E7" "$E6"; do (cd "$sub" && sha256sum -c SHA256SUMS.txt); done
```

Generated result files under `results/` are excluded from the Git commit.
Wall times are not official single-core ARC-AGI-3 timings. This report
records observed results; it does not impose or assume an active-win threshold.

## E9 — 100 paired cases

```json
{
  "benchmark": "Athlesia E9 unlabeled pixel-affordance discovery E7/E6 pilot v0.1",
  "cases": 100,
  "budget_per_arm": 12,
  "seed_schedule_sha256": "f77b451be789496a39eb2c807cfc1e9ae03119f403c675446393e7f134aba918",
  "active": {
    "correct_models": 100,
    "failures": 0,
    "post_probe_actions": 258,
    "post_control_actions": 233,
    "post_real_actions": 491,
    "drift_detection_actions": 346,
    "real_actions_including_detection": 837,
    "status_counts": {
      "committed": 100
    }
  },
  "uniform_random": {
    "correct_models": 96,
    "failures": 4,
    "post_probe_actions": 538,
    "post_control_actions": 562,
    "post_real_actions": 1100,
    "drift_detection_actions": 346,
    "real_actions_including_detection": 1446,
    "status_counts": {
      "budget_exhausted": 4,
      "committed": 96
    }
  },
  "discovery_actions_shared": 800,
  "external_movement_tutorial_actions_shared_per_case": 5,
  "external_historical_action_traces_per_arm": 4,
  "paired": {
    "both_correct": 96,
    "active_only": 4,
    "random_only": 0,
    "neither": 0
  },
  "law_families": [
    {
      "family": 0,
      "cases": 25,
      "active_correct": 25,
      "random_correct": 23
    },
    {
      "family": 1,
      "cases": 25,
      "active_correct": 25,
      "random_correct": 25
    },
    {
      "family": 2,
      "cases": 25,
      "active_correct": 25,
      "random_correct": 24
    },
    {
      "family": 3,
      "cases": 25,
      "active_correct": 25,
      "random_correct": 24
    }
  ],
  "sample_cases": [
    {
      "family": 0,
      "calibration_actions_shared": 8,
      "active": {
        "correct": true,
        "status": "committed",
        "post_probe_actions": 3,
        "post_control_actions": 3,
        "post_real_actions": 6,
        "drift_detection_actions": 2,
        "drift_detection_probes": 1,
        "drift_detection_episodes": 1,
        "real_actions_including_detection": 8,
        "archived": true,
        "error": null
      },
      "random": {
        "correct": true,
        "status": "committed",
        "post_probe_actions": 8,
        "post_control_actions": 7,
        "post_real_actions": 15,
        "drift_detection_actions": 2,
        "drift_detection_probes": 1,
        "drift_detection_episodes": 1,
        "real_actions_including_detection": 17,
        "archived": true,
        "error": null
      }
    },
    {
      "family": 1,
      "calibration_actions_shared": 8,
      "active": {
        "correct": true,
        "status": "committed",
        "post_probe_actions": 2,
        "post_control_actions": 1,
        "post_real_actions": 3,
        "drift_detection_actions": 3,
        "drift_detection_probes": 1,
        "drift_detection_episodes": 1,
        "real_actions_including_detection": 6,
        "archived": true,
        "error": null
      },
      "random": {
        "correct": true,
        "status": "committed",
        "post_probe_actions": 6,
        "post_control_actions": 5,
        "post_real_actions": 11,
        "drift_detection_actions": 3,
        "drift_detection_probes": 1,
        "drift_detection_episodes": 1,
        "real_actions_including_detection": 14,
        "archived": true,
        "error": null
      }
    },
    {
      "family": 2,
      "calibration_actions_shared": 8,
      "active": {
        "correct": true,
        "status": "committed",
        "post_probe_actions": 3,
        "post_control_actions": 3,
        "post_real_actions": 6,
        "drift_detection_actions": 5,
        "drift_detection_probes": 2,
        "drift_detection_episodes": 2,
        "real_actions_including_detection": 11,
        "archived": true,
        "error": null
      },
      "random": {
        "correct": true,
        "status": "committed",
        "post_probe_actions": 3,
        "post_control_actions": 3,
        "post_real_actions": 6,
        "drift_detection_actions": 5,
        "drift_detection_probes": 2,
        "drift_detection_episodes": 2,
        "real_actions_including_detection": 11,
        "archived": true,
        "error": null
      }
    },
    {
      "family": 3,
      "calibration_actions_shared": 8,
      "active": {
        "correct": true,
        "status": "committed",
        "post_probe_actions": 3,
        "post_control_actions": 3,
        "post_real_actions": 6,
        "drift_detection_actions": 2,
        "drift_detection_probes": 1,
        "drift_detection_episodes": 1,
        "real_actions_including_detection": 8,
        "archived": true,
        "error": null
      },
      "random": {
        "correct": false,
        "status": "budget_exhausted",
        "post_probe_actions": 12,
        "post_control_actions": 14,
        "post_real_actions": 26,
        "drift_detection_actions": 2,
        "drift_detection_probes": 1,
        "drift_detection_episodes": 1,
        "real_actions_including_detection": 28,
        "archived": false,
        "error": null
      }
    }
  ],
  "raw_pixels_and_opaque_action_inventory": true,
  "public_control_pairs_prelabelled": false,
  "gate_coordinate_given_to_agent": false,
  "discovery_grammar_fixed_two_single_pixel_reversible_switches": true,
  "five_external_movement_tutorials": true,
  "external_episodes": true,
  "same_author_evaluator": true,
  "official_arc_agi_3": false,
  "wall_ms_total": 4693.902
}
```

## E8 — 100-case integration regression

```json
{
  "benchmark": "Athlesia E8 E7-pixels to E6-guard-learning integration pilot v0.1",
  "cases": 100,
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
  "official_arc_agi_3": false
}
```

## E7 — 100-case visual regression

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

## E6 — 100-case separate-implementation regression

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

## Next falsification experiments

1. Remove the *exactly two reversible single-pixel control* prior; include
   varying controls, distractors, inert or irreversible actions, and wrong
   affordance hypotheses; measure abstentions and wasted discovery actions.
2. Replace the evaluator-supplied episode boundaries with an explicit
   observable reset contract, recording every action and failure.
3. Run in truly separate solver/evaluator processes with strict CPU and RAM
   budgets, and test visual occlusions and scene deformation.
4. Validate on external evaluators and the official ARC-AGI-3 API before
   claiming broader intelligence or task-level progress.
