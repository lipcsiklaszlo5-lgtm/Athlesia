# ATHLESIA

Athlesia is an experimental structural learning and active inference kernel written in Rust.

The project studies whether a compact deterministic system can learn reusable structure from observations without storing the concrete values that produced that structure.

## Core idea

```text
Concrete observation
        |
        v
Structural encoding
        |
        v
Relational structure
        |
        v
Primitive discovery
        |
        v
Hypothesis induction
        |
        v
Structural concepts
        |
        +-------------------+
        |                   |
        v                   v
   Recognition          Prediction
                            |
                            v
                    Experiment generation
                            |
                            v
                    Experiment selection
                            |
                            v
                       Observation
                            |
                            v
                    Prediction evaluation
                       /          \
                      v            v
                Confirmed      Violated
                      \            /
                       v          v
                     State transition

Concrete values are used only at the observation boundary.

For example:

[1, 2, 1, 2, 3]
[10, 20, 10, 20, 30]
[847, 13, 847, 13, 999]

all produce the same structural pattern:

[R0, R1, R0, R1, R2]

with relations:

position 0 == position 2
position 1 == position 3

Persistent concepts retain the structural relationship rather than the original values.

Current proof of concept

The current Rust baseline supports:

deterministic structural encoding
value-independent role representation
relational structure discovery
repeated primitive discovery
compression-based hypothesis induction
structural concept memory
recognition on unseen values
exact structural extent
partial-match rejection
structural prediction
prediction confirmation and violation
experiment generation
structural information-gain scoring
deterministic experiment selection
active state transitions

Current frozen baseline:

162 passing Rust tests
19 / 19 architecture integrity checks
30 frozen implementation files
SHA-256 baseline verification

Verify the repository with:

cargo test
cargo clippy --all-targets --all-features -- -D warnings
python3 tools/verify_state.py
python3 tools/verify_module_23_parity.py
What Athlesia is

Athlesia is currently a research proof of concept for structural abstraction.

It demonstrates that a small deterministic program can:

remove concrete value identity from observations
retain reusable relational structure
recognize learned structure in new observations
derive structural predictions
select informative observations
explicitly preserve prediction failures

The project is intentionally small and inspectable.

What it is not

The current prototype does not yet provide:

general real-world perception
hierarchical concept memory
concept revision
probabilistic beliefs
causal reasoning
long-horizon planning
production-scale learning

These are research directions rather than current capabilities.

Next research stage

The next major architectural problem is revision-capable memory.

The current system can determine whether a prediction was confirmed or violated, but persistent concepts do not yet change in response to accumulated evidence.

The intended next loop is:

observe
   |
   v
form structural models
   |
   v
predict
   |
   v
gather evidence
   |
   v
confirm / contradict
   |
   v
revise model
   |
   v
retain better abstraction

See the docs/ directory for more detail.
