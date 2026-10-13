# Athlesia E7 — Opaque Grid Visual Grounding Spike

This is a **separate, bounded research prototype**, not a refactor of E6 and
not an official ARC-AGI-3 agent or score.

## Question

Can a small CPU-only learner infer opaque action translations and a moving
sprite from pixel-level before/after observations, ground one blocked-tile
colour by failed action, and use them on held-out maps with shuffled action IDs,
colours and walls? How does its complete-map planner compare to a local greedy
controller and a uniform-random controller given **the same 5 tutorials**?

## Observation and prior assumptions

- 9x9 opaque integer-colour frames (not RGB photos, not an object graph).
- Four 1-cell cardinal actions are guaranteed by the model class, but their
  opaque IDs are re-permuted in each case and learned from four movements.
- One failed movement grounds an obstacle colour, with the original outcome
  verified; 5 demonstrations are **supplied by the evaluator** per case.
- The evaluator declares the target coordinate explicitly. Goal discovery is
  **not** being tested; neither is exploration, online action grammar discovery,
  perceptual occlusion, partial observation, stochastic dynamics, or reset.
- At planning time the full maze is visible. The planner uses shortest-path
  search on learned floor/obstacle colours; this structural prior is strong.
- A separately randomized training room shares each case's palette and action
  bindings with its held-out maze. Training/test are not cross-domain.

The solver (`agent.py`) never imports the environment. The scorer verifies the
actual environment state, not a solver success claim. All failed attempts stay
in the denominator. The evaluator is same-author, in-process and intentionally
small. There is no claim of independent third-party validation or AGI.

## Run

```bash
cd experiments/e7_raw_grid_transfer
python3 -m unittest -v test_e7
python3 test_e7.py --cases 100 --budget 60 --output results/e7_raw_grid100.json
```

Generated JSON results are not committed. No GPU or paid API required.
