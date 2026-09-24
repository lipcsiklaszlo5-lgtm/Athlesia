# Athlesia Open Intelligence Core v4

A small, reproducible interaction-learning benchmark with a visible demo and a frozen held-out result.

The claim is deliberately narrow: within a task family, can a model use interaction examples to learn a geometric role schema and reduce search effort in new worlds?

## Quick run

From the repository root:

~~~bash
bash tools/run_open_intelligence_showcase.sh
~~~

This prints a short demonstration on its own seed range. It is illustrative, not part of the scored result.

## Full reproduction

~~~bash
bash tools/run_open_intelligence_demo.sh
~~~

The command checks Python syntax, runs the deterministic unit tests, runs the showcase, and evaluates the full benchmark. It writes the [machine-readable result](../../artifacts/open_intelligence/result.json). Python's standard library is the only dependency; no network or model API call is made.

## World and agent

Each world is a 13×13 grid with twelve decoy entities, up to eighteen interior obstacles, a hidden collectible, and a terminal location. Tile IDs and action IDs are opaque and remapped for every world.

The agent receives a grid, its current position, and movement/reward feedback. It is not given the world seed, action meanings, tile semantics, object coordinates, hidden inventory, or rule-family ID. Its fixed priors and representation are declared in the result file:

- two-dimensional locality and persistent self-position;
- four reversible movement effects;
- object persistence and scalar terminal feedback;
- five geometric features: edge, left/right, upper/lower, checkerboard, and near/far.

The agent uses interaction to learn whether collectible and terminal roles are associated with those features. The experiment trains a separate schema for each of the five rule families. It does not test one shared model's ability to identify an unknown family.

## Frozen v4 result

| Condition | Result | Mean actions |
| --- | ---: | ---: |
| Trained, 16 examples per family | 639/640 (99.84%) | 36.17 |
| One example per family | 625/640 (97.66%) | 45.79 |
| Cold, no retained examples | 621/640 (97.03%) | 54.49 |
| Raw-token memorizer | 621/640 (97.03%) | 54.49 |
| Random actions | 35/640 (5.47%) | 125.11 |
| Reversed roles, trained | 137/160 (85.62%) | 92.31 |
| Reversed roles, cold | 156/160 (97.50%) | 59.36 |

The trained model saves 18.317 actions per held-out world over cold (95% stratified paired-bootstrap interval 16.461–20.056), a 33.62% reduction in mean actions. The near-ceiling success rate alone is not the main result; the cold model already solves 97.03%.

Role reversal causes negative transfer. The trained model uses 32.95 more actions per world than cold on that split (95% paired-bootstrap interval: 27.788–38.462 extra actions).

## Evaluation protocol

| Split | Worlds | Use |
| --- | ---: | --- |
| Training | 16 per family; 80 total | Learn role evidence |
| Held-out | 128 per family; 640 total | Frozen same-family transfer |
| Role reversal | 32 per family; 160 total | Measure harmful transfer |
| Cold and random controls | Paired with held-out worlds | Compare retained evidence and unguided search |

The primary metric is mean environment actions per world. Failure is charged at the full 128-action budget. The trained and control conditions share identical world specifications. Seed and token ranges are disjoint between training, held-out, and role-reversal splits. The evaluator performs 2,000 paired bootstrap resamples, stratified by rule family.

Acceptance criteria are recorded in result.json and enforced by the evaluator. The v4 result passed them. Do not tune policy code against the frozen final seeds.

## Limitations

This is a synthetic grid world with a fixed geometric feature vocabulary and supplied self-position. Each family has its own model. The environment does not provide raw images or natural language. The result does not establish general intelligence, open-world perception, arbitrary relation discovery, robust adaptation, Rust integration, or ARC-AGI performance.
