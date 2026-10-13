# Athlesia E8: E7 pixel observations → E6 causal guard learning (pilot)

This is an **integration experiment**, not an ARC-AGI-3 implementation. It imports the *unchanged*, actual E7 `Agent.fit` (learned opaque directional action semantics from five visual demonstrations), and the *unchanged*, actual E6 `Agent.fit`, `observe_mechanism_shift`, and `run_active_guard_probe` (guard hypothesis version-space and active experimental selection).

A new same-author, instrumented **9×9 pixel** test world has two visible switch-indicator pixels. The environment advertises on/off actions for each indicator; the bridge calibrates both pixel colours and locations through actual public actions. The motor does not read the hidden conjunction. The learned E7 directional token drives a gate-crossing action. The bridge converts actual pixel observations into an E6 Boolean scene comprising two grounded switch channels and the visible crossed-gate goal. A historical gate law is learned from four trace episodes, then the environment changes its guard, causing quarantine and autonomous active *selection/execution of public interventions*, one trial per externally provided fresh episode. E6 alone certifies the new law.

Comparisons use 12 subsequent probe attempts per arm, with failed or incorrect identifications kept in the denominator. The baseline randomly sets publicly accessible switch controls and uses the same actual transition/evidence API; it does not read the hidden rule.

**Hard limitations:** The grid is completely visible. Public switch controls, switch count, gate target cell, training demonstrations, first contradictory trial, and episode resets are external. Pixel-to-Boolean transformation is calibrated but domain-specific and explicitly instrumented. The E7 motion grammar still assumes four cardinal moves; E6 guard grammar remains conjunctive ≤2 literals. Both fixture and bridge were authored for this test; no independently designed ontology, object discovery, general lifelong learning, or official ARC-AGI-3 success is claimed. This pilot does not replace the E6/E7 source code.

From the Athlesia repo root (after running `athlesia_e8_pixel_causal_pilot.sh`):

```sh
E8=experiments/e8_pixel_causal_bridge
E7=experiments/e7_raw_grid_transfer
E6=experiments/e6_online_revision_transfer
PYTHONPATH="$PWD/$E8:$PWD/$E7:$PWD/$E6" python3 -m unittest -v test_e8
PYTHONPATH="$PWD/$E8:$PWD/$E7:$PWD/$E6" python3 "$E8/test_e8.py" --cases 100
```

`bridge.py` imports only the public E7 agent and public E6 learner/protocol. `pixel_gate_world.py` is private to the test. Results are generated locally, not committed.
