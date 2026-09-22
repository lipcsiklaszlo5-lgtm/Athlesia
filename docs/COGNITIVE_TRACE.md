# Passive cognitive flight recorder

TRACE IS NOT COGNITIVE AUTHORITY.

The active execution path remains:

observe → M51 cognition → candidate authority → M48 selection → live command
→ environment observation → cognitive completion/evidence update → passive trace

M48 remains the sole final selector. M50 experiment authority, M51 retained-state
ownership, production evidence/learning updates, and error propagation are unchanged.
The recorder does not repair native beliefs, rank candidates, infer sources, or
participate in cognition. No legacy research code or benchmark harness is involved.

## Opt-in live API

Call `ArcAgi3LiveEnvironmentRuntime::execute_successor_informed_unified_with_trace`
with the existing request and a mutable `ArcAgi3CognitiveTraceSink`. It invokes the
existing `execute_successor_informed_unified` exactly once and observes its result.
`reset_with_trace` similarly wraps the existing reset operation. Existing methods
remain the default path, without trace allocations or a required logger. Callers
must opt in; existing episode runners are not automatically connected to a sink.

The sink receives only a detached data record after the operation has resolved;
it receives no mutable cognition, authority, candidate set, or transport handle.
Its return value is discarded. Sink I/O failures and unwinding panics do not replace
the execution result. As with any synchronous callback, callers must avoid blocking
or process-aborting sinks; process aborts and allocation exhaustion are not recoverable.

## Records and exact provenance

`ArcAgi3CognitiveTraceEvent` distinguishes `Executed`, `Abstained`, `Reset`, and
`TransportFailure`. Completed cognitive counts are the existing runtime counts
(one-based after the first successful cognitive step), not attempted-step indices.
Abstentions, failures, and resets do not advance that cognitive count. Resets also
record the completed reset count; successful outcomes include the session event index.

An executed record copies the selected ARC action, exact M51 source-state and
cognitive-action structures from the returned authority, command action, before/after
observation state, optional observed action echo, completion presence, and cognitive
feedback presence. Structure serialization preserves every atom, ordered/unordered
variant, and child order; it neither hashes nor reconstructs identity from ARC actions.
The command API has only an action identity, so no separate command ID is invented.

Observation snapshots contain game ID, game-state enum, all frame cells and dimensions
(as rows), level counters, and available actions. `structurally_changed` compares
these snapshots exactly, excluding the action echo. It is an observation comparison,
not a causal or world-model judgment. `NotFinished`, `Win`, `GameOver`, and `NotPlayed`
remain distinct; there is no guessed terminal outcome.

Abstention records contain only the pre-execution observation and unchanged completed
count. They have no selected action, outcome, or confidence. No transport call or
cognitive mutation is added. Reset outcomes come from the returned completion.
Transport failures retain the exact error's Debug rendering (including disposition
when provided), pending-command presence/action, and absence of a returned completion.
They do not claim the remote environment did not act. An indeterminate dispatch
remains indeterminate. Failed calls do not expose a returned authority or outcome,
so these are not reconstructed. Non-transport precondition/cognitive errors propagate
unchanged without being mislabeled as abstentions or transport failures.

The authority exposes no explicit exploitation/experimentation tag; none is inferred
from information gain or ARC actions. Probability, confidence, EIG, learning progress,
controllability, causal certainty, and predictions are not recorded in this version.
No unavailable value is filled with a diagnostic guess.

## Streaming JSONL

`event.write_jsonl(&mut writer)` writes one deterministic JSON object plus newline.
`ArcAgi3JsonlTraceSink::new(writer)` adapts any caller-owned `std::io::Write`; its
`last_error` retains the most recent write failure for caller inspection. The caller
owns flushing, destination, and lifetime. An I/O failure can leave a partial line;
there is no retry that might duplicate a record. Serde deserialization supports
round trips into the diagnostic types. Records contain no timestamps or random IDs.

Serialization stays in the adapter. There is no filesystem persistence, hardcoded
path, global state, network logging, or episode-wide buffer. Only the current record
and observation snapshots are needed in memory. These diagnostic values must never
be supplied as new cognitive authority.

## Verification

Adapter regressions reuse the real C16I successor-informed M50 fixture. They compare
whole returned steps and the complete `Eq` cognitive runtime (session, perception,
and retained cognition), exact M51 provenance, subsequent selection, and transport
counts. They also cover abstention, reset, transport failure, deterministic JSONL,
and failing/panicking sinks. Run adapter tests first, then `./tools/verify_repo.sh`.
