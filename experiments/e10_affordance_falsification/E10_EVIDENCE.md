# Athlesia E10 — falsification of the E9 visual affordance discovery grammar

Recorded UTC: 2026-10-13T22:39:06Z. Frozen E9 baseline commit: `2dfc8eecbe512abdd10f5245d77a4c89c8db0a77`.
Frozen schedule SHA-256: `861dbe648ee754f71aebb91b722bc7d076865f12e5e2ba13da2067addfca3988`.
Original unchanged E10 pilot source digests: challenge `0d0d6e0ba89ab18c331c4d7050bae6bce8a30f0432a48f1e9f6d3d6f7c2f2b08`, evaluator/test `e450410f6e29e0a5150a63fb834aa45b15bbf56e2ac1abc2132ae468ea579a4e`.

## What the experiment establishes

This is a **same-author, deterministic, instrumented, 9x9 pixel fixture**
confronting the unchanged E9 visual-action discovery interface with ten world
families: 20 in-class environments and 90 out-of-class environments. The E9
agent knows only public frames, an action inventory, and five external E7
movement demonstrations. It does **not** import the private E10 challenger.
The E10 auditor uses further real public interactions on **the same episode**
after discovery; observed failures disprove the learned stable-control
hypothesis. The audit is deliberately bounded and cannot certify perpetual
stationarity or general open-world control.

**The headline is a genuine negative result**: E9 accepted
**13/90 out-of-class**
cases, with **13** accepted models
behaviorally falsified by additional interventions. Accepting an out-of-class
world is counted as a false positive even if a short post-audit were to pass.
The other **77/90** out-of-class
cases were refused, and all **20/20**
in-class cases were accepted. This is a falsification of unconditional E9
reliability, **not a new causal-transfer or ARC-AGI-3 score**. E9's prior
in-class results remain valid within their original scope.

## Error analysis and accountability

- Late semantic drift: 10/10 out-of-class cases **falsely accepted**.
- Nonstationary control: 3/10 **falsely accepted** and 7/10 refused.
- Seven other unsupported variants: all 70/70 refused, within this test.
- Total discovery operations across 110 worlds: **574**.
- Additional auditor operations: **166**. These are not
  discovery/training actions; external E7 tutorial actions also excluded.
- Each accepted case was audited, but no finite audit establishes indefinite
  stability. The challenge is not independently authored and reuses E9's
  public visual ontology. E6-E9 production sources were not changed.

## Reproduce (Athlesia repo root; Python 3.11+)

```bash
E10=experiments/e10_affordance_falsification
E9=experiments/e9_unlabelled_pixel_affordances
E8=experiments/e8_pixel_causal_bridge
E7=experiments/e7_raw_grid_transfer
E6=experiments/e6_online_revision_transfer
export PYTHONPATH="$PWD/$E10:$PWD/$E9:$PWD/$E8:$PWD/$E7:$PWD/$E6"
python3 -m unittest discover -s "$E10" -p test_e10.py -v
python3 "$E10/test_e10.py" --cases 110 --output "$E10/results/e10_falsification110.json"
python3 "$E9/test_e9.py" --cases 100 --budget 12 --output "$E9/results/e10_e9_100.json"
python3 "$E8/test_e8.py" --cases 100 --budget 12 --output "$E8/results/e10_e8_100.json"
python3 "$E7/test_e7.py" --cases 100 --budget 60 --output "$E7/results/e10_e7_100.json"
(cd "$E6" && PYTHONPATH="$PWD:$PWD/tests" python3 tests/test_day6b_external.py --cases 100 --budget 12 --output results/e10_day6b_100.json)
for sub in "$E10" "$E9" "$E8" "$E7" "$E6"; do (cd "$sub" && sha256sum -c SHA256SUMS.txt); done
```

Result JSON files are deliberately untracked. The same-author challenge
checks discovery consistency, not independent generalization or any official
ARC-AGI-3 level. No fixed success threshold was used to redefine the
negative cases as success. The checkpoint reproduces the frozen observed
110-case distribution; if it changes, finalization stops for investigation.

## E10 adversarial results — all 110 cases

```json
{
  "benchmark": "Athlesia E10 E9 out-of-grammar affordance falsification v0.1",
  "cases": 110,
  "schedule_sha256": "861dbe648ee754f71aebb91b722bc7d076865f12e5e2ba13da2067addfca3988",
  "supported_cases": 20,
  "supported_accepted": 20,
  "supported_refused": 0,
  "unsupported_cases": 90,
  "unsupported_refused": 77,
  "unsupported_accepted_false_positive": 13,
  "false_accepts_falsified_by_post_audit": 13,
  "all_accepted_audited_pass": 20,
  "all_accepted_audited_fail": 13,
  "actual_discovery_actions": 574,
  "post_audit_actions": 166,
  "by_variant": [
    {
      "variant": "supported",
      "cases": 20,
      "accepted": 20,
      "refused": 0,
      "accepted_unsound_at_audit": 0,
      "discovery_actions": 160
    },
    {
      "variant": "inventory_missing",
      "cases": 10,
      "accepted": 0,
      "refused": 10,
      "accepted_unsound_at_audit": 0,
      "discovery_actions": 0
    },
    {
      "variant": "inventory_extra",
      "cases": 10,
      "accepted": 0,
      "refused": 10,
      "accepted_unsound_at_audit": 0,
      "discovery_actions": 0
    },
    {
      "variant": "dead_control",
      "cases": 10,
      "accepted": 0,
      "refused": 10,
      "accepted_unsound_at_audit": 0,
      "discovery_actions": 80
    },
    {
      "variant": "multipixel_effect",
      "cases": 10,
      "accepted": 0,
      "refused": 10,
      "accepted_unsound_at_audit": 0,
      "discovery_actions": 14
    },
    {
      "variant": "aliased_controls",
      "cases": 10,
      "accepted": 0,
      "refused": 10,
      "accepted_unsound_at_audit": 0,
      "discovery_actions": 80
    },
    {
      "variant": "ambiguous_gate",
      "cases": 10,
      "accepted": 0,
      "refused": 10,
      "accepted_unsound_at_audit": 0,
      "discovery_actions": 0
    },
    {
      "variant": "irreversible_latch",
      "cases": 10,
      "accepted": 0,
      "refused": 10,
      "accepted_unsound_at_audit": 0,
      "discovery_actions": 80
    },
    {
      "variant": "late_semantic_drift",
      "cases": 10,
      "accepted": 10,
      "refused": 0,
      "accepted_unsound_at_audit": 10,
      "discovery_actions": 80
    },
    {
      "variant": "nonstationary_control",
      "cases": 10,
      "accepted": 3,
      "refused": 7,
      "accepted_unsound_at_audit": 3,
      "discovery_actions": 80
    }
  ],
  "sample_false_accepts": [
    {
      "case": 3,
      "kind": "late_semantic_drift",
      "in_declared_class": false,
      "discovery_accepted": true,
      "post_discovery_audit_pass": false,
      "acceptance_of_out_of_class": true,
      "accepted_unsound_at_audit": true,
      "discovery_actions": 8,
      "post_audit_actions": 5,
      "reason": "incorrect_learned_successor"
    },
    {
      "case": 11,
      "kind": "late_semantic_drift",
      "in_declared_class": false,
      "discovery_accepted": true,
      "post_discovery_audit_pass": false,
      "acceptance_of_out_of_class": true,
      "accepted_unsound_at_audit": true,
      "discovery_actions": 8,
      "post_audit_actions": 5,
      "reason": "incorrect_learned_successor"
    },
    {
      "case": 47,
      "kind": "late_semantic_drift",
      "in_declared_class": false,
      "discovery_accepted": true,
      "post_discovery_audit_pass": false,
      "acceptance_of_out_of_class": true,
      "accepted_unsound_at_audit": true,
      "discovery_actions": 8,
      "post_audit_actions": 1,
      "reason": "incorrect_learned_successor"
    },
    {
      "case": 50,
      "kind": "late_semantic_drift",
      "in_declared_class": false,
      "discovery_accepted": true,
      "post_discovery_audit_pass": false,
      "acceptance_of_out_of_class": true,
      "accepted_unsound_at_audit": true,
      "discovery_actions": 8,
      "post_audit_actions": 2,
      "reason": "incorrect_learned_successor"
    },
    {
      "case": 53,
      "kind": "late_semantic_drift",
      "in_declared_class": false,
      "discovery_accepted": true,
      "post_discovery_audit_pass": false,
      "acceptance_of_out_of_class": true,
      "accepted_unsound_at_audit": true,
      "discovery_actions": 8,
      "post_audit_actions": 2,
      "reason": "incorrect_learned_successor"
    }
  ],
  "sample_refusals": [
    {
      "case": 0,
      "kind": "inventory_extra",
      "in_declared_class": false,
      "discovery_accepted": false,
      "post_discovery_audit_pass": null,
      "acceptance_of_out_of_class": false,
      "accepted_unsound_at_audit": false,
      "discovery_actions": 0,
      "post_audit_actions": 0,
      "reason": "DiscoveryError: Expected eight unique opaque public actions"
    },
    {
      "case": 1,
      "kind": "inventory_missing",
      "in_declared_class": false,
      "discovery_accepted": false,
      "post_discovery_audit_pass": null,
      "acceptance_of_out_of_class": false,
      "accepted_unsound_at_audit": false,
      "discovery_actions": 0,
      "post_audit_actions": 0,
      "reason": "DiscoveryError: Expected eight unique opaque public actions"
    },
    {
      "case": 2,
      "kind": "inventory_extra",
      "in_declared_class": false,
      "discovery_accepted": false,
      "post_discovery_audit_pass": null,
      "acceptance_of_out_of_class": false,
      "accepted_unsound_at_audit": false,
      "discovery_actions": 0,
      "post_audit_actions": 0,
      "reason": "DiscoveryError: Expected eight unique opaque public actions"
    }
  ],
  "unchanged_e9_discovery": true,
  "same_author_challenger": true,
  "e9_public_ontology_shared": true,
  "no_external_episode_reset_claim": true,
  "official_arc_agi_3": false,
  "wall_ms_total": 96.144
}
```

## Prior in-class regressions, with source and seed schedules unchanged

### E9 — 100 discovery cases

```json
{
  "benchmark": "Athlesia E9 unlabeled pixel-affordance discovery E7/E6 pilot v0.1",
  "cases": 100,
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
  "paired": {
    "both_correct": 96,
    "active_only": 4,
    "random_only": 0,
    "neither": 0
  },
  "official_arc_agi_3": false
}
```

### E8 — 100 pixel/causal cases

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
  "official_arc_agi_3": false
}
```

### E7 — 100 held-out raw-grid cases

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

### E6 — 100 separate-evaluator Boolean cases

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
  "official_arc_agi_3": false
}
```

## Next experimental hypothesis

Treat a successful calibration as **provisional**, not a certified stationary
law. Introduce bounded repeated and held-out public interventions, record
stationarity uncertainty and withdrawal of earlier acceptances upon observed
contradictions. Do not promise complete safety against arbitrarily delayed
changes. Compare verification costs and false acceptances against original E10,
including the previously failing delayed/nonstationary variants.
