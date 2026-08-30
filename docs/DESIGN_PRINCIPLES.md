
Design Principles
Structure before domain semantics

The kernel should operate on reusable relational structure rather than application-specific rules.

Concrete values stay at the observation boundary

Persistent concepts should not depend on the values from which they were learned.

Transfer matters

A concept becomes useful when it applies to new observations with the same structure.

Negative cases matter

Recognition must reject near-matches as well as accept correct matches.

Prediction makes models testable

A useful structural concept should expose consequences that can be checked.

Contradictions are information

Prediction failure should remain explicit and should eventually influence memory revision.

Observation can be active

The system should prefer observations that provide useful structural information.

Determinism first

The current prototype favours reproducibility over stochastic mechanisms.

Memory growth must eventually be bounded

A system that remembers every observation has not solved abstraction.

Tests are architectural contracts

When an established invariant fails, the failure should be treated as information about the architecture.

Claims follow evidence

Implemented, tested and planned capabilities should remain clearly separated.
