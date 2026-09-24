# Athlesia

Athlesia is an open research repository for learning compact, reusable structure from interaction.

## Current public result

The Open Intelligence experiment tests one narrow claim: can an agent use a small set of interaction examples to learn which geometric roles matter in a task, then solve new worlds with new layouts, opaque tile IDs, and remapped actions?

The frozen v4 run covers five task families. Each family has its own trained role model; the agent is not asked to identify an unknown family.

| Condition | Success | Mean actions |
| --- | ---: | ---: |
| 16-example trained model | 639/640 (99.84%) | 36.17 |
| No retained training evidence | 621/640 (97.03%) | 54.49 |
| Random actions | 35/640 (5.47%) | 125.11 |
| One-example model | 625/640 (97.66%) | 45.79 |

On the paired held-out worlds, the trained model saved 18.317 actions per world over the cold model (95% stratified paired-bootstrap interval: 16.461–20.056). This is a 33.62% reduction in mean actions. The raw-token memorizer performed exactly like the cold model.

The result also shows a real limit: when the key and exit roles are reversed, the transferred model averages 92.31 actions and solves 137/160 worlds. The cold model averages 59.36 actions and solves 156/160. The learned rule hurts under this shift.

## Run it

For a short, visible demonstration:

~~~bash
bash tools/run_open_intelligence_showcase.sh
~~~

To run syntax checks, the test suite, the showcase, and the full held-out benchmark:

~~~bash
bash tools/run_open_intelligence_demo.sh
~~~

The full command writes the [machine-readable result](artifacts/open_intelligence/result.json). The benchmark uses Python's standard library and makes no external API calls. The previously measured benchmark portion took about 175 seconds; machine speed changes the runtime.

## What the experiment establishes

- A model trained on 16 worlds in one task family can transfer its learned role schema to 128 held-out worlds from that same family.
- There are five separately trained families: edge, left/right, upper/lower, checkerboard, and near/far.
- Training, held-out, and role-reversal worlds use disjoint seed and tile-token ranges. Action IDs are remapped in every world.
- The agent receives a tile grid, its own position, and interaction feedback. It does not receive the seed, hidden inventory, object labels, goal positions, action meanings, or task-family rule ID.
- The model uses a declared representation containing five geometric features. Those features are prior knowledge, not discovered from raw pixels.

## What it does not establish

This is a small synthetic grid-world experiment. It does not show that one model can discover an unknown family, learn arbitrary relations, perceive raw images, handle realistic environments, or solve ARC-AGI. The Python experiment is not integrated with the Rust research code. The role-reversal result shows negative transfer rather than robust out-of-distribution adaptation.

Those are the next research questions, not claims about the current system.

## Repository map

- [Benchmark, tests, and visible showcase](experiments/open_intelligence_core_demo/README.md)
- [Frozen v4 metrics and protocol](artifacts/open_intelligence/result.json)
- [Funding application draft](docs/OPEN_INTELLIGENCE_APPLICATION.md)
- [Architecture and research boundaries](docs/ARCHITECTURE.md)
- [Research question, evidence, and limitations](docs/RESEARCH.md)
- [Measurable next steps](docs/ROADMAP.md)

## Scope

The benchmark is the current public evidence for the Open Intelligence proposal. The Rust crates remain a separate research codebase; no claim is made that the benchmark learner is already a Rust runtime capability.
