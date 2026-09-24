# Research note: role learning in opaque grid worlds

## Question

Can an agent use interaction outcomes to learn a geometric role schema and reuse it in new worlds where layouts, tile IDs, and action IDs change?

The v4 experiment tests transfer within five known procedural task families. It does not test general intelligence or discovery of an unknown family.

## Protocol

Each world is 13×13, contains twelve decoy entities, and requests eighteen interior obstacles. A hidden key and exit occupy locations determined by one of five geometric relations:

- edge
- left/right
- upper/lower
- checkerboard
- near/far

The learner is given a fixed representation of those five features, but not the active rule ID. It receives a grid of opaque tile IDs, its own position, and outcomes from actions. It is not given object labels, action meanings, goal coordinates, the world seed, or hidden inventory.

There is a separate model for each family. Each model receives 16 training worlds and is frozen before 128 held-out worlds are evaluated. A further 32 role-reversal worlds per family test a known form of distribution shift. In total, the final run contains 80 training worlds, 640 held-out worlds, and 160 shift-probe worlds.

Training, held-out, and shift-probe splits use disjoint seeds and tile-token ranges. Actions are independently remapped per world. No held-out episode updates the model used by another held-out episode.

## Frozen v4 result

| Condition | Worlds solved | Success | Mean actions |
| --- | ---: | ---: | ---: |
| Trained, 16 examples per family | 639/640 | 99.84% | 36.17 |
| One example per family | 625/640 | 97.66% | 45.79 |
| Cold, no retained role evidence | 621/640 | 97.03% | 54.49 |
| Raw-token memorizer | 621/640 | 97.03% | 54.49 |
| Random actions | 35/640 | 5.47% | 125.11 |
| Reversed roles, trained model | 137/160 | 85.62% | 92.31 |
| Reversed roles, cold model | 156/160 | 97.50% | 59.36 |

On the paired held-out worlds, the trained model saved 18.317 actions per world relative to cold (95% stratified paired-bootstrap interval: 16.461–20.056). Its mean action count was 33.62% lower. The one-example model saved 8.697 actions per world (95% interval: 6.362–10.902).

The role-reversal probe is a negative result: transfer adds 32.95 actions on average versus cold, with a 95% paired-bootstrap interval from 27.788 to 38.462 extra actions. It should be treated as a central result, not an exception hidden behind the headline score.

## Interpretation

The evidence supports a narrow claim: within a task family, retained geometric role evidence reduces search effort on new procedurally generated worlds. Near-ceiling success is not the main evidence because the cold model already solves 97.03% of the held-out set. The stronger comparison is paired action cost against that cold model.

This is not a test of raw visual perception. The input is already a grid, self-position is supplied, the geometric feature vocabulary is fixed, and there are five independent models rather than a single family-discovery system. The environment and its task rules are synthetic. The benchmark does not establish performance on ARC-AGI or real-world tasks.

## Next experiments

1. Train one shared learner across all five families without giving it a family ID; test whether it can select or revise a role model from interaction.
2. Measure recovery after role reversal. Compare the transferred model against cold after a fixed number of corrective interactions.
3. Add new relation families that are not among the five declared features. Keep them separate from development seeds.
4. Compare against simple search and hand-coded baselines, report per-family results, and increase the frozen held-out set.
5. Keep the Rust port out of the claim until it matches the reference evaluator on frozen worlds.
