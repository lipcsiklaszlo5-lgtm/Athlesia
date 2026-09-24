# Research roadmap

The current benchmark provides a reproducible result and a clear next problem: learned role schemas help within a family but harm under role reversal, and the five families are trained independently.

## Complete: freeze the v4 reference

- Five procedural relation families, with one separately trained model per family.
- Sixteen training worlds and 128 held-out worlds per family.
- Disjoint seeds, tile-token ranges, and remapped action IDs.
- Cold, random, one-example, and raw-token memorizer comparisons.
- A role-reversal shift probe and a machine-readable result.
- A visible showcase and one-command local runner.

The final result is in artifacts/open_intelligence/result.json.

## Next: one shared learner

Build a learner that receives examples from all five families without an externally supplied family ID. It must represent uncertainty about which relation applies, choose informative interactions, and revise its choice when consequences disagree.

Acceptance gate:

- preserve the v4 benchmark and its baselines;
- add a new preregistered split with at least 1,000 held-out worlds;
- show at least 20% fewer mean actions than the cold baseline;
- report each family separately and include a held-out family-selection test;
- publish results whether or not the gate passes.

## Then: recover from misleading transfer

Use the role-reversal probe to measure how quickly a model detects and corrects a harmful prior. Predeclare the interaction budget and compare transferred, reset-to-cold, and adaptive models on paired worlds. Do not claim robust adaptation until the transferred model reaches the cold model's performance within that budget.

## Only after the learning loop passes

Port the shared learner into the Rust architecture and compare it against the Python reference on identical frozen worlds. Keep the reference evaluator available so implementation changes cannot silently redefine the task.

## Release discipline

Every public result should include source revision, commands, protocol, machine-readable output, hardware and runtime, and known limitations. Do not replace a failed result with a tuned split.
