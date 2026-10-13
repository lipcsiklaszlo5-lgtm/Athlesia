# Athlesia E10 — adversarial falsification of E9 affordance discovery

This is a **negative-result pilot**, not a new AGI benchmark. It imports the actual unchanged E9 `DiscoveredBridge` and the actual unchanged E7 visual-motion learner, and confronts the E9 public API with same-author adversarial effects. The E6/E7/E8/E9 source tree is never modified.

**Pre-registered split:** 20 conforming worlds and 90 out-of-class worlds (ten cases in each of nine failure families), shuffled on a frozen deterministic schedule. A separate, *private-to-scoring* `ChallengeWorld` implements each variant; the E9 discoverer sees only `observe`, `available_actions`, and `send_action`. The 9 variants are: missing action, extra action, silent no-op, multi-pixel side effect, aliased control, second adjacent gate-like marker, irreversible latch, delayed change after eight interactions, and a nonstationary control effect.

We measure: conforming worlds accepted and verified, out-of-class worlds refused, out-of-class **falsely accepted**, actual discovery actions, and post-discovery auditor actions. A claimed discovery is **not** evidence of soundness: for every acceptance, the auditor executes new public `off→on→off` sequences for both learned switches on the *same episode* and checks the full frame for unmodeled side effects. The audit is a bounded behavioural check, not a proof of indefinite stationarity. Out-of-class acceptance counts as a false positive even if the bounded audit happened to pass. The E9 learner is not told its variant label or the hidden world dynamics.

**Known failure mode:** exactly eight calibration interactions do not certify future stationarity. A late semantic change after the eight interactions can be accepted by E9 and falsified only by later interventions. The pilot is intentionally expected to reveal this, rather than asserting perfect OOD rejection. This is not the official ARC-AGI-3 interface, independent external authorship, arbitrary perception, or lifelong learning.

The previous E9 full 100-case result remains a correct in-class result. E10 does not change or retroactively invalidate it, but it narrows the generalization claim. E10 tests the discovery layer only; it is **not** a new end-to-end E6 causal-relearning score. Each supported case uses five external E7 motion tutorials, and tutorial actions are not included in `actual_discovery_actions`.

Run from Athlesia repo root after the pilot script:

```bash
E6=experiments/e6_online_revision_transfer
E7=experiments/e7_raw_grid_transfer
E9=experiments/e9_unlabelled_pixel_affordances
E10=experiments/e10_affordance_falsification
PYTHONPATH="$PWD/$E10:$PWD/$E9:$PWD/$E7:$PWD/$E6" python3 -m unittest discover -s "$E10" -p test_e10.py -v
PYTHONPATH="$PWD/$E10:$PWD/$E9:$PWD/$E7:$PWD/$E6" python3 "$E10/test_e10.py" --cases 110 --output "$E10/results/e10_falsification110.json"
```

The local benchmark JSON is ignored. The pilot does not commit or push.
