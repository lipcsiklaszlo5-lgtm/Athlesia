# Bounded successor episode runtime

B3B adds execution infrastructure, not autonomous cognition. The active stack is:

scorecard/session → start game → live runtime → decision attempt
→ M51 successor authority → M48 selection → live command → observation
→ learning feedback → passive trace → next attempt

M48 remains the only final selector. M50 authority, M51 state ownership, source
provenance, transport error semantics, and B3A tracing are unchanged. The original
`ArcAgi3BoundedEpisodeRuntime` is preserved for its existing cognitive-step contract.

## API and budgets

`ArcAgi3SuccessorEpisodeRuntime::run_with(&mut live, policy, execute_attempt)`
accepts a callback returning the existing
`Result<Option<ArcAgi3LiveUnifiedStep>, ArcAgi3LiveEnvironmentError>`.
The callback must perform exactly one existing successor decision attempt. It can
capture a B3A sink and call `execute_successor_informed_unified_with_trace`.
The driver never constructs a request, chooses a goal/action, ranks candidates,
computes confidence, generates beliefs, or synthesizes trace records.

`ArcAgi3SuccessorEpisodePolicy::new(max_decision_attempts,
max_consecutive_abstentions)` returns `None` if either limit is zero. These are
count limits, not elapsed-time limits or cancellation of a running callback.

DECISION ATTEMPT != EXECUTED COGNITIVE STEP

Each callback invocation consumes one attempt. A successful `Some(step)` consumes
one executed step and clears the consecutive-abstention streak. `None` consumes
one abstention and extends the streak. Total abstentions are not cleared. Thus two
abstentions, one execution, then two abstentions are five attempts, one executed
step, four total abstentions, and a final streak of two.

ABSTENTION IS NOT AN ACTION.

An abstention performs no transport action, creates no pending command, and adds
zero completed cognitive steps. There is no fallback, random action, retry, or
RESET to escape it. The driver verifies unchanged completed count and absence of
a pending command. It also rejects an unexpected reset counter change. The
callback remains responsible for obeying the one-successor-attempt contract; the
driver is not a sandbox for arbitrary callback side effects.

For executions, both the live completed count and the returned step count must
equal `starting_completed_cognitive_step_count + executed_steps`. A pending command
on a supposedly completed step is rejected. Counters use checked arithmetic.

## Termination and errors

`Won` and `GameOver` reflect actual live status. An already-terminal runtime returns
with zero attempts. A terminal status after execution stops immediately, including
on the last allowed attempt. Otherwise the run ends as `DecisionBudgetExhausted`
or `ConsecutiveAbstentionBudgetExhausted`, retaining its actual final status.
If the last abstention reaches both limits, consecutive-abstention exhaustion takes
precedence. Neither budget outcome is converted into environment failure or success.

`ArcAgi3SuccessorEpisodeResult` exposes termination, decision attempts, executed
steps, total abstentions, starting/ending completed cognitive counts, and final
status. It retains no step history and uses constant-size bookkeeping.

`ArcAgi3SuccessorEpisodeError` distinguishes unrunnable/faulted/pending runtimes,
failed attempts, counter overflow, completion-count mismatches, invalid abstention
side effects, unexpected resets, and unexpected post-step status. `StepFailed`
retains the original live error, the attempted decision count (including the failed
attempt), and the count of prior successful executions. Errors return immediately;
indeterminate transport dispatch is never automatically retried or reset.

## Competition and JSONL integration

`ArcAgi3CompetitionGame::run_successor_bounded_with(policy, execute_attempt)` is a
thin delegate to the new driver. Continue using `ArcAgi3CompetitionSession::open`,
`start_game`, `game.finish`, and the existing session `close` lifecycle. The driver
adds no scorecard calls, HTTP implementation, persistence, or benchmark connection.

A caller-owned `ArcAgi3JsonlTraceSink` can be captured by the executor closure. Each
successful traced execution emits one `Executed`; each traced abstention emits one
`Abstained`; a transport failure emits one `TransportFailure` before the original
error reaches the driver. The driver emits no additional records. B3A precondition
or cognitive rejections still follow B3A's existing behavior without fabricated
trace events. Streaming does not require retaining episode history.

## Remaining B3C blocker

The legacy caller-native `native_possibilities` and `beliefs`, plus goals and other
policy/context inputs, are still part of the successor request contract. A valid
per-step decision authority provider is required. This driver does not supply one
and does not make unattended ARC execution fully endogenous.

Production callers must not fill this gap with constants, guessed predictions,
synthetic confidence/EIG, random beliefs, or test fixture values. B3C is the separate
work needed for an endogenous, evidence-faithful per-step request path. B3B neither
fixes nor expands that contract.

## Verification

Adapter tests reuse the real C16I successor fixture. They cover exact M51 provenance,
terminal stopping, abstention budgets and streak reset, nonzero counter baselines,
manual-call cognition/transport/JSONL parity, repeated decisions, failure propagation,
invalid callback effects, and existing competition lifecycle integration. Test-only
fixture assembly does not add a production cognition mutation API. Run focused
adapter tests, then `./tools/verify_repo.sh` for all 33 manifests.
