# ATHLESIA

<!-- ATHLESIA_RESEARCH_STATUS:START -->
## Research Status

ATHLESIA is an experimental **domain-general bounded cognitive architecture** investigating whether intelligent behaviour can emerge from the construction, testing, revision, compression and reuse of internal models under finite computational resources.

The project deliberately separates **architectural priors** from **domain semantics**. The cognitive core does not encode benchmark-specific object meanings, directions, transition laws or task templates. Exact opaque `CognitiveStructure` identity remains the semantic authority until an evidence-backed learning process justifies abstraction.

### Current verified architecture

| Module | Research role | Tests | Status |
| --- | --- | ---: | --- |
| M23-M44 | Recursive reasoning, revision, abstraction and compositional generalization | frozen contract | FROZEN |
| M45 | Mindstone + Sparse Cognition | 216 | FROZEN |
| M46 | Core Knowledge + Perceptual Grounding | 72 | FROZEN |
| M47 | Universal Domain Learning | 156 | FROZEN |
| M48 | Executive Agency | 108 | FROZEN |
| M49 | Meta-learning, skill invention, compression and memory | 12 | ACTIVE |

### Cognitive architecture

```text
                     FINITE COMPUTE
                          |
            surprise / novelty / EIG
                / learning progress
                          |
                          v
               +--------------------+
               | M45 MINDSTONE      |
               | SPARSE COGNITION   |
               +---------+----------+
                         |
                         v
OBSERVATION ---> +--------------------+
                 | M46 PERCEPTUAL     |
                 | GROUNDING          |
                 +---------+----------+
                           |
                           v
                 +--------------------+
                 | M47 UNIVERSAL      |
                 | DOMAIN LEARNING    |
                 | rules / causality  |
                 +---------+----------+
                           |
                           v
                 +--------------------+
                 | M48 EXECUTIVE      |
                 | AGENCY             |
                 | select / act       |
                 | monitor / replan   |
                 | stop / explore     |
                 +---------+----------+
                           |
                    successful grounded
                    state-action traces
                           |
                           v
                 +--------------------+
                 | M49 META-LEARNING  |
                 | exact skill        |
                 | evidence memory    |
                 +---------+----------+
                           |
                           v
                    repeated evidence
                           |
                           v
                 structural abstraction
                    compression / reuse
```

### Research progression

The currently validated architecture implements the following cognitive progression:

```text
PERCEIVE
   |
   v
construct grounded hypotheses
   |
   v
induce predicates / rules / transitions
   |
   v
validate causal structure
   |
   v
form and maintain goals
   |
   v
select bounded multi-step intention
   |
   v
ACT -> OBSERVE CONSEQUENCE
   |
   +---- prediction agrees ----> continue / exploit
   |
   +---- prediction fails -----> replan / reconsider
   |
   +---- learning value high --> explore
   |
   v
successful grounded trajectory
   |
   v
episodic skill evidence
   |
   v
future skill induction
```

### M45 — Mindstone and Sparse Cognition

M45 introduced resource-aware cognition. Novelty, surprise, expected information gain, learning progress and controllability regulate which information reaches expensive reasoning and how finite compute is allocated.

The architecture therefore does not assume that every observation deserves equal processing.

### M46 — Core Knowledge and Perceptual Grounding

M46 moved the system below symbolic reasoning into hypothesis-based perceptual organization.

Objecthood, persistence, topology, motion/change and action consequences are treated as **candidate explanatory structures**, not fixed environmental laws.

This avoids assumptions such as predefined segmentation rules, privileged directions or task-specific object semantics.

### M47 — Universal Domain Learning

M47 introduced grounded discovery of:

- predicates,
- conjunctive rules,
- invariants,
- transition schemas,
- contextual transition rules,
- cross-context generalization,
- exceptions,
- confidence calibration,
- causal contrasts,
- interventional causal validation,
- cross-domain transfer,
- compressed domain models.

The result is a bounded queryable world model whose local causal evidence remains distinguishable from transferred knowledge.

### M48 — Executive Agency

M48 completed the bounded executive control stack:

```text
goal pressure
    |
    v
action selection
    |
    v
goal persistence
    |
    v
conflict arbitration
    |
    v
multi-step intention
    |
    v
execution monitoring
    |
    +--> deviation -> grounded replanning
    |
    v
stop / reconsideration
    |
    v
exploration vs exploitation
    |
    v
integrated executive control
```

A crucial property is **non-compulsive agency**: ATHLESIA can stop, reconsider, abstain or explore rather than blindly continue an existing plan.

### M49 — Meta-learning and Skill Memory

M49 begins the transition from solving individual situations to **learning reusable cognitive procedures from experience**.

The current foundation follows an evidence-first principle:

```text
successful grounded episode
          |
          v
exact state/action/outcome trace
          |
          v
skill evidence memory
          |
          v
repeated evidence
          |
          v
skill candidate
          |
          v
future abstraction
          |
          v
future compressed reusable skill
```

A single successful trajectory is explicitly **not** considered a learned skill.

Exact traces currently preserve:

- initial state,
- opaque goal identity,
- ordered required states,
- actions,
- observed outcomes,
- success confidence,
- step-confidence evidence.

Repeated exact traces aggregate support while retaining conservative confidence floors.

### Core research invariants

1. **Exact identity before abstraction.** Similarity does not silently become semantic equivalence.
2. **Evidence before skill promotion.** One successful episode is evidence, not a reusable skill.
3. **Conservative uncertainty.** Weak evidence remains visible instead of being averaged away.
4. **Bounded cognition.** Search, planning, evaluation, candidate and memory frontiers are finite.
5. **Learning progress over curiosity alone.** Information gain matters when it can improve the world model.
6. **Revisable agency.** Commitments may be interrupted by deviation, low value or insufficient evidence.
7. **No hidden benchmark solver.** Domain-specific semantics remain outside the cognitive kernel.
8. **Frozen regression contracts.** Validated modules are protected by executable integrity verifiers.

### Active research frontier

Current:

```text
M49 grounded skill evidence memory
```

Next:

```text
repeated exact evidence
        |
        v
skill-candidate discovery
        |
        v
structural abstraction induction
        |
        v
cross-context generalization
        |
        v
skill compression
        |
        v
retrieval and reuse
```

The long-term objective is not a manually authored macro library. It is a system capable of discovering which successful internal structures deserve abstraction and compression into reusable cognitive skills while preserving provenance, uncertainty and bounded execution semantics.
<!-- ATHLESIA_RESEARCH_STATUS:END -->

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
