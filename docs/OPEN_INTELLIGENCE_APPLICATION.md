# Open Intelligence funding application draft

Prepared 24 September 2026. Replace the two contact placeholders before submission. The technical claims below refer to the [frozen v4 result](../artifacts/open_intelligence/result.json).

## Form fields

**Your name**

László Lipcsik

**Email**

[YOUR EMAIL]

**Twitter/X**

[YOUR X HANDLE, OR LEAVE EMPTY]

**GitHub profile**

https://github.com/lipcsiklaszlo5-lgtm

**Project**

Athlesia: Low-Cost Role Learning from Interaction

**Funding request**

$10,000

**Project description**

Athlesia is an open research project testing whether a small agent can learn which geometric roles matter in a task from interaction, then reuse that evidence in new worlds. In a frozen CPU benchmark, five task families each have their own model. After 16 training worlds per family, the models solve 639 of 640 held-out worlds and average 36.17 actions, compared with 54.49 for a cold model with no retained role evidence. The paired reduction is 18.317 actions per world (95% interval 16.461–20.056). The worlds use new layouts, opaque tile IDs, and remapped actions; the policy receives no hidden goal or family label. The feature vocabulary is explicitly fixed to five geometric relations, and the benchmark is a synthetic grid-world, not a general-intelligence or ARC result. A role-reversal test exposes harmful transfer: the trained model is slower than cold. The funded milestone will target this gap by building and evaluating one shared learner that selects and revises role models from interaction.

**Primary use**

Research and development. If the form offers only a broader category, choose the closest technical research option.

## What the current evidence shows

| Evaluation | Result |
| --- | ---: |
| Trained, 16 examples per family | 639/640 held-out worlds; 36.17 mean actions |
| Cold, no retained role evidence | 621/640; 54.49 mean actions |
| Random actions | 35/640; 125.11 mean actions |
| Paired trained-versus-cold reduction | 18.317 actions (95% interval 16.461–20.056) |
| Reversed roles, trained | 137/160; 92.31 mean actions |
| Reversed roles, cold | 156/160; 59.36 mean actions |

The v4 run uses 80 training worlds, 640 held-out worlds, and 160 role-reversal worlds. Each of five families has a separately trained model; a single learner selecting among unknown families has not yet been demonstrated.

## Six-week research milestone

| Period | Work | Public evidence |
| --- | --- | --- |
| Week 1 | Tag and reproduce the v4 reference on a clean runner | Frozen result, protocol, checksums, and automated reproduction |
| Weeks 2–3 | Build one shared learner that receives examples from all five families without family labels | Source, interaction traces, and a new development benchmark |
| Weeks 4–5 | Test model selection and recovery under role reversal on a new frozen split | Paired results against cold, reset, and adaptive baselines |
| Week 6 | Publish the implementation, benchmark, failures, and a concise technical report | One-command reproduction and reviewer-ready release |

The Rust port begins only if the shared learner passes its evaluation gate; the Python evaluator remains the reference. If the gate fails, the milestone publishes the failure and the evidence needed to revise the research direction.

## Proposed use of funds

| Use | Amount | Output |
| --- | ---: | --- |
| Researcher time | $7,000 | Shared learner, experiments, and public weekly progress |
| CPU evaluation and storage | $2,000 | Larger frozen splits, ablations, and preserved artifacts |
| Documentation and release | $1,000 | Reproduction guide, technical report, and reviewer package |
| **Total** | **$10,000** | **Open-source, reproducible research milestone** |

This is a proposed allocation, not a claim of funds already spent. No paid model API or GPU training is required by the current benchmark.

## Predeclared evaluation commitments

- Keep the v4 result unchanged as the reference.
- Use new development and final seed ranges for the shared learner.
- Keep the family ID and hidden state out of the policy interface.
- Compare against paired cold, reset, and random baselines.
- Report sample efficiency, action cost, success, and every family separately.
- Publish negative and distribution-shift results.
- Do not describe the system as general intelligence or as an ARC-AGI solver.

## Reproduce and inspect

From a repository checkout:

~~~bash
bash tools/run_open_intelligence_showcase.sh
bash tools/run_open_intelligence_demo.sh
~~~

The first command runs the illustrative visible showcase. The second runs syntax checks, tests, the showcase, and the full benchmark. It writes the [machine-readable result](../artifacts/open_intelligence/result.json). The benchmark uses only the Python standard library and makes zero external API calls.

## Evidence links

- Repository: https://github.com/lipcsiklaszlo5-lgtm/Athlesia
- [Benchmark guide](../experiments/open_intelligence_core_demo/README.md)
- [Full runner](../tools/run_open_intelligence_demo.sh)
- [Frozen result](../artifacts/open_intelligence/result.json)

After publication, replace repository-relative file names with permanent links to the tagged revision.
