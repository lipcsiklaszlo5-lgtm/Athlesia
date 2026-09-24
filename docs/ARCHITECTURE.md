# Architecture

Athlesia currently contains two separate research tracks. The Open Intelligence benchmark is a small Python experiment. The Rust crates explore a broader cognitive architecture. The benchmark learner is not connected to the Rust runtime.

## Python interaction benchmark

The experiment is in experiments/open_intelligence_core_demo/core_demo.py. It uses a deterministic 13×13 grid-world with opaque tile IDs, twelve decoy entities, and up to eighteen interior obstacles.

~~~mermaid
flowchart TD
    A[Grid and self position] --> B[Calibrate four movement actions]
    B --> C[Find reachable unknown entities]
    C --> D[Choose a collectible or terminal target]
    D --> E[Act and observe movement or reward]
    E --> F[Update role evidence]
    F --> C
~~~

The agent uses four declared priors: two-dimensional locality, persistent self-position, four reversible movement effects, and object persistence with scalar terminal feedback. It also receives a fixed five-feature geometric representation: edge, left/right, upper/lower, checkerboard, and near/far.

During a training episode, the agent discovers which opaque action moves in each direction, identifies a collectible when an entity disappears after contact, and identifies a terminal entity from reward feedback. It stores the geometric features of those positions in a role schema. A frozen copy of the schema is used for held-out episodes.

The benchmark has five rule families. Each family gets a separately trained schema and its own held-out set. The policy does not receive the family ID, but the experiment does not test whether one shared model can recognize an unknown family or keep all five models separate by itself.

## Evaluation harness

The evaluator builds training, held-out, and role-reversal worlds from fixed disjoint seed ranges. Every world remaps action IDs and uses a token namespace that does not occur in another split. Paired conditions receive identical world specifications.

The primary comparison is the trained schema against a cold model with the same fixed priors and no retained training evidence. Random actions are a reference baseline. A raw-token memorizer is included as a leakage control; its held-out result matches the cold model because tile IDs are disjoint.

The held-out set contains 128 worlds per family, 640 total. The role-reversal probe contains 32 worlds per family, 160 total. Failed episodes count as the full 128-action budget in the primary mean.

## Separate Rust research line

The repository also contains a broader Rust research codebase. The Python
benchmark does not call those crates, and this release makes no Rust runtime
performance claim. Keep the benchmark evaluator as the reference if a Rust
implementation is tested later.

## Intended next connection

A future integration should have a single observable interface for experience, learned state, action selection, and outcomes. It should preserve the Python benchmark as a reference evaluator and compare the Rust implementation on the same frozen seeds. Integration begins only after the shared learner can handle family selection and the role-reversal failure under predeclared criteria.
