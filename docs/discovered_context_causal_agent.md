# A4 — Observable context discovery

A3 accepted an opaque `ContextId` at an episode boundary. A4 removes that
host-side label in a deliberately bounded way. The solver receives only
complete Boolean states and action outcomes. If the current single-regime
hypothesis class becomes empty, it enumerates observable state-field
projections of size one or two, partitions the retained transitions, and
replays every partition through the existing `AutonomousCausalAgent`.

The selected projection must be the unique minimum under:

1. number of context fields;
2. number of resulting partitions; and
3. retained candidate-space size after replay.

Ties are reported as `ContextAmbiguous`; a hidden separator that is absent
from the observation is reported as `ContextUnavailable`. No semantic meaning
is assigned to token IDs, and no context is inferred from a name or an oracle
label.

## API boundary

`DiscoveredContextCausalAgent::begin_episode(initial, goal)` and `decide()` are
the live interaction path. A returned `DiscoveryActionProposal` contains the
opaque action and the discovered signature used for audit. The host executes
the action and calls `observe(proposal, next_observation)`.

`ingest_transition(goal, TransitionExample)` is a replay path for an existing
event log and deterministic holdout tests. It does not issue an action or
grant execution authority.

The module preserves the A3 boundary: old evidence remains in the bounded
transition ledger, every candidate is replayed through the same grammar, and
an ambiguous or unobservable split never becomes a model.

## Reproducible check

```bash
bash tools/verify_discovered_context_agent.sh 128
```

The benchmark uses fresh opaque token IDs on every case. The mechanism is
`target := X && Y` in one observable mode and `target := !X && !Y` in the
other. A global grammar with at most two guards cannot represent both branches;
partitioning by the mode field can. The benchmark is a bounded Boolean fixture,
not a claim of general human intuition, robotics safety, or AGI.
