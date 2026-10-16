# Autonomous causal agent — bounded milestone A1

This is not an implementation of general human intuition. It is an opt-in,
goal-directed learning loop built from the repository's existing `State`,
`Literal`, `ActionRule`, `TransitionExample` and exact typed graph matcher.
It does not change the ARC policy, demo, WASM bridge or default runtime.

## What changed

The demo/session previously drove a prescribed learn/transfer/sabotage/probe
sequence. This agent receives a visible graph, an observation, a goal and a
sandbox-approved action catalogue. It chooses its own next action. It has no
environment handle, no state setter, no hidden rule and no slot-order input.

1. Generate a **complete, bounded** hypothesis language for each action.
2. Search consensus transitions for a route to the observed-state goal.
3. If no such route is known, select an actual reachable experiment maximizing
   worst-case hypothesis elimination divided by path+action cost.
4. Bind the next full observation to the pending command, episode and sequence.
5. Retain only models consistent with that action's **entire retained history**.
   Other actions are untouched. Never call a single surviving tested prediction
   a universal causal proof.
6. Keep negative observations in persistent memory. Restore by replaying every
   transition, not by trusting serialized models or claimed success flags.
7. Reuse the existing graph matcher for renamed worlds. Mapped knowledge is a
   **structural trial prior**, not proof of target behavior. If falsified, drop
   that target prior and learn from target evidence; leave source memory intact.

Planning is bounded breadth-first search over model-consensus transitions; it
finds a minimum-step route within the searched graph, not a globally minimum-
cost plan. Experiment ranking includes action cost. This is structured symbolic
search, not a zero-search architecture or an unbounded brute-force solver.

## Exact current scope

- At most 8 complete Boolean state fields and 16 opaque, discrete actions.
- At most 32 visible graph nodes and 96 directed typed edges.
- An action is identity, or one constant Boolean assignment, with 0..2 signed
  guard literals on distinct, non-effect fields. Missing fields are rejected.
- Maximum 2048 candidates per action by default. Truncating this language is an
  error, not an excuse to claim identification over a partially searched set.
- Observation, evidence, event, planning, matching and work budgets are explicit.
- A host-approved sandbox is required for exploration. This does **not** prove
  that an exploratory action is safe on a real robot or payment system.
- Visible kinds/relation types are provided. Some tests also provide stable
  structural type tags. Arbitrary symbol meaning is NOT inferred from nothing.
- Full observation consistency is checked; perceptual grounding from pixels,
  partial observability, stochasticity and delayed effects are not solved here.
- The hypothesis class does not express XOR, arbitrary OR, numeric dynamics,
  multiple simultaneous effects or time-dependent mechanisms.
- A true law change contradicting retained same-state/same-action outcomes is
  reported as model insufficiency. Temporal context discovery and separate
  learned mechanism epochs are the NEXT milestone, not silently implemented by
  deleting history. A changed state exposing a previously hidden guard is
  different from changing the transition law itself.

## Persistence and evidence limits

`export_memory` stores visible layout and complete empirical transitions.
`restore` requires the matching layout and re-filters the entire grammar. It
does not claim cryptographic evidence authenticity or snapshot all transient
execution state, analogy priors, telemetry and event chronology. A caller must
provide the factual provenance of tutorial/trace data. Limits stop learning
explicitly; no evidence is evicted to make a benchmark pass.

## Validation

Run from the repo root:

```bash
bash tools/verify_autonomous_causal_agent.sh 128
```

The named integration target includes cold goal solving, warm restart,
retained negative experience, signed/two-guard mechanisms, transfer, false
analogy, wrong commands, stale/cross-episode observations, partial frames,
XOR/multi-effect out-of-language cases and allocation/work/search limits.

The fuzz family has 64 mechanism classes with different signs/guards and
multiple opaque relabellings. That is **not 128 arbitrary domains**. One transfer
test first identifies the source from its full 48-transition truth table;
that test is explicitly not one-shot learning. The native benchmark instead
uses only source experience collected by the autonomous agent itself.

The benchmark reports actual actions for cold, restored and transferred runs.
It measures wall intervals separately from whole-child CPU and RSS, and pins
the native binary to one allowed CPU. No fixed latency or fabricated memory
values, and no animation delay is included in the claimed kernel timing.

## Existing matcher fix

The old exact matcher could return `Unique` when its DFS budget expired after
finding one match but before excluding a second. The patch makes that outcome
`ResourceLimit`. Two witnesses still prove ambiguity. A dedicated regression
test uses two symmetric nodes and a budget expiring at the first full map.

## Next acceptance gates, before a new web demo

1. Independent author-supplied worlds, not only the in-repo generator; publish
   failures as well as successes.
2. Expand the model language with explicit coverage tests and computational
   budgets (OR/XOR/conditional effects), not hand-coded solutions per level.
3. Temporal context/epoch learning for genuine mechanism changes. Preserve
   both old and new evidence under their contexts, test correct retrieval.
4. Remove stable role tags in held-out cases; handle symmetries by experiments
   or abstention rather than arbitrary matching.
5. Compare matched random, memoryless and non-transfer ablations under the same
   observation/action budgets; do not assume an LLM must fail.
6. Measure the actual target edge hardware and build WASM parity tests before
   publishing edge, robotics, sub-millisecond or low-memory performance claims.
