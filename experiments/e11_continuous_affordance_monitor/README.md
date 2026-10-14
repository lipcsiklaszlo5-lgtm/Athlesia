# Athlesia E11 — continuously monitored, *provisional* pixel-affordance hypotheses

E10 is preserved unchanged, including **13/90 false acceptances at discovery**: 10 delayed semantic changes and three nonstationary controls. A finite sequence of observations cannot prove that an action will behave identically forever. The new E11 monitor therefore **does not certify future stationarity**; it checks each later control intervention, records a concrete before/after counterexample on mismatch, quarantines the offending model, and refuses all further use after quarantine.

This is an isolated research adapter that imports the **unchanged E9 `DiscoveredBridge`** and uses only the public `observe`/`send_action` port; it never reads the private E10 variant or simulator internals. It is not a patch to E9 or a production safety guarantee. The `provisional` state is retained even after many consistent transitions.

E11 replays E10's exact frozen 110-case sequence, separately reports discovery refusals and *initial* E9 false acceptances, then monitors a fixed `on/off` schedule of up to 12 subsequent real actions on accepted worlds. An additional 20-case challenge delays semantic drift until at least 28 world actions, later than E9 discovery and the first eight post-discovery checks. Future drift cannot be guaranteed detectable until a causally informative operation actually occurs; no claim is made about environments beyond the tested windows.

Metrics include actual discovery actions, subsequent monitored actions (including violations), false-positive acceptances before monitoring, quarantines after observations, and supported cases left provisional. The subsequent testing actions are **extra real costs**, not free validation or counted in the old E10 574 actions. E6 model-learning, open-world object detection, noise/occlusion, autonomous episode resets, and official ARC-AGI-3 scoring are not tested here. The same author wrote the environment and evaluator. No original E6–E10 files are changed, committed or deleted.

From the repository root after running the pilot:

```bash
E6=experiments/e6_online_revision_transfer
E7=experiments/e7_raw_grid_transfer
E9=experiments/e9_unlabelled_pixel_affordances
E10=experiments/e10_affordance_falsification
E11=experiments/e11_continuous_affordance_monitor
export PYTHONPATH="$PWD/$E11:$PWD/$E10:$PWD/$E9:$PWD/$E7:$PWD/$E6"
python3 -m unittest discover -s "$E11" -p test_e11.py -v
python3 "$E11/test_e11.py" --cases 110 --long-cases 20 --output "$E11/results/e11_monitor110.json"
```
